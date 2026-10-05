# From Shared Meaning to Shared Contracts

## A protocol-semantics response to Signet 2's intents, archetypes, and translation profiles

**Unaffiliated technical response draft — 2026-10-05**

**Response target:** Signet Protocol, *Signet 2: intents, archetypes and translation profiles*, draft proposal v0.1, 2026-10-05.

---

## Abstract

Signet 2 proposes a strong answer to the N-squared integration problem of cross-game interoperability: each game should translate into a shared meaning and back out again, rather than maintaining bespoke game-to-game bridges. It introduces shared intent and archetype vocabularies, capability declarations, calibration, pinned translation profiles, a closed-choice resolver, and a clear separation between the deterministic protocol runtime and an optional AI-assisted development tool.

We agree with that direction.

This paper argues that one additional layer is needed before “shared meaning” can function as a durable interoperability standard: **a versioned, modular, negotiable and testable semantic contract**. A common vocabulary is necessary but does not by itself establish that two independently built adapters attach the same meaning, authority, units, timing assumptions, fallback behavior, or operational effect to the same label.

We propose a deliberately small semantic core for identity, negotiation, authority, ordering and incompatibility; modular profiles for domain vocabularies such as spatial state and FPS combat; separate rulesets for authoritative mechanics; explicit mapping provenance for Signet translation profiles; typed compatibility outcomes instead of an implicit least-common-denominator subset; and semantic/pragmatic conformance tests.

The proposal preserves Signet's most valuable properties: one authoritative simulation, simple and inspectable transport, deterministic pinned runtime mappings, local presentation freedom, and AI outside shared authority. The aim is not to replace Signet 2. It is to make its central phrase — **translate game to meaning, and meaning to game** — precise enough that two teams can implement it independently and prove that they meant the same thing.

---

# 1. Signet 2 is moving in the right direction

The most important claim in Signet 2 is not about AI.

It is this:

> do not translate every game to every other game; translate each game into shared meaning and back out again.

That is the correct scaling move.

For N games, pairwise translators grow toward N(N-1)/2 relationships. A neutral semantic boundary lets each adapter target the shared contract instead.

Signet 2 also gets several architectural boundaries right.

## 1.1 Player intent is not authoritative effect

The player-side translator emits intent.

The server applies shared rules and physics.

The server decides authoritative positions, hits, damage, death and other shared effects.

That distinction is essential.

~~~text
fire button held
    !=
shot emitted
    !=
damage applied
    !=
target dead
~~~

The first is an input/request. The later states are consequences of shared rules and authority.

An interoperability standard that collapses them into one “fire” concept will eventually disagree with itself.

## 1.2 Appearance can remain local

Signet's separation between neutral meaning and local presentation is equally important.

A shared object may appear as a Doom pistol, a Minecraft bow, or an OpenArena weapon model.

The shared simulation does not need to transport those files or force every client to render the same asset.

That is a real interoperability boundary:

~~~text
shared:
  identity
  state
  rules
  authoritative effects

local:
  models
  textures
  sounds
  HUD
  animation
  camera
  control feel
~~~

## 1.3 Pinned translation profiles are better than runtime guessing

Signet 2's lock-file analogy is a good one.

A model, table, maintainer or player may help choose a mapping, but once a simulation-affecting decision is accepted, the runtime should use a deterministic record.

That creates a useful split:

~~~text
proposal time:
  uncertain
  assisted
  human-reviewed

runtime:
  deterministic
  explicit
  reproducible
~~~

We would keep this invariant even if realtime local models eventually become cheap enough to run every frame.

The reason is not merely GPU latency.

The reason is that shared authority should not depend on undeclared, changing model state.

## 1.4 AI as a resolver, not as protocol authority

Constraining a resolver to rank a closed list is also sensible.

It reduces one class of failure: a model cannot invent a local item that was never offered as a candidate.

Signet 2 further states that the model is unvalidated for this task and proposes measuring whether it actually helps.

That is the right epistemic posture.

A model should earn a role through measured performance, not through architectural enthusiasm.

---

# 2. Vocabulary is not the same thing as semantics

Signet 2 proposes an initial shared intent vocabulary and archetype catalog.

Examples include:

~~~text
move
turn
jump
fire
use

weapon.ranged
weapon.melee
health_pickup
armor
~~~

