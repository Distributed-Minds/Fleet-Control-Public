# L1 Continuation 01 — Candidate Semantic Contract

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 1  
**Research cutoff:** 2026-10-05  
**Evidence labels:** OBSERVED / DERIVED / PROPOSED / CLAIMED / UNKNOWN

## Executive delta

This pass narrows the architecture from “neutral gameplay vocabulary” toward a two-level contract:

1. a **small coordination core** that all participants interpret identically; and
2. **modular semantic profiles** that define domain vocabulary and can be negotiated independently.

The main correction is that spatial transforms, body lifecycle, weapons, health, movement and world geometry are not universal enough to be the protocol core. They belong in profiles or rulesets. The core should define how concepts are named, versioned, negotiated, ordered, attributed and rejected.

A second correction concerns identifiers. Earlier package examples use strings such as `urn:signet:core:event:damage@1`. **PROPOSED:** do not standardize that form yet. RFC 8141 requires the URN namespace identifier to be registered; the IANA registry does not currently contain a `signet` namespace. For an early public protocol, identifiers under a domain Signet already controls are less ambiguous, for example:

~~~text
https://signetprotocol.io/sem/core/session/1
https://signetprotocol.io/sem/profile/spatial-3d/1
https://signetprotocol.io/sem/profile/fps-combat/1
~~~

Resolution may be useful, but identity is the exact URI string. A definition hash can bind the bytes of a particular published definition.

## Current Signet/1 baseline

**OBSERVED** at Signet Protocol main head `36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c`:

- transport is UTF-8 JSONL over TCP by default;
- `Hola` declares only `juego` and an optional scalar protocol `version`;
- `Bienvenida` returns a scalar protocol `version`, a mode string, terrain and optional native map;
- `Intencion` is FPS-shaped: forward/strafe/yaw/run/fire/use/weapon plus sequence;
- `Estado` is a full body snapshot plus tick events;
- protocol evolution is “add optional fields with defaults, ignore unknown fields; meaning changes require Signet/2”;
- version mismatch currently warns and keeps playing;
- numbered commands and authoritative state provide a strong deterministic base;
- the translator manifest describes broad importer/world/input/presentation capabilities but those capabilities are not a session negotiation contract.

This is a good beta transport and deterministic-authority foundation. The interoperability gap is not that the wire is insufficiently complicated; it is that **syntactic acceptance is currently doing work that should belong to explicit semantic negotiation**.

## Design invariants

### I1 — Parsing is not compatibility

A message can be valid JSON and valid Signet/1 while still carrying a concept the receiver cannot use correctly.

### I2 — Support is not activation

A participant may implement a profile without using it in a session. OpenXR makes the same distinction for extensions: available functionality must be queried, then explicitly enabled.

### I3 — Optional is not harmless

An optional field is backward compatible only when its absence has a defined meaning that does not silently alter the semantics needed for the current session.

### I4 — Shared identifiers must outlive human labels

English, Spanish, Japanese and other labels are documentation/localization. They must not be the identity of a shared concept.

### I5 — Same identifier/version means same definition

If two peers advertise the same semantic identifier and version but disagree on the canonical definition hash, negotiation must fail rather than guess which meaning is intended.

### I6 — Authority is a relation, not a sender claim

A message can say who issued it, but a participant must not become authoritative merely by including an `authority: true` field. The session contract decides which issuer is authorized for which semantic scope.

### I7 — Degraded operation is explicit

Fallback, observer-only and approximation must be represented in the negotiated result. A degraded mapping must not present itself as full semantic compatibility.

### I8 — Semantic and pragmatic compatibility are separate

LCIM-style semantic interoperability means both sides understand the meaning. Pragmatic interoperability additionally means the receiver can act on that meaning correctly in the current context. A viewer can understand `jump` yet legitimately be observer-only or fallback-only.

---

# Candidate core@1

## What the core owns

**PROPOSED:** `core@1` owns only the coordination semantics needed to assemble independently implemented profiles.

