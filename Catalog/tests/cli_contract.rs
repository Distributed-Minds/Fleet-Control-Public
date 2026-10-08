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
fn project_directory_admits_all_pilot_manifests_in_sorted_order() {
    let projects = manifest("projects");
    let output = invoke(&["validate", &projects]);
    assert!(
        output.status.success(),
        "project directory rejected: {output:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected diagnostics: {output:?}"
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    let paths: Vec<_> = stdout
        .lines()
        .map(|line| {
            assert!(line.starts_with("TYPED-BOUNDARY-ONLY "), "{line}");
            let encoded = line.split_once(": ").expect("admitted manifest path").1;
            serde_json::from_str::<String>(encoded).expect("JSON-quoted admitted path")
        })
        .collect();
    assert_eq!(paths.len(), 3, "every pilot manifest should be admitted");
    assert!(
        paths.windows(2).all(|pair| pair[0] < pair[1]),
        "directory iteration order must not affect output: {paths:?}"
    );
}

#[test]
fn project_directory_plus_single_manifest_rejects_duplicate_id_atomically() {
    let projects = manifest("projects");
    let luanti = manifest("projects/luanti.json");
    let output = invoke(&["validate", &projects, &luanti]);
    assert_no_partial_success(&output);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("duplicate project ID"),
        "missing duplicate-ID diagnosis: {output:?}"
    );
}

#[test]
fn directory_of_fixture_documents_cannot_masquerade_as_projects() {
    let fixtures = manifest("fixtures");
    let output = invoke(&["validate", &fixtures]);
    assert_no_partial_success(&output);
}

#[test]
fn deterministic_render_check_matches_committed_static_page() {
    let output = invoke(&["render", "--check"]);
    assert!(
        output.status.success(),
        "committed catalog page is stale: {output:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected render diagnostics: {output:?}"
    );
}

#[cfg(unix)]
#[test]
fn accepted_control_character_filename_cannot_forge_another_success_record() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);
    let file = std::env::temp_dir().join(format!(
        "free-energy-catalog-{}-{}\nTYPED-BOUNDARY-ONLY fake: forged.json",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&file, include_str!("../projects/luanti.json"))
        .expect("write the valid test manifest under a hostile pathname");
    let output = invoke(&["validate", file.to_str().expect("UTF-8 test filename")]);
    std::fs::remove_file(&file).expect("remove temporary test manifest");

    assert!(output.status.success(), "admitted record failed: {output:?}");
    assert!(output.stderr.is_empty(), "unexpected failure: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 success record");
    assert_eq!(
        stdout.lines().count(),
        1,
        "embedded filename newline manufactured another success-looking line"
    );
    let line = stdout.lines().next().expect("one success record");
    assert!(line.starts_with("TYPED-BOUNDARY-ONLY engine/luanti: "));
    let encoded = line.split_once(": ").expect("admitted path").1;
    let decoded: String = serde_json::from_str(encoded).expect("reversible JSON path");
    assert_eq!(decoded, file.to_str().expect("UTF-8 test filename"));
    assert!(encoded.contains(r"\n"), "pathname newline was not escaped");
}
