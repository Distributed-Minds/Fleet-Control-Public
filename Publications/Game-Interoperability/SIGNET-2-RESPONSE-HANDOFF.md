# Signet 2 response — maintainer handoff

**Status:** concise entry point to the full Distributed Minds / Fleet-Control technical response  
**Date:** 2026-10-05  
**Full paper:** `Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md`  
**Proposal delta / review matrix:** `Publications/Game-Interoperability/SIGNET-2-PROPOSAL-DELTA-MATRIX.md`  
**Maintainer-facing delivery plan:** `Publications/Game-Interoperability/SIGNET-2-FORK-DELIVERY-PLAN.md`  
**Historical internal patch series:** `Publications/Game-Interoperability/Signet-2-Proposed-Upstream-Patches/README.md`  
**Draft upstream protocol proposal:** `Publications/Game-Interoperability/SIGNET-2-UPSTREAM-PROTOCOL-PROPOSAL-DRAFT.md`  
**Upstream checked through:** `kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2`

> This is independent technical feedback. It is not official Signet documentation. The planned maintainer-facing artifact is a public `geromet/signetprotocol` review branch with the changes applied directly; the patch bundle in this repository is retained only as historical research material.

## Short version

We think Signet 2 chose the right architecture:

~~~text
game
-> shared meaning
-> authoritative server
-> shared meaning
-> game
~~~

Keep:

- intents distinct from authoritative effects;
- archetypes distinct from local presentation;
- one server authoritative over shared simulation;
- calibration before model inference;
- closed candidate ranking;
- pinned simulation-relevant mappings;
- Signet Forge outside the real-time game loop.

Our main recommendation is to finish the architecture by making the boundaries around "shared meaning" explicit and executable.

~~~text
deterministic core
+ adaptive edge
+ explicit trust boundary
+ reproducible evidence
~~~

## Highest-value changes

### 1. Make shared meaning a versioned semantic contract

A label such as `fire` is not enough.

Bind semantic items to stable identity, version, definition and conformance assertions.

Use a small core plus separately versioned profiles rather than one giant flat game vocabulary.

### 2. Make capability negotiation explicit

"Use the common subset" is underspecified once features can be required, optional, parameterized or approximated.

The negotiated session should explicitly record:

- required / optional / activated capabilities;
- accepted fallbacks;
- rejected capabilities;
- semantic/profile versions;
- authority/ruleset identity;
- deterministic admission result.

### 3. Separate semantic acceptability from session preference

Profiles should say which ordering / visibility models preserve their semantics.

The session should say which acceptable model it prefers.

Current L1 executable rule:

~~~text
profile:
  acceptable models

session:
  ordered preferences

negotiation:
  first session preference accepted by every activated profile
~~~

This avoids implementation enumeration order becoming hidden protocol policy.

### 4. Make optional profiles deterministic

"Optional" should not mean "enable it if installed."

Attempt optional profiles in explicit session preference order.

After each tentative addition:

- resolve dependencies;
- rerun full composition;
- rerun ordering/visibility selection;
- keep it only if the complete contract remains valid;
- otherwise skip it and record the exact reason.

Current executable examples include:

~~~text
ORDERING_MODEL_CONFLICT
REQUIRED_PROFILE_UNSUPPORTED
~~~

### 5. Split translation-profile authority

At minimum distinguish:

~~~text
INPUT_BINDING
MOTION_OBSERVATION
SEMANTIC_MAPPING
APPEARANCE_MAPPING
SESSION_POLICY
~~~

They do not have the same owner or override rules.

### 6. Split publisher baseline from participant overlay

Use approximately:

~~~text
effective profile
=
versioned publisher baseline
+
participant-owned overlay
+
session-compatible derivation
~~~

A package update must not silently rewrite player-owned pins.

### 7. Bind mappings to evidence and invalidation

A pinned mapping can remain deterministic while becoming stale.

Track source/game/profile/transform/candidate fingerprints plus replay/conformance evidence.

Use explicit freshness states such as:

~~~text
VALID
-> SUSPECT
-> REVALIDATING
-> VALID / INVALID
~~~

### 8. Generate adapter contract glue, not imaginary generic game behavior

Our first generic adapter interface was falsified by the documented Minecraft RCON responsibilities.

The replacement describes operations by target responsibility:

~~~text
HOST
IMPORTER
WORLD
INPUT
PRESENTATION
AUTHORITY
~~~

Generate stable bindings/manifests/tests from that descriptor.

Keep actual engine hooks, RCON commands, coordinate transforms and version quirks explicit.

### 9. Let resolver calls abstain

Closed candidate ranking is good.

It still needs:

~~~text
NO_MATCH / ABSTAIN
~~~

