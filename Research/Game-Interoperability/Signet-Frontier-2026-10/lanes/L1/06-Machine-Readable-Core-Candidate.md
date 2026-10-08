# L1 Continuation 06 — Machine-Readable Semantic Core Candidate

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 4  
**Status:** unaffiliated research draft, not a Signet specification  
**Research cutoff:** 2026-10-05

## Purpose

The preceding L1 work argues that Signet 2's “shared meaning” needs a small machine-checkable coordination contract beneath domain vocabularies.

This pass turns that claim into a concrete candidate artifact.

Files:

- [signet-semantic-core-0.schema.json](schema/signet-semantic-core-0.schema.json)
- [turn-based capability example](schema/examples/turn-based-capabilities.json)
- [hidden-information card session requirements](schema/examples/card-session-requirements.json)
- [hidden-information negotiated contract](schema/examples/card-negotiated-contract.json)

The schema uses **JSON Schema Draft 2020-12**, which the JSON Schema project currently identifies as the latest published version:

- https://json-schema.org/specification
- https://json-schema.org/draft/2020-12/schema

This is a structural encoding choice, not a claim that JSON Schema solves semantic interoperability.

# 1. What the schema tries to standardize

The candidate contains four top-level artifact types.

## 1.1 Semantic envelope

~~~text
signet.semantic-envelope
~~~

Coordination-level fields include:

- semantic category:
  - intent
  - state
  - event
  - ack
  - control
- session/message identity;
- semantic ID + definition hash;
- issuer;
- authority scope/epoch;
- negotiated ordering model + model-defined ordering value;
- causal references;
- optional visibility-policy reference;
- domain payload.

The payload remains profile-defined.

The core schema deliberately does **not** know what a weapon, card, board cell, health value, voxel, economy or city district is.

## 1.2 Capability declaration

~~~text
signet.capabilities
~~~

A participant declares:

- core definition;
- participant/game/adapter identity;
- supported protocol envelopes;
- semantic profiles;
- publish/consume/observe roles;
- required and optional concepts;
- explicit fallbacks;
- ordering models;
- authority scopes;
- visibility policies.

This implements the earlier distinction between:

~~~text
understood
available
required
optional
fallback
~~~

without treating one boolean as sufficient evidence of interoperability.

## 1.3 Session requirements

~~~text
signet.session-requirements
~~~

The session selects:

- exact core definition;
- required profiles/concepts;
- optional concepts;
- exact ordering model;
- exact ruleset;
- visibility policy where needed;
- allowed fallbacks;
- whether observer-only participation is permitted.

This is the key alternative to “take the common subset.”

A session describes the game it intends to run.

Participants are evaluated against that contract.

## 1.4 Negotiated contract

~~~text
signet.negotiated-contract
~~~

The result records one of:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACKS
OBSERVE_ONLY
INCOMPATIBLE
~~~

plus:

- exact activated profile definitions;
- selected ordering model;
- selected ruleset;
- visibility policy;
- accepted fallbacks;
- disabled optional concepts;
- stable reason codes.

The result becomes an inspectable runtime artifact rather than an undocumented decision inside connection logic.

# 2. Structural validity is not semantic conformance

This distinction is critical.

JSON Schema can prove things such as:

- a semantic reference has both an ID and definition hash;
- a result is one of four allowed compatibility states;
- a capability declaration has the expected structural members;
- an ordering object names a model;
- a negotiated contract contains the selected ruleset.

It cannot prove:

- that two definitions actually mean the same thing;
- that a mapping from a Minecraft bow to a presentation role is reasonable;
- that two independently written adapters interpret “damage applied” identically;
- that an approximation preserves fairness;
- that a turn-order implementation matches the ruleset;
- that hidden information was not leaked;
- that replay semantics are correct.

Therefore:

~~~text
schema-valid
!=
semantically conformant
~~~

The JSON Schema artifact is the syntax/admission substrate.

The ID/NEG/MAP/AUTH/ORDER/VIS/PROFILE/LEGACY conformance properties remain necessary.

# 3. Why ordering.value is deliberately open

The core schema requires:

~~~yaml
ordering:
  model:
    id: ...
    definition_hash: ...
  value:
    ...
~~~

but leaves the contents of `value` profile/model-defined.

That is intentional.

A fixed-tick session may use:

~~~yaml
value:
  tick: 18442
  substep: 3
~~~

A turn-based session may use:

~~~yaml
value:
  turn: 42
  phase: action
  action_index: 1
~~~

The selected ordering-model definition supplies the semantic constraints.

A later implementation can validate the model-specific value against the selected ordering-model schema.

This prevents the universal core from hard-coding one genre's clock.

# 4. Why semantic references contain both ID and hash

A semantic reference is:

~~~yaml
id: <globally scoped semantic identifier>
definition_hash: sha256:<digest>
~~~

The ID provides stable identity/version naming.

The hash detects a particularly dangerous state:

~~~text
both peers say "profile X version 1"
but
their actual published definitions differ
~~~

That should become a typed conflict rather than an implicit “close enough.”

