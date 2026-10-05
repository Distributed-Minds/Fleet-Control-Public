# L4 — Validation, Ecosystem & Adoption

## Mission

Turn interoperability claims into reproducible evidence and determine what would make an open ecosystem usable by independent developers rather than only its original authors.

## Core questions

1. How do we prove two independently written adapters actually interoperate?
2. What benchmark/replay corpus exposes semantic disagreement?
3. What evidence should a compatibility registry store?
4. Which claims can third parties reproduce?
5. What is the smallest credible independent ecosystem milestone?
6. Which standards/projects are actually adopted rather than merely specified?
7. What failed interoperability projects or abandoned approaches should constrain the design?
8. What developer/community workflow reduces duplicate effort?

## Primary evidence frontier

### Empirical interoperability

- HLA/DIS conformance and federation testing;
- protocol compliance programs;
- Khronos conformance/extension practices;
- game/mod compatibility databases;
- package ecosystem reproducibility;
- differential testing;
- deterministic replay systems.

### Ecosystem research

Map:

- active independent Signet translators/contributors;
- adjacent open cross-game projects;
- modding communities;
- simulation/open-metaverse projects;
- engine-plugin communities;
- standards working groups.

Distinguish active maintained projects from stale demos.

### Multilingual / historical frontier

Own the search for **failed, superseded, or abandoned interoperability efforts** across language ecosystems.

Useful queries include local equivalents of:

- failed interoperability;
- distributed simulation integration problems;
- semantic mismatch;
- federation failure;
- abandoned middleware;
- game-engine interoperability.

## Research tasks

### L4.1 — Independent implementation criterion

Define a meaningful independent implementation:

- different author/team;
- no copied adapter logic;
- same published contract;
- same conformance vectors.

Research how mature standards organizations define implementation independence.

### L4.2 — Golden replay corpus

Construct research requirements for:

- initial state;
- ordered intents;
- expected state;
- expected events;
- deterministic hashes where applicable;
- tolerated presentation differences.

### L4.3 — Semantic ambiguity benchmark

Include adversarial pairs:

- death vs despawn;
- requested vs applied damage;
- button press vs successful interaction;
- shot requested vs shot fired;
- teleport request vs authoritative transform;
- possession vs presentation-only inventory.

### L4.4 — Compatibility evidence registry

Research schema and governance for:

- game version;
- adapter version;
- profile version;
- platform;
- conformance suite/hash;
- exact result;
- fallbacks;
- known failures;
- reproduction instructions;
- freshness/expiry.

### L4.5 — Reproducibility

Ask whether a stranger can reproduce:

```text
"translator X supports game Y version Z under profile P"
```

without private state or maintainer intervention.

### L4.6 — Ecosystem milestone ladder

Candidate milestones:

1. two independent adapters, one minimal profile;
2. three meaningfully different integration modes;
3. public replay/conformance corpus;
4. reproducible compatibility registry;
5. third-party maintained adapter;
6. independent protocol proposal;
7. alternate implementation of the core server or SDK.

### L4.7 — Negative-space research

Find:

- abandoned metaverse protocols;
- failed federation middleware;
- adapter ecosystems that collapsed under version drift;
- compatibility databases that became stale;
- plugin stores compromised by supply-chain attacks;
- semantic standards that overfit one domain.

These are high-value evidence, not embarrassment.

## Deliverables

- independent-implementation definition;
- conformance/benchmark methodology;
- golden replay corpus specification;
- compatibility registry schema;
- reproduction protocol;
- ecosystem map with activity/freshness evidence;
- negative-results dossier;
- milestone/adoption scorecard.

## Do not duplicate

Hand off to:

- **L1** when tests reveal semantic ambiguity requiring contract changes.
- **L2** when failures are adapter/tooling problems.
- **L3** when validation exposes package/security/governance weaknesses.

## Success condition

L4 is useful when an outsider can reproduce or falsify the project's interoperability claims without trusting the original maintainer.


---

# First evidence pass — conformance, independence, and adoption

**Research date:** 2026-10-05  
**Basis:** Signet Protocol main at 36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c plus the standards/interoperability sources recorded in the package Source Ledger.

