# From Shared Meaning to Verifiable Interoperability

## A validation and ecosystem response to *Signet 2: intents, archetypes and translation profiles*

**Status:** research response / working paper  
**Date:** 2026-10-05  
**Scope:** Signet L4 — Validation, Ecosystem & Adoption  
**Responds to:** *Signet 2: intents, archetypes and translation profiles*, design document v0.1, 2026-10-05, draft proposal, not yet implemented.

---

## Abstract

*Signet 2* proposes replacing pairwise game-to-game translation with a shared semantic layer. Player actions are translated into a common vocabulary of **intents**; world objects are represented as **archetypes**; games declare **capabilities**; calibration records local control and motion mappings; and **translation profiles** pin decisions so that runtime behavior is deterministic. A separate **Signet Forge** tool is proposed for reviewing and improving difficult mappings, optionally with an AI resolver.

This is a materially stronger architecture than a mesh of bespoke pairwise translators. The central idea is well-established in other interoperability domains: a canonical representation can reduce the number of mapping boundaries from quadratic growth toward linear growth. The important qualification is that this reduces **translator topology**, not the semantic validation problem.

A game can map to a shared vocabulary and still disagree with another implementation about what an intent means, what an archetype guarantees, which capability combinations are valid, how authority is preserved, or when a translation is too lossy to claim interoperability. A pinned profile makes a choice repeatable; it does not make the choice correct. A server can remain authoritative over simulation state; that does not make cheating impossible. A closed-list model can be bounded; that does not make its ranking calibrated or semantically valid.

We therefore propose a validation model in which Signet 2 compatibility is not a Boolean property. It is a set of versioned, falsifiable claims over a declared semantic surface. The core empirical unit should be a **pairwise interoperability claim** backed by replayable fixtures, exact profile and capability identities, raw evidence, and independent reproduction.

The paper's central recommendation is:

```text
SHARED_MEANING
+ PINNED_TRANSLATION
+ SERVER_AUTHORITY
!=
PROVEN_INTEROPERABILITY
```

The missing term is evidence:

```text
PROVEN_INTEROPERABILITY =
  explicit semantics
+ negotiated capability intersection
+ independently implemented mappings
+ adversarial conformance fixtures
+ pairwise execution
+ reproducible evidence
+ version/freshness identity
```

This response turns the Signet 2 proposal into a concrete experimental program.

---

## 1. What Signet 2 gets right

### 1.1 Pairwise translation does not scale

The proposal correctly identifies the growth problem in direct game-to-game translation. If every game requires bespoke mappings to every other game, integration edges grow quadratically.

The proposed hub is analogous to a canonical-data-model architecture: each participant maps to and from a shared representation instead of learning every peer's representation. NIST's Smart Grid interoperability work describes the same architectural motivation: canonical models reduce the number of transformations needed as independently modeled systems grow.

This is a legitimate and important improvement.

However, the complexity is **moved and compressed**, not eliminated. The shared vocabulary becomes a high-leverage artifact whose semantics, versioning, governance, and validation affect every participant.

The correct claim is therefore:

```text
PAIRWISE_MAPPING_TOPOLOGY:
  O(n^2) -> approximately O(n)
```

not:

```text
INTEROPERABILITY_COMPLEXITY:
  O(n^2) -> solved
```

### 1.2 Separating intent from presentation is a strong design boundary

The distinction between:

- what a player is trying to do;
- what the authoritative simulation decides happened; and
- how each local game presents that result

is useful.

It avoids forcing Doom, Minecraft, OpenArena, or future games to share assets, UI, animation, or local implementation details. It also gives the protocol a chance to define stable semantic boundaries independent of specific engines.

This aligns with the strongest lesson from HLA/SpaceFOM-style systems: syntactic connectivity is not enough; interoperability requires common semantics for the concepts that cross the boundary.

### 1.3 Keeping AI outside the real-time simulation loop is sensible

The proposal explicitly keeps AI resolution outside the 20 Hz simulation path. Difficult mappings are suggested, reviewed, then pinned. Runtime uses the stored profile rather than repeatedly invoking a model.

This is preferable to making live simulation dependent on probabilistic inference.

It improves:

- replayability;
- latency predictability;
- auditability;
- offline operation;
- debuggability;
- model replaceability.

### 1.4 Human confirmation is better than opaque automatic mapping

