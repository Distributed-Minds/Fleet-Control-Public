# L1 Continuation 02 — Negotiation, Mapping Provenance, and Conformance Vectors

**Lane:** L1 — Protocol & Semantic Interoperability  
**Depends on:** [01-Semantic-Contract-Draft.md](01-Semantic-Contract-Draft.md)

This document turns the L1 semantic design into deterministic data shapes and properties that another team can implement without private verbal clarification.

# 1. Capability advertisement draft

The exact serialization is **PROPOSED**, not normative. The important part is the information model.

~~~yaml
kind: signet.capabilities
core:
  id: https://signetprotocol.io/sem/core/session/1
  definition_hash: sha256:<digest>

participant:
  participant_id: participant:alice
  game_id: minecraft
  adapter:
    implementation_id: https://example.org/adapters/minecraft-gateway
    version: 0.4.2

protocol:
  supported:
    - signet/1
    - signet/2-draft

profiles:
  - id: https://signetprotocol.io/sem/profile/spatial-3d/1
    definition_hash: sha256:<digest>
    support:
      publish:
        - pose
      consume:
        - pose
      observe: []
    required:
      - pose
    optional:
      - velocity
    limits:
      orientation:
        - yaw
      vertical_movement: false
    fallbacks:
      velocity: derive_or_omit

  - id: https://signetprotocol.io/sem/profile/fps-combat/1
    definition_hash: sha256:<digest>
    support:
      publish:
        - intent/move
        - intent/fire
      consume:
        - state/body
        - event/shot
        - event/damage-applied
        - event/death
      observe:
        - event/respawn
    required:
      - state/body
      - event/death
    optional:
      - event/damage-direction

rulesets:
  implemented:
    - id: https://signetprotocol.io/sem/ruleset/doom-deathmatch/1
      definition_hash: sha256:<digest>

time:
  supported_models:
    - id: fixed_tick
      tick_hz: [20]
      numbered_commands: true
      replay: true

authority:
  can_publish:
    - intent/player
  can_consume:
    - state/body
    - event/gameplay

world:
  representations:
    - heightfield-2.5d
    - voxel-grid

extensions:
  used:
    - https://signetprotocol.io/sem/ext/damage-direction/1
  required: []
~~~

## Why these fields are separate

- `game_id` is a label/domain identity.
- `implementation_id` identifies adapter software.
- adapter `version` identifies that software release.
- profile `id` identifies meaning.
- `definition_hash` detects divergent definitions under the same meaning/version identifier.
- `support` describes roles the implementation can perform.
- `required` describes what must be activated for this participant to operate correctly.
- `optional` describes features it can use but can live without.
- `limits` records bounded capability instead of pretending all profile implementations are equivalent.
- `fallbacks` make degraded semantics visible.
- `extensions.used` versus `extensions.required` follows the useful glTF distinction between optional use and hard requirements.

# 2. Session requirement draft

The host/session states what the actual session requires.

~~~yaml
kind: signet.session-requirements

core:
  id: https://signetprotocol.io/sem/core/session/1
  definition_hash: sha256:<digest>

required_profiles:
  - id: https://signetprotocol.io/sem/profile/spatial-3d/1
    definition_hash: sha256:<digest>
    required:
      - pose

  - id: https://signetprotocol.io/sem/profile/fps-combat/1
    definition_hash: sha256:<digest>
    required:
      - state/body
      - intent/move
      - event/damage-applied
      - event/death

ruleset:
  id: https://signetprotocol.io/sem/ruleset/doom-deathmatch/1
  definition_hash: sha256:<digest>

time:
  model: fixed_tick
  tick_hz: 20
  numbered_commands: required

authority:
  intent/player: participant-self
  state/body: server-rules
  event/gameplay: server-rules

allowed_degradation:
  presentation:
    damage-direction: optional
  gameplay:
    jump: absent
~~~

The session requirement should not enumerate every capability in every adapter. It defines only the semantic contract needed for this session.

# 3. Deterministic negotiation algorithm

**PROPOSED:** negotiation is a pure function over:

~~~text
session requirements
+ participant capability declaration
+ protocol compatibility policy
= negotiated contract or rejection
~~~

## N0 — Core definition

1. Find an exact core semantic ID match.
2. If the same ID has different definition hashes, reject with `SEMANTIC_DEFINITION_CONFLICT`.
3. Do not infer equivalence from similar human names.

