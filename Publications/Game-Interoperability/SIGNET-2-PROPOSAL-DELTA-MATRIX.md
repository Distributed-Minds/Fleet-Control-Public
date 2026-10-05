# Signet 2 proposal delta matrix

**Purpose:** map the current Signet 2 / Signet Forge draft to the concrete deltas proposed by the Distributed Minds / Fleet-Control response.
**Date:** 2026-10-05
**Status:** independent technical feedback; not official Signet documentation.

Full response: Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md
Short handoff: Publications/Game-Interoperability/SIGNET-2-RESPONSE-HANDOFF.md

## Evidence language

- **OBSERVED**: directly present in the cited Signet draft or produced by a supporting experiment.
- **DERIVED**: engineering consequence of observed behavior or stated architecture.
- **PROPOSED**: normative change suggested for Signet 2 / Forge.
- **UNKNOWN**: not established by current public evidence.

## Upstream snapshot reviewed

Observed current upstream commit:

~~~text
kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2
~~~

Exact reviewed blobs:

| Document | Blob |
|---|---|
| docs/content/proposals/translation-profiles.mdx | d613b7cb4ff09f697b9d880c691320a98f65f590 |
| docs/content/forge/index.mdx | 5be55a71c775868bb3dfec86ce641e249f67d8ac |
| docs/content/forge/architecture.mdx | 4460961a54285ce088da57fdff6c1fa132366eb6 |
| docs/content/forge/workflow.mdx | 48cb28a97749063d0d732566987b98e7efc72e1b |
| docs/content/forge/understanding-a-game.mdx | a6e09a315727e777a82c26d5f5316f5381338fb4 |
| docs/content/forge/how-the-model-decides.mdx | 25dce1f0cb1cfda98da1f84fd4e13989de2b9ce6 |
| docs/content/forge/reliability-and-limits.mdx | 1ca6b757f69f23b8f1a95a63db31c60e9e6b5ac2 |
| docs/content/forge/fine-tuning.mdx | 085d2ea343479ca66317741befce8f507c26d9f8 |
| docs/content/forge/roadmap.mdx | eca0a9d36498ecad25002fbf9710269d295ecaa9 |
| docs/content/forge/glossary.mdx | 1ed3197504cd079e6b375bfad3bab144079e77d9 |

The proposal still explicitly says Signet 2 / Forge is draft and not implemented.

---

# P0: semantic contract before independent implementations

## 1. Stable semantic identity and versioning

**Upstream seam:** translation-profiles "Three vocabularies"; forge/architecture concepts; roadmap step 1.

**OBSERVED:** the draft primarily identifies semantics through labels such as move, fire, weapon.ranged and health_pickup.

**Risk:**

~~~text
SHARED_LABELS != SHARED_SEMANTICS
~~~

Two implementations can agree on "fire" while disagreeing about request versus accepted action, repetition, cooldown, invalid-state behavior, authority or side effects.

**PROPOSED:** every normative semantic item should carry a stable reference:

~~~text
semantic_ref:
  namespace
  id
  version
  definition_hash
~~~

Also publish definition, examples, non-examples and conformance assertions.

Suggested wording:

> Intent and archetype labels are display names, not sufficient protocol identity. A normative semantic item is identified by a versioned semantic reference. Two implementations claim the same semantic only when they reference the same normative definition or an explicitly compatible successor.

## 2. Small core plus modular profiles

**Upstream seam:** one shared intent vocabulary and one archetype catalog.

**DERIVED:** a flat universal list is likely to inherit assumptions from the first supported game family.

**PROPOSED:** conservative core plus separately versioned profiles/namespaces, for example movement, combat, inventory, vehicles, turn-based and card-game semantics. A session activates only what it needs.

## 3. Capability declaration must become session negotiation

**Upstream seam:** translation-profiles "Capabilities"; understanding-a-game capabilities/gaps; reliability "limited games".

**OBSERVED:** current wording says the server uses the common subset and may ignore or approximate unsupported intents.

Set intersection is insufficient when a feature is required, optional, parameterized, approximable, observe-only or unavailable.

**PROPOSED negotiation states:**

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACK
OBSERVE_ONLY
INCOMPATIBLE
~~~

The negotiated contract should record required, optional, activated and rejected capabilities; accepted fallbacks; semantic/profile versions; ruleset identity; and deterministic admission result.

Suggested wording:

> A capability declaration is an offer, not the session contract. The session computes a deterministic negotiated contract from participant capabilities plus session requirements. Missing required semantics cause incompatibility unless the session explicitly authorizes a typed fallback.

## 4. Fallback must be typed

