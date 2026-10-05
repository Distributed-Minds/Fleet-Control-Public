# From Shared Meaning to Verifiable Interoperability

## An experimental validation framework for Signet 2

**Working paper · Draft 0.3 · 2026-10-05**  
**Research lane:** L4 — Validation, Ecosystem & Adoption  
**Responds to:** *Signet 2: intents, archetypes and translation profiles*, design document v0.1, 2026-10-05.

Detailed fixture inventories, schemas, preregistration fields, and statistical procedures are moved to:

- `Signet-2-Interoperability-Validation-Supplement.md`

---

## Abstract

Signet 2 proposes a shared semantic architecture for cross-game interoperability. Instead of translating every game directly into every other game, each game maps local actions into a common vocabulary of **intents** and maps shared **archetypes** and authoritative state into local presentation. Games declare **capabilities**; calibration records controls and motion; difficult mappings may be ranked by a resolver and confirmed by a person; and confirmed decisions are stored in pinned **translation profiles**. Signet Forge is proposed as a separate tool for creating and improving those profiles [1].

We argue that this architecture plausibly reduces integration topology without, by itself, proving semantic interoperability. The shared representation moves the critical assumptions into one common semantic surface: independent translators must agree on what intents mean, what archetypes guarantee, which capability combinations are valid, and which differences are presentation-only. Deterministic profile selection is not semantic correctness, capability declaration is not negotiated compatibility, and server authority is not general cheat prevention.

We formalize Signet 2 as a canonical semantic hub and distinguish **mapping complexity** from **validation complexity**. For n games, hub translation can require O(n) game-to-hub mappings while exhaustive empirical pairwise evidence can remain O(n²). This does not weaken the architecture; it identifies the evidence program needed to support its strongest claim.

We propose a minimum validation program: a small versioned semantic core, explicit capability negotiation, dependency-bound translation profiles, golden replay fixtures, one genuinely independent adapter implementation, heterogeneous pairwise testing, and clean third-party reproduction before vocabulary expansion or AI integration. Compatibility is treated as a falsifiable claim over a declared profile rather than as a global Boolean property.

---

## 1. Introduction

Signet 2 addresses a real scalability problem. A direct game-to-game integration graph grows rapidly because each additional game creates new bilateral translation work. The proposal replaces that graph with a shared semantic hub: games translate into and out of shared meaning rather than directly into every peer [1].

This is a well-established interoperability pattern. NIST's Smart Grid interoperability framework describes bilateral transformations among independently modeled systems as an order-n² problem and introduces canonical data models to reduce the mapping burden toward n+1 while still requiring semantic harmonization [2].

The key distinction is therefore not whether canonicalization is useful. It is.

The harder question is whether two independently developed translators actually attach the same meaning to the shared representation.

A label such as `fire` may denote:

- a trigger request;
- an accepted weapon action;
- a spawned projectile;
- an emitted shot event;
- applied damage.

These are not interchangeable.

Likewise, `weapon.ranged` can identify a useful functional class while leaving unresolved:

- effective range;
- hitscan versus projectile semantics;
- ammunition;
- cooldown;
- damage model;
- equip timing;
- hit resolution.

Signet 2 improves the architecture by making those shared concepts explicit. That makes semantic disagreement easier to inspect. It does not make disagreement impossible.

The central research question of this paper is:

> **What evidence is sufficient to show that independent game translators actually share the same meaning?**

### 1.1 Contributions

This paper contributes:

1. a compact formal model of Signet 2 semantic translation, negotiation, authority, and profile validity;
2. a distinction between linear mapping growth and potentially quadratic validation demand;
3. a scoped interoperability relation based on authoritative behavior rather than shared labels alone;
4. a claim-to-test framework with explicit falsification criteria;
5. a minimum experimental program centered on independent implementation and reproducible pairwise evidence;
6. an evidence ladder separating conformance, interoperability, reproduction, and operational freshness.

### 1.2 Scope

The Signet 2 document is explicitly a **draft proposal** and states that it is not yet implemented [1]. We therefore evaluate architectural claims and proposed mechanisms, not implementation outcomes that do not yet exist.

The paper does not attempt to define the final Signet intent vocabulary or archetype catalog. It defines how claims about such semantics can be tested.

---

## 2. Signet 2 as a semantic hub

The source architecture separates local game mechanics from shared semantics.

