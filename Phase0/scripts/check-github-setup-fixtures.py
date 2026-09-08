#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def capability(case):
    if not case.get("evidence_current", True):
        return "STALE"
    if case.get("scheduled_capability", "NONE") != "WRITE":
        return "CAPABILITY_ABSENT"
    if case.get("approval_required", False):
        return "ACTION_BLOCKED"
    if not case.get("repository_scope_matches", True):
        return "REPOSITORY_SCOPE_MISMATCH"
    if not case.get("repository_policy_allows", True):
        return "POLICY_BLOCKED"
    if case.get("requires_default_branch_mutation", False):
        return "DEFAULT_BRANCH_FORBIDDEN"
    if not case.get("lineage_current", True):
        return "LINEAGE_UNKNOWN"
    if not case.get("authority_adapter_compatible", True):
        return "AUTHORITY_UNAVAILABLE"
    if not case.get("authority_current", True):
        return "AUTHORITY_STALE"
    return "MUTATION_ELIGIBLE"


def probe(case):
    if not case.get("resource_incarnation_matches", True):
        return "AMBIGUOUS"
    if not case.get("authority_adapter_compatible", True):
        return "AUTHORITY_UNAVAILABLE"
    created = case.get("created", False)
    if created and case.get("cleaned", False) and case.get("cleanup_receipt", False):
        return "PASSED_CLEAN"
    if created and not case.get("authority_current", True):
        if case.get("recovery_authority_current", False):
            return "RECOVERY_AUTHORIZED"
        if case.get("verified", False):
            return "RECOVERY_REQUIRED"
        return "READ_ONLY_RECONCILE"
    if not case.get("authority_current", True):
        return "AUTHORITY_STALE"
    if not case.get("lineage_current", True):
        return "LINEAGE_UNKNOWN"
    if created:
        if not case.get("verified", False):
            return "RECONCILE_EXISTING"
        if not case.get("cleaned", False):
            return "CLEANUP_REQUIRED"
    return "MUTATION_ELIGIBLE"


def lineage(case):
    if case.get("lineage_map_conflict", False):
        return "LINEAGE_UNKNOWN"
    if not case.get("same_installation", False):
        return "LINEAGE_UNKNOWN"
    if not case.get("successor_evidence_current", False):
        return "LINEAGE_UNKNOWN"
    if case.get("unresolved_prior_probe", False):
        if case.get("cleanup_needed", False) and not case.get("recovery_authority_current", False):
            return "RECOVERY_REQUIRED"
        return "RECONCILE_PRIOR"
    return "LINEAGE_CURRENT"


def main():
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "Phase0/fixtures/github-setup-spec5.json")
    data = json.loads(path.read_text())
    failures = []
    total = 0
    for group, reducer in (
        ("capability_cases", capability),
        ("probe_cases", probe),
        ("lineage_cases", lineage),
    ):
        for case in data[group]:
            total += 1
            actual = reducer(case)
            if actual != case["expected"]:
                failures.append((case["id"], case["expected"], actual))
    if failures:
        for item in failures:
            print("FAIL", *item)
        return 1
    print(f"github setup spec5 fixtures: {total} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
