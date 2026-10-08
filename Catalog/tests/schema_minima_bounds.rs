//! Independent negative controls for schema-v0 minimum collection sizes and
//! maximum external URL length. This is a subset of Draft 2020-12, not full
//! schema validation or any human rights/provenance approval.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");
const VELOREN: &str = include_str!("../projects/veloren.json");

fn baseline() -> Value {
    serde_json::from_str(LUANTI).expect("pinned pilot fixture")
}

fn rejected(fixture: &Value, expected: &str) {
    let errors = validate_manifest(&fixture.to_string()).expect_err("must reject schema violation");
    assert!(
        errors.iter().any(|problem| problem.contains(expected)),
        "missing {expected:?} in {errors:?}"
    );
}

#[test]
fn three_unmodified_pilots_still_pass_the_typed_boundary() {
    for pilot in [LUANTI, OPENRA, VELOREN] {
        validate_manifest(pilot).expect("existing draft catalog project");
    }
}

#[test]
fn every_schema_required_top_level_collection_rejects_empty_array() {
    for field in ["rights_claims", "permission_decisions", "evidence"] {
        let mut fixture = baseline();
        fixture[field] = json!([]);
        rejected(&fixture, &format!("{field} requires at least one item"));
    }
    let mut all_empty = baseline();
    for field in ["rights_claims", "permission_decisions", "evidence"] {
        all_empty[field] = json!([]);
    }
    let errors = validate_manifest(&all_empty.to_string()).expect_err("empty record");
    for field in ["rights_claims", "permission_decisions", "evidence"] {
        assert!(
            errors.iter().any(|problem| problem.contains(field)),
            "missing {field} diagnostic: {errors:?}"
        );
    }
}

#[test]
fn every_validated_external_url_obeys_schema_maximum_length() {
    // Repetition stays in the path, so the hostname itself remains plausible.
    let valid_exact = format!("https://example.org/{}", "x".repeat(2048 - 20));
    let invalid_over = format!("{valid_exact}x");
    assert_eq!(valid_exact.chars().count(), 2048);
    assert_eq!(invalid_over.chars().count(), 2049);

    let mut exact = baseline();
    exact["upstream"]["discovery_url"] = json!(valid_exact);
    validate_manifest(&exact.to_string()).expect("exact schema URL limit should pass");

    for pointer in [
        "/upstream/discovery_url",
        "/upstream/contribution_url",
        "/upstream/issue_url",
        "/upstream/canonical_source_url",
        "/play/upstream_download_url",
        "/evidence/0/url",
    ] {
        let mut fixture = baseline();
        *fixture.pointer_mut(pointer).expect("existing target") = json!(invalid_over);
        rejected(&fixture, "inadmissible external URL");
    }
}

#[test]
fn valid_schema_slugs_remain_accepted_at_typed_admission() {
    for slug in ["legal_1", "source-code", "x", "z9"] {
        let mut fixture = baseline();
        fixture["rights_claims"][0]["claim_id"] = json!(slug);
        validate_manifest(&fixture.to_string())
            .unwrap_or_else(|errors| panic!("valid schema slug {slug:?} rejected: {errors:?}"));
    }
}

#[test]
fn spoofed_evidence_identity_is_rejected_after_all_references_are_repaired() {
    // A broken cross-reference alone would not prove syntax enforcement. Keep
    // the evidence graph internally coherent while making its identifier invalid.
    let mut fixture = baseline();
    let original = fixture["evidence"][0]["evidence_id"]
        .as_str()
        .expect("pilot evidence identifier")
        .to_owned();
    let forged = "Bad/ID";
    fixture["evidence"][0]["evidence_id"] = json!(forged);
    for claim in fixture["rights_claims"]
        .as_array_mut()
        .expect("rights claims")
    {
        for id in claim["evidence_ids"]
            .as_array_mut()
            .expect("claim evidence references")
        {
            if id.as_str() == Some(original.as_str()) {
                *id = json!(forged);
            }
        }
    }
    for permission in fixture["permission_decisions"]
        .as_array_mut()
        .expect("permission decisions")
    {
        for id in permission["review_evidence_ids"]
            .as_array_mut()
            .expect("permission evidence references")
        {
            if id.as_str() == Some(original.as_str()) {
                *id = json!(forged);
            }
        }
    }
    rejected(&fixture, "evidence.evidence_id: invalid slug identifier");
    rejected(
        &fixture,
        "rights_claims.evidence_ids: invalid slug identifier",
    );
}