A player action is converted into an **intent**. The authoritative server applies rules and physics. The server then returns state and **archetypes**. Each game chooses how to present those archetypes locally [1].

The major concepts are:

- **Intent** — what the player wants to do;
- **Archetype** — what exists in the shared world by function rather than appearance;
- **Appearance palette** — local game objects capable of representing archetypes;
- **Capabilities** — what a game says it can emit or represent;
- **Motion profile** — calibrated movement measurements;
- **Resolver** — a table, person, or model that ranks mappings;
- **Translation profile** — pinned mapping decisions;
- **Signet Forge** — a companion tool for constructing and improving those profiles.

### Figure 1. Architecture and validation boundaries

~~~mermaid
flowchart LR
  A["Game A<br/>local controls & presentation"] --> EA["Translator A<br/>input semantics"]
  B["Game B<br/>local controls & presentation"] --> EB["Translator B<br/>input semantics"]

  EA -->|"intents"| N["Capability negotiation<br/>selected session profile"]
  EB -->|"intents"| N

  N --> S["Authoritative Signet server<br/>rules + physics"]
  S -->|"state + archetypes"| DA["Translator A<br/>output semantics"]
  S -->|"state + archetypes"| DB["Translator B<br/>output semantics"]

  DA --> A
  DB --> B

  T["Conformance + replay fixtures"] -.-> EA
  T -.-> EB
  T -.-> N
  T -.-> S
  T -.-> DA
  T -.-> DB

  R["Evidence registry<br/>versions · results · reproduction"] -.-> T
~~~

The validation point is not one component. Every semantic boundary is testable:

~~~text
local action
!= intent requested
!= intent accepted
!= intent applied
!= authoritative event
!= local presentation
~~~

This distinction is central because interoperability can fail even when all messages parse correctly.

---

## 3. Formal model

Let:

~~~text
G = {g1, g2, ..., gn}
~~~

be the participating games.

Let a versioned semantic profile be:

~~~text
P = (I, A, R, V)
~~~

where:

- I is the intent vocabulary;
- A is the archetype catalog;
- R is the relevant authoritative semantic/rules contract;
- V identifies the profile and semantic dependency versions.

For game g, define the translator:

~~~text
T_g = (E_g, D_g)
~~~

with:

~~~text
E_g : local_input_context -> shared_intent
D_g : authoritative_state_and_archetype -> local_representation
~~~

The functions may reject or approximate mappings only where the profile explicitly permits that behavior.

### 3.1 Capability negotiation

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

The Signet 2 proposal says the server uses the common subset [1]. A literal set intersection is insufficient once capabilities contain requirements, parameters, alternatives, or fallback policy.

Define:

~~~text
N = NEGOTIATE(P, K_server, K_g1, ..., K_gm)
~~~

where N resolves to either:

~~~text
NEGOTIATED_SESSION_PROFILE
~~~

or:

~~~text
REJECT(reason)
~~~

The negotiated configuration must be explicit and replayable.

RFC 5939 provides useful conceptual precedent by distinguishing advertised capabilities, potential configurations, and the actual selected configuration [3].

### 3.2 Authoritative transition system

Let s_t be authoritative state and U_t the ordered accepted intents at tick t.

Define:

~~~text
delta(s_t, U_t) -> (s_(t+1), E_t)
~~~

where E_t is the authoritative event sequence.

For a fixture f and negotiated profile N, let:

~~~text
TRACE_auth(g, f, N)
~~~

be the authoritative trace induced by game g through its translator.

For exact discrete semantics:

~~~text
TRACE_auth(g, f, N) = TRACE_expected(f, N)
~~~

For continuous quantities such as motion:

~~~text
TRACE_auth(g, f, N) ~=_epsilon TRACE_expected(f, N)
~~~

where epsilon is a preregistered profile-specific tolerance.

Local presentation may differ without violating interoperability if the difference is explicitly permitted.

### 3.3 Translation-profile validity

A pinned translation profile is useful because it makes selected mappings deterministic.

But determinism and freshness are different properties.

A reproducible profile must bind at least:

~~~text
game identity/fingerprint
translator identity/artifact
protocol/profile revision
vocabulary revision
archetype-catalog revision
calibration identity
resolver identity
pinned decisions
~~~

Let VALID(Q_g, D) mean that profile Q_g still matches dependency state D.

Then:

