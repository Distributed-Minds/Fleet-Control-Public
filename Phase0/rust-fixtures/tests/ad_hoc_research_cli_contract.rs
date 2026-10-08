//! Process-level regression for the independently computed ad-hoc research fixture oracle.
//! This is historical fixture parity evidence, not GitHub mutation authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/ad-hoc-research-spec1.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn executable() -> &'static str {
    env!("CARGO_BIN_EXE_ad_hoc_research")
}

fn temporary_path() -> PathBuf {
    let number = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "free-energy-ad-hoc-cli-{}-{number}.json",
        process::id()
    ))
}

fn invoke(path: &Path) -> Output {
    Command::new(executable())
        .arg(path)
        .output()
        .expect("spawn compiled ad-hoc research binary")
}

fn invoke_contents(contents: &str) -> Output {
    let path = temporary_path();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique temporary fixture");
    file.write_all(contents.as_bytes())
        .expect("write exact fixture bytes");
    drop(file);
    let output = invoke(&path);
    fs::remove_file(path).expect("remove temporary fixture");
    output
}

fn invoke_value(value: &Value) -> Output {
    invoke_contents(&serde_json::to_string(value).expect("serialize fixture"))
}

fn assert_denied(output: Output, expected_diagnostic: &str) {
    assert!(
        !output.status.success(),
        "unsafe fixture incorrectly passed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stdout.is_empty(),
        "a rejected fixture emitted a successful result"
    );
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(
        diagnostic.contains(expected_diagnostic),
        "expected {expected_diagnostic:?}, got {diagnostic}"
    );
}

fn source() -> Value {
    serde_json::from_str(BASELINE).expect("pinned historical 47-case fixture")
}

#[test]
fn baseline_47_cases_are_checked_by_the_real_binary() {
    let output = invoke_contents(BASELINE);
    assert!(
        output.status.success(),
        "ad-hoc CLI rejected baseline: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "ad-hoc research fixtures (Rust): 47 passed"
    );
}

#[test]
fn revoked_authority_rejects_a_formerly_valid_publication() {
    let mut fixture = source();
    fixture["publication_cases"][0]["authority_current"] = json!(false);
    assert_denied(invoke_value(&fixture), "expected");
}

#[test]
fn forged_expected_identity_does_not_become_the_oracle() {
    let mut fixture = source();
    fixture["identity_cases"][0]["expected"] = json!("NEW_LINEAGE");
    assert_denied(invoke_value(&fixture), "expected");
}

#[test]
fn invalid_input_types_fail_closed() {
    let mut fixture = source();
    fixture["publication_cases"][0]["authority_current"] = json!("false");
    assert_denied(invoke_value(&fixture), "must be a Boolean");
}

#[test]
fn missing_and_duplicate_fixture_families_cannot_claim_47_cases() {
    let mut fixture = source();
    fixture["publication_cases"].as_array_mut().unwrap().pop();
    assert_denied(invoke_value(&fixture), "incomplete publication_cases");

    let mut fixture = source();
    fixture["source_cases"][0]["id"] = fixture["identity_cases"][0]["id"].clone();
    assert_denied(invoke_value(&fixture), "duplicate case id");
}

#[test]
fn incomplete_packet_templates_and_wrong_spec_are_rejected() {
    let mut fixture = source();
    fixture.as_object_mut().unwrap().remove("packet_templates");
    assert_denied(invoke_value(&fixture), "missing packet_templates");

    let mut fixture = source();
    fixture["spec_version"] = json!(999);
    assert_denied(invoke_value(&fixture), "unsupported ad-hoc");
}

#[test]
fn malformed_json_and_missing_file_are_not_success() {
    assert_denied(
        invoke_contents(r#"{"publication_cases":"#),
        "invalid fixture",
    );
    let missing = temporary_path();
    assert!(!missing.exists(), "unexpected preexisting fixture path");
    assert_denied(invoke(&missing), "cannot read");
}

#[test]
fn extra_positional_arguments_do_not_run_the_fixture() {
    let output = Command::new(executable())
        .arg("unused.json")
        .arg("unexpected.json")
        .output()
        .expect("spawn ad-hoc CLI");
    assert_denied(output, "usage: ad_hoc_research");
}
