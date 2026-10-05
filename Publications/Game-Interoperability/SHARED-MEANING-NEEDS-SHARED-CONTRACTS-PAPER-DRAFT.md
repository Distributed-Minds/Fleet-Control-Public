# Shared Meaning Needs Shared Contracts

## Semantic interoperability, profile composition, and conformance for Signet 2

**Unaffiliated technical response paper — draft 0.2 — 2026-10-05**

### Abstract

Signet 2 proposes replacing pairwise game-to-game translation with a shared semantic layer built around intents, archetypes, capabilities, calibration, and pinned translation profiles. That is the right architectural direction. It changes the scaling problem from maintaining N-squared bespoke bridges to implementing one adapter per game against a common contract.

This paper argues that the remaining problem is not translation quantity but **semantic precision**. A shared label such as `fire` or `weapon.ranged` does not by itself guarantee that independently written adapters attach the same meaning, authority, timing, fallback behavior, or operational effect to that label. We therefore propose a small semantic coordination core beneath Signet 2's gameplay vocabulary, modular domain profiles above it, separate authoritative rulesets, deterministic capability negotiation, evidence-bearing translation mappings, canonical semantic-definition hashing, deterministic profile composition, and cross-implementation conformance tests.

We tested the proposed core conceptually against four non-FPS domains: a turn-based board game, a hidden-information card game, a city-builder/management simulation, and a user-defined tabletop environment. The exercise exposed two FPS-shaped assumptions in our own initial design: fixed simulation ticks are not universal ordering semantics, and server authority does not imply universal visibility of authoritative state. The revised core therefore negotiates ordering models and makes disclosure policy explicit.

We also provide machine-readable JSON Schema candidates and fixed hashing/composition test vectors. The resulting proposal preserves Signet 2's strongest properties—authoritative shared simulation, local presentation freedom, deterministic pinned mappings, and AI outside runtime authority—while making “shared meaning” testable across independent implementations.

**Keywords:** game interoperability; semantic interoperability; distributed simulation; protocol design; capability negotiation; conformance; Signet Protocol

---

# 1. Introduction

Cross-game interoperability is easy to describe and difficult to scale.

If every game translates directly into every other game, the number of bilateral mappings grows rapidly. More importantly, each pair accumulates semantic special cases: a weapon in one game is not mechanically identical to a weapon in another; one game has jumping while another does not; position may refer to feet, camera, collider origin, or model pivot; and the same button press may represent an intent, a prediction, or an authoritative effect.

Signet 2 responds with a stronger model:

> translate each game into shared meaning, and translate shared meaning back into each game.[1]

Its draft architecture introduces:

- an intent vocabulary for what players want to do;
- an archetype catalog for what exists functionally in the shared world;
- per-game appearance palettes;
- capability declaration;
- calibration and motion profiles;
- deterministic translation profiles;
- a closed-choice resolver that may be a table, person, or model;
- one authoritative server for shared rules, physics, damage, and state.[1]

We agree with this direction.

Our disagreement is narrower:

> a shared vocabulary is not yet a shared semantic contract.

If Signet 2 is intended to support independently developed translators, extensions, and eventually very different game genres, the protocol needs a mechanical answer to questions such as:

- What exactly identifies a semantic concept?
- How is a normative definition versioned?
- How do participants prove that they loaded the same definition?
- Which capabilities are merely supported, which are active, and which are required?
- What happens when profiles overlap or conflict?
- When is a fallback semantically safe?
- How are ordering, authority, visibility, and causality represented?
- What does an independent conformance implementation test?

This paper proposes answers while preserving Signet 2's overall architecture.

---

# 2. What Signet 2 already gets right

## 2.1 Game → meaning → game is the right scaling boundary

The core Signet 2 insight is stronger than ordinary crossplay.

Instead of:

~~~text
Game A <-> Game B
Game A <-> Game C
Game B <-> Game C
...
~~~

the system becomes:

~~~text
Game A -> shared meaning -> Game B
Game C -> shared meaning -> Game A
...
~~~

Each adapter targets a shared contract rather than every other supported game.

This resembles successful interoperability architectures in other domains: common federation models in distributed simulation, common extension registries in XR, and common schemas in data exchange.

## 2.2 Intent must remain distinct from effect

Signet 2 sends intent from the participant toward the server, then lets the authoritative simulation decide what happened.[1]

That distinction must remain explicit:

~~~text
fire intent
!=
shot emitted
!=
damage applied
!=
target dead
~~~

Likewise:

~~~text
use intent
!=
door opened

teleport request
!=
authoritative position change
~~~

The client may request an action. It does not own the resulting shared truth.

## 2.3 Local presentation should remain local

A shared ranged weapon may appear as a Doom pistol, a Minecraft bow, or an OpenArena model.

That does not require the shared simulation to redistribute or standardize those assets.

A useful boundary is:

~~~text
shared:
  semantic identity
  authoritative state
  rules
  effects

local:
  models
  textures
  sounds
  HUD
  animation
  camera
  control feel
~~~

This is one of Signet's strongest architectural choices.

## 2.4 Pinned profiles are better than runtime guessing

Signet 2 treats translation profiles like lock files. A person, table, or model may help choose a mapping, but simulation-affecting decisions are saved and reused deterministically.[1]

That is the correct separation:

~~~text
development:
  uncertain
  assisted
  reviewable

runtime:
  explicit
  deterministic
  reproducible
~~~

Even if future models become cheap enough to run every frame, shared authority should not depend on undeclared or changing model state.

---

# 3. Vocabulary is not semantic interoperability

Consider the proposed intent `fire`.

Independent adapters could interpret that label as:

1. the trigger became pressed;
2. the trigger is held;
3. the player requests an attack;
4. the server emitted a projectile;
5. hitscan was resolved;
6. a shot actually consumed ammunition.

These meanings are related but not equivalent.

Similarly:

~~~text
remove entity != death
health == 0 != dead
camera position != body position
message received != command applied
~~~

A protocol can parse identical JSON while participants still disagree conceptually.

The Levels of Conceptual Interoperability Model separates technical and syntactic interoperability from semantic and pragmatic interoperability.[2] Later retrospective work reaches the same general conclusion: common exchange structures do not by themselves guarantee conceptual alignment.[3]

For Signet 2, the implication is simple:

> human-readable names should not be the only identity of shared meaning.

A normative concept needs at least:

~~~text
semantic identity
normative definition
definition version
profile membership
authority/category
conformance properties
~~~

---

# 4. A smaller universal core

Signet 2's first vocabulary is intentionally action-game shaped: move, jump, fire, use, weapons, health pickups, and related concepts.[1]

That is reasonable for its first working examples.

It should not become the permanent universal core.

A city builder may have no player body. A card game may have no continuous position. A turn-based game may have no fixed simulation tick. A tabletop platform may introduce game rules that did not exist when the SDK shipped.

We therefore propose:

~~~text
semantic core
  identity
  session/participant identity
  semantic category
  capability negotiation
  ordering/causality
  authority
  visibility/disclosure
  typed compatibility result
~~~

and modular profiles:

~~~text
profile/spatial-3d@1
profile/entity-lifecycle@1
profile/fps-combat@1
profile/voxel-world@1
profile/turn-taking@1
profile/card-zones@1
profile/economy@1
...
~~~

with separate rulesets:

~~~text
ruleset/doom-deathmatch@1
ruleset/capture-the-flag@1
ruleset/example-card-game@1
...
~~~

A **profile** defines meaning and representation.

A **ruleset** defines authoritative behavior.

That separation prevents a semantic vocabulary from becoming a disguised implementation of one game genre.

---

# 5. Falsifying the core outside FPS games

We applied the proposed core to four deliberately different domain shapes.

