//! Black-box admission boundary for the independent candidate-digest executable.
//! This is not evidence of permission to construct a Git commit or move a ref.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_cli(fixture_path: Option<&Path>) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Phase0/rust-fixtures has a repository root");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_integration_candidate_digest"));
    cmd.current_dir(root);
    if let Some(path) = fixture_path {
        cmd.arg(path);
    }
    cmd.output().expect("the compiled digest CLI must launch")
}

fn temporary_fixture(contents: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("positive system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "free-energy-digest-{}-{unique}.json",
        std::process::id()
    ));
    fs::write(&path, contents).expect("write isolated fixture");
    path
}

#[test]
fn cli_emits_exact_historical_digest_values_and_honest_authority_boundary() {
    let result = run_cli(None);
    assert!(result.status.success(), "{result:?}");
    let out = String::from_utf8(result.stdout).expect("UTF-8 CLI output");
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        lines[0],
        "normal-two-parent: 7e8986591c4293e4eeebf751a2c12e19512d4792e5cd8267e6308fe0a479d8da"
    );
    assert_eq!(
        lines[1],
        "reversed-parents-same-tree: b9bea4f05c5cf67a3ed24dafae1600bb5f58e30563b96981afac9a8322efdaf8"
    );
    let stderr = String::from_utf8(result.stderr).expect("UTF-8 diagnostic");
    assert!(stderr.contains("does not authorize Git mutations"));
}

#[test]
fn cli_rejects_missing_json_and_forged_case_identifiers_without_positive_stdout() {
    let no_file = run_cli(Some(Path::new("this-fixture-does-not-exist.json")));
    assert!(!no_file.status.success());
    assert!(no_file.stdout.is_empty());

    let invalid = temporary_fixture("{invalid-json");
    let invalid_result = run_cli(Some(&invalid));
    fs::remove_file(&invalid).expect("remove test fixture");
    assert!(!invalid_result.status.success());
    assert!(invalid_result.stdout.is_empty());
    assert!(!invalid_result.stderr.is_empty());

    let mut fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/integration-candidate-v1.json"))
            .expect("historical fixture parses");
    fixture["cases"][1]["name"] = Value::String("normal-two-parent".to_owned());
    let duplicate = temporary_fixture(&fixture.to_string());
    let result = run_cli(Some(&duplicate));
    fs::remove_file(&duplicate).expect("remove test fixture");
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate case"));
}
