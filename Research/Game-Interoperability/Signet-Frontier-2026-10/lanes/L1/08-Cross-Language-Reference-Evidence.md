# L1 Continuation 08 — Cross-Language Reference Evidence

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 6  
**Research cutoff:** 2026-10-05  
**Status:** executable research evidence, not a Signet implementation

## Purpose

The previous L1 pass defined canonical semantic-definition hashing and deterministic profile composition.

This pass tests whether those rules are precise enough for two independently written implementations to produce the same results.

## Implementations

Python:

`reference/python/reference.py`

JavaScript:

`reference/javascript/reference.mjs`

Test vectors:

`schema/test-vectors/`

## Local execution environment

The committed source was executed locally before publication with:

~~~text
Python 3.13.5
Node v22.16.0
~~~

Both implementations produced:

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: 7 negative composition vectors
~~~

## Positive evidence

### HASH-VECTOR-001

Both implementations independently produced the published canonical JCS string and:

~~~text
sha256:2ba8cba596be0c3cc843d9f35441a93513df8179952d15e1539332c50fac6989
~~~

This covers:

- UTF-16 property-name ordering;
- deterministic JSON string serialization for the exercised character set;
- nested arrays/objects;
- SHA-256 over canonical UTF-8 bytes.

### COMP-VECTOR-001

Both implementations independently:

1. started with only the selected `card-zones@1` profile;
2. resolved its required `turn-taking@1` dependency;
3. unioned the exported concepts;
4. checked the selected turn-sequence ordering model;
5. sorted the resulting profile and concept sets canonically;
6. generated the published profile-set hash:

~~~text
sha256:a9ca960cc1eec57fb925e44c3cf720ad7bd79ccd69828ae11cbf7d03ba7624b0
~~~

This is stronger evidence than simply hashing a preconstructed expected object because the composition closure is actually derived from the input catalog.

## Negative evidence

Seven negative vectors require exact machine-readable failures.

### COMP-NEG-001

Same semantic profile ID, different definition hash:

~~~text
SEMANTIC_DEFINITION_CONFLICT
~~~

### COMP-NEG-002

Required dependency cycle:

~~~text
REQUIRED_PROFILE_CYCLE
~~~

### COMP-NEG-003

Two activated profiles export the same concept ID with different hashes:

~~~text
CONCEPT_DEFINITION_CONFLICT
~~~

### COMP-NEG-004

Selected ordering model is incompatible with one activated profile:

~~~text
ORDERING_MODEL_CONFLICT
~~~

### COMP-NEG-005

Two activated profiles explicitly conflict:

~~~text
PROFILE_CONFLICT
~~~

### COMP-NEG-006

Selected visibility policy is incompatible with one activated profile:

~~~text
VISIBILITY_POLICY_CONFLICT
~~~

### COMP-NEG-007

Required profile dependency is unavailable:

~~~text
REQUIRED_PROFILE_UNSUPPORTED
~~~

Both implementations returned the exact expected reason for all seven vectors.

## Important limitation: this is a restricted JCS subset

The references do **not** claim to be complete general-purpose RFC 8785 implementations.

They currently support normative JSON values containing:

- strings;
- booleans;
- null;
- arrays;
- objects;
- integers inside the interoperable safe-integer range.

They reject floating-point normative values.

This is intentional.

The current semantic-definition and composition fixtures do not require binary floating-point numbers, and rejecting them avoids accidentally publishing an incomplete ECMAScript number serializer as standards-grade JCS.

A later pass can either:

1. adopt a well-audited full RFC 8785 implementation; or
2. keep normative semantic definitions intentionally free of JSON floating-point values.

The second option may be attractive for protocol definitions where exact decimals/quantities can be represented as strings or rationals.

## Why this matters

Before this pass, the L1 composition model was mechanically specified but not independently exercised.

After this pass:

~~~text
same vectors
    ->
two codebases
    ->
two runtimes
    ->
same canonical bytes
    ->
same hashes
    ->
same closure
    ->
same rejection reasons
~~~

This is the first executable evidence that the proposed coordination semantics are precise enough to implement independently.

It does **not** prove the design is complete or correct.

It does prove that the currently tested subset is not dependent on one hidden implementation.

## Remaining implementation gaps

The strongest next vectors are:

1. selection-order invariance with three or more profiles;
2. duplicate dependency paths coalescing the same profile;
3. valid explicit concept extension;
4. invalid extension target rejection;
5. ordering-model intersection where several models remain possible before session selection;
6. visibility-policy intersection;
7. optional dependency activation/non-activation;
8. full negotiated `contract_hash`, not only `profile_set_hash`;
9. Unicode edge cases from RFC 8785;
10. a second canonicalizer implementation using a mature external JCS library as an external cross-check.

## L4 handoff

~~~yaml
handoff:
  from_lane: L1
  to_lane: L4
  finding: Two independent reference implementations now agree on canonical semantic hashes, profile dependency closure, composition output, and seven deterministic failure vectors.
  evidence:
    - lanes/L1/reference/python/reference.py
    - lanes/L1/reference/javascript/reference.mjs
    - lanes/L1/schema/test-vectors/hash-vector-001.json
    - lanes/L1/schema/test-vectors/composition-vector-001.json
    - lanes/L1/schema/test-vectors/composition-negative-vectors.json
  requested_followup: Reimplement the vectors independently without reusing either reference implementation, then extend the matrix to optional dependencies, extension edges, contract hashes, and Unicode/JCS edge cases.
~~~

# Conclusion

The L1 semantic-contract work has crossed from architecture into executable interoperability evidence.

The next useful disagreement is no longer “what did the prose mean?”

It is whether a third independent implementation can pass the same vectors without private clarification.
