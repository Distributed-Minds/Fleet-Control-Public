# Draft upstream protocol proposal — make Signet 2 shared meaning reproducible

**Target:** `kian-cx/signetprotocol`  
**Target version:** Signet 2 draft  
**Upstream snapshot reviewed:** `2ddb136ee941705d3e1c020eaddad93be65026f2`  
**Intended venue:** GitHub protocol proposal / Discussion  
**Status:** draft only; not posted upstream

## Suggested title

**Signet 2: make shared meaning a versioned, negotiated semantic contract**

## Use case

Signet 2's proposed architecture is a strong scaling direction:

~~~text
game
-> shared meaning
-> authoritative server
-> shared meaning
-> game
~~~

The remaining interoperability problem is that the current draft does not yet define enough of "shared meaning" for independent implementations to be guaranteed to derive the same session behavior.

The gaps become visible in five places:

1. **Semantic identity.** Human labels such as `fire` or `weapon.ranged` do not fully define protocol meaning.
2. **Capability negotiation.** "Use the common subset" is underspecified when semantics can be required, optional, approximated or unavailable.
3. **Profile composition.** Player pins, translator defaults, semantic mappings and session policy do not all have the same authority.
4. **Resolver failure.** A closed candidate list prevents out-of-list invention, but the correct answer can still be absent.
5. **Compatibility freshness.** A pinned choice can remain deterministic after a game/integration/profile dependency changes.

The goal of this proposal is not to replace the Signet 2 architecture.

It is to make the existing architecture deterministic and independently testable.

A useful summary is:

~~~text
deterministic core
+ adaptive edge
+ explicit trust boundary
+ reproducible evidence
~~~

### Why this matters before implementation

If two independent translators start with:

- the same semantic vocabulary/profile;
- the same capability offers;
- the same session requirements;

they should derive the same:

- activated profiles;
- ordering / visibility model;
- optional-profile decisions;
- typed fallbacks;
- effective session-contract identity.

If they do not, the protocol is missing a rule.

### Supporting experiments

This proposal is informed by implementation-facing experiments in a separate research repository.

Observed examples include:

- a pinned semantic mapping correctly becoming `SUSPECT` after its declared game/integration dependencies changed;
- an initial generic adapter interface failing to cover a documented Minecraft RCON target, followed by a responsibility-based role contract that covered the target fixture;
- a resolver pilot where top-1 accuracy alone looked perfect for weak baselines until candidate order and `NO_MATCH` behavior were tested;
- cross-language Python and JavaScript reference vectors reproducing deterministic semantic composition, model pre-selection and optional-profile activation.

These results are not claims about current Signet runtime behavior. Signet 2 / Forge is still a draft.

## New fields or messages

This is a Signet 2 proposal, so the exact wire encoding is intentionally not prescribed here.

The proposal is to define the following protocol concepts before freezing the Signet 2 vocabulary/profile format.

### 1. Versioned semantic reference

Every normative semantic item should have a stable identity beyond its display label.

Illustrative shape:

~~~json
{
  "namespace": "signet.core",
  "id": "intent.fire",
  "version": 1,
  "definition_hash": "sha256:..."
}
~~~

A normative definition should include:

- identity;
- version;
- definition hash;
- definition;
- examples / non-examples;
- conformance assertions.

Suggested invariant:

~~~text
SHARED_LABELS != SHARED_SEMANTICS
~~~

### 2. Semantic core plus modular profiles

Keep the universal core small.

Allow separately versioned semantic profiles such as:

~~~text
movement.realtime
combat.ranged
inventory.basic
vehicle.basic
turn-based.actions
card-game.hidden-information
~~~

A session activates only the profiles it needs.

### 3. Capability offer

Treat a client's declared capabilities as an offer rather than the final session policy.

Illustrative fields:

~~~text
profiles
can_emit
can_present
can_observe
limitations
integration identity
~~~

### 4. Session requirements

The active session/ruleset should state:

~~~text
required_profiles
required_capabilities
optional_capabilities
ordering_preferences
visibility_preferences
allowed_fallbacks
ruleset identity
~~~

### 5. Negotiated session contract

The server derives a deterministic contract from capability offers plus session requirements.

The result should record:

~~~text
semantic/profile identities
required capabilities
activated optional capabilities
rejected capabilities
typed fallbacks
ordering model
visibility policy
authority/ruleset identity
admission result
contract digest
~~~

Useful capability outcomes:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACK
OBSERVE_ONLY
INCOMPATIBLE
~~~

### 6. Explicit preference selection

Profiles declare which semantic models are acceptable.

The session declares which acceptable models it prefers.

~~~text
profile:
  accepted models

session:
  ordered preferences

negotiation:
  first session preference accepted by every activated profile
~~~

This prevents enumeration order from becoming hidden protocol policy.

### 7. Deterministic optional-profile activation

Optional profiles should be attempted in explicit session preference order.

For each candidate:

1. tentatively add it;
2. resolve dependencies;
3. rerun full composition;
4. rerun ordering/visibility selection;
5. keep it if valid;
6. otherwise skip it with an exact reason.

Illustrative reasons:

~~~text
ORDERING_MODEL_CONFLICT
VISIBILITY_POLICY_CONFLICT
REQUIRED_PROFILE_UNSUPPORTED
SEMANTIC_CONFLICT
~~~

### 8. Typed fallback

A fallback should identify:

