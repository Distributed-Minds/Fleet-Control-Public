//! Process-level negative controls for the compiled GitHub capability fixture validator.
//! This is a CLI admission regression, NOT scheduled-context capability proof or full oracle parity.

use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/github-capability-spec5.json");
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_github_capability")
}

fn run(contents: &str) -> Output {
    let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-github-capability-cli-{}-{sequence}.json",
        process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create exclusive temporary fixture");
    use std::io::Write;
    file.write_all(contents.as_bytes()).expect("write fixture");
    drop(file);

    let output = Command::new(binary())
        .arg(&path)
        .output()
        .expect("run compiled github_capability binary");
    fs::remove_file(path).expect("remove temporary fixture");
    output
}

fn fixture() -> Value {
    serde_json::from_str(BASELINE).expect("historical 28-case fixture")
}

fn mutate(id: u64, field: &str, value: Value) -> Value {
    let mut f = fixture();
    let target = f["cases"]
        .as_array_mut()
        .expect("cases array")
        .iter_mut()
        .find(|case| case["id"] == json!(id))
        .expect("existing fixture ID");
    target[field] = value;
    f
}

fn reject(value: &Value, diagnostic: &str) {
    let output = run(&serde_json::to_string(value).expect("serialize mutation"));
    assert!(
        !output.status.success(),
        "invalid fixture accepted: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.is_empty(), "failure must not emit a PASS line");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "missing diagnostic {diagnostic:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn compiled_cli_accepts_original_complete_fixture() {
    let output = run(BASELINE);
    assert!(
        output.status.success(),
        "original fixture rejected: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "GitHub capability invariant fixtures (Rust): 28 checked"
    );
}

#[test]
fn compiled_cli_rejects_forged_fencing_recovery_and_lineage() {
    reject(&mutate(16, "fenced", json!(false)), "ALLOW_FENCED");
    reject(&mutate(24, "successor_evidence", json!(false)), "REUSE_LINEAGE");
    reject(&mutate(21, "recovery", json!(false)), "stale cleanup");
    reject(&mutate(14, "resource", json!("none")), "PASSED_CLEAN");
}

#[test]
fn compiled_cli_rejects_invalid_shape_and_incomplete_inventory() {
    reject(&mutate(16, "fenced", json!("true")), "invalid fixture");
    reject(&mutate(16, "unknown_privilege", json!(true)), "invalid fixture");
    let mut duplicate = fixture();
    duplicate["cases"][1]["id"] = json!(1);
    reject(&duplicate, "duplicate or invalid case id");
    let mut missing = fixture();
    missing["cases"].as_array_mut().unwrap().pop();
    reject(&missing, "missing required case id");
    let malformed = run("{\"spec\":");
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("invalid fixture"));
}

#[test]
fn compiled_cli_rejects_missing_file_and_extra_arguments() {
    let absent: PathBuf = std::env::temp_dir().join(format!(
        "free-energy-github-capability-nonexistent-{}-{}",
        process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    assert!(!absent.exists());
    let output = Command::new(binary()).arg(&absent).output().expect("run CLI");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
    let extra = Command::new(binary())
        .arg("unused.json")
        .arg("second.json")
        .output()
        .expect("run CLI with extra argument");
    assert!(!extra.status.success());
    assert!(extra.stdout.is_empty());
    assert!(String::from_utf8_lossy(&extra.stderr).contains("usage: github_capability"));
}
