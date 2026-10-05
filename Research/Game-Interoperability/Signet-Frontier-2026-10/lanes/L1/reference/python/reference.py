#!/usr/bin/env python3
from __future__ import annotations
import hashlib
import json
import sys
from pathlib import Path

MAX_SAFE_INTEGER = 9007199254740991

class CompositionError(Exception):
    def __init__(self, reason: str):
        super().__init__(reason)
        self.reason = reason

def _utf16_key(value: str):
    raw = value.encode("utf-16-be")
    return tuple(int.from_bytes(raw[i:i+2], "big") for i in range(0, len(raw), 2))

def _quote(value: str) -> str:
    out = ['"']
    for ch in value:
        cp = ord(ch)
        if 0xD800 <= cp <= 0xDFFF:
            raise ValueError("JCS input contains a lone surrogate")
        if ch == '"':
            out.append('\\\"')
        elif ch == '\\':
            out.append('\\\\')
        elif ch == '\b':
            out.append('\\b')
        elif ch == '\t':
            out.append('\\t')
        elif ch == '\n':
            out.append('\\n')
        elif ch == '\f':
            out.append('\\f')
        elif ch == '\r':
            out.append('\\r')
        elif cp <= 0x1F:
            out.append(f"\\u{cp:04x}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)

def canonicalize(value) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        return _quote(value)
    if isinstance(value, int) and not isinstance(value, bool):
        if abs(value) > MAX_SAFE_INTEGER:
            raise ValueError("integer outside I-JSON interoperable range")
        return str(value)
    if isinstance(value, float):
        raise ValueError("floating-point normative values are outside this reference subset")
    if isinstance(value, list):
        return "[" + ",".join(canonicalize(item) for item in value) + "]"
    if isinstance(value, dict):
        if not all(isinstance(k, str) for k in value):
            raise ValueError("object keys must be strings")
        parts = []
        for key in sorted(value.keys(), key=_utf16_key):
            parts.append(_quote(key) + ":" + canonicalize(value[key]))
        return "{" + ",".join(parts) + "}"
    raise ValueError(f"unsupported JSON value: {type(value).__name__}")

def sha256_jcs(value) -> str:
    return "sha256:" + hashlib.sha256(canonicalize(value).encode("utf-8")).hexdigest()

def _ref_key(ref):
    return (ref["id"], ref["definition_hash"])

def compose(inp: dict) -> dict:
    catalog = {item["semantic"]["id"]: item for item in inp["catalog"]}
    state = {}
    closure = {}

    def visit(ref):
        semantic_id = ref["id"]
        requested_hash = ref["definition_hash"]
        if semantic_id not in catalog:
            raise CompositionError("REQUIRED_PROFILE_UNSUPPORTED")
        definition = catalog[semantic_id]
        actual_hash = definition["semantic"]["definition_hash"]
        if actual_hash != requested_hash:
            raise CompositionError("SEMANTIC_DEFINITION_CONFLICT")
        existing = closure.get(semantic_id)
        if existing and existing["semantic"]["definition_hash"] != requested_hash:
            raise CompositionError("SEMANTIC_DEFINITION_CONFLICT")
        if state.get(semantic_id) == 1:
            raise CompositionError("REQUIRED_PROFILE_CYCLE")
        if state.get(semantic_id) == 2:
            return
        state[semantic_id] = 1
        closure[semantic_id] = definition
        for dependency in definition.get("requires", []):
            visit(dependency)
        state[semantic_id] = 2

    for selected in inp["selected_profiles"]:
        visit(selected)

    active_ids = set(closure)
    for definition in closure.values():
        for conflict in definition.get("conflicts", []):
            if conflict["id"] in active_ids:
                other_hash = closure[conflict["id"]]["semantic"]["definition_hash"]
                if other_hash == conflict["definition_hash"]:
                    raise CompositionError("PROFILE_CONFLICT")

    concepts = {}
    for definition in closure.values():
        for exported in definition.get("exports", []):
            existing = concepts.get(exported["id"])
            if existing and existing["definition_hash"] != exported["definition_hash"]:
                raise CompositionError("CONCEPT_DEFINITION_CONFLICT")
            concepts[exported["id"]] = exported

    for definition in closure.values():
        for edge in definition.get("extends", []):
            if edge["child"] == edge["parent"] or edge["parent"] not in concepts or edge["child"] not in concepts:
                raise CompositionError("INVALID_EXTENSION_TARGET")

    ordering = inp.get("ordering_model")
    if ordering:
        wanted = _ref_key(ordering)
        for definition in closure.values():
            accepted = definition.get("ordering_constraints", {}).get("accepted", [])
            if accepted and wanted not in {_ref_key(item) for item in accepted}:
                raise CompositionError("ORDERING_MODEL_CONFLICT")

    visibility = inp.get("visibility_policy")
    if visibility:
        wanted = _ref_key(visibility)
        for definition in closure.values():
            accepted = definition.get("visibility_constraints", {}).get("accepted", [])
            if accepted and wanted not in {_ref_key(item) for item in accepted}:
                raise CompositionError("VISIBILITY_POLICY_CONFLICT")

    result = {
        "profiles": sorted([definition["semantic"] for definition in closure.values()], key=_ref_key),
        "concepts": sorted(concepts.values(), key=_ref_key),
    }
    if ordering:
        result["ordering_model"] = ordering
    if visibility:
        result["visibility_policy"] = visibility
    return result

def run_vectors(vector_dir: Path) -> None:
    failures = []

    hv = json.loads((vector_dir / "hash-vector-001.json").read_text())
    actual_jcs = canonicalize(hv["definition"]["normative"])
    actual_hash = sha256_jcs(hv["definition"]["normative"])
    if actual_jcs != hv["expected_jcs"]:
        failures.append("HASH-VECTOR-001 JCS mismatch")
    if actual_hash != hv["expected_definition_hash"]:
        failures.append("HASH-VECTOR-001 digest mismatch")

    cv = json.loads((vector_dir / "composition-vector-001.json").read_text())
    actual = compose(cv["input"])
    if actual != cv["expected_composition"]:
        failures.append("COMP-VECTOR-001 composition mismatch")
    if canonicalize(actual) != cv["expected_jcs"]:
        failures.append("COMP-VECTOR-001 JCS mismatch")
    if sha256_jcs(actual) != cv["expected_profile_set_hash"]:
        failures.append("COMP-VECTOR-001 digest mismatch")

    contract_vector = json.loads((vector_dir / "contract-vector-001.json").read_text())
    contract_jcs = canonicalize(contract_vector["contract"])
    contract_hash = sha256_jcs(contract_vector["contract"])
    if contract_jcs != contract_vector["expected_jcs"]:
        failures.append("CONTRACT-VECTOR-001 JCS mismatch")
    if contract_hash != contract_vector["expected_contract_hash"]:
        failures.append("CONTRACT-VECTOR-001 digest mismatch")

    negatives = json.loads((vector_dir / "composition-negative-vectors.json").read_text())
    for vector in negatives:
        try:
            compose(vector["input"])
        except CompositionError as exc:
            if exc.reason != vector["expected_reason"]:
                failures.append(f'{vector["vector_id"]} expected {vector["expected_reason"]}, got {exc.reason}')
        else:
            failures.append(f'{vector["vector_id"]} unexpectedly composed successfully')

    if failures:
        for failure in failures:
            print("FAIL:", failure, file=sys.stderr)
        raise SystemExit(1)
    print("PASS: HASH-VECTOR-001")
    print("PASS: COMP-VECTOR-001")
    print("PASS: CONTRACT-VECTOR-001")
    print(f"PASS: {len(negatives)} negative composition vectors")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: reference.py <test-vector-directory>", file=sys.stderr)
        raise SystemExit(2)
    run_vectors(Path(sys.argv[1]))
