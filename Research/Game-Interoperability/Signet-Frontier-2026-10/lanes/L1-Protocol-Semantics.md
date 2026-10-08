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

## Corrected source preservation — 2026-10-08 research checkpoint

This shared L1 charter predates the later continuation passes. The corrected L1 source and regression vectors have been **preserved as historical research artifacts** on this non-default Signet research integration branch through [PR #77](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/77). They must not be mistaken for a published Signet standard, a FREE ENERGY runtime adapter, or the installed Fleet-Control Phase0 toolchain. The original continuation narrative on `correction/l1-cross-language-determinism` was not blindly copied over the shared charter.

Current in-tree entry points for inspecting the corrected evidence:

- [Machine-readable semantic-core candidate](L1/06-Machine-Readable-Core-Candidate.md) and its [Draft 2020-12 schema](L1/schema/signet-semantic-core-0.schema.json): structural acceptance is **not** semantic conformance.
- [Canonical hashing/composition research](L1/07-Canonical-Hashing-and-Profile-Composition.md) and [cross-language reference evidence](L1/08-Cross-Language-Reference-Evidence.md): normative set-array canonicalization is separate from RFC 8785 JSON canonicalization, which does not reorder arrays.
- [Unicode ordering correction vectors](L1/schema/test-vectors/composition-unicode-vectors.json): compare semantic identifiers by UTF-16 code units where the candidate contract requires it, rather than assuming Unicode code-point ordering agrees across implementations.
- [Set-array hashing vectors](L1/schema/test-vectors/hash-set-array-vectors.json) and [selection-order/rejection vectors](L1/schema/test-vectors/composition-set-order-vectors.json): preserve positive and negative fixtures, including ordering-independent failure precedence.
- [Historical reference inventory](L1/reference/README.md) and [JavaScript reference](L1/reference/javascript/reference.mjs): these and the historical Python/shell counterparts are **archival evidence**, not supported maintained FREE ENERGY executable dependencies.

The original historical execution claims in the correction branch have **not** been rerun or promoted into current CI evidence by the archival merge. Maintained tooling and any future conformance runner remain subject to [the Rust-only/no-npm policy in issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70), fresh executable tests, independent rights/provenance review, and actual integration authority. See [issue #73](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/73) for still-open shared-ledger, publication-conflict, and verification obligations; preserving bytes or restoring navigation does not close them.
