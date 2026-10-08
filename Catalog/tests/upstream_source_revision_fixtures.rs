//! Run all authored upstream source-pin/reason fragments against the compiled
//! Catalog typed admission boundary, rather than treating JSON fixture labels
//! as proof of semantic enforcement. No upstream fetch or rights authorization.
use free_energy_catalog::validate_manifest;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashSet;

const MANIFEST: &str = include_str!("../projects/luanti.json");
const FIXTURES: &str = include_str!("../fixtures/unpinned-upstream-reason-v0.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    overrides: Map<String, Value>,
    expected_valid: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    fixture_schema: String,
    target_schema: String,
    target_pointer: String,
    description: String,
    base_upstream: Value,
    cases: Vec<Case>,
}

fn run_case(base: &Value, case: &Case) -> Result<bool, String> {
    let mut manifest: Value = serde_json::from_str(MANIFEST).map_err(|error| error.to_string())?;
    let mut upstream = base.clone();
    let fields = upstream
        .as_object_mut()
        .ok_or_else(|| "base_upstream must be an object".to_string())?;

    for (key, value) in &case.overrides {
        let Some(slot) = fields.get_mut(key) else {
            return Err(format!("{}: unknown upstream override: {key}", case.id));
        };
        *slot = value.clone();
    }
    manifest["upstream"] = upstream;

    let manifest_json = serde_json::to_string(&manifest).map_err(|error| error.to_string())?;
    Ok(validate_manifest(&manifest_json).is_ok())
}

fn check_suite(source: &str) -> Result<(usize, usize), String> {
    let suite: Suite = serde_json::from_str(source).map_err(|error| error.to_string())?;
    if suite.fixture_schema != "free-energy.catalog-upstream-fragment-tests/v0"
        || suite.target_schema != "Catalog/schema/project-v0.schema.json"
        || suite.target_pointer != "#/$defs/upstream"
        || suite.description.trim().is_empty()
    {
        return Err("unsupported or incomplete fixture contract".to_string());
    }
    if suite.cases.len() != 8 {
        return Err(format!(
            "expected 8 upstream cases, got {}",
            suite.cases.len()
        ));
    }
    if !suite.base_upstream.is_object() {
        return Err("base upstream must be an object".to_string());
    }

    let mut ids = HashSet::new();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in &suite.cases {
        if case.id.trim().is_empty() || !ids.insert(case.id.as_str()) {
            return Err(format!("duplicate or blank fixture identity: {}", case.id));
        }
        let actual = run_case(&suite.base_upstream, case)?;
        if actual != case.expected_valid {
            return Err(format!(
                "{}: computed admission {actual} differs from authored expectation {}",
                case.id, case.expected_valid
            ));
        }
        if actual {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    if accepted == 0 || rejected == 0 {
        return Err("fixture has no positive or negative controls".to_string());
    }
    Ok((accepted, rejected))
}

#[test]
fn all_eight_authored_upstream_fragments_execute() {
    assert_eq!(check_suite(FIXTURES), Ok((3, 5)));
}

#[test]
fn falsified_verdicts_cannot_create_success() {
    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["cases"][0]["expected_valid"] = Value::Bool(false);
    assert!(check_suite(&case.to_string()).is_err());

    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["cases"][1]["expected_valid"] = Value::Bool(true);
    assert!(check_suite(&case.to_string()).is_err());
}

#[test]
fn corrupt_identity_contract_or_mutation_path_is_rejected() {
    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["cases"][1]["id"] = case["cases"][0]["id"].clone();
    assert!(check_suite(&case.to_string()).is_err());

    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["cases"].as_array_mut().unwrap().pop();
    assert!(check_suite(&case.to_string()).is_err());

    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["target_pointer"] = Value::String("#/$defs/rights".to_string());
    assert!(check_suite(&case.to_string()).is_err());

    let mut case: Value = serde_json::from_str(FIXTURES).unwrap();
    case["cases"][0]["overrides"]["unrecognized_source_authority"] = Value::Bool(true);
    assert!(check_suite(&case.to_string()).is_err());
}

#[test]
fn pin_and_explanation_rejections_use_actual_semantics() {
    // These are not merely expected-label mutations: the real validator must
    // reject an omitted explanation and a mutable or truncated source pin.
    for (pin, reason) in [
        (Value::Null, Value::String("   ".to_string())),
        (
            Value::String("deadbeef".to_string()),
            Value::String("not pinned".to_string()),
        ),
        (Value::Null, Value::Number(42.into())),
    ] {
        let case = Case {
            id: "independent-negative-control".to_string(),
            overrides: Map::from_iter([
                ("source_revision".to_string(), pin),
                ("source_revision_reason".to_string(), reason),
            ]),
            expected_valid: false,
        };
        let base: Value = serde_json::from_str(FIXTURES).unwrap();
        assert_eq!(
            run_case(&base["base_upstream"], &case),
            Ok(false),
            "unsafe upstream pin/reason admitted"
        );
    }
}
