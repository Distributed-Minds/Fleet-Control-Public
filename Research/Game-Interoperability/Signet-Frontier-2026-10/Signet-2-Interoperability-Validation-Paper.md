
# From Shared Meaning to Verifiable Interoperability

## An experimental validation framework for Signet 2

**Working paper · Draft 0.2 · 2026-10-05**  
**Research lane:** L4 — Validation, Ecosystem & Adoption  
**Responds to:** *Signet 2: intents, archetypes and translation profiles*, design document v0.1, 2026-10-05.

---

## Abstract

Signet 2 proposes a shared semantic architecture for cross-game interoperability. Instead of translating every game directly into every other game, each game maps local actions into a common vocabulary of **intents** and maps shared **archetypes** and authoritative state into local presentation. Games declare **capabilities**; calibration records controls and motion; difficult mappings may be ranked by a resolver and confirmed by a person; and confirmed decisions are stored in pinned **translation profiles**. Signet Forge is proposed as a separate tool for creating and improving those profiles.

We argue that this is a promising reduction in integration topology but does not by itself establish semantic interoperability. A shared representation concentrates the hard problem into one semantic surface: independent translators must agree on what intents mean, what archetypes guarantee, which capability combinations are valid, and which differences are merely presentation. Deterministic profile selection is not semantic correctness, capability declaration is not negotiated compatibility, and server authority is not general cheat prevention.

We formalize Signet 2 as a canonical semantic hub, distinguish mapping complexity from validation complexity, define interoperability as a versioned and falsifiable relation over authoritative traces, and derive a concrete experimental program. The central result is that translator construction can scale linearly while exhaustive pairwise evidence remains quadratic:

~~~text
M_hub(n) = O(n)
V_all_pairs(n) = O(n^2)
~~~

The architecture therefore removes the need to *write* every pairwise translator without removing the need to *validate* the compatibility claims that matter. We propose a small semantic core, explicit capability negotiation, dependency-bound profiles, golden replay fixtures, an independent translator implementation, heterogeneous pairwise testing, and clean third-party reproduction before broad vocabulary growth or AI integration.

---

## 1. Introduction

Signet 2 attacks a real scaling problem. A direct game-to-game integration graph grows rapidly because every new game creates additional pairwise translation work. The proposal replaces that graph with a shared semantic hub: games translate into and out of shared meaning rather than directly into each other [1].

This architectural move has established precedent. NIST's Smart Grid interoperability framework describes bilateral transformations growing on the order of n² and a canonical data model reducing the mapping burden toward n+1 while still requiring semantic harmonization [2].

The important distinction is that **mapping topology** and **semantic evidence topology** are different problems.

A label such as “fire” may mean trigger requested, weapon activation accepted, shot instantiated, or damage applied. A label such as “weapon.ranged” may identify a broad functional class while leaving range, cooldown, ammunition, projectile behavior, and hit resolution to other layers. Two translators can therefore exchange structurally valid shared labels and still disagree about observable behavior.

The question for a response paper is not whether a shared semantic layer is a good idea. It is:

> What evidence is sufficient to show that independent implementations actually share the same meaning?

This paper proposes an answer.

### 1.1 Contributions

We contribute:

1. a formal model of intents, archetypes, capabilities, translation profiles, negotiation, and authoritative simulation;
2. a distinction between linear translator growth and potentially quadratic pairwise validation demand;
3. falsifiable research questions for semantic consistency, negotiation, profile freshness, calibration, resolver value, and reproduction;
4. a minimum experiment centered on a small semantic profile and a genuinely independent adapter;
5. an evidence ladder separating conformance, independent implementation, pairwise interoperability, reproduction, and freshness;
6. a machine-readable compatibility record that preserves scope, versions, skips, fallbacks, failures, and raw evidence.

### 1.2 Scope

The Signet 2 document is explicitly a draft proposal and states that it is not implemented [1]. We therefore treat its architectural statements as hypotheses or design requirements, not as failures already observed.

We do not attempt to define the final intent vocabulary or archetype catalog. We define how claims about those semantics can be tested.

---

## 2. Source architecture

The source proposal defines a neutral authoritative server with translation at the game boundary [1].

The relevant concepts are:

- **Intent:** what the player wants to do, independent of the local key or control.
- **Intent vocabulary:** the shared list of recognized intents.
- **Archetype:** something in the world described by function rather than graphics.
- **Archetype catalog:** the shared list of archetypes.
- **Appearance palette:** the local items a game can use to represent archetypes.
- **Capabilities:** what a game says it can emit or show.
- **Motion profile:** measured movement characteristics produced by calibration.
- **Resolver:** a table, person, or model that ranks closed candidates.
- **Translation profile:** pinned decisions intended to make runtime translation deterministic.
- **Signet Forge:** the separate companion tool for constructing and improving profiles.

The server receives intents, applies common rules and physics, and returns authoritative state and archetypes.

The architecture can therefore be decomposed as:

~~~text
local control
-> intent
-> authoritative simulation
-> state / archetype
-> local presentation
~~~

Each arrow is a separate semantic boundary and should be testable.

---

## 3. Formal model

### 3.1 Semantic profile

Let the set of participating games be:

~~~text
G = {g1, g2, ..., gn}
~~~

Let a versioned Signet semantic profile be:

~~~text
P = (I, A, R, V)
~~~

where:

- I is the intent vocabulary;
- A is the archetype catalog;
- R is the authoritative semantic/rules contract for the profile;
- V identifies the profile and semantic dependency versions.

Versioning is essential. A shared label whose meaning can change without a version change cannot support reproducible evidence.

### 3.2 Translators

For each game g, define:

~~~text
T_g = (E_g, D_g)
~~~

with:

~~~text
E_g : local_input_context -> shared_intent
D_g : authoritative_state_and_archetype -> local_representation
~~~

E_g is the input-side semantic encoder. D_g is the output-side semantic decoder/presenter.

The functions may reject or approximate mappings where the profile explicitly permits that behavior. They are not assumed to be one-to-one.

### 3.3 Capabilities and negotiation

For each game g, define a capability offer:

~~~text
K_g = (
  intents_emittable,
  archetypes_representable,
  parameters,
  requirements,
  fallback_policy
)
~~~

The Signet 2 draft says the server uses the common subset. A set intersection is insufficient when capabilities have parameters, mandatory requirements, mutually exclusive alternatives, or fallback policy.

Define:

~~~text
N = NEGOTIATE(P, K_server, K_g1, ..., K_gm)
~~~

N must resolve to either:

~~~text
NEGOTIATED_SESSION_PROFILE
~~~

or:

~~~text
REJECT(reason)
~~~

The selected configuration must be explicit and replayable.

RFC 5939 provides useful conceptual precedent: advertised capabilities, potential configurations, and the actual selected configuration are distinct objects [3].

### 3.4 Authoritative transition system

Let s_t be authoritative server state and U_t the ordered accepted intents for tick t.

Define:

~~~text
delta(s_t, U_t) -> (s_(t+1), E_t)
~~~

where E_t is the authoritative event sequence.

This gives a crucial semantic distinction:

~~~text
local action
!= intent requested
!= intent accepted
!= intent applied
!= authoritative event
!= local presentation
~~~

A test suite that collapses these stages cannot detect important semantic disagreement.

### 3.5 Translation profiles

Let Q_g be a game's translation profile.

For reproducible evidence, Q_g should bind:

~~~text
game identity + game fingerprint
translator source + commit + artifact digest
protocol version
semantic profile version
vocabulary revision
archetype catalog revision
calibration procedure + measurements
resolver identity
pinned decisions
~~~

Define VALID(Q_g, D) as the profile's dependency identity matching current dependency state D.

Then:

~~~text
PINNED(Q_g) != VALID(Q_g, D_future)
~~~

A profile can be deterministic and stale at the same time.

### 3.6 Authoritative equivalence

Two games do not need identical graphics, UI, animation, or assets.

Let TRACE_auth(g, f, N) be the authoritative trace produced under fixture f and negotiated profile N.

For discrete semantics:

~~~text
TRACE_auth(g, f, N) = TRACE_expected(f, N)
~~~

For continuous quantities:

~~~text
TRACE_auth(g, f, N) ~=_epsilon TRACE_expected(f, N)
~~~

where epsilon is predeclared by the semantic profile.

Local presentation may vary only within the profile's allowed presentation relation.

### 3.7 Interoperability relation

For games g_i and g_j under profile P:

~~~text
INTEROP(g_i, g_j, P) = true
~~~

only if:

