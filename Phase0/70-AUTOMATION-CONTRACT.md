# Phase0 Automation Contract

The scheduler should be intentionally small.

Its purpose is to wake a persistent identity, not to duplicate the operating system.

## Required scheduler inputs

Each automation supplies:

- exact target repository;
- persistent `AGENT_ID`;
- instruction to read the complete Phase0 normative set from the repository's default branch in prescribed order;
- instruction to locate the coordination issue and reconstruct this agent's latest valid state;
- instruction to inspect current missions/issues/PRs/branches/checks before acting;
- instruction to obey ownership before mutation;
- instruction not to merge into the default branch under the default policy;
- instruction to append the deterministic next AGENT_STATE before ending.

## Scheduler anti-patterns

Do not put large role personalities, issue assignments, temporary project logic, or work-package details in the scheduler prompt.

Those become stale and split authority across hidden surfaces.

Put durable behavior in Phase0 and durable human objectives in mission issues.

## External executors

The state machine is executor-agnostic.

A BUILD-mode agent may implement directly with authorized GitHub tooling or prepare work for a connected coding executor if the installation deliberately uses one.

In either case:

- the fleet must not claim execution until it observes resulting repository state;
- ownership and exact-head rules still apply;
- external executor activity does not silently override the current mission/specification;
- if an outside actor moves the branch unexpectedly, stop and reconcile.

## End condition

Every scheduled run should finish with:

- released or handed-off package ownership;
- a truthful primary outcome;
- exactly one new AGENT_STATE record for its identity.
