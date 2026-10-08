//! Exact-fixture-byte differential against the preserved historical Python oracle.
//! This opt-in test is used in CI during migration; normal Rust-only tests
//! and release binaries do not need Python.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};

const BASELINE: &str = include_str!("../../fixtures/containment-spec3.json");

fn checked(label: &str, fixture: &Value, should_pass: bool) {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let python = manifest
        .parent()
        .expect("Phase0 parent")
        .join("scripts/check-containment-fixtures.py");
    let rust = env!("CARGO_BIN_EXE_free-energy-phase0-fixtures");
    let path = std::env::temp_dir().join(format!(
        "free-energy-containment-differential-{}-{label}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("unique temporary fixture");
    file.write_all(
        serde_json::to_string(fixture)
            .expect("serialize fixture")
            .as_bytes(),
    )
    .expect("write fixture bytes");
    drop(file);

    let python_result = Command::new("python3").arg(&python).arg(&path).output();
    let rust_result = Command::new(rust).arg(&path).output();
    fs::remove_file(&path).expect("clean up temporary fixture");
    let python_result = python_result.expect("run historical Python checker");
    let rust_result = rust_result.expect("run compiled Rust checker");

    assert_outcome("Python", label, &python_result, should_pass);
    assert_outcome("Rust", label, &rust_result, should_pass);
}

fn assert_outcome(oracle: &str, label: &str, result: &Output, should_pass: bool) {
    assert_eq!(
        result.status.success(),
        should_pass,
        "{oracle} {label}: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    if should_pass {
        assert!(
            String::from_utf8_lossy(&result.stdout).contains("35 passed"),
            "{oracle} {label}: missing baseline 35-case evidence"
        );
    } else {
        assert!(
            !String::from_utf8_lossy(&result.stdout).contains("35 passed"),
            "{oracle} {label}: rejected fixture printed a success line"
        );
    }
}

#[test]
#[ignore = "requires historical Python 3 checker; explicitly enabled in candidate CI"]
fn historical_python_and_rust_containment_agree_on_negative_controls() {
    let baseline: Value = serde_json::from_str(BASELINE).expect("historical fixture");
    checked("baseline", &baseline, true);

    let mut changed = baseline.clone();
    changed["decision_cases"][0]["authority_current"] = json!(false);
    checked("authority-revoked", &changed, false);

    let mut changed = baseline.clone();
    changed["recovery_cases"][0]["independent_recovery_evidence"] = json!(false);
    checked("recovery-proof-removed", &changed, false);

    let mut changed = baseline.clone();
    changed["trace_cases"][3]["boundary_basis"] =
        changed["trace_cases"][3]["decision_basis"].clone();
    checked("dependency-mismatch-removed", &changed, false);

    let mut changed = baseline;
    changed["decision_cases"][0]["narrowest_effective_level"] = Value::Null;
    checked("explicit-null-narrowest", &changed, false);
}
