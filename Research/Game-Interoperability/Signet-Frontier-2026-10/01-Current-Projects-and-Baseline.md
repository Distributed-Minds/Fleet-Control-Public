# 01 — Current Projects and Baseline

## Signet Protocol

**OBSERVED at `kian-cx/signetprotocol` main `36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c` at the research cutoff.**

Signet describes itself as an open protocol and SDK for players using different games to join the same world.

Current Signet/1 characteristics include:

- TCP, port 7777 by default;
- UTF-8 JSON, one message per line;
- 20 Hz authoritative server state;
- client hello, intent, and compatibility position messages;
- server welcome/state messages;
- numbered client commands for prediction/reconciliation;
- neutral terrain, bodies, and events;
- Rust SDK, C FFI, preview C# and TypeScript bindings;
- Dockerized dedicated server;
- translator manifests;
- a conformance suite.

The project is Apache-2.0 licensed.

### Five-layer decomposition

Signet separates:

1. **World** — shared geometry/collision, server-selected.
2. **Rules** — shared mechanics/physics/combat, server-selected.
3. **Viewer** — per-player rendering engine.
4. **Appearance** — per-player textures/models/sounds.
5. **Input and feel** — per-player controls/camera/rumble/presentation.

That decomposition is strong because it distinguishes facts that must be shared for a coherent match from presentation that can legitimately differ.

### Translator model

A translator may implement any subset of:

- importer: game → neutral world;
- world translator: neutral world → game scene;
- input translator: local input/state → neutral intent;
- presentation translator: neutral bodies/events → local presentation.

Current integration modes are:

- gateway;
- open engine;
- official/permitted mod;
- reimplementation reading player-owned files.

The manifest declares game ID, translator version, protocol version, integration mode, capabilities, supported modes, required local files, and translator license.

### Conformance

Current published checks cover terrain validity, source/material namespacing, spawn validity, deterministic movement, basic rule validity, and translator-manifest structure.

That is the right direction: interoperability must be testable rather than social consensus alone.

### Versioning

Signet/1 is additive:

- new fields are optional with defaults;
- unknown fields are ignored;
- names do not change;
- incompatible semantic/type/name changes require Signet/2.

The Signet/1 wire vocabulary is Spanish. Signet/2 is currently discussed as potentially using English vocabulary.

### Security and governance

The current security policy explicitly states that the reference beta server has **no authentication** and should not be Internet-exposed unless the operator accepts anyone being able to join.

Governance currently has one maintainer, with protocol proposals discussed for at least 14 days and Signet/1 changes restricted to additive evolution.

**DERIVED:** the project is technically beyond a pure concept, but institutionally early enough that compatibility rules can still be formalized before scale makes them expensive to retrofit.

---

## Melty

**OBSERVED from `melty.gg`, Terms of Service, and Privacy Policy on 2026-10-05.**

Melty is solving a different layer:

- discover game mashups/mods;
- one-click install/setup/launch;
- multiplayer invite links;
- AI-assisted creation/publishing;
- coding-agent and GitHub integration;
- use games the player owns.

Its FAQ defines a mashup as a mod that joins games into one.

### Why Melty matters to an open protocol

Melty demonstrates that installation and compatibility UX are first-class product problems. A protocol can be elegant and still fail adoption if users must manually assemble mod loaders, versions, dependencies, paths, launch order, and rollback.

### Important safety wording mismatch

The homepage uses phrases such as:

- “Every upload is checked for dangerous files.”
- “Play verified mashups & mods for free.”

The Terms are more limited:

- mashups are not checked for quality, safety, ownership, or legality before appearing;
- automated checks reject some dangerous file names/types, not file behavior;
- mashup programs run with ordinary program access on the user's PC;
- changing games may corrupt saves, conflict with anti-cheat, or cause bans.

**DERIVED:** future open launchers should avoid one broad “verified” label. Prefer typed claims such as signature verified, static scan passed, install smoke test passed, protocol conformance passed, publisher identity verified, or reproducible build verified.

### Privacy

Melty currently says it may store account/profile data, activity, friend/block/report information, published mashup data, install/play reports, game version, multiplayer IP/port for active games, crash reports, and website analytics.

It also says installed-game lists and game files are inspected locally and are not uploaded.

**DERIVED:** the useful design rule is **keep discovery/content inspection local unless a network feature genuinely needs central data**.

---

## Relationship

| Problem | Signet | Melty |
|---|---|---|
| Shared cross-game runtime semantics | Core goal | Mashup-specific |
| Neutral authoritative world | Yes | Not its general architecture |
| Game adapters/translators | Core abstraction | Mashup/mod code |
| Protocol conformance | Explicit | Product/install oriented |
| One-click setup | Ecosystem gap | Core product strength |
| Mod distribution | Not primary | Core product strength |
| AI-assisted creation | Not core | Core product feature |
| Multiplayer discovery/invites | Basic server model | Productized |
| Open protocol | Yes | App/service governed by proprietary Terms |
| Local game ownership | Explicit principle | Required by product |

Useful synthesis:

```text
protocol/runtime layer      -> Signet-like
adapter ecosystem           -> shared
package/install/discovery   -> Melty-like UX
AI creation/debugging       -> optional tool layer
```

---

## Adjacent standards

### HLA — IEEE 1516

The 2025 HLA revision defines a common architecture for interconnecting distributed simulations, with federates participating in federations through runtime infrastructure and shared object models.

Relevant concepts:

- federate/federation separation;
- publish/subscribe;
- object/interaction models;
- time coordination;
- ownership/authority of shared state.

### DIS — IEEE 1278

Distributed Interactive Simulation defines interoperable protocol data units for real-time simulation domains.

Relevant lesson: define explicit event/entity families and behavior, not only serialization.

### glTF

glTF is an API-neutral runtime 3D asset delivery format with an extension model.

Relevant lesson: **asset interoperability is a separate layer from simulation semantics**.

### OpenUSD

OpenUSD Core Specification 1.0 provides an open standard for large-scale 3D composition/data interchange.

Relevant lesson: world/scene composition can use an existing ecosystem rather than being embedded wholesale into a gameplay protocol.

### OpenXR

OpenXR addresses device/runtime fragmentation with a common API plus extensions.

Relevant lesson: a small core + capability discovery + extensions can scale across heterogeneous implementations.

### Metaverse Standards Forum / Web of Worlds

The 2026 Web of Worlds direction proposes addressable, linked spatial worlds using Web-like identity/navigation principles.

Relevant lesson: stable world identity/discovery should be separate from the wire protocol used inside a session.

---

## Baseline problem statement

Cross-game interoperability is not one problem. It includes:

```text
package discovery
installation
game ownership / local content
process launch
transport
session identity
shared world geometry
shared simulation time
rules
entity/state model
event model
input semantics
capability negotiation
semantic mapping
asset/presentation mapping
security
privacy
licensing/EULA constraints
governance/versioning
conformance
observability/debugging
```

Calling all of that “the universal language” eventually becomes too ambiguous to test.
