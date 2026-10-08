# Containment capacity and restoration progress

Status: public Phase0 protocol package  
Implements: public issue #29, Phase0 spec version 2

## Purpose

Temporary containment must remain temporary even when adjudication or restoration capacity is overloaded. Queue pressure is operational evidence only: it cannot mint, widen, renew, or prolong coercive authority.

This capacity layer composes with `90-CONTAINMENT.md`. It does not weaken the owning containment evidence, authority, expiry, dependency, or recovery contract.

## Core invariants

1. **Backlog is not authority.** Queue length, age, service loss, or non-disposition cannot by themselves extend, widen, renew, or recreate a restriction.
2. **Semantic authority and effectful state are separate.** Expiry or rescission removes authority immediately even when the external effect is still active.
3. **Restoration debt is explicit.** An effect that remains after authority ends is tracked as restoration debt until removal is proven or a declared terminal degraded/failure state is durably recorded.
4. **Restoration cannot starve forever.** Every installation using asynchronous restoration declares a machine-checkable progress bound, fairness rule, or fail-safe escalation. Sustained new adjudication arrivals cannot postpone existing unauthorized-effect restoration indefinitely.
5. **Capacity cannot duplicate cases.** Repeated screening of one semantic case reconciles to one case identity and cannot multiply coercive justification.
6. **Overload stays visible.** Zero or insufficient restoration capacity is a degraded/unsafe state, not successful recovery.
7. **Policy movement is explicit.** Capacity/fairness policy generation is bound to queued work; incompatible movement requires explicit migration rather than silent reinterpretation.
8. **Workload is event-driven.** Capacity planning uses action/event volume, repeated-case semantics, detector behavior, and arrival correlation rather than treating persistent-worker count as a sufficient workload measure.
9. **Progress is derived, not asserted.** A restoration acceptance test must derive service/progress from explicit arrival, capacity, queue, and scheduler inputs. A precomputed `progress=true` flag is not proof of non-starvation.

## Required state

When containment depends on asynchronous work, model at least:

- stable semantic case identity;
- semantic authority/disposition state;
- effectful restriction/restoration state;
- expiry/rescission state;
- adjudication queue length and oldest age;
- restoration debt count and oldest debt age;
- available adjudication and restoration capacity;
- shared-versus-separate service topology when it affects starvation;
- event/action arrival volume and relevant correlation/burst grouping;
- detector/case-planning inputs such as prevalence/base-rate assumptions, false-positive rate and sensitivity when they are used for capacity planning;
- overload/degraded state;
- current governing capacity/fairness policy generation; and
- the declared restoration progress bound or deterministic fail-safe escalation rule.

Numerical workload assumptions are installation inputs or fixture values. They are not empirical deployment claims unless independently observed for that installation.

## Scheduling and fairness

Implementations may choose priority queues, reserved restoration capacity, weighted fairness, deadlines, age escalation, or another deterministic mechanism. The algorithm is installation-specific; the invariant is not.

For each unauthorized effect with restoration debt, one of these must become provable within the declared bound:

- measurable restoration progress derived from the scheduler and service trace;
- completed restoration with effect removal proven; or
- entry into the configured degraded/fail-safe escalation state because progress cannot be guaranteed.

A stream of newly admitted adjudication work may not reset, postpone, or erase that bound.

If adjudication and restoration share capacity, the scheduler must make the non-starvation rule observable in the actual service trace. If they use separate capacities, loss of restoration capacity remains explicit even when adjudication throughput is healthy.

The bound applies repeatedly while restoration work remains. One early unit of progress cannot justify an indefinitely stalled remainder.

## Renewal and degraded mode

Renewal always depends on the owning containment authority contract and current evidence. Queue saturation, missed service targets, worker shortage, stale disposition, or backlog age are insufficient.

On overload, use the predeclared least-harm degraded behavior appropriate to the installation while keeping unresolved wrongful-restriction risk and restoration debt explicit. Capacity escalation may be requested from external authority, but the capacity layer itself cannot create new coercive permission.

A zero-capacity or starvation condition cannot be reported as recovered. If the configured fail-safe activates, the result is an explicit degraded/escalated state until effect removal is independently proven.

## Duplicate admission and policy movement

Repeated screening of one semantic case must reconcile to the same semantic case identity. Duplicate queue entries do not create independent authority or inflate the apparent number of independently justified restrictions.

Queued work binds the capacity/fairness policy generation that governs its semantics. Incompatible movement requires an explicit compatible migration or a fail-closed disposition. New policy does not silently reinterpret historical queue state.

## Deterministic verification

`fixtures/containment-capacity-spec2.json` and `scripts/check-containment-capacity-fixtures.py` use a pure discrete-step workload model. The checker derives outcomes from explicit arrival traces, service capacity, scheduler behavior, restoration work, progress bounds, authority/effect state, duplicate identity, and policy compatibility. Correctness does not depend on wall-clock sleeps, uncontrolled live races, or precomputed success flags.

The fixture set covers:

- ordinary load;
- backlog-only renewal rejection;
- sustained new arrivals with effectful restoration debt;
- a starvation-prone scheduler that is detected rather than accepted;
- deterministic fail-safe escalation when starvation would cross the declared bound;
- zero restoration capacity;
- service loss followed by recovery within the bound;
- shared and separately reserved restoration capacity;
- correlated burst overload that remains operational state rather than authority;
- duplicate semantic-case admission;
- incompatible policy-generation movement;
- expired authority with lingering effect and missing restoration debt;
- terminal restoration;
- rare-event false-positive-heavy planning inputs;
- sensitivity to prevalence/base-rate, false-positive rate, sensitivity and event volume; and
- equal worker count with materially different action/event volume.

The checker also enforces two cross-case properties: false-positive-heavy fixtures must actually be false-positive-heavy, and equal persistent-worker count cannot hide increased event-driven workload.

Run from repository root:

```bash
python3 Phase0/scripts/check-containment-capacity-fixtures.py
```

## Migration and rollback

Adopt additively. Historical containment records do not gain retroactive capacity guarantees. Imported cases with unknown capacity remain explicit `UNKNOWN`; if authority has ended but effect removal is unproven, import restoration debt rather than declaring recovery.

Legacy capacity records that only assert a success/progress boolean without the arrival/service/fairness basis do not gain non-starvation proof by reinterpretation. They remain historical evidence for what was recorded, while fresh consequential decisions establish the current deterministic basis.

Rollback disables fresh behavior that depends on this capacity extension while preserving containment history, unresolved restoration debt, and the owning containment authority/recovery path.

## Package and license invariant

This contract, fixtures, and checker live under `Phase0/`, which the starter package copies recursively. The repository root MIT `LICENSE` remains authoritative and must remain included in generated packages.

## Done condition

An installation can prove from explicit deterministic workload and service traces that overload never creates authority, that an expired or rescinded restriction cannot remain effectfully active forever solely because restoration work is starved by later arrivals, and that inability to prove progress becomes visible degraded debt rather than a false recovery claim.
