# L1 Continuation 05 — Non-FPS Core Falsification

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 3  
**Research cutoff:** 2026-10-05  
**Purpose:** Attempt to break the proposed universal semantic core by applying it to domains where Signet 2's version-0 intent/archetype vocabulary is a poor fit.

This is a **design falsification pass**, not an implementation benchmark. The question is whether an independently implemented participant could use the proposed core without inventing dummy FPS concepts.

# 1. Test criterion

The proposed core is acceptable only if all of the following are true:

1. a participant can join without declaring fake movement, health, weapons, world geometry, or continuous 3D pose;
2. ordering can be expressed without assuming fixed ticks;
3. authority can be expressed without assuming one FPS-style player body;
4. capability negotiation can activate domain profiles without changing the core;
5. intent/state/event distinctions remain useful outside realtime action games;
6. incompatibility and fallback remain explicit;
7. the core does not encode one genre's rules as supposedly universal semantics.

# 2. Fixture A — turn-based board game

## Domain shape

A chess-like or tactics-like session has:

- two or more participants;
- a shared board/state;
- turns;
- legal/illegal move proposals;
- authoritative move validation/application;
- deterministic state transitions;
- no need for continuous position, velocity, health, weapon or physics semantics.

## Local-to-shared flow

~~~text
local UI selection
    ->
move intent
    ->
ruleset validates move
    ->
move-applied event
    ->
new authoritative board state
~~~

This maps cleanly onto the core categories:

- intent: proposed move;
- state: authoritative board position;
- event: move applied, capture, check/end-state transition;
- ack: receipt/application acknowledgement where desired;
- control: session admission/resignation/reconnect.

## Ordering result

A universal tick field is wrong here.

Useful ordering is closer to:

~~~yaml
ordering:
  model: turn-sequence
  turn: 42
  phase: action
  action_index: 1
~~~

Therefore L1 should not define tick as a universal field.

It should define a **negotiated ordering model**.

## Authority result

The active participant may be authorized to issue a move intent.

The rules authority decides whether that intent is legal and applies it.

This supports the existing L1 distinction:

~~~text
intent != authoritative effect
~~~

without any FPS assumptions.

## Profiles needed

Possible profiles:

~~~text
profile/discrete-board@1
profile/turn-taking@1
profile/piece-lifecycle@1
~~~

No spatial-3d or fps-combat profile is needed.

## Falsification outcome

**PASS**, with one correction:

- replace example-first tick semantics in the core with an abstract/negotiated ordering model.

# 3. Fixture B — deckbuilder / card game

## Domain shape

A deckbuilder introduces semantics absent from the FPS examples:

- hidden information;
- zones such as deck/hand/discard/exile;
- draw/shuffle operations;
- card ownership/control;
- turn/phase sequencing;
- resource costs;
- effects that may create further effects;
- deterministic or seeded random choice.

There may be no meaningful world position at all.

## Local-to-shared flow

~~~text
player selects card + targets
    ->
play-card intent
    ->
rules authority checks cost/legality
    ->
resource-spent event
    ->
card-moved event
    ->
effect-resolved event(s)
    ->
authoritative zones/state updated
~~~

Again, intent/state/event remains useful.

## New semantic pressure: visibility

The current core does not explicitly discuss **who may observe which state**.

In an FPS prototype, most authoritative state is broadly broadcast.

In a card game:

~~~text
server knows full deck
player knows own hand
opponent must not know hidden hand
spectator policy may differ
~~~

This is not merely presentation.

It is a semantic visibility/disclosure rule.

### Core or profile?

Visibility should **not** become a giant game-specific core vocabulary.

But the core needs a generic way to bind state/event disclosure to a negotiated authority/visibility policy.

Candidate core coordination concept:

~~~yaml
visibility_scope:
  policy_ref: <session/profile-defined-policy>
~~~

The meaning of concrete scopes belongs to profiles/rulesets.

This is analogous to authority:

- core defines the coordination mechanism;
- profile/ruleset defines who can observe which domain facts.

## New semantic pressure: randomness

A shared card game may require authoritative randomization.

The core does not need a universal RNG API.

But replay/conformance must distinguish:

~~~text
random choice requested
!=
authoritative random outcome
~~~

and the ruleset should identify whatever deterministic/replay contract it uses:

~~~yaml
randomness:
  model: ruleset-defined
  replay_artifact: authoritative-outcomes
~~~

This belongs to the ruleset/time/replay contract, not a universal gameplay vocabulary.

## Profiles needed

~~~text
profile/card-zones@1
profile/turn-taking@1
profile/hidden-information@1
profile/resource-costs@1
~~~

## Falsification outcome

**PASS WITH CORE DELTA**:

- add generic visibility/disclosure policy reference to coordination metadata;
- ensure replay semantics account for authoritative nondeterministic outcomes without pretending the RNG algorithm is universal.

# 4. Fixture C — city builder / management simulation

## Domain shape

A city-builder or management simulation may contain:

- large shared structured state;
- commands such as zone/build/delete/change-policy;
- simulation ticks;
- aggregate populations/economy;
- map/grid/graph state;
- no single player body;
- many entities whose lifecycle is system-generated rather than directly player-controlled.

This is useful because it keeps realtime simulation while removing FPS-body assumptions.

## Local-to-shared flow

~~~text
local build tool
    ->
build intent
    ->
rules validate cost/location
    ->
construction accepted/rejected
    ->
simulation advances
    ->
aggregate state/events updated
~~~

## Ordering result

Fixed ticks are valid here.

This confirms that fixed-tick ordering should remain a supported ordering model, not be removed.

## Authority result

Authority may be split by semantic scope:

~~~text
player:
  planning/build intents

rules:
  budget
  construction legality
  simulation effects
  population/economy evolution

local client:
  UI
  camera
  rendering
~~~

No body-position authority is required.

## Profiles needed

~~~text
profile/grid-or-region@1
profile/construction@1
profile/economy@1
profile/population@1
~~~

Again, no fps-combat profile is required.

## Falsification outcome

**PASS**.

The core works if semantic authority is scope-based rather than body-based.

# 5. Fixture D — user-defined tabletop / game-within-game

This is the hardest conceptual test because the session may introduce rules and concepts not known when the base protocol shipped.

## Domain shape

A Tabletop-Simulator-like environment may let participants define:

- custom pieces/cards/tokens;
- custom zones;
- arbitrary turn structure;
- custom actions;
- entirely new minigames.

A fixed global catalog cannot anticipate all of this.

## Consequence

The core must support **session-selected namespaced semantic definitions** without requiring those definitions to be permanently built into the base SDK.

Candidate model:

~~~text
core@1
  loads/negotiates profile identities

session:
  requires profile A
  requires profile B
  optionally uses extension C

participant:
  proves support for A/B
  may ignore optional C
~~~

Profile identity and definition hashing therefore matter even more for user-defined games.

## Governance result

Not every useful profile needs to be centrally standardized before it can exist.

A healthy path is:

~~~text
experimental namespaced profile
    ->
multiple independent implementations
    ->
conformance evidence
    ->
possible stabilization
~~~

This matches the L1 preference for conservative core governance and modular profiles.

## Falsification outcome

**PASS**, provided profile negotiation allows non-core namespaces/extensions.

# 6. Core changes caused by the falsification pass

The previous core draft survives, but it should be sharpened.

## 6.1 Keep

The following remain genuinely cross-domain:

- semantic identifiers + definition binding;
- session identity;
- participant/adapter identity;
- envelope category:
  - intent
  - state
  - event
  - ack
  - control
- capability/profile negotiation;
- authority scope references;
- typed incompatibility;
- explicit fallback/degradation.

## 6.2 Change

### Ordering

Replace any apparent universal requirement for:

~~~text
tick
sequence
~~~

with:

~~~yaml
ordering:
  model: <negotiated semantic ID>
  value: <model-defined structure>
~~~

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

~~~yaml
ordering:
  model: event-sequence@1
  value:
    sequence: 991827
~~~

The core requires an ordering contract when ordering matters; it does not require one particular clock.

### Visibility

Add a generic **visibility/disclosure policy reference** to semantic claims/state where hidden information exists.

The core should not define “hand,” “fog of war,” or “secret objective.”

It should permit the session/profile/ruleset to define who may receive a semantic fact.

### Causality

Keep stable semantic event/message IDs and optional cause references.

These survive all four fixtures.

### Randomness

Do not add generic RNG semantics to the core.

Require replay/conformance to capture authoritative outcomes where nondeterminism affects shared state.

# 7. Revised core@1 candidate

~~~text
core@1
  semantic identity
    semantic_id
    semantic_version / definition_hash

  session identity
    session_id
    negotiated_contract_id

  participant identity
    participant_id
    adapter implementation/version

  semantic category
    intent
    state
    event
    ack
    control

  ordering/causality
    ordering_model
    ordering_value
    message/event identity
    caused_by

  authority
    issuer
    authority_scope
    authority_epoch where applicable

  visibility/disclosure
    visibility_policy_ref where applicable

  negotiation
    implemented / available / activated / required / optional
    fallbacks
    extensions/profiles

  result
    SUPPORTED
    SUPPORTED_WITH_FALLBACKS
    OBSERVE_ONLY
    INCOMPATIBLE
~~~

# 8. What is definitively not core after this pass

These concepts fail the cross-domain test and belong in profiles/rulesets:

- transform;
- coordinate frame;
- position;
- velocity;
- body;
- health;
- armour;
- weapon;
- ammo;
- jump;
- crouch;
- movement speed;
- physics tick rate;
- world geometry;
- voxel/block;
- board cell;
- card;
- hand/deck;
- economy;
- population;
- deterministic RNG algorithm;
- rendering/presentation asset.

# 9. Consequence for Signet 2

Signet 2's proposed first vocabulary is a good **first profile**, not a good universal core.

A productive reframing is:

~~~text
Signet semantic core
    +
Signet action/FPS profile v0
    +
Signet presentation archetype profile v0
    +
doom-deathmatch ruleset
~~~

This lets the current Doom/OpenArena/Minecraft work move quickly without hard-coding its concepts into the forever-core of the protocol.

# 10. Stronger falsification experiment for implementation

The next step should be actual independent implementation of the revised core in at least two radically different domains.

Recommended pair:

1. current Signet FPS stack;
2. a small turn-based card/board fixture.

Required result:

- identical core negotiation machinery;
- different activated profiles;
- different ordering models;
- no dummy concepts;
- deterministic admission;
- replay/conformance passes.

If the core requires genre-specific exceptions at that point, revise it again before stabilization.

# 11. L4 handoff delta

~~~yaml
handoff:
  from_lane: L1
  to_lane: L4
  finding: Non-FPS falsification preserves the small coordination core but replaces tick-first ordering with negotiated ordering models and adds generic visibility/disclosure policy references.
  evidence:
    - lanes/L1/05-Non-FPS-Core-Falsification.md
  why_material: The conformance suite must prove the same core works across incompatible genre semantics rather than only across several FPS-like adapters.
  requested_followup: Build one FPS and one turn/card fixture using the same core negotiation code; require no dummy domain fields and assert exact negotiated ordering/visibility behavior.
~~~

# Conclusion

The falsification pass supports the central L1 thesis.

The universal core should be **smaller than Signet 2's first vocabulary**.

Signet's proposed intents and archetypes remain useful, but they belong in versioned profiles that a session selects.

The protocol core should standardize the machinery by which different semantic worlds can identify, negotiate, authorize, order, disclose, degrade and test their meaning.