This pass sharpens one core distinction:

~~~text
SELF_CONFORMANCE
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= ECOSYSTEM_ADOPTION
~~~

A credible interoperability ecosystem needs evidence at all four levels. One green test suite must not silently stand in for the others.

## Current Signet baseline

### OBSERVED — maturity and implementation independence

The first public Signet release is 0.1.0-beta.1, dated 2026-10-04.

The current governance document says the project has **one maintainer**. Doom, OpenArena, Minecraft, and native Signet translators are documented inside the same repository and release lineage.

Therefore:

~~~text
MULTIPLE_BUNDLED_TRANSLATORS
!= MULTIPLE_INDEPENDENT_IMPLEMENTATIONS
~~~

They are useful reference implementations and integration fixtures, but this pass found no basis for treating them as the independent implementations required by L4's first ecosystem milestone.

This is not a criticism of a one-day-old public beta. It is the correct baseline against which later adoption evidence should be measured.

### OBSERVED — present conformance scope is narrower than full Signet/1 interoperability

The current conformance documentation exposes:

- terrain T01–T07;
- rules/determinism R01–R02;
- manifest M01–M04.

Presentation P01–P03 is explicitly upcoming.

The protocol documentation says numbered Intencion commands use seq > 0 so the server applies each command exactly once for one tick. The same release changelog records a known gap: the Minecraft gateway still sends unnumbered intents.

That creates a useful real fixture:

~~~text
CURRENT_SUITE_PASS
can coexist with
KNOWN_WIRE_OR_INPUT_SEMANTIC_GAP
~~~

The existing suite is good evidence for the checks it actually executes. It is not yet a complete proof of wire, event, presentation, capability, reconnect, cross-SDK, or pairwise adapter interoperability.

### DERIVED — compatibility claims need explicit scope

Machine-readable evidence should be a tuple, not one Boolean:

~~~text
COMPATIBILITY_CLAIM =
  contract/profile version
+ implementation artifact
+ test-family scope
+ suite identity
+ environment
+ exact result
+ evidence
+ freshness
~~~

A human-facing "compatible" label may remain useful, but the registry behind it should expose the bounded claim.

## Independent implementation criterion v1

Mature standards bodies use implementation independence as evidence that a specification is not merely a formalization of one author's code.

For Signet, independence should be evaluated **per claim**.

An implementation counts as independent evidence only when the relevant dimensions are explicit:

1. **Author/control independence** — different person/team/organization from the implementation it corroborates.
2. **Codebase independence** — not merely a fork, wrapper, generated binding, or lightly modified copy of the same implementation logic for the behavior under test.
3. **Contract-first basis** — implementable from the published protocol/profile, public test vectors, and documentation without private maintainer knowledge.
4. **Build/run independence** — a third party can obtain, build/configure, and run it without private state.
5. **Pairwise evidence** — when the claim concerns interoperability, the implementations actually communicate or consume each other's externally visible output.
6. **Feature-specific evidence** — independence for one feature is not inherited by every feature.

Useful classes:

~~~text
REFERENCE_IMPLEMENTATION
SAME_CODEBASE_VARIANT
INDEPENDENT_ADAPTER
INDEPENDENT_PROTOCOL_IMPLEMENTATION
INDEPENDENT_CORE_OR_SERVER
~~~

A translator written by another developer against the official Rust SDK may be a valid independent adapter while not being an independent implementation of the Rust SDK or wire codec.

Current standards precedent supports this distinction:

- the W3C Process asks whether independent interoperable implementations exist, whether people other than specification authors created implementations, whether implementations are publicly deployed, and whether implementation problems have been reported;
- RFC 6410 requires at least two independent interoperating implementations with widespread deployment and successful operational experience for Internet Standard status, and separately checks interoperability-breaking errata and materially unused complexity.

**Rule:** multiple examples from the specification author's own codebase do not establish independent implementation maturity.

## Validation evidence ladder

Use a ladder rather than one global status.

### V0 — declared/static validity

Manifest parsing, required fields, version syntax, namespaced IDs.