1. negotiation produces a valid N;
2. both translation profiles are fresh for their dependencies;
3. required conformance assertions pass;
4. pairwise fixtures exercise both participants;
5. authoritative traces satisfy exact/tolerance relations;
6. unsupported semantics follow explicit fallback or rejection rules;
7. skips, fallbacks, and non-equivalences are recorded;
8. implementation and suite identities are preserved.

This is intentionally scoped:

~~~text
INTEROP(g_i, g_j, P)
!= compatibility with every future profile
~~~

and:

~~~text
INTEROP(g_i, g_j, P)
!= proof that g_i interoperates with every other game
~~~

---

## 4. Mapping complexity versus validation complexity

Signet 2's translator-scaling claim can be stated precisely.

With one bespoke bidirectional translator per unordered game pair:

~~~text
M_pair(n) = n(n - 1) / 2
~~~

With one bidirectional translator between each game and the shared hub:

~~~text
M_hub(n) = n
~~~

That is the architectural gain. NIST describes the analogous canonical-model transformation from order n² toward n+1 [2].

However, if the ecosystem claims empirical interoperability evidence for every unordered game pair:

~~~text
V_all_pairs(n) = n(n - 1) / 2
~~~

Therefore:

~~~text
M_hub(n) = O(n)
while
V_all_pairs(n) = O(n^2)
~~~

This is the central distinction of this paper:

> Signet 2 can eliminate quadratic *implementation edges* without eliminating quadratic *evidence edges*.

Strong semantic conformance can reduce the amount of pairwise testing needed for bounded claims, but the inference below is not automatically valid:

~~~text
translator A passes self-conformance
+
translator B passes self-conformance
=
A and B have demonstrated pairwise interoperability
~~~

The IPv6 Ready program is a useful precedent because it maintains distinct conformance and interoperability test plans, with stronger certification requiring both [4].

### 4.1 Pair-specific exception pressure

The hub loses its scaling benefit if translators accumulate peer-specific rules:

~~~text
if peer == doom: ...
if peer == minecraft: ...
if peer == openarena: ...
~~~

Let X_g be the number of pair-specific semantic exceptions in translator g.

A central empirical prediction is:

~~~text
X_g approximately 0
~~~

for behavior covered by the shared profile.

Growth in X_g is evidence that the semantic model or profile boundary is incomplete.

---

## 5. Research questions and falsification criteria

### RQ1 — Does the hub remove peer-specific translation logic?

**H1:** adding a new game requires only game-to-profile and profile-to-game mappings.

**Falsified if:** existing translators need meaningful peer-specific branches or tables.

Measure:

- existing translators modified per new game;
- peer-specific exception count;
- implementation time;
- semantic clarification count.

### RQ2 — Do independent translators share semantics?

**H2:** independently authored translators produce equivalent authoritative traces for the same profile fixtures.

**Falsified if:** the same declared semantics produce authoritative disagreement not explicitly permitted by the profile.

Measure:

- checkpoint agreement;
- event-order agreement;
- first divergence;
- continuous-state error;
- disagreement classification.

### RQ3 — Do capabilities produce deterministic safe negotiation?

**H3:** the same capability offers and server policy produce the same session profile or explicit rejection.

**Falsified if:** unsupported authoritative behavior is silently accepted, peers disagree on the selected surface, or negotiation is nondeterministic.

### RQ4 — Are pinned profiles deterministic and fresh?

**H4a:** identical valid dependency-bound profiles produce identical mapping decisions.

**H4b:** relevant dependency changes invalidate or expire affected evidence.

**Falsified if:** changed semantics remain silently covered by old compatibility evidence.

### RQ5 — Is motion calibration faithful enough?

**H5:** repeated calibrated traces remain within predeclared movement tolerances.

Measure:

- position/velocity/orientation error;
- jump-apex error;
- landing-time error;
- slope/step behavior;
- repeatability.

### RQ6 — Does the resolver beat simple baselines?

The source roadmap proposes hand-written rules, text similarity, and a contrastive ranker.

**H6:** the ranker reduces human effort or improves held-out ranking quality without increasing severe semantic error.

Required metrics:

- top-1 accuracy;
- top-k recall;
- mean reciprocal rank;
- override rate;
- time to confirmed mapping;
- severe-error rate;
- abstention quality.

The candidate set must include:

