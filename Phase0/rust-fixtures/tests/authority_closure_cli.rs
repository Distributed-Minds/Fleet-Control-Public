//! Compiled-command admission controls for the historical authority-closure oracle.
//! Unlike in-module unit tests, these execute the actual Cargo-built binary.
//! They do not exercise or grant real revocation or provider authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const HISTORICAL: &str = include_str!("../../fixtures/authority-closure-spec2.json");
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn run_source(source: &str) -> Output {
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-authority-closure-cli-{}-{id}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("unique temporary authority fixture");
    file.write_all(source.as_bytes())
        .expect("write temporary authority fixture");
    drop(file);

    let result = Command::new(env!("CARGO_BIN_EXE_authority_closure"))
        .arg(&path)
        .output()
        .expect("execute compiled authority-closure oracle");
    fs::remove_file(&path).expect("remove temporary authority fixture");
    result
}

fn run_path(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_authority_closure"))
        .arg(path)
        .output()
        .expect("execute compiled authority-closure oracle on supplied path")
}

fn run(fixture: &Value) -> Output {
    run_source(&serde_json::to_string(fixture).expect("serialize fixture"))
}

fn assert_denied(result: &Output) {
    assert!(
        !result.status.success(),
        "invalid fixture accepted: {result:?}"
    );
    assert!(
        result.stdout.is_empty(),
        "invalid fixture produced a misleading success record: {result:?}"
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("authority-closure validation FAILED"),
        "missing stable failure diagnosis: {result:?}"
    );
}

fn case<'a>(fixture: &'a mut Value, name: &str) -> &'a mut Value {
    fixture["cases"]
        .as_array_mut()
        .expect("cases array")
        .iter_mut()
        .find(|item| item["name"].as_str() == Some(name))
        .expect("historical case exists")
}

#[test]
fn compiled_binary_accepts_exact_historical_26_case_fixture() {
    let result = run_source(HISTORICAL);
    assert!(
        result.status.success(),
        "historical fixture rejected: {result:?}"
    );
    assert!(
        result.stderr.is_empty(),
        "unexpected diagnostics: {result:?}"
    );
    assert!(
        String::from_utf8_lossy(&result.stdout)
            .contains("authority closure Rust semantic fixtures: 26 cases passed"),
        "missing actual CLI result: {result:?}"
    );
}


#[test]
fn rejects_oversized_sparse_symlink_and_nonregular_fixtures() {
    let serial = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let scratch = std::env::temp_dir().join(format!(
        "free-energy-authority-fixture-bound-{}-{serial}",
        process::id()
    ));
    fs::create_dir_all(&scratch).expect("create isolated fixture directory");
    let oversized = scratch.join("oversized.json");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&oversized)
        .expect("create sparse fixture");
    file.set_len(8 * 1024 * 1024 + 1)
        .expect("extend beyond input limit");
    drop(file);
    assert_denied(&run_path(&oversized));
    assert_denied(&run_path(&scratch));

    #[cfg(unix)]
    {
        let symlink = scratch.join("symlink.json");
        let historical = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures/authority-closure-spec2.json");
        std::os::unix::fs::symlink(&historical, &symlink)
            .expect("create symlink fixture");
        assert_denied(&run_path(&symlink));
    }
    fs::remove_dir_all(&scratch).expect("remove isolated fixture directory");
}

#[test]
fn exactly_eight_mib_of_valid_json_with_trailing_whitespace_is_admitted() {
    let bound = 8 * 1024 * 1024;
    assert!(HISTORICAL.len() < bound);
    let padded = format!("{HISTORICAL}{}", " ".repeat(bound - HISTORICAL.len()));
    let result = run_source(&padded);
    assert!(result.status.success(), "exact-limit fixture rejected: {result:?}");
    assert!(result.stderr.is_empty(), "unexpected diagnostics: {result:?}");
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "authority closure Rust semantic fixtures: 26 cases passed\\n".replace("\\n", "\n")
    );
}

#[test]
fn fabricated_expected_verdict_cannot_change_computed_result() {
    let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
    case(&mut fixture, "child-after-cutoff-denied")["expected"] = json!("BOUNDED_AUTHORITY");
    assert_denied(&run(&fixture));
}

#[test]
fn revocation_evidence_mutation_invalidates_historical_expected_verdict() {
    let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
    // If external invalidation has completed, the historical outstanding-debt
    // verdict is no longer correct. The oracle must compute the new result.
    case(
        &mut fixture,
        "provider-valid-revoked-credential-denied-with-debt",
    )["external_invalidation_complete"] = json!(true);
    assert_denied(&run(&fixture));
}

#[test]
fn duplicate_and_missing_case_identifiers_fail_closed() {
    let baseline: Value = serde_json::from_str(HISTORICAL).unwrap();
    let mut duplicate = baseline.clone();
    duplicate["cases"][1]["name"] = duplicate["cases"][0]["name"].clone();
    assert_denied(&run(&duplicate));

    let mut missing = baseline;
    missing["cases"].as_array_mut().unwrap().pop();
    assert_denied(&run(&missing));
}

#[test]
fn unknown_authority_input_and_invalid_type_fail_closed() {
    let baseline: Value = serde_json::from_str(HISTORICAL).unwrap();
    let mut unknown = baseline.clone();
    case(&mut unknown, "child-after-cutoff-denied")["mint_new_authority"] = json!(true);
    assert_denied(&run(&unknown));

    let mut wrong_type = baseline;
    case(&mut wrong_type, "child-after-cutoff-denied")["ancestor_cutoff"] = json!("true");
    assert_denied(&run(&wrong_type));
}

#[test]
fn unsupported_spec_and_malformed_json_fail_closed() {
    let mut fixture: Value = serde_json::from_str(HISTORICAL).unwrap();
    fixture["spec"] = json!(3);
    assert_denied(&run(&fixture));
    assert_denied(&run_source("{\"spec\":2,\"cases\":["));
}