This was a design falsification exercise, not an implementation benchmark.

## 5.1 Turn-based board/tactics

A chess-like or tactics-like session needs:

- participants;
- move intents;
- authoritative move validation;
- board state;
- move-applied events;
- turn/phase/action ordering.

It does not need:

- health;
- weapons;
- velocity;
- continuous pose;
- fixed physics ticks.

The intent/state/event distinction remains useful.

However, our initial examples were too tick-shaped.

A turn-based session naturally orders actions as:

~~~yaml
ordering:
  model: turn-sequence@1
  value:
    turn: 42
    phase: action
    action_index: 1
~~~

while current Signet gameplay may use:

~~~yaml
ordering:
  model: fixed-tick@1
  value:
    tick: 18442
    substep: 3
~~~

Therefore the core should negotiate an **ordering model**, not require one universal clock.

## 5.2 Hidden-information card game

A card game adds another semantic dimension:

~~~text
the server may authoritatively know a fact
without being permitted to reveal it to every participant
~~~

For example:

- the server knows both hands;
- each player sees their own hand;
- the opponent does not;
- spectator policy may differ.

This exposed a missing distinction in our first draft:

~~~text
authority:
  who may assert/own this fact?

visibility:
  who may observe this fact?
~~~

These are separate questions.

The core should therefore support a reference to a negotiated visibility/disclosure policy, while concrete concepts such as hand, fog-of-war, or secret objectives remain domain-profile semantics.

## 5.3 City builder / management simulation

A city-builder keeps realtime or tick-based simulation while removing FPS-body assumptions.

The participant may publish:

~~~text
build
zone
delete
change policy
~~~

while authoritative rules own:

~~~text
costs
legality
population
economy
construction
simulation evolution
~~~

This validates scope-based authority without one player-body model.

## 5.4 User-defined tabletop environment

A game-within-game environment may introduce new rules and concepts at runtime or through extensions.

This makes a fixed global catalog especially fragile.

The core must be capable of negotiating namespaced semantic definitions that were not built into the base SDK.

The result of the falsification pass was therefore:

- remove genre semantics from core;
- negotiate ordering rather than assuming ticks;
- separate visibility from authority;
- permit modular namespaced profiles/extensions.

---

# 6. Capability negotiation should not be lowest-common-denominator gameplay

Signet 2 proposes declaring capabilities and using the common subset.[1]

That is a useful starting point, but the mathematical intersection of all participants' capabilities is not always the intended game.

Suppose a ruleset requires jumping:

- adapter A supports it natively;
- adapter B cannot express it;
- adapter C has a bounded deterministic fallback;
- adapter D can observe but not act.

Possible outcomes include:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

There is no universal rule saying “remove jump from the session.”

Doing so may silently change the ruleset for everyone.

We propose separating:

1. **implemented** — the adapter understands the semantic definition;
2. **available** — this local installation can perform it;
3. **activated** — this session selected it;
4. **required** — participation depends on it;
5. **optional** — it may be disabled safely;
6. **fallback** — a declared degraded mapping is acceptable.

OpenXR uses a similar support-versus-enable distinction for extensions.[4] glTF distinguishes extensions that are used from extensions required for correct interpretation.[5]

The shared lesson is:

~~~text
available != active != required
~~~

---

# 7. Translation profiles need semantic provenance

Signet 2's lock-file record is valuable because it makes a choice deterministic.[1]

But deterministic does not mean correct.

A pinned mapping should eventually answer:

- Which source-game version was mapped?
- Which target semantic definition was intended?
- Is the relation exact, close, narrowing, broadening, transform, or approximation?
- What information is lost?
- Which assumptions were made?
- Which tests passed?
- When does the mapping become stale?

Candidate shape:

~~~yaml
source:
  system: minecraft-java
  version_range: "1.21.x"
  concept: bow

target:
  semantic_id: https://signetprotocol.io/sem/presentation/ranged-weapon/1
  definition_hash: sha256:...

