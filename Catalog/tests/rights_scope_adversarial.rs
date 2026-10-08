//! Adversarial rights/provenance admission regressions for the typed catalog.
//! These assert fail-closed candidate admission, not authenticated licensing approval.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");

fn baseline(source: &str) -> Value {
    serde_json::from_str(source).expect("reviewable source fixture")
}

fn rejected(value: &Value, fragment: &str) {
    let result = validate_manifest(&value.to_string());
    let errors = result.expect_err("adversarial candidate must be denied");
    assert!(
        errors.iter().any(|error| error.contains(fragment)),
        "missing {fragment:?} diagnosis: {errors:?}"
    );
}

#[test]
fn untouched_pilot_records_remain_admissible_at_the_typed_boundary() {
    for source in [LUANTI, OPENRA] {
        validate_manifest(source).expect("existing pilot metadata should parse");
    }
}

#[test]
fn rights_path_scope_must_not_escape_the_declared_content_tree() {
    for unsafe_path in [
        "../outside",
        "/etc/passwd",
        "src/../private",
        "./asset.png",
        "dir//file",
        "src\\secrets",
        "",
    ] {
        let mut candidate = baseline(LUANTI);
        candidate["rights_claims"][0]["scope_kind"] = json!("PATH");
        candidate["rights_claims"][0]["scope"] = json!(unsafe_path);
        rejected(&candidate, "unsafe rights path scope");
    }
}

#[test]
fn pinned_provenance_path_and_commit_must_identify_a_safe_immutable_file() {
    for path in ["../LICENSE.txt", "/LICENSE.txt", "./LICENSE.txt", "dir//file", "src\\LICENSE"] {
        let mut candidate = baseline(LUANTI);
        candidate["evidence"][0]["path"] = json!(path);
        rejected(&candidate, "invalid pinned repository evidence");
    }
    for revision in ["main", "deadbeef", "ABCDEF0123456789012345678901234567890123"] {
        let mut candidate = baseline(LUANTI);
        candidate["evidence"][0]["commit"] = json!(revision);
        rejected(&candidate, "invalid pinned repository evidence");
    }
    let mut no_repository = baseline(LUANTI);
    no_repository["evidence"][0]["repository"] = Value::Null;
    rejected(&no_repository, "invalid pinned repository evidence");
}

#[test]
fn invalidated_evidence_cannot_support_a_current_rights_claim() {
    let mut candidate = baseline(LUANTI);
    candidate["evidence"][0]["currentness"] = json!("INVALIDATED");
    rejected(&candidate, "references invalidated evidence");
}

#[test]
fn duplicate_rights_identity_and_missing_evidence_are_not_accepted() {
    let mut duplicate = baseline(LUANTI);
    duplicate["rights_claims"][1]["claim_id"] = duplicate["rights_claims"][0]["claim_id"].clone();
    rejected(&duplicate, "duplicate rights claim ID");

    let mut forged_link = baseline(LUANTI);
    forged_link["rights_claims"][0]["evidence_ids"] = json!(["fabricated-consent"]);
    rejected(&forged_link, "unknown evidence fabricated-consent");
}

#[test]
fn permission_cannot_reference_an_unclaimed_scope_or_unproved_review() {
    let mut wrong_scope = baseline(LUANTI);
    wrong_scope["permission_decisions"][0]["scope"] = json!("private-game-assets");
    rejected(&wrong_scope, "permission scope has no rights claim");

    let mut nonexistent_review = baseline(LUANTI);
    nonexistent_review["permission_decisions"][0]["review_evidence_ids"] =
        json!(["imaginary-legal-approval"]);
    rejected(&nonexistent_review, "permission references missing evidence");

    let mut forged_approval = baseline(LUANTI);
    forged_approval["permission_decisions"][0]["decision"] = json!("APPROVED_FOR_SCOPE");
    rejected(&forged_approval, "JSON/typed manifest");
}

#[test]
fn duplicate_permission_for_the_same_action_is_rejected() {
    let mut duplicate = baseline(LUANTI);
    let first = duplicate["permission_decisions"][0].clone();
    duplicate["permission_decisions"]
        .as_array_mut()
        .expect("permission decisions")
        .push(first);
    rejected(&duplicate, "duplicate permission action");
}

#[test]
fn metadata_cannot_claim_local_play_or_adapter_conformance() {
    let mut play = baseline(OPENRA);
    play["play"]["status"] = json!("FREE_ENERGY_VERIFIED");
    rejected(
        &play,
        "FREE_ENERGY_VERIFIED requires a separately implemented artifact-verification contract",
    );

    let mut adapter = baseline(LUANTI);
    adapter["adapter"]["status"] = json!("TESTED");
    rejected(
        &adapter,
        "TESTED adapter requires a separately implemented conformance contract",
    );
}
