//! Process-level negative controls for the migrated GitHub capability fixture
//! validator. These execute the compiled CLI, not a copy of its decision code.
//! They do not establish actual permission or probe-mutation authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BASELINE: &str = include_str!("../../fixtures/github-capability-spec5.json");
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("checked-in historical fixture")
}

fn run_bytes(bytes: &[u8]) -> Output {
    let path = std::env::temp_dir().join(format!(
        "free-energy-ghcap-negative-{}-{}.json",
        process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique temporary fixture");
    file.write_all(bytes).expect("write temporary fixture");
    drop(file);
    let result = Command::new(env!("CARGO_BIN_EXE_github_capability"))
        .arg(&path)
        .output()
        .expect("execute compiled GitHub capability fixture CLI");
    fs::remove_file(&path).expect("remove temporary fixture");
    result
}

fn run_fixture(fixture: &Value) -> Output {
    let bytes = serde_json::to_vec(fixture).expect("serialize fixture mutation");
    run_bytes(&bytes)
}

fn reject(label: &str, fixture: &Value) {
    let output = run_fixture(fixture);
    assert!(
        !output.status.success(),
        "{label}: invalid fixture was accepted: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "{label}: rejected input emitted a positive success record: {output:?}"
    );
    assert!(
        !output.stderr.is_empty(),
        "{label}: rejection gave no diagnostics"
    );
}

fn change_case(id: u64, field: &str, replacement: Value) -> Value {
    let mut fixture = baseline();
    let case = fixture["cases"]
        .as_array_mut()
        .expect("cases array")
        .iter_mut()
        .find(|c| c["id"] == json!(id))
        .expect("existing case");
    case[field] = replacement;
    fixture
}

#[test]
fn full_twenty_eight_case_baseline_is_executed_by_real_binary() {
    let output = run_bytes(BASELINE.as_bytes());
    assert!(output.status.success(), "baseline rejected: {output:?}");
    assert!(output.stderr.is_empty(), "baseline diagnostics: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 CLI output");
    assert!(
        stdout.contains("28 checked"),
        "missing 28-case CLI completion evidence: {stdout}"
    );
}

#[test]
fn permission_lineage_recovery_and_fence_mutations_fail_closed() {
    for (label, mutated) in [
        (
            "default-branch-write-unblocked",
            change_case(8, "expected", json!("ALLOW_FENCED")),
        ),
        (
            "ordinary-write-stale-authority",
            change_case(16, "authority", json!("stale")),
        ),
        (
            "ordinary-write-forged-authority",
            change_case(16, "authority", json!("forged")),
        ),
        (
            "ordinary-write-without-fencing",
            change_case(16, "fenced", json!(false)),
        ),
        (
            "ordinary-write-unknown-lineage",
            change_case(16, "lineage", json!("unknown")),
        ),
        (
            "ordinary-write-ambiguous-resource",
            change_case(16, "resource", json!("ambiguous")),
        ),
        (
            "ordinary-write-requires-approval",
            change_case(16, "approval", json!("required")),
        ),
        (
            "stale-cleanup-lacks-recovery-transfer",
            change_case(21, "recovery", json!(false)),
        ),
        (
            "stale-cleanup-has-ambiguous-resource",
            change_case(21, "resource", json!("ambiguous")),
        ),
        (
            "successor-lineage-without-proof",
            change_case(24, "successor_evidence", json!(false)),
        ),
    ] {
        reject(label, &mutated);
    }
}

#[test]
fn missing_duplicate_and_malformed_fixture_inputs_cannot_pass() {
    let mut missing = baseline();
    missing["cases"].as_array_mut().expect("cases array").pop();
    reject("missing-required-id", &missing);

    let mut duplicated = baseline();
    duplicated["cases"][1]["id"] = json!(1);
    reject("duplicate-case-id", &duplicated);

    let mut duplicate_name = baseline();
    duplicate_name["cases"][1]["name"] = duplicate_name["cases"][0]["name"].clone();
    reject("duplicate-case-name", &duplicate_name);

    let mut wrong_spec = baseline();
    wrong_spec["spec"] = json!(4);
    reject("unsupported-spec", &wrong_spec);

    let mut unknown_field = baseline();
    unknown_field["cases"][0]["self_authorizes"] = json!(true);
    reject("unknown-authority-field", &unknown_field);

    let mut wrong_type = baseline();
    wrong_type["cases"][15]["fenced"] = json!("true");
    reject("bool-string-coercion", &wrong_type);

    let malformed = run_bytes(b"{invalid JSON");
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());
}

#[test]
fn unexpected_extra_cli_argument_is_rejected_without_success_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_github_capability"))
        .args(["fixture.json", "extra.json"])
        .output()
        .expect("execute compiled CLI with excess arguments");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}
