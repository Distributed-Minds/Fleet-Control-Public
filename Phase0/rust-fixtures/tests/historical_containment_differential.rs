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

/// Historical adaptive-stress's Python oracle reads a hard-coded fixture
/// path. Import its unchanged historical code and override that path in the
/// test process so both implementations see exactly the same mutated bytes.
/// This is a bounded regression differential, not all-family semantic parity.
#[test]
#[ignore = "requires historical Python 3 checker; explicitly enabled in candidate CI"]
fn historical_python_and_rust_adaptive_stress_agree_on_negative_controls() {
    const ADAPTIVE: &str = include_str!("../../fixtures/adaptive-stress-spec2.json");

    fn checked_adaptive(label: &str, fixture: &Value, should_pass: bool) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let legacy = manifest
            .parent()
            .expect("Phase0 parent")
            .join("scripts/check-adaptive-stress-fixtures.py");
        let path = std::env::temp_dir().join(format!(
            "free-energy-adaptive-differential-{}-{label}.json",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create unique adaptive differential fixture");
        file.write_all(
            serde_json::to_string(fixture)
                .expect("serialize adaptive fixture")
                .as_bytes(),
        )
        .expect("write adaptive fixture");
        drop(file);

        // runpy does not invoke the __main__ guard, so the original checker
        // remains untouched; replace only the in-process fixture path.
        let python_entry = concat!(
            "import pathlib, runpy, sys\n",
            "ns = runpy.run_path(sys.argv[1], run_name='historical_import')\n",
            "ns['main'].__globals__['FIXTURE_PATH'] = pathlib.Path(sys.argv[2])\n",
            "ns['main']()\n"
        );
        let python_result = Command::new("python3")
            .arg("-c")
            .arg(python_entry)
            .arg(&legacy)
            .arg(&path)
            .output()
            .expect("run preserved historical adaptive-stress oracle");
        let rust_result = Command::new(env!("CARGO_BIN_EXE_adaptive_stress"))
            .arg(&path)
            .output()
            .expect("run compiled adaptive-stress oracle");
        fs::remove_file(&path).expect("remove adaptive differential fixture");

        for (oracle, result) in [("Python", python_result), ("Rust", rust_result)] {
            assert_eq!(
                result.status.success(),
                should_pass,
                "{oracle} {label}: stdout={} stderr={}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            if should_pass {
                assert!(
                    String::from_utf8_lossy(&result.stdout).contains("23 passed"),
                    "{oracle} {label}: missing 23-case success evidence"
                );
            } else {
                assert!(
                    result.stdout.is_empty(),
                    "{oracle} {label}: rejected fixture emitted success output"
                );
                assert!(
                    !result.stderr.is_empty(),
                    "{oracle} {label}: rejected fixture lacked diagnostics"
                );
            }
        }
    }

    fn change(base: &Value, name: &str, field: &str, value: Value) -> Value {
        let mut mutated = base.clone();
        let case = mutated["cases"]
            .as_array_mut()
            .expect("historical cases array")
            .iter_mut()
            .find(|case| case["name"] == name)
            .expect("historical named scenario");
        case[field] = value;
        mutated
    }

    let baseline: Value = serde_json::from_str(ADAPTIVE).expect("historical adaptive fixture");
    checked_adaptive("baseline", &baseline, true);

    for (label, case, field, replacement) in [
        (
            "fixture-authority",
            "fixture-authority-is-inert",
            "authority_change",
            json!(true),
        ),
        (
            "retry-double-charge",
            "retry-reuses-evaluation-id",
            "extra_workload_charge",
            json!(true),
        ),
        (
            "ack-loss-rerun",
            "ack-loss-reconciles-first",
            "rerun",
            json!(true),
        ),
        (
            "unbounded-reset",
            "material-target-change-can-split",
            "bounded_allowance",
            json!(0),
        ),
        (
            "unknown-becomes-zero",
            "missing-telemetry-is-unknown",
            "numeric_default",
            json!(0),
        ),
        (
            "budget-exhaustion-misreported",
            "evaluation-budget-exhaustion",
            "expected",
            json!("ELIGIBLE"),
        ),
        (
            "false-remediation-closure",
            "observed-failure-not-closure",
            "expected_closed",
            json!(true),
        ),
    ] {
        checked_adaptive(label, &change(&baseline, case, field, replacement), false);
    }
}

#[test]
#[ignore = "requires historical Python 3 checker; explicitly enabled in candidate CI"]
fn historical_python_and_rust_authority_closure_agree_on_negative_controls() {
    const AUTHORITY: &str = include_str!("../../fixtures/authority-closure-spec2.json");

    fn checked_authority(label: &str, fixture: &Value, should_pass: bool) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let legacy = root
            .parent()
            .expect("Phase0 parent")
            .join("scripts/check-authority-closure-fixtures.py");
        let path = std::env::temp_dir().join(format!(
            "free-energy-authority-differential-{}-{label}.json",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create unique authority differential fixture");
        file.write_all(
            serde_json::to_string(fixture)
                .expect("serialize authority fixture")
                .as_bytes(),
        )
        .expect("write authority fixture");
        drop(file);

        // Preserve the legacy checker unchanged while pointing its global
        // fixture path to the identical bytes evaluated by the Rust binary.
        let python_entry = concat!(
            "import pathlib, runpy, sys\n",
            "sys.path.insert(0, str(pathlib.Path(sys.argv[1]).parent))\n",
            "ns = runpy.run_path(sys.argv[1], run_name='historical_import')\n",
            "ns['main'].__globals__['FIXTURE_PATH'] = pathlib.Path(sys.argv[2])\n",
            "ns['main']()\n"
        );
        let python_result = Command::new("python3")
            .arg("-c")
            .arg(python_entry)
            .arg(&legacy)
            .arg(&path)
            .output()
            .expect("run preserved Python authority checker");
        let rust_result = Command::new(env!("CARGO_BIN_EXE_authority_closure"))
            .arg(&path)
            .output()
            .expect("run compiled Rust authority checker");
        fs::remove_file(&path).expect("remove authority differential fixture");

        for (oracle, result) in [("Python", python_result), ("Rust", rust_result)] {
            assert_eq!(
                result.status.success(),
                should_pass,
                "{oracle} {label}: stdout={} stderr={}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            if should_pass {
                assert!(
                    String::from_utf8_lossy(&result.stdout).contains("26 cases"),
                    "{oracle} {label}: missing full authority baseline evidence"
                );
            } else {
                assert!(
                    result.stdout.is_empty(),
                    "{oracle} {label}: rejected fixture emitted success output"
                );
                assert!(
                    !result.stderr.is_empty(),
                    "{oracle} {label}: rejected fixture lacked diagnostics"
                );
            }
        }
    }

    fn change(base: &Value, name: &str, field: &str, replacement: Value) -> Value {
        let mut fixture = base.clone();
        let case = fixture["cases"]
            .as_array_mut()
            .expect("historical authority cases")
            .iter_mut()
            .find(|case| case["name"] == name)
            .expect("historical named authority scenario");
        case[field] = replacement;
        fixture
    }

    let baseline: Value = serde_json::from_str(AUTHORITY).expect("historical authority fixture");
    checked_authority("baseline", &baseline, true);

    // A changed expected label cannot be accepted as a computed decision.
    checked_authority(
        "forged-verdict",
        &change(
            &baseline,
            "child-after-cutoff-denied",
            "expected",
            json!("BOUNDED_AUTHORITY"),
        ),
        false,
    );

    // Missing one historical case must fail full-suite coverage.
    let mut missing = baseline.clone();
    missing["cases"].as_array_mut().expect("cases array").pop();
    checked_authority("missing-case", &missing, false);

    // These are independent semantic input mutations, not changed labels.
    for (label, name, field, replacement) in [
        (
            "cutoff-revoked",
            "child-after-cutoff-denied",
            "ancestor_cutoff",
            json!(false),
        ),
        (
            "root-restored",
            "multi-root-all-required-loses-one-root",
            "surviving_roots",
            json!(2),
        ),
        (
            "provider-debt-settled",
            "provider-valid-revoked-credential-denied-with-debt",
            "external_invalidation_complete",
            json!(true),
        ),
        (
            "cycle-root-restored",
            "cycle-without-external-root",
            "external_root",
            json!(true),
        ),
    ] {
        checked_authority(label, &change(&baseline, name, field, replacement), false);
    }
}
