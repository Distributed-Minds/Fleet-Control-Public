//! Typed, side-effect-free Rust oracle for the Phase0 containment-spec3 fixture.
//! Scope: one of eight independent fixture families in public issue #71.
//! This is NOT runtime containment enforcement or a completed Python migration.

use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn yes() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Level {
    Observe,
    BlockAction,
    SuspendCapability,
    FreezeNewAuthority,
    Isolate,
    Terminate,
    DestructiveCleanup,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DependencyState {
    #[default]
    Current,
    Stale,
    Incompatible,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: u32,
    spec_version: u32,
    decision_cases: Vec<DecisionCase>,
    recovery_cases: Vec<RecoveryCase>,
    trace_cases: Vec<TraceCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionCase {
    id: String,
    requested_level: Option<Level>,
    narrowest_effective_level: Option<Level>,
    #[serde(default)]
    broader_action_justified: bool,
    #[serde(default = "yes")]
    subject_current: bool,
    #[serde(default = "yes")]
    evidence_current: bool,
    #[serde(default = "yes")]
    detector_compatible: bool,
    #[serde(default = "yes")]
    attribution_confident: bool,
    #[serde(default)]
    dependency_state: DependencyState,
    #[serde(default)]
    expired: bool,
    #[serde(default)]
    renewal_authorized: bool,
    #[serde(default)]
    authority_current: bool,
    #[serde(default)]
    independent_lineages: u32,
    required_independent_lineages: Option<u32>,
    #[serde(default)]
    contradictory_evidence: bool,
    max_level_with_contradiction: Option<Level>,
    expected: DecisionOutcome,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct DecisionOutcome {
    disposition: String,
    level: Option<Level>,
}

fn outcome(disposition: &str, level: Option<Level>) -> DecisionOutcome {
    DecisionOutcome {
        disposition: disposition.to_owned(),
        level,
    }
}

fn decide(c: &DecisionCase) -> Result<DecisionOutcome, String> {
    if !c.subject_current {
        return Ok(outcome("IDENTITY_STALE", None));
    }
    if !c.evidence_current {
        return Ok(outcome("EVIDENCE_STALE", None));
    }
    if !c.detector_compatible {
        return Ok(outcome("DETECTOR_INCOMPATIBLE", None));
    }
    if !c.attribution_confident {
        return Ok(outcome("ATTRIBUTION_AMBIGUOUS", Some(Level::Observe)));
    }
    match c.dependency_state {
        DependencyState::Stale => return Ok(outcome("DEPENDENCY_STALE", None)),
        DependencyState::Incompatible => return Ok(outcome("DEPENDENCY_INCOMPATIBLE", None)),
        DependencyState::Current => {}
    }
    if c.expired && !c.renewal_authorized {
        return Ok(outcome("EXPIRED", None));
    }

    let mut requested = c
        .requested_level
        .ok_or_else(|| "missing requested_level on an otherwise current decision".to_owned())?;
    if requested >= Level::BlockAction && !c.authority_current {
        return Ok(outcome("AUTHORITY_MISSING", None));
    }
    if requested >= Level::Isolate
        && c.independent_lineages < c.required_independent_lineages.unwrap_or(2)
    {
        return Ok(outcome("INDEPENDENCE_INSUFFICIENT", None));
    }
    if c.contradictory_evidence && requested >= Level::Isolate {
        requested = c.max_level_with_contradiction.unwrap_or(Level::BlockAction);
    }
    let effective = c.narrowest_effective_level.unwrap_or(requested);
    if effective < requested && !c.broader_action_justified {
        requested = effective;
    }
    Ok(outcome("AUTHORIZED", Some(requested)))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryCase {
    id: String,
    #[serde(default = "yes")]
    subject_current: bool,
    #[serde(default)]
    dependency_state: DependencyState,
    #[serde(default)]
    recovery_authority_current: bool,
    #[serde(default)]
    independent_recovery_evidence: bool,
    expected: String,
}

fn recover(c: &RecoveryCase) -> &'static str {
    if !c.subject_current {
        "IDENTITY_STALE"
    } else if c.dependency_state != DependencyState::Current {
        "DEPENDENCY_STALE"
    } else if !c.recovery_authority_current {
        "AUTHORITY_MISSING"
    } else if !c.independent_recovery_evidence {
        "EVIDENCE_INSUFFICIENT"
    } else {
        "RECOVERED"
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TraceKind {
    CutoffRetry,
    LocatorReuse,
    DependencyBoundary,
    Closure,
    ExternalEffect,
    ResidualHarm,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceCase {
    id: String,
    kind: TraceKind,
    first_operation_id: Option<String>,
    retry_operation_id: Option<String>,
    expected_incarnation: Option<String>,
    current_incarnation: Option<String>,
    decision_basis: Option<String>,
    boundary_basis: Option<String>,
    #[serde(default)]
    compatibility_proven: bool,
    #[serde(default)]
    closure_evidence_current: bool,
    #[serde(default)]
    external_authority_current: bool,
    #[serde(default)]
    durable_secondary_effect: bool,
    expected: String,
}

fn required<'a>(field: &'a Option<String>, name: &str) -> Result<&'a str, String> {
    field
        .as_deref()
        .ok_or_else(|| format!("missing required trace input: {name}"))
}

fn trace(c: &TraceCase) -> Result<&'static str, String> {
    match c.kind {
        TraceKind::CutoffRetry => {
            if required(&c.first_operation_id, "first_operation_id")?
                == required(&c.retry_operation_id, "retry_operation_id")?
            {
                Ok("ONE_SEMANTIC_OPERATION")
            } else {
                Ok("DUPLICATE_OPERATION")
            }
        }
        TraceKind::LocatorReuse => {
            if required(&c.expected_incarnation, "expected_incarnation")?
                != required(&c.current_incarnation, "current_incarnation")?
            {
                Ok("IDENTITY_STALE")
            } else {
                Ok("CURRENT")
            }
        }
        TraceKind::DependencyBoundary => {
            let moved = required(&c.decision_basis, "decision_basis")?
                != required(&c.boundary_basis, "boundary_basis")?;
            if moved && !c.compatibility_proven {
                Ok("BOUNDARY_BLOCKED")
            } else {
                Ok("BOUNDARY_ALLOWED")
            }
        }
        TraceKind::Closure => {
            if c.closure_evidence_current {
                Ok("CLOSED")
            } else {
                Ok("CLOSURE_DEBT")
            }
        }
        TraceKind::ExternalEffect => {
            if c.external_authority_current {
                Ok("EFFECT_ALLOWED")
            } else {
                Ok("EFFECT_BLOCKED")
            }
        }
        TraceKind::ResidualHarm => {
            if c.durable_secondary_effect {
                Ok("RESIDUAL_HARM")
            } else {
                Ok("CLEAR")
            }
        }
    }
}

fn validate(f: &Fixture) -> Result<usize, Vec<String>> {
    let mut failures = Vec::new();
    if f.schema_version != 1 || f.spec_version != 3 {
        failures.push(format!(
            "unsupported fixture schema/spec: {}/{}",
            f.schema_version, f.spec_version
        ));
    }
    // Containment specification v3 has 17 decision, 5 recovery and 13 trace
    // baseline cases. Additional cases are allowed; silently dropping a family
    // must never turn an incomplete fixture into a successful CLI invocation.
    for (family, observed, minimum) in [
        ("decision", f.decision_cases.len(), 17),
        ("recovery", f.recovery_cases.len(), 5),
        ("trace", f.trace_cases.len(), 13),
    ] {
        if observed < minimum {
            failures.push(format!(
                "incomplete {family} fixture coverage: expected at least {minimum}, found {observed}"
            ));
        }
    }
    let mut ids = HashSet::new();

    for c in &f.decision_cases {
        if c.id.trim().is_empty() || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {}", c.id));
        }
        match decide(c) {
            Ok(actual) if actual == c.expected => {}
            Ok(actual) => failures.push(format!(
                "{}: expected {:?}; computed {:?}",
                c.id, c.expected, actual
            )),
            Err(reason) => failures.push(format!("{}: {reason}", c.id)),
        }
    }
    for c in &f.recovery_cases {
        if c.id.trim().is_empty() || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {}", c.id));
        }
        let actual = recover(c);
        if actual != c.expected {
            failures.push(format!(
                "{}: expected {}; computed {actual}",
                c.id, c.expected
            ));
        }
    }
    for c in &f.trace_cases {
        if c.id.trim().is_empty() || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {}", c.id));
        }
        match trace(c) {
            Ok(actual) if actual == c.expected => {}
            Ok(actual) => failures.push(format!(
                "{}: expected {}; computed {actual}",
                c.id, c.expected
            )),
            Err(reason) => failures.push(format!("{}: {reason}", c.id)),
        }
    }
    failures.sort();
    if failures.is_empty() {
        Ok(f.decision_cases.len() + f.recovery_cases.len() + f.trace_cases.len())
    } else {
        Err(failures)
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/containment-spec3.json"));
    if args.next().is_some() {
        return Err("usage: free-energy-phase0-fixtures [fixture.json]".to_owned());
    }
    let content =
        fs::read_to_string(&input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    let fixture: Fixture = serde_json::from_str(&content)
        .map_err(|e| format!("invalid fixture {}: {e}", input.display()))?;
    match validate(&fixture) {
        Ok(count) => {
            println!("containment fixtures (Rust): {count} passed");
            Ok(())
        }
        Err(errors) => Err(errors.join("\nFAIL: ")),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FAIL: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const BASELINE: &str = include_str!("../../fixtures/containment-spec3.json");

    fn fixture(value: Value) -> Fixture {
        serde_json::from_value(value).expect("typed fixture")
    }

    fn original() -> Value {
        serde_json::from_str(BASELINE).expect("historical fixture is valid JSON")
    }

    #[test]
    fn actual_historical_containment_family_passes_35_semantic_cases() {
        let typed: Fixture = serde_json::from_str(BASELINE).expect("typed baseline");
        assert_eq!(validate(&typed), Ok(35));
    }

    #[test]
    fn current_authority_removed_changes_computed_decision() {
        let mut altered = original();
        altered["decision_cases"][0]["authority_current"] = json!(false);
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn changed_expected_result_is_not_treated_as_an_oracle() {
        let mut altered = original();
        altered["recovery_cases"][0]["expected"] = json!("EVIDENCE_INSUFFICIENT");
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn moved_dependency_boundary_fails_when_expected_is_held_constant() {
        let mut altered = original();
        altered["trace_cases"][3]["boundary_basis"] = json!("side-effect:v6:cfg-a");
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn correlated_duplicate_lineages_cannot_be_coerced_from_a_string() {
        let mut altered = original();
        altered["decision_cases"][11]["independent_lineages"] = json!("2");
        assert!(serde_json::from_value::<Fixture>(altered).is_err());
    }

    #[test]
    fn duplicate_case_identity_is_rejected() {
        let mut altered = original();
        altered["trace_cases"][0]["id"] = altered["decision_cases"][0]["id"].clone();
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn unsupported_schema_and_unrecognized_levels_are_rejected() {
        let mut altered = original();
        altered["schema_version"] = json!(999);
        assert!(validate(&fixture(altered)).is_err());
        let mut altered = original();
        altered["decision_cases"][0]["requested_level"] = json!("SUPER_ADMIN");
        assert!(serde_json::from_value::<Fixture>(altered).is_err());
    }

    #[test]
    fn omitted_authority_is_not_implicitly_granted() {
        let mut altered = original();
        altered["decision_cases"][0]
            .as_object_mut()
            .unwrap()
            .remove("authority_current");
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn deleted_decision_case_rejects_incomplete_coverage() {
        let mut altered = original();
        altered["decision_cases"].as_array_mut().unwrap().remove(0);
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn deleted_recovery_case_rejects_incomplete_coverage() {
        let mut altered = original();
        altered["recovery_cases"].as_array_mut().unwrap().remove(0);
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn deleted_trace_case_rejects_incomplete_coverage() {
        let mut altered = original();
        altered["trace_cases"].as_array_mut().unwrap().remove(0);
        assert!(validate(&fixture(altered)).is_err());
    }

    #[test]
    fn empty_containment_fixture_is_not_a_successful_run() {
        let mut altered = original();
        altered["decision_cases"] = json!([]);
        altered["recovery_cases"] = json!([]);
        altered["trace_cases"] = json!([]);
        assert!(validate(&fixture(altered)).is_err());
    }
}
