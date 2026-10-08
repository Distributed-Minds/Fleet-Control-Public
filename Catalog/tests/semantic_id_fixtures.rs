//! Execute all authored semantic-ID/referential-integrity fixture mutations
//! through the production typed manifest admission path (issue #60, spec 5).
//! This is not full JSON Schema 2020-12 validation or rights approval.
use free_energy_catalog::validate_manifest;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;

const BASE: &str = include_str!("../projects/luanti.json");
const FIXTURES: &str = include_str!("../fixtures/semantic-id-resolution-v0.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    op: String,
    pointer: String,
    value: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    changes: Vec<Change>,
    expected_semantic_valid: bool,
}

#[derive(Debug, Deserialize)]
struct Suite {
    cases: Vec<Case>,
}

fn execute_case(base: &str, case: &Case) -> Result<(), String> {
    let mut value: Value = serde_json::from_str(base).map_err(|e| e.to_string())?;
    for change in &case.changes {
        if change.op != "replace" || !change.pointer.starts_with('/') {
            return Err(format!("unsupported mutation: {} {}", change.op, change.pointer));
        }
        let slot = value.pointer_mut(&change.pointer).ok_or_else(|| {
            format!("{}: mutation target does not exist: {}", case.id, change.pointer)
        })?;
        *slot = change.value.clone();
    }
    let input = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    validate_manifest(&input)
        .map(|_| ())
        .map_err(|errors| errors.join("; "))
}

fn check_suite(source: &str) -> Result<(usize, usize), String> {
    let suite: Suite = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if suite.cases.len() != 13 {
        return Err(format!("expected 13 authored semantic cases, got {}", suite.cases.len()));
    }
    if validate_manifest(BASE).is_err() {
        return Err("unmodified baseline is not a valid typed manifest".to_string());
    }
    let mut seen = HashSet::new();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in &suite.cases {
        if case.id.trim().is_empty() || !seen.insert(case.id.as_str()) {
            return Err(format!("duplicate or blank fixture case ID: {}", case.id));
        }
        let result = execute_case(BASE, case);
        if case.expected_semantic_valid {
            result.map_err(|e| format!("{}: expected acceptance: {e}", case.id))?;
            accepted += 1;
        } else {
            match result {
                Ok(()) => return Err(format!("{}: unsafe mutation was accepted", case.id)),
                Err(e) if e.contains("mutation target does not exist") => {
                    return Err(format!("{}: invalid fixture mutation: {e}", case.id));
                }
                Err(_) => rejected += 1,
            }
        }
    }
    if accepted == 0 || rejected == 0 {
        return Err("fixture must test both positive and negative semantics".to_string());
    }
    Ok((accepted, rejected))
}

#[test]
fn all_thirteen_authored_semantic_id_mutations_execute() {
    assert_eq!(check_suite(FIXTURES), Ok((3, 10)));
}

#[test]
fn flipping_a_expected_verdict_does_not_make_it_true() {
    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"][0]["expected_semantic_valid"] = Value::Bool(false);
    assert!(
        check_suite(&fixture.to_string()).is_err(),
        "incorrect expected label cannot pass"
    );
    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"][3]["expected_semantic_valid"] = Value::Bool(true);
    assert!(
        check_suite(&fixture.to_string()).is_err(),
        "duplicate evidence ID cannot become valid by label"
    );
}

#[test]
fn duplicate_missing_or_unapplicable_cases_cannot_pass() {
    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"][1]["id"] = fixture["cases"][0]["id"].clone();
    assert!(check_suite(&fixture.to_string()).is_err());

    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"].as_array_mut().unwrap().pop();
    assert!(check_suite(&fixture.to_string()).is_err());

    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"][0]["changes"] = serde_json::json!([
        {"op":"replace","pointer":"/no-such-field","value":"fake"}
    ]);
    assert!(check_suite(&fixture.to_string()).is_err());

    let mut fixture: Value = serde_json::from_str(FIXTURES).unwrap();
    fixture["cases"][0]["changes"] = serde_json::json!([
        {"op":"remove","pointer":"/id","value":null}
    ]);
    assert!(check_suite(&fixture.to_string()).is_err());
}