Those are useful human-readable names.

But interoperability fails in the gaps between names.

Consider “fire.”

One adapter may interpret it as trigger pressed this tick.

Another may interpret it as a request to perform a weapon attack.

A third may use it for an authoritative shot event after cooldown and ammunition checks.

All three can serialize the same word.

They do not mean the same thing.

This is the distinction between syntactic and semantic interoperability: successful parsing only proves that two systems exchanged a structure. It does not prove conceptual alignment.[2][3]

Therefore Signet 2 needs more than a list of names.

It needs an explicit answer to:

> What mechanically makes two implementations of the same shared concept the same concept?

Our proposed answer is:

~~~text
semantic identity
+ semantic definition
+ semantic version
+ profile membership
+ authority/category
+ conformance properties
~~~

The wire label can remain concise.

The identity underneath it must be stable.

---

# 3. Keep the universal core smaller than the gameplay vocabulary

A danger appears as soon as a successful FPS prototype becomes the template for every future game.

Move, jump, weapon.ranged and health_pickup feel universal when the current examples are Doom, OpenArena and Minecraft.

They are not universal across games.

A city builder may have no player body.

A card game may have no continuous position.

A turn-based strategy game may have no per-tick movement intent.

A rhythm game may care about beat-relative action timing rather than world coordinates.

A tabletop engine may create entirely new game rules inside the session.

If every future participant must implement dummy health, movement and weapon concepts just to satisfy the core, then the core is not a protocol core. It is an FPS profile.

## 3.1 Proposed decomposition

We propose a small coordination core:

~~~text
core@1
  semantic identity
  session identity
  participant/adapter identity
  capability negotiation
  message category
  ordering/causality
  authority references
  typed incompatibility
~~~

Gameplay semantics live in independently versioned profiles:

~~~text
profile/spatial-3d@1
profile/entity-lifecycle@1
profile/fps-combat@1
profile/voxel-world@1
profile/inventory@1
profile/turn-based@1
...
~~~

And authoritative mechanics live in rulesets:

~~~text
ruleset/doom-deathmatch@1
ruleset/capture-the-flag@1
ruleset/co-op-survival@1
...
~~~

A **profile** says what a concept means and how it is represented.

A **ruleset** says what happens.

For example:

~~~text
profile:
  damage-applied is an authoritative event with amount/source/target

ruleset:
  armour absorbs 30% of this damage
  weapon X inflicts Y
  death occurs when condition Z is reached
~~~

A shared vocabulary should not silently become a shared game implementation.

## 3.2 We tried to falsify that core outside FPS games

After drafting the small core, we applied it to four deliberately different domain shapes:

1. a turn-based board/tactics game;
2. a deckbuilder/card game with hidden information;
3. a city-builder/management simulation;
4. a user-defined tabletop/game-within-game environment.

This was a **design falsification pass**, not an implementation benchmark.

The result was mostly positive: none of the fixtures required movement, health, weapons, continuous 3D position, or one player body in the universal core.

But the test did expose two FPS-shaped assumptions in our own first draft.

### Ordering cannot mean “tick”

A turn-based game naturally orders state as:

~~~text
turn
phase
action index
~~~

A city simulation may use fixed ticks.

An event-driven tabletop module may need only a monotonic event sequence.

Therefore the core should negotiate an **ordering model**, not standardize one universal tick field.

Examples:

~~~yaml
ordering:
  model: fixed-tick@1
  value:
    tick: 18442
    substep: 3
~~~

~~~yaml
ordering:
  model: turn-sequence@1
  value:
    turn: 42
    phase: action
    action_index: 1
~~~

This is a useful correction to our own earlier draft.

### Authority and visibility are different

The card-game fixture exposed a second missing distinction.

A server can authoritatively know a player's hand without being allowed to disclose it to the opponent.

So:

~~~text
who may assert this fact?
~~~

and:

~~~text
who may observe this fact?
~~~

are separate questions.

The core therefore needs a generic reference to the session/profile's visibility/disclosure policy, while concrete concepts such as hand, fog-of-war and secret objectives remain domain-profile semantics.

The falsification pass strengthens the central claim rather than weakening it:

> the universal core should standardize coordination semantics, while genre meaning belongs in negotiated profiles.

