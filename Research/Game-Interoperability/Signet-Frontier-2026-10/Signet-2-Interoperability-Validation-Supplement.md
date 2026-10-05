# Supplementary Methods and Evidence Specification

## For *From Shared Meaning to Verifiable Interoperability*

**Version:** Draft 0.1  
**Date:** 2026-10-05  
**Scope:** Signet 2 validation methods, preregistration, fixtures, evidence records, and reproducibility details.

This supplement contains implementation-facing material intentionally removed from the shorter core manuscript. It does not broaden the paper's claims.

---

## S1. Claim-to-test matrix

| Signet 2 claim / design goal | Hypothesis | Experiment | Primary metric | Failure signal |
|---|---|---|---|---|
| A shared hub replaces pairwise translator construction | H1 | Add an independent game to the declared profile | Peer-specific exception count; existing translators modified | Existing peer translators require game-specific branches or tables |
| Shared intents/archetypes carry common meaning | H2 | Independent translators execute the same fixtures | Authoritative checkpoint/event agreement | Unpermitted authoritative divergence |
| Capabilities let heterogeneous games coexist | H3 | Negotiation matrix including incompatible cases | Incorrect acceptance/rejection count | Unsupported semantics accepted or peers disagree on selected configuration |
| Pinned profiles are deterministic | H4a | Replay identical valid bound profiles | Mapping-decision equality | Same valid profile yields different decisions |
| Pinned profiles remain trustworthy after change | H4b | Mutate each declared dependency | Correct invalidation rate | Stale evidence remains current |
| Calibration makes motion faithful | H5 | Repeated timed intent traces | Trajectory error versus tolerance | Error exceeds declared profile tolerance |
| Resolver adds practical value | H6 | Held-out paired resolver benchmark | Severe-error rate plus human effort | No preregistered improvement or severe errors increase |
| Claims are independently usable | H7 | Clean-room reproduction | Reproduction state | Private/undocumented intervention is required |

No row inherits success from another.

---

## S2. Preregistration record

Before the independent implementation phase, freeze:

~~~yaml
study_id: ...
protocol_commit: ...
semantic_profile:
  id: ...
  revision: ...
vocabulary_revision: ...
archetype_catalog_revision: ...
suite_commit: ...

fixtures:
  public: [...]
  held_out_commitment: ...

primary_hypotheses:
  - H1
  - H2
  - H3
  - H4a
  - H4b
  - H5
  - H6
  - H7

primary_metrics: [...]
acceptance_thresholds: [...]
tolerances: [...]
independence_criteria: [...]
exclusion_rules: [...]
analysis_plan_revision: ...
timestamp: ...
~~~

Held-out cases need not be public before evaluation, but their immutable digest/commitment should be.

Post-registration protocol, fixture, threshold, or analysis changes are amendments. They do not silently replace the original plan.

---

## S3. Implementation-independence classes

Use explicit labels:

~~~text
REFERENCE_IMPLEMENTATION
SAME_CODEBASE_VARIANT
INDEPENDENT_ADAPTER
INDEPENDENT_PROTOCOL_IMPLEMENTATION
INDEPENDENT_CORE_OR_SERVER
~~~

The first meaningful Signet 2 validation milestone should reach at least INDEPENDENT_ADAPTER.

An adapter written independently against the official SDK is independent evidence for the adapter/profile boundary, but not for the SDK or wire implementation.

Clarification questions from independent implementers should be logged publicly. If a clarification changes intended semantics, it is itself protocol-design evidence.

---

## S4. Golden replay record

Each replay fixture should bind:

~~~yaml
fixture_id: ...
semantic_profile: ...
profile_revision: ...
vocabulary_revision: ...
archetype_catalog_revision: ...

initial_authoritative_state: ...
ordered_intents: [...]
negotiated_session_profile: ...

expected:
  checkpoints: [...]
  authoritative_events: [...]

tolerances:
  position: ...
  velocity: ...
  orientation: ...
  timing: ...

allowed_presentation_differences: [...]
forbidden_authoritative_differences: [...]
~~~

A replay is an interoperability artifact, not merely captured gameplay.

---

## S5. Required first fixture families

### S5.1 Intent semantics

At minimum:

~~~text
button pressed      != intent requested
intent requested    != intent accepted
intent accepted     != intent applied
shot requested      != shot fired
damage requested    != damage applied
input received      != input applied exactly once
~~~

Exercise:

- numbered input;
- duplicate input;
- delayed input;
- dropped input;
- out-of-order input;
- reconnect/replay;
- unsupported intent;
- missing required intent;
- conflicting simultaneous intents;
- impossible input frequency.

### S5.2 Archetype semantics

Exercise:

- valid representation;
- alternative permitted representation;
- missing representation;
- forbidden substitution;
- archetype version mismatch;
- permitted fallback;
- forbidden fallback.

The profile should define normative guarantees and explicit non-guarantees for each core archetype.

### S5.3 Lifecycle semantics

Exercise:

~~~text
spawn
movement
collision
attack
damage
death
respawn
despawn
~~~

