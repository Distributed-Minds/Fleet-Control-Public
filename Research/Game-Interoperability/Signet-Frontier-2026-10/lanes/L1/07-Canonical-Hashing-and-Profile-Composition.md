# L1 Continuation 07 — Canonical Definition Hashing and Profile Composition

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 5  
**Research cutoff:** 2026-10-05  
**Status:** unaffiliated design proposal

## Executive result

The remaining L1 problems can be reduced to two deterministic contracts:

1. **definition identity** — independent tools must compute the same digest for the same normative semantic definition;
2. **profile composition** — independent tools must either compose the same selected profile set or reject it for the same reason.

The proposed rules are deliberately conservative:

- normative definitions are JSON;
- hash only the normative semantic object, not annotations/documentation;
- require I-JSON-compatible data;
- require canonical ordering for arrays that semantically represent sets;
- apply RFC 8785 JSON Canonicalization Scheme (JCS);
- hash the UTF-8 JCS bytes with SHA-256;
- duplicate semantic identifiers are allowed only when their definition hashes are identical;
- profiles may depend on and extend other profiles, but may not silently mutate an imported definition;
- required profile composition is atomic: either the complete selected closure composes or admission fails.

Primary canonicalization source:

- RFC 8785 — JSON Canonicalization Scheme  
  https://www.rfc-editor.org/rfc/rfc8785.html

Primary composition precedent:

- Möller, Löfstrand, Karlsson, *An overview of the HLA evolved modular FOMs* (2007)
- HLA modular FOM principles summarized there:
  - exact equivalence for duplicate singleton/table definitions;
  - union of distinct elements, with duplicate identifiers required to be equivalent;
  - hierarchical extension without mutating the already-defined parent;
  - atomic module loading/failure when selected modules cannot be combined.

The Signet proposal should use these as precedent, not import HLA's full object model.

---

# 1. Canonical semantic definition envelope

A published semantic definition should separate normative machine meaning from mutable human material.

Candidate shape:

~~~json
{
  "kind": "signet.semantic-definition",
  "normative": {
    "semantic_id": "https://signetprotocol.io/sem/profile/turn-taking/1",
    "definition_kind": "profile",
    "requires": [],
    "exports": [
      "https://signetprotocol.io/sem/profile/turn-taking/1#event/move-applied",
      "https://signetprotocol.io/sem/profile/turn-taking/1#intent/move",
      "https://signetprotocol.io/sem/profile/turn-taking/1#state/board"
    ]
  },
  "annotations": {
    "label": "Turn taking",
    "status": "draft",
    "documentation": "..."
  }
}
~~~

Only `normative` is hashed.

`annotations` can change without creating a semantic-definition conflict.

This lets projects fix:

- spelling;
- translated labels;
- examples;
- prose explanations;
- source links;

without pretending the semantic contract itself changed.

---

# 2. Hashing algorithm

## HASH-1 — Parse as JSON

The normative member MUST be valid JSON and MUST satisfy the I-JSON constraints relied on by RFC 8785.

Reject:

- duplicate object keys;
- NaN / Infinity;
- invalid Unicode;
- implementation-specific non-JSON types.

Do not hash YAML source bytes.

YAML may be used as authoring input only if it is first converted into the normative JSON data model unambiguously.

## HASH-2 — Enforce semantic collection ordering

RFC 8785 sorts object properties but **does not reorder arrays**.

Therefore every normative schema member must declare whether an array is:

- an **ordered sequence**, where order is semantic; or
- a **set**, where order is non-semantic.

For arrays declared as sets:

1. items MUST be unique;
2. items MUST be sorted by the canonical comparison key defined in HASH-2a;
3. a definition containing an unsorted set-array MUST be rejected rather than silently reinterpreted.

Examples:

~~~text
requires[]         -> set, order by SEM-ORDER-1
optional_requires[]-> set, order by SEM-ORDER-1
conflicts[]        -> set, order by SEM-ORDER-1
exports[]          -> set, order by SEM-ORDER-1
extends[]          -> set, order by (child, parent) using the UTF-16 string rule
authority_scopes[] -> set, order by the UTF-16 string rule
ordering_constraints.accepted[]   -> set, order by SEM-ORDER-1
visibility_constraints.accepted[] -> set, order by SEM-ORDER-1
phases[]           -> ordered sequence if the phase order is semantic
tuple fields       -> ordered sequence
~~~