### V1 — single-implementation behavioral conformance

Deterministic movement, terrain invariants, parser behavior, required error handling. Most present Signet tests live here.

### V2 — pairwise interoperability

Two implementations exercise a shared profile against each other. Record exact versions, roles, test matrix, passes, failures, skips, and raw evidence.

### V3 — differential and ambiguity testing

Multiple implementations receive the same adversarial semantic inputs and must agree where the contract requires agreement. A disagreement must be classified as implementation defect, underspecified contract, profile mismatch, or intentionally permitted variation.

### V4 — clean third-party reproduction

A third party reproduces the compatibility claim from published sources and instructions without maintainer intervention.

### V5 — operational freshness

Evidence remains current after game, adapter, protocol/profile, SDK/runtime, and supported-platform changes. A stale green result is historical evidence, not current compatibility.

## Golden replay corpus requirements

A replay must be an interoperability artifact, not just captured gameplay.

It should bind:

- replay ID and profile/version;
- initial world/rules/body state;
- ordered input intents;
- expected authoritative checkpoints;
- expected events and event order;
- deterministic hashes where applicable;
- explicitly tolerated presentation-only differences;
- forbidden authoritative differences;
- corpus generator and revision.

The first public corpus should cover:

1. wire sequencing — numbered, duplicate, delayed, dropped, reconnect, and out-of-order intents;
2. body lifecycle — movement, collision, death, respawn;
3. semantic ambiguity — requested versus applied actions/events;
4. capability mismatch — unsupported, fallback, observer-only, partial presentation;
5. versioning — unknown optional fields and old/new peers;
6. cross-SDK parity — Rust, C ABI, C#, and TypeScript where semantics overlap;
7. game-update drift — automatic invalidation after version/fingerprint changes;
8. malformed and impossible inputs.

Required ambiguity fixtures:

~~~text
death              != despawn
damage requested   != damage applied
button pressed     != interaction succeeded
shot requested     != shot fired
teleport requested != authoritative transform changed
inventory display  != authoritative possession
input received     != input applied exactly once
~~~

The current Minecraft unnumbered-intent gap should become one of the first regression fixtures.

## Pairwise interoperability matrix

Publish the **denominator**, not only successful demonstrations.

For every matrix record:

- eligible pairings;
- attempted pairings;
- not-attempted pairings;
- pass/fail/skip counts;
- reason for every skip;
- exact versions;
- profiles/features exercised.

OpenID's 2025 wallet/credential interoperability events are a strong pattern: OpenID4VP tested 153 of 224 possible pairings with more than 90% passing, while OpenID4VCI reported 47 tested pairs with 87% passing. Reporting both coverage and success prevents a high pass percentage from hiding a small tested subset.

## Compatibility evidence registry — schema requirements

A registry row should be a reproducible claim record, not a popularity badge.

Each claim should bind at least:

~~~yaml
claim_id: ...
claim_type: pairwise_interop
subject:
  game_id: ...
  game_version: ...
  game_fingerprint: ...
  adapter_source: ...
  adapter_commit: ...
  artifact_digest: ...
  implementation_class: INDEPENDENT_ADAPTER
counterparty:
  source: ...
  commit: ...
  artifact_digest: ...
protocol:
  wire: Signet/1
  profiles: [...]
suite:
  source: ...
  commit: ...
  artifact_digest: ...
  test_families: [...]
environment:
  os: ...
  architecture: ...
  runtime: ...
result:
  status: PASS
  passed: ...
  failed: ...
  skipped: ...
  fallbacks: [...]
  known_failures: [...]
evidence:
  raw_log_digest: ...
  replay_digests: [...]
  reproduction_instructions: ...
reproducer:
  identity: ...
  independence_relation: ...
freshness:
  observed_at: ...
  invalidation_triggers: [...]
~~~

Registry invariants:

~~~text
SIGNED_RESULT != CORRECT_RESULT
SELF_TESTED != INDEPENDENTLY_REPRODUCED
CONFORMANCE != PAIRWISE_INTEROPERABILITY
PAIRWISE_INTEROPERABILITY != WIDESPREAD_ADOPTION
POPULARITY != COMPATIBILITY
OLD_PASS != CURRENT_PASS
SKIPPED != PASSED
~~~

