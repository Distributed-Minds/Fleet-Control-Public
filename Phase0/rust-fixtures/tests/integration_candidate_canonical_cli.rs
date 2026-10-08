//! CLI-boundary verification for Python-compatible candidate canonical bytes.
//! The historical Python sha256 identity check remains a separate obligation.
//! These tests do not authorize Git mutation or claim complete digest parity.

use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

const EXECUTABLE: &str = env!("CARGO_BIN_EXE_integration_candidate");

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/integration-candidate-v1.json")
}

fn emit(case_name: &str) -> Output {
    Command::new(EXECUTABLE)
        .arg("--emit-canonical")
        .arg(case_name)
        .arg(fixture())
        .output()
        .expect("run compiled integration candidate oracle")
}

#[test]
fn canonical_cli_preserves_parent_order_in_real_process_output() {
    let normal = emit("normal-two-parent");
    let reversed = emit("reversed-parents-same-tree");
    assert!(normal.status.success(), "normal case: {normal:?}");
    assert!(reversed.status.success(), "reversed case: {reversed:?}");
    assert!(normal.stderr.is_empty());
    assert!(reversed.stderr.is_empty());
    assert_ne!(normal.stdout, reversed.stdout);
    assert!(normal.stdout.starts_with(b"{\"compatibility_basis\":"));
    let parsed: Value = serde_json::from_slice(&normal.stdout).unwrap();
    assert_eq!(parsed["parents"][0], parsed["target_commit"]);
    assert_eq!(parsed["parents"][1], parsed["source_commit"]);
    let reversed_parsed: Value = serde_json::from_slice(&reversed.stdout).unwrap();
    assert_eq!(reversed_parsed["parents"][0], parsed["source_commit"]);
    assert_eq!(reversed_parsed["parents"][1], parsed["target_commit"]);
}

#[test]
fn canonical_cli_preserves_nested_metadata_and_unicode_without_ascii_folding() {
    let output = emit("normal-two-parent");
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\"author_time\":\"1700000000 +0000\""));
    assert!(text.contains("\"message\":\"Integrate source\\n\""));
    assert!(!text.contains("\n"), "JSON control bytes must be escaped");
    // The companion in-process canonical tests cover non-ASCII keys/values
    // because the historical six-case input uses only ASCII metadata.
}

#[test]
fn unsupported_or_unknown_cases_cannot_masquerade_as_successful_output() {
    let missing = emit("not-a-historical-case");
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("FAIL:"));

    let missing_arg = Command::new(EXECUTABLE)
        .arg("--emit-canonical")
        .output()
        .unwrap();
    assert!(!missing_arg.status.success());
    assert!(missing_arg.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_arg.stderr).contains("usage:"));

    let extra_arg = Command::new(EXECUTABLE)
        .args(["--emit-canonical", "normal-two-parent"])
        .arg(fixture())
        .arg("unexpected")
        .output()
        .unwrap();
    assert!(!extra_arg.status.success());
    assert!(extra_arg.stdout.is_empty());
}

#[test]
fn malformed_or_inaccessible_input_fails_before_emitting_any_identity_bytes() {
    let missing = Command::new(EXECUTABLE)
        .args(["--emit-canonical", "normal-two-parent", "/nonexistent/fixture.json"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("FAIL:"));
}
