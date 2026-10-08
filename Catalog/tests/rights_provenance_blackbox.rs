//! Black-box negative regression tests for the evidence-scoped catalog CLI.
//!
//! A valid first manifest must never produce positive stdout if a later
//! manifest contains forged rights, stale evidence, invalid source pins,
//! unsafe URLs, or unsupported play/adapter claims. These checks exercise
//! the compiled CLI, not only a library helper.
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

fn luanti() -> Value {
    serde_json::from_str(include_str!("../projects/luanti.json"))
        .expect("source-controlled Luanti manifest parses")
}

fn changed(mut record: Value, edit: impl FnOnce(&mut Value)) -> Value {
    edit(&mut record);
    record
}

fn write_temporary(record: &Value) -> PathBuf {
    let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-catalog-rights-negative-{}-{serial}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("unique temporary manifest");
    file.write_all(
        &serde_json::to_vec(record).expect("serialize attack manifest"),
    )
    .expect("write attack manifest");
    path
}

fn validate_batch(record: &Value) -> Output {
    let path = write_temporary(record);
    let known_good = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("projects/openra.json");
    let result = Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"))
        .arg("validate")
        .arg(known_good)
        .arg(&path)
        .output()
        .expect("invoke compiled catalog validator");
    fs::remove_file(path).expect("remove temporary manifest");
    result
}

fn assert_rejected(case: &str, record: &Value) {
    let output = validate_batch(record);
    assert!(
        !output.status.success(),
        "{case}: a forged later manifest was accepted"
    );
    assert!(
        output.stdout.is_empty(),
        "{case}: a failed batch emitted a misleading success record"
    );
    assert!(
        !output.stderr.is_empty(),
        "{case}: rejection did not explain the error"
    );
}

#[test]
fn one_valid_later_record_keeps_the_batch_usable() {
    let output = validate_batch(&luanti());
    assert!(
        output.status.success(),
        "valid two-manifest batch failed: {output:?}"
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 CLI output");
    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.contains("TYPED-BOUNDARY-ONLY"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn forged_rights_and_invalidated_provenance_reject_the_entire_batch() {
    let original = luanti();
    let attacks = [
        (
            "forged human permission",
            changed(original.clone(), |v| {
                v["permission_decisions"][0]["decision"] = json!("APPROVED_FOR_SCOPE");
            }),
        ),
        (
            "rights claim without evidence",
            changed(original.clone(), |v| {
                v["rights_claims"][0]["evidence_ids"] = json!([]);
            }),
        ),
        (
            "missing evidence identity",
            changed(original.clone(), |v| {
                v["rights_claims"][0]["evidence_ids"] = json!(["forged-source"]);
            }),
        ),
        (
            "invalidated cited evidence",
            changed(original.clone(), |v| {
                v["evidence"][0]["currentness"] = json!("INVALIDATED");
            }),
        ),
        (
            "unreviewed rights use scope",
            changed(original.clone(), |v| {
                v["permission_decisions"][0]["scope"] = json!("entire-game");
            }),
        ),
        (
            "blank permission decision reason",
            changed(original.clone(), |v| {
                v["permission_decisions"][0]["decision_reason"] = json!("  ");
            }),
        ),
        (
            "duplicated evidence identity",
            changed(original.clone(), |v| {
                v["evidence"][1]["evidence_id"] = v["evidence"][0]["evidence_id"].clone();
            }),
        ),
        (
            "claim-history self-approval",
            changed(original, |v| {
                v["review"]["claim_history"] = json!([{
                    "old_claim_id": "code-lgpl",
                    "new_claim_id": "code-lgpl",
                    "reason": "forged same-claim replacement",
                    "at": "2026-10-08T00:00:00Z"
                }]);
            }),
        ),
    ];
    for (case, attack) in attacks {
        assert_rejected(case, &attack);
    }
}

#[test]
fn invalid_upstream_and_execution_claims_reject_the_entire_batch() {
    let original = luanti();
    let attacks = [
        (
            "moving source pin",
            changed(original.clone(), |v| {
                v["upstream"]["source_revision"] = json!("main");
            }),
        ),
        (
            "forged pinned evidence",
            changed(original.clone(), |v| {
                v["evidence"][0]["commit"] = json!("HEAD");
            }),
        ),
        (
            "script URL",
            changed(original.clone(), |v| {
                v["upstream"]["contribution_url"] = json!("javascript:alert(1)");
            }),
        ),
        (
            "loopback evidence URL",
            changed(original.clone(), |v| {
                v["evidence"][0]["url"] = json!("https://127.0.0.1/private");
            }),
        ),
        (
            "unverified play marked verified",
            changed(original.clone(), |v| {
                v["play"]["status"] = json!("FREE_ENERGY_VERIFIED");
            }),
        ),
        (
            "untested adapter marked tested",
            changed(original.clone(), |v| {
                v["adapter"]["status"] = json!("TESTED");
            }),
        ),
        (
            "unsupported schema",
            changed(original, |v| {
                v["schema"] = json!("free-energy.project/v999");
            }),
        ),
    ];
    for (case, attack) in attacks {
        assert_rejected(case, &attack);
    }
}