Do not collapse these into one generic "verified" state.

## Reproduction protocol v1

A stranger should be able to challenge a registry claim by:

1. resolving exact adapter/core/profile/suite revisions;
2. verifying source and artifact digests;
3. obtaining required game assets locally and checking only allowed version/fingerprint metadata;
4. building or installing in a clean documented environment;
5. running the declared single-implementation test subset;
6. running the pairwise/replay tests relevant to the claim;
7. preserving raw logs, test IDs, skips, fallbacks, environment data, and replay hashes;
8. publishing a reproduction record that references the original claim rather than overwriting it;
9. preserving conflicting results until the cause is classified;
10. expiring or reopening the claim when an invalidation trigger moves.

Useful reproduction states:

~~~text
CONFIRMS
PARTIALLY_CONFIRMS
FAILS_TO_REPRODUCE
INVALIDATED_BY_VERSION_DRIFT
TEST_SUITE_DISPUTED
ENVIRONMENT_SPECIFIC
UNKNOWN
~~~

## What mature ecosystems teach L4

### W3C — implementation experience is broader than a test pass

The current W3C Process explicitly includes independent implementations, implementations by non-authors, public deployment, experience across ecosystem roles, and reports of implementation problems.

**Design delta:** adoption evidence should include outside implementation and difficulty reports, not only green CI.

### IETF — interoperability plus operational experience

RFC 6410 combines independent interoperability with widespread deployment and successful operational experience, while checking errata and unused complexity.

**Design delta:** "two adapters passed once" is a milestone, not protocol maturity.

### Khronos — public CTS plus versioned result records

OpenXR and Vulkan expose public conformance test suites and publish conformant product records. OpenXR CTS reports bind source-revision information, and the suite is explicitly intended to improve consistent behavior among runtimes.

**Design delta:** every Signet result should bind the exact suite revision and environment; historical results should remain preserved.

### OpenID — conformance plus real pairwise events

OpenID combines an open-source conformance suite with pairwise interoperability events, then feeds implementer findings back into specifications and tests.

**Design delta:** combine self-conformance with pairwise events and publish attempted/possible pairings.

### Web Platform Tests / Interop — continuous differential evidence

WPT runs common tests across independent browser engines. Interop 2026 continuously publishes per-engine and shared scores for selected areas tied to real compatibility pain.

**Design delta:** a future Signet dashboard should measure shared behavior across implementations, not a vanity count of tests passed by one implementation.

### OGC — abstract assertions are separate from executable tests

OGC separates abstract testable requirements (ATS), executable tests (ETS), and modular conformance classes.

**Design delta:** Signet test IDs should reference the normative assertion they exercise, and profiles should state exactly which conformance classes they require.

### HLA / SpaceFOM — common architecture did not guarantee semantic interoperability

Space-simulation literature reports incompatible HLA Federation Object Models across organizations despite use of the same HLA architecture. SpaceFOM was created to provide a shared reference model. NATO certification work likewise describes interoperability as depending on HLA compliance plus a shared exchange model, federation agreement, and certification tests.

**Design delta:** common wire transport is not enough. L1 semantic profiles and L4 profile-specific tests are required to avoid reproducing this failure pattern.

### Metaverse Standards Forum — implementation work is the adoption signal

The Forum emphasizes pilots, testbeds, plugfests, open tooling, and implementation prototyping.

**Design delta:** stars, Discord size, membership, and documents are discovery/community indicators, not proof that independent systems interoperate.

## Adoption scorecard v1

Report these dimensions separately.

### Implementation diversity

- independent adapter authors/teams;
- independent protocol implementations;
- independent core/server implementations;
- distinct engines/integration modes;
- externally authored protocol/profile proposals.

### Validation depth

- conformance classes implemented;
- pairwise matrix coverage: attempted / eligible;
- pairwise pass rate;
- replay corpus coverage;
- clean third-party reproductions;
- unresolved reproducibility failures.

### Operational freshness