### 1. Semantic identifiers

Every normative concept/profile/ruleset has:

~~~yaml
semantic_id: https://signetprotocol.io/sem/profile/fps-combat/1
definition_hash: sha256:<canonical-definition-digest>
~~~

The hash is not the identifier. It is an integrity/equivalence check for the definition advertised under that identifier.

### 2. Session identity

~~~yaml
session_id: <opaque-session-id>
contract_id: <opaque-negotiated-contract-id>
~~~

A session binds one negotiated set of protocol/profile/ruleset semantics.

### 3. Participant identity

~~~yaml
participant_id: <session-local-or-global-opaque-id>
adapter:
  implementation_id: <stable implementation identifier>
  version: <implementation version>
~~~

This does not define authentication; that belongs to L3. It does prevent `juego=minecraft` from being overloaded as game identity, adapter identity, adapter version and semantic capability.

### 4. Semantic envelope category

The core distinguishes at least:

- `intent` — a requested action, not an authoritative effect;
- `state` — authoritative or observational state claim;
- `event` — a transition/effect claim;
- `ack` — acknowledgement of receipt/application;
- `control` — negotiation/session lifecycle.

Domain meaning still comes from a profile semantic ID.

### 5. Ordering and causal metadata

The core does **not** require one universal clock such as a fixed simulation tick.

Instead it binds each session to an explicit ordering model and provides stable semantic message/event identity plus optional causal references:

~~~yaml
message_id: <unique within session>
ordering:
  model: https://signetprotocol.io/sem/ordering/fixed-tick/1
  value:
    tick: 18442
    substep: 3
caused_by:
  - <message-or-event-id>
~~~

A turn-based session could instead negotiate:

~~~yaml
ordering:
  model: https://signetprotocol.io/sem/ordering/turn-sequence/1
  value:
    turn: 42
    phase: action
    action_index: 1
~~~

An event-driven session may need only a monotonic event sequence.

The core therefore standardizes **how an ordering model is selected and identified**, not the contents of every ordering model.

### 6. Issuer and authority scope reference

~~~yaml
issuer: participant:104
authority_scope: session:rules
authority_epoch: 7
~~~

The receiver evaluates these values against the negotiated/session authority table. `authority_epoch` prevents a late claim from a previous ownership interval being accepted after responsibility transfers.

### 7. Visibility / disclosure policy reference

Some domains contain authoritative facts that are not visible to every participant: a card hand, fog-of-war state, a secret objective, or private team information.

The core should provide a generic way for a semantic claim/state to reference the negotiated disclosure policy:

~~~yaml
visibility_policy_ref: <session/profile-defined-policy>
~~~

The core does **not** define concrete concepts such as "hand" or "fog of war." Those belong to profiles/rulesets. It only makes selective disclosure an explicit part of the contract rather than an undocumented transport side effect.

### 8. Capability/profile negotiation

The core defines how profiles are advertised, required, activated and rejected. Domain capability content belongs to the profile.

### 9. Typed incompatibility

The core defines deterministic compatibility outcomes and reason codes, so “can parse” does not become “can safely participate.”

## What the core does not own

Move these out of the universal core:

- transforms and coordinate frames;
- body/entity lifecycle;
- health/armour/ammo;
- move/look/fire/use;
- world geometry;
- voxel cells;
- inventory;
- economy;
- rendering/presentation;
- authentication/signatures;
- package/runtime policy.

This is the strongest delta from the first package.

---

# Candidate semantic profiles

## P1 — spatial-3d@1

Purpose: shared pose/space semantics for worlds that actually have a 3D spatial representation.

Candidate concepts:

~~~yaml
coordinate_frame:
  handedness: right
  up_axis: y
  forward_axis: negative_z
units:
  position: metre
  angle: radian
concepts:
  pose:
    position: [x, y, z]
    orientation: <yaw-only or quaternion capability>
  velocity: optional
  bounds: optional
~~~

