# L1 cross-language reference implementation

This directory contains two deliberately independent implementations of the current L1 semantic-definition hashing and profile-composition rules:

- `python/reference.py`
- `javascript/reference.mjs`

They consume the vectors in:

`../schema/test-vectors/`

Run:

~~~bash
python3 python/reference.py ../schema/test-vectors
node javascript/reference.mjs ../schema/test-vectors
~~~

Expected output from both:

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

## What is implemented

Both references independently implement:

- restricted RFC 8785/JCS-compatible canonicalization for the current semantic fixtures;
- SHA-256 definition/profile-set hashing;
- required dependency closure;
- required dependency cycle rejection;
- exact profile-definition hash checking;
- profile conflict rejection;
- exported concept collision rejection;
- extension-edge validation;
- ordering-model constraint checks;
- visibility-policy constraint checks;
- deterministic canonical profile/concept ordering;
- `SEM-ORDER-1` semantic-reference ordering (UTF-16 code units), applied to every
  schema-declared set array;
- deterministic evaluation of semantically unordered inputs (`COMP-1a`);
- `HASH05` source-canonicality validation of set arrays with reason
  `SET_ARRAY_NOT_CANONICAL`;
- ordered session preference negotiation for ordering/visibility models;
- deterministic optional-profile preference activation/skipping;
- full negotiated contract hashing.

## Deliberate restriction

These references currently reject floating-point normative values.

Supported normative JSON values are:

- strings;
- booleans;
- null;
- arrays;
- objects;
- integers inside the interoperable safe-integer range.

This keeps the first cross-language evidence small and exact. A later implementation may adopt a complete RFC 8785 number serializer.

## Why two implementations

The point is not language coverage.

The point is to prove that the prose rules are precise enough for independently written implementations to derive identical:

- canonical bytes;
- SHA-256 hashes;
- dependency closures;
- composition outputs;
- rejection reasons.

If the two implementations disagree, the contract or test vectors are underspecified.
