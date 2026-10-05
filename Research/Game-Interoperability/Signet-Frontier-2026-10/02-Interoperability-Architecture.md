# 02 — Interoperability Architecture

## Thesis: deterministic core, adaptive edge

A cross-game network needs one non-negotiable property:

> Participants must be able to determine what shared-world state means without trusting hidden model reasoning.

AI can still provide advice, ranking, mapping, or generated candidate code. It should not become undeclared authority over shared state.

## Interoperability levels

A useful synthesis from distributed-simulation and semantic-interoperability research:

### Level 0 — transport

Can components exchange messages?

### Level 1 — syntax

Do both sides agree on structure/type?

### Level 2 — semantics

Do both sides mean the same thing?

Examples:

- feet position vs camera position vs collider origin;
- raw damage vs post-armour damage;
- “use” button state vs completed interaction.

### Level 3 — pragmatics

Can a participant correctly act on that meaning in this context?

A viewer may understand a jump event but be unable to render/perform it under the current mode.

**PROPOSED:** Signet should expose these levels explicitly. Parsing is not compatibility.

## Proposed stack

```text
L8  Launcher / package / discovery
L7  Trust, identity, signatures, policy
L6  Assets and local presentation
L5  Adapter implementation
L4  Capability negotiation + semantic profiles
L3  Shared object/event/rule semantics
L2  Session protocol + transport
L1  World/session identity + version
L0  Local game integration mechanism
```

Signet's five layers sit mostly across L3-L6. The added layers become necessary at ecosystem scale.

## Translators as federates, not pairwise bridges

Anti-pattern:

```text
Doom <-> Minecraft
Doom <-> OpenArena
Doom <-> Game X
Minecraft <-> OpenArena
...
```

That becomes N² integration.

Scalable pattern:

```text
Doom      -> neutral semantic contract
Minecraft -> neutral semantic contract
OpenArena -> neutral semantic contract
Game X    -> neutral semantic contract
```

This is close to HLA's federate/federation model.

A mature translator manifest should eventually declare:

- publications;
- subscriptions;
- supported actions;
- supported state/event classes;
- timing assumptions;
- authority assumptions;
- presentation-only features;
- required local resources;
- deterministic limitations.

## Use semantic IDs, not a human language as meaning

Changing Spanish wire names to English may improve accessibility, but natural-language vocabulary should not be the semantic identity mechanism.

**PROPOSED:**

```text
urn:signet:core:event:damage@1
urn:signet:core:event:death@1
urn:signet:core:action:move@1
urn:signet:core:state:body@1
urn:signet:mode:doom-deathmatch@1
```

Human labels can be localized without breaking protocol identity.

## Modular profiles, not one giant ontology

A practical structure:

```text
core/
  space
  time
  identity
  body
  transform
  lifecycle

profiles/
  fps
  platformer
  vehicle
  inventory
  rpg-stats
  voxel-world
  physics
  dialogue
  economy

modes/
  doom-deathmatch
  capture-the-flag
  racing
  co-op-survival
```

Each profile defines concepts, units, states, events, actions/intents, invariants, extensions, conformance vectors, and mappings.

A translator declares which profiles it implements.

## Capability negotiation

Current importer/world/input/presentation booleans are useful but too coarse for a large ecosystem.

A richer descriptor could express:

```json
{
  "protocol": ["signet/1"],
  "profiles": {
    "urn:signet:profile:fps@1": {
      "actions": ["move", "look", "fire"],
      "events": ["shot", "damage", "death", "respawn"],
      "limits": {
        "vertical_movement": false,
        "max_players": 32
      }
    }
  },
  "world": {
    "geometry": ["heightfield-2.5d"]
  },
  "presentation": {
    "damage_direction": true,
    "spatial_audio": false
  }
}
```

A session derives:

- `SUPPORTED`;
- `SUPPORTED_WITH_FALLBACKS`;
- `OBSERVE_ONLY`;
- `INCOMPATIBLE`.

The system should explain why.

## Explicit authority

Every field/event should answer: **who may assert this?**

| Data | Authority |
|---|---|
| input intent | player's authenticated session |
| body position | authoritative rules/server |
| hit/damage result | authoritative rules/server |
| local animation | translator |
| texture choice | translator/local user |
| translator capability manifest | translator package/publisher |
| compatibility result | deterministic negotiation |
| model suggestion | never authoritative by itself |

Information and authority are different data types.

## Time and causality

Signet already uses numbered commands and acknowledgements. Preserve that discipline.

Authoritative events should carry enough context for replay/debugging:

```yaml
tick: 18442
event_id: ...
event_type: urn:signet:core:event:damage@1
actor: body:104
target: body:117
cause_event: ...
ruleset: urn:signet:mode:doom-deathmatch@1
ruleset_hash: ...
```

Do not use wall-clock timestamps as the simulation ordering primitive.

## Transport should be replaceable

JSON-lines/TCP is excellent for a beta because it is easy to inspect.

Keep it.

But specify semantics independently enough that later transports can exist and pass the same conformance vectors:

- TCP JSONL;
- QUIC;
- WebTransport;
- local IPC/shared memory;
- replay/log files.

## Reuse existing standards at their layer

```text
Signet semantics      -> gameplay/world/event contract
glTF                  -> portable runtime assets where useful
OpenUSD               -> complex scene/world composition where useful
OpenXR                 -> XR device/input abstraction
URI/Web concepts      -> world/session addressing
HLA concepts          -> federation/time/ownership/model discipline
DIS concepts          -> realtime entity/event protocol families
```

Do not make one project responsible for every layer.

## Compatibility registry

A compatibility database should contain evidence, not stars:

```yaml
game: openarena
translator: signet-openarena@0.1.0
tested_game_version: 0.8.8
protocol: signet/1
profiles:
  fps@1: PASS
conformance:
  suite: signet-conformance@<hash>
  result: PASS
known_failures:
  - shader-texture fidelity
source:
  repository: ...
  commit: ...
```

That creates a path for community collaboration before everyone agrees on all architecture prose.
