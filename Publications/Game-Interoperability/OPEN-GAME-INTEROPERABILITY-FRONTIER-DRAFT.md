# From Game Mashups to an Open Interoperability Stack

**Unaffiliated public draft — 2026-10-05**

A small group of developers trying to make old and new games talk to each other can look like a niche modding project.

It probably is not.

The same technical problem appears in distributed simulation, digital twins, healthcare data, robotics, XR hardware, 3D asset pipelines, and open-metaverse standards:

> How do independently built systems cooperate without one of them swallowing all the others?

Two current game projects make the question concrete.

**Signet Protocol** is building a neutral server and translator model so players using different games can participate in the same world. **Melty** is building a one-click product for finding, installing, launching, creating, and sharing game mashups.

They overlap, but they are not solving the same layer.

That distinction matters.

---

## Signet's core idea is stronger than ordinary crossplay

Normal crossplay connects different devices to the same game.

Signet proposes something more ambitious: the server owns shared geometry, rules, bodies, and events, while each player can use a different game as a viewer/controller through a translator.

Its current beta already implements a meaningful slice:

- neutral terrain;
- intent messages;
- authoritative state;
- combat events;
- client prediction and reconciliation;
- translators for world/input/presentation;
- manifests;
- conformance checks;
- multiple bindings;
- a dedicated server.

The most important Signet design decision is not JSON, Rust, or port 7777.

It is the separation between:

```text
what must be shared
and
what may remain local
```

Walls, hits, damage, body position, and rules need common truth.

Textures, sounds, camera, HUD, controls, and animation often do not.

That boundary is the beginning of a real interoperability architecture.

---

## Melty solves a different problem: humans hate setup

Protocol developers routinely underestimate installation.

Users do not want to learn:

- which loader they need;
- which game version works;
- where the game is installed;
- which dependencies belong beside which mod;
- which process starts first;
- how to restore the original game.

Melty's value proposition is brutally simple: press Play.

It detects local games, installs what a mashup needs, launches it, supports multiplayer links, and gives coding agents a path to create/publish mashups.

That is a product layer an open protocol ecosystem eventually needs.

But it is not a substitute for a neutral runtime contract.

A mashup can be brilliant and still be one bespoke integration. A protocol becomes valuable when two people can build independent adapters and prove they mean the same thing.

The likely future stack is therefore not “Signet or Melty.”

It is closer to:

```text
open protocol/runtime
        +
translator ecosystem
        +
safe package/launcher UX
        +
optional AI creation/debugging
```

---

## The hard problem is semantics, not networking

Once enough games participate, serialization stops being the difficult part.

Consider a field called `health`.

Does it mean:

- raw hit points;
- percentage;
- pre-armour health;
- post-armour health;
- maximum-normalized health;
- a local display value?

Or an event called `death`.

Is it:

- a gameplay kill;
- entity removal;
- ragdoll start;
- respawn timer start;
- permanent character deletion?

Two programs can parse the same JSON and still disagree completely.

Distributed-simulation researchers have names for this:

1. **syntactic interoperability** — we agree on the structure;
2. **semantic interoperability** — we agree on the meaning;
3. **pragmatic interoperability** — we can use it correctly in context.

That three-level framing appears especially clearly in Spanish-language HLA/ontology research.

A translator that can deserialize an event is not necessarily compatible.

---

## This problem has decades of prior art

The games community should steal shamelessly from simulation engineering.

### HLA

The IEEE High Level Architecture treats independent simulations as **federates** participating in a **federation** with common object/interaction models and runtime services.

That is remarkably close to the useful mental model for game translators.

A Doom adapter and a Minecraft adapter do not need to understand each other directly.

They need to understand the federation contract.

### DIS

Distributed Interactive Simulation defines explicit protocol families for entities and interactions.

The lesson is simple: realtime interoperability needs a disciplined event/state model, not only a socket.

### glTF and OpenUSD

These standards attack 3D data interoperability.

They are useful precisely because asset exchange is **not** the same problem as gameplay semantics.

A game interoperability stack should reuse them where appropriate instead of reinventing every representation.

### OpenXR

OpenXR is a good example of reducing fragmentation with a common core, capability discovery, and extensions.

That pattern fits heterogeneous games better than an ever-growing universal enum.

---

## The multilingual literature adds things English search misses

A native-language research pass changed the architecture in several places.

### China: game-engine adapters are a current simulation problem

A 2024 Chinese paper describes a plugin-based Unreal Engine adapter for HLA distributed simulation.

That is a direct precedent for treating an engine as a reusable federation participant instead of building a custom pairwise bridge.

### Japan: adapter tooling was already being generated two decades ago

Japanese HLA work from the early 2000s describes Manufacturing Adapters, graphical definitions of exchanged information, support tooling, and automatic generation of HLA glue code.

That is almost exactly the right target for modern AI assistance:

