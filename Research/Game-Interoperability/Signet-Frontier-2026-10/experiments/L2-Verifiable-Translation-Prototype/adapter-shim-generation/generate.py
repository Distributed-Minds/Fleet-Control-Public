#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

GENERATOR_ID = "l2-adapter-shim-generator@0"
SUPPORTED_RETURNS = {"u32", "optional_f32", "optional_f64", "bool"}
SUPPORTED_PARAMS = {"string"}

def canonical_descriptor(data: dict) -> bytes:
    return (json.dumps(data, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")

def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def validate_descriptor(d: dict) -> None:
    if d.get("schema") != "signet-adapter-shim@0":
        raise ValueError("unsupported schema")
    names = set()
    for fn in d.get("functions", []):
        name = fn.get("name")
        if not name or name in names:
            raise ValueError(f"missing/duplicate function name: {name!r}")
        names.add(name)
        if fn.get("returns") not in SUPPORTED_RETURNS:
            raise ValueError(f"unsupported return type for {name}: {fn.get('returns')}")
        for p in fn.get("params", []):
            if p.get("type") not in SUPPORTED_PARAMS:
                raise ValueError(f"unsupported param type for {name}.{p.get('name')}: {p.get('type')}")

def c_param(p: dict) -> str:
    return f"const char *{p['name']}"

def render_c(d: dict, descriptor_sha: str) -> str:
    lines = [
        "/* GENERATED. DO NOT EDIT. */",
        f"/* generator: {GENERATOR_ID} */",
        f"/* descriptor-sha256: {descriptor_sha} */",
        "#ifndef SIGNET_ADAPTER_SHIM_RESEARCH_V0_H",
        "#define SIGNET_ADAPTER_SHIM_RESEARCH_V0_H",
        "#include <stdint.h>",
        "#ifdef __cplusplus",
        'extern "C" {',
        "#endif",
        "",
        "typedef struct SgAdapterShim SgAdapterShim;",
        ""
    ]
    for fn in d["functions"]:
        params = ["SgAdapterShim *shim"] + [c_param(p) for p in fn.get("params", [])]
        r = fn["returns"]
        if r == "u32":
            ret, extra = "uint32_t", []
        elif r == "bool":
            ret, extra = "int32_t", []
        elif r == "optional_f32":
            ret, extra = "int32_t", ["float *out_value"]
        elif r == "optional_f64":
            ret, extra = "int32_t", ["double *out_value"]
        else:
            raise AssertionError(r)
        params += extra
        lines.append(f"/* {fn.get('description','')} */")
        lines.append(f"{ret} sg_adapter_{fn['name']}({', '.join(params)});")
        lines.append("")
    lines += ["#ifdef __cplusplus", "}", "#endif", "#endif", ""]
    return "\n".join(lines)

def pascal(s: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in s.split("_"))

def render_rust(d: dict, descriptor_sha: str) -> str:
    lines = [
        "// GENERATED. DO NOT EDIT.",
        f"// generator: {GENERATOR_ID}",
        f"// descriptor-sha256: {descriptor_sha}",
        "",
        "pub trait SignetAdapterShim {"
    ]
    for fn in d["functions"]:
        params = [f"{p['name']}: &str" for p in fn.get("params", [])]
        r = {"u32":"u32", "bool":"bool", "optional_f32":"Option<f32>", "optional_f64":"Option<f64>"}[fn["returns"]]
        comma = ", " if params else ""
        lines.append(f"    /// {fn.get('description','')}")
        lines.append(f"    fn {fn['name']}(&mut self{comma}{', '.join(params)}) -> {r};")
    lines += ["}", ""]
    return "\n".join(lines)

def render_csharp(d: dict, descriptor_sha: str) -> str:
    lines = [
        "// GENERATED. DO NOT EDIT.",
        f"// generator: {GENERATOR_ID}",
        f"// descriptor-sha256: {descriptor_sha}",
        "#nullable enable",
        "namespace Signet.Research;",
        "",
        "public interface ISignetAdapterShim",
        "{"
    ]
    for fn in d["functions"]:
        params = []
        for p in fn.get("params", []):
            name = pascal(p["name"])
            params.append(f"string {name[0].lower() + name[1:]}")
        r = fn["returns"]
        if r == "u32":
            ret = "uint"
        elif r == "bool":
            ret = "bool"
        elif r == "optional_f32":
            ret = "float?"
        elif r == "optional_f64":
            ret = "double?"
        else:
            raise AssertionError(r)
        lines.append(f"    /// <summary>{fn.get('description','')}</summary>")
        lines.append(f"    {ret} {pascal(fn['name'])}({', '.join(params)});")
    lines += ["}", ""]
    return "\n".join(lines)

def render_ts(d: dict, descriptor_sha: str) -> str:
    lines = [
        "// GENERATED. DO NOT EDIT.",
        f"// generator: {GENERATOR_ID}",
        f"// descriptor-sha256: {descriptor_sha}",
        "",
        "export interface SignetAdapterShim {"
    ]
    for fn in d["functions"]:
        params = [f"{p['name']}: string" for p in fn.get("params", [])]
        ret = {"u32":"number", "bool":"boolean", "optional_f32":"number | null", "optional_f64":"number | null"}[fn["returns"]]
        lines.append(f"  /** {fn.get('description','')} */")
        lines.append(f"  {fn['name']}({', '.join(params)}): {ret};")
    lines += ["}", ""]
    return "\n".join(lines)

def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} DESCRIPTOR OUTPUT_DIR", file=sys.stderr)
        return 2
    src = Path(argv[1])
    out = Path(argv[2])
    d = json.loads(src.read_text(encoding="utf-8"))
    validate_descriptor(d)
    descriptor_sha = digest(canonical_descriptor(d))
    rendered = {
        "signet_adapter_shim.h": render_c(d, descriptor_sha),
        "signet_adapter_shim.rs": render_rust(d, descriptor_sha),
        "ISignetAdapterShim.cs": render_csharp(d, descriptor_sha),
        "signet-adapter-shim.ts": render_ts(d, descriptor_sha),
    }
    out.mkdir(parents=True, exist_ok=True)
    for name in sorted(rendered):
        (out / name).write_text(rendered[name], encoding="utf-8", newline="\n")
    print(f"descriptor-sha256: {descriptor_sha}")
    for name in sorted(rendered):
        print(f"{name}: sha256:{digest(rendered[name].encode('utf-8'))}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
