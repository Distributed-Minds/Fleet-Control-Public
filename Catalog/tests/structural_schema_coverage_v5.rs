//! Schema-v5 multiplicity and identifier regressions for the typed Rust boundary.
//! This does not claim full JSON Schema 2020-12 validation or authenticated rights.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const BASE: &str = include_str!("../projects/luanti.json");

fn changed(edit: impl FnOnce(&mut Value)) -> Value {
    let mut v: Value = serde_json::from_str(BASE).expect("valid source pilot");
    edit(&mut v);
    v
}

fn deny(record: &Value, expected: &str) {
    let errors = validate_manifest(&record.to_string())
        .expect_err("invalid v5 structural constraint must reject");
    assert!(
        errors.iter().any(|error| error.contains(expected)),
        "missing {expected:?} in {errors:?}"
    );
}

#[test]
fn three_pilots_still_admit() {
    for json in [
        BASE,
        include_str!("../projects/openra.json"),
        include_str!("../projects/veloren.json"),
    ] {
        validate_manifest(json).expect("unchanged pilot");
    }
}

#[test]
fn required_arrays_cannot_be_empty() {
    for (field, diagnostic) in [
        ("rights_claims", "rights_claims requires at least one item"),
        (
            "permission_decisions",
            "permission_decisions requires at least one item",
        ),
        ("evidence", "evidence requires at least one item"),
    ] {
        let record = changed(|v| v[field] = json!([]));
        deny(&record, diagnostic);
    }
}

#[test]
fn mirror_and_evidence_reference_sets_are_unique() {
    let mirrors = changed(|v| {
        let url = v["upstream"]["canonical_source_url"].clone();
        v["upstream"]["read_only_mirror_urls"] = json!([url.clone(), url]);
    });
    deny(&mirrors, "duplicate upstream read-only mirror URL");

    let rights = changed(|v| {
        let id = v["rights_claims"][0]["evidence_ids"][0].clone();
        v["rights_claims"][0]["evidence_ids"] = json!([id.clone(), id]);
    });
    deny(&rights, "duplicates evidence reference");

    let permission = changed(|v| {
        let id = v["evidence"][0]["evidence_id"].clone();
        v["permission_decisions"][0]["review_evidence_ids"] = json!([id.clone(), id]);
    });
    deny(
        &permission,
        "duplicate permission review evidence reference",
    );
}

#[test]
fn all_identity_surfaces_use_the_published_slug_grammar() {
    for (pointer, value, expected) in [
        (
            "/rights_claims/0/claim_id",
            "Uppercase",
            "rights_claims.claim_id",
        ),
        (
            "/evidence/0/evidence_id",
            "0starts-with-digit",
            "evidence.evidence_id",
        ),
        (
            "/rights_claims/0/evidence_ids/0",
            "bad/segment",
            "rights_claims.evidence_ids",
        ),
        (
            "/adapter/test_evidence_id",
            "space is invalid",
            "adapter.test_evidence_id",
        ),
        (
            "/play/local_test_evidence_id",
            "bad.id",
            "play.local_test_evidence_id",
        ),
    ] {
        let record = changed(|v| {
            *v.pointer_mut(pointer).expect("existing pilot field") = json!(value);
        });
        deny(&record, "invalid slug identifier");
        deny(&record, expected);
    }

    let history = changed(|v| {
        v["review"]["claim_history"] = json!([{
            "old_claim_id": "bad/id",
            "new_claim_id": "BadID",
            "reason": "Adversarial fixture",
            "at": "2026-10-08T00:00:00Z"
        }]);
    });
    deny(&history, "review.claim_history.old_claim_id");
    deny(&history, "review.claim_history.new_claim_id");
}

#[test]
fn link_admission_respects_schema_maximum_length() {
    let record = changed(|v| {
        v["upstream"]["contribution_url"] =
            json!(format!("https://github.com/{}", "a".repeat(2049)));
    });
    deny(
        &record,
        "inadmissible external URL: upstream.contribution_url",
    );
}
