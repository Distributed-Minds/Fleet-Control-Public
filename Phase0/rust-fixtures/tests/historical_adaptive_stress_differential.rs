//! #71: independently exercise the computed Rust adaptive-stress outcomes
//! against the archived Python fixture checker. The latter is an integrity
//! baseline, not a complete semantic oracle. In particular, its historical
//! assertions miss duplicate case names and extra object keys.
//!
//! Normal Rust-only CI executes the metamorphic test. The Python comparison
//! is explicitly opt-in and never required by consumers of the Rust binary.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/adaptive-stress-spec2.json");
const RUST_BINARY: &str = env!("CARGO_BIN_EXE_adaptive_stress");
const PYTHON_WRAPPER: &str = r#"
import importlib.util
import pathlib
import sys
spec = importlib.util.spec_from_file_location("archived_adaptive_stress", sys.argv[1])
model = importlib.util.module_from_spec(spec)
spec.loader.exec_module(model)
model.FIXTURE_PATH = pathlib.Path(sys.argv[2])
model.main()
"#;
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

struct Input {
    path: PathBuf,
}

impl Input {
    fn new(value: &Value) -> Self {
        let number = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-adaptive-invariant-diff-{}-{number}.json",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create exclusive fixture input");
        file.write_all(
            serde_json::to_string(value)
                .expect("serializable fixture")
                .as_bytes(),
        )
        .expect("write fixture before executing either oracle");
        drop(file);
        Self { path }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn rust(path: &Path) -> Output {
    Command::new(RUST_BINARY)
        .arg(path)
        .output()
        .expect("execute compiled Rust adaptive-stress checker")
}

fn python(path: &Path) -> Output {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 directory")
        .join("scripts/check-adaptive-stress-fixtures.py");
    Command::new("python3")
        .arg("-c")
        .arg(PYTHON_WRAPPER)
        .arg(script)
        .arg(path)
        .output()
        .expect("opt-in Python 3 archival checker")
}

fn fixture() -> Value {
    serde_json::from_str(BASELINE).expect("tracked 23-case baseline")
}

fn case_mut<'a>(root: &'a mut Value, name: &str) -> &'a mut Value {
    root["cases"]
        .as_array_mut()
        .expect("cases array")
        .iter_mut()
        .find(|case| case["name"] == name)
        .expect("named baseline case")
}

fn check_rust(value: &Value, allowed: bool, label: &str) {
    let input = Input::new(value);
    let result = rust(&input.path);
    assert_eq!(
        result.status.success(),
        allowed,
        "{label}: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    if !allowed {
        assert!(
            result.stdout.is_empty(),
            "{label}: false positive success output"
        );
    }
}

#[test]
fn computed_outcomes_follow_changed_facts_not_historical_expected_labels() {
    let original = fixture();
    check_rust(&original, true, "unchanged complete family");

    // Each original expected verdict becomes wrong when its causal facts move.
    // After updating the verdict to the independently specified consequence,
    // the SAME Rust executable must admit the entire 23-case family.
    let changes: &[(&str, &str, Value, &str, Value)] = &[
        (
            "evaluation-budget-exhaustion",
            "evaluation_budget_remaining",
            json!(2),
            "expected",
            json!("ELIGIBLE"),
        ),
        (
            "family-budget-exhaustion",
            "family_budget_remaining",
            json!(1),
            "expected",
            json!("ELIGIBLE"),
        ),
        (
            "ack-loss-reconciles-first",
            "durable_result_exists",
            json!(false),
            "expected",
            json!("REVALIDATE"),
        ),
        (
            "patch-accepted-is-closure",
            "independent_acceptance",
            json!(false),
            "expected_closed",
            json!(false),
        ),
        (
            "correlated-evaluators-not-independent",
            "shared_lineage",
            json!(false),
            "expected_independence_gain",
            json!(true),
        ),
        (
            "anti-overfit-requires-siblings",
            "siblings_pass",
            json!(true),
            "expected_accept",
            json!(true),
        ),
    ];

    for (name, fact, new_fact, verdict, new_verdict) in changes {
        let mut mutated = original.clone();
        case_mut(&mut mutated, name)[*fact] = new_fact.clone();
        check_rust(&mutated, false, name);
        case_mut(&mut mutated, name)[*verdict] = new_verdict.clone();
        if *name == "evaluation-budget-exhaustion" || *name == "family-budget-exhaustion" {
            case_mut(&mut mutated, name)["fresh_workload"] = json!(true);
        }
        if *name == "ack-loss-reconciles-first" {
            case_mut(&mut mutated, name)["rerun"] = json!(true);
        }
        check_rust(&mutated, true, name);
    }
}

#[test]
#[ignore = "opt-in historical Python comparison; no Python in normal Rust tooling"]
fn archived_python_and_rust_agree_on_shared_guards_but_not_all_shape_semantics() {
    let original = fixture();
    let input = Input::new(&original);
    assert!(rust(&input.path).status.success(), "Rust baseline");
    assert!(python(&input.path).status.success(), "Python baseline");

    // These material historical safety guards reject identically.
    for (name, field, value) in [
        (
            "fixture-authority-is-inert",
            "authority_change",
            json!(true),
        ),
        (
            "material-target-change-can-split",
            "bounded_allowance",
            json!(0),
        ),
        ("missing-telemetry-is-unknown", "numeric_default", json!(0)),
        ("ack-loss-reconciles-first", "rerun", json!(true)),
    ] {
        let mut changed = original.clone();
        case_mut(&mut changed, name)[field] = value;
        let input = Input::new(&changed);
        let rust_result = rust(&input.path);
        let python_result = python(&input.path);
        assert!(
            !rust_result.status.success(),
            "Rust accepted {name}/{field}"
        );
        assert!(
            !python_result.status.success(),
            "Python accepted {name}/{field}"
        );
        assert!(rust_result.stdout.is_empty(), "Rust false success: {name}");
    }

    // The original Python fixture checker has narrower shape validation.
    // Preserve this divergence as explicit migration debt, not false parity.
    let mut duplicate = original.clone();
    let first_case = duplicate["cases"][0].clone();
    duplicate["cases"].as_array_mut().unwrap().push(first_case);
    let input = Input::new(&duplicate);
    assert!(
        !rust(&input.path).status.success(),
        "Rust must reject duplicate"
    );
    assert!(
        python(&input.path).status.success(),
        "historical Python silently deduplicates"
    );

    let mut unknown = original.clone();
    case_mut(&mut unknown, "retry-reuses-evaluation-id")["untrusted_extra"] = json!(true);
    let input = Input::new(&unknown);
    assert!(
        !rust(&input.path).status.success(),
        "Rust must reject unknown fields"
    );
    assert!(
        python(&input.path).status.success(),
        "historical Python permits extra fields"
    );

    let mut missing_causal_evidence = original;
    case_mut(
        &mut missing_causal_evidence,
        "evaluator-version-churn-same-family",
    )["evaluator_changed"] = json!(false);
    let input = Input::new(&missing_causal_evidence);
    assert!(
        !rust(&input.path).status.success(),
        "Rust rejects unjustified version churn"
    );
    assert!(
        python(&input.path).status.success(),
        "archived Python did not evaluate this causal fact"
    );
}
