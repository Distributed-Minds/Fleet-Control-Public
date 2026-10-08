//! Published v5 schema-vs-Rust admission conformance for identifier syntax
//! and unique per-claim evidence references. Uses the real manifest validator.
//! These are offline shape tests, not proof of rights or evidence authenticity.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");
const VELOREN: &str = include_str!("../projects/veloren.json");

fn changed(f: impl FnOnce(&mut Value)) -> String {
    let mut original: Value = serde_json::from_str(LUANTI).expect("valid pilot fixture");
    f(&mut original);
    serde_json::to_string(&original).expect("serialize isolated mutation")
}

fn assert_rejected_with(content: &str, expected: &str) {
    let errors = validate_manifest(content).expect_err("invalid manifest must not be admitted");
    assert!(
        errors.iter().any(|problem| problem.contains(expected)),
        "missing specific admission failure {expected:?}: {errors:?}"
    );
}

#[test]
fn original_pilots_remain_valid() {
    for source in [LUANTI, OPENRA, VELOREN] {
        assert!(validate_manifest(source).is_ok());
    }
}

#[test]
fn claim_id_follows_schema_slug_not_arbitrary_serde_string() {
    for invalid in ["", "2start", "Bad", "unicode-é", "space in id", "has/slash"] {
        let item = changed(|project| project["rights_claims"][0]["claim_id"] = json!(invalid));
        assert_rejected_with(&item, "invalid claim ID slug");
    }

    for valid in ["legal_1", "source-code", "x", "z9"] {
        let item = changed(|project| project["rights_claims"][0]["claim_id"] = json!(valid));
        assert!(
            validate_manifest(&item).is_ok(),
            "published schema slug should admit {valid:?}"
        );
    }
}

#[test]
fn evidence_identity_is_checked_even_when_all_references_are_repaired() {
    for invalid in ["2source", "Bad-Source", "evidence/path", "space id"] {
        let content = changed(|project| {
            let old = project["evidence"][0]["evidence_id"]
                .as_str()
                .expect("original evidence id")
                .to_owned();
            project["evidence"][0]["evidence_id"] = json!(invalid);
            for claim in project["rights_claims"].as_array_mut().expect("rights claims") {
                for id in claim["evidence_ids"].as_array_mut().expect("claim evidence refs") {
                    if id.as_str() == Some(old.as_str()) {
                        *id = json!(invalid);
                    }
                }
            }
        });
        assert_rejected_with(&content, "invalid evidence ID slug");
    }
}

#[test]
fn duplicate_evidence_within_a_claim_is_rejected_separately_from_duplicate_records() {
    let content = changed(|project| {
        let existing = project["rights_claims"][0]["evidence_ids"][0].clone();
        project["rights_claims"][0]["evidence_ids"]
            .as_array_mut()
            .expect("evidence list")
            .push(existing);
    });
    assert_rejected_with(&content, "duplicate evidence reference in claim");
}

#[test]
fn duplicate_evidence_within_a_permission_is_rejected() {
    let content = changed(|project| {
        let id = project["evidence"][0]["evidence_id"].clone();
        project["permission_decisions"][0]["review_evidence_ids"] = json!([id, id]);
    });
    assert_rejected_with(&content, "duplicate permission evidence reference");

    // An ordinary single reference is structurally allowed for REVIEW_REQUIRED.
    let content = changed(|project| {
        let id = project["evidence"][0]["evidence_id"].clone();
        project["permission_decisions"][0]["review_evidence_ids"] = json!([id]);
    });
    assert!(validate_manifest(&content).is_ok());
}
