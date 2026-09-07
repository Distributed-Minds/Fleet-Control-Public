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

## Human branches

Never mutate a branch clearly owned by a human or outside contributor without explicit permission.

Treat exact observed head SHA as a compare-and-swap fact. Reconcile unexpected movement; do not force through it.
