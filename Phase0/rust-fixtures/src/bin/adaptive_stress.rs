//! Typed, executable checks for the historical adaptive-stress-spec2 family.
//! A second independent Phase0 Rust fixture family; NOT a runtime scheduler.
//! Run from the repository root:
//! cargo run --manifest-path Phase0/rust-fixtures/Cargo.toml --locked --offline --bin adaptive_stress -- Phase0/fixtures/adaptive-stress-spec2.json

use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

#[cfg(test)]
const HISTORICAL: &str = include_str!("../../../fixtures/adaptive-stress-spec2.json");
const REQUIRED_NAMES: [&str; 23] = [
    "fixture-authority-is-inert",
    "retry-reuses-evaluation-id",
    "ack-loss-reconciles-first",
    "evaluator-version-churn-same-family",
    "metric-version-churn-same-family",
    "control-version-churn-same-family",
    "fixture-version-churn-same-family",
    "cosmetic-family-split-rejected",
    "material-target-change-can-split",
    "measurement-semantic-correction-can-split",
    "ambiguous-equivalence-fails-closed",
    "missing-telemetry-is-unknown",
    "evaluation-budget-exhaustion",
    "family-budget-exhaustion",
    "duplicate-evidence-not-independent",
    "correlated-evaluators-not-independent",
    "observed-failure-not-closure",
    "patch-accepted-is-closure",
    "patch-blocked-is-closure",
    "no-patch-justified-is-closure",
    "anti-overfit-requires-siblings",
    "regression-blocks-score-optimization",
    "recurrence-links-remediation-lineage",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    spec: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    fixture: Option<String>,
    authority_change: Option<bool>,
    same_semantics: Option<bool>,
    retry: Option<bool>,
    expected_same_evaluation_id: Option<bool>,
    extra_workload_charge: Option<bool>,
    ack_lost: Option<bool>,
    durable_result_exists: Option<bool>,
    rerun: Option<bool>,
    claim_delta: Option<bool>,
    evaluator_changed: Option<bool>,
    metric_changed: Option<bool>,
    control_changed: Option<bool>,
    fixture_changed: Option<bool>,
    semantic_equivalence: Option<bool>,
    identifier_changed: Option<bool>,
    delta_kind: Option<String>,
    lineage_preserved: Option<bool>,
    bounded_allowance: Option<u32>,
    prior_evidence_incomparable: Option<bool>,
    equivalence: Option<String>,
    fresh_family_budget: Option<bool>,
    telemetry_present: Option<bool>,
    numeric_default: Option<i64>,
    evaluation_budget_remaining: Option<i64>,
    family_budget_remaining: Option<i64>,
    fresh_workload: Option<bool>,
    duplicate_evidence: Option<bool>,
    shared_lineage: Option<bool>,
    disposition: Option<String>,
    candidate_identity: Option<bool>,
    independent_acceptance: Option<bool>,
    named_blocker: Option<bool>,
    corrective_surface: Option<bool>,
    original_case_passes: Option<bool>,
    siblings_pass: Option<bool>,
    score_improved: Option<bool>,
    regression_present: Option<bool>,
    post_promotion_recurrence: Option<bool>,
    lineage_present: Option<bool>,
    expected: Option<String>,
    expected_new_family: Option<bool>,
    expected_independence_gain: Option<bool>,
    expected_closed: Option<bool>,
    expected_accept: Option<bool>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Computed<'a> {
    disposition: Option<&'a str>,
    same_evaluation_id: Option<bool>,
    extra_workload_charge: Option<bool>,
    rerun: Option<bool>,
    new_family: Option<bool>,
    fresh_family_budget: Option<bool>,
    fresh_workload: Option<bool>,
    independence_gain: Option<bool>,
    closed: Option<bool>,
    accept: Option<bool>,
}

