#!/usr/bin/env python3
import hashlib
import json
import sys
from pathlib import Path


SEMANTIC_IDENTITY_VERSION = "adhoc-packet-semantic-v1"
SEMANTIC_PACKET_FIELDS = (
    "packet_schema",
    "topic",
    "authoritative_baseline",
    "unique_nondefault_sources",
    "external_sources",
    "observations",
    "derived_conclusions",
    "predictions",
    "unknowns",
    "contradictions",
    "stale_source_warnings",
    "discovery_vocabulary",
    "affected_packages",
    "proposed_deltas",
    "unresolved_questions",
    "useful_next_actions",
)


def canonical_semantic_packet(packet):
    missing = [field for field in SEMANTIC_PACKET_FIELDS if field not in packet]
    if missing:
        raise ValueError(f"missing semantic packet fields: {','.join(missing)}")
    semantic = {field: packet[field] for field in SEMANTIC_PACKET_FIELDS}
    return json.dumps(
        semantic,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        allow_nan=False,
    ).encode("utf-8")


def semantic_packet_identity(packet):
    digest = hashlib.sha256(canonical_semantic_packet(packet)).hexdigest()
    return f"{SEMANTIC_IDENTITY_VERSION}:sha256:{digest}"


def packets_match(left, right):
    return semantic_packet_identity(left) == semantic_packet_identity(right)


def publication(case):
    if not case.get("inventory_complete", False):
        return "BLOCK_INVENTORY_UNKNOWN"
    if not case.get("schema_compatible", False):
        return "BLOCK_SCHEMA_INCOMPATIBLE"
    if not case.get("authority_current", False):
        return "BLOCK_AUTHORITY_STALE"
    if "packet" not in case:
        return "BLOCK_PACKET_IDENTITY_UNKNOWN"
    candidate_id = semantic_packet_identity(case["packet"])
    count = case.get("matching_packets", 0)
    if count == 0:
        return "CREATE"
    if count == 1:
        matching = case.get("matching_packet")
        if matching is None or semantic_packet_identity(matching) != candidate_id:
            return "BLOCK_LOCATOR_CONFLICT"
        return "ADOPT_EXISTING" if case.get("matching_complete", False) else "RECONCILE_INCOMPLETE"
    matches = case.get("matching_packet_payloads", [])
    if len(matches) != count:
        return "BLOCK_LOCATOR_CONFLICT"
    if all(semantic_packet_identity(packet) == candidate_id for packet in matches):
        return "CANONICALIZE_DUPLICATES"
    return "BLOCK_LOCATOR_CONFLICT"


def identity(case):
    case_id = case["id"]
    if case_id in {
        "same-semantic-retry",
        "presentation-only-variance-same-identity",
        "changed-content-new-identity",
        "changed-source-new-identity",
    }:
        return "SAME_LINEAGE" if packets_match(case["packet"], case["retry_packet"]) else "NEW_LINEAGE"
    if case_id in {"temporary-cannot-claim-persistent-state", "shared-transport-principal-not-persistent-identity"}:
        return "REJECT_IDENTITY_ESCALATION" if case.get("claims_persistent_identity") else "OK"
    if case_id == "packet-storage-not-policy":
        return "EVIDENCE_ONLY" if case.get("packet_complete") and not case.get("canonicalized") else "CANONICAL"
    if case_id == "recursive-derivative-not-independent":
        return "SAME_LINEAGE" if case.get("same_upstream_lineage") else "INDEPENDENT"
    if case_id == "pure-research-branch-creation":
        return "REJECT_BRANCH_CREATION" if case.get("pure_research") and case.get("attempts_git_branch") else "OK"
    if case_id == "exceptional-existing-surface-without-authority":
        return "READ_ONLY" if not case.get("explicit_existing_surface_authority") else "MUTATION_ELIGIBLE"
    if case_id == "exceptional-mutation-active-collision":
        if not case.get("explicit_existing_surface_authority"):
            return "READ_ONLY"
        return "YIELD" if case.get("active_overlap") and not case.get("safe_takeover") else "MUTATION_ELIGIBLE"
    raise ValueError(f"unknown identity case: {case_id}")


def source(case):
    if not case.get("default_head_resolved", False):
        return "UNKNOWN"
    if case.get("default_head_moved", False):
        return "REVALIDATE"
    if case.get("source_ref_deleted", False):
        return "PRESERVE_PROVENANCE" if case.get("durable_content_identity", False) else "UNKNOWN"
    if case.get("same_path_competes", False):
        return "USE_DEFAULT_AUTHORITY"
    return "CONSUME_UNIQUE_DELTA" if case.get("nondefault_unique_proven", False) else "SKIP_NO_UNIQUE_DELTA"