## N1 — Protocol transport/envelope compatibility

1. Select a protocol/envelope version supported by both sides.
2. Signet/1 bridging may be allowed only through an explicitly declared legacy mapping.
3. A scalar version mismatch is not automatically safe.

## N2 — Required profiles

For every session-required profile:

1. require exact profile semantic ID;
2. require matching definition hash;
3. require every session-required concept/role;
4. evaluate participant limits;
5. fail when no declared safe fallback exists.

## N3 — Optional profiles/extensions

Optional features may be activated only when both sides advertise support.

Unknown optional features are ignored **as capabilities**, not silently interpreted as known meaning.

Unknown required features produce `REQUIRED_CONCEPT_UNSUPPORTED`.

## N4 — Time model

Require an explicit compatible time model.

For current Signet-style play, a participant that cannot provide numbered commands may be:

- rejected;
- accepted through a specifically declared legacy fallback; or
- limited to observer mode.

Which result is valid is a session policy decision, but the outcome must be deterministic.

## N5 — Authority roles

Check whether the participant can perform the roles expected by the session.

Example:

- a normal player adapter must publish player intent;
- an observer does not need to publish gameplay intent;
- a client must not claim server-rules authority.

This is a semantic authority check. Authentication of the issuer belongs to L3.

## N6 — Derive result

Return exactly one top-level result:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

Also return:

- selected protocol;
- activated profiles;
- activated extensions;
- fallbacks;
- disabled optional features;
- machine-readable reasons;
- human-readable diagnostics.

# 4. Negotiated contract example

~~~yaml
kind: signet.negotiated-contract
contract_id: contract:7d6d...
result: SUPPORTED_WITH_FALLBACKS

protocol: signet/2-draft

activated_profiles:
  - id: https://signetprotocol.io/sem/profile/spatial-3d/1
    definition_hash: sha256:<digest>
    concepts:
      pose: full
      velocity: fallback-omit

  - id: https://signetprotocol.io/sem/profile/fps-combat/1
    definition_hash: sha256:<digest>
    concepts:
      intent/move: full
      intent/fire: full
      state/body: full
      event/damage-applied: full
      event/death: full
      event/damage-direction: disabled

ruleset:
  id: https://signetprotocol.io/sem/ruleset/doom-deathmatch/1
  definition_hash: sha256:<digest>

time:
  model: fixed_tick
  tick_hz: 20
  numbered_commands: true

authority:
  intent/player: participant:self
  state/body: server:rules
  event/gameplay: server:rules

fallbacks:
  - concept: velocity
    behavior: omit
  - concept: event/damage-direction
    behavior: disabled

reason_codes:
  - OPTIONAL_CONCEPT_DISABLED
~~~

# 5. Mapping provenance record

A mapping is a durable artifact, not merely code hidden inside an adapter.

**PROPOSED shape:**

~~~yaml
mapping_id: https://example.org/mappings/minecraft-player-to-spatial3d/12

source:
  system: minecraft-java
  version_range: "1.21.x"
  concept: player-position
  local_definition: feet-aligned-player-location

target:
  semantic_id: https://signetprotocol.io/sem/profile/spatial-3d/1#pose.position
  definition_hash: sha256:<digest>

relation:
  mapping_type: transform
  semantic_compatibility: exact
  pragmatic_compatibility: supported

transforms:
  - kind: unit
    from: minecraft-block
    to: metre
    expression: "1 block = 1 metre"
  - kind: coordinate-frame
    notes: "explicit axis/sign mapping"

loss:
  lossy: false
  omitted: []

assumptions:
  - "source location denotes feet position"
  - "world scale is 1 block == 1 metre for this adapter"

produced_by:
  actor_type: human-reviewed-tool
  tool: adapter-generator
  tool_version: 0.3.0

evidence:
  conformance_vectors:
    - MAP-SPATIAL-001
    - MAP-SPATIAL-002
  replay_hashes:
    - sha256:<fixture>

validity:
  source_from: 1.21.0
  source_through: 1.21.9
  target_profile_major: 1
~~~

## Mapping relation vocabulary

Minimum useful mapping types:

~~~text
exact
rename
unit-transform
coordinate-transform
narrowing
broadening
approximate
presentation-only
unsupported
~~~

### exact