The proposal's resolver chooses from a closed candidate set and lets a person confirm difficult mappings. This is safer than allowing a model to synthesize arbitrary protocol actions or asset identifiers.

The closed list establishes a useful **output bound**.

But output boundedness is not semantic correctness. That distinction becomes important in Section 5.

### 1.5 Capabilities are necessary

A game that cannot jump, display a class of object, represent inventory, or expose a required control surface should say so.

Capability declaration is therefore a necessary part of the architecture.

The current proposal, however, treats capability declaration too much like a static feature list. Mature negotiation systems distinguish:

- supported;
- required;
- optional;
- mutually acceptable configurations;
- incompatible combinations;
- fallback behavior.

IETF capability-negotiation work is useful precedent here: capability declaration alone is not enough; endpoints negotiate an actual configuration and specify behavior when required capabilities are not shared.

---

## 2. The central correction: shared meaning must be tested as a protocol

The phrase "shared meaning" is doing most of the architectural work.

That makes it the part that must be specified and tested most rigorously.

Consider the intent:

```text
fire
```

Possible implementations may silently disagree about whether this means:

- trigger pressed;
- weapon activation requested;
- a shot was successfully instantiated;
- ammunition was consumed;
- an attack cooldown began;
- a projectile exists;
- damage was applied.

Those are not interchangeable.

Likewise:

```text
weapon.ranged
```

does not by itself answer:

- effective range;
- projectile versus hitscan;
- rate of fire;
- ammunition semantics;
- area effects;
- cooldown;
- equip time;
- whether it can damage the same target classes;
- whether aim direction is continuous or quantized.

If Signet 2 intentionally leaves those properties to the authoritative server, that is valid—but then the protocol must say which properties belong to the archetype and which belong to simulation rules.

The key validation invariant is:

```text
SAME_LABEL != SAME_SEMANTICS
```

A semantic vocabulary becomes interoperable only when independent implementations can agree on observable behavior under the same test conditions.

---

## 3. A stronger model of interoperability

We propose that a Signet 2 interoperability claim be represented as:

```text
I = (
  protocol_version,
  semantic_profile,
  vocabulary_revision,
  archetype_catalog_revision,
  capability_offer,
  capability_selection,
  translation_profile,
  implementation_A,
  implementation_B,
  test_suite_revision,
  environment,
  evidence,
  freshness
)
```

A useful public statement is then not:

> Game X supports Signet 2.

It is closer to:

> Translator X revision A interoperated with implementation Y revision B under profile P, negotiated capability set C, vocabulary V, catalog K, and test suite T on environment E; evidence R was independently reproduced on date D.

That sounds heavier, but the machine-readable registry carries the detail. Humans can still see a compact badge.

The important point is that the badge must be derived from a bounded claim rather than replacing it.

---

## 4. Translation profiles: deterministic is not equivalent to correct

The proposal compares translation profiles to package-manager lock files. This is a productive analogy, but it needs one correction.

A pinned decision gives:

```text
same profile entry
-> same selected mapping
```

It does not necessarily give:

```text
same profile entry
-> same game behavior
```

because the game, mod set, server configuration, translator, protocol vocabulary, archetype catalog, motion parameters, or simulation rules may have changed.

Therefore every profile should bind at least:

```yaml
profile_format: ...
profile_revision: ...
game:
  id: ...
  version: ...
  fingerprint: ...
translator:
  source: ...
  commit: ...
  artifact_digest: ...
protocol:
  version: ...
semantic_profile:
  id: ...
  version: ...
vocabulary:
  revision: ...
archetype_catalog:
  revision: ...
calibration:
  procedure_version: ...
  measurements: ...
resolver:
  type: human|table|model
  identity: ...
  revision: ...
decisions:
  ...
invalidation:
  triggers:
    - game_fingerprint_changed
    - translator_changed
    - vocabulary_changed
    - archetype_semantics_changed
    - calibration_procedure_changed
```

This produces a stronger invariant:

```text
PINNED != ETERNALLY_VALID
```

A translation profile should be deterministic **and invalidatable**.

---

## 5. The AI resolver: bounded suggestion, not semantic authority

The Signet 2 proposal is careful not to put AI in the runtime loop. That is good.

The remaining risk is epistemic: a model ranking candidate mappings can look more certain than the evidence warrants.

A score such as:

```text
bow 0.81
crossbow 0.74
trident 0.55
```

must not automatically be interpreted as:

```text
81% probability that bow is semantically correct
```

unless the scoring system has been explicitly calibrated for that interpretation.

The resolver should therefore be evaluated as a ranking component.

### 5.1 Recommended experiment

The paper already proposes a 30-case comparison among:

- hand-written rules;
- text similarity;
- contrastive model.

Expand this into a reproducible benchmark with:

1. hidden test cases not used to tune prompts or rules;
2. multiple games and genres;
3. multiple candidate-set sizes;
4. deliberately ambiguous candidates;
5. cases where **none of the candidates is acceptable**;
6. independent human annotations;
7. disagreement labels rather than forced consensus;
8. top-1 accuracy;
9. top-k recall;
10. mean reciprocal rank;
11. abstention quality;
12. calibration only if scores are intended as probabilities;
13. error severity, not only error count.

The critical addition is a **NONE / NO SAFE MAPPING** option.

A closed list prevents invention outside the candidate set, but if all candidates are wrong, forcing the resolver to pick one converts boundedness into a guaranteed semantic error.

### 5.2 Training data provenance

Signet Forge's proposed feedback loop turns approved mappings into training examples.

That creates a governance surface:

- who approved the mapping;
- which game version it applied to;
- whether the mapping was later invalidated;
- whether two communities disagree;
- whether the example is licensed for model training;
- whether popularity is being mistaken for semantic correctness.

Approved examples should therefore remain versioned evidence records, not be collapsed into an anonymous training set.

---

## 6. Capabilities need negotiation, not just declaration

The Signet 2 proposal gives examples such as:

- "I cannot jump";
- "I can show weapons and health pickups."

Those declarations are useful but too coarse for robust interoperability.

Consider two peers:

```text
A supports:
  move
  fire
  weapon.ranged

B supports:
  move
  weapon.ranged
```

Does the match:

- reject B;
- place B in observer mode;
- disable firing globally;
- give B an alternate intent;
- allow asymmetric gameplay;
- use a fallback archetype;
- start with a warning?

The protocol needs a negotiated configuration, not merely two capability documents.

### 6.1 Proposed capability categories

Each capability should be classifiable as:

```text
REQUIRED
OPTIONAL
PRESENTATION_ONLY
AUTHORITATIVE_INPUT
AUTHORITATIVE_OUTPUT
FALLBACK_ALLOWED
FALLBACK_FORBIDDEN
```

and carry parameters where relevant.

For example:

```yaml
intent: move
status: REQUIRED
parameters:
  dimensions: 2
  analog_resolution: continuous

intent: jump
status: OPTIONAL
fallback:
  mode: ignore

archetype: weapon.ranged
status: REQUIRED
representation:
  minimum_semantic_class: generic_ranged
```

### 6.2 Negotiation result must be explicit

The server should emit the selected session surface:

```text
CAPABILITY_OFFER_A
+ CAPABILITY_OFFER_B
+ SERVER_PROFILE
- INCOMPATIBILITIES
- DISALLOWED_FALLBACKS
=
NEGOTIATED_SESSION_PROFILE
```

This selected profile becomes part of replay and compatibility evidence.

---

## 7. "Automatically play with all the others" should be narrowed

The proposal says that if each game translates to and from shared meaning, a new game can automatically play with the others.

That statement is directionally correct about architecture, but empirically too broad.

A new game can automatically become a **candidate** for interoperability with existing participants if:

1. it implements the same protocol version;
2. it maps the required shared semantics correctly;
3. its declared capabilities have a valid intersection with the target session;
4. its translation profile is valid for its current version;
5. unsupported semantics have explicit fallback or rejection rules;
6. pairwise behavior passes the relevant tests.

The stronger formulation is:

```text
ONE_TRANSLATOR
-> access to the shared interoperability surface

ONE_TRANSLATOR
!= proof of compatibility with every participant
```

The shared hub removes the need to author a bespoke translator for every peer. It does not remove the need to validate the peer combinations that matter.

---

## 8. Server authority improves integrity but does not make cheating impossible

The design document states that because the server is the only authority, a player cannot cheat by modifying their client.

The first half is strong architecture. The second half overclaims.

A server-authoritative model can prevent or reduce classes of cheats where a modified client tries to directly assert authoritative world state.

