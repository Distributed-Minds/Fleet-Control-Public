# L1 Continuation 09 — Deterministic Pre-Selection Negotiation

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 8  
**Research cutoff:** 2026-10-05  
**Status:** executable research proposal

## Problem

Earlier L1 work could validate an already selected ordering model or visibility policy.

That leaves one question underspecified:

> If several models are acceptable, how does the session choose one without hidden implementation policy?

Lexicographic selection is mechanically deterministic but semantically arbitrary.

“First one my implementation happens to enumerate” is worse.

The session therefore needs to publish an explicit ordered preference list.

## Rule

Profiles contribute **accepted sets**.

The session contributes an **ordered preference list**.

Negotiation selects:

> the first session-preferred semantic reference accepted by every activated profile.

For ordering:

~~~text
session preferences:
  fixed-tick@1
  event-sequence@1
  turn-sequence@1

profile A accepts:
  fixed-tick@1
  event-sequence@1

profile B accepts:
  event-sequence@1
  turn-sequence@1
~~~

Result:

~~~text
event-sequence@1
~~~

For visibility:

~~~text
session preferences:
  public@1
  private@1

profile A accepts:
  public@1
  private@1

profile B accepts:
  private@1
~~~

Result:

~~~text
private@1
~~~

If no session preference is accepted by every activated profile:

~~~text
ORDERING_MODEL_CONFLICT
~~~

or:

~~~text
VISIBILITY_POLICY_CONFLICT
~~~

## Why the session owns preference

Profiles should define semantic validity, not deployment preference.

A profile can say:

> these ordering models preserve my semantics.

It should not normally say:

> always prefer fixed-tick over event-sequence for every application.

That preference belongs to the session/ruleset/operator because it may depend on:

- desired gameplay;
- replay requirements;
- network topology;
- server implementation;
- latency policy;
- tournament policy.

This keeps reusable profile meaning separate from session policy.

## Exact selection remains allowed

The session may still name one exact model:

~~~yaml
ordering_model:
  id: ...
  definition_hash: ...
~~~

That exact choice is validated against every activated profile.

Alternatively it may publish:

~~~yaml
ordering_preferences:
  - id: ...
    definition_hash: ...
  - id: ...
    definition_hash: ...
~~~

The candidate core schema now requires one of those two forms.

For visibility, a session may:

- name one exact policy;
- provide an ordered preference list;
- omit visibility negotiation where activated semantics require no selective disclosure.

## Executable vector

Published:

~~~text
schema/test-vectors/negotiation-vector-001.json
~~~

The vector activates two profiles with overlapping but non-identical accepted sets.

Expected ordering result:

~~~text
event-sequence@1
~~~

Expected visibility result:

~~~text
private@1
~~~

Expected canonical composition hash:

~~~text
sha256:58a4897daca9b857f4caeabcffba293ba3b9c54ce1d08605ebf5c903bb1a5fb2
~~~

Both Python and JavaScript reference implementations reproduce that result.

## Current complete local conformance output

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: NEGOTIATION-VECTOR-001
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
~~~

## Remaining policy problem: optional profile selection

Ordering/visibility selection is now deterministic because the session provides an explicit preference order.

Optional profile activation needs the same treatment.

A future session contract should not merely say:

~~~text
optional:
  profile X
  profile Y
~~~

and leave implementations to decide independently whether to enable both.

Candidate next rule:

~~~text
optional_profiles:
  - profile: X
    policy: prefer
    priority: 10

  - profile: Y
    policy: prefer
    priority: 20
~~~

with an explicit deterministic activation algorithm.

That algorithm must account for the fact that adding an optional profile can shrink the valid ordering/visibility intersection.

This is the next unresolved composition problem.

## Conclusion

The core should not choose semantic models through implementation accident.

It should expose policy.

The resulting distinction is:

~~~text
profile:
  what is semantically acceptable?

session:
  among acceptable choices, what do we prefer?

negotiation:
  first preference accepted by all activated semantics
~~~

That rule is small, deterministic, inspectable, and independently testable.
