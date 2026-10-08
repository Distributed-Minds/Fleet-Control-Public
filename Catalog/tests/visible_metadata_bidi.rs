//! Adversarial admission checks for Unicode directional spoofing in human-
//! visible catalog metadata (issue #60, spec v5). Does not authenticate rights.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");

fn pilot() -> Value {
    serde_json::from_str(LUANTI).expect("checked-in valid pilot")
}

#[test]
fn all_supported_directional_controls_are_rejected_in_display_name() {
    assert!(validate_manifest(LUANTI).is_ok());
    for control in [
        '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}',
        '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}',
        '\u{2068}', '\u{2069}',
    ] {
        let mut record = pilot();
        record["display_name"] = json!(format!("Luanti{control} verified"));
        let errors = validate_manifest(&record.to_string())
            .expect_err("directional spoof must be rejected");
        assert!(
            errors.iter().any(|e| e.contains("display_name: bidirectional formatting control")),
            "control U+{:04X} was not rejected as visible metadata: {errors:?}",
            control as u32
        );
    }
}

#[test]
fn right_claim_permission_evidence_and_review_text_cannot_spoof_display() {
    for pointer in [
        "/display_name",
        "/upstream/source_revision_reason",
        "/rights_claims/0/scope",
        "/rights_claims/0/statement",
        "/rights_claims/0/license_id",
        "/rights_claims/0/exceptions_or_restrictions/0",
        "/permission_decisions/0/scope",
        "/permission_decisions/0/decision_reason",
        "/evidence/0/subject_scope",
        "/evidence/0/reviewer",
        "/evidence/0/observed_at",
        "/review/reviewer",
        "/review/reviewed_at",
    ] {
        let mut record = pilot();
        let field = record.pointer_mut(pointer).expect("pilot field");
        *field = json!("trusted\u{202e}denied");
        let errors = validate_manifest(&record.to_string())
            .expect_err("bidi metadata must fail closed");
        assert!(
            errors.iter().any(|e| e.contains("bidirectional formatting control")),
            "{pointer} bypassed displayed-metadata guard: {errors:?}"
        );
    }
}

#[test]
fn optional_and_nested_text_cannot_bypass_the_guard() {
    let mut record = pilot();
    record["permission_decisions"][0]["reviewer"] = json!("forged\u{2066}authority");
    let errors = validate_manifest(&record.to_string()).expect_err("invalid reviewer");
    assert!(errors.iter().any(|e| e.contains("permission_decisions[0].reviewer: bidirectional")));

    let mut record = pilot();
    record["review"]["claim_history"] = json!([{
        "old_claim_id": record["rights_claims"][0]["claim_id"],
        "new_claim_id": null,
        "reason": "historical\u{202e}reversal",
        "at": "2026-10-08T00:00:00Z"
    }]);
    let errors = validate_manifest(&record.to_string()).expect_err("invalid claim history");
    assert!(errors.iter().any(|e| e.contains("review.claim_history[0].reason: bidirectional")));
}

#[test]
fn ordinary_international_text_remains_admissible() {
    let mut record = pilot();
    record["display_name"] = json!("Luanti — édition française 日本語");
    record["rights_claims"][0]["statement"] = json!("Source du moteur — résumé: 使用可能");
    assert!(validate_manifest(&record.to_string()).is_ok());
}
