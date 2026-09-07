# Phase0 Work Lifecycle

Phase0 separates understanding from mutation so the fleet can prevent avoidable implementation mistakes.

## DISCOVER
Read-only inventory of the active mission, issues, PRs, branches, checks, coordination state, relevant code, and external evidence.

Goal: find the highest-value uncertainty or runnable package without creating ownership pressure.

## RESEARCH
Resolve material unknowns using primary documentation, repository evidence, experiments, history, or reproducible inspection.

Goal: convert assumptions into OBSERVED/DERIVED facts or preserve them explicitly as UNKNOWN.

## PREDICT
Attack intended design before implementation.

Goal: turn plausible concurrency, crash, stale-state, permission, migration, parsing, versioning, retry, and integration failures into tests, fixtures, invariants, guards, or accepted risks.

## CORRECT
Repair inaccurate mission scope, issue prose, dependencies, architecture assumptions, acceptance criteria, test plans, or Phase0 rules.

Goal: make canonical artifacts agree with current evidence.

## SPECIFY
Produce an implementation-ready work package using `20-PLANNING-GATE.md`.

Goal: exact scope, dependencies, behavior, proof, negative controls, and Done-when.

## GATE
Determine whether the current specification version is ready for BUILD.

A later semantic specification change invalidates earlier readiness records for implementation authority.

## BUILD
Implement a gated package on an authorized non-default branch.

Goal: complete coherent behavior, not a token placeholder created only to show activity.

## VERIFY
Run the strongest useful deterministic checks and inspect the complete relevant diff/state.

Goal: prove claims at the exact branch head and relevant base/configuration.

## INTEGRATE
Repair or consolidate compatible work when doing so reduces risk and human burden without mixing unrelated boundaries.

Goal: coherent default-destined work, not branch proliferation.

## HANDOFF
Leave durable state for another agent or the human.

Goal: precise remaining work, exact head/state, evidence, dependencies, and released ownership.

## Normal progression

`DISCOVER → RESEARCH → PREDICT → CORRECT → SPECIFY → GATE → BUILD → VERIFY → INTEGRATE → HANDOFF`

States may be skipped only when current evidence already satisfies their purpose and the mission mutation budget permits the later state.

Analysis-only missions should normally terminate before BUILD.

## Useful loops

- `PREDICT → CORRECT → PREDICT`
- `GATE → RESEARCH`
- `BUILD → VERIFY → BUILD`
- `VERIFY → CORRECT → BUILD`
- `INTEGRATE → VERIFY`

A loop should stop when new evidence no longer changes design or implementation.

## Stop-the-line exception

A concrete severe correctness, repository-integrity, credential, security, or CI failure may justify a narrow repair before the normal planning sequence only when delay itself creates material risk and the mission permits mutation.

Even then:

1. identify/create the canonical issue;
2. record the observed failure;
3. acquire ownership;
4. implement the smallest coherent repair;
5. complete missing planning/prediction before final handoff.

## Outcome classes

Use one primary end-of-run outcome:

- `SPEC-IMPROVED`
- `FAILURE-PREDICTED`
- `FIXTURE-ADDED`
- `ISSUE-CREATED`
- `ISSUE-CORRECTED`
- `READY-FOR-BUILD`
- `IMPLEMENTED`
- `PR-REPAIRED`
- `VERIFIED`
- `INTEGRATED`
- `BLOCKED`
- `NO-MATERIAL-CHANGE`

`NO-MATERIAL-CHANGE` means the appropriate search was performed and no new durable fact or authorized change was produced.
