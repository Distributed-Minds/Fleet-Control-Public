#!/usr/bin/env python3
import json
from pathlib import Path

FIXTURE_PATH = Path(__file__).resolve().parents[1] / "fixtures" / "authority-closure-spec2.json"


def main() -> None:
    data = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    assert data["spec"] == 2
    cases = {case["name"]: case for case in data["cases"]}
    required = {
        "child-after-cutoff-denied",
        "grandchild-race-fenced",
        "delayed-job-requires-current-authority",
        "ack-loss-reconciles-before-retry",
        "partial-cancel-not-closed",
        "locator-reuse-incarnation-safe",
        "independent-handoff-survives-bounded",
        "revoked-ancestor-self-handoff-rejected",
        "multi-root-all-required-loses-one-root",
        "multi-root-any-declared-survives",
        "cycle-without-external-root",
        "cycle-with-independent-root",
        "incomplete-pagination-not-complete",
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
    assert required == set(cases)

    assert cases["child-after-cutoff-denied"]["expected"] == "DENY"
    assert cases["grandchild-race-fenced"]["expected"] == "NO_AUTHORITY"
    assert cases["delayed-job-requires-current-authority"]["expected"] == "DENY_OR_UNRESOLVED"
    assert cases["ack-loss-reconciles-before-retry"]["expected"] == "READBACK_FIRST"
    assert cases["partial-cancel-not-closed"]["expected_closure"] == "PARTIAL"
    assert cases["locator-reuse-incarnation-safe"]["expected"] == "DO_NOT_CANCEL_REPLACEMENT"
    assert cases["independent-handoff-survives-bounded"]["expected"] == "PRESERVE_BOUNDED"
    assert cases["revoked-ancestor-self-handoff-rejected"]["expected"] == "REJECT"
    assert cases["multi-root-all-required-loses-one-root"]["expected"] == "NO_AUTHORITY"
    assert cases["multi-root-any-declared-survives"]["expected"] == "BOUNDED_AUTHORITY"
    assert cases["cycle-without-external-root"]["expected"] == "NO_AUTHORITY"
    assert cases["cycle-with-independent-root"]["expected"] == "ROOT_BOUNDED_ONLY"
    assert cases["incomplete-pagination-not-complete"]["expected_closure"] == "UNKNOWN"
    assert cases["runtime-minted-authority-is-descendant"]["expected"] == "TRACK_DERIVATIVE"

    for name in ("committed-obligation-not-revoked", "historical-effect-not-revoked"):
        assert cases[name]["expected"] == "RESIDUAL_STATE"

    assert cases["fresh-cleanup-needs-recovery-authority"]["expected"] == "DENY"
    assert cases["independent-sibling-not-over-revoked"]["expected"] == "PRESERVE"
    assert cases["provider-unavailable-is-debt"]["expected_closure"] == "ERROR"
    assert cases["unrelated-deletion-not-attributed"]["expected"] == "NO_CAUSAL_CLAIM"
    assert cases["repeat-cancel-is-idempotent"]["expected_logical_cancellations"] == 1
    assert cases["closure-complete-only-declared-surfaces"]["expected_closure"] == "COMPLETE_FOR_DECLARED_SURFACES"

    print(f"authority closure fixtures: {len(cases)} passed")


if __name__ == "__main__":
    main()