Important distinctions include:

~~~text
death != despawn
teleport requested != authoritative transform changed
inventory displayed != authoritative possession
~~~

### S5.4 Capability negotiation

Exercise:

- complete compatible intersection;
- missing optional capability;
- missing required capability;
- permitted fallback;
- forbidden fallback;
- unknown optional extension;
- unknown required extension;
- incompatible required parameters;
- malformed declaration;
- inconsistent/misreported declaration.

### S5.5 Profile freshness

Invalidate or re-evaluate evidence after:

- game fingerprint change;
- translator source/artifact change;
- protocol/profile revision;
- vocabulary revision;
- archetype semantic revision;
- calibration-procedure change;
- relevant server-rule revision.

---

## S6. Translation-profile identity

A reproducible translation profile should bind more than selected mappings:

~~~yaml
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

decisions: [...]

invalidation:
  triggers:
    - game_fingerprint_changed
    - translator_changed
    - protocol_changed
    - semantic_profile_changed
    - vocabulary_changed
    - archetype_semantics_changed
    - calibration_procedure_changed
~~~

Core invariant:

~~~text
PINNED != ETERNALLY_VALID
~~~

---

## S7. Capability-negotiation evidence

Capability declaration and negotiated configuration are separate.

Model each participant offer as:

~~~text
K_g = (
  intents_emittable,
  archetypes_representable,
  parameters,
  requirements,
  fallback_policy
)
~~~

The negotiation result is:

~~~text
NEGOTIATE(P, K_server, K_g1, ..., K_gm)
->
NEGOTIATED_SESSION_PROFILE
or
REJECT(reason)
~~~

Recommended capability properties include:

~~~text
REQUIRED
OPTIONAL
PRESENTATION_ONLY
AUTHORITATIVE_INPUT
AUTHORITATIVE_OUTPUT
FALLBACK_ALLOWED
FALLBACK_FORBIDDEN
~~~

A correct rejection is a successful protocol outcome.

An **incorrect acceptance**—starting a session whose authoritative semantics are not mutually supported—is the more dangerous failure.

---

## S8. Motion/calibration measurements

For every supported movement mode:

1. start from defined authoritative state;
2. apply a timed intent trace;
3. capture expected authoritative trajectory;
4. capture resulting authoritative trajectory;
5. compute error metrics;
6. separate presentation-only differences;
7. compare against preregistered tolerances.

Report:

- number of independent runs;
- mean and median error;
- 95th percentile error;
- maximum error;
- time outside tolerance;
- first divergence time;
- jump-apex error;
- landing-time error;
- angular/orientation error;
- calibration repeatability.

Relevant non-scalar behavior may include:

- acceleration/deceleration;
- friction;
- air control;
- slope/step logic;
- crouch transitions;
- momentum preservation;
- tick-rate dependence;
- collision geometry;
- gravity variation;
- ladders/swimming/flying/vehicles.

The pass/fail criterion is the preregistered engineering tolerance, not a p-value.

---

## S9. Resolver benchmark

Compare at least:

1. hand-written rules;
2. text-similarity baseline;
3. proposed contrastive ranking model.

Every candidate set should permit:

~~~text
NONE / NO SAFE MAPPING
~~~

Otherwise a closed list containing only wrong candidates forces a semantic error.

### S9.1 Dataset requirements

Include:

- development cases;
- immutable held-out cases;
- multiple games/genres;
- multiple candidate-set sizes;
- deliberately ambiguous cases;
- cases with no acceptable candidate;
- independently annotated labels;
- disagreement records.

Do not tune on the held-out set.

### S9.2 Primary outcomes

Preregister in this order:

1. severe semantic error rate;
2. human time to confirmed mapping;
3. top-1 accuracy;
4. mean reciprocal rank;
5. top-k recall;
6. abstention performance.

A resolver is not “better” if ranking accuracy rises while severe semantic errors rise.

### S9.3 Statistical analysis

Use paired analysis because all resolver classes receive the same held-out cases.

For paired binary outcomes such as top-1 correctness:

- show the raw disagreement table;
- report paired effect size;
- optionally use McNemar's test if inferential testing is warranted.

For continuous paired outcomes such as confirmation time or reciprocal rank:

- report paired differences;
- report 95% confidence intervals;
- use paired permutation/bootstrap inference when parametric assumptions are weak.

Multiple secondary metrics should be exploratory or use a preregistered multiplicity correction.

If model scores are not explicitly calibrated probabilities, describe them as ranking scores only.

---

## S10. Deterministic conformance reporting

Normative conformance is primarily exact, not statistical.

Report:

~~~text
PASS
FAIL
SKIP
NOT_APPLICABLE
~~~

with:

- assertions eligible;
- assertions attempted;
- assertions passed;
- assertions failed;
- assertions skipped;
- reason for every skip;
- exact failure IDs;
- first divergent authoritative state/event.

A single reproducible violation of a MUST-level assertion is evidence against conformance for that tested version/profile.

Do not hide failures inside an aggregate success percentage.

