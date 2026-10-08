# Open Cross-Game Interoperability Frontier — Signet, Mashups, and the Standards Behind Them

**Research cutoff:** 2026-10-05  
**Status:** research package / unaffiliated technical analysis  
**Primary project examined:** Signet Protocol (`kian-cx/signetprotocol`)  
**Adjacent product examined:** Melty (`melty.gg`)

## Executive conclusion

The strongest architecture is:

> **deterministic core, adaptive edge**

Shared-world truth should remain deterministic and testable: coordinates, bodies, tick order, authoritative state transitions, event semantics, capability declarations, protocol versions, and conformance. Small decision models or larger language models can still be useful around that core for bounded classification, compatibility routing, adapter generation, presentation choices, error triage, and offline improvement.

Signet is already pointed in that direction. Its strongest current ideas are:

- a neutral authoritative server;
- game-specific translators instead of pairwise game-to-game integrations;
- separation of world/rules from viewer/appearance/input;
- intent-based control instead of client-authoritative position;
- manifest-declared capabilities;
- conformance tests;
- additive protocol evolution;
- local use of a player's own game files rather than redistribution.

The main scaling risk is **semantic interoperability**: once dozens or hundreds of games participate, the system needs machine-readable meaning for objects, actions, events, rules, units, capabilities, fallbacks, and partial support.

The main ecosystem opportunity is that **Signet and Melty are complementary rather than duplicates**:

- **Signet** is a neutral runtime/protocol + translator architecture.
- **Melty** is currently a distribution, installation, launch, multiplayer-discovery, and AI-assisted mashup-creation product.

A future open ecosystem could use a Signet-like protocol underneath and a Melty-like one-click experience above it.

## Package

1. [Current projects and baseline](01-Current-Projects-and-Baseline.md)
2. [Interoperability architecture](02-Interoperability-Architecture.md)
3. [Polyglot frontier ledger](03-Polyglot-Frontier-Ledger.md)
4. [Decision models and adaptive translation](04-Decision-Models-and-Adaptive-Translation.md)
5. [Security, privacy, legal, and supply chain](05-Security-Privacy-Legal-and-Supply-Chain.md)
6. [Concrete recommendations and experiment roadmap](06-Recommendations-and-Roadmap.md)
7. [Source ledger](Source-Ledger.md)

Public-facing draft:

- `Publications/Game-Interoperability/OPEN-GAME-INTEROPERABILITY-FRONTIER-DRAFT.md`

Compact community handoff:

- `Publications/Game-Interoperability/DISCORD-HANDOFF.md`

## Corrected L1/L2 source archives (integrated 2026-10-08)

This research tree now **preserves the corrected primary-source experiments**, not just the response-paper summaries. The original corrected branches remain available as Git provenance. Both archival imports landed in this **non-default research branch**, not the public `main`, installable Phase0 preview, release package, or a deployed FREE ENERGY product:

- **L1 — corrected semantic protocol evidence:** [semantic-core JSON Schema](lanes/L1/schema/signet-semantic-core-0.schema.json) and [cross-language JavaScript reference](lanes/L1/reference/javascript/reference.mjs), with the corrected canonicalization/composition vectors under `lanes/L1/`. [PR #77](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/77) preserved all **29** selected corrected L1 source blobs and merged into this research branch at `9a1af620210338c2ae91ee07fef92b2f86ba4de7`.
- **L2 — corrected adapter/role-contract research evidence:** [correction notes and historical broken-source segregation](experiments/L2-Verifiable-Translation-Prototype/adapter-role-contract-v1/CORRECTION-NOTES.md), [resolver pilot](experiments/L2-Resolver-Pilot/README.md), and [third-party source/rights ledger](experiments/L2-THIRD-PARTY-PROVENANCE.md). [PR #78](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/78) preserved all **55** corrected L2 source blobs plus an explicit supplemental provenance ledger and merged into this research branch at `480fdf31cd44e7211c880c6b829a5fdc22df339c`.

**Evidence/rights boundary:** These byte-preservation results do **not** validate the historical Python/JavaScript/TypeScript research scripts as maintained executables, prove interoperability with a real game or Godot runtime, establish full license/asset clearance, or qualify a release. The [L2 provenance ledger](experiments/L2-THIRD-PARTY-PROVENANCE.md) retains **RIGHTS_PENDING** and records an offline C host-harness strict-build limitation; the original archive remains unchanged. Legacy research scripts and deliberately broken historical fixtures are **inert archival evidence**, not part of the supported FREE ENERGY toolchain. Any promoted maintained validation must satisfy [Rust-only/no-npm issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70). [Issue #73](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/73) tracks remaining source-ledger, rights and conformance disposition; do not infer its completion from these merges.

## Parallel continuation lanes

The next research pass is split into four non-overlapping lanes:

1. [L1 — Protocol & Semantic Interoperability](lanes/L1-Protocol-Semantics.md)
2. [L2 — Adapter Engineering & Adaptive Translation](lanes/L2-Adapters-Adaptive-Translation.md)
3. [L3 — Trust, Distribution, Privacy, Legal & Governance](lanes/L3-Trust-Distribution-Governance.md)
4. [L4 — Validation, Ecosystem & Adoption](lanes/L4-Validation-Ecosystem-Adoption.md)

See [LANES.md](LANES.md) for ownership boundaries, handoff format, shared evidence rules, and the synthesis target.

Each lane continues the native-language / recursive-discovery method inside its own domain and hands material cross-lane findings to the lane that owns the follow-up.

## Evidence labels

- **OBSERVED** — directly present in current project/source material.
- **DERIVED** — consequence inferred from observed facts.
- **PROPOSED** — design recommendation.
- **CLAIMED** — vendor/project self-report not independently reproduced here.
- **UNKNOWN** — unresolved or inaccessible in this pass.

## Compact architecture

```text
                         discovery / packaging / launcher
                                    |
                                    v
 game A -- adapter --\        capability negotiation        /-- adapter -- game B
 game C -- adapter ---+----> neutral semantic contract <----+--- adapter -- game D
 game E -- adapter --/               |                      \-- adapter -- game F
                                    v
                     deterministic authoritative core
                      world + rules + bodies + events
                                    |
                         optional adaptive helpers
              ranking / mapping / generation / diagnostics
```

The key property is that an adaptive helper can suggest or select a mapping, but the **wire contract, selected capability, resulting event, and authoritative world state remain explicit and inspectable**.

## What the multilingual pass changed

The native-language research produced actual design deltas:

- Chinese work provides a direct modern precedent for a **plugin Unreal Engine ↔ HLA adapter**.
- Japanese work from the early 2000s already explored **plug-in adapters, graphical exchange definitions, support tooling, and automatic HLA glue-code generation**.
- Spanish research explicitly separates **syntactic, semantic, and pragmatic interoperability** and uses **networks of ontologies** above HLA.
- Russian work emphasizes **declarative domain models, knowledge graphs, and lifecycle-specific ontologies**.
- German work provides an anti-N² pattern: use a **mediating ontology** rather than direct standard-to-standard mappings.
- French work explores federated interoperability with **contextual/ephemeral ontologies**.
- Brazilian work adds a useful warning: declaring one ontology does not make it perspective-free.

## Suggested immediate shared milestone

> **A machine-readable, versioned adapter contract + conformance suite that lets two independently written translators prove they agree on a minimal shared world and event model.**

That is narrower and more useful than “make every game understand every game.”
