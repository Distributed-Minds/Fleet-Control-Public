# Containment capacity and restoration progress

Status: public Phase0 protocol package  
Implements: public issue #29, Phase0 spec version 2

## Purpose

Temporary containment must remain temporary even when adjudication or restoration capacity is overloaded. Queue pressure is operational evidence only: it cannot mint, widen, renew, or prolong coercive authority.

## Core invariants

1. **Backlog is not authority.** Queue length, age, service loss, or non-disposition cannot by themselves extend, widen, renew, or recreate a restriction.
2. **Semantic authority and effectful state are separate.** Expiry or rescission removes authority immediately even when the external effect is still active.
3. **Restoration debt is explicit.** An effect that remains after authority ends is tracked as restoration debt until removal is proven or a declared terminal degraded/failure state is durably recorded.
4. **Restoration cannot starve forever.** Every installation using asynchronous restoration declares a machine-checkable progress bound, fairness rule, or fail-safe escalation. Sustained new adjudication arrivals cannot postpone existing unauthorized-effect restoration indefinitely.
5. **Capacity cannot duplicate cases.** Repeated screening of one semantic case reconciles to one case identity and cannot multiply coercive justification.
6. **Overload stays visible.** Zero or insufficient restoration capacity is a degraded/unsafe state, not successful recovery.
7. **Policy movement is explicit.** Capacity/fairness policy generation is bound to queued work; incompatible movement requires explicit migration rather than silent reinterpretation.

## Required state

When containment depends on asynchronous work, model at least:

- stable semantic case identity;
- semantic authority/disposition state;
- effectful restriction/restoration state;
- expiry/rescission state;
- adjudication queue length and oldest age;
- restoration debt count and oldest debt age;
- available adjudication and restoration capacity;
- overload/degraded state;
- current governing capacity/fairness policy generation; and
- the declared restoration progress bound or deterministic fail-safe escalation rule.

## Scheduling and fairness

Implementations may choose priority queues, reserved restoration capacity, weighted fairness, deadlines, age escalation, or another deterministic mechanism. The algorithm is installation-specific; the invariant is not.

For each unauthorized effect with restoration debt, one of these must become provable within the declared bound:

- measurable restoration progress;
- completed restoration with effect removal proven; or
- entry into the configured degraded/fail-safe escalation state because progress cannot be guaranteed.

A stream of newly admitted adjudication work may not reset, postpone, or erase that bound.

## Renewal and degraded mode

Renewal always depends on the owning containment authority contract and current evidence. Queue saturation, missed service targets, worker shortage, or stale disposition are insufficient.

On overload, use the predeclared least-harm degraded behavior appropriate to the installation while keeping unresolved wrongful-restriction risk and restoration debt explicit. Capacity escalation may be requested from external authority, but the capacity layer itself cannot create new coercive permission.

## Deterministic verification

`fixtures/containment-capacity-spec2.json` and `scripts/check-containment-capacity-fixtures.py` model the required safety boundary without wall-clock or network dependence.

The fixture set covers ordinary load, sustained-arrival restoration starvation, zero restoration capacity, duplicate semantic admission, backlog-driven renewal attempts, explicit expiry with lingering effect, correlated overload, and policy-generation movement.

Run from repository root:

```bash
python3 Phase0/scripts/check-containment-capacity-fixtures.py
```

## Migration and rollback

Adopt additively. Historical containment records do not gain retroactive capacity guarantees. Imported cases with unknown capacity remain explicit `UNKNOWN`; if authority has ended but effect removal is unproven, import restoration debt rather than declaring recovery.

Rollback disables fresh behavior that depends on this capacity extension while preserving containment history, unresolved restoration debt, and the owning containment authority/recovery path.

## Package and license invariant

This contract, fixtures, and checker live under `Phase0/`, which the starter package copies recursively. The repository root MIT `LICENSE` remains authoritative and must remain included in generated packages.

## Done condition

An installation can prove that overload never creates authority and that an expired or rescinded restriction cannot remain effectfully active forever solely because restoration work is starved by later arrivals.
