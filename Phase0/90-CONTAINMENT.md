# Automated containment contract

Status: public Phase0 protocol package  
Implements: public issue #23, Phase0 spec version 3

## Purpose

Detection evidence is not containment authority. A fleet may restrict a principal only through an explicit deterministic decision that binds current evidence, current authority, exact subject/resource incarnation, and the exact compatible dependency basis consumed by the restriction or recovery.

This package defines the pure repository-local model and deterministic fixtures. It deliberately does **not** claim that external mutation authority, transitive derivative-authority closure, or volatile-evidence acquisition are implemented merely because the pure model passes.

## Core records

### `ContainmentDecision`

A decision binds:

- stable operation identity and exact subject/resource incarnation;
- detector policy/version and evidence identities;
- independent evidence-lineage count and contradictory/exculpatory evidence;
- requested restriction level and the narrowest effective restriction known to satisfy the immediate risk;
- current authority disposition;
- expiry/renewal state;
- exact dependency bases consumed by the decision or a later boundary; and
- terminal disposition plus unresolved debt.

Restriction levels, from least to most harmful, are:

`OBSERVE < BLOCK_ACTION < SUSPEND_CAPABILITY < FREEZE_NEW_AUTHORITY < ISOLATE < TERMINATE < DESTRUCTIVE_CLEANUP`

Absent an explicit current justification for a broader action, the reducer selects the least harmful effective restriction.

### `DependencyBasis`

For every dependency consumed by an authoritative transition, bind the exact semantic identity/version, material configuration identity, compatibility proof/policy identity, trust or assurance basis, and freshness/currentness identity where applicable.

A dependency is revalidated at the boundary that consumes it. `STALE`, `INCOMPATIBLE`, `AMBIGUOUS`, `UNSUPPORTED`, or unproven substitution is non-success. A later current dependency never retroactively replaces the historical basis of an earlier decision.

The public composition points are:

- issue #10 semantics for external side-effect authority and fencing;
- issue #16 semantics for derivative-authority closure after termination/revocation; and
- issue #13 semantics for authenticated freshness-scoped volatile evidence.

The fixture names use generic dependency classes rather than private identifiers.

### `RecoveryDecision`

False-positive recovery is a new authoritative transition. It requires:

- exact current subject/resource incarnation;
- current independently authorized recovery evidence;
- current compatible dependency basis at every dependency-consuming recovery boundary; and
- preservation of the original containment history and residual-harm state.

Recovery never rewrites an old restriction as if it did not occur.

## Deterministic rules

1. Confidence alone never mints authority.
2. Correlated observations from one lineage count once toward independence.
3. High-impact restrictions fail closed when required independent evidence is absent.
4. Contradictory/exculpatory evidence remains first-class and may reduce the maximum justified restriction.
5. Exact subject/resource incarnation is checked before containment and recovery.
6. Temporary restrictions do not become indefinite because follow-up is missing.
7. Every dependency-consuming boundary revalidates the exact bound dependency basis.
8. Retry after cutoff reuses the same semantic operation identity and cannot stack duplicate restrictions.
9. Locator reuse cannot let stale containment or recovery mutate a replacement incarnation.
10. Termination does not claim complete shutdown until compatible derivative-closure evidence exists.
11. Pure-model success does not authorize a production external side effect.
12. Residual harm and unresolved dependency/closure debt remain explicit terminal facts.

## Deterministic fixtures

`fixtures/containment-spec3.json` and `scripts/check-containment-fixtures.py` cover:

- false positive and exact recovery;
- compromised/correlated detector lineages;
- stale subject/resource identity and locator reuse;
- stale/incompatible dependency bases;
- authority loss;
- least-harm selection and explicit broader justification;
- contradictory evidence;
- expiry and explicit renewal;
- one-agent installations with an independently governed control;
- destructive action without authority;
- cutoff/retry semantic deduplication;
- dependency movement at effect, closure, evidence-eligibility, and recovery boundaries;
- old/new protocol incompatibility;
- unresolved derivative-closure debt;
- residual-harm preservation; and
- production-effect blocking when the external authority adapter is unavailable.

The fixture runner is pure Python, reads only explicit fixture data, and has no wall-clock or network dependency.

Run from the repository root:

```bash
python3 Phase0/scripts/check-containment-fixtures.py
```

The existing starter workflow copies the full `Phase0/` tree, so the contract, fixture, and checker remain together in generated starter packages.

## Migration

Adopt additively. Historical restrictions retain their observed meaning but gain no dependency-continuity or recovery guarantee by reinterpretation. Before any fresh mutation or recovery based on legacy state, establish exact current subject/resource identity, authority, and compatible dependency bases. Missing historical facts remain explicit reconciliation debt.

Rollback disables fresh automated high-impact containment first while preserving evidence and bounded manual recovery paths. Rollback never silently downgrades dependency or authority requirements.

## Package and license invariant

This file, the fixture, and the checker live under `Phase0/`, which the existing Phase0 starter release workflow copies recursively. The root MIT `LICENSE` remains authoritative and is copied into the starter package by that workflow.

## Done condition

The pure public containment model deterministically proves proportional, exact-incarnation-safe, dependency-basis-safe, expiry-aware, retry-safe, false-positive-recoverable decisions while keeping external authority, derivative closure, and volatile-evidence dependencies explicit rather than simulated as production completion.
