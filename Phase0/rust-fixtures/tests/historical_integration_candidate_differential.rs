//! Independent CLI acceptance of the Rust integration-candidate oracle.
//!
//! The historical Python script is retained as a diagnostic baseline, not a
//! maintained build dependency or an authority source. Its known false passes
//! are asserted explicitly rather than misreported as semantic parity.
//! The ordinary test runs the compiled Rust binary only; the Python comparison
//! is an opt-in, ignored migration differential.
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};

const HISTORICAL: &str = include_str!("../../fixtures/integration-candidate-v1.json");

struct IsolatedFixture {
    root: PathBuf,
    fixture: PathBuf,
    python: PathBuf,
}

impl IsolatedFixture {
    fn new(value: &Value) -> Self {
        let root = (0..64)
            .find_map(|n| {
                let candidate = std::env::temp_dir().join(format!(
                    "free-energy-integration-differential-{}-{n}",
                    process::id()
                ));
                match fs::create_dir(&candidate) {
                    Ok(()) => Some(candidate),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => None,
                    Err(e) => panic!("cannot create isolated test directory: {e}"),
                }
            })
            .expect("no free isolated fixture directory");

        let scripts = root.join("scripts");
        let fixtures = root.join("Phase0/fixtures");
        fs::create_dir_all(&scripts).expect("create isolated scripts directory");
        fs::create_dir_all(&fixtures).expect("create isolated fixture directory");
        let python = scripts.join("check-integration-candidate-fixtures.py");
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Phase0")
            .parent()
            .expect("repository root");
        fs::copy(
            repository_root.join("scripts/check-integration-candidate-fixtures.py"),
            &python,
        )
        .expect("copy unchanged historical oracle for isolated fixture input");
        let fixture = fixtures.join("integration-candidate-v1.json");
        fs::write(
            &fixture,
            serde_json::to_vec(value).expect("serialize fixture mutation"),
        )
        .expect("write isolated fixture");
        Self {
            root,
            fixture,
            python,
        }
    }

    fn rust(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_integration_candidate"))
            .arg(&self.fixture)
            .output()
            .expect("execute compiled Rust candidate oracle")
    }

    fn historical_python(&self) -> Output {
        Command::new("python3")
            .arg(&self.python)
            .output()
            .expect("execute preserved historical Python candidate checker")
    }
}

impl Drop for IsolatedFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn historical() -> Value {
    serde_json::from_str(HISTORICAL).expect("parse pinned original fixture")
}

fn check(label: &str, result: &Output, expected: bool, success_marker: &str) {
    assert_eq!(
        result.status.success(),
        expected,
        "{label}: expected success={expected}; stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    if expected {
        assert!(
            String::from_utf8_lossy(&result.stdout).contains(success_marker),
            "{label}: missing actual checker success receipt"
        );
    } else {
        assert!(
            !String::from_utf8_lossy(&result.stdout).contains(success_marker),
            "{label}: checker reported success after rejecting a mutation"
        );
    }
}

#[test]
fn rust_binary_enforces_original_envelope_and_negative_controls() {
    let original = historical();
    check(
        "rust baseline",
        &IsolatedFixture::new(&original).rust(),
        true,
        "6 passed",
    );

    // Parent order, constructor cardinality and stale-head rejection are
    // evaluated by the Rust oracle, not by this test's expected labels.
    let mut reversed_baseline = original.clone();
    let parents = reversed_baseline["cases"][0]["candidate"]["parents"]
        .as_array_mut()
        .expect("ordered parents");
    parents.reverse();
    check(
        "rust altered normal parent order",
        &IsolatedFixture::new(&reversed_baseline).rust(),
        false,
        "6 passed",
    );

    let mut wrong_verdict = original.clone();
    wrong_verdict["cases"][2]["expect"] = json!("SUPPORTED");
    check(
        "rust unsupported cardinality mislabeled supported",
        &IsolatedFixture::new(&wrong_verdict).rust(),
        false,
        "6 passed",
    );

    let mut stale_becomes_current = original.clone();
    stale_becomes_current["stale_head_cases"][0]["live_target"] =
        stale_becomes_current["stale_head_cases"][0]["predicted_target"].clone();
    check(
        "rust stale head now current",
        &IsolatedFixture::new(&stale_becomes_current).rust(),
        false,
        "6 passed",
    );

    let mut empty_constructor = original.clone();
    empty_constructor["cases"][0]["constructor_support"] = json!([]);
    check(
        "rust absent constructor support",
        &IsolatedFixture::new(&empty_constructor).rust(),
        false,
        "6 passed",
    );

    let mut forged_normal_expectation = original;
    forged_normal_expectation["cases"][0]["expect"] = json!("UNSUPPORTED_PARENT_CARDINALITY");
    check(
        "rust forged normal expectation",
        &IsolatedFixture::new(&forged_normal_expectation).rust(),
        false,
        "6 passed",
    );
}

#[test]
#[ignore = "opt-in archival differential requires historical Python 3; not a Rust runtime dependency"]
fn historical_python_differential_preserves_known_gaps() {
    let original = historical();
    let fixture = IsolatedFixture::new(&original);
    check(
        "Python baseline",
        &fixture.historical_python(),
        true,
        "fixtures: OK",
    );
    check("Rust baseline", &fixture.rust(), true, "6 passed");

    // These are genuine common rejection paths.
    let mut wrong_three_parent_verdict = original.clone();
    wrong_three_parent_verdict["cases"][2]["expect"] = json!("SUPPORTED");
    let fixture = IsolatedFixture::new(&wrong_three_parent_verdict);
    check(
        "Python altered 3-parent verdict",
        &fixture.historical_python(),
        false,
        "fixtures: OK",
    );
    check(
        "Rust altered 3-parent verdict",
        &fixture.rust(),
        false,
        "6 passed",
    );

    let mut stale_becomes_current = original.clone();
    stale_becomes_current["stale_head_cases"][0]["live_target"] =
        stale_becomes_current["stale_head_cases"][0]["predicted_target"].clone();
    let fixture = IsolatedFixture::new(&stale_becomes_current);
    check(
        "Python stale target revalidated",
        &fixture.historical_python(),
        false,
        "fixtures: OK",
    );
    check(
        "Rust stale target revalidated",
        &fixture.rust(),
        false,
        "6 passed",
    );

    // The historical checker never examines the normal expected verdict or
    // constructor-support vector. Rust must reject both: semantic delta,
    // deliberately NOT a claim of full Python/Rust acceptance parity.
    let mut forged_expected = original.clone();
    forged_expected["cases"][0]["expect"] = json!("UNSUPPORTED_PARENT_CARDINALITY");
    let fixture = IsolatedFixture::new(&forged_expected);
    check(
        "Python known false pass: expected",
        &fixture.historical_python(),
        true,
        "fixtures: OK",
    );
    check(
        "Rust rejects forged expected",
        &fixture.rust(),
        false,
        "6 passed",
    );

    let mut empty_support = original;
    empty_support["cases"][0]["constructor_support"] = json!([]);
    let fixture = IsolatedFixture::new(&empty_support);
    check(
        "Python known false pass: support",
        &fixture.historical_python(),
        true,
        "fixtures: OK",
    );
    check(
        "Rust rejects empty support",
        &fixture.rust(),
        false,
        "6 passed",
    );
}
