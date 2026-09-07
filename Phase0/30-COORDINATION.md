# Phase0 Coordination Protocol

The fleet coordinates through exactly one append-only GitHub issue.

Find it by title and body marker from `05-FLEET-CONFIG.md`; do not hard-code an issue number.

## Bootstrap

Expected title:

```text
[fleet-control] coordination
```

Expected body marker:

```text
FLEET_COORDINATION_V1
```

If it does not exist and the run is authorized to create GitHub artifacts, search carefully for an equivalent before creating one.

## Read-only work

DISCOVER / RESEARCH / PREDICT may proceed without ownership until a run will mutate a potentially conflicting surface.

Acquire ownership before changing issue body/title/state, PR body/title/state, branch/file, formal PR feedback, Phase0 documentation, shared mission state, or another shared GitHub control surface.

Appending this run's own acquisition/state transitions follows this protocol.

## Run identity

Every run creates a unique short token:

```text
run=<agent>-<timestamp>-<random-short>
```

Use the same token for the run.

## Scope

Claim the narrowest meaningful overlapping scope using fields such as:

```text
issue=#N
pr=#N
branch=<ref>
seam=<short-machine-safe-description>
mission=<mission-id>
```

List multiple surfaces only when inseparable for the intended mutation. Repository-wide claims are invalid except a genuine stop-the-line repair requiring repository-wide control.

## Two-phase acquisition

### INTENT

Immediately before acquisition, reread the coordination log and target state. Append:

```text
PHASE0 | seq=1 | run=<token> | agent=<id> | state=INTENT | mission=<id-or-none> | issue=<N-or-none> | pr=<N-or-none> | branch=<ref-or-none> | head=<sha-or-none> | seam=<scope> | ts=<ISO8601> | note=<short>
```

INTENT is not ownership.

### Election

Reread relevant transitions.

- Existing overlapping `OWNED` or `WORKING` wins over a new INTENT.
- Among concurrent overlapping INTENTs without an incumbent, the lower GitHub comment ID wins.
- A loser appends `YIELD` and selects independent work.

### OWNED

A winner appends:

```text
PHASE0 | seq=2 | run=<token> | agent=<id> | state=OWNED | mission=<...> | issue=<...> | pr=<...> | branch=<...> | head=<...> | seam=<...> | prev=<intent-comment-id> | ts=<ISO8601> | note=<short>
```

Reread once more before the first conflicting mutation. If an earlier winner appeared, append YIELD.

## WORKING

Before branch/file mutation, record exact branch and observed head:

```text
PHASE0 | seq=<n> | run=<token> | agent=<id> | state=WORKING | mission=<...> | issue=<...> | pr=<...> | branch=<ref> | head=<sha> | seam=<...> | prev=<comment-id> | ts=<ISO8601> | note=<short>
```

After a remote head change caused by this run, observe and record the new head before later remote mutation.

Unexpected head movement is a stop condition: reread coordination and target state.

## Terminal transitions

### HANDOFF

```text
PHASE0 | seq=<n> | run=<token> | agent=<id> | state=HANDOFF | mission=<...> | issue=<...> | pr=<...> | branch=<...> | head=<...> | prev=<comment-id> | ts=<ISO8601> | outcome=<category> | note=<next-durable-fact>
```

HANDOFF releases ownership.

### RELEASE

```text
PHASE0 | seq=<n> | run=<token> | agent=<id> | state=RELEASE | mission=<...> | issue=<...> | pr=<...> | branch=<...> | head=<...> | prev=<comment-id> | ts=<ISO8601> | outcome=<category> | note=<short>
```

RELEASE releases ownership.

### YIELD

```text
PHASE0 | seq=<n> | run=<token> | agent=<id> | state=YIELD | mission=<...> | issue=<...> | pr=<...> | branch=<...> | head=<...> | prev=<comment-id> | ts=<ISO8601> | note=<collision>
```

YIELD never grants ownership.

## Active state reconstruction

Active ownership states: `OWNED`, `WORKING`.

Non-owning states: `INTENT`, `YIELD`, `HANDOFF`, `RELEASE`, `RECOVERED`.

A stale INTENT does not block future work.

## Crash recovery

An active ownership record older than `STALE_OWNERSHIP_MINUTES` is only eligible for recovery investigation. Age alone is insufficient.

Before recovery, verify:

1. no newer transition for the run;
2. no target activity plausibly showing the owner is still working;
3. recorded branch/PR heads and checks do not show continuing owner progress;
4. recovery will not overlap another current owner.

Then append:

```text
PHASE0 | seq=1 | run=<new-token> | agent=<id> | state=RECOVERED | mission=<...> | issue=<...> | pr=<...> | branch=<...> | head=<...> | seam=<...> | stale_run=<old-token> | ts=<ISO8601> | note=<evidence>
```

RECOVERED is not ownership. Follow INTENT → election → OWNED.

## Capacity

One agent may hold at most one materially independent mutation package at a time.

Read-only research may continue without ownership.

Do not create extra work solely to occupy idle agents.

## Collision backoff

Acquisition loss must reduce repeated contention instead of causing an immediate retry loop.

- After one lost overlap, append `YIELD` and choose the next-best materially independent package when one is available.
- After two consecutive acquisition losses at the same work tier during one run, stop competing for that tier and switch to useful read-only research, issue/specification preparation, fixture design, or another independent package.
- Do not reset the loss count merely by emitting another INTENT for the same overlapping scope.
- Backoff applies to conflicting mutation acquisition only. Independent read-only analysis remains allowed and does not consume ownership.

The purpose is deterministic convergence: agents that repeatedly collide should fan out toward independent useful work rather than livelock on one package.

## Readiness and gate-comment serialization

Analytical agents may independently assess the same issue and unchanged specification read-only.

Before mutating a canonical issue with a readiness/gate record or equivalent shared acceptance decision, acquire a narrow issue-comment mutation scope through this coordination protocol. The election is evaluated against the same unchanged specification identity/version used by the assessment.

After acquisition and immediately before the comment mutation, reread the issue specification and relevant coordination transitions. If the specification changed, or an earlier overlapping winner already mutated that same specification decision surface, do not publish a stale or duplicate record; release/yield and reassess the new state.

This serialization rule does not suppress independent analysis. It serializes only the shared mutation that turns an assessment into durable authority.

Deterministic examples live in `Phase0/fixtures/coordination-convergence.md`.

## Human branches

Never mutate a branch clearly owned by a human or outside contributor without explicit permission.

Treat exact observed head SHA as a compare-and-swap fact. Reconcile unexpected movement; do not force through it.
