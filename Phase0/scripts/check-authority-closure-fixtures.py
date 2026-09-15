#!/usr/bin/env python3
import json
from pathlib import Path

from authority_closure_model import evaluate_authority_closure

FIXTURE_PATH = Path(__file__).resolve().parents[1] / "fixtures" / "authority-closure-spec2.json"
EXPECTED_KEYS = {"expected", "expected_closure", "expected_logical_cancellations"}
REQUIRED_CASES = {
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
}


def expected_disposition(case: dict) -> dict:
    expected = {key: case[key] for key in EXPECTED_KEYS if key in case}
    assert len(expected) == 1, f"{case.get('name')}: exactly one expected disposition is required"
    return expected


def semantic_inputs(case: dict) -> dict:
    # Expected fixture labels are deliberately removed before execution so they
    # cannot influence the reducer result.
    return {key: value for key, value in case.items() if key not in EXPECTED_KEYS}


def main() -> None:
    data = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    assert data["spec"] == 2
    cases = {case["name"]: case for case in data["cases"]}
    assert REQUIRED_CASES == set(cases)

    for name in sorted(REQUIRED_CASES):
        case = cases[name]
        computed = evaluate_authority_closure(semantic_inputs(case))
        expected = expected_disposition(case)
        assert computed == expected, f"{name}: computed={computed!r} expected={expected!r}"

    # Semantic negative controls prove that behavior changes when material
    # authority facts change, rather than merely reproducing case labels.
    assert evaluate_authority_closure({
        "composition": "ALL_REQUIRED", "surviving_roots": 2, "required_roots": 2
    }) == {"expected": "BOUNDED_AUTHORITY"}
    assert evaluate_authority_closure({"cycle": True, "external_root": False}) == {
        "expected": "NO_AUTHORITY"
    }
    assert evaluate_authority_closure({"cycle": True, "external_root": True}) == {
        "expected": "ROOT_BOUNDED_ONLY"
    }
    assert evaluate_authority_closure({
        "provider_credential_valid": True,
        "fleet_authority_revoked": True,
        "external_invalidation_complete": True,
    }) == {"expected": "DENY"}
    assert evaluate_authority_closure({
        "cycle": True,
        "external_root": True,
        "composition": "ALL_REQUIRED",
        "surviving_roots": 2,
        "required_roots": 2,
    }) == {"expected": "BOUNDED_AUTHORITY"}
    assert evaluate_authority_closure({
        "cycle": True,
        "external_root": True,
        "composition": "ANY_OF_DECLARED",
        "surviving_roots": 0,
        "required_roots": 2,
    }) == {"expected": "NO_AUTHORITY"}

    print(f"authority closure executable fixtures: {len(cases)} cases + 6 negative controls passed")


if __name__ == "__main__":
    main()
