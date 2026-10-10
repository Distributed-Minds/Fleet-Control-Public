//! Process-level adversarial controls for the historical coordination-history oracle.
//! These test computed decisions after mutating *inputs*, never by changing only
//! expected outcomes or accepting fixture labels as an oracle. Not archive authority.
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/coordination-history-spec5.json");
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

fn source() -> Value {
    serde_json::from_str(BASELINE).expect("parse checked-in historical fixture")
}

fn run_fixture(text: &str) -> Output {
    let sequence = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-history-adversarial-{}-{sequence}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create isolated adversarial input");
    file.write_all(text.as_bytes())
        .expect("write adversarial input");
    drop(file);
    let result = Command::new(env!("CARGO_BIN_EXE_coordination_history"))
        .arg(&path)
        .output()
        .expect("run compiled coordination-history oracle");
    fs::remove_file(path).expect("remove adversarial fixture");
    result
}

fn run_json(value: &Value) -> Output {
    run_fixture(&serde_json::to_string(value).expect("serialize adversarial fixture"))
}

fn rejects(output: Output) {
    assert!(
        !output.status.success(),
        "oracle accepted unsafe mutation: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "failed verification emitted a success record: {output:?}"
    );
    assert!(
        !output.stderr.is_empty(),
        "failed verification lacked actionable diagnostic"
    );
}

#[test]
fn checked_in_fixture_exercises_the_real_binary() {
    let output = run_fixture(BASELINE);
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("PASS: 18 independently evaluated coordination-history cases"),
        "{output:?}"
    );
}

#[test]
fn revoking_each_currentness_predicate_blocks_a_positive_archive_cut() {
    for property in [
        "archive_exact",
        "order_current",
        "manifest_current",
        "snapshot_coherent",
        "source_current",
        "authority_current",
        "horizon_current",
        "durability_current",
    ] {
        let mut fixture = source();
        fixture["cases"][0]["facts"][property] = json!(false);
        rejects(run_json(&fixture));
    }
}

#[test]
fn unknown_effect_and_partial_delete_cannot_succeed_as_eligible() {
    for (property, value) in [
        ("manifest_transition_outcome", json!("unknown")),
        ("partial_effect", json!(true)),
        ("protected", json!(true)),
        ("archive_live_overlap", json!(true)),
        ("horizon_state", json!("shorter_successor")),
    ] {
        let mut fixture = source();
        fixture["cases"][0]["facts"][property] = value;
        rejects(run_json(&fixture));
    }
}

#[test]
fn missing_baseline_or_corrupt_fixtures_fail_closed() {
    let mut fixture = source();
    fixture["cases"].as_array_mut().unwrap().remove(0);
    rejects(run_json(&fixture));

    let mut fixture = source();
    fixture["cases"][0]["facts"]["archive_exact"] = Value::Null;
    rejects(run_json(&fixture));

    let mut fixture = source();
    fixture["cases"][0]["facts"]["authority_current"] = json!("true");
    rejects(run_json(&fixture));

    let mut fixture = source();
    fixture["cases"][0]["facts"]["unknown_deletion_authority"] = json!(true);
    rejects(run_json(&fixture));

    rejects(run_fixture("{"));
}

#[test]
fn duplicate_identity_and_forged_expected_dispositions_are_not_evidence() {
    let mut fixture = source();
    fixture["cases"][1]["id"] = fixture["cases"][0]["id"].clone();
    rejects(run_json(&fixture));

    let mut fixture = source();
    fixture["cases"][0]["expect"] = json!("INELIGIBLE");
    rejects(run_json(&fixture));

    let mut fixture = source();
    fixture["cases"][14]["expect"] = json!("ELIGIBLE");
    rejects(run_json(&fixture));
}
