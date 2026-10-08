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

class SetArrayError(Exception):
    def __init__(self, reason: str, detail: str = ""):
        super().__init__(f"{reason} ({detail})" if detail else reason)
        self.reason = reason
        self.detail = detail

SET_ARRAY_NOT_CANONICAL = "SET_ARRAY_NOT_CANONICAL"

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
    # SEM-ORDER-1: lexicographic (id, definition_hash) over UTF-16 code units,
    # matching RFC 8785 object-property ordering and ECMAScript string comparison.
    return (_utf16_key(ref["id"]), _utf16_key(ref["definition_hash"]))

def _string_key(value: str):
    return _utf16_key(value)

def _require_strictly_ascending(keys, field: str):
    if keys != sorted(keys) or len(set(keys)) != len(keys):
        raise SetArrayError(SET_ARRAY_NOT_CANONICAL, field)

def _validate_ref_array(refs, field: str):
    keys = [_ref_key(item) for item in refs]
    _require_strictly_ascending(keys, field)
    ids = [item["id"] for item in refs]
    if len(set(ids)) != len(ids):
        raise SetArrayError(SET_ARRAY_NOT_CANONICAL, f"{field}:duplicate_id")

def validate_set_arrays(normative: dict) -> None:
    """HASH05: reject unsorted/duplicate values in schema-declared set arrays."""
    for field in ("requires", "optional_requires", "conflicts", "exports"):
        if field in normative:
            _validate_ref_array(normative[field], field)
    if "extends" in normative:
        pairs = [
            (_string_key(edge["child"]), _string_key(edge["parent"]))
            for edge in normative["extends"]
        ]
        _require_strictly_ascending(pairs, "extends")
    for field in ("ordering_constraints", "visibility_constraints"):
        if field in normative:
            _validate_ref_array(normative[field].get("accepted", []), f"{field}.accepted")
    if "authority_scopes" in normative:
        _require_strictly_ascending(
            [_string_key(scope) for scope in normative["authority_scopes"]],
            "authority_scopes",
        )

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
        for dependency in sorted(definition.get("requires", []), key=_ref_key):
            visit(dependency)
        state[semantic_id] = 2

    for selected in sorted(inp["selected_profiles"], key=_ref_key):
        visit(selected)

    for definition in sorted(closure.values(), key=lambda item: _ref_key(item["semantic"])):
        for optional in sorted(definition.get("optional_requires", []), key=_ref_key):
            if optional["id"] in closure:
                active_hash = closure[optional["id"]]["semantic"]["definition_hash"]
                if active_hash != optional["definition_hash"]:
                    raise CompositionError("SEMANTIC_DEFINITION_CONFLICT")

    active_ids = set(closure)
    for definition in sorted(closure.values(), key=lambda item: _ref_key(item["semantic"])):
        for conflict in sorted(definition.get("conflicts", []), key=_ref_key):
            if conflict["id"] in active_ids:
                other_hash = closure[conflict["id"]]["semantic"]["definition_hash"]
                if other_hash == conflict["definition_hash"]:
                    raise CompositionError("PROFILE_CONFLICT")

    concepts = {}
    for definition in sorted(closure.values(), key=lambda item: _ref_key(item["semantic"])):
        for exported in definition.get("exports", []):
            existing = concepts.get(exported["id"])
            if existing and existing["definition_hash"] != exported["definition_hash"]:
                raise CompositionError("CONCEPT_DEFINITION_CONFLICT")
            concepts[exported["id"]] = exported

    for definition in sorted(closure.values(), key=lambda item: _ref_key(item["semantic"])):
        for edge in definition.get("extends", []):
            if edge["child"] == edge["parent"] or edge["parent"] not in concepts or edge["child"] not in concepts:
                raise CompositionError("INVALID_EXTENSION_TARGET")

    ordering = inp.get("ordering_model")
    if ordering is None and "ordering_preferences" in inp:
        for candidate in inp["ordering_preferences"]:
            wanted = _ref_key(candidate)
            if all(
                not definition.get("ordering_constraints", {}).get("accepted", [])
                or wanted in {
                    _ref_key(item)
                    for item in definition.get("ordering_constraints", {}).get("accepted", [])
                }
                for definition in closure.values()
            ):
                ordering = candidate
                break
        if inp["ordering_preferences"] and ordering is None:
            raise CompositionError("ORDERING_MODEL_CONFLICT")

    if ordering:
        wanted = _ref_key(ordering)
        for definition in closure.values():
            accepted = definition.get("ordering_constraints", {}).get("accepted", [])
            if accepted and wanted not in {_ref_key(item) for item in accepted}:
                raise CompositionError("ORDERING_MODEL_CONFLICT")

    visibility = inp.get("visibility_policy")
    if visibility is None and "visibility_preferences" in inp:
        for candidate in inp["visibility_preferences"]:
            wanted = _ref_key(candidate)
            if all(
                not definition.get("visibility_constraints", {}).get("accepted", [])
                or wanted in {
                    _ref_key(item)
                    for item in definition.get("visibility_constraints", {}).get("accepted", [])
                }
                for definition in closure.values()
            ):
                visibility = candidate
                break
        if inp["visibility_preferences"] and visibility is None:
            raise CompositionError("VISIBILITY_POLICY_CONFLICT")

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

