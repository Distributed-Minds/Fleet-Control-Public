//! Compiled-command admission controls for the historical authority-closure oracle.
//! Unlike in-module unit tests, these execute the actual Cargo-built binary.
//! They do not exercise or grant real revocation or provider authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
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

fn run(fixture: &Value) -> Output {
    run_source(&serde_json::to_string(fixture).expect("serialize fixture"))
}

fn run_path(path: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_authority_closure"))
        .arg(path)
        .output()
        .expect("execute compiled authority-closure oracle")
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
fn compiled_binary_rejects_nonregular_symlink_and_oversized_fixture_files() {
    const LIMIT: usize = 8 * 1024 * 1024;
    let id = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let scratch = std::env::temp_dir().join(format!(
        "free-energy-authority-closure-input-{}-{id}",
        process::id()
    ));
    fs::create_dir(&scratch).expect("create distinct test folder");

    // An exact 8 MiB UTF-8 JSON document is a positive boundary control.
    let exact_path = scratch.join("exact.json");
    let mut exact = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&exact_path)
        .unwrap();
    assert!(HISTORICAL.len() < LIMIT);
    exact.write_all(HISTORICAL.as_bytes()).unwrap();
    exact.write_all(&vec![b' '; LIMIT - HISTORICAL.len()])
        .unwrap();
    drop(exact);
    let accepted = run_path(&exact_path);
    assert!(accepted.status.success(), "8 MiB historical JSON was rejected: {accepted:?}");
    assert!(
        String::from_utf8_lossy(&accepted.stdout).contains("26 cases passed")
    );

    // Directory and sparse oversized inputs must fail before JSON decoding.
    assert_denied(&run_path(&scratch));
    let oversized_path = scratch.join("oversized.json");
    fs::File::create(&oversized_path)
        .unwrap()
        .set_len(LIMIT as u64 + 1)
        .unwrap();
    assert_denied(&run_path(&oversized_path));

    #[cfg(unix)]
    {
        let link = scratch.join("alias.json");
        std::os::unix::fs::symlink(&exact_path, &link).unwrap();
        assert_denied(&run_path(&link));
    }
    fs::remove_dir_all(&scratch).expect("remove test folder");
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
