//! Black-box CLI contract for the Rust coordination-history fixture oracle (#71).
//! The historical Python checker is not needed to execute these checks.
//! This tests an existing validator, not archival safety or deletion authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BASELINE: &str = include_str!("../../fixtures/coordination-history-spec5.json");
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

struct FixtureFile(PathBuf);

impl FixtureFile {
    fn from_value(value: &Value) -> Self {
        let path = std::env::temp_dir().join(format!(
            "free-energy-coordination-cli-{}-{}.json",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("create unique fixture without overwriting another file");
        file.write_all(
            serde_json::to_string(value)
                .expect("serialize mutated fixture")
                .as_bytes(),
        )
        .expect("write mutated fixture");
        Self(path)
    }

    fn as_str(&self) -> &str {
        self.0.to_str().expect("fixture path must be UTF-8")
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("checked-in historical fixture is valid JSON")
}

fn execute(value: &Value) -> Output {
    let fixture = FixtureFile::from_value(value);
    Command::new(env!("CARGO_BIN_EXE_coordination_history"))
        .arg(fixture.as_str())
        .output()
        .expect("execute compiled coordination_history binary")
}

fn rejects_without_success(value: &Value, reason: &str) {
    let output = execute(value);
    assert!(
        !output.status.success(),
        "{reason} unexpectedly accepted: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "{reason} emitted partial PASS output: {output:?}"
    );
    assert!(
        !output.stderr.is_empty(),
        "{reason} rejected without diagnostic: {output:?}"
    );
}

#[test]
fn complete_historical_suite_succeeds_via_compiled_cli() {
    let output = execute(&baseline());
    assert!(
        output.status.success(),
        "valid baseline rejected: {output:?}"
    );
    assert!(output.stderr.is_empty(), "baseline diagnostics: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 CLI output");
    assert_eq!(
        stdout.trim(),
        "PASS: 18 independently evaluated coordination-history cases"
    );
}

#[test]
fn expected_verdict_tampering_and_missing_authority_fail_closed() {
    let mut wrong_expect = baseline();
    wrong_expect["cases"][0]["expect"] = json!("INELIGIBLE");
    rejects_without_success(&wrong_expect, "forged expected verdict");

    let mut revoked = baseline();
    revoked["cases"][0]["facts"]["authority_current"] = json!(false);
    rejects_without_success(&revoked, "revoked mutation authority");

    let mut lost_ack = baseline();
    lost_ack["cases"][0]["facts"]["manifest_transition_outcome"] = json!("unknown");
    rejects_without_success(&lost_ack, "ambiguous acknowledgement");

    let mut partial = baseline();
    partial["cases"][0]["facts"]["partial_effect"] = json!(true);
    rejects_without_success(&partial, "partially applied archive mutation");
}

#[test]
fn incomplete_duplicate_or_future_version_suites_fail_closed() {
    let mut missing = baseline();
    missing["cases"].as_array_mut().expect("case array").pop();
    rejects_without_success(&missing, "missing historical case");

    let mut duplicated = baseline();
    duplicated["cases"][1]["id"] = duplicated["cases"][0]["id"].clone();
    rejects_without_success(&duplicated, "duplicate case identity");

    let mut future_schema = baseline();
    future_schema["schema"] = json!("fleet-control/coordination-history-spec6-fixtures/v1");
    rejects_without_success(&future_schema, "unrecognized fixture schema");
}

#[test]
fn unrecognized_manifest_transition_outcomes_fail_closed() {
    // An unrecognized receipt previously inherited the all-current fixture
    // baseline and produced a misleading positive PASS from the real binary.
    for outcome in [
        "",
        "success",
        "failed",
        "UNKNOWN",
        "unknown ",
        "already_committed",
    ] {
        let mut fixture = baseline();
        fixture["cases"][0]["facts"]["manifest_transition_outcome"] = json!(outcome);
        rejects_without_success(&fixture, "unrecognized manifest transition outcome");
    }
}

#[test]
fn malformed_fact_types_and_unknown_fields_fail_closed() {
    let mut string_bool = baseline();
    string_bool["cases"][0]["facts"]["archive_exact"] = json!("true");
    rejects_without_success(&string_bool, "string used for Boolean admission fact");

    let mut extra_field = baseline();
    extra_field["cases"][0]["facts"]["untrusted_authority_override"] = json!(true);
    rejects_without_success(&extra_field, "unknown authority override");
}