Meaning and operational effect are preserved.

### rename

Only local naming differs.

### unit-transform / coordinate-transform

Meaning is preserved through a deterministic reversible transform.

### narrowing

Source concept is more specific than target. Information may be lost.

### broadening

Target concept is more specific than source. Adapter must not invent missing detail.

### approximate

Meaning/effect is only approximate. Must not produce a `SUPPORTED` claim for the affected required concept unless the session explicitly accepts that approximation.

### presentation-only

Useful locally but carries no shared authoritative meaning.

### unsupported

No safe mapping exists.

# 6. Semantic ambiguity corpus

These are mandatory negative examples for future L4 testing.

| Pair | Why they are not equivalent |
|---|---|
| fire button held / shot emitted | weapon cooldown/ammo/rules may suppress shot |
| damage requested / damage applied | armour/rules/authority may change result |
| remove entity / death | removal can mean unload/disconnect/despawn |
| respawn / create new entity | identity may persist through respawn |
| use button / door opened | use can target nothing or another interaction |
| requested teleport / authoritative pose changed | server can reject request |
| local animation / gameplay event | presentation is not authoritative effect |
| camera position / body feet position | distinct spatial reference points |
| health zero / dead | some rulesets may have downed or delayed-death states |
| sequence received / command applied | receipt is not authoritative application |

# 7. Exact conformance properties for L4

The following properties are the L1 -> L4 handoff. L4 owns implementation, fixtures, benchmarking and compatibility evidence.

## ID family — semantic identity

### ID01 — Label independence

Changing localized human labels does not change semantic identity.

### ID02 — Definition conflict rejection

Same semantic ID/version + different canonical definition hash => deterministic rejection.

### ID03 — Namespace collision resistance

Two independently authored extensions cannot accidentally acquire the same accepted identifier through bare short names.

## NEG family — capability negotiation

### NEG01 — Determinism

Identical session requirements and capability manifests produce byte-for-byte equivalent normalized negotiation results, ignoring explicitly non-semantic diagnostic ordering.

### NEG02 — Support is not activation

A supported optional profile is inactive unless selected into the contract.

### NEG03 — Required unknown fails closed

Unknown required profile/concept => `INCOMPATIBLE` with a stable reason code.

### NEG04 — Optional unknown degrades safely

Unknown optional profile/concept does not prevent admission and is recorded disabled.

### NEG05 — Hash conflict fails closed

Matching human name/ID with divergent definition hash cannot fall back to “close enough.”

### NEG06 — Observer derivation

A participant that understands required state/events but cannot provide required action roles can deterministically become `OBSERVE_ONLY` when the session permits observers.

## MAP family — mapping meaning

### MAP01 — Exact round trip

For mappings declared exact and reversible, source -> semantic -> source preserves the fixture value and required meaning.

### MAP02 — Unit transform invariance

Equivalent physical values in different source units normalize to the same semantic value.

### MAP03 — Coordinate reference-point correctness

Feet/collider/camera origins cannot pass the same pose test unless the declared mapping correctly transforms them.

### MAP04 — Loss declaration

A narrowing/broadening/approximate mapping must expose its declared loss; undeclared loss is a failure.

### MAP05 — Approximation cannot silently become full

If a required concept is only approximate, the result cannot be `SUPPORTED` unless the session explicitly declares the approximation acceptable.

## AUTH family — semantic authority

### AUTH01 — Self-assertion is insufficient

A participant cannot gain a semantic authority scope by placing that scope in its own message.

### AUTH02 — Epoch stale-claim rejection

After authority transfer to epoch N+1, claims from epoch N beyond the transfer boundary are rejected/ignored deterministically.

### AUTH03 — Intent/effect separation

A valid player intent cannot be accepted as authoritative state/effect merely because its fields resemble the result.

## TIME family — ordering and causality

### TIME01 — Replay-order independence from transport framing

The same ordered semantic event set reproduces the same authoritative result across live JSONL and replay-file framing.

### TIME02 — Wall clock is not simulation order

Perturbing observation timestamps does not change authoritative tick/sequence ordering.

### TIME03 — Duplicate event identity

Replaying the same authoritative event ID cannot produce the effect twice where exactly-once effect semantics are specified.

### TIME04 — Cause references survive replay

`caused_by` references remain resolvable and stable in captured replay fixtures.