~~~text
NONE / NO SAFE MAPPING
~~~

because a closed list containing only wrong candidates otherwise forces an error.

### RQ7 — Can a third party reproduce the claim?

**H7:** an outsider can reproduce pairwise evidence using only public specification, artifacts, profiles, fixtures, and instructions.

**Falsified if:** reproduction requires private maintainer state or undocumented intervention.

### 5.1 Claim-to-test matrix

This table is the compact experimental contract for the paper.

| Signet 2 claim / design goal | Research question | Test | Primary metric | Falsification / failure signal |
|---|---|---|---|---|
| One shared translator path replaces pairwise translators | RQ1 / H1 | Add an independent game to the declared profile | Peer-specific exception count; existing translators modified | Existing peer translators require game-specific branches or mappings |
| Shared intents/archetypes carry common meaning | RQ2 / H2 | Independent translators execute identical fixtures | Authoritative checkpoint/event agreement | Unpermitted authoritative divergence |
| Capabilities let heterogeneous games coexist | RQ3 / H3 | Negotiation matrix including incompatible cases | Incorrect acceptance/rejection count | Session accepts semantics that are not mutually supported, or peers disagree on selection |
| Pinned profiles make decisions deterministic | RQ4a / H4a | Replay identical bound profile repeatedly | Mapping-decision equality | Same valid profile produces different mapping decisions |
| Profiles remain trustworthy after updates | RQ4b / H4b | Mutate each declared dependency | Correct invalidation rate | Stale evidence remains current after a relevant dependency change |
| Calibration makes motion faithful | RQ5 / H5 | Repeated timed intent traces | Trajectory error against predeclared tolerance | Error exceeds declared profile tolerance |
| Resolver adds practical value | RQ6 / H6 | Held-out paired benchmark of three resolver classes | Severe-error rate plus human effort/ranking metric | No pre-registered improvement or worse severe-error rate |
| Public evidence is independently usable | RQ7 / H7 | Clean-room reproduction | Reproduction state | Undocumented/private maintainer intervention is required |

A claim can pass one row while failing another. No row inherits success automatically from another.

---

## 6. Minimum experimental program

### Phase 0 — pre-register

Publish before observing results:

- profile identity;
- vocabulary/catalog revision;
- normative assertions;
- fixture definitions;
- tolerances;
- negotiation and fallback rules;
- suite revision;
- metrics;
- acceptance thresholds;
- independence criteria.

### Phase 1 — small semantic core

Implement only:

- 5–8 intents;
- 4–6 archetypes;
- one movement/body model;
- one damage/death lifecycle;
- capability negotiation;
- translation-profile format.

Exclude AI.

The goal is not coverage. It is to determine whether “shared meaning” is precise enough for independent implementation.

### Phase 2 — reference implementation

Use the existing Signet implementation as reference and freeze exact repository, SDK, server, protocol/profile, and suite revisions.

Reference code is useful evidence, but not independent evidence for itself.

### Phase 3 — independent adapter

Recruit an implementer who did not design the reference translator.

Provide only public material available to any implementer.

Log every semantic clarification request. A question that changes intended behavior is a specification finding.

Classify implementations explicitly:

~~~text
REFERENCE_IMPLEMENTATION
SAME_CODEBASE_VARIANT
INDEPENDENT_ADAPTER
INDEPENDENT_PROTOCOL_IMPLEMENTATION
INDEPENDENT_CORE_OR_SERVER
~~~

The first milestone should reach at least INDEPENDENT_ADAPTER.

### Phase 4 — heterogeneous pair

The source paper already uses Doom, Minecraft, and OpenArena as motivating examples.

Prefer a pair with meaningfully different semantics rather than two nearly identical FPS engines. Minecraft versus Doom/OpenArena creates useful pressure on movement, objects, presentation, and capability handling.

### Phase 5 — golden replay corpus

Each fixture records:

~~~text
fixture id
profile revision
initial authoritative state
ordered intents
negotiated session profile
expected checkpoints
expected events
allowed tolerances
allowed presentation differences
forbidden authoritative differences
~~~

Required fixture families:

**Intent semantics**
- requested vs accepted;
- accepted vs applied;
- duplicate;
- delayed;
- out-of-order;
- reconnect/replay;
- impossible rate;
- conflicting input.

