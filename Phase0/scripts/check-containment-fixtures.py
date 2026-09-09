#!/usr/bin/env python3
import json
import sys
from pathlib import Path

LEVELS = {
    "OBSERVE": 0,
    "BLOCK_ACTION": 1,
    "SUSPEND_CAPABILITY": 2,
    "FREEZE_NEW_AUTHORITY": 3,
    "ISOLATE": 4,
    "TERMINATE": 5,
    "DESTRUCTIVE_CLEANUP": 6,
}


def dependency_ok(case):
    return case.get("dependency_state", "CURRENT") == "CURRENT"


def decide(case):
    if not case.get("subject_current", True):
        return {"disposition": "IDENTITY_STALE", "level": None}
    if not case.get("evidence_current", True):
        return {"disposition": "EVIDENCE_STALE", "level": None}
    if not case.get("detector_compatible", True):
        return {"disposition": "DETECTOR_INCOMPATIBLE", "level": None}
    if not case.get("attribution_confident", True):
        return {"disposition": "ATTRIBUTION_AMBIGUOUS", "level": "OBSERVE"}
    dep = case.get("dependency_state", "CURRENT")
    if dep == "STALE":
        return {"disposition": "DEPENDENCY_STALE", "level": None}
    if dep != "CURRENT":
        return {"disposition": "DEPENDENCY_INCOMPATIBLE", "level": None}
    if case.get("expired", False) and not case.get("renewal_authorized", False):
        return {"disposition": "EXPIRED", "level": None}

    requested = case["requested_level"]
    if LEVELS[requested] >= LEVELS["BLOCK_ACTION"] and not case.get("authority_current", False):
        return {"disposition": "AUTHORITY_MISSING", "level": None}
    if LEVELS[requested] >= LEVELS["ISOLATE"]:
        if case.get("independent_lineages", 0) < case.get("required_independent_lineages", 2):
            return {"disposition": "INDEPENDENCE_INSUFFICIENT", "level": None}

    if case.get("contradictory_evidence", False) and LEVELS[requested] >= LEVELS["ISOLATE"]:
        requested = case.get("max_level_with_contradiction", "BLOCK_ACTION")

    effective = case.get("narrowest_effective_level", requested)
    if LEVELS[effective] < LEVELS[requested] and not case.get("broader_action_justified", False):
        requested = effective
    return {"disposition": "AUTHORIZED", "level": requested}


def recover(case):
    if not case.get("subject_current", True):
        return "IDENTITY_STALE"
    if not dependency_ok(case):
        return "DEPENDENCY_STALE"
    if not case.get("recovery_authority_current", False):
        return "AUTHORITY_MISSING"
    if not case.get("independent_recovery_evidence", False):
        return "EVIDENCE_INSUFFICIENT"
    return "RECOVERED"


def trace(case):
    kind = case["kind"]
    if kind == "cutoff_retry":
        return "ONE_SEMANTIC_OPERATION" if case["first_operation_id"] == case["retry_operation_id"] else "DUPLICATE_OPERATION"
    if kind == "locator_reuse":
        return "IDENTITY_STALE" if case["expected_incarnation"] != case["current_incarnation"] else "CURRENT"
    if kind == "dependency_boundary":
        moved = case["decision_basis"] != case["boundary_basis"]
        return "BOUNDARY_BLOCKED" if moved and not case.get("compatibility_proven", False) else "BOUNDARY_ALLOWED"
    if kind == "closure":
        return "CLOSURE_DEBT" if not case.get("closure_evidence_current", False) else "CLOSED"
    if kind == "external_effect":
        return "EFFECT_BLOCKED" if not case.get("external_authority_current", False) else "EFFECT_ALLOWED"
    if kind == "residual_harm":
        return "RESIDUAL_HARM" if case.get("durable_secondary_effect", False) else "CLEAR"
    raise ValueError(f"unknown trace kind: {kind}")


def main():
    fixture_path = Path(sys.argv[1] if len(sys.argv) > 1 else "Phase0/fixtures/containment-spec3.json")
    data = json.loads(fixture_path.read_text())
    failures = []
    for case in data["decision_cases"]:
        actual = decide(case)
        if actual != case["expected"]:
            failures.append((case["id"], case["expected"], actual))
    for case in data["recovery_cases"]:
        actual = recover(case)
        if actual != case["expected"]:
            failures.append((case["id"], case["expected"], actual))
    for case in data["trace_cases"]:
        actual = trace(case)
        if actual != case["expected"]:
            failures.append((case["id"], case["expected"], actual))

    total = sum(len(data[key]) for key in ("decision_cases", "recovery_cases", "trace_cases"))
    if failures:
        for failure in failures:
            print("FAIL", *failure)
        return 1
    print(f"containment fixtures: {total} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
