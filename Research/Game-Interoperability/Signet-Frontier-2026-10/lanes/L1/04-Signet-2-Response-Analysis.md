# L1 Continuation 04 — Response Analysis: Signet 2 Intents, Archetypes, and Translation Profiles

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 2  
**Research cutoff:** 2026-10-05  
**Primary new source:** Signet Protocol commit `490dfa9423841a45f2917d8d013e010ca0eb5548`  
**Upstream status:** draft proposal, explicitly not implemented

## Why this changes L1

Signet published its Signet 2 architecture proposal after the first L1 continuation pass.

The proposal is not a competing architecture. It converges on several of the same pressure points:

- stop pairwise game-to-game translation;
- translate local game behavior into shared meaning;
- distinguish player intent from authoritative effects;
- declare capabilities before play;
- keep AI outside the per-tick authoritative path;
- save human-confirmed choices in a deterministic translation profile;
- retain one authoritative server for shared physics/rules/state.

This materially strengthens the case for the L1 direction.

The new task is not to reject Signet 2. It is to specify what **shared meaning** has to mean mechanically if independent teams are expected to implement it without hidden verbal assumptions.

## Source chronology

The first public L1 semantic-contract commit was:

~~~text
e4ba7160ed400960fbee20375fa2b618219af304
2026-10-05T16:13:53Z
~~~

The first L1 pass was complete at:

~~~text
57048f8982289d751616d29b54b9aa4901c6ddd6
2026-10-05T16:17:48Z
~~~

Signet's public Signet 2 proposal/PDF commit was:

~~~text
490dfa9423841a45f2917d8d013e010ca0eb5548
2026-10-05T16:31:40Z
~~~

This chronology is **not a priority claim** and does not prove that the work was independent. It does establish that the two public records converged within the same half hour and that L1's profile/negotiation design was not retrofitted after reading the Signet 2 publication.

---

# 1. Strong convergence: keep these Signet 2 decisions

## 1.1 Game -> meaning -> game

Signet 2's central move is correct:

~~~text
game A -> shared meaning -> game B
game C -> shared meaning -> game A
~~~

instead of pairwise translators.

This matches L1's neutral semantic-contract direction and HLA-style federation reasoning.

## 1.2 Intent is not effect

Signet 2 sends intents upward and lets the server apply physics/rules/damage and produce authoritative state/events.

Keep this boundary.

Examples that must remain distinct:

~~~text
fire intent       != shot emitted
use intent        != door opened
jump intent       != authoritative body trajectory
teleport request  != authoritative pose update
damage proposal   != damage applied
~~~

## 1.3 Local appearance is not shared truth

The server can say what something **means** while each game decides how to render it.

That is the right layer boundary for models, textures, sounds, HUD, animation, and local camera/feel.

## 1.4 Pinned translation profiles

The lock-file analogy is strong.

A model/person/table may propose a mapping, but a simulation-affecting mapping becomes deterministic only after it is pinned.

Keep this.

## 1.5 AI outside the authoritative loop

The proposal explicitly keeps the resolver out of the 20 Hz simulation loop.

Keep this invariant even if future local models become cheap enough for realtime use. Cost/latency is not the fundamental reason; **shared authority must not depend on undeclared model state**.

## 1.6 Human-confirmed closed-choice resolution

Closed candidate ranking reduces one important failure class: inventing unsupported local actions/items.

It does not prove semantic correctness, but it is much better than free generation.

---

# 2. Where "shared meaning" is still underspecified

## 2.1 A vocabulary is not yet a semantic contract

Signet 2 proposes roughly 15 intents and 15 archetypes in version 0.

That is useful for a beta, but the following strings are only labels until their meaning is versioned and testable:

~~~text
move
fire
use
weapon.ranged
health_pickup
armor
~~~

Two teams can both implement `fire` and disagree on whether it means trigger pressed, trigger held, request to attack, projectile spawned, hitscan resolved, or shot actually emitted.

