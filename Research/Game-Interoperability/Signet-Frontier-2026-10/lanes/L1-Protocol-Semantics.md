# L1 — Protocol & Semantic Interoperability

## Mission

Determine the smallest explicit semantic contract that lets independently built game adapters participate in one shared simulation without hidden assumptions.

## Core questions

1. What belongs in the stable core versus optional profiles?
2. What does a capability negotiation message need to express?
3. How should actions, state, events, units, authority, time, and fallbacks be identified?
4. Which semantics can be inherited from HLA/DIS/OpenXR-style extension patterns?
5. How should source-game meaning and mapping provenance survive normalization?
6. When is a mapping syntactic, semantic, or pragmatic compatibility?
7. How should incompatible or partial participants fail safely?

## Primary evidence frontier

### Standards / technical systems

- IEEE HLA 1516 / OMT / RTI
- DIS / SISO standards and enumerations
- OpenXR extension and capability mechanisms
- glTF extension design where relevant
- OpenUSD schemas/composition where relevant
- Web/world identity work where relevant

### Native-language lanes

Prioritize terminology around:

- Chinese: `互操作`, `语义互操作`, `分布式仿真`, `本体`, `联邦`
- Japanese: `相互運用性`, `分散シミュレーション`, `オントロジー`, `連邦`
- German: `semantische Interoperabilität`, `Ontologie`, `Mediator`
- Spanish: `interoperabilidad semántica`, `interoperabilidad pragmática`, `ontología`
- Russian: `интероперабельность`, `онтология`, `предметная область`
- French: `interopérabilité sémantique`, `ontologie`, `fédération`

Expand recursively from citations and native terminology.

## Research tasks

### L1.1 — Core/profile boundary

Compare candidate decompositions:

```text
core:
  identity
  transform
  time
  lifecycle
  authority

profiles:
  fps
  platformer
  vehicle
  voxel
  inventory
  rpg-stats
  economy
```

Test whether each proposed core concept is truly cross-domain.

### L1.2 — Semantic identity

Compare:

- natural-language wire names;
- integer enums;
- URIs/URNs;
- schema IDs;
- content-addressed definitions.

Goal: stable machine identity with localizable human labels.

### L1.3 — Capability negotiation

Define what must be known before session admission:

- actions emitted;
- events understood;
- state consumed/published;
- timing assumptions;
- authority expectations;
- world representations;
- fallbacks;
- observer capability;
- hard incompatibilities.

### L1.4 — Authority and causality

Research how standards express:

- ownership;
- publish/subscribe authority;
- simulation time;
- causality;
- late/out-of-order events;
- transfer of responsibility.

### L1.5 — Semantic mapping provenance

Investigate whether mappings should carry:

- source concept/version;
- target concept/version;
- mapping type;
- lossiness;
- assumptions;
- author/tool;
- conformance evidence;
- validity interval.

## Deliverables

- candidate `core@1` concept set;
- 2–4 candidate profile schemas;
- capability-negotiation schema draft;
- semantic ID/versioning recommendation;
- mapping-provenance record;
- contradiction table against current Signet/1 assumptions;
- exact conformance properties that L4 can test.

## Do not duplicate

Hand off to:

- **L2** for code generation, decision models, SDK ergonomics, adapter runtime.
- **L3** for signing/authentication/package policy/legal questions.
- **L4** for benchmarks, independent implementations, compatibility registry design.

## Success condition

L1 is useful when two teams can read its candidate contract and independently implement adapters without needing private verbal clarification about what the shared state means.


---

## Continuation pass 1 — 2026-10-05

Durable outputs:

1. [Candidate Semantic Contract](L1/01-Semantic-Contract-Draft.md)
2. [Negotiation, Mapping Provenance, and Conformance Vectors](L1/02-Negotiation-Mapping-and-Conformance.md)
3. [Standards Delta, Contradictions, and Lane Handoffs](L1/03-Standards-Delta-and-Handoffs.md)

Material design changes from this pass:

- shrink `core@1` to coordination semantics rather than FPS/spatial gameplay concepts;
- move spatial transforms, lifecycle and FPS vocabulary into separately negotiated profiles;
- separate semantic profiles from deterministic rulesets/modes;
- replace placeholder `urn:signet:...` identifiers with a draft URI strategy that does not assume an unregistered URN namespace;
- distinguish implemented capability, activated capability and required capability;
- add definition hashes to detect semantic-ID collisions/drift;
- define deterministic compatibility outcomes and reason codes;
- make authority/time/causality explicit without importing full HLA runtime complexity;
- define durable semantic mapping provenance;
- hand L4 exact ID/NEG/MAP/AUTH/TIME/PROFILE/LEGACY conformance properties.

The next L1 recursion should test the proposed core against at least one non-spatial/non-FPS domain and specify canonical definition hashing/profile-composition conflict rules.


---

## Continuation pass 2 — Signet 2 response — 2026-10-05

New upstream primary source:

- Signet Protocol commit `490dfa9423841a45f2917d8d013e010ca0eb5548`
- *Signet 2: intents, archetypes and translation profiles*, draft proposal v0.1
- upstream proposal: https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/content/proposals/translation-profiles.mdx
- upstream PDF: https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/public/signet-2-architecture.pdf

Durable outputs:

1. [Signet 2 response analysis](L1/04-Signet-2-Response-Analysis.md)
2. [Public response draft: From Shared Meaning to Shared Contracts](../../../Publications/Game-Interoperability/FROM-SHARED-MEANING-TO-SHARED-CONTRACTS-DRAFT.md)

Material synthesis:

- **accept** Signet 2's game→meaning→game architecture, server authority, capabilities, calibration, pinned profiles, and AI-outside-authority boundary;
- strengthen “shared meaning” into stable semantic identity + definitions + versions/hashes;
- keep the universal core smaller than the proposed global FPS/action vocabulary;
- separate semantic profiles, authoritative instance state, rulesets, and local presentation roles;
- replace automatic “common subset” semantics with session requirements plus typed `SUPPORTED`, `SUPPORTED_WITH_FALLBACKS`, `OBSERVE_ONLY`, and `INCOMPATIBLE` outcomes;
- extend translation profiles from deterministic decision caches into provenance/evidence-bearing mapping manifests;
- treat motion calibration as adapter/transduction evidence, not an alternate physics authority;
- require semantic ambiguity/conformance vectors before claiming independent interoperability;
- preserve Signet/1 through an explicit legacy bridge rather than requiring a rewrite.

The next L1 falsification target remains a non-FPS/non-spatial domain. Signet 2 makes this test more urgent because its proposed version-0 intent/archetype vocabulary is deliberately action/FPS shaped.


---

## Continuation pass 3 — non-FPS falsification — 2026-10-05

New durable output:

- [Non-FPS Core Falsification](L1/05-Non-FPS-Core-Falsification.md)

Tested the proposed core conceptually against four non-FPS domain shapes:

1. turn-based board/tactics;
2. deckbuilder/card game with hidden information;
3. city-builder/management simulation;
4. user-defined tabletop/game-within-game.

Results:

- **PASS** on removing movement/body/weapon/health/world-geometry concepts from the universal core;
- **CORRECTION**: fixed `tick` must not look universal — the core now negotiates ordering models such as fixed-tick, turn-sequence, or event-sequence;
- **CORRECTION**: authority and visibility are distinct — hidden-information domains require an explicit disclosure-policy reference;
- **PASS** on intent/state/event separation outside FPS gameplay;
- **PASS** on scope-based authority without a player-body assumption;
- **PASS** on modular profiles as the mechanism for radically different game semantics.

The public Signet 2 response draft now includes these falsification results rather than only proposing the experiment.

Next implementation-grade falsification:

- one current Signet/FPS fixture;
- one minimal turn/card fixture;
- same core negotiation machinery;
- different profiles + ordering models;
- no dummy domain fields;
- exact cross-implementation conformance outcomes.
