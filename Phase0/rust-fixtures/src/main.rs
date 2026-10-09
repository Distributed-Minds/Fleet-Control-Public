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
    #[serde(default, deserialize_with = "deserialize_non_null_level")]
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

fn deserialize_non_null_level<'de, D>(deserializer: D) -> Result<Option<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Level::deserialize(deserializer).map(Some)
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
    if requested >= Level::Isolate {
        // A fixture cannot waive the independently governed evidence requirement
        // by declaring zero or one lineage sufficient for high-impact action.
        let required = c.required_independent_lineages.unwrap_or(2);
        if required < 2 {
            return Err(
                "high-impact restriction requires at least two independent lineages".to_owned(),
            );
        }
        if c.independent_lineages < required {
            return Ok(outcome("INDEPENDENCE_INSUFFICIENT", None));
        }
    }
    if c.contradictory_evidence && requested >= Level::Isolate {
        let maximum = c.max_level_with_contradiction.unwrap_or(Level::BlockAction);
        // Exculpatory evidence may narrow a restriction, never escalate it.
        if maximum >= requested {
            return Err(
                "contradictory evidence cannot raise or preserve high-impact severity".to_owned(),
            );
        }
        requested = maximum;
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
    match field.as_deref() {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("missing or blank required trace input: {name}")),
    }
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

const REQUIRED_DECISION_CASE_IDS: &[&str] = &[
    "least-harm",
    "explicit-broader",
    "stale-subject",
    "stale-detector-evidence",
    "ambiguous-attribution-bounds-to-observation",
    "incompatible-detector-versions",
    "stale-dependency",
    "incompatible-dependency",
    "missing-authority-block-action",
    "missing-authority",
    "single-lineage-high-impact",
    "correlated-duplicate-does-not-count",
    "contradiction-bounds-action",
    "expired-no-renewal",
    "expired-renewed",
    "one-agent-human-independent-control",
    "destructive-needs-authority",
];

const REQUIRED_RECOVERY_CASE_IDS: &[&str] = &[
    "false-positive-recovery",
    "recovery-stale-subject",
    "recovery-dependency-moved",
    "recovery-missing-authority",
    "recovery-insufficient-evidence",
];

const REQUIRED_TRACE_CASE_IDS: &[&str] = &[
    "cutoff-retry",
    "changed-intent-new-operation",
    "locator-reuse",
    "effect-dependency-moved",
    "effect-dependency-compatible-upgrade",
    "closure-dependency-moved",
    "evidence-assurance-moved",
    "recovery-dependency-moved-trace",
    "termination-without-closure-proof",
    "termination-with-closure-proof",
    "pure-model-no-production-authority",
    "production-authority-present",
    "secondary-harm-preserved",
];

// Fixture identities are machine tokens and may appear in diagnostics.
// Keep visible international text, but reject record separators and invisible
// formatting that would make two different case IDs look identical.
fn valid_case_id(id: &str) -> bool {
    !id.is_empty()
        && !id.chars().any(|ch| {
            ch.is_control()
                || ch.is_whitespace()
                || matches!(
                    ch,
                    '\u{00ad}'
                        | '\u{034f}'
                        | '\u{061c}'
                        | '\u{180e}'
                        | '\u{200b}'..='\u{200f}'
                        | '\u{202a}'..='\u{202e}'
                        | '\u{2060}'..='\u{206f}'
                        | '\u{fe00}'..='\u{fe0f}'
                        | '\u{feff}'
                        | '\u{e0001}'
                        | '\u{e0020}'..='\u{e007f}'
                        | '\u{e0100}'..='\u{e01ef}'
                )
        })
}