~~~text
PINNED(Q_g) != VALID(Q_g, D_future)
~~~

A profile can be deterministic and stale simultaneously.

### 3.4 Scoped interoperability relation

For games g_i and g_j under semantic profile P, define:

~~~text
INTEROP(g_i, g_j, P) = true
~~~

only if:

1. negotiation produces a valid session profile;
2. both translation profiles are valid for their current dependencies;
3. required conformance assertions pass;
4. pairwise fixtures exercise both participants;
5. authoritative traces satisfy the profile's equality/tolerance rules;
6. unsupported semantics follow declared fallback or rejection rules;
7. skips, fallbacks, and non-equivalences remain visible;
8. implementation and test-suite identities are recorded.

This is deliberately narrower than “Game X supports Signet.”

---

## 4. Mapping complexity is not validation complexity

The strongest Signet 2 architectural claim is its reduction in translation edges.

With one bespoke bidirectional translator per unordered game pair:

~~~text
M_pair(n) = n(n - 1) / 2
~~~

With one translator between each game and the shared hub:

~~~text
M_hub(n) = n
~~~

Thus:

~~~text
M_hub(n) = O(n)
~~~

rather than O(n²).

NIST describes the analogous canonical-model advantage in another interoperability domain [2].

However, if an ecosystem wants direct empirical evidence for every unordered game pair:

~~~text
V_all_pairs(n) = n(n - 1) / 2 = O(n^2)
~~~

Therefore:

~~~text
LINEAR MAPPING GROWTH != LINEAR VALIDATION DEMAND
~~~

This is not a defect in Signet 2.

It means the architecture removes quadratic **implementation duplication** without automatically removing quadratic **evidence demand**.

Strong conformance can reduce the amount of pairwise testing needed for some bounded claims, but the inference below is not automatically valid:

~~~text
A passes self-conformance
+
B passes self-conformance
=
A and B have demonstrated interoperability
~~~

IPv6 Ready is useful precedent because it explicitly distinguishes conformance and interoperability testing [4].

### 4.1 Pair-specific exceptions as a diagnostic

The hub stops delivering its intended scaling benefit if translators accumulate logic like:

~~~text
if peer == doom: ...
if peer == minecraft: ...
if peer == openarena: ...
~~~

Let X_g be the number of peer-specific semantic exceptions in translator g.

A core architectural prediction is:

~~~text
X_g approximately 0
~~~

for behavior genuinely captured by the shared profile.

Growth in X_g is evidence that the shared semantic model or profile boundary is incomplete.

---

## 5. Table 1 — claims and required evidence

| Signet 2 claim / design goal | Hypothesis | Required experiment | Primary evidence | Falsification / failure signal |
|---|---|---|---|---|
| One shared translator path replaces pairwise translators | **H1** | Add an independent game to the declared profile | Peer-specific exception count; existing translators modified | Existing peer translators require game-specific branches or mappings |
| Shared intents/archetypes carry common meaning | **H2** | Independent translators execute identical fixtures | Authoritative checkpoint and event agreement | Unpermitted authoritative divergence |
| Capabilities let heterogeneous games coexist | **H3** | Negotiation matrix including incompatible cases | Incorrect acceptance/rejection count | Unsupported semantics are accepted or peers disagree on the selected configuration |
| Pinned profiles are deterministic | **H4a** | Replay identical valid bound profiles | Mapping-decision equality | Same valid profile yields different decisions |
| Profiles remain trustworthy after updates | **H4b** | Mutate each declared dependency | Correct invalidation | Stale evidence remains current |
| Calibration makes motion faithful | **H5** | Repeated timed intent traces | Trajectory error versus preregistered tolerance | Error exceeds declared profile tolerance |
| Resolver adds practical value | **H6** | Held-out paired resolver benchmark | Severe-error rate and human effort | No preregistered improvement or severe errors increase |
| Public evidence is independently usable | **H7** | Clean-room reproduction | Reproduction state | Private or undocumented maintainer intervention is required |

No row inherits success from another.

That matters because several Signet 2 statements currently combine different evidence classes.

---

## 6. Evaluation of the strongest Signet 2 claims

### 6.1 “Each new game only needs one translator”

This is the proposal's strongest architectural claim and is plausible.

The empirical question is whether adding a new game leaves existing translators unchanged except for shared-profile evolution.

Evidence against the claim would be persistent peer-specific exceptions.

