#!/usr/bin/env python3
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures" / "merge-base-topology-spec2.json"


def canonical_bases(case):
    return sorted(set(case.get("bases", [])))


def topology(case):
    if not case.get("complete", False):
        return "UNPROVABLE"
    bases = canonical_bases(case)
    if not bases:
        return "NONE"
    if len(bases) == 1:
        return "UNIQUE"
    return "MULTIPLE"


def set_identity(case):
    payload = {"history_view": case["history_view"], "bases": canonical_bases(case)}
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def virtual_disposition(case):
    virtual = case.get("virtual")
    if topology(case) != "MULTIPLE":
        return None
    if not virtual or not virtual.get("supported", False):
        return "UNSUPPORTED"
    required = ("algorithm", "version", "options", "intermediates", "result")
    if any(k not in virtual for k in required):
        return "UNPROVABLE"
    return "SUPPORTED"


def computation_identity(case):
    if virtual_disposition(case) != "SUPPORTED":
        return None
    v = case["virtual"]
    payload = {
        "history_view": case["history_view"],
        "bases": canonical_bases(case),
        "algorithm": v["algorithm"],
        "version": v["version"],
        "options": v["options"],
        "intermediates": v["intermediates"],
        "result": v["result"],
    }
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def stale_against(case, previous):
    return (
        case["history_view"] != previous["history_view"]
        or canonical_bases(case) != sorted(set(previous.get("bases", [])))
    )


def main():
    cases = json.loads(FIXTURES.read_text())
    by_name = {c["name"]: c for c in cases}
    errors = []

    for case in cases:
        if "expect" in case and topology(case) != case["expect"]:
            errors.append(f"{case['name']}: expected {case['expect']}, got {topology(case)}")
        if "expect_virtual" in case and virtual_disposition(case) != case["expect_virtual"]:
            errors.append(f"{case['name']}: expected virtual {case['expect_virtual']}, got {virtual_disposition(case)}")
        if "same_set_as" in case and set_identity(case) != set_identity(by_name[case["same_set_as"]]):
            errors.append(f"{case['name']}: canonical base-set identity changed under reordering")
        if "same_computation_as" in case and computation_identity(case) != computation_identity(by_name[case["same_computation_as"]]):
            errors.append(f"{case['name']}: computation identity changed under equivalent reordering")
        if "different_computation_from" in case and computation_identity(case) == computation_identity(by_name[case["different_computation_from"]]):
            errors.append(f"{case['name']}: material computation drift aliased")
        if "stale_against" in case and not stale_against(case, case["stale_against"]):
            errors.append(f"{case['name']}: history/base drift was not stale")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        raise SystemExit(1)
    print(f"PASS: {len(cases)} merge-base topology fixtures")


if __name__ == "__main__":
    main()