A bounded wrong answer is still wrong.

The resolver remains advisory:

~~~text
MODEL_SUGGESTION != AUTHORITY
~~~

### 10. Treat public translators as supply-chain software

Do not collapse these into one "verified" badge:

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
PACKAGE_BYTES         != EFFECTIVE_PROFILE
~~~

Integration mode belongs in the trust subject.

A native Godot extension and an RCON gateway have materially different authority surfaces.

### 11. Keep Forge local-first

Prefer:

~~~text
runtime:
  pinned profile only

Forge:
  local by default

remote resolver:
  explicit opt-in + declared export schema

training contribution:
  separate explicit action
~~~

Local mapping approval is not automatically consent to contribute shared training data.

### 12. Require independent interoperability evidence

The semantic hub can reduce translator authoring toward O(n).

That does not mean empirical compatibility evidence automatically becomes O(n).

~~~text
LINEAR_MAPPING_GROWTH
!=
LINEAR_VALIDATION_DEMAND
~~~

Use an evidence ladder:

~~~text
V0 structural validity
V1 behavioral conformance
V2 independent implementation
V3 pairwise interoperability
V4 independent reproduction
V5 operational freshness
~~~

## Experimental observations already available

### Typed mapping drift

Observed after changing a mapping dependency:

~~~text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
~~~

Result: pinned does not mean eternally valid.

### Godot 4.7.2 GDExtension target

Observed:

~~~text
godot-entry-init: PASS
godot-version-discovery: 0x040702
generated-shim-surface: PASS
~~~

Evidence level is intentionally:

~~~text
ABI_HOST_HARNESS_PASS
~~~

not `GODOT_RUNTIME_PASS`.

### Minecraft RCON target

Representative framing passed, but the first generic adapter interface failed to cover the documented responsibilities:

~~~text
v0-shim-coverage: NO=5, PARTIAL=2
v0-shim-sufficiency: FAIL_EXPECTED
~~~

The role-contract revision then represented the documented responsibility surface:

~~~text
minecraft-responsibility-coverage: 7/7
minecraft-role-contract: PASS
~~~

### Resolver pilot

Original candidate order made weak mechanisms look artificially good.

Observed:

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

Recommendation: the planned 30-case experiment should include candidate-order perturbation and NO_MATCH behavior, not top-1 accuracy alone.

### Executable L1 negotiation

Current Python and JavaScript reference implementations reproduce:

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: NEGOTIATION-VECTOR-001
PASS: OPTIONAL-NEGOTIATION-VECTOR-001
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
~~~

These are candidate research semantics, not implemented Signet behavior.

## Direct answers to the current open questions

### Minimum motion profile?

Profile/ruleset dependent.

Measure only what is required for normalization, fidelity/prediction and compatibility evidence.

Do not let local motion calibration redefine authoritative server physics.

### No position observable?

Use the strongest legitimate source available.

If it cannot be observed reliably:

~~~text
UNKNOWN
UNSUPPORTED
OBSERVE_ONLY
~~~

is preferable to fake precision.

### Who owns vocabulary/catalog evolution?

Protocol governance over:

- stable IDs;
- small core;
- separately versioned profiles/namespaces;
- proposal/review;
- conformance vectors;
- deprecation/migration.

### Where should the resolver run?

Development/calibration time, local by default.

Hosted resolver optional.

Never required in the deterministic per-tick path.

### Conflicting community profiles?

Allow alternatives.

Resolve exact session behavior through versioned publisher baseline + participant overlay + deterministic session derivation.

Do not silently merge by popularity.

## Recommended first joint experiment

Do not start with universal game support.

Start with:

1. 5–8 intents;
2. 4–6 archetypes;
3. explicit capability/session negotiation;
4. two materially different integration modes;
5. generated role-contract glue;
6. no-AI calibration first;
7. evidence-bearing effective profiles;
8. replay + pairwise tests;
9. profile-only drift test;
10. participant-overlay preservation test;
11. current-authorization revocation test;
12. only then the resolver benchmark.

A useful first result is either:

- two independently developed adapters agree; or
- they disagree in a way that exposes exactly what the semantic contract failed to specify.

Both outcomes improve the protocol.

## Supporting research snapshot

Observed heads for the current synthesis:

~~~text
L1  b8f47a332fcb0aa641cfe676e22b49a801731628
L2  a5a677f7bf3e80d2face8f2185b27fcb8e394ab3
L3  821d13fe33f293281c85dc9da3a32e2a3fb47148
L4  22cdaf37b08d940a518c4a6a9dcea51553cd7256
~~~

The full paper contains the detailed reasoning, experiment descriptions, trust model, validation model, 24 proposed draft changes, and references.
