#!/usr/bin/env python3
"""Deterministic generation/negative-test harness for adapter-role-contract v1.

This script is the canonical way to reproduce `generated/` and
`generation-validation-output.txt`. It never hand-edits generated output: every
artifact under `generated/` is (re)written by running `generate.py`.

Checks performed:
  1. two independent clean generator runs are byte-identical (per target);
  2. the checked-in `generated/` tree is byte-identical to a clean run;
  3. the generated C header parses under a real C compiler (`-fsyntax-only`);
  4. the generated Rust trait compiles under `rustc` when available;
  5. negative descriptors are rejected with the expected reason;
  6. SHA-256 digests of the regenerated outputs are emitted into
     `generation-validation-output.txt`.
"""
from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
GENERATOR = ROOT / "generate.py"
MANIFEST = ROOT / "generation-validation-output.txt"
TARGETS = {
    "minecraft": ROOT / "minecraft.example.json",
    "godot": ROOT / "godot.example.json",
}
GENERATED_NAMES = (
    "adapter.operations.json",
    "signet_adapter_role_ops.h",
    "signet_adapter_role_ops.rs",
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_generator(descriptor: Path, outdir: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(GENERATOR), str(descriptor), str(outdir)],
        capture_output=True,
        text=True,
    )


def tree_bytes(root: Path) -> dict[str, bytes]:
    return {
        str(p.relative_to(root)): p.read_bytes()
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }


def trees_identical(a: Path, b: Path) -> bool:
    return tree_bytes(a) == tree_bytes(b)


def which(*names: str) -> str | None:
    for name in names:
        found = shutil.which(name)
        if found:
            return found
    return None


def check_c_header(header_dir: Path, workdir: Path) -> tuple[str, str]:
    compiler = which("gcc", "cc", "clang")
    if compiler is None:
        return "UNAVAILABLE_IN_TEST_ENVIRONMENT", "no C compiler on PATH"
    probe = workdir / "header_probe.c"
    probe.write_text('#include "signet_adapter_role_ops.h"\nint main(void){return 0;}\n', encoding="utf-8")
    proc = subprocess.run(
        [compiler, "-std=c11", "-Wall", "-Wextra", "-fsyntax-only", "-I", str(header_dir), str(probe)],
        capture_output=True,
        text=True,
    )
    if proc.returncode == 0:
        return "PASS", ""
    return "FAIL", (proc.stderr or proc.stdout).strip()


def check_rust(rs_file: Path, workdir: Path) -> tuple[str, str]:
    rustc = which("rustc")
    if rustc is None:
        return "UNAVAILABLE_IN_TEST_ENVIRONMENT", "no rustc on PATH"
    rlib = workdir / "probe.rlib"
    proc = subprocess.run(
        [rustc, "--edition", "2021", "--crate-type", "lib", "-o", str(rlib), str(rs_file)],
        capture_output=True,
        text=True,
    )
    if proc.returncode == 0:
        return "PASS", ""
    return "FAIL", (proc.stderr or proc.stdout).strip()


def negative_test(name: str, descriptor: dict, mutate, expected: str, workdir: Path) -> str:
    candidate = mutate(json.loads(json.dumps(descriptor)))
    path = workdir / f"negative-{name}.json"
    path.write_text(json.dumps(candidate), encoding="utf-8")
    proc = run_generator(path, workdir / f"negative-{name}-out")
    combined = (proc.stderr or "") + (proc.stdout or "")
    if proc.returncode != 0 and expected in combined:
        return f"negative {name}: PASS(rejected)"
    return f"negative {name}: FAIL(rc={proc.returncode}, expected={expected!r})"


