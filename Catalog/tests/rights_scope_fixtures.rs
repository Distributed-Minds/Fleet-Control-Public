//! Execute the existing rights-scope schema fragments against the compiled
//! Catalog typed-admission boundary (issue #60, spec v5).
//!
//! This is not legal clearance or a complete JSON Schema Draft 2020-12 engine.
//! It proves the currently implemented Rust path-safety decisions agree with
//! *every* maintained rights-scope fixture, including negative mutations.

use free_energy_catalog::validate_manifest;
use serde_json::Value;
use std::collections::HashSet;

const PILOT: &str = include_str!("../projects/luanti.json");
const FIXTURES: &str = include_str!("../fixtures/rights-path-scope-v0.json");

fn base_manifest() -> Value {
    serde_json::from_str(PILOT).expect("valid pilot manifest")
}

fn append_fixture_claim(record: &mut Value, claim: &Value) {
    let mut claim = claim.clone();
    let existing_evidence = record["evidence"][0]["evidence_id"]
        .as_str()
        .expect("pilot evidence identifier");
    // The isolated fragment uses a synthetic evidence ID which does not
    // exist in the actual pilot manifest; bind its claim to real pilot
    // evidence so unrelated foreign-reference guards do not mask path rules.
    claim["evidence_ids"] = serde_json::json!([existing_evidence]);
    record["rights_claims"]
        .as_array_mut()
        .expect("pilot rights claims")
        .push(claim);
}

#[test]
fn all_authored_rights_path_scope_vectors_execute_through_real_admission() {
    let suite: Value = serde_json::from_str(FIXTURES).expect("valid fixture JSON");
    assert_eq!(
        suite["fixture_schema"].as_str(),
        Some("free-energy.catalog-rights-scope-fragment-tests/v0")
    );
    assert_eq!(suite["target_pointer"].as_str(), Some("#/$defs/rights"));
    let cases = suite["cases"].as_array().expect("fixture case list");
    assert_eq!(cases.len(), 23, "do not silently drop authored cases");
    let mut ids = HashSet::new();
    let mut accepted = 0;
    let mut rejected = 0;

    for case in cases {
        let id = case["id"].as_str().expect("case ID");
        assert!(!id.is_empty() && ids.insert(id), "duplicate case ID: {id}");
        let expected = case["expected_valid"]
            .as_bool()
            .expect("boolean expected validity");
        let overrides = case["overrides"].as_object().expect("case overrides");
        let mut claim = suite["base_rights"].clone();
        for (field, value) in overrides {
            assert!(
                matches!(field.as_str(), "scope" | "scope_kind"),
                "unknown case override {field} in {id}"
            );
            claim[field] = value.clone();
        }
        let mut manifest = base_manifest();
        append_fixture_claim(&mut manifest, &claim);

        let verdict = validate_manifest(&manifest.to_string());
        assert_eq!(
            verdict.is_ok(),
            expected,
            "actual Rust typed admission disagreed for {id}: {:?}",
            verdict.err()
        );
        if expected {
            accepted += 1;
        } else {
            rejected += 1;
            let errors = verdict.expect_err("negative case must reject");
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("unsafe rights path scope: fixture-scope")),
                "expected a path-specific negative decision for {id}, got {errors:?}"
            );
        }
    }
    assert_eq!(ids.len(), cases.len());
    assert!(accepted >= 5, "positive coverage was lost");
    assert!(rejected >= 10, "negative coverage was lost");
}

#[test]
fn pinned_evidence_paths_do_not_bypass_rights_path_safety_rules() {
    let baseline = base_manifest();
    let first_evidence = &baseline["evidence"][0];
    assert_eq!(
        first_evidence["evidence_kind"].as_str(),
        Some("PINNED_REPOSITORY_FILE"),
        "pilot evidence kind changed; do not mask path admission"
    );
    for path in [
        "../LICENSE",
        "./Cargo.toml",
        "assets/../LICENSE",
        "assets/./README.md",
        "/etc/passwd",
        "C:/Windows/system.ini",
        "assets\\media",
        "assets//license",
        "assets/",
        " assets/LICENSE",
        "assets/ LICENSE",
        "assets/\nLICENSE",
    ] {
        let mut changed = baseline.clone();
        changed["evidence"][0]["path"] = Value::String(path.to_owned());
        let errors = validate_manifest(&changed.to_string())
            .expect_err("unsafe pinned repository path was accepted");
        assert!(
            errors
                .iter()
                .any(|e| e.contains("invalid pinned repository evidence")),
            "path {path:?} did not fail on pin provenance: {errors:?}"
        );
    }
    for path in [
        "LICENSE",
        "assets/Cargo.toml",
        "références/文件",
        "assets/audio files/track.ogg",
    ] {
        let mut changed = baseline.clone();
        changed["evidence"][0]["path"] = Value::String(path.to_owned());
        let outcome = validate_manifest(&changed.to_string());
        assert!(outcome.is_ok(), "valid evidence path {path:?}: {outcome:?}");
    }
}