Therefore a shared vocabulary must bind each concept to a stable semantic definition.

### L1 consequence

Use:

~~~text
semantic ID
+ semantic version
+ canonical definition hash
+ profile membership
+ role/category
~~~

Human labels remain aliases/documentation.

## 2.2 A flat global vocabulary will become a monolith

`move`, `jump`, `weapon.ranged` and `health_pickup` are already FPS/action-game concepts.

They should not become universal requirements for turn-based games, card games, strategy games, city builders, colony sims, rhythm games, tabletop games, or non-spatial simulations.

### L1 consequence

Keep the universal core small.

~~~text
core@1
  semantic identity
  session/participant identity
  capability negotiation
  ordering/causality
  authority references
  typed incompatibility

profile/spatial-3d@1
profile/entity-lifecycle@1
profile/fps-combat@1
profile/voxel-world@1
profile/inventory@1
profile/turn-based@1
...
~~~

A session activates only the profiles it needs.

## 2.3 "Use the common subset" is not sufficient compatibility semantics

The proposal says clients declare what they can emit/show and the server uses the common subset.

That is not always the desired result.

Suppose the session ruleset requires jump:

- Game A supports jump natively.
- Game B cannot represent jump as player input.
- Game C can represent jump only through a deterministic emulation.

The correct result is not mechanically "remove jump from the match."

Possible outcomes:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

The session/ruleset decides which features are required.

### L1 consequence

Separate implemented capability, available capability, activated capability, required capability, optional capability, and fallback capability.

OpenXR and glTF provide mature examples of this distinction.

## 2.4 Ignore/approximate must be typed, not generic

The Signet 2 draft says unsupported actions can be ignored or approximated and reported.

For local appearance, this is fine.

For authoritative gameplay it can change fairness or causal meaning.

Examples:

~~~text
missing damage-direction presentation -> IGNORE is probably safe
missing jump intent in no-jump mode    -> irrelevant
missing jump intent in jump-required mode -> maybe INCOMPATIBLE
missing crouch                         -> ruleset-specific fallback
weapon.ranged -> visually different item -> presentation mapping
fire intent -> use intent              -> not a safe generic approximation
~~~

### L1 consequence

Every fallback needs a target concept, fallback type, loss declaration, session policy permission, and conformance evidence when simulation-affecting.

## 2.5 Archetype and ruleset are currently conflated

The paper's example says every client can render `weapon.ranged` differently while the server gives the same weapon damage/range.

That is useful, but it reveals different semantic layers:

~~~text
presentation role/archetype:
  "show this as a ranged weapon"

ruleset instance:
  damage = ...
  range = ...
  cadence = ...
  ammunition = ...
  hit resolution = ...
~~~

`weapon.ranged` alone cannot carry both meanings without becoming underspecified.

### L1 consequence

Separate:

1. profile concept/type;
2. authoritative instance state;
3. ruleset behavior;
4. local presentation role.

A Minecraft bow can satisfy the local presentation role without claiming Minecraft's native bow mechanics are the authoritative mechanics.

## 2.6 Pinned deterministic is not pinned correct

A mapping file such as:

~~~json
{
  "key": "archetype:weapon.ranged",
  "choice": "minecraft:bow",
  "source": "human",
  "pinned": true
}
~~~

guarantees repeatability.

It does not establish which source-game version was mapped, which target semantic definition was intended, whether the mapping is exact/approximate/narrower/broader, what information is lost, which assumptions were made, whether tests passed, or when the mapping becomes stale.

### L1 consequence

A simulation-grade translation profile should carry mapping provenance.

Minimum:

~~~yaml
source:
  system: minecraft-java
  version_range: 1.21.x
  concept: local-bow-action

target:
  semantic_id: https://signetprotocol.io/sem/profile/fps-combat/1#intent/fire
  definition_hash: sha256:...

relation:
  type: exact | close | narrowing | broadening | transform | approximate
  lossy: false

assumptions:
  - ...

evidence:
  conformance:
    - ...
  replay:
    - ...

