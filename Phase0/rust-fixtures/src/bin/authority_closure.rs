//! Independent Rust evaluator for the historical authority-closure spec-2 fixtures.
//! This is an inert fixture oracle, NOT permission to revoke any real authority.

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process;

const MAX_FIXTURE_BYTES: u64 = 8 * 1024 * 1024;

/// File admission is bounded independently of typed fixture validation.
/// This offline tool is not a provider authority or a live revocation path.
fn read_fixture_with_observed_metadata(
    path: &Path,
    observed: &fs::Metadata,
) -> Result<String, String> {
    if !observed.file_type().is_file() {
        return Err("fixture must be a regular, non-symlink file".to_owned());
    }
    if observed.len() > MAX_FIXTURE_BYTES {
        return Err("fixture exceeds the 8 MiB input limit".to_owned());
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        // On Linux, refuse a symlink swapped in at open and do not block on
        // a FIFO swapped in between symlink_metadata and OpenOptions::open.
        const O_NONBLOCK: i32 = 0o4000;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("cannot open {path:?}: {error}"))?;
    let opened = file.metadata().map_err(|error| error.to_string())?;
    if !opened.file_type().is_file() {
        return Err("opened fixture is not a regular file".to_owned());
    }
    #[cfg(unix)]
    if observed.dev() != opened.dev() || observed.ino() != opened.ino() {
        return Err("fixture changed between metadata check and open".to_owned());
    }
    if opened.len() > MAX_FIXTURE_BYTES {
        return Err("fixture exceeds the 8 MiB input limit".to_owned());
    }

    // A concurrent writer may extend the opened inode after the size check.
    // Read at most MAX + 1 bytes before parsing, never the entire growing file.
    let mut bytes = Vec::new();
    file.take(MAX_FIXTURE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_FIXTURE_BYTES {
        return Err("fixture exceeds the 8 MiB input limit".to_owned());
    }
    String::from_utf8(bytes).map_err(|_| "fixture input is not UTF-8".to_owned())
}

fn read_fixture(path: &Path) -> Result<String, String> {
    let observed =
        fs::symlink_metadata(path).map_err(|error| format!("cannot inspect {path:?}: {error}"))?;
    read_fixture_with_observed_metadata(path, &observed)
}

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
            let Some(text) = value.as_str() else {
                return Err(format!("invalid type for semantic input: {name}"));
            };
            // An unknown enum value must not bypass validation just because
            // another unrelated rule short-circuits semantic evaluation.
            let supported = match name.as_str() {
                "provider_access" => text == "ERROR",
                "state_class" => matches!(text, "COMMITTED_OBLIGATION" | "HISTORICAL_EFFECT"),
                "composition" => matches!(text, "ALL_REQUIRED" | "ANY_OF_DECLARED"),
                _ => false,
            };
            if !supported {
                return Err(format!("unsupported value for semantic input: {name}"));
            }
            true
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

/// The fixture is an observation, not a declaration that omitted evidence is
/// false. Require the paired witness when a branch's positive trigger is present,
/// even if an unrelated earlier branch would otherwise short-circuit evaluation.
fn require_necessary_witnesses(case: &Map<String, Value>) -> Result<(), String> {
    const REQUIRED_IF_TRUE: &[(&str, &str)] = &[
        ("provider_credential_valid", "fleet_authority_revoked"),
        ("fleet_authority_revoked", "external_invalidation_complete"),
        ("cancel_ack_lost", "authoritative_state_known"),
        ("locator_reused", "same_incarnation"),
        ("same_operation_id", "same_incarnation"),
        ("external_deletion", "closure_caused_deletion"),
        ("cleanup_consequential", "recovery_authority"),
        ("handoff_independent", "retained_scope_exact"),
        ("cycle", "external_root"),
        ("job_started_after_cutoff", "current_authority"),
        ("ancestor_cutoff", "child_effect_after_cutoff"),
        ("shared_infrastructure", "authority_dependency"),
        ("runtime_minted", "initiator_stopped"),
        (
            "declared_surfaces_complete",
            "undeclared_external_surfaces_unknown",
        ),
    ];
    for (trigger, required) in REQUIRED_IF_TRUE {
        if case.get(*trigger).and_then(Value::as_bool) == Some(true)
            && !case.contains_key(*required)
        {
            return Err(format!("{trigger} requires current witness: {required}"));
        }
    }
    if case
        .get("lineage_protocol_compatible")
        .and_then(Value::as_bool)
        == Some(false)
        && !case.contains_key("mutation_requested")
    {
        return Err("incompatible lineage requires current witness: mutation_requested".to_owned());
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
    let Some(rule) = string(case, "composition") else {
        return Ok(None);
    };
    // A missing or empty set of declared roots must never mint authority by
    // vacuous ALL_REQUIRED truth (0 >= 0) or an undeclared ANY_OF_DECLARED root.
    let required = case
        .get("required_roots")
        .and_then(Value::as_u64)
        .ok_or("composition requires an explicit required_roots count")?;
    let surviving = case
        .get("surviving_roots")
        .and_then(Value::as_u64)
        .ok_or("composition requires an explicit surviving_roots count")?;
    if required == 0 || surviving > required {
        return Err("composition has no declared roots or impossible root counts".to_owned());
    }
    let result = match rule {
        "ALL_REQUIRED" if surviving == required => "BOUNDED_AUTHORITY",
        "ALL_REQUIRED" => "NO_AUTHORITY",
        "ANY_OF_DECLARED" if surviving > 0 => "BOUNDED_AUTHORITY",
        "ANY_OF_DECLARED" => "NO_AUTHORITY",
        other => return Err(format!("unsupported composition: {other}")),
    };
    Ok(Some(verdict(result)))
}

/// Deliberately receives *only* semantic inputs; never receives fixture expected labels.
fn evaluate(case: &Map<String, Value>) -> Result<(String, Value), String> {
    validate_inputs(case)?;
    require_necessary_witnesses(case)?;

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
    // Explicit denials and reconciliation obligations outrank independent
    // positive composition, handoff, cycle and closure claims. A mixed input
    // must never mint BOUNDED_AUTHORITY from revoked or fenced roots.
    if yes(case, "cancel_ack_lost") && no(case, "authoritative_state_known") {
        return Ok(verdict("READBACK_FIRST"));
    }
    if yes(case, "locator_reused") && no(case, "same_incarnation") {
        return Ok(verdict("DO_NOT_CANCEL_REPLACEMENT"));
    }
    if yes(case, "external_deletion") && no(case, "closure_caused_deletion") {
        return Ok(verdict("NO_CAUSAL_CLAIM"));
    }
    if yes(case, "cleanup_consequential") && no(case, "recovery_authority") {
        return Ok(verdict("DENY"));
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
    let outcome = read_fixture(Path::new(&args[1])).and_then(|source| validate_fixture(&source));
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
    fn historical_necessary_witnesses_cannot_be_deleted_or_masked_by_other_inputs() {
        let source = include_str!("../../../fixtures/authority-closure-spec2.json");
        let original: Value = serde_json::from_str(source).expect("historical fixture parses");
        let omissions = [
            (
                "provider-valid-revoked-credential-denied-with-debt",
                "external_invalidation_complete",
            ),
            (
                "ack-loss-reconciles-before-retry",
                "authoritative_state_known",
            ),
            ("locator-reuse-incarnation-safe", "same_incarnation"),
            (
                "independent-handoff-survives-bounded",
                "retained_scope_exact",
            ),
            ("cycle-with-independent-root", "external_root"),
            (
                "delayed-job-requires-current-authority",
                "current_authority",
            ),
            (
                "unrelated-deletion-not-attributed",
                "closure_caused_deletion",
            ),
            (
                "fresh-cleanup-needs-recovery-authority",
                "recovery_authority",
            ),
            (
                "independent-sibling-not-over-revoked",
                "authority_dependency",
            ),
            (
                "lineage-version-disagreement-fails-closed",
                "mutation_requested",
            ),
            (
                "closure-complete-only-declared-surfaces",
                "undeclared_external_surfaces_unknown",
            ),
            (
                "runtime-minted-authority-is-descendant",
                "initiator_stopped",
            ),
            ("repeat-cancel-is-idempotent", "same_incarnation"),
            ("child-after-cutoff-denied", "child_effect_after_cutoff"),
        ];
        for (name, field) in omissions {
            let mut fixture = original.clone();
            let case = fixture["cases"]
                .as_array_mut()
                .expect("cases array")
                .iter_mut()
                .find(|item| item["name"].as_str() == Some(name))
                .expect("original named fixture case");
            case.as_object_mut().expect("case object").remove(field);
            let mut semantic = case.as_object().expect("case object").clone();
            semantic.remove("name");
            for key in EXPECTED_KEYS {
                semantic.remove(*key);
            }
            let error = evaluate(&semantic).expect_err("missing witness must not yield a verdict");
            assert!(
                error.contains("requires current witness"),
                "{name}/{field}: {error}"
            );
            assert!(
                validate_fixture(&fixture.to_string()).is_err(),
                "historical fixture {name} accepted omission of {field}"
            );
        }
    }

    #[test]
    fn unrelated_terminal_hint_does_not_mask_incomplete_revocation_witnesses() {
        let case = input(json!({
            "provider_access": "ERROR",
            "provider_credential_valid": true,
            "fleet_authority_revoked": true
        }));
        assert!(evaluate(&case).is_err());
        let complete = input(json!({
            "provider_access": "ERROR",
            "provider_credential_valid": true,
            "fleet_authority_revoked": true,
            "external_invalidation_complete": false
        }));
        assert_eq!(
            evaluate(&complete).unwrap(),
            result("expected_closure", "ERROR")
        );
    }

    #[test]
    fn explicit_denials_preempt_unrelated_positive_authority_claims() {
        check(
            json!({
                "composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1,
                "provider_credential_valid":true,"fleet_authority_revoked":true,
                "external_invalidation_complete":false
            }),
            "DENY_AND_RECORD_REVOCATION_DEBT",
        );
        check(
            json!({
                "composition":"ANY_OF_DECLARED","surviving_roots":1,"required_roots":1,
                "ancestor_cutoff":true,"child_effect_after_cutoff":true
            }),
            "DENY",
        );
        check(
            json!({
                "cycle":true,"external_root":true,
                "job_started_after_cutoff":true,"current_authority":false
            }),
            "DENY_OR_UNRESOLVED",
        );
        check(
            json!({
                "handoff_independent":true,"retained_scope_exact":true,
                "grandchild_created_after_fence":true
            }),
            "NO_AUTHORITY",
        );
        check(
            json!({
                "declared_surfaces_complete":true,
                "undeclared_external_surfaces_unknown":true,
                "cleanup_consequential":true,"recovery_authority":false
            }),
            "DENY",
        );
        check(
            json!({
                "composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1,
                "locator_reused":true,"same_incarnation":false
            }),
            "DO_NOT_CANCEL_REPLACEMENT",
        );
        check(
            json!({
                "composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1,
                "cancel_ack_lost":true,"authoritative_state_known":false
            }),
            "READBACK_FIRST",
        );
        check(
            json!({
                "composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1,
                "external_deletion":true,"closure_caused_deletion":false
            }),
            "NO_CAUSAL_CLAIM",
        );
        check(
            json!({"composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1}),
            "BOUNDED_AUTHORITY",
        );
    }

    #[test]
    fn provider_unavailable_is_preserved_even_with_complete_revocation_evidence() {
        let case = input(json!({
            "provider_access": "ERROR",
            "provider_credential_valid": true,
            "fleet_authority_revoked": true,
            "external_invalidation_complete": false
        }));
        assert_eq!(
            evaluate(&case).unwrap(),
            result("expected_closure", "ERROR")
        );
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
    fn unsupported_string_states_fail_before_unrelated_short_circuits() {
        for (field, invalid) in [
            ("provider_access", "NOT_ERROR"),
            ("state_class", "UNKNOWN_STATE"),
            ("composition", "QUORUM"),
        ] {
            // Previously the false inventory flag returned UNKNOWN while an
            // unrelated malformed enumeration was silently accepted.
            let mut case = input(json!({"inventory_complete": false}));
            case.insert(field.to_owned(), json!(invalid));
            assert!(
                evaluate(&case).is_err(),
                "unrecognized {field}={invalid} bypassed semantic input admission"
            );
        }
    }

    #[test]
    fn historical_string_states_remain_admitted() {
        for case in [
            json!({"provider_access":"ERROR"}),
            json!({"state_class":"COMMITTED_OBLIGATION"}),
            json!({"state_class":"HISTORICAL_EFFECT"}),
            json!({"composition":"ALL_REQUIRED","surviving_roots":1,"required_roots":1}),
            json!({"composition":"ANY_OF_DECLARED","surviving_roots":1,"required_roots":2}),
        ] {
            assert!(evaluate(&input(case.clone())).is_ok(), "rejected {case}");
        }
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
    fn multi_root_composition_does_not_mint_authority_from_absent_roots() {
        for rule in ["ALL_REQUIRED", "ANY_OF_DECLARED"] {
            for bad in [
                json!({"composition":rule}),
                json!({"composition":rule,"surviving_roots":0}),
                json!({"composition":rule,"required_roots":0}),
                json!({"composition":rule,"surviving_roots":0,"required_roots":0}),
                json!({"composition":rule,"surviving_roots":1,"required_roots":0}),
                json!({"composition":rule,"surviving_roots":3,"required_roots":2}),
            ] {
                assert!(
                    evaluate(&input(bad.clone())).is_err(),
                    "unbounded root composition must be rejected: {bad}"
                );
            }
            assert_eq!(
                evaluate(&input(json!({
                    "composition":rule,"surviving_roots":0,"required_roots":2
                })))
                .unwrap(),
                verdict("NO_AUTHORITY")
            );
            assert_eq!(
                evaluate(&input(json!({
                    "composition":rule,"surviving_roots":2,"required_roots":2
                })))
                .unwrap(),
                verdict("BOUNDED_AUTHORITY")
            );
        }
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
