# 06 — Recommendations and Roadmap

These recommendations are deliberately incremental. Signet should not become HLA, a metaverse stack, a package manager, and an AI platform all at once.

## Preserve the good beta properties

Keep:

- authoritative server;
- intent-based control;
- deterministic shared rules;
- neutral network data;
- local owned-game files;
- translator abstraction;
- open protocol/SDK;
- additive Signet/1 evolution;
- conformance tests.

Do not destabilize the working core merely because the frontier is large.

---

## R1 — Make capability negotiation a protocol feature

Promote a session-safe subset of translator metadata into handshake negotiation.

The server should know before play:

- translator protocol version;
- semantic profiles;
- required actions;
- supported events;
- world representation support;
- observer ability;
- known fallbacks.

Return a typed compatibility result:

```text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
```

---

## R2 — Use namespaced semantic IDs

Keep a tiny core. Add profiles.

Examples:

```text
urn:signet:core:action:move@1
urn:signet:core:event:damage@1
urn:signet:profile:fps:weapon-fire@1
urn:signet:profile:voxel:block-change@1
```

Human labels may be English, Spanish, Japanese, German, etc. Wire meaning should not depend on that label.

---

## R3 — Add semantic and pragmatic conformance

Current tests correctly validate structure and deterministic movement.

Next tests should cover:

- event meaning;
- unit conversion;
- edge cases;
- fallback behavior;
- unsupported-feature handling;
- presentation acknowledgement where claimed;
- round-trip mapping where meaningful.

Example result:

```yaml
fps@1:
  syntactic: PASS
  semantic: PASS
  pragmatic: PASS_WITH_FALLBACKS
```

---

## R4 — Generate adapter scaffolding from profiles

Given a profile, generate:

- Rust trait stubs;
- C ABI stubs;
- C# interfaces;
- TypeScript types;
- manifest template;
- conformance skeleton;
- replay fixtures.

Then AI can fill implementation details without inventing the contract.

This directly matches the most useful Japanese/HLA prior art.

---

## R5 — Require independent implementations before stabilizing a new profile

A profile implemented by one game may encode that game's assumptions.

Before declaring a new stable core/profile:

- two independent game/engine integrations implement it;
- cross-play fixture passes;
- fallback behavior is exercised;
- ambiguous semantics are resolved.

---

## R6 — Keep Signet/1 transport simple

JSONL/TCP is debuggable and appropriate for beta.

Instead of premature transport optimization, define canonical semantics and test vectors so a future second transport can be proven equivalent.

---

## R7 — Add authentication before public Internet hosting

A future public-server profile should define:

- server identity;
- session authentication;
- replay protection;
- rate limits;
- client/translator version;
- optional player identity;
- ban/revocation behavior.

Do not mix player identity with proof of game ownership.

---

## R8 — Build a compatibility evidence registry

Store evidence:

- translator version;
- game version;
- protocol/profile versions;
- suite commit/hash;
- exact result;
- fallbacks;
- known failures;
- OS/platform;
- integration mode.

This registry helps users now and creates future training data for local compatibility models.

---

## R9 — Make replay a first-class artifact

A replay should include:

- initial world/rules state;
- input intents;
- expected authoritative state/events.

Use it for:

- regressions;
- SDK parity;
- differential tests;
- adapter debugging;
- deterministic benchmarks.

---

## R10 — Treat AI output as candidate configuration

When a model proposes:

```text
local event X -> Signet event Y
```

store the accepted mapping explicitly.

Do not require the model at runtime forever.

---

## R11 — Separate protocol governance from package-directory governance

Protocol governance owns:

- semantic contracts;
- versions;
- conformance.

Directory/launcher governance owns:

- listings;
- signatures;
- publisher identity;
- package metadata;
- install UX.

A registry dispute should not require a protocol fork.

---

## R12 — Learn from Melty's one-click UX without inheriting ambiguous trust

A launcher may eventually handle:

- game detection;
- translator selection;
- version matching;
- dependency install;
- safe local configuration;
- server discovery;
- invite links;
- uninstall/rollback.

But verification labels must remain specific and auditable.

---

## R13 — Publish a threat model

At minimum cover:

- malicious client;
- malicious translator;
- malicious server;
- malicious world/package;
- compromised dependency;
- incompatible game update;
- protocol downgrade;
- replay/duplicate commands;
- hostile public lobby;
- filesystem overreach.

---