**Archetypes**
- valid representation;
- permitted alternative;
- missing representation;
- forbidden substitution;
- version mismatch;
- fallback.

**Lifecycle**
- spawn;
- movement;
- collision;
- attack;
- damage;
- death;
- respawn;
- despawn.

**Capability negotiation**
- complete intersection;
- missing optional;
- missing required;
- fallback permitted;
- fallback forbidden;
- unknown optional extension;
- unknown required extension;
- malformed declaration.

**Freshness**
- changed game fingerprint;
- changed translator;
- changed semantic profile;
- changed vocabulary/catalog;
- changed calibration procedure.

### Phase 6 — expected failures

Construct sessions that should be rejected.

A correct rejection is a successful interoperability result. A session starting is not automatically a success.

### Phase 7 — resolver benchmark

Only after the deterministic path works, compare:

- hand-written rules;
- text similarity;
- contrastive ranking.

Use separate development and held-out cases.

### Phase 8 — clean reproduction

Publish exact artifacts, game fingerprints, profiles, replay fixtures, raw logs, pairwise matrix, and known failures.

Have a third party reproduce without private communication.

### 6.1 Statistical analysis plan

The evaluation mixes **normative conformance questions** with **empirical measurements**. They should not be analyzed as if they were the same kind of evidence.

#### Deterministic conformance and negotiation

For finite normative assertions, the primary report is exact:

~~~text
PASS
FAIL
SKIP
NOT_APPLICABLE
~~~

with the denominator and every skip reason exposed.

No p-value is needed to decide whether a required assertion failed. A single reproducible violation of a MUST-level semantic assertion is evidence against conformance for that tested version/profile.

Report at minimum:

- assertions eligible;
- assertions attempted;
- assertions passed;
- assertions failed;
- assertions skipped;
- exact failure IDs;
- first divergent authoritative state/event.

For pairwise matrices, report exact attempted/eligible coverage as well as pass/fail counts. Where a proportion is summarized, accompany it with a confidence interval only when making an inference beyond the finite tested matrix.

#### Motion and calibration measurements

Motion evaluation contains repeated continuous measurements and therefore does require distributional reporting.

For each fixture and implementation report:

- number of independent runs;
- median and mean error;
- standard deviation where meaningful;
- 95th percentile error;
- maximum error;
- time spent outside the predeclared tolerance;
- first divergence time.

The primary pass/fail rule remains the **predeclared engineering tolerance**, not statistical significance.

When comparing two calibration procedures or implementations, report the paired difference and a 95% confidence interval. Prefer bootstrap intervals when the error distribution is clearly non-normal or sample size is small.

#### Resolver benchmark

The resolver experiment is a paired held-out comparison because each resolver should see the same cases and candidate sets.

Primary outcomes should be pre-registered in this order:

1. **severe semantic error rate**;
2. human time to confirmed mapping;
3. top-1 accuracy;
4. mean reciprocal rank;
5. top-k recall;
6. abstention performance.

A resolver should not be declared better merely because ranking accuracy improves while severe semantic errors increase.

For paired binary outcomes such as top-1 correctness, use a paired test such as McNemar's test only if inferential testing is useful; always report the paired effect size and raw disagreement table.

For continuous paired outcomes such as confirmation time or reciprocal rank, report paired differences with confidence intervals. A paired permutation test or bootstrap is appropriate when parametric assumptions are weak.

If multiple secondary metrics are tested, label them exploratory or apply a predeclared multiplicity correction. The primary decision should depend on a small number of pre-registered outcomes rather than whichever metric becomes favorable.

#### Reproduction

Independent reproduction is categorical evidence, not a significance test.

Report one of:

~~~text
CONFIRMS
PARTIALLY_CONFIRMS
FAILS_TO_REPRODUCE
INVALIDATED_BY_VERSION_DRIFT
TEST_SUITE_DISPUTED
ENVIRONMENT_SPECIFIC
UNKNOWN
~~~

The reproducer's raw artifact should remain linked even after the underlying defect is fixed.

### 6.2 Pre-registration record

Before the independent implementation phase starts, freeze a machine-readable pre-registration containing:

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

The held-out cases need not be public before evaluation, but their immutable commitment/digest should be.

Any post-registration protocol, fixture, threshold, or analysis change must be logged as an amendment rather than silently replacing the original plan.

### 6.3 Stopping rules