A useful Signet 2 reframing is therefore:

~~~text
semantic core
  +
action/FPS profile v0
  +
presentation/archetype profile v0
  +
doom-deathmatch ruleset
~~~

The version-0 vocabulary can stay small and practical without being mistaken for the forever-core of cross-game interoperability.

---

# 4. Archetype, authoritative state, rules, and appearance are different layers

Signet 2 uses weapon.ranged as a motivating archetype.

The paper explains that Minecraft can show a bow, Doom can show a pistol and OpenArena can show another weapon while the shared server keeps the same damage and range.

That example is useful precisely because it exposes several meanings.

## 4.1 Presentation role

The client needs to know something like:

~~~text
show this entity as a ranged weapon
~~~

That is presentation-oriented semantic information.

## 4.2 Authoritative state

The shared entity may also have:

~~~text
ammunition
cooldown
owner
active/inactive state
position
~~~

## 4.3 Ruleset behavior

The authoritative rules may define:

~~~text
damage
range
projectile/hitscan behavior
fire cadence
spread
reload
collision
~~~

Those mechanics are not supplied by the Minecraft bow or the Doom pistol.

They belong to the shared ruleset.

## 4.4 Why the distinction helps Signet

With these layers separated, a translator can honestly say:

> I can present this semantic object using a bow.

without accidentally saying:

> Minecraft bow mechanics are the shared mechanics.

That makes the translator simpler and the protocol clearer.

---

# 5. Capability intersection should not decide the game

Signet 2 proposes declaring capabilities when a client connects and using the common subset.

This is a good start.

But the mathematical intersection of all client capabilities is not always the correct session.

Suppose a ruleset contains jumping.

- Adapter A supports jump directly.
- Adapter B cannot emit jump at all.
- Adapter C can map jump to a deterministic local action with limitations.
- Adapter D can understand the session but is an observer.

There are several valid outcomes:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

There is no universal rule saying “remove jump because one participant lacks it.”

Doing that can silently change the ruleset for everyone.

## 5.1 Separate six questions

A capability system should distinguish:

1. **implemented** — does this adapter understand the definition?
2. **available** — can this local installation perform it?
3. **activated** — did this session select it?
4. **required** — does correct participation depend on it?
5. **optional** — can it be disabled without changing required meaning?
6. **fallback** — is there an explicitly accepted degraded mapping?

OpenXR requires applications to query available extensions and then enable a selected subset.[4]

glTF separately identifies extensions that are used and extensions that are required to load/render correctly.[5]

The same distinction is valuable here.

## 5.2 Session requirements, not lowest common denominator

A session should say what it requires.

Example:

~~~yaml
required_profiles:
  fps-combat@1:
    - intent/move
    - state/body
    - event/damage-applied
    - event/death

optional:
  - event/damage-direction

ruleset:
  doom-deathmatch@1

allowed_fallbacks:
  presentation/damage-direction: omit
~~~

Then an adapter declaration can be compared against that contract.

The result is explicit.

---

# 6. Approximation needs a type and a loss declaration

Signet 2 correctly wants limited games to remain useful where possible.

But “ignore or approximate” is too broad for a semantic standard.

Some approximation is harmless.

Some changes the game.

### Often safe

~~~text
no directional damage HUD
  -> omit local indicator

no exact local weapon model
  -> show a semantically acceptable local substitute
~~~

### Requires ruleset-specific policy

~~~text
no crouch
no jump
limited turning
different inventory model
~~~

### Not safe as an implicit approximation

~~~text
fire -> use
death -> remove entity
damage requested -> damage applied
teleport request -> authoritative position
~~~

Therefore a fallback should carry:

~~~yaml
concept: ...
mapping_type: ...
lossy: true
loss:
  - ...
accepted_by_session: true
evidence:
  - ...
~~~

A participant should never present degraded semantics as full compatibility.

---

# 7. Translation profiles should record why a mapping is valid

Signet 2's proposed profile entry is intentionally simple:

~~~json
{
  "key": "archetype:weapon.ranged",
  "choice": "minecraft:bow",
  "source": "human",
  "resolver": "clm-v0.1-8b",
  "pinned": true
}
~~~

This is a good decision cache.

A mature interoperability profile needs more.

Determinism and correctness are different properties.