### 6.2 “It can automatically play with all the others”

This is directionally correct about architecture but too broad as an empirical compatibility statement.

A stronger formulation is:

> A new translator becomes eligible to interoperate with implementations that negotiate a compatible semantic profile; compatibility for the claimed surface remains subject to conformance and interoperability evidence.

The hub removes the need to author one translator per peer. It does not automatically prove every peer combination.

### 6.3 “The server uses the common subset”

A common set of capability labels is not yet a negotiated configuration.

For example, two games may both advertise movement but disagree on:

- dimensionality;
- analog range;
- required jump semantics;
- fallback policy;
- authoritative timing assumptions.

The selected session profile should therefore be an explicit protocol artifact.

### 6.4 “Once pinned, the result is always the same”

A pinned mapping decision can be deterministic.

But:

~~~text
PINNED != ETERNALLY_VALID
~~~

A game update, translator change, vocabulary change, archetype change, or calibration change may invalidate earlier evidence.

### 6.5 “From here on, the game works with no ambiguity”

Pinned decisions remove one class of runtime choice.

They do not prove that the pinned choice is semantically correct.

Recommended formulation:

> The selected mapping decisions are deterministic for the bound dependency set; semantic correctness and freshness remain independently testable.

### 6.6 “Limited games do not break the match”

Graceful degradation is valuable, but some mismatches should reject the session.

A missing optional cosmetic capability may be harmless.

A missing required authoritative action may make the session invalid.

Therefore:

~~~text
GRACEFUL REJECTION
is a valid interoperability outcome
~~~

### 6.7 “The server is the only authority, so nobody can cheat by modifying their client”

Server authority is a strong integrity boundary because it reduces trust in client-reported world state.

It does not inherently prevent:

- input automation;
- protocol-valid timing abuse;
- information exploitation;
- capability misreporting;
- malicious translator behavior;
- other client-side assistance.

The narrower claim is defensible:

> The server is authoritative over simulation state, reducing trust in client-reported state and preventing classes of direct state-forgery cheats.

### 6.8 “The model cannot go outside the list”

Closed-list ranking bounds model output.

It does not guarantee the list contains a correct answer.

The resolver therefore needs:

~~~text
NONE / NO SAFE MAPPING
~~~

as a legitimate result.

A model should be evaluated as a ranking assistant, not as semantic authority.

---

## 7. Minimum experimental program

The quickest credible test of Signet 2 is not a large vocabulary.

It is a small profile designed to fail informatively.

### Phase 0 — preregister

Freeze before observing results:

- semantic-profile revision;
- vocabulary/catalog revision;
- normative assertions;
- fixture definitions;
- tolerances;
- negotiation/fallback rules;
- test-suite revision;
- primary metrics;
- independence criteria.

### Phase 1 — minimal semantic core

Implement only:

- 5–8 intents;
- 4–6 archetypes;
- one movement/body model;
- one damage/death lifecycle;
- capability negotiation;
- translation-profile format.

Exclude AI.

### Phase 2 — reference implementation

Freeze exact server, SDK, protocol/profile, and suite revisions.

A reference implementation is useful but is not independent evidence for itself.

### Phase 3 — independent adapter

Recruit an implementer who did not design the reference translator.

Provide only public material available to any implementer.

Log every semantic clarification request.

A clarification that changes intended behavior is itself specification evidence.

### Phase 4 — heterogeneous pair

Use games with meaningfully different semantics.

The Signet 2 draft already motivates the design using Doom, Minecraft, and OpenArena [1]. A Minecraft-versus-Doom/OpenArena pair is more informative than two nearly identical FPS integrations because it stresses object representation, movement, and capability differences.

### Phase 5 — golden replay

Exercise at least:

- requested/accepted/applied action distinctions;
- duplicate and reordered input;
- attack/damage/death lifecycle;
- valid and invalid archetype substitutions;
- capability mismatch and rejection;
- profile invalidation after dependency changes.

Detailed fixture definitions are in the supplement.

### Phase 6 — deliberate failure cases

Construct sessions that **should not start**.

A correct rejection is evidence of correct negotiation.

This prevents “the match launched” from being treated as synonymous with “interoperability succeeded.”

### Phase 7 — independent reproduction

Publish:

- exact revisions;
- artifacts/digests;
- game fingerprints;
- profiles;
- replay fixtures;
- raw logs;
- pairwise results;
- known failures.