Rejecting unsorted set arrays is preferable to hidden normalization because the source file itself remains reviewably canonical.

## HASH-2a — Semantic-reference comparator (SEM-ORDER-1)

The earlier draft said each set array was ordered by "the member's specified canonical
comparison key" without pinning that key. That left one degree of freedom, and two
conforming implementations could disagree on non-BMP identifiers:

- Unicode **code-point** order compares U+E000 before U+10000;
- **UTF-16 code-unit** order (the order RFC 8785 uses for object property names)
  compares U+10000 (surrogate pair `D800 DC00`) before U+E000.

This section removes that freedom.

A **semantic reference** is the JSON object pair `(id, definition_hash)`. Two semantic
references MUST be compared as follows:

1. compare `id` as a sequence of **UTF-16 code units**, exactly as RFC 8785 §3.2.3
   orders object property names (equivalently, ECMAScript `<` on strings): the first
   differing code unit decides; if one sequence is a proper prefix of the other, the
   shorter sequence sorts first;
2. if the `id` values are equal, compare `definition_hash` by the same UTF-16
   code-unit rule.

A **bare semantic ID** (a `semanticId` string, e.g. in `authority_scopes[]`) is compared
by the same UTF-16 code-unit rule.

This is the ONLY canonical comparison key for schema-declared set arrays. Implementations
MUST NOT use Unicode code-point order, locale/collation order, or code-unit order
truncated to the BMP.

`SEM-ORDER-1` is therefore *consistent with* RFC 8785 object-property ordering rather
than a second ordering regime: both order strings by UTF-16 code units. Under
`SEM-ORDER-1` a non-BMP code point in the range U+10000..U+10FFFF sorts **before**
BMP code points U+E000..U+FFFF.

Set arrays of semantic references MUST be strictly ascending under `SEM-ORDER-1` and
MUST NOT contain two entries with the same `id` (even when their `definition_hash`
values differ). This is the `HASH05` rejection rule; its deterministic reason code is:

~~~text
SET_ARRAY_NOT_CANONICAL
~~~


## HASH-3 — Keep exact numerics out of binary-float ambiguity

RFC 8785 canonicalizes JSON numbers using ECMAScript-compatible serialization.

For normative values where exact decimal identity matters, do not rely on implementation-specific arbitrary-precision parsing.

Recommended encoding:

~~~yaml
exact integer within interoperable JSON range:
  20

exact decimal / rational:
  "0.1"
  or
  { "numerator": 1, "denominator": 10 }

quantity:
  { "value": "0.1", "unit": "https://example.org/unit/metre" }
~~~

Profile schemas can impose stronger numeric rules.

## HASH-4 — Keep human prose out of the normative object

JCS deliberately does not perform Unicode normalization.

That is acceptable if normative semantics are expressed through controlled machine fields and semantic identifiers rather than prose.

Human labels/descriptions belong in annotations.

If a future normative field genuinely depends on a Unicode string, the exact code-point sequence is part of the definition unless that field defines an additional normalization rule.

## HASH-5 — Apply RFC 8785 JCS

Canonicalize the normative JSON object according to RFC 8785.

This provides:

- deterministic JSON primitive serialization;
- recursively sorted object property names;
- preserved array order;
- canonical UTF-8 output suitable for hashing.

## HASH-6 — SHA-256

Compute:

~~~text
digest = SHA-256(UTF8(JCS(normative)))
definition_hash = "sha256:" + lowercase_hex(digest)
~~~

The digest algorithm should be an explicit part of the reference string so a future protocol can introduce another digest without ambiguity.

## HASH-7 — Same semantic ID, different hash fails closed

If two participants advertise:

~~~text
same semantic_id
different definition_hash
~~~

the result is:

~~~text
SEMANTIC_DEFINITION_CONFLICT
~~~

No “newer wins,” textual similarity, or model-based reconciliation occurs in the runtime negotiation path.

---

# 3. Canonical hashing test vector

The published fixture is itself valid against the candidate semantic-definition schema.

Normative object:

