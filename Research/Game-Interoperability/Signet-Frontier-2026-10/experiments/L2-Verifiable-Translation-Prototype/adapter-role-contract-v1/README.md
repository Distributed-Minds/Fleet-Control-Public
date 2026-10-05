# Adapter role contract v1

This revises the first L2 shim **because the Minecraft gateway target falsified it**.

The v0 generated function set was calibration-oriented and did not cover the real translator responsibilities documented by Signet.

v1 therefore describes operations by translator role:

- HOST
- IMPORTER
- WORLD
- INPUT
- PRESENTATION
- AUTHORITY

The descriptor records the local mechanism and authority class, but leaves `semantic_contract_ref` external. L2 owns adapter/runtime shape; the shared meaning belongs to the semantic/profile layer.

The Minecraft example covers all seven responsibilities extracted from Signet's current gateway documentation. The generic Godot example claims only the engine host/version boundary; it does **not** pretend a bare GDExtension knows a particular game's controls, terrain or presentation hooks.

This is a descriptor/evidence model, not yet the final generated ABI.


## Executed generator pass

v1 now has an executable generator:

- `generate.py`

It consumes the role descriptor and emits:

- `adapter.operations.json`;
- `signet_adapter_role_ops.h`;
- `signet_adapter_role_ops.rs`.

The generated operation signatures intentionally use opaque byte payloads. Payload semantics remain owned by the external semantic contract rather than being invented by L2.

`run_generation_validation.py` is the canonical reproduction harness. It runs
`generate.py` twice per target, fails if the two clean runs differ, rewrites
`generated/`, compiles the generated C header with `-fsyntax-only`, compiles the
generated Rust with `rustc` when present, runs the negative descriptor tests, and
regenerates `generation-validation-output.txt` from the outputs. No generated
artifact is hand-edited.

Reproduce with:

```text
python3 run_generation_validation.py
python3 check.py > validation-output.txt
```

Observed local validation:

```text
minecraft-deterministic-generation: PASS
godot-deterministic-generation: PASS
generated-tree-matches-clean-run: PASS
generated-c-header-syntax: PASS
generated-rust-compile: PASS

negative duplicate-operation-id: PASS(rejected)
negative role-authority-mismatch: PASS(rejected)
negative invalid-operation-id: PASS(rejected)
```

Generated artifact digests are recorded in `generation-validation-output.txt`.

The corrected v1 evidence is reproduced from this directory. The pre-correction
broken artifacts (malformed schema; generated JSON/header files that disagreed
with the digest manifest) are frozen under `historical/` and are not current
evidence; see `historical/README.md` and `CORRECTION-NOTES.md`.

## Live Godot runtime status

The Godot 4.7.2 release and Linux x86_64 asset were confirmed through the upstream release metadata. The execution environment contains no Godot binary, and binary release assets could not be materialized into this test container.

Therefore the evidence remains:

`ABI_HOST_HARNESS_PASS`

and explicitly **not**:

`GODOT_RUNTIME_PASS`.

That is an infrastructure limitation of this pass, not evidence for or against the GDExtension target.