def negotiate_optional(inp: dict) -> dict:
    selected = list(inp["selected_profiles"])
    baseline_input = dict(inp)
    baseline_input["selected_profiles"] = selected
    composition = compose(baseline_input)

    activated = []
    skipped = []

    for candidate in inp.get("optional_profile_preferences", []):
        trial = dict(inp)
        trial["selected_profiles"] = selected + [candidate]
        try:
            trial_composition = compose(trial)
        except CompositionError as exc:
            skipped.append({"profile": candidate, "reason": exc.reason})
            continue

        selected.append(candidate)
        activated.append(candidate)
        composition = trial_composition

    return {
        "composition": composition,
        "activated_optional": activated,
        "skipped_optional": skipped,
    }


def run_vectors(vector_dir: Path) -> None:
    failures = []

    hv = json.loads((vector_dir / "hash-vector-001.json").read_text())
    try:
        validate_set_arrays(hv["definition"]["normative"])
    except SetArrayError as exc:
        failures.append(f"HASH-VECTOR-001 unexpected set-array rejection: {exc.reason}")
    actual_jcs = canonicalize(hv["definition"]["normative"])
    actual_hash = sha256_jcs(hv["definition"]["normative"])
    if actual_jcs != hv["expected_jcs"]:
        failures.append("HASH-VECTOR-001 JCS mismatch")
    if actual_hash != hv["expected_definition_hash"]:
        failures.append("HASH-VECTOR-001 digest mismatch")

    set_vectors = json.loads((vector_dir / "hash-set-array-vectors.json").read_text())
    for vector in set_vectors:
        expected_reason = vector.get("expected_reason")
        try:
            validate_set_arrays(vector["normative"])
        except SetArrayError as exc:
            if expected_reason is None:
                failures.append(f'{vector["vector_id"]} unexpectedly rejected: {exc.reason}')
            elif exc.reason != expected_reason:
                failures.append(f'{vector["vector_id"]} expected {expected_reason}, got {exc.reason}')
        else:
            if expected_reason is not None:
                failures.append(f'{vector["vector_id"]} expected {expected_reason}, accepted instead')
            expected_hash = vector.get("expected_definition_hash")
            if expected_hash is not None and sha256_jcs(vector["normative"]) != expected_hash:
                failures.append(f'{vector["vector_id"]} digest mismatch')

    cv = json.loads((vector_dir / "composition-vector-001.json").read_text())
    actual = compose(cv["input"])
    if actual != cv["expected_composition"]:
        failures.append("COMP-VECTOR-001 composition mismatch")
    if canonicalize(actual) != cv["expected_jcs"]:
        failures.append("COMP-VECTOR-001 JCS mismatch")
    if sha256_jcs(actual) != cv["expected_profile_set_hash"]:
        failures.append("COMP-VECTOR-001 digest mismatch")

    additional = json.loads((vector_dir / "composition-additional-vectors.json").read_text())
    for vector in additional:
        actual = compose(vector["input"])
        if actual != vector["expected_composition"]:
            failures.append(f'{vector["vector_id"]} composition mismatch')
        if sha256_jcs(actual) != vector["expected_profile_set_hash"]:
            failures.append(f'{vector["vector_id"]} digest mismatch')

    unicode_vectors = json.loads((vector_dir / "composition-unicode-vectors.json").read_text())
    for vector in unicode_vectors:
        actual = compose(vector["input"])
        if actual != vector["expected_composition"]:
            failures.append(f'{vector["vector_id"]} composition mismatch')
        if canonicalize(actual) != vector["expected_jcs"]:
            failures.append(f'{vector["vector_id"]} JCS mismatch')
        if sha256_jcs(actual) != vector["expected_profile_set_hash"]:
            failures.append(f'{vector["vector_id"]} digest mismatch')

    negotiation_vector = json.loads((vector_dir / "negotiation-vector-001.json").read_text())
    negotiation_result = compose(negotiation_vector["input"])
    if negotiation_result != negotiation_vector["expected_composition"]:
        failures.append("NEGOTIATION-VECTOR-001 composition mismatch")
    if sha256_jcs(negotiation_result) != negotiation_vector["expected_profile_set_hash"]:
        failures.append("NEGOTIATION-VECTOR-001 digest mismatch")

    optional_vector = json.loads((vector_dir / "optional-negotiation-vector-001.json").read_text())
    optional_result = negotiate_optional(optional_vector["input"])
    if optional_result != optional_vector["expected_result"]:
        failures.append("OPTIONAL-NEGOTIATION-VECTOR-001 result mismatch")
    if sha256_jcs(optional_result) != optional_vector["expected_result_hash"]:
        failures.append("OPTIONAL-NEGOTIATION-VECTOR-001 digest mismatch")

    contract_vector = json.loads((vector_dir / "contract-vector-001.json").read_text())
    try:
        _validate_ref_array(contract_vector["contract"]["profiles"], "contract.profiles")
    except SetArrayError as exc:
        failures.append(f"CONTRACT-VECTOR-001 unexpected set-array rejection: {exc.reason}")
    contract_jcs = canonicalize(contract_vector["contract"])
    contract_hash = sha256_jcs(contract_vector["contract"])
    if contract_jcs != contract_vector["expected_jcs"]:
        failures.append("CONTRACT-VECTOR-001 JCS mismatch")
    if contract_hash != contract_vector["expected_contract_hash"]:
        failures.append("CONTRACT-VECTOR-001 digest mismatch")

    set_order_negatives = json.loads((vector_dir / "composition-set-order-vectors.json").read_text())
    for vector in set_order_negatives:
        try:
            compose(vector["input"])
        except CompositionError as exc:
            if exc.reason != vector["expected_reason"]:
                failures.append(f'{vector["vector_id"]} expected {vector["expected_reason"]}, got {exc.reason}')
        else:
            failures.append(f'{vector["vector_id"]} unexpectedly composed successfully')

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
    print("PASS: NEGOTIATION-VECTOR-001")
    print("PASS: OPTIONAL-NEGOTIATION-VECTOR-001")
    print("PASS: CONTRACT-VECTOR-001")
    print(f"PASS: {len(additional)} additional composition vectors")
    print(f"PASS: {len(negatives)} negative composition vectors")
    print(f"PASS: {len(set_vectors)} HASH05 set-array vectors")
    print(f"PASS: {len(unicode_vectors)} non-BMP composition vectors")
    print(f"PASS: {len(set_order_negatives)} deterministic set-order negative vectors")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: reference.py <test-vector-directory>", file=sys.stderr)
        raise SystemExit(2)
    run_vectors(Path(sys.argv[1]))
