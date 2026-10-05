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

Public-facing drafts:

- `Publications/Game-Interoperability/OPEN-GAME-INTEROPERABILITY-FRONTIER-DRAFT.md`
- `Publications/Game-Interoperability/FROM-SHARED-MEANING-TO-SHARED-CONTRACTS-DRAFT.md` — direct L1 response to the Signet 2 intents/archetypes/translation-profiles proposal

Compact community handoff:

- `Publications/Game-Interoperability/DISCORD-HANDOFF.md`

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