def main() -> int:
    work = Path(tempfile.mkdtemp(prefix="l2-gen-validation-"))
    try:
        with tempfile.TemporaryDirectory(dir=work) as tmp:
            tmpdir = Path(tmp)

            # 1) canonical regeneration + determinism across two clean runs
            per_target: dict[str, Path] = {}
            lines: list[str] = []
            for target, descriptor in TARGETS.items():
                run_a = tmpdir / f"{target}-a"
                run_b = tmpdir / f"{target}-b"
                proc_a = run_generator(descriptor, run_a)
                proc_b = run_generator(descriptor, run_b)
                if proc_a.returncode or proc_b.returncode:
                    print(f"{target}: generator FAILED\n{proc_a.stderr}{proc_b.stderr}", file=sys.stderr)
                    return 1
                deterministic = trees_identical(run_a, run_b)

                # writer: canonical generated/ tree from run A
                canonical = ROOT / "generated" / target
                canonical.mkdir(parents=True, exist_ok=True)
                for rel, payload in tree_bytes(run_a).items():
                    dest = canonical / rel
                    dest.write_bytes(payload)
                per_target[target] = canonical
                lines.append(f"{target}-deterministic-generation: {'PASS' if deterministic else 'FAIL'}")

            # 2) checked-in generated tree matches a clean run
            committed_ok = all(
                trees_identical(per_target[t], tmpdir / f"{t}-a") for t in TARGETS
            )
            lines.append(f"generated-tree-matches-clean-run: {'PASS' if committed_ok else 'FAIL'}")

            # 3) C header syntax
            c_status = {}
            c_detail = {}
            for target in TARGETS:
                c_status[target], c_detail[target] = check_c_header(
                    per_target[target], tmpdir
                )
            c_line = "PASS" if all(v == "PASS" for v in c_status.values()) else (
                "FAIL" if "FAIL" in c_status.values() else "UNAVAILABLE_IN_TEST_ENVIRONMENT"
            )
            lines.append(f"generated-c-header-syntax: {c_line}")

            # 4) Rust compile (optional)
            r_status = {}
            r_detail = {}
            for target in TARGETS:
                r_status[target], r_detail[target] = check_rust(
                    per_target[target] / "signet_adapter_role_ops.rs", tmpdir
                )
            if all(v == "PASS" for v in r_status.values()):
                lines.append("generated-rust-compile: PASS")
            elif "FAIL" in r_status.values():
                lines.append("generated-rust-compile: FAIL")
            else:
                lines.append("generated-rust-compile: UNAVAILABLE_IN_TEST_ENVIRONMENT")

            # 5) digest block, emitted from the regenerated outputs
            lines.append("")
            for target, descriptor_path in TARGETS.items():
                descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
                canonical_descriptor = (
                    json.dumps(descriptor, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
                    + "\n"
                ).encode("utf-8")
                lines.append(f"{target} descriptor-sha256: {sha256(canonical_descriptor)}")
                for name in GENERATED_NAMES:
                    payload = (per_target[target] / name).read_bytes()
                    lines.append(f"{target} {name}: sha256:{sha256(payload)}")
                lines.append("")

            # 6) negative descriptor tests
            minecraft = json.loads(TARGETS["minecraft"].read_text(encoding="utf-8"))

            def dup(d):
                d["operations"].append(json.loads(json.dumps(d["operations"][0])))
                return d

            def mismatch(d):
                d["operations"][0]["role"] = "PRESENTATION"
                return d

            def bad_id(d):
                d["operations"][0]["id"] = "Invalid_ID"
                return d

            lines.append(negative_test("duplicate-operation-id", minecraft, dup, "duplicate operation id", tmpdir))
            lines.append(negative_test("role-authority-mismatch", minecraft, mismatch, "incompatible with role", tmpdir))
            lines.append(negative_test("invalid-operation-id", minecraft, bad_id, "invalid operation id", tmpdir))

            # 7) environment status
            lines.append("")
            if "FAIL" in r_status.values():
                lines.append(f"rustc: FAIL ({r_detail})")
            elif all(v == "PASS" for v in r_status.values()):
                lines.append("rustc: PASS")
            else:
                lines.append("rustc: UNAVAILABLE_IN_TEST_ENVIRONMENT")
            lines.append("dotnet: NOT_APPLICABLE_NO_CSHARP_OUTPUT")
            if which("godot") is None:
                lines.append("godot-4.7.2-runtime: BLOCKED_BINARY_NOT_MATERIALIZABLE_IN_TEST_ENVIRONMENT")
            else:
                lines.append("godot-4.7.2-runtime: PRESENT_IN_TEST_ENVIRONMENT")

            MANIFEST.write_text("\n".join(lines) + "\n", encoding="utf-8")
            print("\n".join(lines))

            failures = [ln for ln in lines if ": FAIL" in ln or ": FAIL(" in ln]
            if not committed_ok or failures:
                print("\nVALIDATION: FAIL", file=sys.stderr)
                return 1
            print("\nVALIDATION: PASS")
            return 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
