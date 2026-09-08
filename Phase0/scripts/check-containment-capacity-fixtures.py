#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA = json.loads((ROOT / "fixtures/containment-capacity-spec2.json").read_text())


def decide(c):
    if not c.get("policy_compatible", True):
        return "REQUIRE_MIGRATION"
    if c.get("duplicate_case"):
        return "RECONCILE_DUPLICATE"
    if not c.get("authority_current", False) and not c.get("effect_active", False):
        return "RESTORED"
    if c.get("backlog_only_renewal"):
        return "REJECT_RENEWAL"
    if not c.get("authority_current", False) and c.get("effect_active", False):
        if not c.get("restoration_debt", False):
            return "INVALID_MISSING_DEBT"
        if c.get("progress_within_bound", False):
            return "RESTORE_PROGRESS"
        if c.get("fail_safe_escalates", False):
            return "DEGRADED_ESCALATION"
        return "INVALID_STARVATION"
    return "OK"


failures = []
for case in DATA["cases"]:
    got = decide(case)
    if got != case["expected"]:
        failures.append((case["id"], case["expected"], got))

if failures:
    for ident, expected, got in failures:
        print(f"FAIL {ident}: expected={expected} got={got}")
    raise SystemExit(1)

print(f"PASS {len(DATA['cases'])}/{len(DATA['cases'])} containment-capacity cases")