It does not inherently prevent:

- aimbots;
- input automation;
- impossible-but-protocol-valid timing patterns if not constrained;
- information extraction from data legitimately sent to the client;
- collusion;
- translator defects that emit advantageous intents;
- exploitative capability declarations;
- game-specific observation advantages;
- denial-of-service behavior.

Modern anti-cheat research explicitly studies **server-side** detection of cheating because server authority alone does not make the problem disappear.

The paper should therefore say:

```text
SERVER_AUTHORITY
reduces trust in client-reported state
and blocks classes of state-forgery cheats
```

not:

```text
SERVER_AUTHORITY
=> nobody can cheat by modifying the client
```

This is not merely wording. It changes the test plan.

L4 should add adversarial fixtures for:

- impossible input rates;
- replayed/duplicated intents;
- timing manipulation;
- capability lies;
- malformed profile identifiers;
- translator/state desynchronization;
- stale translation profiles.

---

## 9. Motion profiles need experimental semantics

The proposed calibration wizard measures:

- walking/running speed;
- jump height;
- turn speed;
- eye height;
- maximum step height.

This is a useful start.

But a scalar motion profile may be insufficient for games whose movement is stateful or nonlinear.

Potential differences include:

- acceleration curves;
- deceleration/friction;
- air control;
- slope behavior;
- step-up logic;
- crouch transitions;
- sprint stamina;
- momentum preservation;
- tick-rate dependence;
- collision capsule shape;
- camera versus body orientation;
- variable gravity;
- ladders, swimming, flying, vehicles.

Therefore motion calibration should be defined as an empirical measurement procedure, not just a bag of numbers.

### 9.1 Recommended motion fixture

For each supported movement mode:

1. start from a defined authoritative state;
2. apply a timed intent trace;
3. capture expected authoritative trajectory;
4. capture local displayed trajectory;
5. compute error metrics;
6. identify which differences are presentation-only;
7. fail if authoritative semantics diverge beyond declared tolerance.

This allows "faithful translation" to become a measured statement.

---

## 10. Archetypes should have contracts, not only names

An archetype such as:

```text
weapon.ranged
```

should have a normative semantic contract.

At minimum:

```yaml
id: weapon.ranged
kind: item
guarantees:
  - can initiate a ranged attack intent
authority:
  damage: server
  hit_resolution: server
presentation:
  client_may_substitute: true
non_guarantees:
  - projectile_visual
  - exact_fire_rate
  - exact_animation
```

A richer profile may refine it:

```text
weapon.ranged
  -> weapon.ranged.projectile
  -> weapon.ranged.projectile.single_shot
```

But Signet should resist making the core catalog a universal ontology.

NASA's current SpaceFOM evolution is relevant here: interoperability across heterogeneous systems requires common semantics, but successful standards evolve through real implementations and domain-specific extensions rather than attempting complete semantic coverage before deployment.

The safest strategy is:

```text
SMALL_STABLE_CORE
+ VERSIONED_DOMAIN_PROFILES
+ EXPLICIT_COMPOSITION_TESTS
```

---

## 11. Validation architecture for Signet 2

We propose six evidence levels.

### V0 — structural validity

Examples:

- profile parses;
- identifiers are well-formed;
- required fields exist;
- vocabulary/catalog revisions resolve.

### V1 — single-implementation semantic conformance

Examples:

- intent sequencing;
- archetype contract invariants;
- capability parsing;
- profile invalidation;
- deterministic replay within one implementation.

### V2 — independent translation implementation

A different author implements a translator from the public contract without private guidance.

This tests whether the written specification is sufficient.

### V3 — pairwise interoperability

Two implementations execute the same negotiated semantic profile and agree on authoritative outcomes.

### V4 — clean third-party reproduction

An outsider reproduces the claim from public sources, exact revisions, and documented instructions.

### V5 — operational freshness

The claim remains valid after relevant game, adapter, SDK, vocabulary, catalog, or profile changes.

The levels must not be collapsed:

```text
SELF_CONFORMANCE
!= INDEPENDENT_IMPLEMENTATION
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= OPERATIONAL_ADOPTION
```

---

## 12. A Signet 2 golden replay corpus

The replay corpus should directly target the new architecture.

### 12.1 Intent fixtures

Required cases:

