#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def publication(case):
    if not case.get("inventory_complete", False):
        return "BLOCK_INVENTORY_UNKNOWN"
    if not case.get("schema_compatible", False):
        return "BLOCK_SCHEMA_INCOMPATIBLE"
    if not case.get("authority_current", False):
        return "BLOCK_AUTHORITY_STALE"
    count = case.get("matching_packets", 0)
    if count == 0:
        return "CREATE"
    if count == 1:
        if not case.get("content_identity_matches", True):
            return "BLOCK_LOCATOR_CONFLICT"
        return "ADOPT_EXISTING" if case.get("matching_complete", False) else "RECONCILE_INCOMPLETE"
    if case.get("all_semantically_equivalent", False):
        return "CANONICALIZE_DUPLICATES"
    return "BLOCK_LOCATOR_CONFLICT"


def identity(case):
    case_id = case["id"]
    if case_id in {"same-semantic-retry", "changed-content-new-identity"}:
        return "SAME_LINEAGE" if case["packet_identity"] == case["retry_packet_identity"] else "NEW_LINEAGE"
    if case_id == "temporary-cannot-claim-persistent-state":
        return "REJECT_IDENTITY_ESCALATION" if case.get("temporary_worker") and case.get("claims_persistent_identity") else "OK"
    if case_id == "packet-storage-not-policy":
        return "EVIDENCE_ONLY" if case.get("packet_complete") and not case.get("canonicalized") else "CANONICAL"
    if case_id == "exceptional-existing-surface-without-authority":
        return "READ_ONLY" if not case.get("explicit_existing_surface_authority") else "MUTATION_ELIGIBLE"
    raise ValueError(f"unknown identity case: {case_id}")


def source(case):
    if not case.get("default_head_resolved", False):
        return "UNKNOWN"
    return "CONSUME_UNIQUE_DELTA" if case.get("nondefault_unique_proven", False) else "UNKNOWN"


def main():
    fixture_path = Path(sys.argv[1] if len(sys.argv) > 1 else "Phase0/fixtures/ad-hoc-research-spec1.json")
    data = json.loads(fixture_path.read_text())
    failures = []
    groups = (("publication_cases", publication), ("identity_cases", identity), ("source_cases", source))
    total = 0
    for key, reducer in groups:
        for case in data[key]:
            total += 1
            actual = reducer(case)
            if actual != case["expected"]:
                failures.append((case["id"], case["expected"], actual))
    if failures:
        for failure in failures:
            print("FAIL", *failure)
        return 1
    print(f"ad-hoc research fixtures: {total} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
