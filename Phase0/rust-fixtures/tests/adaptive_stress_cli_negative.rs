//! Process-level regression for the real adaptive-stress Rust fixture oracle.
//! No real scheduler, authority mutation, external network, or Git provider
//! operation is performed. This proves executable behavior of the staged
//! migration candidate, not Python/Rust differential equivalence (issue #71).
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/adaptive-stress-spec2.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn fixture() -> Value {
    serde_json::from_str(BASELINE).expect("parse historical spec-2 fixture")
}

fn unique_path() -> PathBuf {
    let sequence = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "free-energy-adaptive-cli-{}-{sequence}.json",
        process::id()
    ))
}

fn run_contents(contents: &str) -> Output {
    let path = unique_path();
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .expect("create unique test fixture");
    file.write_all(contents.as_bytes())
        .expect("write test fixture");
    drop(file);

    let output = Command::new(env!("CARGO_BIN_EXE_adaptive_stress"))
        .arg(&path)
        .output()
        .expect("execute compiled adaptive-stress Rust binary");
    fs::remove_file(&path).expect("remove temporary test fixture");
    output
}

fn run_fixture(fixture: &Value) -> Output {
    run_contents(&serde_json::to_string(fixture).expect("serialize test fixture"))
}

fn change_case(name: &str, field: &str, value: Value) -> Value {
    let mut root = fixture();
    let case = root["cases"]
        .as_array_mut()
        .expect("cases must be a JSON array")
        .iter_mut()
        .find(|case| case["name"] == name)
        .expect("historical case must exist");
    case[field] = value;
    root
}

fn assert_rejected(output: Output) {
    assert!(
        !output.status.success(),
        "unexpected CLI acceptance: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "rejected fixture emitted success: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("FAIL: "),
        "failed fixture lacks deterministic diagnostic: {stderr}"
    );
}

#[test]
fn actual_binary_executes_complete_historical_fixture() {
    let output = run_contents(BASELINE);
    assert!(
        output.status.success(),
        "historical fixture failed: {output:?}"
    );
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "adaptive-stress semantic fixtures (Rust): 23 passed"
    );
}

#[test]
fn process_rejects_incorrect_expected_labels() {
    // If an implementation simply echoes the fixture's expectation, this
    // adversarial replacement will yield a false positive.
    assert_rejected(run_fixture(&change_case(
        "ack-loss-reconciles-first",
        "expected",
        json!("RERUN"),
    )));
}

#[test]
fn process_rejects_meaningful_semantic_mutations() {
    assert_rejected(run_fixture(&change_case(
        "fixture-authority-is-inert",
        "authority_change",
        json!(true),
    )));
    assert_rejected(run_fixture(&change_case(
        "material-target-change-can-split",
        "bounded_allowance",
        json!(0),
    )));
    assert_rejected(run_fixture(&change_case(
        "anti-overfit-requires-siblings",
        "siblings_pass",
        json!(true),
    )));
}

#[test]
fn process_rejects_missing_duplicate_and_invalid_fixture_cases() {
    let mut missing = fixture();
    missing["cases"].as_array_mut().unwrap().pop();
    assert_rejected(run_fixture(&missing));

    let mut duplicate = fixture();
    let original = duplicate["cases"][0].clone();
    duplicate["cases"].as_array_mut().unwrap().push(original);
    assert_rejected(run_fixture(&duplicate));

    let mut wrong_spec = fixture();
    wrong_spec["spec"] = json!(3);
    assert_rejected(run_fixture(&wrong_spec));

    assert_rejected(run_contents("{\"spec\":2,\"cases\":"));
    assert_rejected(run_fixture(&change_case(
        "missing-telemetry-is-unknown",
        "secret_override",
        json!(true),
    )));
}

#[test]
fn process_rejects_missing_input_file_and_extra_args() {
    let nonexistent = unique_path();
    assert!(!nonexistent.exists(), "test target unexpectedly exists");
    assert_rejected(
        Command::new(env!("CARGO_BIN_EXE_adaptive_stress"))
            .arg(&nonexistent)
            .output()
            .expect("execute Rust oracle"),
    );

    assert_rejected(
        Command::new(env!("CARGO_BIN_EXE_adaptive_stress"))
            .arg("unused-first.json")
            .arg("unexpected-second.json")
            .output()
            .expect("execute Rust oracle"),
    );
}
