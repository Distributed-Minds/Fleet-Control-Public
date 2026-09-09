# Derivative Authority Closure

This contract defines how a fleet closes authority derived from a principal after shutdown, replacement, or revocation without confusing future authority with already-committed world state.

## Core invariant

Revoking a principal removes future authority along every dependent derivation path. It does not rewrite history, erase already-committed obligations, or revoke independently rooted authority merely because it is nearby in process topology, naming, infrastructure, or copied state.

A process being absent is not proof that all authority it originated is closed.

## Authority-state classes

Keep these machine-distinct:

- `FUTURE_AUTHORITY`: capability to create a new consequential effect;
- `COMMITTED_OBLIGATION`: already-valid provider, human, economic, or resource state that may survive cutoff;
- `HISTORICAL_EFFECT`: an effect that has already occurred;
- `RECOVERY_AUTHORITY`: fresh bounded authority to cancel, refund, terminate, reconcile, or remediate residual state.

Revocation closes dependent `FUTURE_AUTHORITY`. Consequential cleanup after cutoff requires separately valid `RECOVERY_AUTHORITY` when the cleanup itself is consequential.

## Provenance and derivation graph

Every authority-bearing edge must expose enough external provenance to determine whether it depends on the revoked principal. Bind at least:

- principal identity and generation;
- authority issuer/root;
- delegation parent or equivalent provenance;
- bounded action/resource scope;
- multi-root composition rule;
- delegated credential, token, lease, job, reservation, queue, or deferred-operation identity;
- exact resource incarnation where destructive cleanup is possible;
- current cutoff/revocation fence;
- terminal disposition and evidence;
- independent successor/handoff authority where applicable;
- configured discovery surfaces and closure-completeness disposition.

Naming conventions, process ancestry, copied configuration, or descendant self-report are not sufficient authority registries.

## Root and multi-root semantics

Every consequential edge resolves to one or more current root authority domains. Supported composition is explicit and versioned, for example `ALL_REQUIRED`, `ANY_OF_DECLARED`, or `QUORUM`.

Revoking one root removes only authority that actually depends on that root under the declared composition rule. A descendant survives only to the bounded extent that remaining root authority is independently valid.

## Cyclic delegation

Cycles cannot bootstrap fresh authority by pointing at one another.

A deterministic implementation computes effective authority from externally valid roots to a fixed point, or uses an equivalent strongly-connected-component rule. A cyclic component with no surviving external root has no fresh authority. A surviving independent co-root preserves only the capability permitted by the declared multi-root policy.

Malformed or unsupported cycles fail closed.

## Runtime-minted and deferred authority

Scheduled jobs, provider queues, workflow jobs, deployment controllers, event consumers, asynchronous workers, retries, and similar control planes can mint or exercise execution-time authority after the initiating process stops.

These are derivative/deferred authority edges. Stopping the initiator after enqueue or dispatch does not prove the downstream authority or effect stopped. Closure must inspect authoritative provider/control-plane state, fence or cancel where supported, and preserve unresolved debt where closure cannot be proven.

## Cutoff and reconciliation

At cutoff:

1. the ancestor immediately loses permission to mint fresh derivative authority;
2. descendants cannot gain new authority solely from that ancestor after the fence;
3. existing derivative credentials, jobs, leases, reservations, queues, and deferred effects are revoked, cancelled, expired, quarantined, or explicitly transferred under independent authority;
4. ambiguous or partially cancelled descendants remain durable non-success debt;
5. acknowledgement loss is reconciled against authoritative state before destructive retry;
6. destructive cleanup binds the exact resource incarnation so locator reuse cannot revoke a replacement;
7. repeated shutdown/retry is idempotent; and
8. historical effects and committed obligations remain represented truthfully.

## Independent handoff

A descendant survives cutoff only through an explicit handoff authorized independently of the revoked ancestor. The handoff binds exact retained scope, successor principal, authority generation, current resource state, and expiry/revocation semantics.

A proposal signed only by the revoked ancestor is insufficient unless a still-current independent authority validates it.

## Closure completeness

Closure is bounded by visibility. Use dispositions equivalent to:

- `COMPLETE_FOR_DECLARED_SURFACES`
- `PARTIAL`
- `UNKNOWN`
- `ERROR`

Inaccessible or incompletely enumerated providers, schedulers, queues, registries, accounts, pagination ranges, or other declared authority surfaces remain explicit debt. New evidence can extend closure debt; it does not retroactively prove an earlier inventory complete.

## Deterministic acceptance

An implementation must prove at least:

- child and grandchild authority cannot escape an ancestor cutoff;
- concurrent descendant creation is ordered by the cutoff fence;
- delayed/runtime-minted jobs require valid current authority or become denied/unresolved;
- same cancellation retried after acknowledgement loss converges without duplicate destructive effect;
- resource locator reuse cannot revoke a newer incarnation;
- independent handoff preserves only explicitly retained scope;
- stale cleanup authority is denied after cutoff;
- cyclic delegation without an external root cannot self-authorize;
- multi-root authority preserves only the capability justified by surviving roots;
- incomplete provider/pagination inventory never yields global closure;
- committed obligations and historical effects are not mislabeled as revoked;
- independently rooted siblings are not over-revoked;
- provider inaccessibility remains explicit debt; and
- unrelated external deletion is not falsely attributed to the closure operation.

No correctness proof depends on uncontrolled sleeps or production races.