**Upstream seam:** "ignore or approximate".

Ignoring a cosmetic effect and approximating an authoritative gameplay intent are not equivalent.

A fallback should state source semantic, target/no-op, fallback class, information loss, shared-state impact, approving authority and conformance evidence.

> Presentation-only fallbacks may be permissive. A fallback that changes shared simulation semantics must be explicitly allowed by the active profile or ruleset.

## 5. Archetype is not the authoritative ruleset

**Upstream seam:** forge/architecture "One archetype, three appearances".

**PROPOSED:** archetype identifies semantic role/category; server-owned state/ruleset carries damage, range, cooldown, health, quantity and other authoritative parameters.

---

# P0: deterministic composition and policy

## 6. Separate semantic acceptability from session preference

**OBSERVED supporting evidence:** L1 has executable Python and JavaScript vectors.

~~~text
profile:
  accepted semantic models

session:
  ordered preferences

negotiation:
  first session preference accepted by every activated profile
~~~

Example:

~~~text
session: fixed-tick, event-sequence, turn-sequence
profile A: fixed-tick, event-sequence
profile B: event-sequence, turn-sequence
selected: event-sequence
~~~

**PROPOSED:** never derive semantic policy from enumeration order or a hidden implementation default.

> When multiple models satisfy every activated semantic profile, the session MUST select from an explicit ordered preference list.

## 7. Optional profiles need deterministic activation

**OBSERVED L1 vector:**

~~~text
activate optional-event@1

skip optional-fixed@1
  reason = ORDERING_MODEL_CONFLICT

skip missing@1
  reason = REQUIRED_PROFILE_UNSUPPORTED
~~~

**PROPOSED algorithm:**

1. compose required baseline;
2. process optionals in declared preference order;
3. tentatively add candidate;
4. resolve dependencies;
5. rerun full composition and policy selection;
6. keep on success;
7. otherwise skip with exact machine-readable reason.

Earlier accepted optionals remain preferred.

## 8. Profile composition is protocol-relevant

**Upstream seam:** workflow profile storage; translation-profiles resolution order.

Current priority is player pin, translator profile, resolver suggestion, safe default.

That does not fully define composition across publisher defaults, participant overrides, profile versions and session policy.

**PROPOSED:** canonical composition/hashing plus test vectors covering dependency conflicts, stale overlays, duplicate IDs, version mismatch, optional activation and ordering.

---

# P0/P1: profile authority and drift

## 9. Split profile entries by authority class

Current profile concepts combine decisions with different owners.

| Class | Natural authority |
|---|---|
| input binding | participant / adapter |
| motion observation | measured adapter evidence |
| semantic mapping | translator/profile authority |
| appearance mapping | participant/client |
| session policy | server/session |

**PROPOSED:** authority class is explicit; precedence is per class.

There should not be one universal "player pin wins" rule for simulation-affecting semantics.

## 10. Publisher baseline versus participant overlay

**Upstream seam:** profile is both a community translator artifact and a place for player decisions.

**PROPOSED model:**

~~~text
publisher_profile
  immutable/versioned baseline

participant_overlay
  local participant-owned bindings/pins

session_resolution
  derived effective profile
~~~

Publisher updates must not silently rewrite participant-owned decisions.

## 11. Pinned does not mean current or correct

**OBSERVED experiment:** a mapping stayed pinned after game/integration dependency fingerprints changed and was correctly marked SUSPECT.

~~~text
PINNED_CHOICE != CORRECT_CHOICE
~~~

Bind simulation-relevant mappings to game/version identity, integration digest, semantic definition, transform, candidate fingerprint, resolver identity where relevant, and replay/conformance evidence.

Suggested freshness states:

~~~text
VALID
SUSPECT
REVALIDATING
INVALID
~~~

Suggested wording:

> A pinned decision is immutable within that profile revision. Its compatibility status may still change when declared dependencies change.

## 12. Effective profile identity

~~~text
PACKAGE_BYTES != EFFECTIVE_PROFILE
~~~

Identical translator bytes can behave differently because of overlays, calibration, session policy, semantic-profile selection and fallback approval.

**PROPOSED:** effective_profile_digest over the fully resolved behavior-bearing profile cut.

---

# P0/P1: resolver and calibration

## 13. Resolver needs NO_MATCH / ABSTAIN

**Upstream seam:** closed-list ranking.

Closed candidates prevent out-of-list invention; they do not guarantee that a correct candidate exists.

~~~text
CLOSED_CANDIDATES != CORRECT_CANDIDATE_PRESENT
~~~

**PROPOSED API:** ranked candidates OR NO_MATCH/ABSTAIN.

