#!/usr/bin/env python3
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "Phase0" / "fixtures" / "integration-candidate-v1.json"


def canonical_bytes(candidate):
    return json.dumps(candidate, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def candidate_id(candidate):
    return hashlib.sha256(canonical_bytes(candidate)).hexdigest()


def main():
    data = json.loads(FIXTURE.read_text(encoding="utf-8"))
    cases = {case["name"]: case for case in data["cases"]}

    normal = cases["normal-two-parent"]
    reversed_case = cases["reversed-parents-same-tree"]
    unsupported = cases["unsupported-three-parent"]
    migrated = cases["compatible-constructor-migration"]

    assert normal["candidate"]["parents"] == [
        normal["candidate"]["target_commit"],
        normal["candidate"]["source_commit"],
    ]
    assert normal["candidate"]["parent_count"] == len(normal["candidate"]["parents"]) == 2

    assert reversed_case["candidate"]["tree"] == normal["candidate"]["tree"]
    assert reversed_case["candidate"]["metadata"] == normal["candidate"]["metadata"]
    assert candidate_id(reversed_case["candidate"]) != candidate_id(normal["candidate"])

    support = set(unsupported["constructor_support"])
    assert unsupported["candidate"]["parent_count"] == 3
    assert unsupported["candidate"]["parent_count"] not in support
    assert unsupported["expect"] == "UNSUPPORTED_PARENT_CARDINALITY"

    assert migrated["candidate"]["parents"] == normal["candidate"]["parents"]
    assert migrated["candidate"]["tree"] == normal["candidate"]["tree"]
    assert migrated["candidate"]["constructor_version"] != normal["candidate"]["constructor_version"]
    assert migrated["candidate"]["compatibility_basis"] != normal["candidate"]["compatibility_basis"]
    assert migrated["expect"] == "SUPPORTED_COMPATIBLE_WITH:normal-two-parent"

    for stale in data["stale_head_cases"]:
        target_ok = stale["predicted_target"] == stale["live_target"]
        source_ok = stale["predicted_source"] == stale["live_source"]
        assert not (target_ok and source_ok)
        assert stale["expect"] == "STALE_OR_INCOMPATIBLE"

    print("integration candidate fixtures: OK")
    print("normal candidate id:", candidate_id(normal["candidate"]))
    print("reversed candidate id:", candidate_id(reversed_case["candidate"]))


if __name__ == "__main__":
    main()
