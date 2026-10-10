//! Historical case identity guard for eight independent Phase0 fixture families (#71).
//!
//! The existing inventory checks counts and first entries. Those assertions
//! cannot detect same-count substitution of a later original case. These
//! immutable expected IDs are independent of the JSON being validated at test
//! time. Ad-hoc-research identities belong to a separate active implementation
//! in PR #129 and intentionally are not duplicated here.
//!
//! This is a structural regression guard, NOT semantic/oracle parity, rights
//! clearance, real-Git topology verification or production authority.

use serde_json::{json, Value};
use std::collections::HashSet;

type FamilyGroup = (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
);

fn historical_groups() -> Vec<FamilyGroup> {
    vec![
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "publication_cases",
            &[
                "complete-absent-current-authority",
                "existing-complete",
                "existing-incomplete",
                "duplicate-equivalent",
                "inventory-unknown",
                "authority-stale",
                "self-asserted-authority-ignored",
                "schema-incompatible",
                "locator-conflict",
                "provider-locator-unknown-before-create",
            ],
        ),
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "identity_cases",
            &[
                "same-semantic-retry",
                "presentation-only-variance-same-identity",
                "changed-content-new-identity",
                "changed-source-new-identity",
                "temporary-cannot-claim-persistent-state",
                "shared-transport-principal-not-persistent-identity",
                "packet-storage-not-policy",
                "recursive-derivative-not-independent",
                "pure-research-branch-creation",
                "exceptional-existing-surface-without-authority",
                "exceptional-mutation-active-collision",
            ],
        ),
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "source_cases",
            &[
                "default-authority-first",
                "branch-fully-contained",
                "diverged-unique-evidence",
                "stale-pr-unique-caveat",
                "branch-name-only",
                "default-unresolved",
                "default-head-moved-during-research",
                "consumed-source-ref-deleted",
                "same-path-default-vs-nondefault",
            ],
        ),
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "recovery_cases",
            &[
                "cutoff-before-create",
                "cutoff-after-create-before-ack",
                "lost-ack-retry",
                "authority-expired-after-create-before-ack",
                "incomplete-retry-inventory",
                "lost-ack-schema-incompatible",
                "lost-ack-content-conflict",
            ],
        ),
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "concurrency_cases",
            &[
                "two-empty-atomic-unique",
                "two-empty-serialized",
                "two-empty-reconcile-after-create",
                "two-empty-blind-create",
                "atomic-create-lost-ack-retry",
            ],
        ),
        (
            "ad-hoc-research-spec1.json",
            include_str!("../../fixtures/ad-hoc-research-spec1.json"),
            "packet_field_cases",
            &[
                "required-handoff-fields-preserved",
                "truthfully-empty-handoff-fields-preserved",
                "missing-stale-source-warning-rejected",
                "missing-discovery-vocabulary-rejected",
                "missing-useful-next-action-rejected",
            ],
        ),
        (
            "authority-closure-spec2.json",
            include_str!("../../fixtures/authority-closure-spec2.json"),
            "cases",
            &[
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
            ],
        ),
        (
            "adaptive-stress-spec2.json",
            include_str!("../../fixtures/adaptive-stress-spec2.json"),
            "cases",
            &[
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
            ],
        ),
        (
            "containment-capacity-spec2.json",
            include_str!("../../fixtures/containment-capacity-spec2.json"),
            "decision_cases",
            &[
                "backlog-cannot-renew",
                "duplicate-semantic-case",
                "policy-generation-move",
                "missing-restoration-debt",
            ],
        ),
        (
            "containment-capacity-spec2.json",
            include_str!("../../fixtures/containment-capacity-spec2.json"),
            "workload_cases",
            &[
                "ordinary-load",
                "correlated-burst-overload",
                "sustained-arrivals-restoration-progress",
                "priority-starvation-detected",
                "priority-starvation-failsafe",
                "zero-restoration-capacity",
                "service-loss-then-recovery",
                "restored-terminal",
                "separate-restoration-capacity",
            ],
        ),
        (
            "containment-capacity-spec2.json",
            include_str!("../../fixtures/containment-capacity-spec2.json"),
            "planning_cases",
            &[
                "rare-event-false-positive-heavy",
                "sensitivity-baseline",
                "low-action-volume",
                "high-action-volume",
            ],
        ),
        (
            "containment-spec3.json",
            include_str!("../../fixtures/containment-spec3.json"),
            "decision_cases",
            &[
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
            ],
        ),
        (
            "containment-spec3.json",
            include_str!("../../fixtures/containment-spec3.json"),
            "recovery_cases",
            &[
                "false-positive-recovery",
                "recovery-stale-subject",
                "recovery-dependency-moved",
                "recovery-missing-authority",
                "recovery-insufficient-evidence",
            ],
        ),
        (
            "containment-spec3.json",
            include_str!("../../fixtures/containment-spec3.json"),
            "trace_cases",
            &[
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
            ],
        ),
        (
            "github-capability-spec5.json",
            include_str!("../../fixtures/github-capability-spec5.json"),
            "cases",
            &[
                "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
                "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28",
            ],
        ),
        (
            "integration-candidate-v1.json",
            include_str!("../../fixtures/integration-candidate-v1.json"),
            "cases",
            &[
                "normal-two-parent",
                "reversed-parents-same-tree",
                "unsupported-three-parent",
                "compatible-constructor-migration",
            ],
        ),
        (
            "integration-candidate-v1.json",
            include_str!("../../fixtures/integration-candidate-v1.json"),
            "stale_head_cases",
            &["target-moved", "source-moved"],
        ),
        (
            "merge-base-topology-spec2.json",
            include_str!("../../fixtures/merge-base-topology-spec2.json"),
            "$",
            &[
                "unique-complete",
                "multiple-complete",
                "multiple-reordered",
                "none-complete",
                "incomplete-view",
                "multi-no-virtual",
                "multi-virtual-a",
                "multi-virtual-a-reordered",
                "multi-virtual-version-drift",
                "multi-virtual-intermediate-drift",
                "same-heads-replaced-history",
                "best-base-moved",
            ],
        ),
    ]
}

