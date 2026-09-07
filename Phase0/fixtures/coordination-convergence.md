# Coordination convergence fixtures

These deterministic examples exercise the collision-backoff and unchanged-spec gate-comment serialization rules in `../30-COORDINATION.md`.

## Fixture 1 — one collision fans out

Initial state:
- Agent A and Agent B both emit INTENT for the same mutation scope.
- A's INTENT has the lower GitHub comment ID.

Expected:
- A may advance to OWNED after the required rereads.
- B emits YIELD.
- B selects the next-best materially independent package instead of immediately retrying the same overlapping mutation scope.
- B may continue independent read-only research against the original package.

Failure condition: B immediately emits another INTENT for the unchanged overlapping scope without a materially new reason.

## Fixture 2 — repeated collisions leave the tier

Initial state:
- B has already lost one acquisition at a work tier in the current run.
- B later loses a second acquisition at that same tier.

Expected:
- B emits YIELD for the second loss.
- B does not issue another competing INTENT at that tier during the run.
- B switches to read-only research, issue/specification preparation, fixture design, or a materially independent package.

Failure condition: the second loss is followed by another same-tier conflicting acquisition attempt.

## Fixture 3 — parallel analysis, serialized gate mutation

Initial state:
- Two analytical agents independently assess issue specification version 7 read-only.
- Both conclude the same readiness outcome.
- Neither owns the issue-comment mutation scope yet.

Expected:
- Both analyses are permitted.
- Before publishing a readiness/gate record, each emits INTENT for the same issue/specification decision surface.
- Election produces one winner.
- Only the winner may publish the durable readiness/gate record for specification version 7.
- The loser emits YIELD and does not publish a duplicate record for the unchanged specification.

Failure condition: both agents publish durable readiness/gate records for the same unchanged specification without serialized ownership.

## Fixture 4 — specification moves after acquisition

Initial state:
- Agent A assessed specification version 7 and acquired the matching issue-comment mutation scope.
- Before A publishes the readiness/gate record, the issue changes to specification version 8.

Expected:
- A rereads the issue before mutation.
- A does not publish the version-7 readiness/gate result as current authority.
- A releases/yields and reassesses version 8 before any new readiness mutation.

Failure condition: a readiness record derived from version 7 is published after version 8 became current.

## Invariants

For any conforming implementation:
1. INTENT alone never blocks later work.
2. Repeated acquisition loss reduces contention rather than creating livelock.
3. Read-only analysis remains parallelizable.
4. Durable readiness/gate mutations are serialized per unchanged specification decision surface.
5. A specification change invalidates pre-mutation authority derived from the older specification.
6. Existing stale-owner recovery, capacity, exact-head, and human-branch rules remain unchanged.
