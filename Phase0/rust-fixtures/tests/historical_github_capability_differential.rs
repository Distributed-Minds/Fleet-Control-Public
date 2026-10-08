//! Opt-in real-process differential for the historical GitHub capability fixtures.
//!
//! The Python script is preserved research/migration evidence, not a maintained
//! runtime dependency. This test deliberately distinguishes shared integrity
//! checks from safety invariants that the Rust checker now enforces but the
//! historical Python checker did not. It does NOT certify real GitHub capability.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/github-capability-spec5.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug)]
enum Expected {
    BothAccept,
    BothReject,
    LegacyAcceptRustReject,
}

fn input_file(fixture: &Value) -> PathBuf {
    let sequence = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-ghcap-differential-{}-{sequence}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create exclusive test input");
    file.write_all(
        serde_json::to_string(fixture)
            .expect("serialize fixture")
            .as_bytes(),
    )
    .expect("write fixture");
    path
}

fn check(label: &str, fixture: &Value, expected: Expected) {
    let path = input_file(fixture);
    let python = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 directory")
        .join("scripts/check-github-capability-fixtures.py");
    let python_output = Command::new("python3")
        .arg(python)
        .arg(&path)
        .output()
        .expect("execute historical Python script");
    let rust_output = Command::new(env!("CARGO_BIN_EXE_github_capability"))
        .arg(&path)
        .output()
        .expect("execute compiled Rust binary");
    fs::remove_file(&path).expect("remove differential fixture");

    let (python_pass, rust_pass) = match expected {
        Expected::BothAccept => (true, true),
        Expected::BothReject => (false, false),
        Expected::LegacyAcceptRustReject => (true, false),
    };
    for (name, actual, should_pass, success_marker) in [
        (
            "historical Python",
            &python_output,
            python_pass,
            "28 GitHub capability acceptance fixtures",
        ),
        (
            "compiled Rust",
            &rust_output,
            rust_pass,
            "GitHub capability invariant fixtures (Rust): 28 checked",
        ),
    ] {
        assert_eq!(
            actual.status.success(),
            should_pass,
            "{label}: {name} unexpected exit; stdout={} stderr={}",
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&actual.stderr)
        );
        let stdout = String::from_utf8_lossy(&actual.stdout);
        assert_eq!(
            stdout.contains(success_marker),
            should_pass,
            "{label}: {name} wrong success-record disposition: {stdout}"
        );
        if !should_pass {
            assert!(
                !actual.stderr.is_empty(),
                "{label}: {name} silently rejected a fixture"
            );
        }
    }
}

fn source() -> Value {
    serde_json::from_str(BASELINE).expect("canonical issue #11 spec-5 fixture")
}

fn mutate_case(mut fixture: Value, id: u64, property: &str, value: Value) -> Value {
    let case = fixture["cases"]
        .as_array_mut()
        .expect("case array")
        .iter_mut()
        .find(|case| case["id"] == json!(id))
        .expect("historical case id");
    case[property] = value;
    fixture
}

#[test]
#[ignore = "requires preserved historical Python 3 checker; opt in explicitly during CI migration"]
fn python_and_rust_agree_on_integrity_but_expose_historical_safety_gaps() {
    check("original-28", &source(), Expected::BothAccept);

    // Independently pinned identity and expected-result metadata must not
    // self-certify fabricated fixture rows in either implementation.
    check(
        "tampered-name",
        &mutate_case(source(), 1, "name", json!("relabelled-forged-probe")),
        Expected::BothReject,
    );
    check(
        "tampered-result",
        &mutate_case(source(), 16, "expected", json!("ACTION_BLOCKED")),
        Expected::BothReject,
    );
    let mut missing = source();
    missing["cases"].as_array_mut().expect("cases").pop();
    check("missing-case", &missing, Expected::BothReject);
    let mut duplicate = source();
    duplicate["cases"][1]["id"] = json!(1);
    check("duplicate-id", &duplicate, Expected::BothReject);
    check(
        "recovery-transfer-withdrawn",
        &mutate_case(source(), 21, "recovery", json!(false)),
        Expected::BothReject,
    );

    // Historical checker verifies labels and a few invariant implications,
    // but did not enforce the positive proof requirements below. Rust must
    // reject these while Python continues to accept them; a passing test
    // records a *divergence*, not a claim of semantic equivalence.
    check(
        "allow-without-fence",
        &mutate_case(source(), 16, "fenced", json!(false)),
        Expected::LegacyAcceptRustReject,
    );
    check(
        "successor-without-lineage-evidence",
        &mutate_case(source(), 24, "successor_evidence", json!(false)),
        Expected::LegacyAcceptRustReject,
    );
    check(
        "cleanup-without-exact-receipt",
        &mutate_case(source(), 14, "resource", json!("none")),
        Expected::LegacyAcceptRustReject,
    );
}