---

## S11. Pairwise matrix reporting

For every compatibility campaign publish:

~~~text
eligible_pairings
attempted_pairings
not_attempted_pairings
passed_pairings
failed_pairings
skipped_pairings
~~~

For each cell bind:

- implementation identities;
- game versions/fingerprints;
- semantic-profile revision;
- translation-profile digests;
- negotiated session profile;
- suite revision;
- pass/fail/skip;
- fallback use;
- raw evidence.

A percentage without a denominator is insufficient.

---

## S12. Compatibility evidence record

Recommended machine-readable shape:

~~~yaml
claim_id: ...
claim_type: pairwise_interop

protocol:
  version: ...

semantic_profile:
  id: ...
  version: ...

vocabulary_revision: ...
archetype_catalog_revision: ...

negotiation:
  offers: [...]
  selected_profile_digest: ...

participants:
  - game_id: ...
    game_version: ...
    game_fingerprint: ...
    translator_source: ...
    translator_commit: ...
    translator_artifact_digest: ...
    translation_profile_digest: ...
    implementation_class: ...
  - ...

suite:
  source: ...
  revision: ...
  artifact_digest: ...
  fixture_families: [...]

environment:
  os: ...
  architecture: ...
  runtime: ...

result:
  status: PASS|PARTIAL|FAIL
  passed: ...
  failed: ...
  skipped: ...
  fallbacks: [...]
  known_failures: [...]

evidence:
  raw_log_digests: [...]
  replay_digests: [...]
  reproduction_instructions: ...

reproduction:
  state: ...
  reproducer: ...
  observed_at: ...

freshness:
  observed_at: ...
  invalidation_triggers: [...]
~~~

Registry invariants:

~~~text
SIGNED_RESULT             != CORRECT_RESULT
SELF_TESTED               != INDEPENDENTLY_REPRODUCED
CONFORMANCE               != PAIRWISE_INTEROPERABILITY
PAIRWISE_INTEROPERABILITY != ECOSYSTEM_ADOPTION
POPULARITY                != COMPATIBILITY
PINNED_PROFILE            != FRESH_PROFILE
SKIPPED                   != PASSED
~~~

Negative results remain addressable. A later passing version may supersede a historical failure without erasing it.

---

## S13. Reproduction protocol

A clean reproducer should:

1. resolve exact protocol/profile/suite revisions;
2. verify source and artifact digests;
3. obtain required game assets locally;
4. verify game fingerprints;
5. build/install in a documented clean environment;
6. run declared conformance fixtures;
7. run relevant pairwise replays;
8. preserve raw logs, skips, fallbacks, environment metadata, and replay hashes;
9. publish a separate reproduction record;
10. never overwrite conflicting results.

Reproduction states:

~~~text
CONFIRMS
PARTIALLY_CONFIRMS
FAILS_TO_REPRODUCE
INVALIDATED_BY_VERSION_DRIFT
TEST_SUITE_DISPUTED
ENVIRONMENT_SPECIFIC
UNKNOWN
~~~

---

## S14. Stopping rules

To avoid a moving target:

- deterministic conformance testing stops when the preregistered suite is complete;
- a normative failure is recorded even if immediately fixed;
- semantic clarification continues only through the public clarification mechanism;
- resolver sample size and held-out set are fixed before evaluation;
- failures can seed a future regression suite but remain failures in the original experiment;
- protocol amendments after observed failures receive new revision identity.

---

## S15. Initial milestone ladder

### M0 — architecture proposal

- design documented;
- no independent interoperability evidence required.

### M1 — minimal semantic core

- versioned intents/archetypes;
- normative assertions;
- capability negotiation;
- golden replay corpus;
- reference implementation.

### M2 — independent adapter proof

- one independent adapter;
- clarification log;
- conformance results public.

### M3 — heterogeneous pairwise proof

- two meaningfully different games;
- negotiated profile;
- public pairwise fixtures;
- raw evidence;
- explicit fallbacks/skips.

### M4 — clean reproduction

- independent third party reproduces a pairwise claim;
- exact artifacts/instructions public.

### M5 — independent protocol implementation

- alternate core/SDK/protocol implementation;
- cross-implementation tests;
- ambiguity findings fed back into the specification.

### M6 — operational ecosystem

- multiple independently maintained translators;
- compatibility registry;
- freshness/expiry;
- version-drift reruns;
- third-party tests;
- documented failures and revisions.

---

## S16. Evidence labels

Recommended claim classes:

~~~text
STRUCTURAL_CONFORMANCE
BEHAVIORAL_CONFORMANCE
INDEPENDENT_IMPLEMENTATION
PAIRWISE_INTEROPERABILITY
REPRODUCTION
OPERATIONAL_FRESHNESS
~~~

Recommended result states:

~~~text
PASS
PARTIAL
FAIL
NOT_APPLICABLE
NOT_TESTED
INVALIDATED
UNKNOWN
~~~

The evidence model is intentionally multi-dimensional. A single global “verified” flag is insufficient.
