# L1 Continuation 10 — Deterministic Optional-Profile Negotiation

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 9  
**Research cutoff:** 2026-10-05  
**Status:** executable research proposal

## Problem

Required profile composition is already deterministic.

Optional profiles are harder because enabling one optional module can:

- introduce new required dependencies;
- collide with an existing concept;
- narrow the valid ordering-model set;
- narrow the valid visibility-policy set;
- conflict with another optional profile.

Therefore:

~~~text
"optional"
!=
"enable if installed"
~~~

If implementations independently decide which optionals to activate, they can derive different semantic contracts from the same participants.

## Session policy

The session publishes an explicit ordered list:

~~~yaml
optional_profile_preferences:
  - id: https://example.org/sem/profile/optional-event/1
    definition_hash: sha256:...

  - id: https://example.org/sem/profile/optional-fixed/1
    definition_hash: sha256:...
~~~

Order is policy.

Earlier candidates have priority.

## Algorithm

Start with the required selected profile roots.

### Step 1 — Compose required baseline

If the required baseline fails, negotiation fails.

Optional features never rescue an invalid required contract.

### Step 2 — Attempt each optional candidate in preference order

For candidate N:

1. tentatively add it to the already accepted profile roots;
2. resolve all of its required dependencies;
3. run the complete composition algorithm;
4. rerun ordering-model selection;
5. rerun visibility-policy selection.

If the tentative set succeeds:

~~~text
activate candidate
~~~

If it fails:

~~~text
skip candidate
record exact reason
continue
~~~

### Step 3 — Earlier accepted optionals stay preferred

The algorithm is deliberately greedy.

If optional A is accepted first and later optional B conflicts with A:

~~~text
keep A
skip B
~~~

This is not claimed to find a mathematically maximal optional feature set.

It follows the **session's declared preference order**, which is the relevant policy.

A more complex optimization algorithm would make the protocol harder to reason about and would need its own explicit objective function.

## Example vector

Published:

~~~text
schema/test-vectors/optional-negotiation-vector-001.json
~~~

Required profile:

~~~text
required@1
~~~

Optional preference order:

~~~text
1. optional-event@1
2. optional-fixed@1
3. missing@1
~~~

Session ordering preference:

~~~text
1. fixed-tick@1
2. event-sequence@1
~~~

Semantic constraints:

~~~text
required@1 accepts:
  fixed-tick
  event-sequence

optional-event@1 accepts:
  event-sequence

optional-fixed@1 accepts:
  fixed-tick
~~~

Result:

~~~text
activate optional-event@1

skip optional-fixed@1
  reason = ORDERING_MODEL_CONFLICT

skip missing@1
  reason = REQUIRED_PROFILE_UNSUPPORTED

selected ordering:
  event-sequence@1
~~~

The final result hash is:

~~~text
sha256:5df2fb5f3c1660aa844027f1980d0a0a9165d539650c9414a7e698b307860dd2
~~~

## A useful fixture failure

The first published expected result accidentally listed the final profiles in non-canonical order.

Local execution caught the mismatch.

The implementations were correct; the fixture was wrong.

The vector was corrected before this pass was recorded.

This is exactly why executable canonical vectors are useful: they detect mistakes in the research artifact itself, not only in implementations.

## Machine-readable output

The candidate core schema now has:

~~~text
session requirements:
  optional_profile_preferences

negotiated contract:
  activated_optional_profiles
  skipped_optional_profiles:
    profile
    reason
~~~

This means optional degradation is inspectable.

Two peers do not merely know that they "mostly support" a session.

They can compare the exact activated/skipped semantic modules.

## Complete current local reference output

Both Python 3.13.5 and Node.js 22.16.0 produce:

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: NEGOTIATION-VECTOR-001
PASS: OPTIONAL-NEGOTIATION-VECTOR-001
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
~~~

## Remaining semantic-policy gaps

The remaining high-value problems are narrower:

1. ruleset dependency composition;
2. normative source-canonical set-array validation, not only canonical output;
3. Unicode edge-case vectors for RFC 8785;
4. complete floating-point policy;
5. external JCS implementation cross-check;
6. third independent implementation;
7. migration mapping from concrete Signet 2 intent/archetype vocabulary into the proposed profile structure.

## Conclusion

Optional semantics must be policy-driven, not implementation-driven.

The resulting rule is:

~~~text
required profiles:
  must compose or session fails

optional profiles:
  attempt in explicit preference order
  keep when full candidate contract remains valid
  otherwise skip with exact reason
~~~

This keeps graceful degradation deterministic without turning optional features into hidden semantic divergence.