## PROFILE family — modularity

### PROF01 — Optional profile independence

A participant lacking a non-required profile can still join without pretending to understand it.

### PROF02 — Ruleset/profile separation

Changing a ruleset while keeping profile meaning constant does not silently redefine the profile's concept schema.

### PROF03 — Domain leakage test

A non-spatial/non-FPS fixture can implement `core@1` without dummy transforms, health, weapons or movement fields.

This test is important: it proves that the core is actually cross-domain.

## LEGACY family — Signet/1 bridge

### LEG01 — Legacy sequence semantics explicit

A `seq=0`/unnumbered Signet/1 participant cannot be classified as equivalent to numbered-command semantics without an explicit fallback result.

### LEG02 — Position compatibility explicit

A `Posicion` gateway that bypasses movement-rule intent is represented as a different capability path, not full intent equivalence.

### LEG03 — Optional-default presence semantics

Fixtures must distinguish “field absent and defaulted” from “field explicitly present with a value” wherever the profile treats presence as meaningful.

# 8. Concrete negotiation vectors

## V1 — Full support

Input:

- same core ID/hash;
- same required profile IDs/hashes;
- all required roles present;
- fixed 20 Hz + numbered commands supported.

Expected:

~~~text
SUPPORTED
~~~

## V2 — Missing optional presentation concept

Input:

- all required concepts present;
- `damage-direction` absent;
- session marks it optional.

Expected:

~~~text
SUPPORTED_WITH_FALLBACKS
reason: OPTIONAL_CONCEPT_DISABLED
~~~

## V3 — Same profile ID, divergent definition

Input:

- both advertise `fps-combat/1`;
- hashes differ.

Expected:

~~~text
INCOMPATIBLE
reason: SEMANTIC_DEFINITION_CONFLICT
~~~

No human-label comparison or “newer wins” heuristic is allowed.

## V4 — Viewer understands but cannot act

Input:

- can consume all required state/events;
- cannot publish required player intent;
- observers allowed.

Expected:

~~~text
OBSERVE_ONLY
~~~

## V5 — Legacy Signet/1 unnumbered commands

Input:

- bridge understands Signet/1;
- participant uses `seq=0` semantics;
- session requires numbered commands with no legacy fallback.

Expected:

~~~text
INCOMPATIBLE
reason: TIME_MODEL_UNSUPPORTED
~~~

If the session permits the legacy path, expected instead:

~~~text
SUPPORTED_WITH_FALLBACKS
reason: LEGACY_SEMANTICS_ONLY
~~~

# 9. Handoffs

~~~yaml
handoff:
  from_lane: L1
  to_lane: L2
  finding: Capability manifests and accepted semantic mappings should be machine-readable durable artifacts.
  evidence:
    - L1 capability and mapping drafts
  why_material: Enables scaffold/code generation without letting models invent the contract.
  requested_followup: Test whether profile + local API description can generate adapter stubs and mapping skeletons while preserving these fields.
~~~

~~~yaml
handoff:
  from_lane: L1
  to_lane: L3
  finding: L1 defines issuer and authority-scope semantics but deliberately does not define how issuer identity is authenticated.
  evidence:
    - authority table and epoch model
  why_material: Semantic authority is meaningless if package/session identity can be spoofed.
  requested_followup: Bind identity/signature/session-auth mechanisms to participant and adapter identifiers without changing L1 meaning.
~~~

~~~yaml
handoff:
  from_lane: L1
  to_lane: L4
  finding: The ID/NEG/MAP/AUTH/TIME/PROFILE/LEGACY properties above are the exact semantic conformance surface.
  evidence:
    - deterministic vectors V1-V5
  why_material: Turns semantic interoperability claims into independent reproducible evidence.
  requested_followup: Implement fixtures and cross-implementation tests; report any property that cannot be tested without changing the semantic contract.
~~~

# 10. Next recursion

Highest-value L1 follow-ups:

1. test `core@1` against a **non-spatial game** to prove the core has no hidden FPS assumptions;
2. define canonicalization rules for semantic-definition hashes;
3. compare profile composition conflict rules with HLA modular FOM merge semantics;
4. test whether ruleset/profile separation survives Doom/OpenArena/Minecraft plus a turn-based/card-game fixture;
5. formalize profile dependency constraints without turning the protocol into a general-purpose ontology engine.