// Distinct scenario fields select a semantic rule. The asserted expected
// fields and the case name NEVER select the computed outcome.
fn compute(c: &Case) -> Result<Computed<'_>, &'static str> {
    // Every historical case represents exactly one semantic operation. Without
    // this admission guard, a case can append a second, contradictory operation
    // and obtain PASS because the first matching else-if branch ignores it.
    // Group paired alternative fields together (budget, independence, scoring).
    let selector_groups = [
        c.fixture.is_some(),
        c.same_semantics.is_some() || c.retry.is_some(),
        c.ack_lost.is_some() || c.durable_result_exists.is_some(),
        c.claim_delta.is_some(),
        c.equivalence.is_some(),
        c.telemetry_present.is_some(),
        c.evaluation_budget_remaining.is_some() || c.family_budget_remaining.is_some(),
        c.duplicate_evidence.is_some() || c.shared_lineage.is_some(),
        c.disposition.is_some(),
        c.original_case_passes.is_some() || c.score_improved.is_some(),
        c.post_promotion_recurrence.is_some(),
    ];
    if selector_groups
        .into_iter()
        .filter(|selected| *selected)
        .count()
        != 1
    {
        return Err("scenario must select exactly one semantic operation");
    }

    let mut result = Computed::default();
    if c.fixture.is_some() {
        result.disposition = Some(if c.authority_change == Some(false) {
            "EVALUATION_FAILURE"
        } else {
            "UNSUPPORTED_AUTHORITY_CHANGE"
        });
    } else if let (Some(same), Some(retry)) = (c.same_semantics, c.retry) {
        result.same_evaluation_id = Some(same && retry);
        result.extra_workload_charge = Some(!same || !retry);
    } else if let (Some(ack_lost), Some(durable)) = (c.ack_lost, c.durable_result_exists) {
        result.disposition = Some(if ack_lost && durable {
            "READBACK"
        } else {
            "REVALIDATE"
        });
        result.rerun = Some(!durable);
    } else if let Some(changed) = c.claim_delta {
        if changed {
            let supported_delta = match c.delta_kind.as_deref() {
                Some("target-behavior") => true,
                Some("measurement-semantics") => c.prior_evidence_incomparable == Some(true),
                _ => false,
            };
            result.disposition = Some(
                if supported_delta
                    && c.lineage_preserved == Some(true)
                    && c.bounded_allowance.unwrap_or(0) > 0
                {
                    "ALLOW_BOUNDED_RESET"
                } else {
                    "REJECT_RESET"
                },
            );
        } else if c.identifier_changed == Some(true) {
            result.disposition = Some("REJECT_RESET");
        } else {
            // Version churn alone does not create a new semantic family;
            // require an actual version-only change (and equivalent fixture
            // semantics when the fixture itself changes).
            let version_only_change = c.evaluator_changed == Some(true)
                || c.metric_changed == Some(true)
                || c.control_changed == Some(true)
                || (c.fixture_changed == Some(true) && c.semantic_equivalence == Some(true));
            if !version_only_change {
                return Err("missing justified version-only change");
            }
            result.new_family = Some(false);
            result.fresh_family_budget = Some(false);
        }
    } else if let Some(equivalence) = c.equivalence.as_deref() {
        result.disposition = Some(if equivalence == "UNKNOWN" {
            "UNKNOWN"
        } else {
            "REVIEW_EQUIVALENCE"
        });
        result.fresh_family_budget = Some(false);
    } else if let Some(telemetry) = c.telemetry_present {
        result.disposition = Some(if telemetry {
            "REVIEW_TELEMETRY"
        } else {
            "UNKNOWN"
        });
        // Missing telemetry is unknown, never silently a numeric zero.
        if !telemetry && c.numeric_default.is_some() {
            return Err("missing telemetry cannot acquire a numeric default");
        }
    } else if c.evaluation_budget_remaining.is_some() || c.family_budget_remaining.is_some() {
        let has_budget = c.evaluation_budget_remaining.unwrap_or(1) > 0
            && c.family_budget_remaining.unwrap_or(1) > 0;
        result.disposition = Some(if has_budget { "ELIGIBLE" } else { "SUPPRESS" });
        result.fresh_workload = Some(has_budget);
    } else if c.duplicate_evidence.is_some() || c.shared_lineage.is_some() {
        result.independence_gain =
            Some(!c.duplicate_evidence.unwrap_or(false) && !c.shared_lineage.unwrap_or(false));
    } else if let Some(disposition) = c.disposition.as_deref() {
        result.closed = Some(match disposition {
            "PATCH-ACCEPTED" => {
                c.candidate_identity == Some(true) && c.independent_acceptance == Some(true)
            }
            "PATCH-BLOCKED" => c.named_blocker == Some(true),
            "NO-PATCH-JUSTIFIED" => c.corrective_surface == Some(true),
            "FAILURE-OBSERVED" => false,
            _ => return Err("unsupported remediation disposition"),
        });
    } else if c.original_case_passes.is_some() || c.score_improved.is_some() {
        result.accept = Some(
            c.original_case_passes.unwrap_or(true)
                && c.siblings_pass.unwrap_or(true)
                && !c.regression_present.unwrap_or(false),
        );
    } else if let Some(recurred) = c.post_promotion_recurrence {
        result.disposition = Some(if recurred && c.lineage_present == Some(true) {
            "REOPEN_ELIGIBILITY"
        } else {
            "RECONCILE_RECURRENCE"
        });
    } else {
        return Err("unrecognized scenario inputs");
    }
    Ok(result)
}

