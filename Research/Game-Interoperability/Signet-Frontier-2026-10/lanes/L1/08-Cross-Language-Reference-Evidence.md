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
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
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

Nine negative vectors require exact machine-readable failures.

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

Both implementations returned the exact expected reason for all nine vectors.

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

## Additional composition evidence

Five additional positive vectors now cover:

1. selection-order invariance;
2. duplicate required-dependency paths coalescing one exact profile;
3. optional dependencies remaining inactive merely because they are available;
4. explicit activation of an optional dependency;
5. valid explicit concept extension through a distinct semantic ID.

Two additional negative vectors cover:

- active optional dependency with the wrong declared definition hash;
- extension to a missing parent semantic concept.

The full negotiated contract is also canonically hashed. Both implementations reproduce:

~~~text
sha256:49539b9e09a204fe8d1a5a44f247247a19c591f66d902623d62695e2819fe2b0
~~~

for a contract binding core, profiles, ordering model, ruleset, visibility policy, fallbacks, and authority assignments.

## Remaining implementation gaps

The strongest next vectors are now:

1. ordering-model intersection where several models remain possible before session selection;
2. visibility-policy intersection before session selection;
3. optional dependency activation chosen by negotiation rather than preselected roots;
4. ruleset dependency composition;
5. Unicode edge cases from RFC 8785;
6. explicit set-array source-canonicality rejection;
7. a second canonicalizer using a mature external JCS library as an external cross-check;
8. third independent implementation that does not copy either reference algorithm.

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

# Correction pass — `correction/l1-cross-language-determinism`

The independent validation run found three latent L1 determinism gaps that the committed
fixtures did not cover (see `Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`). This
section records the correction. The historical evidence above is unchanged.

Base (historical L1 head, not rewritten):

~~~text
b8f47a332fcb0aa641cfe676e22b49a801731628
~~~

Corrections applied on the correction branch:

1. **One normative comparator.** Added `HASH-2a` / `SEM-ORDER-1` to
   `07-Canonical-Hashing-and-Profile-Composition.md`: semantic references are compared
   as `(id, definition_hash)` over UTF-16 code units, matching RFC 8785 object-property
   ordering. Python previously used Unicode code-point order; JavaScript used UTF-16
   order. Both now implement `SEM-ORDER-1`.
2. **Non-BMP adversarial vectors.** Added `composition-unicode-vectors.json`
   (`COMP-UNICODE-001`): U+10000 (`D800 DC00`) must sort before U+E000. The historical
   Python reference emits the opposite order and fails this vector; the historical
   JavaScript reference passes it. After the correction both pass identically.
3. **Deterministic reason for unordered input.** Added `COMP-1a` to `07`: all
   semantically unordered set arrays are canonicalized before evaluation, and the first
   violation in a fixed phase order wins. Added
   `composition-set-order-vectors.json` (`COMP-NEG-SET-ORDER-001a/001b`): the same
   selected-profile set in both source orders must reject with
   `SEMANTIC_DEFINITION_CONFLICT`. Both historical references returned
   `REQUIRED_PROFILE_UNSUPPORTED` for the reversed order.
4. **HASH05 enforced.** Both references now validate source canonicality of
   schema-declared set arrays before hashing and reject with
   `SET_ARRAY_NOT_CANONICAL`. Added `hash-set-array-vectors.json` with positive and
   negative cases, including a non-BMP array that is sorted by code point but not by
   `SEM-ORDER-1`.

Re-run commands:

~~~bash
bash reference/run-all.sh
python3 reference/python/reference.py schema/test-vectors
node reference/javascript/reference.mjs schema/test-vectors
~~~

Corrected output, both languages (Python 3.14.7, Node v26.10.0):

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: NEGOTIATION-VECTOR-001
PASS: OPTIONAL-NEGOTIATION-VECTOR-001
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
PASS: 7 HASH05 set-array vectors
PASS: 1 non-BMP composition vectors
PASS: 2 deterministic set-order negative vectors
~~~

The full pre-existing corpus still passes unchanged.

# Conclusion

The L1 semantic-contract work has crossed from architecture into executable interoperability evidence.

The next useful disagreement is no longer “what did the prose mean?”

It is whether a third independent implementation can pass the same vectors without private clarification.