To avoid a moving target:

- deterministic conformance testing stops when the pre-registered suite is complete;
- a normative failure is recorded even if immediately fixed;
- independent-implementation clarification continues only through the public clarification mechanism;
- resolver sample size and held-out set are fixed before model comparison;
- failed cases may create a **future** regression suite but do not disappear from the original experiment.

---

## 7. Evidence ladder

A Boolean “compatible” label collapses distinct claims.

### V0 — structural validity

Files/messages parse and required identities resolve.

### V1 — behavioral conformance

One implementation satisfies named normative assertions.

### V2 — independent implementation

Another author/team implements the declared contract independently for the tested layer.

### V3 — pairwise interoperability

Two implementations actually execute the same negotiated profile and satisfy shared fixtures.

### V4 — independent reproduction

An outside party reproduces the claim from public artifacts.

### V5 — operational freshness

The evidence remains current after relevant dependency changes and real use.

Therefore:

~~~text
V1 != V2 != V3 != V4 != V5
~~~

W3C implementation-experience criteria and IETF maturity rules both treat independent implementation and operational experience as evidence beyond specification text [5][6].

---

## 8. Compatibility evidence record

A registry should store bounded claims, not badges.

Minimum shape:

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
    translator_commit: ...
    translator_artifact_digest: ...
    translation_profile_digest: ...
    implementation_class: ...
  - ...

suite:
  revision: ...
  artifact_digest: ...
  fixture_families: [...]

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
  invalidation_triggers: [...]
~~~

Important invariants:

~~~text
SELF_TESTED != INDEPENDENTLY_REPRODUCED
CONFORMANCE != PAIRWISE_INTEROPERABILITY
PAIRWISE_INTEROPERABILITY != ECOSYSTEM_ADOPTION
PINNED_PROFILE != FRESH_PROFILE
SKIPPED != PASSED
~~~

Negative results must remain addressable rather than being overwritten by later successes.

OpenID's interoperability events provide a useful reporting precedent by publishing attempted/possible pairings and pass rates rather than only successful demonstrations [7].

---

## 9. Evaluation of Signet 2's strongest claims

### “Each new game only needs one translator”

**Assessment:** plausible architectural claim.

**Evidence required:** new games do not require peer-specific changes in existing translators.

### “It can automatically play with all the others”

**Assessment:** too broad as an empirical claim.

Recommended wording:

> A new translator becomes eligible to interoperate with implementations that negotiate a compatible semantic profile; compatibility for the claimed surface remains subject to conformance and interoperability evidence.

### “The server uses the common subset”

**Assessment:** insufficiently specified for negotiation.

Capabilities may have requirements, parameters, alternatives, and fallback policy. The selected configuration should be explicit.

### “Once pinned, the result is always the same”

**Assessment:** true only for the selected decision under stable dependencies.

Recommended invariant:

~~~text
PINNED != ETERNALLY_VALID
~~~

### “From here on, the game works with no ambiguity”

**Assessment:** determinism is narrower than semantic correctness.

Recommended wording:

> The selected mapping decisions are deterministic for the bound dependency set; semantic correctness and freshness remain testable.

### “Limited games do not break the match”

**Assessment:** useful goal but not universal rule.

Some mismatches should produce a clear rejection rather than silent approximation.

### “The server is the only authority, so nobody can cheat by modifying their client”

**Assessment:** too broad.

Server authority reduces trust in client-reported state and prevents classes of direct state-forgery attacks. It does not inherently prevent input automation, protocol-valid abuse, information exploitation, capability lies, or other client-side cheating classes.

### “The model cannot go outside the list”

**Assessment:** valuable bounded-output property, not proof of semantic correctness.

If all candidates are wrong, the system needs an explicit abstention/no-safe-mapping result.

---

## 10. Relation to established interoperability practice

The proposed validation model is conservative because established standards programs routinely distinguish specification text, conformance, interoperability, implementation diversity, and operational evidence.

### 10.1 Canonical semantic models

NIST's Smart Grid interoperability framework describes bilateral transformations among n independently modeled systems as an order-n² problem and introduces a canonical data model to reduce mappings toward n+1 [2].

That precedent supports Signet 2's central architecture. It also explicitly retains the need for semantic harmonization. A canonical model changes the number of mappings; it does not make semantic agreement automatic.

