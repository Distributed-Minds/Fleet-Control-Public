//! #71: compare the *existing historical* Python containment fixture checker
//! against the compiled Rust oracle for bounded, semantically varied fixtures.
//! No maintained Python implementation is introduced. Python here is an old
//! baseline executable, not a product runtime or a source of mutation authority.
//!
//! Equal success on these controls is NOT full historical parity: unexercised
//! input combinations, malformed JSON admission, and provider effects remain
//! separate acceptance work.

use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Output};

const HISTORICAL: &str = include_str!("../../fixtures/containment-spec3.json");

fn baseline() -> Value {
    serde_json::from_str(HISTORICAL).expect("tracked historical fixture must be JSON")
}

fn diagnostic(output: &Output) -> String {
    format!(
        "exit={:?}, stdout={:?}, stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn compare_with_historical(label: &str, value: &Value, expected_success: bool) {
    let phase0 = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory has a Phase0 parent");
    let historical = phase0.join("scripts/check-containment-fixtures.py");
    assert!(historical.is_file(), "missing tracked historical baseline");
    let path = std::env::temp_dir().join(format!(
        "free-energy-containment-differential-{}-{label}.json",
        std::process::id()
    ));
    std::fs::write(
        &path,
        serde_json::to_vec(value).expect("serializable fixture"),
    )
    .expect("write independent, bounded fixture");

    let python = Command::new("python3")
        .arg(&historical)
        .arg(&path)
        .output()
        .expect("historical Python 3 baseline must run");
    let rust = Command::new(env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"))
        .arg(&path)
        .output()
        .expect("compiled Rust fixture oracle must run");
    std::fs::remove_file(&path).expect("remove scratch fixture");

    assert_eq!(
        python.status.success(),
        expected_success,
        "{label}: historical baseline mismatch: {}",
        diagnostic(&python)
    );
    assert_eq!(
        rust.status.success(),
        expected_success,
        "{label}: compiled Rust mismatch: {}",
        diagnostic(&rust)
    );
    assert_eq!(
        rust.status.success(),
        python.status.success(),
        "{label}: Python/Rust exit divergence"
    );
}

#[test]
fn baseline_and_decision_precedence_have_historical_parity() {
    compare_with_historical("baseline", &baseline(), true);

    let states = [
        ("stale-subject", "subject_current", "IDENTITY_STALE"),
        ("stale-evidence", "evidence_current", "EVIDENCE_STALE"),
        (
            "incompatible-detector",
            "detector_compatible",
            "DETECTOR_INCOMPATIBLE",
        ),
    ];
    for (name, changed_key, disposition) in states {
        let mut fixture = baseline();
        for case in fixture["decision_cases"]
            .as_array_mut()
            .expect("decision family")
        {
            case["subject_current"] = json!(true);
            case["evidence_current"] = json!(true);
            case["detector_compatible"] = json!(true);
            case[changed_key] = json!(false);
            case["expected"] = json!({"disposition": disposition, "level": null});
        }
        compare_with_historical(name, &fixture, true);

        let cases = fixture["decision_cases"]
            .as_array_mut()
            .expect("decision family");
        cases[0]["expected"] = json!({"disposition": "AUTHORIZED", "level": "ISOLATE"});
        compare_with_historical(&format!("{name}-forged-acceptance"), &fixture, false);
    }
}

#[test]
fn recovery_ordering_and_error_controls_have_historical_parity() {
    let variants = [
        (
            "recovery-subject-stale",
            "subject_current",
            "IDENTITY_STALE",
        ),
        (
            "recovery-authority-missing",
            "recovery_authority_current",
            "AUTHORITY_MISSING",
        ),
    ];
    for (name, key, expected) in variants {
        let mut fixture = baseline();
        for case in fixture["recovery_cases"]
            .as_array_mut()
            .expect("recovery family")
        {
            case["subject_current"] = json!(true);
            case["dependency_state"] = json!("CURRENT");
            case["recovery_authority_current"] = json!(true);
            case[key] = json!(false);
            case["expected"] = json!(expected);
        }
        compare_with_historical(name, &fixture, true);
        fixture["recovery_cases"][0]["expected"] = json!("RECOVERED");
        compare_with_historical(&format!("{name}-forged-recovery"), &fixture, false);
    }

    let mut dependency = baseline();
    for case in dependency["recovery_cases"]
        .as_array_mut()
        .expect("recovery family")
    {
        case["subject_current"] = json!(true);
        case["dependency_state"] = json!("STALE");
        case["expected"] = json!("DEPENDENCY_STALE");
    }
    compare_with_historical("recovery-dependency-stale", &dependency, true);
}

#[test]
fn trace_effects_and_causal_equivalence_have_historical_parity() {
    let mut fixture = baseline();
    let mut changed = 0;
    for case in fixture["trace_cases"].as_array_mut().expect("trace family") {
        match case["kind"].as_str().expect("trace kind") {
            "closure" => {
                case["closure_evidence_current"] = json!(false);
                case["expected"] = json!("CLOSURE_DEBT");
                changed += 1;
            }
            "external_effect" => {
                case["external_authority_current"] = json!(false);
                case["expected"] = json!("EFFECT_BLOCKED");
                changed += 1;
            }
            "residual_harm" => {
                case["durable_secondary_effect"] = json!(false);
                case["expected"] = json!("CLEAR");
                changed += 1;
            }
            "dependency_boundary" => {
                case["decision_basis"] = json!("matching-basis");
                case["boundary_basis"] = json!("matching-basis");
                case["expected"] = json!("BOUNDARY_ALLOWED");
                changed += 1;
            }
            _ => {}
        }
    }
    assert!(changed >= 4, "fixture must cover all four trace classes");
    compare_with_historical("trace-causal-equivalence", &fixture, true);

    // Changing the expectation alone must fail in BOTH implementations, not
    // become an accidental success due to counting 13 nominal trace records.
    let first = fixture["trace_cases"]
        .as_array()
        .expect("trace family")
        .iter()
        .position(|case| case["kind"] == "external_effect")
        .expect("external-effect trace exists");
    fixture["trace_cases"][first]["expected"] = json!("EFFECT_ALLOWED");
    compare_with_historical("trace-forged-effect-permission", &fixture, false);
}
