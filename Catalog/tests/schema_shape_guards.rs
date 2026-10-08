//! Executable structural boundary regressions for catalog specification v5 (#60).
//! These cover checked-in string minima and display-name length, NOT complete
//! Draft 2020-12 format validation or human reviewer authentication.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const BASELINE: &str = include_str!("../projects/luanti.json");

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("valid Luanti pilot JSON")
}

fn deny(value: &Value, fragment: &str) {
    let errors = validate_manifest(&value.to_string())
        .expect_err("invalid manifest shape must not pass typed admission");
    assert!(
        errors.iter().any(|error| error.contains(fragment)),
        "missing shape diagnostic {fragment:?}: {errors:?}"
    );
}

#[test]
fn valid_three_pilots_remain_accepted_after_shape_enforcement() {
    for source in [
        BASELINE,
        include_str!("../projects/openra.json"),
        include_str!("../projects/veloren.json"),
    ] {
        validate_manifest(source).expect("authored pilot remains valid");
    }
}

#[test]
fn display_name_length_is_codepoint_bounded_and_blank_names_fail() {
    for name in ["", "  ", "\n"] {
        let mut record = baseline();
        record["display_name"] = json!(name);
        deny(&record, "display_name must contain");
    }
    let mut record = baseline();
    record["display_name"] = json!("é".repeat(140));
    validate_manifest(&record.to_string()).expect("140 Unicode scalar values accepted");
    record["display_name"] = json!("é".repeat(141));
    deny(&record, "display_name must contain");
}

#[test]
fn rights_claims_require_actual_scopes_statements_and_exception_text() {
    for (pointer, diagnostic) in [
        (
            "/rights_claims/0/scope",
            "rights_claims[0].scope must be nonblank",
        ),
        (
            "/rights_claims/0/statement",
            "rights_claims[0].statement must be nonblank",
        ),
    ] {
        let mut record = baseline();
        *record.pointer_mut(pointer).expect("pilot field") = json!("   ");
        deny(&record, diagnostic);
    }

    let mut record = baseline();
    record["rights_claims"][0]["exceptions_or_restrictions"] = json!([""]);
    deny(
        &record,
        "rights_claims[0].exceptions_or_restrictions[0] must be nonblank",
    );
}

#[test]
fn evidence_and_review_metadata_cannot_be_empty() {
    for (pointer, diagnostic) in [
        (
            "/evidence/0/subject_scope",
            "evidence[0].subject_scope must be nonblank",
        ),
        (
            "/evidence/0/reviewer",
            "evidence[0].reviewer must be nonblank",
        ),
        (
            "/evidence/0/observed_at",
            "evidence[0].observed_at must be nonblank",
        ),
        ("/review/reviewer", "review.reviewer must be nonblank"),
        ("/review/reviewed_at", "review.reviewed_at must be nonblank"),
    ] {
        let mut record = baseline();
        *record.pointer_mut(pointer).expect("pilot field") = json!(" ");
        deny(&record, diagnostic);
    }
}

#[test]
fn empty_play_requirements_and_claim_history_metadata_fail_closed() {
    let mut record = baseline();
    record["play"]["content_requirements"] = json!([""]);
    deny(&record, "play.content_requirements[0] must be nonblank");

    let claim_id = baseline()["rights_claims"][0]["claim_id"].clone();
    for (field, diagnostic) in [
        ("reason", "review.claim_history[0].reason must be nonblank"),
        ("at", "review.claim_history[0].at must be nonblank"),
    ] {
        let mut record = baseline();
        record["review"]["claim_history"] = json!([{
            "old_claim_id": claim_id,
            "new_claim_id": null,
            "reason": "Revised claim",
            "at": "2026-10-08T00:00:00Z"
        }]);
        record["review"]["claim_history"][0][field] = json!(" ");
        deny(&record, diagnostic);
    }
}