### 10.2 Capability negotiation

RFC 5939 distinguishes:

- capabilities;
- potential configurations;
- the actual configuration;
- the negotiation process that selects it [3].

The lesson is conceptual rather than a recommendation to reuse SDP. A Signet client saying “I can jump” is not the same protocol fact as the session saying “jump is part of this negotiated profile under these parameters and fallback rules.”

### 10.3 Conformance versus interoperability

The IPv6 Ready Logo Program explicitly defines itself as both a conformance and interoperability testing program. Its current SRv6 Gold program requires both test plans, whereas its earlier Silver level required conformance only [4].

This is direct precedent for:

~~~text
SELF_CONFORMANCE != PAIRWISE_INTEROPERABILITY
~~~

The program also requires rerunning conformance and interoperability logs when a product version changes the relevant IPv6 stack, which supports the proposed freshness/invalidation model.

### 10.4 Independent implementation experience

The current W3C Process treats implementation experience as evidence that a specification is sufficiently clear and complete. Its considerations explicitly include independent interoperable implementations, implementations by people other than specification authors, public deployment, and reports of implementation difficulty [5].

RFC 6410 similarly links mature Internet Standard status to independent interoperating implementations, deployment, operational experience, and absence of interoperability-breaking defects [6].

These are strong precedents for treating an independent Signet adapter as specification evidence rather than merely community growth.

### 10.5 Pairwise matrices

OpenID's 2025 interoperability events reported both denominator and success: 153 of 224 possible OpenID4VP pairings were attempted with more than 90% passing, and 47 OpenID4VCI pairs were tested with 87% passing [7].

This is a useful reporting model because a pass percentage without attempted/eligible coverage can conceal a narrowly tested subset.

### 10.6 Living semantic profiles

SpaceFOM exists because shared simulation middleware did not by itself ensure common domain semantics. Current SpaceFOM V2 work continues to emphasize common semantics and evolution from operational, multi-organization implementation experience [8].

This supports a small-core strategy for Signet 2:

~~~text
SMALL VERSIONED CORE
-> INDEPENDENT IMPLEMENTATIONS
-> OBSERVED FAILURES
-> SEMANTIC REVISION
-> REVALIDATION
~~~

rather than attempting a complete universal game ontology before use.

---

## 11. Threats to validity

**Shared server.** Independent adapters tested against one server do not establish an independent server implementation.

**Shared SDK.** Different adapters using one SDK may share protocol defects. Count them as independent adapters, not independent protocol implementations.

**Game selection.** Doom, Minecraft, and OpenArena do not span all game genres. Early claims must remain profile-scoped.

**Public-suite overfitting.** Maintain held-out, property-based, adversarial, and third-party cases.

**Subjective presentation mappings.** Separate authoritative semantic correctness from local aesthetic preference.

**Calibration observability.** If a game cannot expose reliable motion/state measurements, mark the claim as unverifiable rather than inferring precision.

**Version drift.** Bind evidence to game/translator/profile fingerprints and expire it proactively.

**Maintainer influence.** An “independent” implementer who receives private semantic guidance no longer provides strong specification evidence. Log clarification questions publicly.

**Security scope.** Interoperability tests do not prove general anti-cheat or security properties.

---

## 12. Recommended development order

The source roadmap proposes vocabulary/catalog publication, additive capabilities, profiles/calibration, an end-to-end prototype, a 30-case model experiment, then Forge.

We recommend inserting validation before vocabulary expansion and AI evaluation:

1. publish a **small** versioned semantic core;
2. attach normative assertions to each core semantic item;
3. define capability negotiation and explicit rejection;
4. bind profiles to dependencies and invalidation triggers;
5. implement the non-AI reference path;
6. publish golden replay fixtures;
7. recruit an independent adapter author;
8. run heterogeneous pairwise tests;
9. publish raw evidence and the pairwise matrix;
10. run clean third-party reproduction;
11. revise semantics from observed failures;
12. only then evaluate the resolver against simpler baselines;
13. expand vocabulary from demonstrated use cases or unresolved semantic gaps.

---

## 13. Discussion

Signet 2 makes the correct architectural move by introducing a shared semantic boundary rather than requiring every game to understand every peer.

That shared boundary is exactly where the protocol must become most precise.

The architecture should resist five equivalences:

~~~text
SHARED LABELS          != SHARED SEMANTICS
PINNED CHOICE          != CORRECT CHOICE
SERVER AUTHORITY       != CHEAT IMPOSSIBILITY
ONE TRANSLATOR         != UNIVERSAL COMPATIBILITY
LINEAR MAPPING GROWTH  != LINEAR VALIDATION DEMAND
~~~

The last distinction is especially important. Signet 2 can succeed at its main scaling goal even if the ecosystem still chooses to maintain a large pairwise evidence matrix. The architecture removes quadratic implementation duplication; it does not require pretending pairwise empirical evidence is free.

The most valuable first result is therefore not a large vocabulary.

It is one of two outcomes:

1. two independently developed translators interpret a deliberately small semantic profile equivalently; or
2. they fail, and the failure identifies which semantic contract is underspecified.

Both results improve the standard.

---

## 14. Conclusion

Signet 2's intent/archetype architecture is a plausible way to reduce the engineering cost of cross-game integration. The canonical-hub pattern has precedent, and the proposal contains several strong choices: separation of semantics from presentation, authoritative server simulation, explicit capabilities, deterministic pinned decisions, and AI outside the real-time loop.

These choices create the conditions for interoperability.

They are not yet evidence of interoperability.

We propose that Signet 2 compatibility be treated as a falsifiable claim over a declared semantic profile, negotiated capabilities, exact implementation identities, authoritative traces, test-suite revision, and freshness state.

The target is:

~~~text
PROVEN_INTEROPERABILITY =
  explicit semantics
+ negotiated capability selection
+ independently implemented mappings
+ adversarial fixtures
+ pairwise execution
+ reproducible evidence
+ freshness identity
~~~

If this validation layer is adopted early, the shared semantic hub can become more than a scalable translator architecture. It can become a protocol surface that outsiders can independently implement, reproduce, falsify, and trust.

---

# References

[1] **Signet Protocol.** *Signet 2: intents, archetypes and translation profiles.* Design document v0.1, 2026-10-05. Draft proposal, not implemented at publication.  
https://github.com/kian-cx/signetprotocol

[2] **National Institute of Standards and Technology.** *NIST Framework and Roadmap for Smart Grid Interoperability Standards, Release 2.0.* NIST SP 1108R2, 2012. Section 3.7.3 describes bilateral mapping growth on the order of n² and canonical models reducing mappings toward n+1.  
https://www.nist.gov/system/files/documents/smartgrid/NIST_Framework_Release_2-0_corr.pdf

[3] **IETF.** *RFC 5939 — Session Description Protocol (SDP) Capability Negotiation.* 2010.  
https://www.rfc-editor.org/rfc/rfc5939.html

[4] **IPv6 Ready Logo Committee.** Program documentation and test-plan updates. The program distinguishes conformance and interoperability testing; current SRv6 Gold requires both.  
https://www.ipv6ready.org/news.html  
https://www.ipv6ready.org/faq.html

[5] **World Wide Web Consortium.** *W3C Process Document — Implementation Experience.*  
https://www.w3.org/policies/process/#implementation-experience

[6] **IETF.** *RFC 6410 — Reducing the Standards Track to Two Maturity Levels.* 2011.  
https://www.rfc-editor.org/rfc/rfc6410.html

[7] **OpenID Foundation.** *OpenID4VP and OpenID4VCI Conformance Tests Are Complete and Open for Self-Certification.* 2026, reporting 2025 pairwise interoperability results.  
https://openid.net/openid4vp-and-openid4vci-conformance-tests-are-complete-and-open-for-self-certification/

[8] **Crues, Z.; Möller, B.; Garro, A.; Dexter, D.** *SpaceFOM V2: The Next Generation in Space Simulation Interoperability and How You Can Help.* NASA NTRS 20260008222, SISO SIMposium, 2026.  
https://ntrs.nasa.gov/citations/20260008222

---

## Appendix A — first semantic fixture distinctions

~~~text
button pressed      != intent requested
intent requested    != intent accepted
intent accepted     != intent applied
shot requested      != shot fired
damage requested    != damage applied
death               != despawn
teleport requested  != authoritative transform changed
inventory displayed != authoritative possession
input received      != input applied exactly once
~~~

Any distinction that Signet intentionally collapses should be collapsed explicitly in the normative semantic contract, not by implementation accident.
