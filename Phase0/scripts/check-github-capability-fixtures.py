#!/usr/bin/env python3
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "github-capability-spec5.json"

REQUIRED_IDS = set(range(1, 29))
MUTATING_ACTIONS = {"scheduled_write", "cleanup", "default_branch_write"}
MUTATION_ALLOW = {"ALLOW_FENCED", "ALLOW_RECOVERY"}

# Integrity baseline for the historical 28 scenarios, separate from fixture data.
# This detects silent fixture expectation/name drift; it is NOT a semantic
# outcome reducer and does NOT prove scheduled GitHub behavior.
CANONICAL_CASES = {
    1: ("interactive-write-scheduled-read-only", "CAPABILITY_ABSENT"),
    2: ("interactive-action-scheduled-missing", "CAPABILITY_ABSENT"),
    3: ("scheduled-stricter-approval", "ACTION_PAUSED"),
    4: ("repository-scope-differs", "ACTION_BLOCKED"),
    5: ("capability-evidence-drift", "REVALIDATE"),
    6: ("connector-without-mutation-action", "CAPABILITY_ABSENT"),
    7: ("repository-policy-rejects-action", "ACTION_BLOCKED"),
    8: ("probe-would-touch-default-branch", "ACTION_BLOCKED"),
    9: ("cutoff-after-create", "CLEANUP_REQUIRED"),
    10: ("cutoff-before-cleanup", "CLEANUP_REQUIRED"),
    11: ("duplicate-retry", "REUSE_ATTEMPT"),
    12: ("cleanup-permission-lost", "RECOVERY_REQUIRED"),
    13: ("locator-reused", "AMBIGUOUS"),
    14: ("cleanup-ack-lost", "PASSED_CLEAN"),
    15: ("repeated-cutoff-bounded", "CLEANUP_REQUIRED"),
    16: ("concurrent-generations-one-namespace", "ALLOW_FENCED"),
    17: ("old-probe-after-successor", "AUTHORITY_STALE"),
    18: ("stale-probe-cleanup-without-transfer", "RECOVERY_REQUIRED"),
    19: ("authority-lost-before-cleanup", "RECOVERY_REQUIRED"),
    20: ("forged-authority-metadata", "AUTHORITY_UNAVAILABLE"),
    21: ("bounded-recovery-transfer", "ALLOW_RECOVERY"),
    22: ("authority-adapter-incompatible", "AUTHORITY_UNAVAILABLE"),
    23: ("capability-current-authority-expired", "AUTHORITY_STALE"),
    24: ("context-recreated-successor", "REUSE_LINEAGE"),
    25: ("context-id-reused-different-installation", "LINEAGE_UNKNOWN"),
    26: ("product-migration-access-only", "LINEAGE_UNKNOWN"),
    27: ("successor-orphan-cleanup-needs-transfer", "RECOVERY_REQUIRED"),
    28: ("lineage-map-version-skew", "LINEAGE_UNKNOWN"),
}



def fail(message: str) -> None:
    raise SystemExit(f"FAIL: {message}")


def main() -> None:
    if len(sys.argv) > 2:
        fail("usage: check-github-capability-fixtures.py [fixture.json]")
    fixture = Path(sys.argv[1]) if len(sys.argv) == 2 else FIXTURE
    data = json.loads(fixture.read_text(encoding="utf-8"))
    if data.get("spec") != 5 or data.get("issue") != 11:
        fail("fixture must identify issue 11 spec 5")

    cases = data.get("cases")
    if not isinstance(cases, list):
        fail("cases must be a list")

    if any(not isinstance(case, dict) for case in cases):
        fail("every case must be an object")

    ids = [case.get("id") for case in cases]
    if any(type(cid) is not int for cid in ids):
        fail("fixture IDs must be integers")
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
    for cid, (name, expected) in CANONICAL_CASES.items():
        case = by_id[cid]
        if case.get("name") != name:
            fail(f"case {cid}: expected canonical scenario name {name!r}")
        if case.get("expected") != expected:
            fail(f"case {cid}: expected canonical outcome {expected}")

    print(f"PASS: {len(cases)} GitHub capability acceptance fixtures")


if __name__ == "__main__":
    main()
