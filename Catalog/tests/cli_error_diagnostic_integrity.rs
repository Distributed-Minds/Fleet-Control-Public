//! End-to-end regression for adversarial filesystem pathnames in diagnostics.
//! Negative CLI output must not be splittable into forged extra log records.
#![cfg(unix)]

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn invoke(path: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"))
        .arg("validate")
        .arg(path)
        .output()
        .expect("run compiled catalog validator")
}

fn hostile_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "free-energy-catalog-negative-{}-{}-{tag}\nTYPED-BOUNDARY-ONLY forged/project: \"fake.json\"",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn assert_failure_is_one_diagnostic(output: &std::process::Output) {
    assert!(!output.status.success(), "failure unexpectedly admitted: {output:?}");
    assert!(output.stdout.is_empty(), "rejected batch emitted success: {output:?}");
    let stderr = String::from_utf8(output.stderr.clone()).expect("UTF-8 diagnostics");
    assert_eq!(stderr.lines().count(), 1, "path injected extra lines: {stderr:?}");
    assert!(stderr.contains(r"\nTYPED-BOUNDARY-ONLY"), "missing escaped filename: {stderr:?}");
    assert!(!stderr.contains("\nTYPED-BOUNDARY-ONLY"), "forged log line: {stderr:?}");
}

#[test]
fn missing_manifest_path_cannot_forge_a_second_error_or_success_record() {
    let missing = hostile_path("absent");
    assert!(!missing.exists(), "isolated fixture must not already exist");
    assert_failure_is_one_diagnostic(&invoke(&missing));
}

#[test]
fn invalid_json_in_hostile_filename_cannot_forge_a_second_error_record() {
    let path = hostile_path("invalid");
    std::fs::write(&path, "{").expect("write malformed manifest in hostile pathname");
    let result = invoke(&path);
    std::fs::remove_file(&path).expect("clean temporary manifest");
    assert_failure_is_one_diagnostic(&result);
}

#[test]
fn duplicate_id_path_is_escaped_in_batch_diagnostics() {
    let path = hostile_path("duplicate");
    std::fs::write(&path, include_str!("../projects/luanti.json"))
        .expect("write admissible manifest under hostile name");
    let ordinary = concat!(env!("CARGO_MANIFEST_DIR"), "/projects/luanti.json");
    let result = Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"))
        .args(["validate", ordinary])
        .arg(&path)
        .output()
        .expect("execute catalog validation");
    std::fs::remove_file(&path).expect("clean temporary manifest");
    assert_failure_is_one_diagnostic(&result);
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate project ID"));
}