This draft does not yet define canonical serialization for definition hashing.

That remains an L1 task.

Until canonicalization is specified, the field should be considered a design commitment rather than a deployable interoperable digest.

# 5. Cross-domain examples

## 5.1 Turn-based capability declaration

The turn-based example demonstrates that a participant can advertise:

- move intent;
- board state;
- move-applied event;
- turn-sequence ordering;

without any:

- fixed simulation tick;
- player body;
- health;
- weapon;
- FPS movement;
- 3D pose.

That directly exercises the non-FPS falsification requirement.

## 5.2 Hidden-information card session

The card-session example adds:

- card-zone profile;
- turn-sequence ordering;
- a session-specific ruleset;
- a private-hand visibility policy.

This proves that “server is authoritative” does not imply “broadcast all authoritative state.”

Authority and disclosure are separate coordination dimensions.

## 5.3 Negotiated card contract

The negotiated contract demonstrates that the output can remain generic even though the activated domain is unrelated to current Signet gameplay.

The exact same top-level result shape should work for an FPS session with:

- different profiles;
- fixed-tick ordering;
- a different ruleset;
- no hidden-information policy.

# 6. Intentional omissions

The schema does not yet attempt to define:

- canonical semantic-definition serialization/hashing;
- profile dependency graphs;
- profile composition conflict resolution;
- authority transfer protocol;
- schema discovery/resolution;
- cryptographic identity/signatures;
- transport framing;
- package/launcher metadata;
- legal/licensing policy;
- model/resolver APIs;
- domain payload schemas;
- concrete visibility-policy semantics;
- concrete ordering-value schemas.

Those omissions are deliberate.

A universal core that tries to standardize all of them at once would repeat the monolithic-vocabulary problem at another layer.

# 7. Candidate reason codes

The schema currently includes:

~~~text
PROTOCOL_MAJOR_MISMATCH
SEMANTIC_DEFINITION_CONFLICT
REQUIRED_PROFILE_UNSUPPORTED
REQUIRED_CONCEPT_UNSUPPORTED
AUTHORITY_MODEL_UNSUPPORTED
ORDERING_MODEL_UNSUPPORTED
VISIBILITY_POLICY_UNSUPPORTED
WORLD_REPRESENTATION_UNSUPPORTED
NO_SAFE_FALLBACK
LEGACY_SEMANTICS_ONLY
OPTIONAL_CONCEPT_DISABLED
~~~

This list is provisional.

A useful rule is:

> add a core reason code only when independent implementations need to distinguish the failure to make the same admission decision.

Domain failures belong in profile/ruleset diagnostics.

# 8. What Signet could prototype with this

A minimal experiment does not need Signet 2 to adopt this exact schema.

The useful experiment is:

1. define one machine-readable FPS capability manifest;
2. define one machine-readable turn/card capability manifest;
3. define two session requirement artifacts;
4. run one pure deterministic negotiation function;
5. compare independently implemented outputs;
6. reject same-ID/different-definition cases;
7. demonstrate optional feature disablement;
8. demonstrate observer-only derivation;
9. demonstrate ordering-model mismatch;
10. demonstrate visibility-policy mismatch.

That test would answer a more important question than whether this JSON shape is aesthetically ideal:

> Can two teams derive the same compatibility decision from the same explicit semantic inputs?

# 9. L4 handoff

~~~yaml
handoff:
  from_lane: L1
  to_lane: L4
  finding: A JSON-Schema 2020-12 candidate now encodes the L1 coordination contract across FPS-independent capability/session/result examples.
  evidence:
    - lanes/L1/schema/signet-semantic-core-0.schema.json
    - lanes/L1/schema/examples/turn-based-capabilities.json
    - lanes/L1/schema/examples/card-session-requirements.json
    - lanes/L1/schema/examples/card-negotiated-contract.json
  why_material: The design can now be implemented independently instead of interpreted only from prose.
  requested_followup: Validate the examples structurally, build an independent negotiation implementation, add negative vectors for each stable reason code, and report schema/semantic mismatches back to L1.
~~~

# 10. Next L1 work

The two highest-value remaining semantic-design questions are now narrower:

## 10.1 Canonical definition hashing

Specify exactly which bytes are hashed.

Requirements:

- semantically irrelevant formatting must not create accidental conflicts;
- two independent tools must calculate the same digest;
- references/dependencies must be represented unambiguously;
- canonicalization must not make implementation unreasonably complex.

## 10.2 Profile composition

Define what happens when a session activates several profiles that:

- define overlapping concepts;
- require incompatible versions;
- attach different constraints to one field;
- depend on different ordering/visibility models;
- extend one another.

This is where HLA modular-FOM conflict/merge rules and schema-composition literature are likely most useful next.

# Conclusion

The Signet 2 response has now crossed an important line.

“Shared meaning” is no longer only an architectural slogan or prose proposal in the L1 work.

There is now a candidate machine-readable coordination surface that can be independently implemented, rejected, revised and tested.

That is the level at which semantic interoperability claims become falsifiable.