fn require_historical_case_ids<'a>(
    family: &str,
    expected_ids: &[&str],
    observed_ids: impl Iterator<Item = &'a str>,
    failures: &mut Vec<String>,
) {
    let observed: HashSet<&str> = observed_ids.collect();
    for id in expected_ids {
        if !observed.contains(id) {
            failures.push(format!("missing required {family} case id: {id}"));
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
    require_historical_case_ids(
        "decision",
        REQUIRED_DECISION_CASE_IDS,
        f.decision_cases.iter().map(|c| c.id.as_str()),
        &mut failures,
    );
    require_historical_case_ids(
        "recovery",
        REQUIRED_RECOVERY_CASE_IDS,
        f.recovery_cases.iter().map(|c| c.id.as_str()),
        &mut failures,
    );
    require_historical_case_ids(
        "trace",
        REQUIRED_TRACE_CASE_IDS,
        f.trace_cases.iter().map(|c| c.id.as_str()),
        &mut failures,
    );
    let mut ids = HashSet::new();

    for c in &f.decision_cases {
        if !valid_case_id(&c.id) || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {:?}", c.id));
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
        if !valid_case_id(&c.id) || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {:?}", c.id));
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
        if !valid_case_id(&c.id) || !ids.insert(c.id.as_str()) {
            failures.push(format!("duplicate or empty case id: {:?}", c.id));
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
        fs::read_to_string(&input).map_err(|e| format!("cannot read {input:?}: {e}"))?;
    let fixture: Fixture = serde_json::from_str(&content)
        .map_err(|e| format!("invalid fixture {input:?}: {e}"))?;
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
    fn hostile_case_ids_fail_closed_without_injected_log_records() {
        for family in ["decision_cases", "recovery_cases", "trace_cases"] {
            for hostile_id in [
                "new\nPASS forged",
                "new\rPASS forged",
                "new\u{200b}hidden",
                "new\u{202e}reversed",
                "new\u{e0020}tagged",
            ] {
                let mut changed = original();
                let mut additional = changed[family][0].clone();
                additional["id"] = json!(hostile_id);
                changed[family].as_array_mut().unwrap().push(additional);
                let errors = validate(&fixture(changed)).expect_err("hostile ID was accepted");
                assert!(
                    errors
                        .iter()
                        .any(|error| error.contains("duplicate or empty case id")),
                    "{family}: missing explicit identity denial: {errors:?}"
                );
                assert!(
                    errors
                        .iter()
                        .all(|error| !error.contains('\n') && !error.contains('\r')),
                    "{family}: diagnostic injected a second line: {errors:?}"
                );
            }
        }
    }

    #[test]
    fn visible_international_case_ids_remain_valid() {
        let mut changed = original();
        let mut additional = changed["decision_cases"][0].clone();
        additional["id"] = json!("zusätzlicher-fall");
        changed["decision_cases"]
            .as_array_mut()
            .unwrap()
            .push(additional);
        assert_eq!(validate(&fixture(changed)), Ok(36));
    }

    #[test]
    fn actual_historical_containment_family_passes_35_semantic_cases() {
        let typed: Fixture = serde_json::from_str(BASELINE).expect("typed baseline");
        assert_eq!(validate(&typed), Ok(35));
    }

    #[test]
    fn absent_narrowest_level_defaults_but_explicit_null_is_invalid() {
        let mut absent = original();
        let case = absent["decision_cases"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|case| case["id"] == "one-agent-human-independent-control")
            .unwrap();
        case.as_object_mut()
            .unwrap()
            .remove("narrowest_effective_level");
        let typed: Fixture = serde_json::from_value(absent).expect("omitted optional level");
        let case = typed
            .decision_cases
            .iter()
            .find(|case| case.id == "one-agent-human-independent-control")
            .unwrap();
        assert_eq!(
            decide(case),
            Ok(outcome("AUTHORIZED", Some(Level::Isolate)))
        );

        let mut explicit_null = original();
        let case = explicit_null["decision_cases"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|case| case["id"] == "one-agent-human-independent-control")
            .unwrap();
        case["narrowest_effective_level"] = json!(null);
        assert!(serde_json::from_value::<Fixture>(explicit_null).is_err());
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

    #[test]
    fn equal_blank_operation_ids_cannot_pass_cutoff_retry_deduplication() {
        let mut altered = original();
        altered["trace_cases"][0]["first_operation_id"] = json!(" \t ");
        altered["trace_cases"][0]["retry_operation_id"] = json!(" \t ");
        let typed = fixture(altered);
        assert!(trace(&typed.trace_cases[0]).is_err());
        assert!(validate(&typed).is_err());
    }

    #[test]
    fn equal_blank_incarnations_cannot_claim_current_locator() {
        let mut altered = original();
        altered["trace_cases"][2]["expected_incarnation"] = json!(" \t ");
        altered["trace_cases"][2]["current_incarnation"] = json!(" \t ");
        altered["trace_cases"][2]["expected"] = json!("CURRENT");
        let typed = fixture(altered);
        assert!(trace(&typed.trace_cases[2]).is_err());
        assert!(validate(&typed).is_err());
    }

    #[test]
    fn equal_blank_dependency_bases_cannot_authorize_boundary() {
        let mut altered = original();
        altered["trace_cases"][3]["decision_basis"] = json!(" \t ");
        altered["trace_cases"][3]["boundary_basis"] = json!(" \t ");
        altered["trace_cases"][3]["expected"] = json!("BOUNDARY_ALLOWED");
        let typed = fixture(altered);
        assert!(trace(&typed.trace_cases[3]).is_err());
        assert!(validate(&typed).is_err());
    }
    #[test]
    fn renamed_historical_id_cannot_hide_missing_coverage() {
        for family in ["decision_cases", "recovery_cases", "trace_cases"] {
            let mut changed = original();
            let first_id = changed[family][0]["id"].as_str().unwrap().to_owned();
            changed[family][0]["id"] = json!(format!("{first_id}-renamed"));
            let result = validate(&fixture(changed));
            assert!(
                result.is_err(),
                "{family}: renamed baseline case was accepted"
            );
            assert!(
                result
                    .unwrap_err()
                    .iter()
                    .any(|error| error.contains("missing required")),
                "{family}: expected missing historical ID diagnostic"
            );
        }
    }

    #[test]
    fn independent_extra_case_does_not_invalidate_historical_coverage() {
        let mut changed = original();
        let mut additional = changed["decision_cases"][0].clone();
        additional["id"] = json!("additional-valid-decision-case");
        changed["decision_cases"]
            .as_array_mut()
            .unwrap()
            .push(additional);
        assert_eq!(validate(&fixture(changed)), Ok(36));
    }

    #[test]
    fn high_impact_independence_floor_cannot_be_disabled_by_fixture_input() {
        for threshold in [0, 1] {
            let mut changed = original();
            changed["decision_cases"][15]["required_independent_lineages"] = json!(threshold);
            let typed = fixture(changed);
            let decision = &typed.decision_cases[15];
            assert!(decide(decision).is_err(), "threshold {threshold} admitted");
            assert!(
                validate(&typed).is_err(),
                "threshold {threshold} passed fixtures"
            );
        }

        let mut changed = original();
        changed["decision_cases"][15]["required_independent_lineages"] = json!(3);
        let typed = fixture(changed);
        assert_eq!(
            decide(&typed.decision_cases[15]),
            Ok(outcome("INDEPENDENCE_INSUFFICIENT", None))
        );
    }

    #[test]
    fn contradictory_evidence_cannot_escalate_high_impact_restrictions() {
        for maximum in ["ISOLATE", "TERMINATE", "DESTRUCTIVE_CLEANUP"] {
            let mut changed = original();
            changed["decision_cases"][12]["max_level_with_contradiction"] = json!(maximum);
            let typed = fixture(changed);
            assert!(
                decide(&typed.decision_cases[12]).is_err(),
                "cap {maximum} admitted"
            );
            assert!(validate(&typed).is_err(), "cap {maximum} passed fixtures");
        }

        let mut changed = original();
        changed["decision_cases"][12]["max_level_with_contradiction"] =
            json!("FREEZE_NEW_AUTHORITY");
        let typed = fixture(changed);
        assert_eq!(
            decide(&typed.decision_cases[12]),
            Ok(outcome("AUTHORIZED", Some(Level::FreezeNewAuthority)))
        );
    }
}