> do not ask AI to invent the interoperability contract; ask it to implement a strict contract and then run conformance tests.

### Spanish: syntax is not semantics

Spanish-language distributed-simulation work builds ontology networks above HLA so independent simulators agree on objects, events, interactions, and metrics.

That argues for semantic profiles rather than one giant wire vocabulary.

### German: do not build pairwise mappings

German semantic-interoperability work provides a useful mediator pattern: do not directly map every standard to every other standard. Map each to a shared application ontology.

For games, this is the difference between:

```text
N games -> N adapters
```

and:

```text
N games -> N² bridges
```

### French: not every mapping must be permanent

French work on federated interoperability includes contextual or “ephemeral” ontologies.

That is a useful antidote to protocol bloat.

A temporary compatibility mapping can exist for one scenario/version without becoming eternal core vocabulary.

### Russian and Brazilian work: the knowledge model itself needs provenance

Russian knowledge-engineering literature emphasizes declarative domain models and knowledge graphs.

Brazilian semantic-interoperability work adds a warning: declaring an ontology does not make it perspective-free.

A good game protocol should preserve where a mapping came from and which source-game concept it represents.

---

## So what is the “universal language”?

Probably not one language.

A scalable design looks more like:

```text
small stable core
+ namespaced semantic profiles
+ explicit mappings
+ capabilities
+ conformance fixtures
```

For example:

```text
core:
  transform
  identity
  lifecycle
  time

profiles:
  fps
  platformer
  vehicle
  inventory
  voxel
  rpg-stats
```

Each game implements the profiles it understands.

A session negotiates what is actually shared.

That is more honest than claiming universal compatibility while one participant silently drops half the mechanics.

---

## AI belongs at the edge

The current wave of small decision models—projects such as Laya and CLM—makes the idea of a “very smart if” concrete.

These models can cheaply select or rank from a closed candidate set.

Useful jobs include:

- selecting a valid fallback;
- suggesting an event mapping;
- ranking compatibility fixes;
- triaging install failures;
- selecting conformance tests;
- helping generate adapters.

Bad jobs include:

- authoritative position;
- hits;
- damage;
- inventory;
- score;
- authentication;
- protocol interpretation.

The rule should be:

```text
model suggests
contract validates
server decides
```

not:

```text
model decides reality
```

A generated mapping can be accepted, versioned, tested, and shipped. The runtime should not need the same model to improvise the meaning again every frame.

---

## A compatibility database could become extremely valuable

Instead of a wiki saying “works,” record evidence.

For every translator:

- game version;
- translator version;
- protocol version;
- profiles;
- OS/platform;
- integration mode;
- conformance suite version;
- exact results;
- fallbacks;
- known failures.

That does three jobs at once.

It helps users.

It gives maintainers reproducible bug reports.

And later it becomes clean training data for local compatibility models.

---

## The launcher must treat adapters as supply-chain software

One-click mashups may:

- download tools;
- write into game directories;
- build code locally;
- run programs;
- connect peers.

That is serious authority.

An open launcher should eventually have:

- signed manifests;
- hashes;
- dependency locks;
- declared permissions;
- restricted working directories;
- network controls;
- reproducible conformance;
- explicit verification labels.

Do not use one badge called “verified” to mean six different things.

“Signed by publisher” is not “safe.”

“Static scan passed” is not “legal.”

“Protocol conformant” is not “sandboxed.”

---

## Privacy can be local-first

Most game-specific information never needs to leave the machine.

Keep local:

- game file contents;
- installed-game inventory;
- local paths;
- raw screen/video;
- saves;
- detailed model traces.

Transmit only neutral session data.

If telemetry is useful, make it narrow and opt-in.

That is both a privacy win and an architectural simplification.

---

## The next milestone should be smaller than “all games”

A credible open interoperability movement does not need 100 games next month.

It needs one thing outsiders can independently verify.

A strong target is:

> **A versioned adapter contract and conformance suite where two independently written translators prove they agree on one minimal shared world/event profile.**

Then add a third meaningfully different integration.

Then generate adapter scaffolding from the schema.

Then add a compatibility registry.

Then experiment with decision models.

The project becomes credible through repeated interoperable implementations, not through the size of the vocabulary.

---

## The opportunity

There is a real gap between today's worlds.

On one side are modding communities: creative, fast, fragmented, game-specific.

On another are simulation standards: rigorous, interoperable, often heavy and inaccessible to hobbyists.

On another are open 3D standards: excellent at scenes/assets, deliberately incomplete as gameplay protocols.

And now there are local AI models that can cheaply help build and route adapters.

A good open project can sit in the middle.

Not by replacing all those systems.

By connecting the useful pieces:

```text
open semantics
deterministic shared state
small adapters
existing asset/device standards
safe packaging
machine-readable conformance
local adaptive helpers
```

If that works, “every game, one world” stops being a slogan and becomes a testable engineering claim.
