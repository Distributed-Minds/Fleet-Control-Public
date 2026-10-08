# L2 correction record — role-contract reproducibility

**Branch:** `correction/l2-role-contract-reproducibility`
**Historical L2 base head (unchanged, not rewritten):** `a5a677f7bf3e80d2face8f2185b27fcb8e394ab3`
**Correction fix commit:** `dcf1f55`

This branch fixes the HIGH L2 finding in
`Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md` §2. The original research lane
and its head are preserved as history; this is an additive correction branch.

## Defects fixed

1. `adapter-role-contract-v1/role-contract.schema.json` had one extra `}` and
   did not parse, so `check.py` died at `json.loads` and the committed 7/7
   Minecraft claim could not be reproduced.
2. `generated/{minecraft,godot}/adapter.operations.json` and
   `signet_adapter_role_ops.h` did not match the digest manifest in
   `generation-validation-output.txt` nor fresh `generate.py` output. (The `.rs`
   files already matched.)

## Exact commands (from the repo root, in this worktree)

```bash
BASE=$(git rev-parse HEAD)   # a5a677f7bf3e80d2face8f2185b27fcb8e394ab3
# worktree: /tmp/opencode/signet-validation/l2

# 1) fix malformed schema (removed one extra closing brace; JSON now parses)
python3 -c "import json;json.load(open('Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/adapter-role-contract-v1/role-contract.schema.json'));print('schema JSON OK')"

# 2+3+5) regenerate artifacts + manifest and run deterministic/negative checks
cd Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/adapter-role-contract-v1
python3 run_generation_validation.py

# 4) role coverage / schema validation
python3 check.py | tee validation-output.txt

# 6) resolver pilot determinism
cd ../../L2-Resolver-Pilot
python3 benchmark.py
```

## Results

```text
minecraft-deterministic-generation: PASS
godot-deterministic-generation: PASS
generated-tree-matches-clean-run: PASS
generated-c-header-syntax: PASS          # gcc -std=c11 -Wall -Wextra -fsyntax-only
generated-rust-compile: PASS             # rustc --edition 2021 --crate-type lib

negative duplicate-operation-id: PASS(rejected)
negative role-authority-mismatch: PASS(rejected)
negative invalid-operation-id: PASS(rejected)

minecraft-responsibility-coverage: 7/7    # check.py, from source
minecraft-role-contract: PASS
godot-generic-gameplay-claims: 0
godot-engine-vs-game-boundary: PASS

resolver gold_oracle: 1.000/1.000/1.000   # oracle upper bound, NOT a resolver
resolver lexical:     1.000/0.000/0.167
resolver first:       1.000/0.000/0.000
```

Regenerated digests are in `generation-validation-output.txt`; they match the
values already recorded at the historical head, confirming the committed JSON and
header files (not the manifest) were the corrupt side.

## Historical evidence

The pre-correction broken files are frozen under `adapter-role-contract-v1/historical/`
with their git blob SHAs; see `historical/README.md`. The original branch is not
rewritten.

## Remaining limitations

- `rustc`, `gcc`, `dotnet` were available in this correction environment; the
  historical manifest's `rustc/dotnet: UNAVAILABLE` lines reflected the original
  environment. The regenerated manifest records what actually ran here.
- The Godot 4.7.2 runtime binary is still not materializable in this container;
  ABI host-harness evidence stands, runtime evidence is still blocked.
- The resolver pilot remains a 6-case seed harness. `gold_oracle` is an oracle
  upper bound and must not be cited as resolver performance; only `lexical` and
  `first` are resolver-shaped controls.