Key rule: a profile must identify **what point** a position denotes. Feet position, collider origin, camera origin and model pivot are different semantics.

## P2 — entity-lifecycle@1

Purpose: identity and existence transitions for shared entities.

Candidate concepts:

- create/appear;
- present;
- remove/despawn;
- transfer/replace identity only when explicitly defined.

Important non-equivalence:

~~~text
remove entity != death
death != disconnect
disconnect != observer leave
respawn != create new identity
~~~

This profile exists specifically to keep generic lifecycle semantics separate from FPS death/respawn rules.

## P3 — fps-combat@1

Purpose: the current Signet beta gameplay domain, extracted from the universal protocol surface.

Candidate action concepts:

- move;
- look;
- run;
- fire;
- use;
- select weapon.

Candidate event/state concepts:

- shot;
- damage applied;
- death;
- respawn;
- health;
- armour;
- weapon selection;
- ammunition.

The profile must distinguish request/effect pairs:

~~~text
fire intent != shot emitted
damage request != damage applied
use intent != door opened
teleport request != authoritative position change
~~~

## P4 — voxel-world@1

Purpose: worlds represented as discrete addressable cells/blocks.

Candidate concepts:

- grid/world coordinate;
- cell identity;
- block/material semantic ID;
- block create/change/remove event;
- chunk/region addressing;
- deterministic conflict/order rule.

This profile should not be required merely because one participant is Minecraft.

---

# Profiles versus modes/rulesets

A **profile** defines vocabulary and semantic constraints.

A **mode/ruleset** defines normative behavior and state transitions.

For example:

~~~text
profile: fps-combat@1
ruleset: doom-deathmatch@1
~~~

The profile can say what `damage-applied` means and which fields it has. The ruleset can say how armour modifies damage, which weapons exist, movement speed, respawn timing and win conditions.

This distinction prevents a profile from becoming a disguised game implementation.

---

# Semantic identifier and versioning recommendation

## Recommended identifier shape

**PROPOSED:**

~~~text
https://signetprotocol.io/sem/{kind}/{name}/{major}
~~~

Examples:

~~~text
https://signetprotocol.io/sem/core/session/1
https://signetprotocol.io/sem/profile/spatial-3d/1
https://signetprotocol.io/sem/profile/fps-combat/1
https://signetprotocol.io/sem/ruleset/doom-deathmatch/1
https://signetprotocol.io/sem/event/damage-applied/1
~~~

Properties:

- globally scoped by a domain under project control;
- no IANA URN namespace is assumed;
- human labels can be localized separately;
- resolution can serve documentation/schema but is not required for runtime identity;
- meaning-changing revisions get a new major semantic identifier.

## Definition hash

Each published semantic definition should have a canonical representation and digest:

~~~yaml
semantic_id: https://signetprotocol.io/sem/event/damage-applied/1
definition_hash: sha256:...
~~~

Rules:

1. same ID + same major + different digest => `SEMANTIC_DEFINITION_CONFLICT`;
2. a documentation typo that changes no canonical definition does not require a new semantic version;
3. meaning/units/authority/category changes require a new semantic version;
4. an additive optional field may remain within the same semantic major only if absence preserves all required meaning for participants that do not use the field.

## Do not overload SemVer

Adapter/package versions can use SemVer.

Semantic contract versions should not rely on `1.4.7` range arithmetic to infer meaning compatibility. Negotiation should operate on exact advertised semantic identifiers plus explicitly declared compatibility.

---

# Capability model

A participant should advertise three different facts:

1. **implemented** — the adapter knows the semantic definition;
2. **operable** — it can perform specific roles under the current local integration;
3. **required** — the session cannot satisfy this participant without the capability.

Example:

~~~yaml
profiles:
  - id: https://signetprotocol.io/sem/profile/fps-combat/1
    definition_hash: sha256:...
    implemented: true
    roles:
      publish:
        - intent/move
        - intent/fire
      consume:
        - state/body
        - event/damage-applied
        - event/death
      observe:
        - event/respawn
    required:
      - state/body
    optional:
      - event/damage-direction
    fallbacks:
      event/damage-direction: ignore