A mapping can be stable and wrong.

## 7.1 Proposed mapping record

~~~yaml
mapping_id: https://example.org/maps/minecraft-ranged-12

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
  - source asset is used for presentation only
  - authoritative fire mechanics come from the session ruleset

produced_by:
  actor: human-confirmed
  resolver: clm-v0.1-8b

evidence:
  conformance:
    - PRESENTATION-RANGED-001

validity:
  source_version_range: "1.21.x"
  target_major: 1
~~~

## 7.2 Reuse mature mapping vocabulary where useful

The W3C SKOS standard already distinguishes mappings such as exact match, close match, broader match and narrower match.[6]

SKOS was designed for knowledge-organization systems, not gameplay simulation, so it should not simply be imported as the complete Signet mapping model.

But it demonstrates an important principle:

> “similar” is not one relation.

Signet mappings also need operational relations such as:

~~~text
unit transform
coordinate transform
presentation-only
approximate
unsupported
~~~

and explicit loss/assumption metadata.

---

# 8. Names should be localizable; identity should not depend on English

Signet/1 currently uses Spanish field names.

Signet 2 proposes an English vocabulary for the new version.

English aliases may be convenient for contributors.

But changing the human language does not solve the identity problem.

A semantic concept should be identifiable independently of its label.

## 8.1 Use globally scoped semantic identifiers

One possible draft form is:

~~~text
https://signetprotocol.io/sem/core/session/1
https://signetprotocol.io/sem/profile/fps-combat/1
https://signetprotocol.io/sem/event/damage-applied/1
https://signetprotocol.io/sem/ruleset/doom-deathmatch/1
~~~

Human-facing labels can then be localized without changing wire identity.

## 8.2 Why not simply use urn:signet?

Our earlier research used that syntax illustratively.

It should not become normative by accident.

RFC 8141 makes URN namespace assignment a managed process; a syntactically plausible urn:<nid>:... string is not a valid public URN merely because it looks like one.[7]

Unless a Signet URN namespace is formally registered, identifiers under a domain the project controls are simpler and mechanically honest.

---

# 9. Definition hashes turn names into checkable agreements

A versioned semantic identifier is useful.

A definition digest makes one dangerous disagreement detectable:

~~~text
adapter A:
  fps-combat@1
  hash = AAA

adapter B:
  fps-combat@1
  hash = BBB
~~~

If both claim the same semantic version but have different canonical definition bytes, the system should not choose whichever copy is newer.

It should say:

~~~text
SEMANTIC_DEFINITION_CONFLICT
~~~

and fail or require explicit reconciliation.

The hash is not a replacement for the identifier.

It is evidence that both sides are talking about the same published definition.

---

# 10. The server-authority model should be explicit in the contract

Signet already has a strong authority model:

- clients issue intent;
- server rules decide shared state/effects;
- clients own local presentation.

We recommend making that relation explicit rather than leaving it implicit in documentation.

For example:

~~~yaml
authority:
  intent/player:
    issuer: participant-self

  state/body:
    issuer: server-rules

  event/damage-applied:
    issuer: server-rules

  presentation/local:
    issuer: local-adapter
~~~

This does not require a complex distributed authority system today.

It simply makes a crucial semantic fact machine-readable.

---

# 11. Time and causality should survive transport changes

Signet/1 has useful deterministic machinery already:

- fixed simulation ticks;
- numbered client commands;
- acknowledgement through authoritative state;
- client reconciliation.

Signet 2 should preserve the semantics independently from JSONL/TCP.

A future replay file, QUIC transport or local IPC path should be able to represent the same causal record.

A shared event can therefore carry logical information such as:

~~~yaml
event_id: ...
tick: 18442
sequence: 3
caused_by:
  - intent:player104:1042
~~~

The point is not to import the full HLA time-management API.

The point is to avoid accidentally making “TCP arrived first” the definition of simulation causality.

HLA is useful precedent for treating coordinated exchange, ownership and time as protocol concerns, while its Object Model Template specification explicitly separates object-model syntax from domain content.[8]

---

# 12. Calibration is useful, but it should not become hidden physics authority

Signet 2 proposes measuring movement properties such as walk/run speed, jump height, turn rate, eye height and step height.

That is useful adapter work.

At the same time, the proposal correctly says that common server physics remains authoritative.