- median age of compatibility evidence;
- supported game versions with current evidence;
- claims invalidated by updates;
- time to revalidate after drift;
- stale claims still visible to users.

### Ecosystem maintenance

- third-party maintained adapters;
- maintainers from independent projects;
- outside contributors to the conformance suite;
- outside reproductions or bug reports that changed protocol/tests;
- stale adapters retained with explicit historical status.

### Usability

- clean-machine time to first reproduced interop result;
- undocumented/manual maintainer steps;
- reproducible setup success rate;
- fraction of failures with actionable diagnostics.

Popularity indicators such as stars, forks, downloads, traffic, and community size may be tracked separately but must never be summed into an interoperability score.

## Ecosystem milestone ladder — evidence gates

### M0 — author-controlled beta

Required: published protocol, at least one reference implementation, executable bounded conformance tests.

**Current Signet state:** this is the strongest milestone established by the evidence reviewed in this pass.

### M1 — independent adapter proof

Required: at least two adapters controlled by independent authors/teams, one minimal shared profile, pairwise test evidence, and public replay/test artifacts.

### M2 — heterogeneous integration proof

Required: three meaningfully different integration modes/engines, explicit capability/fallback tests, and clean third-party reproduction of at least one pair.

### M3 — reproducible registry

Required: immutable/versioned claim records, exact suite/environment identity, freshness/invalidation rules, and outsider reproduction records.

### M4 — independent protocol implementation

Required: alternate wire/client/core implementation from a different codebase, cross-implementation replay/pairwise parity, and ambiguity findings fed back into the contract.

### M5 — operational ecosystem

Required: third-party maintained adapters surviving real game/protocol updates, visible known incompatibilities, and governance that includes independent implementers. Widespread use must not be inferred from downloads alone.

## Immediate L4 experiment order

1. Freeze a **minimal core profile** and map every current test to an explicit assertion.
2. Add missing **wire/event sequence fixtures**, beginning with numbered-intent semantics.
3. Publish a portable **golden replay corpus**.
4. Recruit one independent adapter author and require implementation from public contract material.
5. Run a public **pairwise matrix** against a reference adapter/core.
6. Publish raw result artifacts and a first **registry record**.
7. Have a third party reproduce one result from a clean environment.
8. Only then call the first independent interoperability milestone reached.

## Cross-lane handoffs

### To L1

**Finding:** current conformance evidence does not cover all Signet/1 wire/event semantics; the Minecraft gateway's documented unnumbered-intent gap is a concrete negative fixture.

**Requested follow-up:** define profile-scoped normative assertions for intent sequencing, event semantics, capability negotiation, and other wire behavior that L4 can test.

### To L2

**Finding:** independent reproduction needs adapter builds that do not depend on maintainer-local setup.

**Requested follow-up:** make generated/scaffolded adapters emit reproducible build/run metadata and game-version fingerprints.

### To L3

**Finding:** registry trust labels must distinguish self-tested, independently reproduced, pairwise interoperated, signed, and fresh.

**Requested follow-up:** bind package/directory trust UI to typed evidence claims rather than one global verification state.

## Contradictions and corrections to preserve

1. **"Passes the suite" can support a broad "compatible with Signet/1" shorthand.**  
   L4 correction: the current suite covers named test families only; broader compatibility should remain profile-scoped until wire/event/pairwise evidence exists.

2. **A shared protocol eliminates interoperability problems.**  
   HLA evidence: common architecture still produced incompatible domain FOMs; common semantic models and federation agreements were required.

3. **Multiple bundled examples imply adoption.**  
   Correction: examples under one maintainer/code lineage are not independent ecosystem implementations.

4. **High pass percentage means broad interoperability.**  
   Correction: always publish eligible/attempted denominator and skips.

5. **A green compatibility result stays current until a test fails.**  
   Correction: version drift should proactively invalidate or expire claims before rerun.

## Next recursion

Search next for:

