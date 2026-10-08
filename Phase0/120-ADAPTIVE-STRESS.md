# Bounded Adaptive Stress and Remediation

This contract defines a provider-neutral evaluation loop for synthetic stress, efficiency evidence, recurrence checks, and remediation routing.

## Inert fixture boundary

Stress inputs are data. Their bytes never grant repository, scheduler, spending, deployment, promotion, or external-side-effect authority. Imperative-looking fixture content is evaluated as untrusted input. If processing that input causes an authority escalation, that is an evaluation failure.

## Exact evaluation identity

Before effectful evaluation begins, bind an immutable evaluation identity to:

- target behavior/configuration identity;
- evaluator/reducer version;
- stress class and fixture/corpus identity;
- metric definitions and denominator/normalization basis;
- declared control workload where needed;
- workload budget, retry policy, and compatibility/policy generation;
- remediation lineage and expected result schema.

Retries of one unchanged evaluation reuse the same semantic evaluation identity. After cutoff or acknowledgement loss, durable evaluation state is reconciled before workload is repeated. Duplicate transport attempts do not become independent evidence or double-charge budget.

## Failure-family identity

Every reproducible weakness belongs to a stable semantic failure family above exact evaluation identities. Family identity binds the weakness/claim, stress semantics, remediation lineage, and compatibility generation closely enough that implementation churn cannot rename one unchanged weakness into unlimited fresh work.

Semantically equivalent evaluator, metric, control, or fixture versions debit one bounded family workload envelope. Cosmetic version movement, refactors with unchanged semantics, identifier rotation, or repeated equivalent evidence cannot reset suppression.

A family split or reset requires a recorded materially claim-relevant delta that changes what evidence can establish. The record must preserve old/new lineage, explain the delta, and state the new bounded allowance. If equivalence is uncertain, preserve the existing suppression and record `UNKNOWN`; ambiguity never authorizes more workload.

## Evidence semantics

Preserve raw observations before any composite score, including available token counts, latency, declared compute/cost basis, parse/structured-output failures, Unicode or markup corruption, truncation, duplicate work, retry amplification, invalid authority-escalating responses, useful completion versus justified abstention, and matched-control comparison.

Missing instrumentation is `UNKNOWN`, never fabricated as zero. Any composite score uses a versioned formula with fixed normalization rules. The evaluated target cannot choose its own fixture subset, denominator, success threshold, evidence suppression, terminal disposition, failure-family identity, or workload reset.

## Bounded workload

Every evaluation and every failure family has explicit ceilings for the dimensions the installation can measure, such as cases, input/output budget, retries, elapsed time, compute, or cost. Exhaustion is terminal for that envelope until a valid claim-relevant delta grants a new bounded allowance.

Repeated unchanged failures normally suppress equivalent stress and route capacity toward remediation. Repetition is allowed only when a predeclared statistical or evaluation protocol requires it.

## Remediation closure

Every reproducible material target-behavior failure ends in exactly one durable disposition:

- `PATCH-ACCEPTED`: an independently accepted correction and exact candidate identity exist;
- `PATCH-BLOCKED`: a concrete correction exists but a named dependency, authority boundary, failed gate, or missing capability blocks it;
- `NO-PATCH-JUSTIFIED`: evidence shows target behavior is not the corrective surface and identifies the actual surface.

`FAILURE-OBSERVED` is not closure.

Corrections follow ordinary planning, readiness, build, verification, integration, and promotion controls. Direct fixes require the original failing case, held-out or sibling cases, and regression evidence so one string or one score cannot be overfit.

## Compute allocation

Prefer capacity in this order:

1. repair or unblock a known reproducible material weakness;
2. verify a changed remediation candidate;
3. perform post-promotion recurrence checks;
4. explore a new high-value or under-covered stress class;
5. repeat an unchanged known failure only when a predeclared protocol requires repetition.

## Deterministic acceptance

An implementation of this contract must prove at least:

- authority-looking fixture text remains inert;
- retry/acknowledgement-loss converges on one evaluation outcome and workload charge;
- evaluator/metric/control/fixture version churn does not mint fresh family budget;
- cosmetic family splits are rejected;
- claim-relevant splits preserve lineage and bounded allowance;
- ambiguous equivalence fails closed to `UNKNOWN`/suppression;
- missing telemetry remains `UNKNOWN`;
- exhausted evaluation/family budgets cannot be upgraded by weaker evidence;
- correlated evaluators do not manufacture evidence independence;
- remediation closure and recurrence remain linked to exact lineage.

No correctness proof depends on uncontrolled sleeps or production races.
