# Phase0 Planning Gate

BUILD is downstream of specification quality.

## Canonical spec version

Every issue intended for substantive implementation should contain:

```text
Phase0 spec version: N
```

Start at `1`. Increment it whenever the issue changes in a way that materially alters scope, architecture, dependencies, acceptance criteria, verification, migration, security, or semantics.

Gate records refer to a specific version. A later semantic version invalidates older gate authority.

## Implementation-ready issue sections

Use what is applicable, but do not omit material decisions.

### Problem / observed state

- concrete current behavior or missing capability;
- relevant paths/types/workflows;
- evidence separated into OBSERVED/DERIVED/PREDICTED/UNKNOWN;
- why the current state matters.

### Goal

- exact intended outcome;
- durable behavior after completion.

### Scope

- in scope;
- out of scope;
- likely implementation seams;
- compatibility/migration boundaries.

### Dependencies and ordering

- prerequisite issues/PRs/decisions;
- stop gates;
- independent work;
- work that must not start yet.

### Alternatives considered

Required for material architecture/protocol choices, optional for trivial mechanical changes.

### Predicted failures

Each material failure maps to at least one:

- deterministic test/fixture;
- invariant;
- implementation guard;
- explicit accepted risk with rationale.

### Acceptance criteria

Observable outcomes, not restated implementation intent.

### Verification plan

Use the strongest deterministic evidence expected: unit/property/model tests, integration fixtures, simulation, schema validation, static checks, CI, migration/replay proof, generated-output comparison, or other relevant evidence.

### Rollback / migration

Required when changing durable state formats, canonical policy, coordination semantics, storage, repository bootstrap behavior, or other live authority.

### Done when

A durable terminal condition.

## Readiness records

Append readiness records to the canonical issue after acquiring appropriate mutation scope.

Planning pass:

```text
PHASE0-GATE | kind=PLAN | spec=N | agent=<id> | result=READY|NOT-READY | evidence=<short> | ts=<ISO8601>
```

Independent adversarial/audit pass:

```text
PHASE0-GATE | kind=ADVERSARIAL | spec=N | agent=<id> | result=READY|NOT-READY | evidence=<short> | ts=<ISO8601>
```

## Normal BUILD requirement

For fleets with two or more agents, substantive BUILD normally requires:

1. PLAN READY for the current spec; and
2. ADVERSARIAL READY for the current spec from a different agent/run.

For a single-agent fleet, the same agent may produce both records, but they must be separate passes and the second pass must re-read current repository/spec state.

The builder still revalidates the spec before mutation and may reject stale readiness.

## NOT-READY

A NOT-READY record identifies a concrete missing decision, contradiction, dependency, fixture, acceptance gap, or evidence gap.

Correct the canonical source; do not create chains of comments merely agreeing it is incomplete.

## Mechanical exception

A truly mechanical change with no meaningful semantic choice may record:

```text
PHASE0-GATE | kind=MECHANICAL | spec=N | agent=<id> | result=READY | evidence=<why full gate is unnecessary>
```

Do not use this for coordination, security/permissions, state/schema semantics, migrations, concurrency/retry logic, or changes that alter another agent's behavior.