These statements are compatible if the motion profile is treated as information at the **local transduction boundary**.

It can help:

- map local controls into shared intent;
- tune local presentation and prediction;
- detect that an adapter cannot faithfully represent a required capability;
- generate evidence for a profile/ruleset proposal.

It should not silently override the shared rules.

If the server says the shared body moves according to ruleset X, a local game's native speed is not an alternative source of authority.

---

# 13. Closed-choice AI reduces hallucination; it does not establish meaning

Signet 2's resolver design deserves support.

A ranker restricted to known candidates cannot fabricate an unknown candidate.

But the remaining error is the important one:

~~~text
the wrong known candidate
~~~

Therefore:

~~~text
high score
!=
semantic conformance
~~~

and:

~~~text
human confirmation
!=
proof that two independent implementations agree
~~~

For simulation-affecting mappings we recommend:

~~~text
candidate proposal
    ->
human/maintainer acceptance
    ->
pinned mapping with provenance
    ->
conformance/replay evidence
    ->
session admission
~~~

This complements Signet Forge rather than replacing it.

Forge becomes the tool that produces and improves evidence-bearing mapping artifacts.

---

# 14. Conformance should test meaning, not only structure

The strongest way to make Signet 2 credible is to publish semantic ambiguity tests early.

These tests should contain pairs that are easy to confuse.

## 14.1 Example corpus

~~~text
fire button held
vs
shot emitted

remove entity
vs
death

health == 0
vs
dead

use button pressed
vs
interaction completed

teleport requested
vs
authoritative pose changed

camera position
vs
body feet position

message received
vs
command applied
~~~

Two adapters pass only if they make the same distinction.

## 14.2 Proposed conformance families

### ID — semantic identity

- localized label changes preserve identity;
- same ID + divergent definition hash fails closed.

### NEG — negotiation

- supported but unselected capability stays inactive;
- unknown required capability rejects;
- unknown optional capability disables explicitly;
- observer-only is represented explicitly.

### MAP — translation mappings

- exact mappings preserve meaning;
- coordinate/unit transforms are deterministic;
- lossy mappings expose loss;
- approximate mappings cannot claim full support.

### AUTH — authority

- intent does not become effect;
- client cannot self-assign authoritative state ownership;
- local presentation cannot mutate shared truth.

### ORDER — ordering and causality

- fixed-tick, turn-sequence and event-sequence participants can use the same core with different negotiated ordering models;
- replay preserves authoritative ordering independently from transport framing;
- duplicates do not repeat exactly-once effects;
- receipt and application remain distinct.

### VIS — disclosure

- authoritative knowledge is not automatically visible to every participant;
- hidden semantic state obeys the negotiated disclosure policy;
- spectator/observer visibility is explicit rather than assumed.

### PROFILE — modularity

- a non-FPS/non-spatial game can implement the core without fake movement or weapons.

That last test tells us whether the semantic core is actually universal.

---

# 15. A concrete negotiation example

Consider a session requiring:

~~~yaml
ruleset: doom-deathmatch@1

required:
  spatial-3d@1:
    - pose

  fps-combat@1:
    - intent/move
    - state/body
    - event/damage-applied
    - event/death

optional:
  fps-combat@1:
    - event/damage-direction
~~~

A Minecraft adapter declares:

~~~yaml
supported:
  pose: full
  intent/move: full
  state/body: full
  event/damage-applied: full
  event/death: full
  event/damage-direction: unsupported
~~~

The correct result is:

~~~yaml
result: SUPPORTED_WITH_FALLBACKS
disabled:
  - event/damage-direction
~~~

The missing local damage-direction display does not change shared game meaning.

Now change the session so that intent/jump is required and the adapter has no safe jump mapping.

The result may be:

~~~yaml
result: INCOMPATIBLE
reason: REQUIRED_CONCEPT_UNSUPPORTED
~~~

or, if the ruleset explicitly permits observers:

~~~yaml
result: OBSERVE_ONLY
~~~

That is more informative than silently shrinking the game to the common subset.

---

# 16. Migration: Signet does not need to throw away Signet/1

None of this requires a rewrite-first strategy.

Signet/1 already has useful seeds:

- intent-based control;
- server authority;
- numbered commands;
- state/event snapshots;
- translator separation;
- additive compatibility rules.

