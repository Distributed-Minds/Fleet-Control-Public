# Adapter-shim deterministic generation proof

This is an **unaffiliated L2 research fixture**, not a proposed final Signet ABI.

Purpose:

> test whether one small machine-readable shim descriptor can deterministically generate repetitive target-language contract glue while leaving game/engine behavior in handwritten shims.

The descriptor deliberately references Signet semantic contracts opaquely. It does not define intents or archetypes.

## Input

- `adapter-shim.v0.example.json`

Logical operations in this first fixture:

- report integration-surface version;
- capture a local control;
- observe a calibration metric when legitimately observable;
- present an already-resolved local appearance choice.

## Generator

- `generate.py`

Current generated targets:

- C header;
- Rust trait;
- C# interface;
- TypeScript interface.

## Reproducibility result

The generator was run twice from a clean output directory on 2026-10-05. Byte comparison passed for every generated file.

```text
descriptor-sha256: 29663431523e65018681c1644020750af03edc4d82bf478c97966960da0981a5
ISignetAdapterShim.cs: sha256:74ed97fde58bb717b665bc80fb03d495aea8489a8e80321bcf750e70bde169bc
signet-adapter-shim.ts: sha256:366253479f7073a66bf65c8c7b2712c16f77f90c357d3e91804bdec48e4332a4
signet_adapter_shim.h: sha256:1750f7f62e7b36868183166a06f488449aa76e6d5ebeda2359c21e0ad06559bb
signet_adapter_shim.rs: sha256:4b0308e7efb754b3f83655c392ece5327ff1588cf321624cce37b6813417461f
deterministic-generation: PASS
c-header-syntax: PASS
typescript-syntax: PASS
```

The C header was checked with `cc -std=c11 -fsyntax-only`. The TypeScript interface was checked with `tsc --noEmit`.

Rust and C# compilers were not available in the local execution environment, so those two generated files are **generated and inspected but not compiler-validated in this pass**.

## Design boundary

This fixture does **not** attempt to generate game-specific behavior.

Handwritten code still owns:

- engine callbacks;
- mod/plugin API calls;
- RCON/console interactions;
- coordinate/unit transforms;
- file-format parsing;
- local lifecycle;
- version-specific quirks.

The generator owns only the repetitive, reproducible interface shape.

That is the specific claim being tested.