Simulation-affecting misses should fail closed or require explicit pinning.

## 14. Resolver experiment must test order stability

**OBSERVED pilot:**

~~~text
lookup:
  base_accuracy=1.000
  no_match=1.000
  order_stability=1.000

lexical:
  base_accuracy=1.000
  no_match=0.000
  order_stability=0.167

first-candidate:
  base_accuracy=1.000
  no_match=0.000
  order_stability=0.000
~~~

Original order made weak mechanisms look artificially good.

**PROPOSED:** add candidate-order perturbation and no-correct-candidate cases to the planned 30-case benchmark.

## 15. Calibration must expose observability limits

**Upstream seam:** motion-profile open question.

Prefer legitimate sources in roughly this order:

~~~text
open engine instrumentation
official mod/plugin API
controlled-server telemetry
official external API
manual declaration
UNKNOWN / UNSUPPORTED / OBSERVE_ONLY
~~~

A calibration result should state whether a value was measured, declared, inferred or unavailable.

## 16. Motion profile does not redefine physics

The draft already correctly says the server keeps canonical physics.

Make this normative: motion profile is normalization/fidelity evidence, not a replacement for authoritative server physics.

---

# P1: adapter contract and real targets

## 17. Generate contract glue, not imaginary generic game behavior

**OBSERVED:** deterministic adapter glue can be generated; real target hooks remain target-specific.

Generate stable bindings, role manifests, operation IDs, serialization and test skeletons.

Keep engine callbacks, RCON commands, coordinate transforms and version quirks explicit.

## 18. Role/operation contracts instead of one generic shim

**OBSERVED Minecraft experiment:**

~~~text
v0 coverage:
  NO=5
  PARTIAL=2
v0-shim-sufficiency:
  FAIL_EXPECTED
~~~

Revised roles:

~~~text
HOST
IMPORTER
WORLD
INPUT
PRESENTATION
AUTHORITY
~~~

covered all seven documented responsibility classes.

A real target must be allowed to falsify the descriptor.

## 19. Evidence labels must match what ran

**OBSERVED Godot result:**

~~~text
ABI_HOST_HARNESS_PASS
~~~

not:

~~~text
GODOT_RUNTIME_PASS
~~~

Do not convert ABI loadability, harness behavior or static checks into runtime interoperability claims.

---

# P1: trust and distribution

## 20. Integration mode is part of the trust subject

A native Godot extension runs in the host process. A Minecraft RCON gateway holds a controlled-server credential boundary.

These require different trust claims.

Useful integration-mode classes include:

~~~text
HOST_PROCESS_EXTENSION
SANDBOXED_WORKER
CONTROLLED_SERVER_GATEWAY
EXTERNAL_OFFICIAL_API
REIMPLEMENTATION
FULL_APPLICATION
~~~

## 21. Role contract is not enforcement

~~~text
ROLE_CONTRACT != SANDBOX_POLICY
DECLARED_PERMISSION != ENFORCED_PERMISSION
~~~

Separate declared capabilities from broker/API mediation, OS isolation, network policy and credential scoping.

## 22. Separate supply-chain claims

~~~text
VALID_SIGNATURE != CURRENT_AUTHORIZATION
SAFE_ARTIFACT != PROTOCOL_CONFORMANCE
PROTOCOL_CONFORMANCE != GAME_COMPATIBILITY
~~~

Record artifact digest, signer, current authorization, provenance, reproducibility, integration mode, confinement class, conformance evidence, compatibility evidence and lifecycle state separately.

---

# P1: Forge privacy and training

## 23. Forge local-first

**Upstream open question:** resolver location.

**PROPOSED:**

~~~text
runtime:
  pinned deterministic profile

Forge:
  local by default

remote resolver:
  optional explicit service

per-tick resolver:
  never required
~~~

Hosted resolution should have an explicit export contract.

## 24. Local approval is not dataset contribution

The current proposal says every approved choice is a training example.

~~~text
LOCAL_APPROVAL != DATASET_CONTRIBUTION
~~~

Suggested replacement:

> Every approved choice is eligible to become a training example. Contribution to a shared dataset is a separate explicit action under the dataset contribution terms.

Dataset examples should carry immutable identity, provenance, semantic/profile version, license and invalidation/supersession lineage.

---

# P1/P2: validation

## 25. O(n) authoring does not imply O(n) evidence

The semantic hub correctly reduces translator implementation topology.

~~~text
O(n) mapping topology
!=
O(n) empirical validation demand
~~~

Two locally conformant adapters can still interact badly because of timing, fallback composition, ordering, visibility, lifecycle semantics or authority boundaries.

