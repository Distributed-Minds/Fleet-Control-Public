#!/usr/bin/env python3
import json
from pathlib import Path

FIXTURE_PATH = Path(__file__).resolve().parents[1] / "fixtures" / "adaptive-stress-spec2.json"


def main() -> None:
    data = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    assert data["spec"] == 2
    cases = {case["name"]: case for case in data["cases"]}
    required = {
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
    }
    assert required == set(cases)

    assert cases["fixture-authority-is-inert"]["authority_change"] is False
    assert cases["retry-reuses-evaluation-id"]["extra_workload_charge"] is False
    assert cases["ack-loss-reconciles-first"]["rerun"] is False

    for name in (
        "evaluator-version-churn-same-family",
        "metric-version-churn-same-family",
        "control-version-churn-same-family",
        "fixture-version-churn-same-family",
    ):
        assert cases[name]["fresh_family_budget"] is False
        assert cases[name]["expected_new_family"] is False

    assert cases["cosmetic-family-split-rejected"]["expected"] == "REJECT_RESET"
    for name in ("material-target-change-can-split", "measurement-semantic-correction-can-split"):
        case = cases[name]
        assert case["lineage_preserved"] is True
        assert case["bounded_allowance"] > 0
        assert case["expected"] == "ALLOW_BOUNDED_RESET"

    assert cases["ambiguous-equivalence-fails-closed"]["fresh_family_budget"] is False
    assert cases["ambiguous-equivalence-fails-closed"]["expected"] == "UNKNOWN"
    assert cases["missing-telemetry-is-unknown"]["expected"] == "UNKNOWN"
    assert cases["missing-telemetry-is-unknown"]["numeric_default"] is None

    for name in ("evaluation-budget-exhaustion", "family-budget-exhaustion"):
        assert cases[name]["fresh_workload"] is False
        assert cases[name]["expected"] == "SUPPRESS"

    assert cases["duplicate-evidence-not-independent"]["expected_independence_gain"] is False
    assert cases["correlated-evaluators-not-independent"]["expected_independence_gain"] is False
    assert cases["observed-failure-not-closure"]["expected_closed"] is False
    for name in ("patch-accepted-is-closure", "patch-blocked-is-closure", "no-patch-justified-is-closure"):
        assert cases[name]["expected_closed"] is True
    assert cases["anti-overfit-requires-siblings"]["expected_accept"] is False
    assert cases["regression-blocks-score-optimization"]["expected_accept"] is False
    assert cases["recurrence-links-remediation-lineage"]["expected"] == "REOPEN_ELIGIBILITY"

    print(f"adaptive stress fixtures: {len(cases)} passed")


if __name__ == "__main__":
    main()
