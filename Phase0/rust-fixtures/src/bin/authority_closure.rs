//! Independent Rust evaluator for the historical authority-closure spec-2 fixtures.
//! This is an inert fixture oracle, NOT permission to revoke any real authority.

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const REQUIRED: [&str; 26] = [
    "child-after-cutoff-denied",
    "grandchild-race-fenced",
    "delayed-job-requires-current-authority",
    "provider-valid-revoked-credential-denied-with-debt",
    "ack-loss-reconciles-before-retry",
    "partial-cancel-not-closed",
    "locator-reuse-incarnation-safe",
    "independent-handoff-survives-bounded",
    "revoked-ancestor-self-handoff-rejected",
    "multi-root-all-required-loses-one-root",
    "multi-root-any-declared-survives",
    "cycle-without-external-root",
    "cycle-with-independent-root",
    "cycle-all-required-missing-root",
    "cycle-any-declared-surviving-root",
    "incomplete-pagination-not-complete",
    "lineage-version-disagreement-fails-closed",
    "runtime-minted-authority-is-descendant",
    "committed-obligation-not-revoked",
    "historical-effect-not-revoked",
    "fresh-cleanup-needs-recovery-authority",
    "independent-sibling-not-over-revoked",
    "provider-unavailable-is-debt",
    "unrelated-deletion-not-attributed",
    "repeat-cancel-is-idempotent",
    "closure-complete-only-declared-surfaces",
];

const BOOLS: &[&str] = &[
    "lineage_protocol_compatible",
    "mutation_requested",
    "inventory_complete",
    "declared_surfaces_complete",
    "undeclared_external_surfaces_unknown",
    "cancel_ack_lost",
    "authoritative_state_known",
    "locator_reused",
    "same_incarnation",
    "same_operation_id",
    "external_deletion",
    "closure_caused_deletion",
    "cleanup_consequential",
    "recovery_authority",
    "handoff_independent",
    "retained_scope_exact",
    "cycle",
    "external_root",
    "runtime_minted",
    "initiator_stopped",
    "grandchild_created_after_fence",
    "job_started_after_cutoff",
    "current_authority",
    "provider_credential_valid",
    "fleet_authority_revoked",
    "external_invalidation_complete",
    "ancestor_cutoff",
    "child_effect_after_cutoff",
    "shared_infrastructure",
    "authority_dependency",
];
const STRINGS: &[&str] = &["provider_access", "state_class", "composition"];
const COUNTS: &[&str] = &["failed_descendants", "surviving_roots", "required_roots"];
const EXPECTED_KEYS: &[&str] = &[
    "expected",
    "expected_closure",
    "expected_logical_cancellations",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    spec: u32,
    cases: Vec<Map<String, Value>>,
}

fn validate_inputs(case: &Map<String, Value>) -> Result<(), String> {
    for (name, value) in case {
        let good = if BOOLS.contains(&name.as_str()) {
            value.is_boolean()
        } else if STRINGS.contains(&name.as_str()) {
            value.is_string()
        } else if COUNTS.contains(&name.as_str()) {
            value.as_u64().is_some()
        } else {
            return Err(format!("unsupported semantic input: {name}"));
        };
        if !good {
            return Err(format!("invalid type for semantic input: {name}"));
        }
    }
    Ok(())
}

fn yes(case: &Map<String, Value>, key: &str) -> bool {
    case.get(key).and_then(Value::as_bool) == Some(true)
}

fn no(case: &Map<String, Value>, key: &str) -> bool {
    case.get(key).and_then(Value::as_bool) == Some(false)
}

fn string<'a>(case: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    case.get(key).and_then(Value::as_str)
}