validity:
  source_version_range: ...
  target_profile_major: 1
~~~

SKOS is useful precedent for explicit `exactMatch`, `closeMatch`, `broadMatch` and `narrowMatch` distinctions, although gameplay mappings need additional operational transform/loss metadata.

## 2.7 Closed-list AI still needs conformance

A resolver constrained to five candidates cannot invent candidate six.

It can still rank the wrong candidate first.

A wrong answer that is pinned becomes deterministically wrong.

Signet 2 already acknowledges that no model has been validated for this task.

### L1 consequence

Model score is evidence about a **proposal**, not evidence that the mapping is semantically conformant.

For simulation-affecting mappings:

~~~text
proposal
-> human/maintainer acceptance
-> deterministic profile record
-> conformance/replay evidence
-> session admission
~~~

For presentation-only mappings, policy can be looser.

## 2.8 Motion profile belongs at the transduction boundary, not in shared authority

Signet 2 proposes measuring walk/run speed, jump height, turn speed, eye height, and step height.

This is useful adapter calibration.

But the paper also correctly says the server remains the physics authority.

Therefore the motion profile cannot independently define the shared simulation's physics.

### L1 consequence

Treat motion measurements as local-control transduction parameters, local prediction/presentation parameters, evidence used when constructing a ruleset/profile, or compatibility limits.

Do not let local motion measurements silently override authoritative rules.

## 2.9 "English wire vocabulary" is not the semantic fix

Signet/1 uses Spanish wire field names; Signet 2 proposes English vocabulary later.

English may improve contributor accessibility.

It does not solve semantic identity.

### L1 consequence

Use stable machine identifiers independent of human language.

Short aliases may be English, Spanish, Japanese, etc.

Earlier L1 examples used `urn:signet:...`. That should remain only a placeholder: RFC 8141 requires a registered URN namespace identifier. Until such a namespace exists, project-controlled HTTPS URI identifiers are mechanically cleaner.

---

# 3. Revised Signet 2 model

The strongest synthesis of Signet 2 and L1 is:

~~~text
local game
   |
   | local input/event/state
   v
adapter + pinned translation profile
   |
   | semantic intents / profile state
   v
negotiated semantic contract
   |
   v
authoritative ruleset/server
   |
   | authoritative state/events
   v
negotiated semantic contract
   |
   v
adapter + pinned translation profile
   |
   v
local presentation/game
~~~

The important addition is the **negotiated semantic contract** between translation and simulation.

It contains:

- core semantic version;
- activated profiles;
- exact semantic definition identities/hashes;
- required/optional features;
- accepted fallbacks;
- time model;
- authority table;
- ruleset identity;
- deterministic admission result.

---

# 4. Revised translation-profile role

Signet 2's translation profile should become an **executable mapping manifest**, not just a decision cache.

Suggested logical layers:

~~~text
translation profile
  metadata
    game / adapter / versions

  input mappings
    local input -> semantic intent

  state mappings
    local state <-> semantic state

  presentation mappings
    semantic presentation role -> local asset/action

  limits
    unsupported concepts
    ranges
    approximations

  provenance
    mapping relation
    author/tool
    assumptions
    semantic definition hashes

  evidence
    conformance vectors
    replay fixtures
    compatibility records
~~~

The resolver/Forge edits this artifact.

The runtime only consumes the pinned artifact.

---

# 5. Direct answers to Signet 2 open questions

## Q1 — What is the minimum a motion profile must measure?

There should not be one universal answer.

The minimum is profile/ruleset dependent.

For an FPS adapter, useful calibration may include input response curve/deadzone, yaw/pitch convention, local camera/eye offset, discrete versus continuous input semantics, local maximum representable update rate, and local movement/presentation limits.

If the authoritative server owns walk speed, jump height and step height, those values are **not client authority**. They may be used to assess compatibility or tune local feel, but the shared ruleset remains normative.

## Q2 — How do we measure it when the game does not report position?