fn case_id(case: &Value) -> Result<String, String> {
    match case.get("id").or_else(|| case.get("name")) {
        Some(Value::String(id)) if !id.trim().is_empty() => Ok(id.clone()),
        Some(Value::Number(n)) if n.as_u64().is_some_and(|x| x > 0) => Ok(n.to_string()),
        _ => Err("case identity must be a nonempty string or positive integer".to_owned()),
    }
}

fn checked_cases<'a>(source: &'a Value, key: &str) -> Result<&'a [Value], String> {
    let selected = if key == "$" {
        source
    } else {
        source
            .get(key)
            .ok_or_else(|| format!("missing fixture group {key}"))?
    };
    selected
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("fixture group {key} is not an array"))
}

fn check_all_required_ids(cases: &[Value], originals: &[&str]) -> Result<(), String> {
    let mut actual = HashSet::new();
    for case in cases {
        let id = case_id(case)?;
        if !actual.insert(id.clone()) {
            return Err(format!("duplicate historical case identity {id}"));
        }
    }
    for id in originals {
        if !actual.contains(*id) {
            return Err(format!("missing historical case identity {id}"));
        }
    }
    Ok(())
}

#[test]
fn all_194_original_cases_across_eight_fixture_families_are_present() {
    let mut total = 0;
    for (family, source, group, originals) in historical_groups() {
        let document: Value = serde_json::from_str(source).expect("original authored fixture JSON");
        let cases = checked_cases(&document, group).expect("historical group is present");
        assert!(
            check_all_required_ids(cases, originals).is_ok(),
            "{family}/{group}: original scenario identity disappeared"
        );
        total += originals.len();
    }
    assert_eq!(
        total, 194,
        "exact historical inventory must not silently shrink"
    );
}

#[test]
fn substituting_a_nonfirst_case_at_unchanged_length_is_rejected_in_every_group() {
    for (family, source, group, originals) in historical_groups() {
        let document: Value = serde_json::from_str(source).expect("original authored fixture JSON");
        let cases = checked_cases(&document, group).expect("historical group is present");
        assert!(
            cases.len() >= 2,
            "{family}/{group} requires a second original case"
        );
        let mut changed = cases.to_vec();
        let replacement = json!("invented-same-count-false-pass");
        let second = changed.get_mut(1).expect("second case");
        let field = if second.get("id").is_some() {
            "id"
        } else {
            "name"
        };
        second[field] = replacement;
        assert_eq!(
            changed.len(),
            cases.len(),
            "mutation must preserve group count"
        );
        let error = check_all_required_ids(&changed, originals)
            .expect_err("same-count replacement must not pass case inventory");
        let removed = case_id(&cases[1]).expect("original case identity");
        assert!(
            error.contains(&removed),
            "{family}/{group}: missing displaced original {removed} was not diagnosed: {error}"
        );
    }
}

#[test]
fn duplicates_and_unreadable_case_identity_never_pass() {
    let (_, source, group, originals) = historical_groups()[0];
    let document: Value = serde_json::from_str(source).unwrap();
    let cases = checked_cases(&document, group).unwrap();
    let mut changed = cases.to_vec();
    let first_id = case_id(&changed[0]).expect("first historical case identity");
    let field = if changed[1].get("id").is_some() {
        "id"
    } else {
        "name"
    };
    changed[1][field] = json!(first_id);
    let error = check_all_required_ids(&changed, originals).expect_err("duplicate must fail");
    assert!(error.contains("duplicate"));
    changed[1][field] = json!(null);
    assert!(check_all_required_ids(&changed, originals).is_err());
}