A practical migration can be incremental.

## Stage A — semantic metadata beside Signet/1

Add optional capability/profile metadata to the greeting.

Treat current Signet/1 gameplay as an explicitly named legacy profile/ruleset combination.

Signet/1's append-only optional-field rule can preserve **syntactic** compatibility while this metadata is introduced. It should not be treated as proof of **semantic** compatibility: an old client that ignores a new capability field may still be unable to satisfy a session's required meaning. Parser compatibility and session admission are separate decisions.

## Stage B — negotiate before authoritative participation

Before a player body is admitted, derive one typed compatibility result.

Older clients can continue on the documented legacy path.

## Stage C — publish machine-readable profile definitions

Start with the concepts already exercised by the beta.

Do not attempt to model all games.

## Stage D — add mapping provenance to translation profiles

Extend Forge's profile format without changing the basic workflow:

~~~text
suggest
review
pin
test
publish
~~~

## Stage E — require a second domain before declaring the core stable

Implement the semantic core against something deliberately unlike Doom/OpenArena/Minecraft.

A card game, turn-based game or city-builder experiment is more valuable here than adding a fourth FPS.

If the core survives that test without dummy FPS concepts, it is becoming a real interoperability core.

---

# 17. Answers to the Signet 2 open questions

Signet 2 ends with concrete unresolved questions.

Here are our proposed answers.

## 17.1 What is the minimum a motion profile must measure?

There should be no universal motion-profile schema.

Its required fields come from the activated semantic profile and ruleset.

For an FPS adapter, measurements may include input response, coordinate/reference-point conventions, camera offset and representational limits.

Authoritative movement constants remain ruleset values.

## 17.2 How do we measure motion where the game does not expose position?

Use legitimate observable evidence:

- official plugin/mod APIs;
- a controlled server;
- open-source engine state;
- game-provided telemetry/replay;
- sufficiently accurate permitted external observation.

When the required quantity cannot be measured reliably, record that limitation.

UNKNOWN or UNSUPPORTED is better than false precision.

## 17.3 Who maintains the vocabulary and catalog?

Do not create one ever-growing global list.

Maintain:

- a conservatively governed core;
- separately versioned profiles;
- namespaced experimental extensions;
- conformance vectors with each profile;
- independent implementations before stabilization.

The extension-governance patterns of OpenXR and modular FOM patterns in HLA are useful precedents.[4][9]

## 17.4 Where should the model run?

This should not be a protocol requirement.

The protocol consumes a pinned mapping.

Forge can run locally, on a development workstation, or on a shared service.

The important property is that model execution is not required to interpret an already pinned simulation contract.

## 17.5 How are conflicting community profiles resolved?

Let them coexist until policy/evidence chooses among them.

A session binds exact:

~~~text
profile ID
version
definition hash
mapping version
source-game applicability
~~~

Then conformance evidence, maintainership and local/server policy decide what is accepted.

Do not make “latest file wins” the semantic rule.

---

# 18. What this means for Signet Forge

This response does not shrink the role of Forge.

It makes Forge's output more valuable.

Today the draft workflow is:

~~~text
discover
classify
review
pin
~~~

A mature workflow can become:

~~~text
discover
classify
review
pin
validate
publish evidence
~~~

The model can still rank candidates.

Humans can still make the final decision.

The difference is that a mapping can now answer:

- What definition was targeted?
- For which game versions?
- Is the mapping exact or approximate?
- What information is lost?
- What assumptions does it rely on?
- Which conformance tests passed?

That is exactly the kind of artifact a community can improve through Git.

---

# 19. What should be implemented next

We have now completed the paper-design version of the non-FPS falsification across board/turn, card/hidden-information, city-simulation and user-defined tabletop domains.

The next step is to convert that into executable evidence.

## Experiment 1 — cross-domain core implementation

Implement the same core negotiation machinery in:

- the current Signet FPS path; and
- one small turn-based card/board fixture.

Use different activated profiles and different ordering models.

**Pass:** both use the same core without dummy domain fields.

**Failure:** genre-specific exceptions leak back into core semantics.

## Experiment 2 — incomplete capability matrix

Create deliberately incomplete adapters:

- no jump;
- observer-only;
- input-only;
- presentation-only;
- legacy unnumbered Signet/1 commands.