If a legitimate integration path cannot observe the quantity with sufficient confidence, do not fabricate a measurement.

Possible evidence sources include an official mod/plugin API, controlled server API, open-source engine state, replay/telemetry exposed by the game, or calibrated external observation where permitted and sufficiently accurate.

If none exists:

~~~text
UNKNOWN
UNSUPPORTED
OBSERVE_ONLY
~~~

is better than a false exact mapping.

## Q3 — Who maintains the vocabulary/catalog?

Avoid one giant global catalog.

Use:

- a small stable core governed conservatively;
- separately versioned semantic profiles;
- namespaced extensions;
- independent implementations before stabilization;
- reproducible conformance vectors;
- explicit deprecation/supersession;
- `used` versus `required` capability declarations.

## Q4 — Where does the model run?

For protocol semantics, it does not matter.

The runtime contract should depend only on the pinned result.

Practical preference:

- Forge/development environment by default;
- local model where useful/privacy-preserving;
- shared service optional;
- never required for deterministic runtime participation.

## Q5 — How are conflicting community profiles resolved?

Do not resolve them by "latest wins" or popularity alone.

Bind a session to exact profile identity/version/hash.

Each mapping/profile should carry source game/adapter version applicability, semantic target identity/hash, provenance, conformance evidence, and known limitations.

A server/session selects an exact accepted profile according to policy.

Conflicting profiles may coexist as competing artifacts until evidence/governance converges.

---

# 6. Required Signet 2 conformance properties

## Semantic identity

- changing human/localized labels does not change concept identity;
- same semantic ID with different definition bytes fails closed;
- namespaced extensions do not collide.

## Negotiation

- supported but unused concepts remain inactive;
- unknown required concepts reject admission;
- unknown optional concepts are disabled explicitly;
- observer-only participation is derivable where allowed;
- accepted fallback is represented in the negotiated contract.

## Mapping

- exact mappings round-trip where meaningful;
- unit/coordinate transforms preserve meaning;
- lossy mappings declare loss;
- approximation cannot silently claim full support;
- mapping version invalidates when source-game changes break assumptions.

## Authority

- input intent cannot be treated as authoritative effect;
- client cannot self-assign server authority;
- local presentation cannot change shared gameplay state.

## Causality/replay

- equivalent replay semantics are independent of transport framing;
- duplicate authoritative event IDs do not duplicate exactly-once effects;
- acknowledgement/receipt is distinct from authoritative application.

## Profile modularity

- a non-FPS/non-spatial participant can implement the core without dummy movement/weapons/health fields;
- profile/ruleset changes do not silently redefine other profiles.

---

# 7. Highest-value experiments now

## E1 — Non-FPS falsification test

Take a game/domain with no natural player body, jump, gun, or health pickup.

Examples: card/deckbuilder, turn-based tactics, city builder, or board game.

Attempt to implement `core@1`.

If dummy movement/combat concepts are required, the core is still too game-specific.

## E2 — Capability negotiation matrix

Construct adapters:

- full FPS;
- no jump;
- observer only;
- presentation only;
- legacy Signet/1 unnumbered intent;
- spatial but non-combat.

For each session profile/ruleset, assert one deterministic result.

## E3 — Semantic ambiguity corpus

Use paired concepts that look similar but differ:

~~~text
fire pressed / shot emitted
remove / death
health zero / dead
button use / world interaction completed
position / feet position / camera position
received command / applied command
~~~

Require independent implementations to agree.

## E4 — Profile drift

Change the source game version so one mapping assumption breaks.

The old compatibility record must become invalid/unknown rather than silently persist.

---

# 8. Publication thesis

The response paper should use this thesis:

> Signet 2 has selected the right architectural direction: games should translate into shared meaning, AI-assisted choices should be pinned before they affect simulation, and one authoritative server should own shared effects. The remaining interoperability problem is to turn "shared meaning" from a small global vocabulary into a versioned, modular, negotiable and testable semantic contract.

That is a constructive extension, not a competing project.