relation:
  type: close
  lossy: true

assumptions:
  - local bow is presentation only
  - authoritative firing mechanics come from the session ruleset

evidence:
  conformance:
    - PRESENTATION-RANGED-001
~~~

SKOS provides useful precedent for distinguishing exact, close, broader, and narrower mappings.[6] Gameplay interoperability also needs operational transformations, loss declarations, and conformance evidence.

Signet Forge can remain the tool that proposes, reviews, and pins mappings.

Its mature workflow becomes:

~~~text
discover
classify
review
pin
validate
publish evidence
~~~

---

# 8. Stable semantic identity

Human labels should be localizable.

The semantic identity underneath them should not depend on whether the wire vocabulary is Spanish or English.

A draft identifier scheme could use project-controlled URIs:

~~~text
https://signetprotocol.io/sem/core/session/1
https://signetprotocol.io/sem/profile/fps-combat/1
https://signetprotocol.io/sem/event/damage-applied/1
~~~

Earlier research examples used `urn:signet:...` illustratively. That should not become normative accidentally: RFC 8141 defines URN namespaces as registered namespaces, and there is no current registered Signet URN namespace.[7]

The exact URI shape is negotiable.

The important property is stable, globally scoped machine identity.

---

# 9. Canonical semantic-definition hashing

Semantic identity alone does not detect a dangerous failure:

~~~text
adapter A says profile X version 1
adapter B says profile X version 1
but they loaded different normative definitions
~~~

We therefore propose pairing each semantic ID with a definition digest.

## 9.1 Normative versus annotative content

A semantic-definition document should separate:

~~~text
normative:
  machine meaning
  dependencies
  exported concepts
  constraints

annotations:
  labels
  translations
  examples
  explanatory prose
~~~

Only the normative object is hashed.

This allows documentation and localization improvements without changing the semantic definition.

## 9.2 Canonicalization

We propose:

~~~text
normative JSON
    ->
schema-declared set arrays already sorted + unique
    ->
RFC 8785 JSON Canonicalization Scheme
    ->
UTF-8 canonical bytes
    ->
SHA-256
~~~

RFC 8785 defines deterministic JSON serialization for cryptographic operations.[11]

One important qualification is that JCS preserves array order.

Therefore every normative array must be classified as either:

- ordered sequence; or
- semantic set.

Set-valued arrays such as dependency lists or exported semantic references must be source-canonical—sorted and unique—before JCS.

## 9.3 Published test vector

The accompanying L1 fixture defines one schema-valid semantic definition with expected hash:

~~~text
sha256:2ba8cba596be0c3cc843d9f35441a93513df8179952d15e1539332c50fac6989
~~~

Independent implementations should reproduce the same canonical bytes and digest.

If two participants advertise:

~~~text
same semantic ID
different definition hash
~~~

negotiation fails with a typed semantic-definition conflict.

---

# 10. Deterministic profile composition

Profiles are useful only if independently authored modules can be composed predictably.

HLA Evolved's modular Federation Object Model work provides mature precedent here. Its composition model includes unioning distinct definitions, requiring duplicate identifiers to be equivalent, extending structures without mutating an already-defined parent, and atomically failing selected module loads on hard conflicts.[12]

We do not propose importing HLA's RTI or XML object model.

We propose retaining the composition principles.

## 10.1 Candidate rules

~~~text
same profile ID + same definition hash
  -> coalesce

same profile ID + different hash
  -> reject

same exported concept ID + same definition hash
  -> coalesce

same exported concept ID + different hash
  -> reject

required dependency missing
  -> reject

required dependency cycle
  -> reject

need to extend an imported concept
  -> define a new semantic ID and explicit relation

profile ordering constraints
  -> intersect

profile visibility constraints
  -> intersect

empty required intersection
  -> reject

hard conflict
  -> do not partially activate the remaining required set
~~~

This blocks semantic “monkey-patching,” where one module silently changes the meaning of a concept defined by another.

