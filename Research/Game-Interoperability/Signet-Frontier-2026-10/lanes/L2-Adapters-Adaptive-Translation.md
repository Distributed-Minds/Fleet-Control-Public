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
