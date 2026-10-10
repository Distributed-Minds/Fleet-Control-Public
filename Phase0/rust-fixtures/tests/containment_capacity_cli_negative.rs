//! Process-level acceptance for the real containment-capacity Rust oracle.
//! A historical fixture verifier only: these checks grant no live scheduling,
//! resource admission, restoration, GitHub, or policy mutation authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/containment-capacity-spec2.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("parse historical capacity fixture")
}

fn run(contents: &str) -> Output {
    let sequence = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path: PathBuf = std::env::temp_dir().join(format!(
        "free-energy-capacity-cli-{}-{sequence}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("reserve unique temporary fixture");
    file.write_all(contents.as_bytes())
        .expect("write complete fixture bytes");
    drop(file);

    let output = Command::new(env!("CARGO_BIN_EXE_containment_capacity"))
        .arg(&path)
        .output()
        .expect("execute compiled containment-capacity binary");
    fs::remove_file(path).expect("remove temporary fixture");
    output
}

fn run_mutated(root: &Value) -> Output {
    run(&serde_json::to_string(root).expect("serialize mutated fixture"))
}

fn rejected(output: Output, required_diagnostic: &str) {
    assert!(
        !output.status.success(),
        "invalid fixture returned process success: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "failure printed misleading success: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("FAIL: ") && stderr.contains(required_diagnostic),
        "missing {required_diagnostic:?} error in {stderr:?}"
    );
}

#[test]
fn real_binary_accepts_original_seventeen_case_fixture() {
    let output = run(BASELINE);
    assert!(output.status.success(), "baseline failed: {output:?}");
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "containment-capacity fixtures (Rust): 17 passed"
    );
}

#[test]
fn process_rejects_forged_decision_and_changed_restoration_outcomes() {
    let mut decision = baseline();
    decision["decision_cases"][0]["renewal_evidence_current"] = json!(true);
    rejected(run_mutated(&decision), "backlog-cannot-renew:");

    let mut starved = baseline();
    starved["workload_cases"][3]["fail_safe_escalates"] = json!(true);
    rejected(run_mutated(&starved), "priority-starvation-detected:");

    let mut restored = baseline();
    restored["workload_cases"][8]["restoration_capacity"][0] = json!(0);
    rejected(run_mutated(&restored), "separate-restoration-capacity:");
}

#[test]
fn process_rejects_changed_capacity_and_alert_volume() {
    let mut load = baseline();
    load["workload_cases"][0]["service_capacity"][0] = json!(0);
    rejected(run_mutated(&load), "ordinary-load:");

    let mut alerts = baseline();
    alerts["planning_cases"][0]["false_positive_num"] = json!(0);
    rejected(run_mutated(&alerts), "rare-event-false-positive-heavy:");
}

#[test]
fn process_rejects_identity_collisions_and_invalid_schema_or_capacity() {
    let mut duplicate = baseline();
    duplicate["workload_cases"][0]["id"] = duplicate["decision_cases"][0]["id"].clone();
    rejected(run_mutated(&duplicate), "duplicate or empty case id");

    let mut wrong_spec = baseline();
    wrong_spec["spec_version"] = json!(3);
    rejected(
        run_mutated(&wrong_spec),
        "unsupported containment-capacity schema/spec",
    );

    let mut negative_capacity = baseline();
    negative_capacity["workload_cases"][0]["service_capacity"][0] = json!(-1);
    rejected(run_mutated(&negative_capacity), "negative service capacity");

    let mut unknown_scheduler = baseline();
    unknown_scheduler["workload_cases"][0]["scheduler"] = json!("ignore_limits");
    rejected(run_mutated(&unknown_scheduler), "invalid fixture");
}

#[test]
fn process_accepts_valid_inactive_capacity_traces_without_changing_verdicts() {
    // A supplied, well-formed trace must remain admissible even when the
    // selected capacity mode (or a terminal state) does not consume it.
    for (case_index, field, valid) in [
        (0, "adjudication_capacity", json!([4, 4, 4, 4])),
        (0, "restoration_capacity", json!([8, 8, 8, 8])),
        (7, "restoration_capacity", json!([2, 2])),
        (8, "service_capacity", json!([200, 200])),
    ] {
        let mut fixture = baseline();
        fixture["workload_cases"][case_index][field] = valid;
        let output = run_mutated(&fixture);
        assert!(
            output.status.success(),
            "{field} rejected despite valid inactive evidence: {output:?}"
        );
        assert!(
            output.stderr.is_empty(),
            "{field}: unexpected stderr: {output:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "containment-capacity fixtures (Rust): 17 passed",
            "{field}: historical verdicts changed"
        );
    }
}

#[test]
fn process_rejects_malformed_inactive_capacity_even_on_restored_fast_path() {
    // The baseline supplies no inactive trace for these cases. Before #241's
    // preflight, every changed input below could reach a PASS from the real
    // compiled CLI despite contradictory or truncated capacity evidence.
    for (case_index, field, malformed) in [
        (0, "adjudication_capacity", json!([-1, 0, 0, 0])),
        (0, "restoration_capacity", json!([1])),
        (7, "adjudication_capacity", json!(-1)),
        (7, "restoration_capacity", json!([1])),
        (8, "service_capacity", json!(-1)),
        (8, "service_capacity", json!([1])),
    ] {
        let mut fixture = baseline();
        fixture["workload_cases"][case_index][field] = malformed;
        rejected(run_mutated(&fixture), field);
    }
}
