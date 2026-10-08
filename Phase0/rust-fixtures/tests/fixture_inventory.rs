//! Structural guard for historical Phase0 fixture coverage during Rust migration (#71).
//! A passing inventory is NOT semantic parity, runtime containment, or an installable fleet.

use serde_json::Value;
use std::collections::HashSet;

type Group = (&'static str, usize, &'static str);
type Family = (&'static str, &'static str, &'static [Group]);

fn families() -> [Family; 8] {
    [
        (
            "authority-closure-spec2",
            include_str!("../../fixtures/authority-closure-spec2.json"),
            &[("cases", 26, "child-after-cutoff-denied")],
        ),
        (
            "ad-hoc-research-spec1",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            &[
                ("publication_cases", 10, "complete-absent-current-authority"),
                ("identity_cases", 11, "same-semantic-retry"),
                ("source_cases", 9, "default-authority-first"),
                ("recovery_cases", 7, "cutoff-before-create"),
                ("concurrency_cases", 5, "two-empty-atomic-unique"),
                ("packet_field_cases", 5, "required-handoff-fields-preserved"),
            ],
        ),
        (
            "adaptive-stress-spec2",
            include_str!("../../fixtures/adaptive-stress-spec2.json"),
            &[("cases", 23, "fixture-authority-is-inert")],
        ),
        (
            "containment-capacity-spec2",
            include_str!("../../fixtures/containment-capacity-spec2.json"),
            &[
                ("decision_cases", 4, "backlog-cannot-renew"),
                ("workload_cases", 9, "ordinary-load"),
                ("planning_cases", 4, "rare-event-false-positive-heavy"),
            ],
        ),
        (
            "containment-spec3",
            include_str!("../../fixtures/containment-spec3.json"),
            &[
                ("decision_cases", 17, "least-harm"),
                ("recovery_cases", 5, "false-positive-recovery"),
                ("trace_cases", 13, "cutoff-retry"),
            ],
        ),
        (
            "github-capability-spec5",
            include_str!("../../fixtures/github-capability-spec5.json"),
            &[("cases", 28, "1")],
        ),
        (
            "merge-base-topology-spec2",
            include_str!("../../fixtures/merge-base-topology-spec2.json"),
            &[("$", 12, "unique-complete")],
        ),
        (
            "integration-candidate-v1",
            include_str!("../../fixtures/integration-candidate-v1.json"),
            &[
                ("cases", 4, "normal-two-parent"),
                ("stale_head_cases", 2, "target-moved"),
            ],
        ),
    ]
}

fn case_identity(case: &Value) -> Result<String, String> {
    let identity = case
        .get("id")
        .or_else(|| case.get("name"))
        .ok_or_else(|| "case is missing id/name".to_owned())?;
    let id = match identity {
        Value::String(s) if !s.trim().is_empty() => s.clone(),
        Value::Number(n) if n.as_u64().is_some_and(|x| x > 0) => n.to_string(),
        _ => return Err("case id/name must be a non-empty string or positive integer".into()),
    };
    Ok(id)
}

fn check_group(
    family: &str,
    document: &Value,
    key: &str,
    expected_count: usize,
    first_identity: &str,
) -> Result<usize, String> {
    let group = if key == "$" {
        document
    } else {
        document
            .get(key)
            .ok_or_else(|| format!("{family}: missing {key}"))?
    };
    let cases = group
        .as_array()
        .ok_or_else(|| format!("{family}/{key}: not an array"))?;
    let mut seen = HashSet::new();
    for case in cases {
        let id = case_identity(case).map_err(|e| format!("{family}/{key}: {e}"))?;
        if !seen.insert(id.clone()) {
            return Err(format!("{family}/{key}: duplicate case id {id}"));
        }
    }
    if cases.len() != expected_count {
        return Err(format!(
            "{family}/{key}: expected {expected_count} historical cases; found {}",
            cases.len()
        ));
    }
    let first = cases
        .first()
        .ok_or_else(|| format!("{family}/{key}: empty historical family"))?;
    if case_identity(first)? != first_identity {
        return Err(format!(
            "{family}/{key}: first historical case identity changed from {first_identity}"
        ));
    }
    Ok(cases.len())
}

fn validate_family(
    label: &str,
    source: &str,
    groups: &[Group],
) -> Result<usize, String> {
    let document: Value =
        serde_json::from_str(source).map_err(|e| format!("{label}: invalid JSON: {e}"))?;
    let mut total = 0;
    for &(key, count, first) in groups {
        total += check_group(label, &document, key, count, first)?;
    }
    Ok(total)
}

#[test]
fn all_eight_historical_fixture_families_retain_194_cases() {
    let mut total = 0;
    for (label, source, groups) in families() {
        total += validate_family(label, source, groups).unwrap_or_else(|error| panic!("{error}"));
    }
    assert_eq!(total, 194);
}

#[test]
fn historical_schema_and_spec_markers_remain_explicit() {
    let required = [
        ("authority-closure-spec2", "spec", serde_json::json!(2)),
        ("ad-hoc-research-spec1", "schema_version", serde_json::json!(3)),
        ("ad-hoc-research-spec1", "spec_version", serde_json::json!(1)),
        ("adaptive-stress-spec2", "spec", serde_json::json!(2)),
        ("containment-capacity-spec2", "schema_version", serde_json::json!(2)),
        ("containment-capacity-spec2", "spec_version", serde_json::json!(2)),
        ("containment-spec3", "schema_version", serde_json::json!(1)),
        ("containment-spec3", "spec_version", serde_json::json!(3)),
        ("github-capability-spec5", "spec", serde_json::json!(5)),
        ("github-capability-spec5", "issue", serde_json::json!(11)),
        (
            "integration-candidate-v1",
            "schema_version",
            serde_json::json!("integration-candidate-fixture-v1"),
        ),
        (
            "integration-candidate-v1",
            "digest",
            serde_json::json!("sha256"),
        ),
    ];
    for (label, source, _) in families() {
        let document: Value = serde_json::from_str(source).expect("historical valid JSON");
        for (expected_label, key, expected_value) in &required {
            if label == *expected_label {
                assert_eq!(
                    document.get(*key),
                    Some(expected_value),
                    "{label}: {key} marker changed"
                );
            }
        }
    }
}

#[test]
fn deleting_a_historical_case_is_a_test_failure() {
    let mut doc: Value =
        serde_json::from_str(include_str!("../../fixtures/containment-capacity-spec2.json"))
            .expect("historical valid JSON");
    doc["workload_cases"].as_array_mut().unwrap().remove(0);
    let error = check_group(
        "containment-capacity-spec2",
        &doc,
        "workload_cases",
        9,
        "ordinary-load",
    )
    .unwrap_err();
    assert!(error.contains("expected 9 historical cases"));
}

#[test]
fn duplicate_identity_is_a_test_failure_even_at_unchanged_case_count() {
    let mut doc: Value =
        serde_json::from_str(include_str!("../../fixtures/containment-capacity-spec2.json"))
            .expect("historical valid JSON");
    let first_id = doc["decision_cases"][0]["id"].clone();
    doc["decision_cases"][1]["id"] = first_id;
    let error = check_group(
        "containment-capacity-spec2",
        &doc,
        "decision_cases",
        4,
        "backlog-cannot-renew",
    )
    .unwrap_err();
    assert!(error.contains("duplicate case id"));
}

#[test]
fn missing_family_and_malformed_json_cannot_pass() {
    let mut doc: Value =
        serde_json::from_str(include_str!("../../fixtures/containment-capacity-spec2.json"))
            .expect("historical valid JSON");
    doc.as_object_mut().unwrap().remove("planning_cases");
    assert!(
        check_group(
            "containment-capacity-spec2",
            &doc,
            "planning_cases",
            4,
            "rare-event-false-positive-heavy"
        )
        .is_err()
    );
    assert!(validate_family("broken", "{invalid", &[("cases", 1, "x")]).is_err());
}

#[test]
fn original_legacy_checker_sources_remain_available_until_parity_migration() {
    // Deliberate migration tripwire: retire/update this assertion only when
    // independent per-family Rust parity is proven and original history preserved.
    let sources = [
        include_str!("../../scripts/authority_closure_model.py"),
        include_str!("../../scripts/check-authority-closure-fixtures.py"),
        include_str!("../../scripts/check-ad-hoc-research-fixtures.py"),
        include_str!("../../scripts/check-adaptive-stress-fixtures.py"),
        include_str!("../../scripts/check-containment-capacity-fixtures.py"),
        include_str!("../../scripts/check-containment-fixtures.py"),
        include_str!("../../scripts/check-github-capability-fixtures.py"),
        include_str!("../../scripts/check-merge-base-topology-fixtures.py"),
        include_str!("../../../scripts/check-integration-candidate-fixtures.py"),
    ];
    assert_eq!(sources.len(), 9);
    assert!(sources.iter().all(|source| source.len() > 100));
}