## R14 — Defer neutral-foundation complexity until there are independent implementers

A foundation becomes valuable when there are:

- several maintained third-party translators;
- maintainers from independent projects;
- protocol proposals from multiple engines;
- outsiders using the conformance suite.

Until then, lightweight open governance is probably enough.

---

# Experiment Roadmap

## E1 — Independent translator triangle

Use three meaningfully different integration styles, e.g.:

- Doom reimplementation;
- OpenArena reimplementation/open-engine path;
- Minecraft gateway.

Success criteria:

- same authoritative replay;
- same body lifecycle;
- same damage/death ordering;
- viewer-specific presentation allowed;
- no game files cross between participants.

---

## E2 — Capability matrix

Create deliberately incomplete translators:

- no spatial audio;
- no jump;
- observer-only;
- no material mapping;
- input-only.

Expected outcome must be deterministic:

```text
FULL
FALLBACK
OBSERVER
REJECT
```

---

## E3 — Semantic ambiguity corpus

Seed similar-looking but different concepts:

- remove entity vs death;
- damage requested vs damage applied;
- button pressed vs door opened;
- teleport requested vs authoritative position changed;
- ammo consumed vs shot fired.

This becomes a benchmark for adapters and model-assisted mapping.

---

## E4 — Generated adapter scaffold

Input:

- machine-readable profile;
- local game API description.

Compare:

- hand-written adapter;
- pure template generation;
- LLM generation;
- template + LLM completion.

Measure:

- compile success;
- conformance pass rate;
- time to first working adapter;
- semantic bugs;
- unsafe host access introduced.

---

## E5 — Decision-model fallback selector

Benchmark Laya/CLM/simple rules/general LLM on a held-out Signet-specific compatibility set.

Measure:

- exact-choice accuracy;
- calibration;
- abstention quality;
- latency;
- multilingual metadata behavior;
- out-of-domain behavior.

Use project benchmarks only as hypotheses, not results.

---

## E6 — HLA bridge spike

Build the smallest possible bridge:

```text
Signet session <-> HLA federate
```

Map only:

- entity transform;
- lifecycle;
- one interaction/event.

Purpose: identify what maps cleanly, what is underspecified, and what HLA complexity games do not need.

---

## E7 — Replay parity across SDKs

Run identical golden replays through every SDK/runtime implementation.

Expected output must match where deterministic semantics are implemented.

---

## E8 — Wire fuzzing

Targets:

- arbitrary bytes never crash parser;
- unknown optional fields preserve Signet/1 compatibility;
- numeric extremes handled consistently;
- duplicate/out-of-order sequence behavior explicit;
- oversized messages bounded.

---

## E9 — Package sandbox

Use a deliberately overreaching test translator that attempts:

- home-directory read;
- SSH key read;
- browser-cookie read;
- arbitrary outbound network;
- undeclared child process;
- path traversal.

The launcher should deny or visibly prompt based on declared permissions.

---

## E10 — Game-update drift

Perturb:

- file version;
- path;
- API name;
- event signature.

Measure detection, diagnostics, rollback, and safe invalidation of compatibility evidence.

---

## E11 — Registry reproducibility

Given:

```yaml
translator: X
game_version: Y
profile: Z
status: PASS
```

a third party should be able to reproduce the claim from exact source/tests.

---

## E12 — Privacy redaction

Seed telemetry with username, home path, IP, account ID, save path, and token-like strings.

Verify undeclared personal/local data is not exported.

---

# Proposed conformance code families

```text
C — core wire
W — world
R — rules / determinism
E — events / semantics
P — presentation claims
N — negotiation / capabilities
S — security
L — launcher / package
X — cross-implementation / replay
```

Stable codes become a shared language for Discord, issues, CI, and compatibility databases.

# Milestone sequence

## M0 — Beta hardening

- authentication boundary documented;
- fuzzing;
- replays;
- capability negotiation draft;
- conformance expansion.

## M1 — Two-profile semantic system

- `core@1`;
- `fps@1`;
- namespaced IDs;
- compatibility results.

## M2 — Independent adapter ecosystem

- at least three integration styles;
- public compatibility registry;
- reproducible conformance evidence.

## M3 — Generated tooling

- profile → SDK scaffold;
- AI-assisted implementation;
- safe package permissions.

## M4 — Federation / alternate transport experiments

- HLA bridge;
- second transport;
- world addressing/discovery.

At every milestone, prefer measured interoperability over vocabulary growth.
