//! Schema-v5 object presence is not the same as Rust Option<T> nullability.
//! All published required keys must be present, including explicit JSON nulls.
//! This checks structural admission, not human authority or rights clearance.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");

fn baseline() -> Value {
    serde_json::from_str(LUANTI).expect("pinned pilot fixture")
}

#[test]
fn three_existing_pilots_preserve_valid_explicit_nullable_keys() {
    for pilot in [
        LUANTI,
        include_str!("../projects/openra.json"),
        include_str!("../projects/veloren.json"),
    ] {
        validate_manifest(pilot).expect("complete pinned catalog pilot");
    }
}

#[test]
fn every_required_nullable_property_rejects_absence_at_its_own_path() {
    for (pointer, key, expected) in [
        ("/upstream", "source_revision", "upstream.source_revision"),
        (
            "/upstream",
            "source_revision_reason",
            "upstream.source_revision_reason",
        ),
        (
            "/play",
            "upstream_download_url",
            "play.upstream_download_url",
        ),
        (
            "/play",
            "local_test_evidence_id",
            "play.local_test_evidence_id",
        ),
        ("/adapter", "target_id", "adapter.target_id"),
        ("/adapter", "test_evidence_id", "adapter.test_evidence_id"),
        (
            "/review",
            "supersedes_record_revision",
            "review.supersedes_record_revision",
        ),
        ("/rights_claims/0", "license_id", "rights_claims[0].license_id"),
        (
            "/permission_decisions/0",
            "decided_at",
            "permission_decisions[0].decided_at",
        ),
        (
            "/permission_decisions/0",
            "reviewer",
            "permission_decisions[0].reviewer",
        ),
        ("/evidence/0", "repository", "evidence[0].repository"),
        ("/evidence/0", "commit", "evidence[0].commit"),
        ("/evidence/0", "path", "evidence[0].path"),
    ] {
        let mut record = baseline();
        let object = record
            .pointer_mut(pointer)
            .expect("known pilot object")
            .as_object_mut()
            .expect("nested JSON object");
        object.remove(key).expect("schema-required key existed");
        let errors =
            validate_manifest(&record.to_string()).expect_err("missing nullable key must reject");
        assert!(
            errors.iter().any(|error| {
                error == &format!("{expected}: missing schema-required field")
            }),
            "omitted {expected} bypassed the schema key-presence gate: {errors:?}"
        );
    }
}

#[test]
fn missing_nullable_claim_history_successor_is_detected() {
    let mut record = baseline();
    let original_claim = record["rights_claims"][0]["claim_id"].clone();
    record["review"]["claim_history"] = json!([{
        "old_claim_id": original_claim,
        "new_claim_id": null,
        "reason": "Historical replacement event",
        "at": "2026-10-08T00:00:00Z"
    }]);
    // Missing key is not equivalent to the explicit null above.
    record["review"]["claim_history"][0]
        .as_object_mut()
        .expect("event object")
        .remove("new_claim_id");
    let errors =
        validate_manifest(&record.to_string()).expect_err("incomplete event must reject");
    assert!(
        errors.iter().any(|error| error
            == "review.claim_history[0].new_claim_id: missing schema-required field"),
        "missing historical successor key not detected: {errors:?}"
    );
}
