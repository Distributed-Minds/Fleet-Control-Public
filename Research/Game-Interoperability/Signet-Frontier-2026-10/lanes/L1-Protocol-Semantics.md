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
