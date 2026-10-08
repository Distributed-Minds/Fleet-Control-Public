//! Calendar-valid RFC3339 admission regression for #60, specification v5.
//! Offline parsing does not authenticate timestamps or assert source freshness.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const BASELINE: &str = include_str!("../projects/luanti.json");

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("authored pilot JSON")
}

fn reject_at(pointer: &str, date: &str, diagnostic: &str) {
    let mut manifest = baseline();
    *manifest.pointer_mut(pointer).expect("known date field") = json!(date);
    let errors = validate_manifest(&manifest.to_string())
        .expect_err("invalid calendar/time zone must not enter the catalog");
    assert!(
        errors.iter().any(|error| error.contains(diagnostic)),
        "{pointer} with {date:?}: {errors:?}"
    );
}

#[test]
fn all_pilot_manifests_keep_valid_timestamps() {
    for manifest in [
        BASELINE,
        include_str!("../projects/openra.json"),
        include_str!("../projects/veloren.json"),
    ] {
        validate_manifest(manifest).expect("pilot timestamp remains accepted");
    }
}

#[test]
fn leap_dates_fractional_seconds_and_explicit_offsets_are_admitted() {
    for date in [
        "2024-02-29T00:00:00Z",
        "2000-02-29T23:59:59.000123Z",
        "2026-10-08T22:15:10+02:00",
        "2026-10-08T03:00:05-05:30",
        "2026-01-01T00:00:00.1Z",
    ] {
        let mut manifest = baseline();
        manifest["evidence"][0]["observed_at"] = json!(date);
        manifest["review"]["reviewed_at"] = json!(date);
        manifest["permission_decisions"][0]["decided_at"] = json!(date);
        validate_manifest(&manifest.to_string())
            .unwrap_or_else(|errors| panic!("valid date {date} incorrectly rejected: {errors:?}"));
    }
}

#[test]
fn impossible_and_ambiguous_evidence_dates_fail_closed() {
    for date in [
        "2026-02-29T00:00:00Z",
        "1900-02-29T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "0000-01-01T00:00:00Z",
        "2026-00-10T00:00:00Z",
        "2026-13-10T00:00:00Z",
        "2026-10-00T00:00:00Z",
        "2026-10-08T24:00:00Z",
        "2026-10-08T23:60:00Z",
        "2026-10-08T23:59:60Z",
        "2026-10-08T12:00:00",
        "2026-10-08T12:00:00+24:00",
        "2026-10-08T12:00:00+01:60",
        "2026-10-08T12:00:00+0100",
        "2026-10-08T12:00:00.Z",
        "2026-10-08T12:00:00Zmore",
        "2026-10-08t12:00:00Z",
        "2026-10-08T12:00:00z",
        "2026-10-08T12:00:00+01:0X",
        "2026-10-08T12:00:00Z\n",
    ] {
        reject_at(
            "/evidence/0/observed_at",
            date,
            "evidence[0].observed_at must be an RFC3339 timestamp",
        );
    }
}

#[test]
fn the_same_admission_guard_covers_all_review_and_decision_date_sinks() {
    reject_at(
        "/review/reviewed_at",
        "2026-02-30T00:00:00Z",
        "review.reviewed_at must be an RFC3339 timestamp",
    );
    let mut decision = baseline();
    decision["permission_decisions"][0]["decided_at"] = json!("yesterday");
    let errors = validate_manifest(&decision.to_string()).expect_err("invalid decision date");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("permission_decisions[0].decided_at must be an RFC3339 timestamp")),
        "{errors:?}"
    );
    let mut history = baseline();
    let old_id = history["rights_claims"][0]["claim_id"].clone();
    history["review"]["claim_history"] = json!([{
        "old_claim_id": old_id,
        "new_claim_id": null,
        "reason": "Historical claim was retracted",
        "at": "2026-02-30T00:00:00Z"
    }]);
    let errors = validate_manifest(&history.to_string()).expect_err("invalid claim history date");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("review.claim_history[0].at must be an RFC3339 timestamp")),
        "{errors:?}"
    );
}