Then ask a third party to reproduce without private maintainer intervention.

### Phase 8 — resolver benchmark

Only after the deterministic non-AI path works should Signet test whether the proposed resolver improves translator construction.

Compare:

- hand-written rules;
- text-similarity baseline;
- contrastive ranking model.

Use held-out cases and make severe semantic error rate a primary outcome.

---

## 8. Evidence ladder

A single “compatible” label hides distinct claims.

We define:

### V0 — structural validity

Messages/profiles parse and required identifiers resolve.

### V1 — behavioral conformance

One implementation satisfies named normative assertions.

### V2 — independent implementation

Another author/team independently implements the tested contract.

### V3 — pairwise interoperability

Two implementations execute the same negotiated profile and satisfy shared authoritative fixtures.

### V4 — independent reproduction

An outside party reproduces the claim from published artifacts.

### V5 — operational freshness

Evidence remains current after relevant dependency changes and actual use.

Thus:

~~~text
V1 != V2 != V3 != V4 != V5
~~~

W3C implementation-experience criteria explicitly consider independent interoperable implementations, implementations by people other than specification authors, deployment, and implementation difficulty [5]. RFC 6410 similarly connects standards maturity with independent interoperating implementations and operational experience [6].

OpenID's interoperability events provide a practical reporting precedent: they publish both attempted/possible pairing counts and pass rates rather than successful demonstrations alone [7].

---

## 9. Relation to established interoperability practice

### 9.1 Canonical semantic models

NIST's Smart Grid framework provides direct precedent for the architectural benefit claimed by Signet 2: canonical models can reduce bilateral transformation growth while still requiring common semantic understanding [2].

This supports Signet 2 rather than contradicting it.

### 9.2 Capability negotiation

RFC 5939 distinguishes capabilities, potential configurations, actual configurations, and the negotiation process [3].

The lesson for Signet is conceptual:

~~~text
"I can do X"
!=
"this session selected X under these parameters"
~~~

### 9.3 Conformance versus interoperability

IPv6 Ready explicitly distinguishes conformance and interoperability testing [4].

That supports:

~~~text
SELF_CONFORMANCE != PAIRWISE_INTEROPERABILITY
~~~

and provides precedent for rerunning evidence when relevant product versions change.

### 9.4 Independent implementation

W3C and IETF both treat independent implementation experience as evidence that a specification is sufficiently clear and mature [5][6].

For Signet 2, an independent adapter is therefore not merely an adoption milestone. It is a specification test.

### 9.5 Pairwise matrices

OpenID's 2025 interoperability events reported attempted pairings as well as pass rates [7].

This avoids the misleading situation where a high success percentage hides low coverage.

### 9.6 Living semantic profiles

SpaceFOM is relevant because shared simulation middleware alone did not guarantee shared domain semantics. Current SpaceFOM V2 work continues to emphasize common semantics and evolution through operational multi-organization implementation experience [8].

That supports:

~~~text
SMALL VERSIONED CORE
-> INDEPENDENT IMPLEMENTATIONS
-> OBSERVED FAILURES
-> SEMANTIC REVISION
-> REVALIDATION
~~~

rather than attempting a complete universal game ontology before real implementations exist.

---

## 10. Threats to validity

**Shared server.** Independent adapters tested against one server do not establish an independent server implementation.

**Shared SDK.** Different adapters using the same SDK may share wire/codec defects. They count as independent adapters, not independent protocol implementations.

**Game selection.** Doom, Minecraft, and OpenArena do not represent every genre. Early conclusions must remain profile-scoped.

**Public-suite overfitting.** Public fixtures can become targets. Held-out and third-party cases are needed.

**Subjective presentation.** Local appearance preference must be separated from authoritative semantic correctness.

**Calibration observability.** If a game cannot expose reliable state, claims requiring that measurement should be marked unverifiable.

**Version drift.** Compatibility evidence must expire or be invalidated when relevant dependencies change.

**Maintainer influence.** An “independent” implementer receiving extensive private semantic guidance is weaker specification evidence.

**Security scope.** Interoperability testing is not a general anti-cheat or security proof.

---

## 11. Recommended development order

We recommend modifying the Signet 2 roadmap as follows:

1. publish a **small** versioned semantic core;
2. attach normative assertions to every core semantic item;
3. define explicit capability negotiation and rejection;
4. bind translation profiles to dependency identity and invalidation triggers;
5. implement the deterministic non-AI reference path;
6. publish golden replay fixtures;
7. recruit an independent adapter author;
8. run heterogeneous pairwise tests;
9. publish raw evidence and coverage;
10. run clean third-party reproduction;
11. revise semantics from observed failures;
12. only then evaluate the AI resolver;
13. expand vocabulary from demonstrated use cases or unresolved semantic gaps.

The most useful early result is not “all tests green.”

It is either:

1. two independent translators interpret a small profile equivalently; or
2. they disagree in a way that exposes an underspecified semantic contract.

Both outcomes improve the protocol.

---

## 12. Discussion

Signet 2 makes the right architectural move by introducing a shared semantic boundary rather than requiring every game to understand every peer.

That shared boundary is precisely where the standard must become most explicit.

The architecture should resist five equivalences:

~~~text
SHARED LABELS          != SHARED SEMANTICS
PINNED CHOICE          != CORRECT CHOICE
SERVER AUTHORITY       != CHEAT IMPOSSIBILITY
ONE TRANSLATOR         != UNIVERSAL COMPATIBILITY
LINEAR MAPPING GROWTH  != LINEAR VALIDATION DEMAND
~~~

The fifth distinction is the paper's central contribution.

Signet 2 can fully succeed at reducing integration work while an ecosystem still chooses to maintain substantial pairwise evidence.

That is not duplicated engineering. It is validation.

As the semantic profile matures, stronger conformance may justify sampling rather than exhaustive pairwise testing for some bounded claims. But that reduction in evidence burden must itself be earned by precise semantics, independent implementation, and observed agreement.

---

## 13. Conclusion

Signet 2's intent/archetype architecture is a plausible way to reduce the engineering cost of cross-game integration.

Its design contains several strong choices:

- semantics separated from presentation;
- authoritative server simulation;
- explicit capabilities;
- deterministic pinned mapping decisions;
- AI outside the real-time loop.

These choices create the conditions for interoperability.

They are not yet evidence of interoperability.

We propose that Signet 2 compatibility be treated as a falsifiable claim over:

~~~text
declared semantic profile
+ negotiated capabilities
+ exact implementation identities
+ authoritative traces
+ test-suite revision
+ reproducible evidence
+ freshness state
~~~

The practical target is:

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

If Signet adopts that validation layer early, its semantic hub can become more than a scalable translator architecture.

It can become a protocol surface that outsiders can independently implement, reproduce, falsify, and trust.

---

# References

[1] **Signet Protocol.** *Signet 2: intents, archetypes and translation profiles.* Design document v0.1, 2026-10-05. Draft proposal, not implemented at publication.  
https://github.com/kian-cx/signetprotocol

[2] **National Institute of Standards and Technology.** *NIST Framework and Roadmap for Smart Grid Interoperability Standards, Release 2.0.* NIST SP 1108R2, 2012.  
https://www.nist.gov/system/files/documents/smartgrid/NIST_Framework_Release_2-0_corr.pdf

[3] **IETF.** *RFC 5939 — Session Description Protocol (SDP) Capability Negotiation.* 2010.  
https://www.rfc-editor.org/rfc/rfc5939.html

[4] **IPv6 Ready Logo Committee.** Program documentation and test-plan updates.  
https://www.ipv6ready.org/news.html  
https://www.ipv6ready.org/faq.html

[5] **World Wide Web Consortium.** *W3C Process Document — Implementation Experience.*  
https://www.w3.org/policies/process/#implementation-experience

[6] **IETF.** *RFC 6410 — Reducing the Standards Track to Two Maturity Levels.* 2011.  
https://www.rfc-editor.org/rfc/rfc6410.html

[7] **OpenID Foundation.** *OpenID4VP and OpenID4VCI Conformance Tests Are Complete and Open for Self-Certification.* 2026; reports 2025 pairwise interoperability results.  
https://openid.net/openid4vp-and-openid4vci-conformance-tests-are-complete-and-open-for-self-certification/

[8] **Crues, Z.; Möller, B.; Garro, A.; Dexter, D.** *SpaceFOM V2: The Next Generation in Space Simulation Interoperability and How You Can Help.* NASA NTRS 20260008222, SISO SIMposium, 2026.  
https://ntrs.nasa.gov/citations/20260008222