- native-language HLA/DIS federation failures and certification lessons;
- game/mod/plugin compatibility registries that became stale after upstream updates;
- standards with strong conformance programs but weak deployment;
- interoperability programs that removed unused features after implementation experience;
- public plugfest methodologies with machine-readable pair matrices;
- independent Signet adapters/forks/external implementers as the ecosystem forms;
- benchmark-gaming where implementations overfit a public suite while remaining incompatible off-suite.

Prefer reproducible failure reports and implementation matrices over generic interoperability commentary.


---

# Second evidence pass — layered profiles and living standards

## OBSERVED — even a successful domain profile needs further profile layers

NASA's Artemis Distributed Simulation is a useful current stress test for the idea that one accepted interoperability standard or domain profile can finish the semantic problem.

The Artemis work explicitly builds on **HLA + SpaceFOM**, but still requires additional common datatypes, message definitions, execution protocols, object classes, and interaction classes for the Artemis mission domain. Those additions are packaged in an ADS Federation Object Model.

This yields a stronger L4 rule:

~~~text
BASE_STANDARD_CONFORMANT
+ DOMAIN_PROFILE_CONFORMANT
!=
MISSION_PROFILE_COMPLETE
~~~

A real ecosystem can remain interoperable while adding narrower profiles above a stable core, but every additional profile needs its own assertions, vectors, pairwise evidence, and version/freshness identity.

**Design delta for Signet:** do not aim for one permanently complete universal semantic vocabulary. Stabilize a small core, then let narrower profiles compose above it. L4 should validate each profile boundary independently and also validate the claimed compositions.

## OBSERVED — standards evolve from implementation experience

NASA's 2025 SpaceFOM lessons-learned work makes a complementary point: the first SpaceFOM deliberately focused on the highest-value core interoperability problems, shipped a practical initial version, then used real deployments and a growing implementation community to drive the next version.

This supports the L4 milestone ordering already proposed:

~~~text
SMALL_TESTABLE_CORE
-> REAL IMPLEMENTATIONS
-> FAILURE / GAP EVIDENCE
-> PROFILE OR STANDARD REVISION
-> REVALIDATION
~~~

rather than:

~~~text
DESIGN UNIVERSAL MODEL FIRST
-> WAIT FOR COMPLETENESS
~~~

The practical consequence is that **test-suite findings are protocol input**, not just a release gate. An ambiguity discovered by independent adapters should produce a durable handoff to L1 and remain represented as a regression fixture after the semantic correction.

## OBSERVED — cross-standard interoperability needs explicit bridges

NASA's 2022 work on HLA, RPR FOM, SMP, and SpaceFOM describes the lack of rules/guidelines for interoperability across those standards and presents explicit bridge solutions.

For Signet, this argues against claiming that external standards can be "supported" merely by mentioning them in documentation.

A bridge claim should bind:

- source standard/profile and version;
- target Signet profile and version;
- semantic mapping;
- unsupported or lossy fields;
- time/authority model translation;
- generated or hand-written bridge implementation;
- conformance vectors in both directions where meaningful;
- known non-equivalences.

### Cross-standard bridge state

~~~text
LOSSLESS_FOR_DECLARED_SUBSET
LOSSY_WITH_DECLARED_FALLBACKS
ONE_WAY_ONLY
STRUCTURALLY_MAPPABLE_BUT_SEMANTICALLY_UNVALIDATED
INCOMPATIBLE
UNKNOWN
~~~

This is a future validation shape, not a requirement that Signet implement HLA, DIS, OpenXR, glTF, or any other external standard now.

## L4 design correction — profile composition must be tested, not inferred

If future Signet profiles compose, for example:

~~~text
core@1
+ fps@1
+ inventory@1
+ vehicle@1
~~~

then passing each profile independently does not prove their composition is conflict-free.

L4 should eventually add composition fixtures for:

- overlapping field/semantic ownership;
- contradictory fallback rules;
- incompatible units or coordinate frames;
- conflicting authority assumptions;
- event ordering;
- lifecycle coupling;
- version combinations;
- optional-feature interactions.

Candidate evidence state:

~~~text
PROFILE_A_PASS
+ PROFILE_B_PASS
!= PROFILE_A_PLUS_B_PASS
~~~

That becomes especially important once independently governed profiles appear.
