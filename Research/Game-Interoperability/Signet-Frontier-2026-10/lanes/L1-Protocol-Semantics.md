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

---

## Historical L1 continuation index — corrected research, not maintained runtime

This index reconciles the corrected L1 continuation narrative at immutable
`correction/l1-cross-language-determinism@f109e15ed1c73b0c76876524ec3c126e34bd63ae`
with this branch's narrower shared L1 charter. The charter and the archival,
rights and Rust-only notices above remain authoritative for what this FREE ENERGY
repository actually supports. Original experiment writeups, candidate schemas,
test vectors, and historical reference code are preserved at the links below;
the original source narrative is **not** blindly substituted for the current
research integration text.

| Historical pass (2026-10-05) | Preserved artifact / decision | Boundary for a current implementer |
| --- | --- | --- |
| 1 — candidate core/profiles | [Core contract](L1/01-Semantic-Contract-Draft.md), [negotiation/mapping](L1/02-Negotiation-Mapping-and-Conformance.md), [standards delta](L1/03-Standards-Delta-and-Handoffs.md) | Candidate semantics, not a deployed shared-game protocol |
| 2 — Signet 2 response | [Primary-source response](L1/04-Signet-2-Response-Analysis.md) | Comparative proposal; not upstream approval or implemented Signet 2 adapter |
| 3 — non-FPS falsification | [Turn/card/city/tabletop analysis](L1/05-Non-FPS-Core-Falsification.md) | Conceptual model test, not a playable cross-domain runtime |
| 4 — structural schema | [Core candidate](L1/06-Machine-Readable-Core-Candidate.md), [JSON Schema](L1/schema/signet-semantic-core-0.schema.json) | Schema-valid is not semantic conformance |
| 5 — canonical hashing/composition | [Hashing rules](L1/07-Canonical-Hashing-and-Profile-Composition.md), [semantic definition schema](L1/schema/signet-semantic-definition-0.schema.json) | RFC 8785/JCS does **not** sort arrays; schema-declared sets require separate source canonicality |
| 6 — independent references | [Cross-language evidence](L1/08-Cross-Language-Reference-Evidence.md), [historical reference inventory](L1/reference/README.md) | Python/JavaScript reference code remains archival; no maintained Rust port or fresh execution claim |
| 7 — expanded composition | [Composition and negative vectors](L1/schema/test-vectors/composition-negative-vectors.json), [reference inventory](L1/reference/README.md) | Historical reported outcomes are not current CI or full RFC 8785 numeric coverage |
| 8 — deterministic preselection | [Preselection model](L1/09-Deterministic-Preselection-Negotiation.md), [negotiation vector](L1/schema/test-vectors/negotiation-vector-001.json) | Candidate ordering/visibility negotiation, not an installed session authority |
| 9 — optional-profile negotiation | [Optional activation rules](L1/10-Optional-Profile-Negotiation.md), [optional vector](L1/schema/test-vectors/optional-negotiation-vector-001.json) | Preserve ordered optional-skipping reason codes; no runtime/production compatibility certification |

### Corrected follow-on evidence and next verification boundary

The corrected L1 tip adds or repairs [non-BMP/UTF-16 semantic ordering
vectors](L1/schema/test-vectors/composition-unicode-vectors.json),
[set-array source-canonicality vectors](L1/schema/test-vectors/hash-set-array-vectors.json)
and [selection-order/rejection vectors](L1/schema/test-vectors/composition-set-order-vectors.json).
Their original bytes already exist in this research integration tree. This
navigation update does **not** reinterpret them, reproduce the historical
Python/Node test matrix, prove cross-language identity, or grant upstream or
asset rights.

Before promoting any of this research to a maintained executable contract:
implement a separately reviewed, compiled Rust conformance runner per
[#70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70);
run the corrected positive and negative vectors against **exact** pinned
toolchain/dependency heads; establish independently checked Unicode/set-array
canonicalization and failure precedence; and document tested scope,
provenance, rights, and repeatability. The remaining shared-publication and
ledger collision dispositions stay open under
[#73](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/73).
