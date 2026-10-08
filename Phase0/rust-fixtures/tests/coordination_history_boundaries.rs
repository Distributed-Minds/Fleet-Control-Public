//! Process-level negative controls for the Rust coordination-history spec-5 oracle.
//!
//! The model's historical verdicts are not proof of production archival or deletion
//! authority. These tests exercise the compiled binary with changed factual inputs
//! while preserving expectations, so an oracle that merely echoes fixture labels
//! cannot pass them.
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/coordination-history-spec5.json"
    ))
    .expect("historical coordination-history suite must parse")
}

fn case_mut<'a>(suite: &'a mut Value, id: &str) -> &'a mut Value {
    suite["cases"]
        .as_array_mut()
        .expect("cases array")
        .iter_mut()
        .find(|case| case["id"].as_str() == Some(id))
        .expect("named historical case")
}

fn temp_input(suite: &Value) -> PathBuf {
    let serial = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-coordination-boundaries-{}-{serial}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique fixture input");
    let bytes = serde_json::to_vec(suite).expect("serialize mutated suite");
    file.write_all(&bytes).expect("write mutated fixture");
    path
}

fn invoke(suite: &Value) -> Output {
    let path = temp_input(suite);
    let output = Command::new(env!("CARGO_BIN_EXE_coordination_history"))
        .arg(&path)
        .output()
        .expect("execute compiled coordination-history Rust oracle");
    fs::remove_file(path).expect("remove temporary fixture");
    output
}

fn assert_fail_closed(suite: &Value, changed_case: &str) {
    let output = invoke(suite);
    assert!(
        !output.status.success(),
        "{changed_case}: mutated safety facts were accepted"
    );
    assert!(
        output.stdout.is_empty(),
        "{changed_case}: rejected input emitted a success record"
    );
    let diagnostic = String::from_utf8(output.stderr).expect("UTF-8 failure diagnostic");
    assert!(
        diagnostic.contains(changed_case),
        "{changed_case}: missing case-specific diagnostic: {diagnostic}"
    );
}

#[test]
fn unmodified_historical_suite_is_still_executable() {
    let output = invoke(&fixture());
    assert!(
        output.status.success(),
        "historical suite unexpectedly failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty(), "missing successful run output");
}

#[test]
fn every_independent_currentness_prerequisite_is_required() {
    // This positive case inherits the all-current baseline. Flipping one
    // independent predicate at a time must change the computed disposition,
    // not just the value asserted by the historical fixture.
    for field in [
        "archive_exact",
        "order_current",
        "manifest_current",
        "snapshot_coherent",
        "source_current",
        "authority_current",
        "horizon_current",
        "durability_current",
    ] {
        let mut suite = fixture();
        case_mut(&mut suite, "mixed-cut-overlap-dedup")["facts"][field] = json!(false);
        assert_fail_closed(&suite, "mixed-cut-overlap-dedup");
    }
}

#[test]
fn overlapping_archive_and_live_records_need_deduplication_identity() {
    let mut suite = fixture();
    case_mut(&mut suite, "mixed-cut-overlap-dedup")["facts"]["stable_record_identity"] =
        json!(false);
    assert_fail_closed(&suite, "mixed-cut-overlap-dedup");
}

#[test]
fn active_owner_protection_cannot_be_overridden_by_other_current_evidence() {
    let mut suite = fixture();
    case_mut(&mut suite, "mixed-cut-overlap-dedup")["facts"]["protected"] = json!(true);
    assert_fail_closed(&suite, "mixed-cut-overlap-dedup");
}

#[test]
fn opaque_ordering_evidence_is_not_a_trusted_order_basis() {
    let mut suite = fixture();
    case_mut(&mut suite, "mixed-cut-overlap-dedup")["facts"]["order_evidence"] =
        json!("opaque_timestamp");
    assert_fail_closed(&suite, "mixed-cut-overlap-dedup");
}

#[test]
fn uncertain_remote_effect_requires_reconciliation_not_eligibility() {
    for (field, value) in [
        ("partial_effect", json!(true)),
        ("manifest_transition_outcome", json!("unknown")),
    ] {
        let mut suite = fixture();
        case_mut(&mut suite, "mixed-cut-overlap-dedup")["facts"][field] = value;
        assert_fail_closed(&suite, "mixed-cut-overlap-dedup");
    }
}

#[test]
fn authorized_horizon_retirement_requires_predecessor_retirement_proof() {
    let mut suite = fixture();
    let case = case_mut(&mut suite, "authorized-bounded-retirement");
    case["facts"]["horizon_state"] = json!("shorter_successor");
    case["facts"]["predecessor_record_retired"] = json!(false);
    assert_fail_closed(&suite, "authorized-bounded-retirement");
}

#[test]
fn altering_expected_verdict_cannot_make_revoked_authority_eligible() {
    let mut suite = fixture();
    case_mut(&mut suite, "authority-revoked")["expect"] = json!("ELIGIBLE");
    assert_fail_closed(&suite, "authority-revoked");
}
