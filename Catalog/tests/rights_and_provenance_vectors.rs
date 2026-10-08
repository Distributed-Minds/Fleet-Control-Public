//! Execute authored rights-path and pinned-evidence fragments against the
//! complete typed catalog admission boundary (#60 spec v5).
//!
//! 50 source-controlled vector expectations become runnable Rust regressions.
//! These are NOT Draft 2020-12 schema-fragment conformance, source existence,
//! human rights approval, or actual distribution/play verification.

use serde_json::{json, Value};
use std::collections::HashSet;

const LUANTI: &str = include_str!("../projects/luanti.json");
const RIGHTS: &str = include_str!("../fixtures/rights-path-scope-v0.json");
const EVIDENCE: &str = include_str!("../fixtures/pinned-evidence-structure-v0.json");

fn full_manifest() -> Value {
    serde_json::from_str(LUANTI).expect("checked-in positive pilot")
}

fn parse(source: &str) -> Value {
    serde_json::from_str(source).expect("source-controlled authored vectors")
}

fn cases<'a>(suite: &'a Value, schema: &str, count: usize) -> &'a [Value] {
    assert_eq!(suite["fixture_schema"].as_str(), Some(schema));
    let cases = suite["cases"].as_array().expect("authored cases array");
    assert_eq!(cases.len(), count, "unexpected authored fixture count");
    let mut ids = HashSet::new();
    for case in cases {
        let id = case["id"].as_str().expect("case ID");
        assert!(!id.is_empty() && ids.insert(id), "duplicate/blank case ID");
        assert!(case["expected_valid"].is_boolean(), "missing expected verdict");
        assert!(case["overrides"].is_object(), "missing overrides map");
    }
    cases
}

fn apply_overrides(target: &mut Value, overrides: &Value) {
    for (key, value) in overrides.as_object().expect("overrides object") {
        let field = target.get_mut(key).expect("override known fragment key");
        *field = value.clone();
    }
}

fn check_rights(suite: &Value) -> Result<(), String> {
    assert_eq!(suite["target_pointer"].as_str(), Some("#/$defs/rights"));
    for case in cases(
        suite,
        "free-energy.catalog-rights-scope-fragment-tests/v0",
        23,
    ) {
        let mut manifest = full_manifest();
        let mut claim = suite["base_rights"].clone();
        apply_overrides(&mut claim, &case["overrides"]);
        // A synthetic claim must cite a real record in this local fixture;
        // the source fragment's placeholder must never masquerade as a pin.
        claim["evidence_ids"] = json!([
            manifest["evidence"][0]["evidence_id"]
                .as_str()
                .expect("existing evidence identity")
        ]);
        manifest["rights_claims"]
            .as_array_mut()
            .expect("rights claims")
            .push(claim);
        let actual = free_energy_catalog::validate_manifest(&manifest.to_string()).is_ok();
        let expected = case["expected_valid"].as_bool().expect("expected verdict");
        if actual != expected {
            return Err(format!(
                "rights path {}: expected valid={expected}, observed {actual}",
                case["id"].as_str().expect("case ID")
            ));
        }
    }
    Ok(())
}

fn check_evidence(suite: &Value) -> Result<(), String> {
    assert_eq!(suite["target_pointer"].as_str(), Some("#/$defs/evidence"));
    for case in cases(
        suite,
        "free-energy.catalog-evidence-fragment-tests/v0",
        27,
    ) {
        let mut manifest = full_manifest();
        let mut additional = suite["base_evidence"].clone();
        apply_overrides(&mut additional, &case["overrides"]);
        // An unreferenced supplemental evidence record remains traceable even
        // when INVALIDATED. This does not approve any rights claim.
        manifest["evidence"]
            .as_array_mut()
            .expect("evidence records")
            .push(additional);
        let actual = free_energy_catalog::validate_manifest(&manifest.to_string()).is_ok();
        let expected = case["expected_valid"].as_bool().expect("expected verdict");
        if actual != expected {
            return Err(format!(
                "pinned evidence {}: expected valid={expected}, observed {actual}",
                case["id"].as_str().expect("case ID")
            ));
        }
    }
    Ok(())
}

#[test]
fn all_23_authored_rights_scope_vectors_agree_with_typed_admission() {
    if let Err(error) = check_rights(&parse(RIGHTS)) {
        panic!("{error}");
    }
}

#[test]
fn all_27_authored_pinned_evidence_vectors_agree_with_typed_admission() {
    if let Err(error) = check_evidence(&parse(EVIDENCE)) {
        panic!("{error}");
    }
}

#[test]
fn tampering_a_historical_expected_verdict_cannot_manufacture_success() {
    let mut rights = parse(RIGHTS);
    rights["cases"][0]["expected_valid"] = json!(false);
    assert!(check_rights(&rights).is_err(), "forged rights verdict accepted");

    let mut evidence = parse(EVIDENCE);
    evidence["cases"][0]["expected_valid"] = json!(false);
    assert!(
        check_evidence(&evidence).is_err(),
        "forged evidence verdict accepted"
    );
}

#[test]
fn removing_authored_cases_cannot_manufacture_completeness() {
    for source in [RIGHTS, EVIDENCE] {
        let mut fixture = parse(source);
        fixture["cases"].as_array_mut().expect("case array").pop();
        let rejected = std::panic::catch_unwind(|| {
            if source == RIGHTS {
                let _ = check_rights(&fixture);
            } else {
                let _ = check_evidence(&fixture);
            }
        });
        assert!(rejected.is_err(), "incomplete historical suite accepted");
    }
}
