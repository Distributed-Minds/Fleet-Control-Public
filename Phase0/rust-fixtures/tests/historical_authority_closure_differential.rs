//! Optional historical Python/Rust semantic differential for issue #71.
//!
//! The Python reference remains archival evidence. Maintained Rust consumers do
//! not depend on Python: this integration test is explicitly ignored by default.
//! Run during migration with:
//! cargo test --locked --offline --test historical_authority_closure_differential -- --ignored

use serde_json::{Map, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);
const BASELINE: &str = include_str!("../../fixtures/authority-closure-spec2.json");
const EXPECTED_KEYS: &[&str] = &[
    "expected",
    "expected_closure",
    "expected_logical_cancellations",
];

fn semantic_inputs(case: &Map<String, Value>) -> Map<String, Value> {
    case.iter()
        .filter(|(key, _)| key.as_str() != "name" && !EXPECTED_KEYS.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn historical_python_disposition(case: &Map<String, Value>) -> Map<String, Value> {
    // Import the original preserved model without rewriting the Python archive.
    const EVALUATE: &str = concat!(
        "import json,sys\n",
        "sys.path.insert(0,sys.argv[1])\n",
        "from authority_closure_model import evaluate_authority_closure\n",
        "value=json.load(sys.stdin)\n",
        "json.dump(evaluate_authority_closure(value),sys.stdout,sort_keys=True)\n"
    );
    let scripts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 parent")
        .join("scripts");
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(EVALUATE)
        .arg(&scripts)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("historical Python interpreter required for explicit opt-in differential");
    child
        .stdin
        .take()
        .expect("Python standard input")
        .write_all(
            serde_json::to_string(&semantic_inputs(case))
                .expect("serialize semantic facts")
                .as_bytes(),
        )
        .expect("write Python semantic input");
    let output = child
        .wait_with_output()
        .expect("run historical Python model");
    assert!(
        output.status.success(),
        "historical Python model failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Map<String, Value>>(&output.stdout)
        .expect("historical model emitted an object")
}

fn original_expectation(case: &Map<String, Value>) -> Map<String, Value> {
    let expected: Map<_, _> = case
        .iter()
        .filter(|(key, _)| EXPECTED_KEYS.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    assert_eq!(
        expected.len(),
        1,
        "fixture must have one expected disposition"
    );
    expected
}

fn rust_cli(fixture: &Value) -> Output {
    let counter = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-authority-crosslang-{}-{counter}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .expect("exclusive temporary fixture");
    file.write_all(
        serde_json::to_string(fixture)
            .expect("serialize full fixture")
            .as_bytes(),
    )
    .expect("write temporary fixture");
    drop(file);
    let result = Command::new(env!("CARGO_BIN_EXE_authority_closure"))
        .arg(&path)
        .output();
    fs::remove_file(&path).expect("remove temporary fixture");
    result.expect("run compiled Rust validator")
}

fn assert_rust(fixture: &Value, should_pass: bool, label: &str) {
    let output = rust_cli(fixture);
    assert_eq!(
        output.status.success(),
        should_pass,
        "Rust {label}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !should_pass {
        assert!(
            output.stdout.is_empty(),
            "Rust {label} emitted a positive record after rejection"
        );
    }
}

fn case_mut<'a>(fixture: &'a mut Value, name: &str) -> &'a mut Map<String, Value> {
    fixture["cases"]
        .as_array_mut()
        .expect("fixture cases array")
        .iter_mut()
        .find(|case| case["name"] == name)
        .expect("named historical case")
        .as_object_mut()
        .expect("named case object")
}

#[test]
#[ignore = "opt-in historical Python 3 reference; no Python required by Rust consumers"]
fn python_and_rust_agree_on_all_cases_and_changed_authority_facts() {
    let baseline: Value = serde_json::from_str(BASELINE).expect("checked-in historical JSON");
    let cases = baseline["cases"].as_array().expect("historical case array");
    assert_eq!(cases.len(), 26);

    // Independently compute every reference disposition from semantic facts,
    // not from the checked-in expected labels or the Rust implementation.
    for case in cases {
        let object = case.as_object().expect("case object");
        assert_eq!(
            historical_python_disposition(object),
            original_expectation(object),
            "Python reference disagrees with fixture case {}",
            object["name"]
        );
    }
    assert_rust(&baseline, true, "unchanged 26-case baseline");

    // Each alteration changes the correct answer. A Rust evaluator that simply
    // echoes fixture labels cannot pass both the rejected old answer and the
    // independently recomputed Python answer for the same semantic facts.
    for (name, field, new_value) in [
        (
            "cycle-without-external-root",
            "external_root",
            Value::Bool(true),
        ),
        (
            "provider-valid-revoked-credential-denied-with-debt",
            "external_invalidation_complete",
            Value::Bool(true),
        ),
        (
            "multi-root-all-required-loses-one-root",
            "surviving_roots",
            Value::from(2_u64),
        ),
    ] {
        let mut changed = baseline.clone();
        let case = case_mut(&mut changed, name);
        let prior = original_expectation(case);
        assert!(case.insert(field.to_owned(), new_value).is_some());
        let independent_answer = historical_python_disposition(case);
        assert_ne!(independent_answer, prior, "mutation did not change {name}");
        assert_rust(&changed, false, name);

        let case = case_mut(&mut changed, name);
        for key in EXPECTED_KEYS {
            case.remove(*key);
        }
        case.extend(independent_answer);
        assert_rust(&changed, true, name);
    }
}