fn validate(fixture: &Fixture) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    if fixture.spec != 2 {
        errors.push(format!(
            "unsupported adaptive-stress spec: {}",
            fixture.spec
        ));
    }
    let names: HashSet<&str> = fixture
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect();
    if names.len() != fixture.cases.len() {
        errors.push("duplicate scenario name".to_owned());
    }
    if names != REQUIRED_NAMES.into_iter().collect::<HashSet<_>>() {
        errors.push("missing, extra or renamed adaptive-stress scenario".to_owned());
    }

    for c in &fixture.cases {
        match compute(c) {
            Err(reason) => errors.push(format!("{}: {reason}", c.name)),
            Ok(actual) => {
                let declared = Computed {
                    disposition: c.expected.as_deref(),
                    same_evaluation_id: c.expected_same_evaluation_id,
                    extra_workload_charge: c.extra_workload_charge,
                    rerun: c.rerun,
                    new_family: c.expected_new_family,
                    fresh_family_budget: c.fresh_family_budget,
                    fresh_workload: c.fresh_workload,
                    independence_gain: c.expected_independence_gain,
                    closed: c.expected_closed,
                    accept: c.expected_accept,
                };
                if actual != declared {
                    errors.push(format!(
                        "{}: computed {:?}, fixture asserts {:?}",
                        c.name, actual, declared
                    ));
                }
            }
        }
    }
    errors.sort();
    if errors.is_empty() {
        Ok(fixture.cases.len())
    } else {
        Err(errors)
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/adaptive-stress-spec2.json"));
    if args.next().is_some() {
        eprintln!("Usage: adaptive_stress [fixture.json]");
        process::exit(2);
    }
    let result = fs::read_to_string(&path)
        .map_err(|err| format!("{}: {err}", path.display()))
        .and_then(|content| {
            serde_json::from_str::<Fixture>(&content)
                .map_err(|err| format!("invalid adaptive-stress JSON: {err}"))
        })
        .and_then(|fixture| {
            validate(&fixture)
                .map(|count| {
                    println!("adaptive-stress semantic fixtures (Rust): {count} passed");
                })
                .map_err(|errors| errors.join("\nFAIL: "))
        });
    if let Err(err) = result {
        eprintln!("FAIL: {err}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn baseline() -> Value {
        serde_json::from_str(HISTORICAL).expect("valid historical fixture")
    }

    fn checked(value: Value) -> Result<usize, Vec<String>> {
        let fixture: Fixture = serde_json::from_value(value).expect("typed mutated fixture");
        validate(&fixture)
    }

    fn change(name: &str, field: &str, value: Value) -> Value {
        let mut root = baseline();
        let case = root["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|c| c["name"] == name)
            .expect("named historical case");
        case[field] = value;
        root
    }

    #[test]
    fn all_23_historical_cases_are_independently_computed() {
        assert_eq!(checked(baseline()), Ok(23));
    }

    #[test]
    fn mutated_assertion_is_not_used_as_the_computed_result() {
        let modified = change("ack-loss-reconciles-first", "expected", json!("RERUN"));
        assert!(checked(modified).is_err());
    }

    #[test]
    fn fixture_text_cannot_mint_authority() {
        let modified = change(
            "fixture-authority-is-inert",
            "authority_change",
            json!(true),
        );
        assert!(checked(modified).is_err());
    }

    #[test]
    fn reset_requires_a_positive_bounded_allowance() {
        let modified = change(
            "material-target-change-can-split",
            "bounded_allowance",
            json!(0),
        );
        assert!(checked(modified).is_err());
    }

    #[test]
    fn cosmetic_changes_cannot_refresh_family_budget() {
        let modified = change(
            "evaluator-version-churn-same-family",
            "claim_delta",
            json!(true),
        );
        assert!(checked(modified).is_err());
    }

    #[test]
    fn sibling_regression_blocks_acceptance_even_when_original_passes() {
        let modified = change(
            "anti-overfit-requires-siblings",
            "siblings_pass",
            json!(true),
        );
        assert!(checked(modified).is_err());
    }

    #[test]
    fn missing_or_duplicate_case_is_not_a_partial_pass() {
        let mut missing = baseline();
        missing["cases"].as_array_mut().expect("cases").pop();
        assert!(checked(missing).is_err());

        let mut duplicate = baseline();
        let first = duplicate["cases"][0].clone();
        duplicate["cases"]
            .as_array_mut()
            .expect("cases")
            .push(first);
        assert!(checked(duplicate).is_err());
    }

    #[test]
    fn unsupported_spec_and_unknown_fields_fail_closed() {
        let mut wrong_spec = baseline();
        wrong_spec["spec"] = json!(3);
        assert!(checked(wrong_spec).is_err());

        let mutated = change(
            "missing-telemetry-is-unknown",
            "secret_override",
            json!(true),
        );
        assert!(serde_json::from_value::<Fixture>(mutated).is_err());
    }

    #[test]
    fn present_telemetry_may_carry_a_measured_numeric_value() {
        let mut present = change(
            "missing-telemetry-is-unknown",
            "telemetry_present",
            json!(true),
        );
        let case = present["cases"]
            .as_array_mut()
            .expect("cases")
            .iter_mut()
            .find(|c| c["name"] == "missing-telemetry-is-unknown")
            .expect("telemetry case");
        case["numeric_default"] = json!(7);
        case["expected"] = json!("REVIEW_TELEMETRY");
        assert_eq!(checked(present), Ok(23));
    }

    #[test]
    fn missing_telemetry_cannot_fabricate_numeric_measurements() {
        let modified = change("missing-telemetry-is-unknown", "numeric_default", json!(0));
        let errors = checked(modified).expect_err("missing telemetry must fail closed");
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing telemetry cannot acquire a numeric default")),
            "{errors:?}"
        );
    }
}

#[cfg(test)]
mod selector_exclusivity_regressions {
    use super::*;
    use serde_json::{json, Value};

    const ORIGINAL: &str = include_str!("../../../fixtures/adaptive-stress-spec2.json");

    fn checked_mutation(name: &str, extra_field: &str, additional_input: Value) {
        let mut document: Value = serde_json::from_str(ORIGINAL).expect("original fixture");
        let original = document["cases"]
            .as_array_mut()
            .expect("historical cases")
            .iter_mut()
            .find(|case| case["name"] == name)
            .expect("historical case identity");
        // Preserve all expected verdicts; only add a meaning-bearing input
        // that would otherwise be ignored by compute's first matching branch.
        original[extra_field] = additional_input;
        let typed: Fixture = serde_json::from_value(document).expect("well-typed adversarial case");
        let failures = validate(&typed).expect_err("ambiguous operation must fail closed");
        assert!(
            failures
                .iter()
                .any(|message| message.contains("exactly one semantic operation")),
            "missing selector diagnosis for {name}: {failures:?}"
        );
    }

    #[test]
    fn original_complete_case_inventory_remains_valid() {
        let original: Fixture = serde_json::from_str(ORIGINAL).expect("historical fixture");
        assert_eq!(validate(&original), Ok(23));
    }

    #[test]
    fn fixture_authority_cannot_smuggle_telemetry_admission() {
        checked_mutation(
            "fixture-authority-is-inert",
            "telemetry_present",
            json!(false),
        );
    }

    #[test]
    fn retry_evidence_cannot_smuggle_remediation_closure() {
        checked_mutation(
            "ack-loss-reconciles-first",
            "disposition",
            json!("PATCH-ACCEPTED"),
        );
    }

    #[test]
    fn telemetry_unknown_cannot_smuggle_positive_budget() {
        checked_mutation(
            "missing-telemetry-is-unknown",
            "evaluation_budget_remaining",
            json!(100),
        );
    }

    #[test]
    fn budget_exhaustion_cannot_smuggle_independence_evidence() {
        checked_mutation(
            "evaluation-budget-exhaustion",
            "shared_lineage",
            json!(false),
        );
    }

    #[test]
    fn recurrence_evaluation_cannot_smuggle_independent_scoring() {
        checked_mutation(
            "recurrence-links-remediation-lineage",
            "score_improved",
            json!(true),
        );
    }
}