- numbered input;
- duplicate input;
- delayed input;
- dropped input;
- out-of-order input;
- reconnect/replay;
- unsupported intent;
- required intent missing;
- conflicting intents;
- impossible input frequency.

Assertions should distinguish:

```text
intent requested
intent accepted
intent applied
authoritative event emitted
presentation rendered
```

### 12.2 Archetype fixtures

For each core archetype:

- normative guarantees;
- allowed local appearances;
- forbidden substitutions;
- missing representation;
- fallback;
- asymmetric representation;
- profile mismatch.

### 12.3 Capability fixtures

Test:

- compatible optional capabilities;
- missing required capability;
- incompatible required combinations;
- fallback accepted;
- fallback rejected;
- old peer ignoring a new optional capability;
- unknown required extension;
- lying/malformed declaration.

### 12.4 Translation-profile fixtures

Test:

- exact pinned replay;
- stale game fingerprint;
- changed translator digest;
- changed vocabulary;
- changed archetype semantics;
- invalid resolver identity;
- conflicting community profiles;
- deterministic selection under same profile.

### 12.5 Calibration fixtures

Test:

- repeated calibration on same build;
- calibration variance;
- changed keybinds;
- changed movement settings;
- modded movement;
- frame/tick-rate sensitivity;
- nonlinear movement.

---

## 13. The 30-case resolver experiment should become a pre-registered benchmark

The roadmap's proposed 30-case experiment is a useful seed but is too small and too easy to overfit if it becomes both the development set and the reported evidence.

Before implementation, publish:

1. task definition;
2. candidate-generation procedure;
3. scoring metrics;
4. train/development/test split;
5. abstention rules;
6. human-annotation protocol;
7. disagreement handling;
8. failure-severity taxonomy;
9. model/version identity;
10. exact prompts/configuration if applicable.

Then preserve a held-out set.

This avoids a common interoperability failure mode:

```text
IMPLEMENTATION_OVERFITS_PUBLIC_FIXTURES
while
OFF_SUITE_SEMANTICS_REMAIN_INCOMPATIBLE
```

A public suite is necessary; a suite that becomes the entire definition of correctness is fragile.

---

## 14. Community profiles need provenance and conflict semantics

The proposal intentionally allows the community to improve translation profiles through pull requests.

That is useful, but conflicting profiles are inevitable.

The protocol should not assume there is always one globally correct mapping.

A registry should support:

```text
CANONICAL_PROJECT_PROFILE
COMMUNITY_PROFILE
SERVER_POLICY_PROFILE
PLAYER_OVERRIDE
EXPERIMENTAL_PROFILE
DEPRECATED_PROFILE
```

Each should carry:

- maintainer identity;
- game/version scope;
- semantic profile scope;
- creation source;
- review status;
- supersedes relation;
- known failures;
- evidence;
- compatibility claims.

The selection order proposed by Signet 2—player pinned, translator profile, model suggestion, safe default—is an implementation policy. It should not silently become a universal protocol rule unless the interoperability consequences are specified.

---

## 15. Governance follows from the architecture

Once Signet 2 introduces a shared intent vocabulary and archetype catalog, governance becomes part of technical correctness.

Questions include:

- who can add an intent;
- when two intents should remain separate;
- when an archetype is too game-specific;
- whether definitions can change without identifier changes;
- how deprecation works;
- how profiles are versioned;
- who decides whether a mapping is lossy;
- what happens when two communities disagree.

The design document already lists maintenance of vocabulary/catalog and conflicting community profiles as open questions.

L4's recommendation is to make those questions empirically constrained:

> Vocabulary changes should be justified by implementation evidence, unresolved semantic failures, or demonstrated missing use cases—not by taxonomy completeness.

This mirrors successful living interoperability standards: implement a small core, observe real failures, revise, and revalidate.

---

## 16. Proposed compatibility evidence registry

Every public compatibility claim should be machine-readable.

Example:

```yaml
claim_id: signet2-interop-...
claim_type: pairwise_interop

protocol:
  version: signet/2-draft-0
semantic_profile:
  id: core
  version: 0

vocabulary:
  revision: ...
archetype_catalog:
  revision: ...

negotiation:
  offer_a: ...
  offer_b: ...
  selected_profile_digest: ...

subject:
  game: ...
  game_version: ...
  game_fingerprint: ...
  translator_commit: ...
  translator_artifact_digest: ...
  translation_profile_digest: ...
  implementation_class: INDEPENDENT_ADAPTER

counterparty:
  implementation: ...
  commit: ...
  artifact_digest: ...

suite:
  revision: ...
  fixture_families:
    - intent
    - archetype
    - capability
    - replay

result:
  passed: ...
  failed: ...
  skipped: ...
  fallbacks: ...
  known_failures: ...

evidence:
  raw_logs: ...
  replay_digests: ...
  environment: ...
  reproduction_instructions: ...

reproduction:
  state: CONFIRMS
  reproducer: ...
  observed_at: ...

freshness:
  invalidation_triggers:
    - game_version
    - translator
    - protocol
    - vocabulary
    - catalog
    - semantic_profile
```

The registry should preserve negative evidence.

A failed reproduction must not be overwritten by a later pass.

---

## 17. Minimum credible experimental program

The fastest path to making Signet 2 convincing is not implementing every proposed feature.

It is implementing the smallest experiment that can falsify the architecture.

### Experiment A — semantic core without AI

Implement only:

- 5–8 intents;
- 4–6 archetypes;
- explicit capability negotiation;
- translation-profile format;
- deterministic replay.

No model.

### Experiment B — independent translator

Give the public specification and fixtures to a developer who did not design the original translator.

Do not provide private implementation guidance.

Measure:

- time to first successful build;
- clarification questions;
- ambiguous spec points;
- fixture failures;
- pairwise result.

### Experiment C — heterogeneous pair

Use two games with meaningfully different control and representation models.

Do not choose only nearly identical FPS engines.

### Experiment D — adversarial capability mismatch

Deliberately create sessions where:

- one game cannot jump;
- one cannot represent the required archetype;
- one lacks a required input;
- fallback is disallowed.

The correct result may be a rejected session. Graceful refusal is interoperability behavior.

### Experiment E — profile drift

Change:

- game version;
- translator revision;
- vocabulary;
- archetype contract.

Verify that stale claims are invalidated before they are presented as current.

### Experiment F — resolver benchmark

Only after the non-AI path works, test whether a model measurably reduces translation effort without increasing severe semantic errors.

---

## 18. Proposed acceptance criteria for Signet 2's major claims

### Claim: translator growth becomes linear

**Accept when:** each new game requires only mappings to/from the shared semantic model for the declared profile, without new peer-specific code.

**Fail when:** peer-specific exceptions accumulate inside translators or profiles.

### Claim: shared meaning enables cross-game play

**Accept when:** independently built translators agree on authoritative outcomes across a declared semantic profile.

**Fail when:** identical intent/archetype labels produce incompatible outcomes.

### Claim: pinned profiles remove runtime ambiguity

**Accept when:** identical version-bound profiles produce reproducible decisions and stale dependencies invalidate the profile.

**Fail when:** a pinned profile silently remains "valid" after its semantic dependencies change.

### Claim: capabilities let limited games coexist

**Accept when:** session negotiation deterministically selects a valid common configuration or rejects the session with an explicit reason.

**Fail when:** unsupported behavior is silently ignored in ways that alter authoritative fairness.

### Claim: AI helps difficult mappings

**Accept when:** held-out evaluation shows lower human effort or better ranking quality than simpler baselines without increasing severe semantic errors.

**Fail when:** the model merely produces plausible labels or confidence-looking scores.

### Claim: server authority prevents client cheating

**Replace claim.**

**Accept narrower claim when:** authoritative state cannot be directly forged by a client and adversarial intent/replay fixtures are enforced.

---

## 19. Recommended wording changes to the Signet 2 proposal

Several statements can be made stronger by making them narrower.

### Current architectural idea

> each game translates into a shared meaning and out of a shared meaning

### Suggested research wording

> each translator maps a declared subset of game behavior to and from a versioned shared semantic profile; interoperability is established for the negotiated subset by conformance and pairwise evidence.

---

### Current architectural idea

> it can automatically play with all the others

### Suggested research wording

> it becomes eligible to interoperate with other implementations that negotiate a compatible semantic profile; pairwise behavior remains testable evidence.

---

### Current architectural idea

> from here on, the game works with no ambiguity

### Suggested research wording

> the selected translation decisions are pinned and deterministic for the bound dependency set; semantic correctness and freshness remain independently testable.

---

### Current architectural idea

