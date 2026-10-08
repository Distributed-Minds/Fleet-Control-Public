#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "github-capability-spec5.json"

REQUIRED_IDS = set(range(1, 29))
MUTATING_ACTIONS = {"scheduled_write", "cleanup", "default_branch_write"}
MUTATION_ALLOW = {"ALLOW_FENCED", "ALLOW_RECOVERY"}


def fail(message: str) -> None:
    raise SystemExit(f"FAIL: {message}")


def main() -> None:
    data = json.loads(FIXTURE.read_text(encoding="utf-8"))
    if data.get("spec") != 5 or data.get("issue") != 11:
        fail("fixture must identify issue 11 spec 5")

    cases = data.get("cases")
    if not isinstance(cases, list):
        fail("cases must be a list")

    ids = [case.get("id") for case in cases]
    if len(ids) != len(set(ids)):
        fail("fixture IDs must be unique")
    if set(ids) != REQUIRED_IDS:
        fail(f"fixture IDs must be exactly 1..28; got {sorted(set(ids))}")

    names = [case.get("name") for case in cases]
    if len(names) != len(set(names)) or any(not isinstance(name, str) or not name for name in names):
        fail("fixture names must be unique non-empty strings")

    for case in cases:
        cid = case["id"]
        lineage = case.get("lineage")
        authority = case.get("authority")
        resource = case.get("resource")
        action = case.get("action")
        expected = case.get("expected")
        recovery = bool(case.get("recovery", False))

        if action == "default_branch_write" and expected in MUTATION_ALLOW:
            fail(f"case {cid}: default-branch probe mutation cannot be allowed")

        if lineage in {"unknown", "conflicting"} and expected in MUTATION_ALLOW:
            fail(f"case {cid}: unresolved lineage cannot authorize mutation")

        if authority in {"none", "stale", "forged", "incompatible"} and expected == "ALLOW_FENCED":
            fail(f"case {cid}: non-current authority cannot authorize ordinary mutation")

        if resource == "ambiguous" and expected in MUTATION_ALLOW:
            fail(f"case {cid}: ambiguous resource incarnation cannot authorize mutation")

        if action == "cleanup" and authority == "stale":
            if recovery and expected != "ALLOW_RECOVERY":
                fail(f"case {cid}: explicit bounded recovery transfer must be modeled as allowed recovery")
            if not recovery and expected == "ALLOW_RECOVERY":
                fail(f"case {cid}: stale cleanup cannot gain recovery authority implicitly")

        if case.get("approval") == "required" and expected in MUTATION_ALLOW:
            fail(f"case {cid}: approval-required scheduled action cannot be autonomous success")

    by_id = {case["id"]: case for case in cases}
    exact = {
        8: "ACTION_BLOCKED",
        13: "AMBIGUOUS",
        17: "AUTHORITY_STALE",
        18: "RECOVERY_REQUIRED",
        19: "RECOVERY_REQUIRED",
        20: "AUTHORITY_UNAVAILABLE",
        21: "ALLOW_RECOVERY",
        22: "AUTHORITY_UNAVAILABLE",
        23: "AUTHORITY_STALE",
        25: "LINEAGE_UNKNOWN",
        26: "LINEAGE_UNKNOWN",
        27: "RECOVERY_REQUIRED",
        28: "LINEAGE_UNKNOWN",
    }
    for cid, expected in exact.items():
        if by_id[cid].get("expected") != expected:
            fail(f"case {cid}: expected {expected}")

    print(f"PASS: {len(cases)} GitHub capability acceptance fixtures")


if __name__ == "__main__":
    main()
