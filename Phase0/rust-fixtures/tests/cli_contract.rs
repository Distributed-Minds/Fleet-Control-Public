//! Process-level checks for the actual Phase0 fixture CLI.
//! These exercise exit codes and diagnostics, not only validate() unit paths.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/containment-spec3.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_free-energy-phase0-fixtures")
}

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("historical containment fixture must parse")
}

fn unique_path() -> PathBuf {
    let sequence = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "free-energy-phase0-fixtures-cli-{}-{sequence}.json",
        process::id()
    ))
}

fn run_contents(contents: &str) -> Output {
    let path = unique_path();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create an isolated input fixture");
    file.write_all(contents.as_bytes())
        .expect("write isolated fixture");
    drop(file);

    let result = Command::new(binary())
        .arg(&path)
        .output()
        .expect("spawn the compiled fixture CLI");
    fs::remove_file(&path).expect("remove isolated fixture");
    result
}

fn run_json(fixture: &Value) -> Output {
    run_contents(&serde_json::to_string(fixture).expect("serialize controlled fixture"))
}

fn assert_rejected(output: &Output, diagnostic: &str) {
    assert!(
        !output.status.success(),
        "unexpected CLI success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(diagnostic),
        "expected diagnostic {diagnostic:?}, got: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "failed fixture should not print a PASS line"
    );
}

#[test]
fn compiled_cli_accepts_the_full_historical_fixture() {
    let output = run_contents(BASELINE);
    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "containment fixtures (Rust): 35 passed"
    );
}

#[test]
fn compiled_cli_rejects_malformed_json_and_invalid_types() {
    assert_rejected(&run_contents("{\"schema_version\":"), "invalid fixture");

    let mut changed = baseline();
    changed["decision_cases"][0]["authority_current"] = json!("false");
    assert_rejected(&run_json(&changed), "invalid fixture");

    let mut changed = baseline();
    changed["trace_cases"][0]["kind"] = json!("invented_trace_kind");
    assert_rejected(&run_json(&changed), "invalid fixture");
}

#[test]
fn compiled_cli_rejects_semantic_mutations_in_all_three_case_families() {
    let mut changed = baseline();
    changed["decision_cases"][0]["authority_current"] = json!(false);
    assert_rejected(&run_json(&changed), "AUTHORITY_MISSING");

    let mut changed = baseline();
    changed["recovery_cases"][0]["independent_recovery_evidence"] = json!(false);
    assert_rejected(&run_json(&changed), "false-positive-recovery");

    let mut changed = baseline();
    changed["trace_cases"][3]["boundary_basis"] = json!("side-effect:v6:cfg-a");
    assert_rejected(&run_json(&changed), "effect-dependency-moved");
}

#[test]
fn compiled_cli_rejects_forged_expected_verdict_and_duplicate_ids() {
    let mut changed = baseline();
    changed["recovery_cases"][0]["expected"] = json!("EVIDENCE_INSUFFICIENT");
    assert_rejected(&run_json(&changed), "false-positive-recovery");

    let mut changed = baseline();
    changed["trace_cases"][0]["id"] = changed["decision_cases"][0]["id"].clone();
    assert_rejected(&run_json(&changed), "duplicate or empty case id");
}

#[test]
fn compiled_cli_rejects_missing_file_and_extra_arguments() {
    let nonexistent = unique_path();
    assert!(!nonexistent.exists(), "unexpected existing test fixture");
    let missing = Command::new(binary())
        .arg(nonexistent)
        .output()
        .expect("spawn CLI for missing file");
    assert_rejected(&missing, "cannot read");

    let extra = Command::new(binary())
        .arg("unused-first-argument.json")
        .arg("unexpected-second-argument.json")
        .output()
        .expect("spawn CLI with invalid argument count");
    assert_rejected(&extra, "usage: free-energy-phase0-fixtures");
}