For each session contract, require an exact expected admission result.

**Pass:** all independent implementations derive the same result/reason.

## Experiment 3 — ambiguous semantic pairs

Publish a corpus of deceptively similar concepts.

Examples:

~~~text
request / effect
remove / death
pose origin / camera origin
received / applied
~~~

**Pass:** independently built adapters agree on the distinctions and replay outcome.

These experiments will reveal more than another fifty vocabulary entries.

---

# 20. Conclusion

Signet 2 has moved toward the right abstraction.

Its strongest ideas should be preserved:

- game-to-meaning translation instead of pairwise bridges;
- intents on the client side;
- authoritative shared effects on the server;
- local presentation freedom;
- capabilities;
- calibration;
- deterministic pinned translation profiles;
- AI as an offline/exception-path assistant rather than runtime authority.

The remaining problem is smaller, but fundamental.

“Shared meaning” cannot be merely a short list of shared names.

A protocol for independently developed adapters must also define how meaning is:

~~~text
identified
versioned
selected
required
degraded
mapped
authorized
ordered
tested
~~~

The central recommendation of this paper is therefore:

> Keep Signet 2's architecture, but place a small negotiated semantic contract between translation and simulation. Put gameplay vocabulary into modular profiles, mechanics into rulesets, mapping assumptions into evidence-bearing translation profiles, and interoperability claims behind reproducible conformance tests.

If Signet does that, its current proposal can grow beyond a clever cross-game translator architecture into something more durable:

**a protocol by which different games can disagree locally while still proving that the shared parts mean the same thing.**

---

# References

[1] **Signet Protocol.** “Signet 2: intents, archetypes and translation profiles.” Draft proposal v0.1, 5 October 2026. Public source commit 490dfa9423841a45f2917d8d013e010ca0eb5548.  
https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/content/proposals/translation-profiles.mdx  
PDF: https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/public/signet-2-architecture.pdf

[2] **Tolk, Andreas; Muguira, James A.** “The Levels of Conceptual Interoperability Model.” 2003 Fall Simulation Interoperability Workshop, Orlando, 2003.  
https://www.mscoe.org/content/uploads/2017/12/Tolk-Muguira-The-Levels-of-Conceptual-Interoperability-Models.pdf

[3] **Tolk, Andreas.** “Conceptual alignment for simulation interoperability: lessons learned from 30 years of interoperability research.” *SIMULATION* 100(7), 2024, pp. 709-726. DOI: 10.1177/00375497231216471.  
https://doi.org/10.1177/00375497231216471

[4] **Khronos Group.** OpenXR 1.1 specification and extension process.  
https://registry.khronos.org/OpenXR/specs/1.1-khr/html/xrspec.html  
https://registry.khronos.org/OpenXR/specs/1.1/extprocess.html

[5] **Khronos Group.** glTF 2.0 Specification, extension mechanism, including extensionsUsed and extensionsRequired.  
https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html

[6] **W3C.** SKOS Simple Knowledge Organization System Reference, Mapping Properties. W3C Recommendation, 2009.  
https://www.w3.org/TR/skos-reference/#mapping

[7] **Saint-Andre, Peter; Klensin, John.** RFC 8141: Uniform Resource Names (URNs). IETF, 2017.  
https://www.rfc-editor.org/rfc/rfc8141.html

[8] **IEEE.** IEEE 1516.2-2025, *High Level Architecture (HLA) — Object Model Template (OMT) Specification*.  
https://standards.ieee.org/ieee/1516.2/6689/

[9] **Simulation Interoperability Standards Organization (SISO).** Data Files, including HLA Evolved modular FOM examples and reference modules.  
https://www.sisostandards.org/page/DataFiles

---

## Provenance note

The first public commit of the L1 semantic-contract proposal used in this response was e4ba7160ed400960fbee20375fa2b618219af304 at 2026-10-05T16:13:53Z. The first L1 pass completed at 57048f8982289d751616d29b54b9aa4901c6ddd6 at 16:17:48Z. Signet's public Signet 2 proposal/PDF commit 490dfa9423841a45f2917d8d013e010ca0eb5548 followed at 16:31:40Z.

This timing is recorded for research provenance only. It is not presented as evidence of priority or independence.