~~~

This is intentionally richer than four manifest booleans but remains declarative.

---

# Authority model

## Session authority table

**PROPOSED:** authority is selected by the session/ruleset and distributed as part of the negotiated contract.

Example:

~~~yaml
authority:
  intent/player:
    issuers: [participant:self]
  state/body:
    issuers: [server:rules]
  event/damage-applied:
    issuers: [server:rules]
  presentation/local:
    issuers: [participant:self]
~~~

For current Signet this preserves the existing architecture:

- clients publish intent;
- authoritative rules/server owns position, hits, damage, death and respawn;
- local translators own presentation;
- model suggestions own nothing.

## Responsibility transfer

When an authority scope can move, transfer increments an epoch:

~~~yaml
scope: world:object:42
from: server:a
to: server:b
new_epoch: 8
effective_tick: 21000
~~~

Claims from epoch 7 at or after the transfer boundary are stale even if they arrive later.

The cryptographic/authentication mechanism proving issuer identity is an L3 concern.

---

# Time and causality

Do not import the full HLA time-management surface into a beta game protocol.

Instead, make the **time model explicit**.

Candidate ordering models:

~~~text
fixed_tick
turn_sequence
logical_step
event_sequence
~~~

For current Signet:

~~~yaml
ordering_model:
  id: fixed_tick
  tick_hz: 20
  command_application: exactly_once_when_seq_positive
  late_event_policy: reject_or_reconcile
  wall_clock_authoritative: false
~~~

For a turn-based game:

~~~yaml
ordering_model:
  id: turn_sequence
  phases: [start, action, resolution, end]
~~~

Authoritative events should have stable IDs and enough model-specific ordering context for replay:

~~~yaml
event_id: evt:93b...
ordering:
  model: fixed_tick
  value:
    tick: 18442
    sequence: 3
caused_by:
  - intent:104:1042
~~~

TCP ordering is useful but should not be the semantic definition of causality. Replay files and future transports must be able to preserve the same order independently.

---

# Failure and degradation semantics

Negotiation produces one of:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

Every non-`SUPPORTED` result carries machine-readable reasons.

Candidate reason codes:

~~~text
PROTOCOL_MAJOR_MISMATCH
SEMANTIC_DEFINITION_CONFLICT
REQUIRED_PROFILE_UNSUPPORTED
REQUIRED_CONCEPT_UNSUPPORTED
AUTHORITY_MODEL_UNSUPPORTED
TIME_MODEL_UNSUPPORTED
WORLD_REPRESENTATION_UNSUPPORTED
NO_SAFE_FALLBACK
LEGACY_SEMANTICS_ONLY
~~~

A participant may understand a profile but still be `OBSERVE_ONLY` because it cannot perform the required pragmatic role.

---

# Contradiction table against current Signet/1 assumptions

| Current Signet/1 behavior | L1 finding | Proposed consequence |
|---|---|---|
| Version mismatch warns and continues | syntactic version mismatch is not enough evidence for semantic compatibility | negotiate before session admission; reject required semantic mismatches |
| Unknown fields are ignored | safe only for genuinely optional meaning | distinguish used/required capabilities and explicit defaults |
| Optional fields carry defaults | absence can collapse important semantic distinctions | define absence semantics per profile; presence-sensitive concepts need explicit support |
| `Hola` carries game id + scalar version | insufficient to establish participant capability | add capability/profile advertisement |
| `juego` identifies the game | does not identify adapter implementation/version/capabilities | separate game label, adapter implementation and semantic profile |
| `Intencion` is universal protocol vocabulary | it is FPS-domain vocabulary | move into `fps-combat@1` |
| `Posicion` is a compatibility escape | it bypasses normal movement semantics | advertise as explicit legacy/fallback capability |
| `modo` is a free string | lacks definition identity/equivalence check | ruleset semantic ID + definition hash |
| fixed 20 Hz is implicit protocol behavior | valid for current rules but not universal | advertise/activate session time model |
| `seq=0` legacy behavior coexists with numbered commands | two causality/application models share one surface | negotiate legacy vs numbered-command semantics explicitly |
| Spanish field names are fixed for Signet/1 | human-language labels are not a scalable semantic identity mechanism | preserve /1 compatibility; use language-neutral semantic IDs in next contract |
| current conformance is primarily terrain/rules/manifest structure | structure does not prove shared meaning or usable behavior | add semantic/pragmatic conformance properties |