fn number(case: &Map<String, Value>, key: &str) -> u64 {
    case.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn result(key: &str, label: &str) -> (String, Value) {
    (key.to_owned(), Value::String(label.to_owned()))
}

fn verdict(label: &str) -> (String, Value) {
    result("expected", label)
}

fn composition(case: &Map<String, Value>) -> Result<Option<(String, Value)>, String> {
    let output = match string(case, "composition") {
        None => None,
        Some("ALL_REQUIRED") => Some(verdict(
            if number(case, "surviving_roots") >= number(case, "required_roots") {
                "BOUNDED_AUTHORITY"
            } else {
                "NO_AUTHORITY"
            },
        )),
        Some("ANY_OF_DECLARED") => Some(verdict(if number(case, "surviving_roots") > 0 {
            "BOUNDED_AUTHORITY"
        } else {
            "NO_AUTHORITY"
        })),
        Some(other) => return Err(format!("unsupported composition: {other}")),
    };
    Ok(output)
}

/// Deliberately receives *only* semantic inputs; never receives fixture expected labels.
fn evaluate(case: &Map<String, Value>) -> Result<(String, Value), String> {
    validate_inputs(case)?;

    // Compatibility, inventory, and durable closure evidence are gates.
    if no(case, "lineage_protocol_compatible") && yes(case, "mutation_requested") {
        return Ok(verdict("FAIL_CLOSED_UNTIL_COMPATIBLE"));
    }
    if string(case, "provider_access") == Some("ERROR") {
        return Ok(result("expected_closure", "ERROR"));
    }
    if no(case, "inventory_complete") {
        return Ok(result("expected_closure", "UNKNOWN"));
    }
    if number(case, "failed_descendants") > 0 {
        return Ok(result("expected_closure", "PARTIAL"));
    }
    if yes(case, "declared_surfaces_complete") && yes(case, "undeclared_external_surfaces_unknown")
    {
        return Ok(result("expected_closure", "COMPLETE_FOR_DECLARED_SURFACES"));
    }

    // Acknowledgement loss is not permission to retry a destructive operation.
    if yes(case, "cancel_ack_lost") && no(case, "authoritative_state_known") {
        return Ok(verdict("READBACK_FIRST"));
    }
    if yes(case, "locator_reused") && no(case, "same_incarnation") {
        return Ok(verdict("DO_NOT_CANCEL_REPLACEMENT"));
    }
    if yes(case, "same_operation_id") && yes(case, "same_incarnation") {
        return Ok((
            "expected_logical_cancellations".to_owned(),
            Value::from(1_u64),
        ));
    }
    if yes(case, "external_deletion") && no(case, "closure_caused_deletion") {
        return Ok(verdict("NO_CAUSAL_CLAIM"));
    }

    if matches!(
        string(case, "state_class"),
        Some("COMMITTED_OBLIGATION" | "HISTORICAL_EFFECT")
    ) {
        return Ok(verdict("RESIDUAL_STATE"));
    }
    if yes(case, "cleanup_consequential") && no(case, "recovery_authority") {
        return Ok(verdict("DENY"));
    }

    if case.contains_key("handoff_independent") {
        return Ok(
            if yes(case, "handoff_independent") && yes(case, "retained_scope_exact") {
                verdict("PRESERVE_BOUNDED")
            } else {
                verdict("REJECT")
            },
        );
    }

    // Cyclic edges cannot manufacture authority; multi-root rules still bind.
    if yes(case, "cycle") {
        if !yes(case, "external_root") {
            return Ok(verdict("NO_AUTHORITY"));
        }
        return Ok(composition(case)?.unwrap_or_else(|| verdict("ROOT_BOUNDED_ONLY")));
    }
    if let Some(outcome) = composition(case)? {
        return Ok(outcome);
    }

    if yes(case, "runtime_minted") && yes(case, "initiator_stopped") {
        return Ok(verdict("TRACK_DERIVATIVE"));
    }
    if yes(case, "grandchild_created_after_fence") {
        return Ok(verdict("NO_AUTHORITY"));
    }
    if yes(case, "job_started_after_cutoff") && no(case, "current_authority") {
        return Ok(verdict("DENY_OR_UNRESOLVED"));
    }
    if yes(case, "provider_credential_valid") && yes(case, "fleet_authority_revoked") {
        return Ok(verdict(if no(case, "external_invalidation_complete") {
            "DENY_AND_RECORD_REVOCATION_DEBT"
        } else {
            "DENY"
        }));
    }
    if yes(case, "ancestor_cutoff") && yes(case, "child_effect_after_cutoff") {
        return Ok(verdict("DENY"));
    }
    if yes(case, "shared_infrastructure") && no(case, "authority_dependency") {
        return Ok(verdict("PRESERVE"));
    }

    Err("unsupported authority-closure semantic state".to_owned())
}

fn validate_fixture(source: &str) -> Result<usize, String> {
    let fixture: Fixture = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if fixture.spec != 2 {
        return Err(format!(
            "unsupported authority-closure spec {}",
            fixture.spec
        ));
    }
    if fixture.cases.len() != REQUIRED.len() {
        return Err(format!(
            "wrong fixture coverage: expected {}, found {}",
            REQUIRED.len(),
            fixture.cases.len()
        ));
    }
    let mut seen = HashSet::new();
    let required: HashSet<&str> = REQUIRED.iter().copied().collect();
    for mut case in fixture.cases {
        let name = case
            .remove("name")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(|| "missing/invalid fixture name".to_owned())?;
        if !seen.insert(name.clone()) || !required.contains(name.as_str()) {
            return Err(format!("duplicate or unknown fixture name: {name}"));
        }
        let expectations: Vec<_> = EXPECTED_KEYS
            .iter()
            .filter_map(|key| case.remove(*key).map(|v| ((*key).to_owned(), v)))
            .collect();
        if expectations.len() != 1 {
            return Err(format!("{name}: requires exactly one expected output"));
        }
        let computed = evaluate(&case)?;
        if computed != expectations[0] {
            return Err(format!(
                "{name}: semantic verdict {:?}, fixture expects {:?}",
                computed, expectations[0]
            ));
        }
    }
    if seen.len() != required.len() {
        return Err("missing required fixture identities".to_owned());
    }
    Ok(seen.len())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: authority_closure <Phase0/fixtures/authority-closure-spec2.json>");
        process::exit(2);
    }
    let outcome = fs::read_to_string(&args[1])
        .map_err(|e| e.to_string())
        .and_then(|source| validate_fixture(&source));
    match outcome {
        Ok(count) => println!("authority closure Rust semantic fixtures: {count} cases passed"),
        Err(reason) => {
            eprintln!("authority-closure validation FAILED: {reason}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn input(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    fn check(value: Value, expected: &str) {
        assert_eq!(evaluate(&input(value)).unwrap(), verdict(expected));
    }

    #[test]
    fn historical_spec2_fixtures_all_pass() {
        let source = include_str!("../../../fixtures/authority-closure-spec2.json");
        assert_eq!(validate_fixture(source).unwrap(), 26);
    }

    #[test]
    fn semantic_negative_controls_do_not_use_fixture_labels() {
        check(
            json!({"composition":"ALL_REQUIRED","surviving_roots":2,"required_roots":2}),
            "BOUNDED_AUTHORITY",
        );
        check(json!({"cycle":true,"external_root":false}), "NO_AUTHORITY");
        check(
            json!({"cycle":true,"external_root":true}),
            "ROOT_BOUNDED_ONLY",
        );
        check(
            json!({"provider_credential_valid":true,"fleet_authority_revoked":true,"external_invalidation_complete":true}),
            "DENY",
        );
        check(
            json!({"cycle":true,"external_root":true,"composition":"ALL_REQUIRED","surviving_roots":2,"required_roots":2}),
            "BOUNDED_AUTHORITY",
        );
        check(
            json!({"cycle":true,"external_root":true,"composition":"ANY_OF_DECLARED","surviving_roots":0,"required_roots":2}),
            "NO_AUTHORITY",
        );
    }

    #[test]
    fn invalid_types_and_unknown_semantics_fail_closed() {
        assert!(evaluate(&input(json!({"cycle":"false","external_root":true}))).is_err());
        assert!(evaluate(&input(
            json!({"cycle":true,"external_root":true,"composition":"QUORUM"})
        ))
        .is_err());
        assert!(evaluate(&input(json!({"expected":"NO_AUTHORITY","cycle":true}))).is_err());
        assert!(evaluate(&input(json!({"failed_descendants":-1}))).is_err());
    }

    #[test]
    fn missing_and_tampered_case_sets_fail() {
        let source = include_str!("../../../fixtures/authority-closure-spec2.json");
        let mut fixture: Value = serde_json::from_str(source).unwrap();
        fixture["cases"].as_array_mut().unwrap().pop();
        assert!(validate_fixture(&fixture.to_string()).is_err());
        fixture["cases"].as_array_mut().unwrap().clear();
        assert!(validate_fixture(&fixture.to_string()).is_err());
        let mut fixture: Value = serde_json::from_str(source).unwrap();
        fixture["cases"][0]["expected"] = json!("BOUNDED_AUTHORITY");
        assert!(validate_fixture(&fixture.to_string()).is_err());
        let mut fixture: Value = serde_json::from_str(source).unwrap();
        fixture["cases"][0]["name"] = fixture["cases"][1]["name"].clone();
        assert!(validate_fixture(&fixture.to_string()).is_err());
    }

    #[test]
    fn material_changes_produce_different_verdicts() {
        let denied = input(
            json!({"provider_credential_valid":true,"fleet_authority_revoked":true,"external_invalidation_complete":false}),
        );
        let settled = input(
            json!({"provider_credential_valid":true,"fleet_authority_revoked":true,"external_invalidation_complete":true}),
        );
        assert_ne!(evaluate(&denied).unwrap(), evaluate(&settled).unwrap());
        check(
            json!({"handoff_independent":false,"retained_scope_exact":true}),
            "REJECT",
        );
        check(
            json!({"handoff_independent":true,"retained_scope_exact":true}),
            "PRESERVE_BOUNDED",
        );
    }
}