## 10.2 Composition and contract hashes

After deterministic composition, the activated profile-set record can itself be canonically hashed.

L1 publishes a fixed composition fixture with expected profile-set hash:

~~~text
sha256:a9ca960cc1eec57fb925e44c3cf720ad7bd79ccd69828ae11cbf7d03ba7624b0
~~~

A complete negotiated session can similarly produce a contract hash over exact:

- profiles;
- ruleset;
- ordering model;
- visibility policy;
- accepted fallbacks;
- authority assignments.

That gives logs and replays a precise statement:

> these participants were running the same semantic contract.

---

# 11. Conformance must test meaning

A schema can validate structure.

It cannot prove semantic agreement.

We therefore distinguish:

~~~text
schema-valid
!=
semantically conformant
~~~

Useful semantic ambiguity fixtures include:

~~~text
fire button held / shot emitted
remove entity / death
health zero / dead
use pressed / interaction completed
camera position / body position
message received / command applied
~~~

Candidate conformance families are:

### ID — semantic identity

- localized labels do not change identity;
- same ID + different definition fails closed.

### NEG — negotiation

- supported but inactive capability stays inactive;
- unknown required capability rejects;
- optional features disable explicitly;
- observer-only participation is represented explicitly.

### MAP — mapping

- exact mappings preserve meaning;
- lossy mappings declare loss;
- approximate mappings cannot silently claim full support.

### AUTH — authority

- participant intent does not become authoritative effect;
- clients cannot self-assign server authority.

### ORDER — ordering and causality

- fixed-tick and turn-sequence participants use the same core;
- replay semantics do not depend on transport framing;
- duplicate event identities do not duplicate exactly-once effects.

### VIS — disclosure

- authoritative state is not automatically visible to everyone;
- hidden state follows negotiated policy.

### HASH — canonical definition identity

- property order and whitespace do not change hashes;
- normative changes do;
- independent implementations produce identical JCS bytes.

### COMP — profile composition

- selection order does not affect output;
- exact duplicates coalesce;
- divergent duplicates reject;
- dependency closure is deterministic;
- composition failures are atomic.

---

# 12. Machine-readable candidate

The L1 response now includes machine-readable research artifacts:

~~~text
signet-semantic-core-0.schema.json
signet-semantic-definition-0.schema.json

examples:
  turn-based-capabilities.json
  card-session-requirements.json
  card-negotiated-contract.json

test vectors:
  hash-vector-001.json
  composition-vector-001.json
~~~

They use JSON Schema Draft 2020-12 for structural validation.[10]

These are not proposed as finished Signet specifications.

Their purpose is falsifiability.

We have now also implemented the current hashing/composition subset independently in **Python 3.13.5** and **Node.js 22.16.0**.

Both reference implementations consume the same published vectors and independently produce:

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: 7 negative composition vectors
~~~

The positive composition vector does not merely hash a preconstructed expected result: it begins with the selected `card-zones@1` profile, resolves the required `turn-taking@1` dependency, unions the concept exports, checks the selected ordering model, canonicalizes the result, and derives:

~~~text
sha256:a9ca960cc1eec57fb925e44c3cf720ad7bd79ccd69828ae11cbf7d03ba7624b0
~~~

The seven negative vectors require exact agreement on:

- semantic definition conflict;
- required dependency cycle;
- concept definition conflict;
- ordering-model conflict;
- explicit profile conflict;
- visibility-policy conflict;
- unavailable required dependency.

Both implementations return the same required reason codes.

This remains deliberately narrower than a general RFC 8785 library: floating-point normative values are rejected. The current semantic fixtures use strings, booleans, null, arrays/objects, and safe integers. That restriction prevents an incomplete numeric serializer from being mistaken for full JCS conformance.

The executable evidence changes the status of the proposal:

~~~text
prose contract
    ->
machine-readable schemas
    ->
fixed vectors
    ->