---

# Standards findings

## HLA / OMT

IEEE 1516-2025 defines HLA as a common architecture for distributed simulations. IEEE 1516.2-2025 explicitly defines the **format and syntax, but not the content**, of HLA object models.

**DERIVED:** HLA is useful precedent for federation, object model, publication/subscription, ownership and time concepts, but merely adopting an HLA-like schema does not solve the meaning problem.

SISO's public data files include modular FOM examples. HLA Evolved modular FOMs support independently maintained pieces of a federation object model.

**Design delta:** Signet profiles should be independently versioned/composable modules rather than one global gameplay ontology.

## OpenXR

OpenXR 1.1 explicitly requires applications to query available extensions and then enable a chosen subset.

**Design delta:** capability support and session activation are separate states.

OpenXR extension naming also uses controlled namespaces to avoid collisions.

**Design delta:** do not use bare human vocabulary as a global concept identifier.

## glTF

glTF separates `extensionsUsed` from `extensionsRequired`; required extensions are a subset of used extensions.

**Design delta:** Signet negotiation should distinguish “I can use this if available” from “I cannot participate correctly without this.”

glTF minor-version rules also require backward/forward compatible additions not to change existing behavior.

**Design delta:** adding a syntactically optional field is not enough; existing semantic behavior must remain unchanged.

## LCIM / semantic-pragmatic distinction

LCIM distinguishes technical/syntactic interoperability from semantic and pragmatic interoperability.

**Design delta:** report “understands meaning” separately from “can correctly act on meaning.” `OBSERVE_ONLY` is a valid outcome rather than a failure to parse.

---

# Native-language / recursive discovery delta

This pass did not find a better replacement for the already-captured Spanish SCFHLA source, but it sharpened its relevance:

- SCFHLA explicitly says an HLA FOM provides the syntactic contract while the values/concepts still need semantic agreement.
- It favors a **network of modular ontologies** over one monolithic ontology to support maintenance, collaboration and domain separation.

This independently reinforces the profile/module direction rather than a single giant “game ontology.”

No material new Chinese/Japanese source was accepted in this pass; searches mostly rediscovered already-ledgered HLA adapter literature. Record that as **no-delta evidence**, not as absence of relevant work.

---

# Recommended next contract shape

~~~text
Signet/1
  transport + current beta vocabulary
        |
        | bridge / compatibility adapter
        v
semantic core@1
  identifiers
  versions + definition hashes
  session/participant identity
  envelope categories
  ordering/causality
  authority references
  capability negotiation
  typed incompatibility
        |
        +-- spatial-3d@1
        +-- entity-lifecycle@1
        +-- fps-combat@1
        +-- voxel-world@1
        |
        +-- ruleset doom-deathmatch@1
~~~

Do not rewrite Signet/1 merely to make the model aesthetically pure. A bridge can expose current Signet/1 as a legacy profile/ruleset combination while independently built adapters prove the next semantic contract.

## L1 conclusion

The minimum shared contract should be **smaller** than the current draft core, not larger.

The protocol must standardize how meanings are identified, selected, versioned, ordered, authorized and rejected. Gameplay meaning should live in modular profiles. Rulesets should then compose those profiles into deterministic shared behavior.

That architecture preserves Signet's strongest property — a deterministic authoritative world — while making interoperability explicit enough for independent implementations.
