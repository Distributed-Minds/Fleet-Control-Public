#!/usr/bin/env python3
from __future__ import annotations
import hashlib
import json
import re
import sys
from pathlib import Path

GENERATOR = "l2-role-contract-generator@1"

ROLE_AUTHORITY = {
    "HOST": {"INTEGRATION_EVIDENCE"},
    "IMPORTER": {"SEMANTIC_MAPPING", "LOCAL_OBSERVATION"},
    "WORLD": {"SEMANTIC_MAPPING", "LOCAL_PRESENTATION"},
    "INPUT": {"LOCAL_OBSERVATION", "SEMANTIC_MAPPING"},
    "PRESENTATION": {"LOCAL_PRESENTATION"},
    "AUTHORITY": {"AUTHORITY_CORRECTION"},
}

def canonical(obj: dict) -> bytes:
    return (json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def macro(name: str) -> str:
    return re.sub(r"[^A-Za-z0-9]+", "_", name).upper().strip("_")

def rust_variant(name: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in re.split(r"[^A-Za-z0-9]+", name) if part)

def validate_descriptor(d: dict) -> None:
    if d.get("schema") != "signet-adapter-role-contract@1":
        raise ValueError("unsupported schema")
    operations = d.get("operations")
    if not isinstance(operations, list) or not operations:
        raise ValueError("operations must be non-empty")

    seen = set()
    for op in operations:
        oid = op.get("id", "")
        if not re.fullmatch(r"[a-z][a-z0-9_.-]+", oid):
            raise ValueError(f"invalid operation id: {oid!r}")
        if oid in seen:
            raise ValueError(f"duplicate operation id: {oid}")
        seen.add(oid)

        role = op.get("role")
        authority = op.get("authority_class")
        if role not in ROLE_AUTHORITY:
            raise ValueError(f"unknown role: {role}")
        if authority not in ROLE_AUTHORITY[role]:
            raise ValueError(f"authority {authority} incompatible with role {role} for {oid}")
        if not op.get("local_mechanism"):
            raise ValueError(f"missing local mechanism: {oid}")

def generated_manifest(d: dict, descriptor_sha: str) -> dict:
    return {
        "schema": "signet-adapter-generated-operations@1",
        "generator": GENERATOR,
        "descriptor_sha256": descriptor_sha,
        "target": d["target"],
        "integration_mode": d["integration_mode"],
        "surface_fingerprint": d.get("surface_fingerprint"),
        "operations": [
            {key: op.get(key) for key in (
                "id", "role", "direction", "local_mechanism",
                "semantic_contract_ref", "authority_class", "evidence"
            )}
            for op in sorted(d["operations"], key=lambda op: op["id"])
        ],
    }

def render_c(d: dict, descriptor_sha: str) -> str:
    lines = [
        "/* GENERATED. DO NOT EDIT. */",
        f"/* generator: {GENERATOR} */",
        f"/* descriptor-sha256: {descriptor_sha} */",
        "#pragma once",
        "#include <stddef.h>",
        "#include <stdint.h>",
        "typedef struct SgAdapterContext SgAdapterContext;",
        "typedef struct { const uint8_t *ptr; size_t len; } SgAdapterBytes;",
        "typedef struct { uint8_t *ptr; size_t capacity; size_t *written; } SgAdapterOut;",
        "",
    ]
    for op in sorted(d["operations"], key=lambda op: op["id"]):
        m = macro(op["id"])
        fn = op["id"].replace(".", "_").replace("-", "_")
        lines.append(f'#define SG_ADAPTER_OP_{m} "{op["id"]}"')
        lines.append(f'#define SG_ADAPTER_ROLE_{m} "{op["role"]}"')
        if op.get("semantic_contract_ref") is not None:
            lines.append(f'#define SG_ADAPTER_SEMANTIC_{m} "{op["semantic_contract_ref"]}"')
        lines.append(
            f"int32_t sg_adapter_op_{fn}(SgAdapterContext *ctx, "
            "SgAdapterBytes request, SgAdapterOut response);"
        )
        lines.append("")
    return "\n".join(lines) + "\n"

def render_rust(d: dict, descriptor_sha: str) -> str:
    operations = sorted(d["operations"], key=lambda op: op["id"])
    lines = [
        "// GENERATED. DO NOT EDIT.",
        f"// generator: {GENERATOR}",
        f"// descriptor-sha256: {descriptor_sha}",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
        "pub enum AdapterOperation {",
    ]
    for op in operations:
        lines.append(f"    {rust_variant(op['id'])},")
    lines += ["}", "", "pub trait SignetAdapterRoleOps {"]
    for op in operations:
        fn = op["id"].replace(".", "_").replace("-", "_")
        lines.append(
            f"    /// role={op['role']}; authority={op['authority_class']}; evidence={op['evidence']}"
        )
        lines.append(
            f"    fn {fn}(&mut self, request: &[u8], response: &mut Vec<u8>) -> Result<(), i32>;"
        )
    lines += ["}", ""]
    return "\n".join(lines)

def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} DESCRIPTOR OUTPUT_DIR", file=sys.stderr)
        return 2

    source = Path(argv[1])
    output = Path(argv[2])
    descriptor = json.loads(source.read_text(encoding="utf-8"))
    validate_descriptor(descriptor)

    descriptor_sha = sha256(canonical(descriptor))
    rendered = {
        "adapter.operations.json": json.dumps(
            generated_manifest(descriptor, descriptor_sha),
            indent=2,
            ensure_ascii=False,
        ) + "\n",
        "signet_adapter_role_ops.h": render_c(descriptor, descriptor_sha),
        "signet_adapter_role_ops.rs": render_rust(descriptor, descriptor_sha),
    }

    output.mkdir(parents=True, exist_ok=True)
    print(f"descriptor-sha256: {descriptor_sha}")
    for name in sorted(rendered):
        payload = rendered[name]
        (output / name).write_text(payload, encoding="utf-8", newline="\n")
        print(f"{name}: sha256:{sha256(payload.encode('utf-8'))}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