> the server is the only authority, so nobody can cheat by modifying their client

### Suggested research wording

> the server is authoritative over simulation state, reducing trust in client-reported state and blocking classes of direct state-forgery cheats; protocol-valid input abuse and other cheating classes require separate controls.

---

## 20. Broader standards evidence

The proposed validation model is not exotic.

### Canonical-model precedent

NIST's Smart Grid interoperability architecture explicitly discusses canonical data models as a way to reduce transformation growth from quadratic toward linear while preserving local semantic models.

Source:  
https://www.govinfo.gov/content/pkg/GOVPUB-C13-9d0dcbadd953acc4e50af65c754f83bb/pdf/GOVPUB-C13-9d0dcbadd953acc4e50af65c754f83bb.pdf

### Capability-negotiation precedent

IETF RFC 5939 separates supported capabilities, required capability-negotiation extensions, potential configurations, and the actually negotiated configuration.

Source:  
https://datatracker.ietf.org/doc/html/rfc5939

### Independent-implementation precedent

W3C implementation experience and IETF standards maturity both treat independent implementation/interoperation and operational experience as important evidence rather than assuming a specification proves itself.

Sources:  
https://www.w3.org/policies/process/#implementation-experience  
https://www.rfc-editor.org/rfc/rfc6410.html

### Living semantic-profile precedent

NASA's current SpaceFOM work states that heterogeneous simulation interoperability requires common semantics for time, space, physical entities, and execution control, and reports evolution through real multi-organization implementations.

Source:  
https://ntrs.nasa.gov/citations/20260008222

### Pairwise evidence precedent

OpenID's interoperability events publish attempted pairings and pass rates rather than reporting only successful demonstrations.

Source:  
https://openid.net/openid4vp-and-openid4vci-conformance-tests-are-complete-and-open-for-self-certification/

---

## 21. Proposed paper thesis

The strongest publishable position is not adversarial to Signet 2.

It is:

> **Signet 2's shared-intent/archetype architecture is a plausible way to make cross-game integration scale, but its success depends on treating "shared meaning" as an experimentally validated protocol surface rather than a naming convention.**

The contribution of this response is a validation architecture that makes the proposal falsifiable.

The paper can be framed around three transitions:

```text
PAIRWISE TRANSLATION
-> SHARED SEMANTIC HUB

SHARED SEMANTIC HUB
-> NEGOTIATED VERSIONED PROFILES

NEGOTIATED VERSIONED PROFILES
-> REPRODUCIBLE INTEROPERABILITY EVIDENCE
```

The first transition is Signet 2's architectural proposal.

The second and third are the missing validation and ecosystem layers.

---

## 22. Next experiments and paper work

Immediate research/implementation order:

1. formalize 5–8 core intents as normative assertions;
2. formalize 4–6 core archetypes as semantic contracts;
3. define required/optional capability negotiation and selected-session output;
4. define dependency-bound translation-profile identity and invalidation;
5. create golden replay fixtures for intent/request/applied/event distinctions;
6. create adversarial capability-mismatch fixtures;
7. recruit one genuinely independent translator implementation;
8. publish pairwise matrix and raw results;
9. run clean third-party reproduction;
10. only then evaluate the AI resolver against simpler baselines;
11. preserve all failures as paper evidence;
12. revise the protocol from observed ambiguity rather than anticipated completeness.

The most valuable next result is not a larger vocabulary.

It is the first case where two independent implementations interpret the same small vocabulary identically—or fail, and thereby reveal exactly what the specification still needs to say.

---

## Conclusion

Signet 2 moves the project in the right architectural direction.

Its central insight—translate games into and out of a shared semantic layer—can reduce integration-edge growth and create a stable place for capabilities, profiles, and tooling.

But the architecture should resist four seductive equivalences:

```text
SHARED LABELS          != SHARED SEMANTICS
PINNED CHOICE          != CORRECT CHOICE
SERVER AUTHORITY       != CHEAT IMPOSSIBILITY
ONE TRANSLATOR         != UNIVERSAL COMPATIBILITY
```

The solution is not to abandon the design.

It is to make the design measurable.

A small semantic core, negotiated profiles, exact dependency identities, independent implementations, pairwise replay, preserved negative results, and third-party reproduction would turn Signet 2 from an attractive architecture into a credible interoperability standard.

That is the L4 target.
