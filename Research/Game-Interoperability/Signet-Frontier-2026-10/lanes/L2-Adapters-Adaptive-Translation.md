# L2 — Adapter Engineering & Adaptive Translation

## Mission

Research how real games and engines can be connected quickly and maintainably while keeping adaptive models outside authoritative shared-world truth.

## Core questions

1. What integration patterns scale across open engines, official mods, gateways, and reimplementations?
2. What adapter scaffolding can be generated mechanically?
3. Where do Laya/CLM/Jev-like decision models outperform ordinary rules?
4. Where should LLMs generate code offline rather than reason at runtime?
5. How can accepted model suggestions be frozen into explicit mappings?
6. How do we detect drift after game updates?
7. What debugging/replay tooling makes translator failures explainable?

## Primary evidence frontier

### Existing implementations

- Signet translator traits and current Doom/OpenArena/Minecraft integrations;
- Unreal/Unity/Godot/open-engine plugin architectures;
- HLA adapter/federate integration tooling;
- historical simulator-adapter code generation;
- mod-loader/plugin ecosystems with stable APIs.

### Decision systems

- Laya;
- CLM;
- Jev/System-One APIs;
- lightweight classifiers/rankers;
- rules/lookup baselines;
- retrieval and schema-guided generation.

### Native-language lanes

Search for engine-adapter and code-generation work in:

- Chinese;
- Japanese;
- Korean;
- German;
- Russian.

Prefer local engine/simulation literature over translated English summaries.

## Research tasks

### L2.1 — Integration pattern matrix

For each mode:

```text
OPEN_ENGINE
OFFICIAL_MOD
OFFICIAL_API
CONTROLLED_SERVER_GATEWAY
LOCAL_REIMPLEMENTATION
```

record:

- available hooks;
- authority;
- latency;
- update fragility;
- asset access;
- multiplayer limitations;
- maintainability;
- legal/policy dependencies.

### L2.2 — Schema → adapter scaffolding

Research generation of:

- Rust traits;
- C ABI;
- C# interfaces;
- TypeScript types;
- manifest skeletons;
- replay fixtures;
- conformance tests;
- engine plugin boilerplate.

Compare template generation against LLM-assisted completion.

### L2.3 — “Very smart if” benchmark design

Candidate tasks:

- event mapping;
- fallback selection;
- failure triage;
- capability classification;
- test prioritization.

Baselines:

- deterministic rules;
- lookup table;
- Laya;
- CLM;
- general LLM.

Measure:

- exact-choice accuracy;
- calibration;
- abstention;
- multilingual robustness;
- latency;
- memory/compute;
- out-of-domain failure.

### L2.4 — Generate → test → freeze

Research a pipeline where AI proposes mappings/code but release artifacts contain explicit versioned output.

Questions:

- what provenance is needed?
- what changes require revalidation?
- what can be cached/frozen?
- when must a human inspect?

### L2.5 — Drift detection

Study:

- game-version fingerprints;
- API/schema diffing;
- binary/file-format drift;
- event-signature changes;
- replay regressions;
- automatic invalidation of compatibility claims.

### L2.6 — Debugging surface

Define developer evidence:

- raw local event;
- mapped neutral event;
- ruleset result;
- replay/tick;
- capability decision;
- model hint, if used;
- fallback reason.

## Deliverables

- adapter integration matrix;
- generated scaffold specification;
- model-evaluation dataset design;
- decision-model benchmark plan;
- accepted-mapping/provenance format;
- game-update drift strategy;
- debugging/replay tooling proposal.

## Do not duplicate

Hand off to:

- **L1** when the adapter needs a new shared semantic concept/profile.
- **L3** when execution permissions, package trust, user privacy, or EULA boundaries dominate.
- **L4** for independent experimental execution and public benchmark methodology.

## Success condition

L2 is useful when a new adapter can be built mostly by filling a well-defined contract, and any AI assistance produces inspectable artifacts rather than hidden runtime semantics.

## Current pass — 2026-10-05

**Status: first design/research pass complete; empirical implementation pass remains open.**

Durable outputs:

- [Adapter Engineering Findings](L2-Adapter-Engineering-Findings.md)
- [L2 Source Ledger](L2-Source-Ledger.md)

The current design delta is:

> **generated contract glue + thin handwritten game/engine hook shim + explicit frozen semantic mappings + replay/conformance evidence**

The next high-value step is an empirical adapter-generator prototype against at least one native engine/plugin target and one gateway/reimplementation target, followed by the closed-set mapping benchmark defined in the findings.

Independent public benchmark execution belongs to L4.

### Signet 2 response

Signet subsequently published a concrete Signet 2 draft that converges with the lane's deterministic-core/adaptive-edge direction.

Public response manuscript:

- `Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md`

The response keeps the Signet 2 core direction and concentrates on the remaining engineering boundary: typed authority inside profiles, evidence-bearing locks, calibration observability, resolver abstention/OOD handling, drift invalidation, generated adapter glue, and a stronger staged benchmark.

### Executable profile/drift experiment

The empirical pass has started with a deliberately small contract test:

- `Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/`

Observed local result:

```text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
```

This proves the proposed state distinction is executable: `pinned` keeps the choice deterministic while changed evidence moves the mapping out of `VALID`.

Next L2 implementation seam: machine-readable adapter ABI descriptor → deterministic generated glue, followed by replay fixtures.

### Deterministic generated-glue experiment

The second executable L2 fixture is:

- `Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/adapter-shim-generation/`

Observed result:

```text
deterministic-generation: PASS
c-header-syntax: PASS
typescript-syntax: PASS
```

The same descriptor generated C, Rust, C# and TypeScript contract surfaces with stable hashes across two clean runs.

This advances L2.2 from design prose to a proof-of-mechanism. It does **not** prove the proposed shim API is the right final Signet API; it proves the repetitive interface layer can be made machine-readable and reproducible while target-specific behavior remains handwritten.

Next empirical step: implement one generated surface against a real engine/plugin or controlled gateway and drive it through replay/conformance evidence.

### Real-target empirical correction

The empirical pass now includes two different integration modes:

1. **OPEN_ENGINE / Godot 4.7.2 GDExtension**
   - real ABI-source subset;
   - compiled shared library;
   - host-harness initialization/version test PASS.

2. **CONTROLLED_SERVER_GATEWAY / documented Signet Minecraft gateway**
   - Source-RCON framing PASS;
   - v0 logical shim coverage FAIL_EXPECTED.

The negative gateway result invalidated the first fixed logical shim as a candidate generic adapter API.

Replacement research fixture:

- `Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/adapter-role-contract-v1/`

v1 describes target operations by translator role and validates 7/7 documented Minecraft responsibilities while making zero generic Godot gameplay claims.

**Current status:** empirical implementation pass active. Code-generation mechanism retained; logical contract revised from evidence.