Suggested wording:

> Shared semantics reduce implementation topology. Compatibility evidence remains scoped: local conformance does not by itself prove every pairwise runtime interaction.

## 26. Publish an evidence ladder

~~~text
V0 structural validity
V1 behavioral conformance
V2 independent implementation
V3 pairwise interoperability
V4 independent reproduction
V5 operational freshness
~~~

Preserve negative and invalidated evidence, not only PASS records.

---

# P2: governance and lifecycle

## 27. Separate governance authorities

Distinguish protocol, semantic-profile, translator/profile publisher, participant, package-directory, update-root, incident, Forge/dataset/model and conformance-suite authority.

The same maintainers may occupy several roles initially; artifacts should still name the roles separately.

## 28. Profile PRs are behavior-bearing supply-chain changes

A profile-only PR can change runtime behavior without code changes.

Surface semantic subject, behavior diff, authority class, invalidated evidence, required replay/conformance and resulting immutable profile digest.

## 29. Lifecycle states need more than "latest"

At minimum:

~~~text
ACTIVE
SUPERSEDED
DEPRECATED
YANKED_COMPATIBILITY
COMPATIBILITY_INVALIDATED
SECURITY_REVOKED
PUBLISHER_REVOKED
LEGAL_OR_POLICY_BLOCKED
~~~

Compatibility failure, security compromise and legal/policy scope are different events.

---

# Early wording corrections

## Server authority / anti-cheat

Current forge/architecture wording says:

> It is the only authority, so nobody can cheat by modifying their client.

That is too strong.

**PROPOSED replacement:**

> The server is authoritative over shared simulation state, so clients cannot directly dictate authoritative positions, hits, damage or deaths. This removes important classes of client-side state forgery but does not by itself prevent every form of cheating.

~~~text
SERVER_AUTHORITY != CHEAT_IMPOSSIBILITY
~~~

## Pinned profile wording

Clarify "once pinned, it does not change" as:

> A pinned decision is immutable within that profile revision. Compatibility may still become stale when its declared dependencies change.

## Common-subset wording

Replace the underspecified common-subset rule conceptually with:

> The server derives a negotiated session contract from participant capabilities, active semantic profiles and session requirements. Required unsupported semantics fail admission unless a typed fallback is explicitly allowed. Optional semantics are activated deterministically according to session policy.

---

# Proposed roadmap order

Current draft:

~~~text
vocabulary/catalog
capabilities
profile/calibration
no-AI prototype
30-case model experiment
Forge v1
~~~

Suggested order:

~~~text
1. freeze tiny versioned semantic core/profile
2. publish normative definitions + conformance assertions
3. define capability/session negotiation + typed fallback
4. define deterministic profile composition + hashing
5. define authority classes + participant overlay
6. define calibration observability/provenance
7. implement no-AI two-target prototype
8. run behavioral + pairwise + drift tests
9. define package trust/revocation metadata
10. run resolver experiment with NO_MATCH + order perturbation
11. define explicit dataset contribution/invalidation
12. build Forge workflow around the proven contracts
~~~

This is not an argument to delay Forge. It keeps the model from becoming the place where undefined protocol semantics are accidentally decided.

---

# Smallest useful joint experiment

Freeze:

- 5-8 intents;
- 4-6 archetypes;
- one movement/body profile;
- one damage/death lifecycle;
- explicit capability negotiation;
- one optional profile;
- one typed fallback.

Implement two materially different integration modes, such as one open-engine/native adapter and one controlled-server gateway.

Require:

- versioned semantic identity;
- deterministic session-preference selection;
- deterministic optional-profile activation;
- participant-overlay preservation;
- effective-profile digest;
- no-AI calibration first;
- replay/conformance evidence;
- pairwise runtime test;
- dependency-drift invalidation;
- authorization revocation;
- resolver NO_MATCH case.

Useful success:

~~~text
two independently developed implementations
derive the same negotiated semantic contract
and reproduce the same normative vectors
~~~

Useful failure:

~~~text
same profile inputs
+ same session requirements
+ different negotiated contract
= specification bug
~~~

Both outcomes improve a draft protocol.

---

# Supporting research heads

~~~text
L1  b8f47a332fcb0aa641cfe676e22b49a801731628
L2  a5a677f7bf3e80d2face8f2185b27fcb8e394ab3
L3  821d13fe33f293281c85dc9da3a32e2a3fb47148
L4  22cdaf37b08d940a518c4a6a9dcea51553cd7256
~~~

For detailed evidence, rationale and references, see the full response paper.