~~~text
source semantic
target semantic / no-op
fallback class
information loss
shared-state impact
approving authority
conformance evidence
~~~

Presentation substitution and authoritative simulation substitution should not be treated as the same thing.

### 9. Profile authority classes

At minimum distinguish:

~~~text
INPUT_BINDING
MOTION_OBSERVATION
SEMANTIC_MAPPING
APPEARANCE_MAPPING
SESSION_POLICY
~~~

The priority rule "player pin -> translator profile -> model -> default" is useful for participant-owned choices.

It should not become a universal authority rule for simulation-affecting semantics or server policy.

### 10. Publisher baseline and participant overlay

Suggested composition:

~~~text
effective profile
=
versioned publisher baseline
+
participant-owned overlay
+
session-compatible derivation
~~~

A publisher update must not silently rewrite participant-owned choices.

### 11. Mapping provenance and freshness

A pinned mapping can remain deterministic while becoming stale.

Suggested provenance inputs include:

~~~text
game/version identity
integration-surface identity/digest
semantic definition
transform identity
candidate-set fingerprint
resolver identity, if applicable
conformance/replay evidence
~~~

Suggested states:

~~~text
VALID
SUSPECT
REVALIDATING
INVALID
~~~

Invariant:

~~~text
PINNED_CHOICE != CORRECT_CHOICE
~~~

### 12. Effective profile digest

Installed package bytes are not the entire behavior-bearing state.

~~~text
PACKAGE_BYTES != EFFECTIVE_PROFILE
~~~

Define an identity over the resolved publisher profile, participant overlay, semantic profiles, calibration results and session policy.

This can be referenced by replay and compatibility evidence.

### 13. Resolver abstention

Change the conceptual resolver API from:

~~~text
closed candidates -> ranked candidates
~~~

to:

~~~text
closed candidates -> ranked candidates | NO_MATCH
~~~

A closed list prevents invention but does not guarantee the correct candidate is present.

Simulation-affecting unresolved mappings should fail closed or require explicit confirmation.

### 14. Calibration observability

Calibration results should say whether a value was:

~~~text
MEASURED
DECLARED
INFERRED
UNKNOWN
UNSUPPORTED
~~~

Unknown is preferable to invented precision.

The motion profile helps with normalization/fidelity and does not replace authoritative server physics.

### 15. Canonical composition and test vectors

Profile/session composition is protocol-relevant behavior.

Please define canonical:

- dependency resolution;
- conflict handling;
- ordering;
- serialization;
- hashing.

Ship executable vectors for:

- semantic hash;
- required composition;
- preference selection;
- optional activation;
- negative conflicts;
- negotiated contract output.

## Effect on existing translators

### Signet/1

No breaking Signet/1 change is requested by this proposal.

If any capabilities work is backported to Signet/1, the existing governance rule should remain: additive optional fields with defaults only.

### Signet 2 draft translators

The proposal would add more explicit metadata and validation rules, but it should reduce hidden implementation-specific behavior.

A translator that already:

- declares capabilities;
- stores pinned decisions;
- performs calibration;

would gain explicit identity/provenance and deterministic composition around those concepts.

### Forge

Forge remains a development/calibration companion tool, not a per-tick dependency.

Recommended behavior:

~~~text
runtime:
  deterministic resolved profile

Forge:
  local by default

remote resolver:
  optional explicit service

training contribution:
  separate explicit action
~~~

The proposed resolver experiment should also measure:

- `NO_MATCH` correctness;
- candidate-order stability;
- override rate;

not top-1 accuracy alone.

### Package / community profile workflow

A community profile change can alter behavior without changing translator code.

Profile PRs should therefore expose:

- semantic subject;
- authority class;
- behavior-bearing diff;
- invalidated evidence;
- new immutable profile identity.

This does not require a complex package registry before Signet 2 can proceed.

It only requires the profile format to leave room for these distinctions.

## Concrete questions for maintainers

The highest-value decisions are:

1. Do you agree that semantic labels need stable versioned identity beyond strings?
2. Should capability declarations be treated as offers that feed a negotiated session contract?
3. Should reusable profiles declare semantic acceptability while the session declares preference?
4. Should optional semantic profiles be activated in explicit deterministic preference order?
5. Should resolver APIs support `NO_MATCH` / abstention?
6. Should publisher profile data and participant-owned pins be separate layers?
7. Should a pinned decision have an explicit compatibility-freshness state?
8. Should local approval and contribution to the Forge training dataset be separate actions?
9. Should Signet publish executable semantic composition / negotiation vectors before the model experiment?

Agreement on those questions would be enough to narrow the next draft substantially.

## Patch-ready proposal

A three-part documentation patch series has been prepared against the exact current upstream snapshot.

It is intentionally staged:

~~~text
PATCH 1:
  harden current draft wording and roadmap

PATCH 2:
  additive semantic-contract/profile-composition proposal

PATCH 3:
  additive translator trust/validation proposal
~~~

The series was mechanically replayed against:

~~~text
kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2
~~~

with:

~~~text
APPLY_CHECK = PASS
~~~

This means the unified-diff context matches that snapshot. It is not a claim that upstream docs CI or maintainer review has passed.

## Full supporting material

The public response package contains:

- a concise maintainer handoff;
- an upstream-section delta matrix;
- the patch-ready series;
- the full technical response;
- supporting L1-L4 research and experiments.

The intent is to make the feedback falsifiable and easy to split, not to ask Signet to accept one giant design wholesale.
