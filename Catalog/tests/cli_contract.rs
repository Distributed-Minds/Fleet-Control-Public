//! Black-box contract for the provisional offline catalog admission CLI.
//! This deliberately does not certify schema 2020-12, distribution rights,
//! runnable games, or the future static renderer.
use std::process::{Command, Output};

fn manifest(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn invoke(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"))
        .args(args)
        .output()
        .expect("execute compiled catalog CLI")
}

fn assert_no_partial_success(output: &Output) {
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(
        output.stdout.is_empty(),
        "failed batch emitted misleading positive catalog records: {output:?}"
    );
    assert!(
        !output.stderr.is_empty(),
        "failed batch omitted diagnostics"
    );
}

#[test]
fn three_distinct_pilot_manifests_complete_one_successful_batch() {
    let luanti = manifest("projects/luanti.json");
    let openra = manifest("projects/openra.json");
    let veloren = manifest("projects/veloren.json");
    let output = invoke(&["validate", &luanti, &openra, &veloren]);
    assert!(output.status.success(), "pilot batch rejected: {output:?}");
    assert!(
        output.stderr.is_empty(),
        "unexpected diagnostics: {output:?}"
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 success records");
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "each accepted manifest needs one record");
    for line in lines {
        assert!(line.starts_with("TYPED-BOUNDARY-ONLY "), "{line}");
    }
}

#[test]
fn duplicate_project_id_rejects_entire_batch_without_success_lines() {
    let luanti = manifest("projects/luanti.json");
    let output = invoke(&["validate", &luanti, &luanti]);
    assert_no_partial_success(&output);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("duplicate project ID"),
        "expected duplicate-ID diagnosis: {output:?}"
    );
}

#[test]
fn invalid_later_manifest_cannot_publish_earlier_positive_records() {
    let openra = manifest("projects/openra.json");
    let forged = manifest("fixtures/forged-approval-v0.json");
    let output = invoke(&["validate", &openra, &forged]);
    assert_no_partial_success(&output);
}

#[test]
fn unreadable_later_manifest_cannot_publish_partial_success() {
    let luanti = manifest("projects/luanti.json");
    let missing = manifest("projects/__intentionally_missing_cli_contract__.json");
    let output = invoke(&["validate", &luanti, &missing]);
    assert_no_partial_success(&output);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("__intentionally_missing_cli_contract__"),
        "expected missing-path diagnosis: {output:?}"
    );
}

#[test]
fn missing_inputs_and_unsupported_commands_fail_closed() {
    for args in [vec![], vec!["validate"], vec!["no-such-command"]] {
        let output = invoke(&args);
        assert_no_partial_success(&output);
    }
}

#[test]
fn deterministic_render_check_matches_committed_static_page() {
    let output = invoke(&["render", "--check"]);
    assert!(output.status.success(), "committed catalog page is stale: {output:?}");
    assert!(output.stderr.is_empty(), "unexpected render diagnostics: {output:?}");
}