~~~json
{
  "semantic_id": "https://example.org/sem/profile/example/1",
  "definition_kind": "profile",
  "requires": [],
  "optional_requires": [],
  "conflicts": [],
  "exports": [
    {
      "id": "https://example.org/sem/profile/example/1#intent/a",
      "definition_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    },
    {
      "id": "https://example.org/sem/profile/example/1#state/b",
      "definition_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }
  ]
}
~~~

Required JCS byte sequence:

~~~text
{"conflicts":[],"definition_kind":"profile","exports":[{"definition_hash":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","id":"https://example.org/sem/profile/example/1#intent/a"},{"definition_hash":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","id":"https://example.org/sem/profile/example/1#state/b"}],"optional_requires":[],"requires":[],"semantic_id":"https://example.org/sem/profile/example/1"}
~~~

SHA-256:

~~~text
sha256:2ba8cba596be0c3cc843d9f35441a93513df8179952d15e1539332c50fac6989
~~~

Changing:

- whitespace;
- object property order;
- annotations outside the normative member;

MUST NOT change that hash.

Changing the order of `exports` **would** change JCS bytes, which is why the semantic-definition schema requires that set-valued array to already be sorted canonically.

---

# 4. Definition immutability rule

Once a semantic definition is published as stable:

~~~text
semantic_id + definition_hash
~~~

is immutable.

A normative meaning change requires a new semantic identity/version.

A project may keep a human-friendly major in the URI:

~~~text
.../profile/fps-combat/1
.../profile/fps-combat/2
~~~

and use the digest to guarantee that all participants claiming version 1 actually loaded the same normative version-1 definition.

During unstable drafting, use an explicitly experimental/draft identifier rather than repeatedly mutating a supposedly stable ID.

OpenXR's provisional/experimental extension process is useful precedent: experimental iterations do not promise compatibility, while stabilized extensions do.

---

# 5. Profile-definition model

A profile is a module of domain meaning, not a bag of arbitrary patches.

Candidate normative fields:

~~~yaml
semantic_id: <profile ID>
definition_kind: profile

requires:
  - id: <required profile ID>
    definition_hash: <exact accepted hash>

optional_requires:
  - id: <optional profile ID>
    definition_hash: <accepted hash>

conflicts:
  - id: <incompatible profile ID>
    definition_hash: <specific hash or constrained identity>

exports:
  - <concept semantic ID>

extends:
  - child: <new concept ID>
    parent: <imported concept ID>

ordering_constraints:
  accepted:
    - <ordering model semantic ref>

visibility_constraints:
  accepted:
    - <visibility-policy semantic ref>

authority_scopes:
  - <semantic scope ID>
~~~

The exact schema may evolve; the composition semantics below are the important part.

---

# 6. Profile composition invariants

## COMP-1 — Selected profiles form a set

The selected profile list is semantically unordered.

Normalize it by exact semantic reference:

~~~text
semantic_id + definition_hash
~~~

before composition.

## COMP-1a — Deterministic evaluation of unordered collections

Several composition inputs are declared semantically unordered sets (COMP-1), while
others are ordered sequences whose order is policy:

~~~text
unordered (MUST be canonicalized before evaluation):
  selected_profiles[]
  requires[]
  optional_requires[]
  conflicts[]
  exports[]
  extends[]
  ordering_constraints.accepted[]
  visibility_constraints.accepted[]
  authority_scopes[]

ordered (order is semantic; MUST NOT be reordered):
  ordering_preferences[]
  visibility_preferences[]
  optional_profile_preferences[]
  phases[] and other explicitly ordered sequences
~~~

Before evaluating a composition, an implementation MUST canonicalize every unordered
collection into ascending `SEM-ORDER-1` order (bare strings by the UTF-16 string rule).
It MUST then traverse the canonicalized collections.

Consequently the reported failure reason for a given input object graph MUST NOT depend
on the source order of any unordered collection. Two JSON inputs that differ only by a
permutation of an unordered set array MUST produce the same composition result or the
same rejection reason.

When more than one violation is present, the first violation reached by the canonical
evaluation wins. Composition evaluates in this fixed phase order:

~~~text
1. selected-reference resolution and required closure
     REQUIRED_PROFILE_UNSUPPORTED
     SEMANTIC_DEFINITION_CONFLICT
     REQUIRED_PROFILE_CYCLE
2. active optional-dependency hash check
     SEMANTIC_DEFINITION_CONFLICT
3. explicit profile conflicts
     PROFILE_CONFLICT
4. exported concept collisions
     CONCEPT_DEFINITION_CONFLICT
5. extension-edge validation
     INVALID_EXTENSION_TARGET
6. ordering-model constraints
     ORDERING_MODEL_CONFLICT
7. visibility-policy constraints
     VISIBILITY_POLICY_CONFLICT
~~~

Within phase 1, every `selected_profiles[]` entry and every `requires[]` array is
visited in `SEM-ORDER-1` order, so the reason is total and order-independent.

The `optional_profile_preferences[]` list is deliberately ordered policy (see 10);
`negotiate_optional` MUST preserve its order and MUST NOT sort it.


## COMP-2 — Expand the required dependency closure

For every selected profile:

1. recursively add all `requires` dependencies;
2. verify exact semantic IDs/hashes;
3. fail if any required dependency is unavailable;
4. reject required-dependency cycles.

Optional dependencies are activated only if session selection/policy chooses them.

A dependency being installed locally does not automatically activate it.

## COMP-3 — Duplicate profile identity requires exact equivalence

If the composition closure contains the same semantic ID more than once:

~~~text
same ID + same hash -> one definition
same ID + different hash -> SEMANTIC_DEFINITION_CONFLICT
~~~

This mirrors HLA modular FOM's useful “duplicate identifiers must be equivalent” principle.

## COMP-4 — Export union; duplicate concept IDs must be equivalent

The composite concept surface is the union of exported concepts.

For duplicate concept IDs:

~~~text
same concept ID + same definition hash -> one concept
same concept ID + different hash -> CONCEPT_DEFINITION_CONFLICT
~~~

Do not merge field lists heuristically.

## COMP-5 — No monkey-patching imported concepts

A profile MUST NOT add fields, change units, change authority category, change cardinality, or change meaning of an already imported concept under the same semantic ID.

To extend a concept, define a **new semantic concept ID** with an explicit `extends` relationship.

This is directly analogous to the HLA modular FOM rule that a module cannot add attributes to an already-defined class merely by repeating the class with extra attributes; extension is represented structurally instead.

## COMP-6 — Explicit profile conflicts fail

If activated profile A declares profile B incompatible, selecting both is a hard composition failure unless a later standardized mechanism explicitly supersedes that rule.

Candidate reason:

~~~text
PROFILE_CONFLICT
~~~

## COMP-7 — Ordering-model constraints compose by intersection

A profile may declare the ordering models under which its semantics are valid.

Example:

~~~text
profile A accepts:
  fixed-tick@1
  event-sequence@1

profile B accepts:
  event-sequence@1
  turn-sequence@1
~~~

Composite acceptable set:

~~~text
event-sequence@1
~~~

If the intersection is empty:

~~~text
ORDERING_MODEL_CONFLICT
~~~

A session-selected ordering model must be inside the resulting intersection.

## COMP-8 — Visibility-policy constraints compose by intersection

Apply the same rule to required disclosure semantics.

If one profile requires private state and another session policy can only broadcast all state, they are not pragmatically compatible.

Candidate failure:

~~~text
VISIBILITY_POLICY_CONFLICT
~~~

## COMP-9 — Authority semantics do not assign runtime issuers

Profiles may define authority **scopes/categories**.

They do not decide that a particular participant ID is authoritative.

The session contract assigns actual issuers to scopes after composition.

This prevents reusable domain definitions from embedding one deployment topology.

## COMP-10 — Required composition is atomic

Composition of the selected required profile set either:

- succeeds completely; or
- fails completely with deterministic reason(s).

Do not silently drop a required profile to make the remainder compose.

This follows the useful HLA modular FOM property that a selected module load succeeds as a whole or fails on conflict.

Optional profiles may be omitted before the activated set is finalized.

Once the negotiated contract records an activated profile set, that exact set is the runtime contract.

---

# 7. Deterministic composition algorithm

Given:

~~~text
selected profiles
available profile definitions
session ordering/visibility constraints
~~~

perform:

## Step 1 — Canonicalize inputs

Sort selected semantic refs by:

~~~text
semantic_id
definition_hash
~~~

## Step 2 — Resolve required closure

Depth-first or breadth-first is implementation-specific.

The output MUST be the same canonical set.

Detect:

- missing dependencies;
- dependency cycles;
- same-ID/different-hash conflicts.

## Step 3 — Build concept union

For every exported concept:

- add new IDs;
- coalesce exact duplicate ID/hash;
- reject duplicate ID/different-hash.

## Step 4 — Validate extension edges

Every `extends.parent` must exist in the closure or in an explicitly allowed core definition.

The child must have a distinct semantic ID.

No in-place mutation is allowed.

## Step 5 — Check explicit conflicts

Evaluate profile conflict declarations over the complete closure.

## Step 6 — Intersect singleton/session constraint domains

Compute compatible sets for:

- ordering model;
- visibility policy;
- any future core-negotiated singleton dimensions.

An empty required intersection is a hard conflict.

## Step 7 — Validate selected session choices

The session's:

- ordering model;
- visibility policy;
- ruleset dependencies;

must satisfy the composite profile constraints.

## Step 8 — Produce canonical composition record

Output a deterministic record containing at least:

~~~yaml
profiles:
  - id: ...
    definition_hash: ...

concepts:
  - id: ...
    definition_hash: ...

ordering_model: ...
visibility_policy: ...

composition_result: SUPPORTED
~~~

Sort all set-valued arrays canonically.

## Step 9 — Hash the composition record

The resulting activated semantic surface may itself be hashed with the same JCS + SHA-256 process.

Candidate:

~~~text
profile_set_hash
~~~

The full negotiated contract should separately hash/include:

- profile-set hash;
- ruleset;
- ordering model;
- visibility policy;
- accepted fallbacks;
- authority assignments.

This yields a `contract_hash` that can be logged/replayed without conflating it with any one profile identity.

---

# 8. Conflict taxonomy

Recommended composition-specific failures:

~~~text
SEMANTIC_DEFINITION_CONFLICT
REQUIRED_PROFILE_UNSUPPORTED
REQUIRED_PROFILE_CYCLE
CONCEPT_DEFINITION_CONFLICT
PROFILE_CONFLICT
INVALID_EXTENSION_TARGET
ORDERING_MODEL_CONFLICT
VISIBILITY_POLICY_CONFLICT
RULESET_DEPENDENCY_CONFLICT
~~~

Negotiation may map some of these to the existing top-level:

~~~text
INCOMPATIBLE
~~~

while preserving the machine-readable reason.

---

# 9. Worked composition example

Selected:

~~~text
profile/turn-taking@1
profile/card-zones@1
profile/hidden-information@1
~~~

Dependencies:

~~~text
card-zones@1
  requires turn-taking@1

hidden-information@1
  no gameplay dependency
~~~

Ordering constraints:

~~~text
turn-taking@1:
  [turn-sequence@1]

card-zones@1:
  [turn-sequence@1, event-sequence@1]

hidden-information@1:
  [turn-sequence@1, event-sequence@1]
~~~

Intersection:

~~~text
turn-sequence@1
~~~

Visibility constraints:

~~~text
hidden-information@1 requires a policy capable of participant-private state
~~~

A broadcast-only policy therefore fails:

~~~text
VISIBILITY_POLICY_CONFLICT
~~~

A private-hands policy succeeds.

No position, body, health, weapon, physics tick or 3D concept enters the core.

---

# 10. HLA composition lessons worth retaining

The HLA modular FOM literature provides several useful mature lessons.

## Exact equivalence where two modules define the same singleton

If two modules specify incompatible logical-time representations, composition fails.

Signet analogue:

- the same semantic identity cannot resolve to two definitions;
- session singleton semantics must have a compatible intersection.

## Union distinct elements; duplicates must be equivalent

Signet analogue:

- union profile exports;
- duplicate concept IDs require exact definition equivalence.

## Extend structurally, not by silently changing an existing class

Signet analogue:

- a profile cannot monkey-patch an imported semantic concept;
- create a new concept/profile identity with an explicit relation.

## Atomic loading

Signet analogue:

- required activated profiles compose as one contract or admission fails;
- do not keep a partially composed semantic world after a hard conflict.

These principles are more valuable to Signet than the HLA-specific XML/RTI machinery.

---

# 11. OpenXR lessons worth retaining

OpenXR extension practice adds complementary rules:

- optional functionality is explicitly enabled rather than merely installed;
- extensions carry dependencies/interactions;
- namespace/registration prevents collision;
- experimental extensions explicitly avoid compatibility guarantees;
- stabilized multi-vendor functionality carries stronger conformance expectations.

Signet analogue:

~~~text
available profile
!=
activated profile
~~~

and:

~~~text
experimental semantic definition
!=
stable semantic contract
~~~

---

# 12. Contract-hash recommendation

After profile composition and session negotiation, produce a contract record containing exact semantic references.

Illustrative normative object:

~~~json
{
  "authority": {
    "https://example.org/authority/player-intent/1": "participant:self",
    "https://example.org/authority/session-rules/1": "server:rules"
  },
  "ordering_model": {
    "id": "https://signetprotocol.io/sem/ordering/turn-sequence/1",
    "definition_hash": "sha256:..."
  },
  "profiles": [
    {
      "id": "https://signetprotocol.io/sem/profile/card-zones/1",
      "definition_hash": "sha256:..."
    },
    {
      "id": "https://signetprotocol.io/sem/profile/turn-taking/1",
      "definition_hash": "sha256:..."
    }
  ],
  "ruleset": {
    "id": "https://example.org/sem/ruleset/example-card-game/1",
    "definition_hash": "sha256:..."
  },
  "visibility_policy": {
    "id": "https://example.org/sem/visibility/private-hands/1",
    "definition_hash": "sha256:..."
  }
}
~~~

Apply the same set-order rules + JCS + SHA-256.

The resulting `contract_hash` can be recorded in:

- session logs;
- replay metadata;
- bug reports;
- conformance fixtures.

Two parties with different contract hashes are not running the exact same semantic contract, even if their human-readable session names match.

---

# 13. Conformance additions for L4

## HASH family

### HASH01 — Property-order invariance

Reordering JSON object properties does not change the definition hash.

### HASH02 — Whitespace invariance

Whitespace changes do not change the definition hash.

### HASH03 — Annotation independence

Changing excluded annotations does not change the definition hash.

### HASH04 — Normative change sensitivity

Changing normative semantic content changes the definition hash.

### HASH05 — Set-array source canonicality

Unsorted or duplicate values in schema-declared set arrays are rejected before hashing.
The canonical order is `SEM-ORDER-1` (HASH-2a); the deterministic rejection reason is
`SET_ARRAY_NOT_CANONICAL`. Enforcement is exercised by
`schema/test-vectors/hash-set-array-vectors.json`.


### HASH06 — Cross-language JCS agreement

At least two independent implementations produce the same JCS bytes and SHA-256 for all hash fixtures.

## COMP family

### COMP01 — Selection-order invariance

Selecting the same profile set in a different order produces the same composition record/hash.

### COMP02 — Exact duplicate coalescing

Same profile ID/hash loaded through two dependency paths yields one profile.

### COMP03 — Divergent duplicate rejection

Same profile ID with different hashes rejects deterministically.

### COMP04 — Concept collision rejection

Same exported concept ID with different hashes rejects.

### COMP05 — No monkey-patching

A dependent profile cannot add/change members of an imported concept under the same semantic ID.

### COMP06 — Dependency closure determinism

Independent implementations derive the same required dependency closure.

### COMP07 — Cycle rejection

Required dependency cycles reject with the same reason.

### COMP08 — Ordering intersection

Profile ordering constraints are intersected deterministically.

### COMP09 — Visibility intersection

Profile disclosure constraints are intersected deterministically.

### COMP10 — Atomic failure

A hard conflict cannot result in a partially activated required profile set.

### COMP11 — Contract hash agreement

Independent implementations produce the same contract hash for the same negotiated contract.

---

# 14. Consequence for the response paper

The Signet 2 response can now make a stronger, narrower recommendation:

> A translation profile should not merely pin local choices. It should pin choices **against exact semantic definitions**, and the activated profile set should be composed under deterministic equivalence/dependency rules before the session starts.

This preserves the simplicity of Signet Forge.

Forge remains:

~~~text
discover
suggest
review
pin
validate
publish
~~~

but `pin` now records an exact target semantic definition, and `validate` includes profile-composition/conformance evidence.

---

# 15. Next implementation milestone

L1's semantic design is now sufficiently concrete for a small reference implementation.

Minimum implementation:

1. JCS + SHA-256 definition hashing;
2. semantic-definition validator;
3. profile dependency resolver;
4. deterministic composition engine;
5. contract-hash generator;
6. fixtures for HASH01–HASH06 and COMP01–COMP11.

Do this twice, independently, preferably in two languages.

The success condition is not that both programs share code.

The success condition is:

> given the same semantic definitions and session requirements, both produce the same activated profile closure, failures, canonical bytes, and hashes.

That would be the first strong executable evidence that the proposed semantic layer is implementable rather than merely well-written prose.