def recovery(case):
    if not case.get("authority_current", False):
        return "READ_ONLY_RECONCILE" if case.get("created", False) else "BLOCK_AUTHORITY_STALE"
    if not case.get("inventory_complete", True):
        return "BLOCK_INVENTORY_UNKNOWN"
    if not case.get("schema_compatible", False):
        return "BLOCK_SCHEMA_INCOMPATIBLE"
    if "packet" not in case:
        return "BLOCK_PACKET_IDENTITY_UNKNOWN"
    semantic_packet_identity(case["packet"])
    if case.get("created", False):
        matching = case.get("matching_packet")
        if matching is None or not packets_match(case["packet"], matching):
            return "BLOCK_LOCATOR_CONFLICT"
        return "ADOPT_EXISTING" if case.get("matching_complete", False) else "RECONCILE_INCOMPLETE"
    return "RECONCILE_THEN_CREATE_IF_ABSENT"


def concurrent_publication(case):
    if not case.get("authority_current", False):
        initial = case.get("initial_packets", 0)
        return {"disposition":"BLOCK_AUTHORITY_STALE","provider_artifacts":initial,"canonical_lineages":initial}
    if not case.get("same_semantic_identity", False):
        initial = case.get("initial_packets", 0)
        return {"disposition":"DISTINCT_LINEAGES","provider_artifacts":initial,"canonical_lineages":initial}

    strategy = case["strategy"]
    publishers = int(case.get("publishers", 2))
    artifacts = int(case.get("initial_packets", 0))
    lineages = 1 if artifacts else 0

    if strategy in {"atomic_unique", "serialized"}:
        if artifacts == 0 and publishers:
            artifacts = 1
            lineages = 1
    elif strategy == "reconcile_after_create":
        artifacts += publishers
        if not case.get("inventory_complete", False) or not case.get("semantic_identity_indexed", False):
            return {"disposition":"BLOCK_INVENTORY_UNKNOWN","provider_artifacts":artifacts,"canonical_lineages":max(lineages, artifacts)}
        lineages = 1 if artifacts else 0
    elif strategy == "blind_presearch":
        artifacts += publishers
        lineages = artifacts
    else:
        raise ValueError(f"unknown concurrency strategy: {strategy}")

    if case.get("lost_ack_retry", False):
        if strategy in {"atomic_unique", "serialized"}:
            artifacts = max(artifacts, 1)
            lineages = 1
        elif strategy == "reconcile_after_create" and not case.get("inventory_complete", True):
            return {"disposition":"BLOCK_INVENTORY_UNKNOWN","provider_artifacts":artifacts,"canonical_lineages":lineages}

    disposition = "ONE_CANONICAL_LINEAGE" if lineages <= 1 else "BLOCK_UNSAFE_ADAPTER"
    return {"disposition":disposition,"provider_artifacts":artifacts,"canonical_lineages":lineages}


def packet_fields(case):
    required = (
        "stale_source_warnings",
        "discovery_vocabulary",
        "useful_next_actions",
    )
    if any(field not in case for field in required):
        return "REJECT_INCOMPLETE_PACKET"
    return "PRESERVE_REQUIRED_FIELDS"


def expand_packet_refs(case, templates):
    expanded = dict(case)
    for target in ("packet", "retry_packet", "matching_packet"):
        ref_key = f"{target}_ref"
        if ref_key in expanded:
            expanded[target] = templates[expanded.pop(ref_key)]
    if "matching_packet_refs" in expanded:
        expanded["matching_packet_payloads"] = [templates[ref] for ref in expanded.pop("matching_packet_refs")]
    return expanded


def check_expected(case, actual):
    expected = case["expected"]
    if actual != expected:
        raise AssertionError(f"{case['id']}: expected {expected!r}, got {actual!r}")


def main():
    fixture_path = Path(sys.argv[1] if len(sys.argv) > 1 else "Phase0/fixtures/ad-hoc-research-spec1.json")
    data = json.loads(fixture_path.read_text(encoding="utf-8"))
    templates = data.get("packet_templates", {})
    failures = []
    groups = (
        ("publication_cases", publication),
        ("identity_cases", identity),
        ("source_cases", source),
        ("recovery_cases", recovery),
        ("concurrency_cases", concurrent_publication),
        ("packet_field_cases", packet_fields),
    )
    total = 0
    for key, reducer in groups:
        for raw_case in data[key]:
            total += 1
            case = expand_packet_refs(raw_case, templates)
            try:
                actual = reducer(case)
                check_expected(case, actual)
            except Exception as exc:
                failures.append((case["id"], str(exc)))
    if failures:
        for failure in failures:
            print("FAIL", *failure)
        return 1
    print(f"ad-hoc research fixtures: {total} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