two independent implementations
    ->
same bytes, hashes, closures and failures
~~~

A third implementation can now challenge the contract without needing private clarification.

---

# 13. Migration from Signet/1

This proposal does not require a rewrite-first strategy.

Signet/1 already contains valuable primitives:

- intent-based control;
- authoritative state;
- numbered commands;
- prediction/reconciliation;
- translator boundaries;
- additive protocol evolution.

A practical migration is:

## Stage 1 — semantic metadata beside Signet/1

Add optional capability/profile metadata.

Treat current gameplay semantics as an explicit legacy/action profile rather than the universal future vocabulary.

## Stage 2 — negotiate before authoritative admission

Derive a typed compatibility result before creating an authoritative participant body.

Parser compatibility remains separate from semantic compatibility.

## Stage 3 — publish machine-readable semantic definitions

Start with the vocabulary already exercised by Doom/OpenArena/Minecraft.

Do not try to model all games at once.

## Stage 4 — extend translation profiles with provenance

Keep Forge's basic workflow while binding each pin to an exact target semantic definition.

## Stage 5 — require independent implementations

Before stabilizing the semantic core, require at least:

- one current Signet/FPS implementation;
- one turn/card implementation;
- separate codebases;
- the same canonical test vectors and negotiation results.

---

# 14. Direct answers to Signet 2's open questions

Signet 2 ends with several explicit design questions.[1]

## What is the minimum motion profile?

There should be no universal motion-profile schema.

Required calibration depends on the activated profile and ruleset.

If the server owns authoritative movement constants, local measured walk speed or jump height is compatibility/transduction evidence, not alternate physics authority.

## What if the game does not expose position?

Use legitimate observable interfaces such as:

- official plugins/mod APIs;
- controlled server APIs;
- open-source engine state;
- exposed replay/telemetry;
- sufficiently accurate permitted external observation.

If the necessary quantity cannot be observed reliably:

~~~text
UNKNOWN
UNSUPPORTED
OBSERVE_ONLY
~~~

is preferable to fabricated precision.

## Who maintains the vocabulary/catalog?

Keep the core conservative.

Allow independently namespaced experimental profiles.

Stabilize profiles after multiple implementations and conformance evidence rather than making one ever-growing global vocabulary.

## Where does the model run?

The protocol should not care.

Runtime consumes a pinned mapping.

Forge may use a local model, development machine, or shared service.

## How are conflicting profiles resolved?

Do not use “latest wins.”

Bind sessions to exact semantic IDs/hashes and use deterministic composition rules.

Competing profiles may coexist until evidence and governance converge.

---

# 15. Discussion

The proposal intentionally adds machinery.

The question is whether that machinery solves a real problem or merely makes a small protocol abstract.

We think the line is defensible because each additional core mechanism corresponds to a failure that appears as soon as Signet grows beyond one tightly coordinated implementation team:

| Mechanism | Failure prevented |
|---|---|
| semantic ID | name collision / localization dependence |
| definition hash | same version, different definition |
| required vs optional capability | silent lowest-common-denominator gameplay |
| ordering model | assuming all games are fixed-tick |
| visibility policy | leaking hidden authoritative state |
| mapping provenance | pinned-but-wrong/stale translation |
| composition rules | extensions silently redefining each other |
| conformance vectors | implementations agreeing only in prose |

The proposal also deliberately refuses to standardize many things in the core:

- weapons;
- health;
- transforms;
- board cells;
- cards;
- physics;
- economy;
- RNG algorithm;
- rendering;
- package distribution;
- model APIs.

Those remain profiles, rulesets, or tooling concerns.

The goal is therefore not a giant ontology.

It is a small protocol for safely composing ontologies and domain contracts.

---

# 16. Conclusion

Signet 2 has selected the right architectural direction.

Its strongest ideas should remain intact:

- translate game → meaning → game;
- keep player intent separate from authoritative effect;
- keep shared simulation authoritative;
- keep presentation local;
- declare capabilities;
- calibrate where useful;
- pin simulation-affecting mappings;
- keep AI outside runtime authority.

The remaining interoperability problem is to make “shared meaning” precise enough for independent implementations.

We recommend that Signet 2 add a small semantic coordination layer that makes meaning:

~~~text
identified
versioned
hashed
negotiated
required
degraded
composed
authorized
ordered
disclosed
tested
~~~

Gameplay vocabulary should remain modular profiles. Mechanics should remain rulesets. Translation profiles should record provenance and evidence. Selected semantic modules should compose deterministically. Independent implementations should prove agreement through canonical vectors and conformance tests.

The resulting architecture is still recognizably Signet 2.

It simply upgrades the central promise from:

> we use the same words

to:

> we can prove which shared meanings, definitions, fallbacks, and rules we are running.

That is the difference between a common vocabulary and an interoperability standard.

---

# References

[1] **Signet Protocol.** *Signet 2: intents, archetypes and translation profiles.* Draft proposal v0.1, 5 October 2026. Commit `490dfa9423841a45f2917d8d013e010ca0eb5548`.  
https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/content/proposals/translation-profiles.mdx

[2] **Tolk, Andreas; Muguira, James A.** “The Levels of Conceptual Interoperability Model.” 2003 Fall Simulation Interoperability Workshop.

[3] **Tolk, Andreas.** “Conceptual alignment for simulation interoperability: lessons learned from 30 years of interoperability research.” *SIMULATION* 100(7), 2024. DOI: 10.1177/00375497231216471.

[4] **Khronos Group.** OpenXR 1.1 Specification and Extension Process.  
https://registry.khronos.org/OpenXR/specs/1.1/html/xrspec.html  
https://registry.khronos.org/OpenXR/specs/1.1/extprocess.html

[5] **Khronos Group.** glTF 2.0 Specification.  
https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html

[6] **W3C.** SKOS Simple Knowledge Organization System Reference.  
https://www.w3.org/TR/skos-reference/#mapping

[7] **Saint-Andre, Peter; Klensin, John.** RFC 8141: Uniform Resource Names (URNs). IETF, 2017.  
https://www.rfc-editor.org/rfc/rfc8141.html

[8] **IEEE.** IEEE 1516.2-2025, *High Level Architecture (HLA)—Object Model Template (OMT) Specification*.  
https://standards.ieee.org/ieee/1516.2/6689/

[9] **Simulation Interoperability Standards Organization.** HLA-related data files and modular FOM examples.  
https://www.sisostandards.org/page/DataFiles

[10] **JSON Schema.** Draft 2020-12 Specification.  
https://json-schema.org/specification  
https://json-schema.org/draft/2020-12/schema

[11] **Rundgren, Anders; Jordan, Bret; Erdtman, Samuel.** RFC 8785: JSON Canonicalization Scheme (JCS). 2020.  
https://www.rfc-editor.org/rfc/rfc8785.html

[12] **Möller, Björn; Löfstrand, Björn; Karlsson, Mikael.** “An Overview of the HLA Evolved Modular FOMs.” 2007 Spring Simulation Interoperability Workshop.  
https://pitchtechnologies.com/wp-content/uploads/2020/09/07s-siw-108-1.pdf

---

## Technical appendices

Detailed derivation, schemas, test vectors, and conformance properties live in:

~~~text
Research/Game-Interoperability/Signet-Frontier-2026-10/lanes/L1/
  01-Semantic-Contract-Draft.md
  02-Negotiation-Mapping-and-Conformance.md
  04-Signet-2-Response-Analysis.md
  05-Non-FPS-Core-Falsification.md
  06-Machine-Readable-Core-Candidate.md
  07-Canonical-Hashing-and-Profile-Composition.md
  schema/
~~~

The publication paper intentionally keeps implementation-detail depth in those appendices rather than duplicating every schema rule here.
