# GitHub setup capability, scheduled-context acceptance, and probe lineage

Status: public Phase0 setup protocol package  
Implements: public issue #11, Phase0 spec version 5

## Purpose

A GitHub connection is evidence of a configured integration surface. It is not, by itself, proof that an interactive conversation or a scheduled/background run can perform a particular repository mutation, and it is never repository mutation authority.

Fleet-Control setup must keep these facts separate:

- product/app/plugin capability;
- exact execution context;
- GitHub installation and repository scope;
- workspace/account permission and approval state;
- repository and organization policy;
- stable installation/probe-namespace lineage;
- exact disposable-resource incarnation;
- current mutation authority/fencing; and
- recovery authority after supersession.

A mutation-capable scheduled setup succeeds only when every required dimension is current and compatible for the exact action class.

## Current product evidence discipline

Product behavior is volatile. Setup guidance must use current, dated evidence for the exact product surface and must not turn one observed menu or capability into a universal rule.

As implementation evidence on 2026-09-09, current OpenAI Help Center material states that:

- GitHub connections are authorized to selected repositories and remain bounded by provider/app/workspace controls;
- scheduled tasks can use supported connected apps, including GitHub, when available for the account or workspace;
- connected-app permissions and workspace controls continue to apply in scheduled execution;
- an external-data action that requires approval can pause a scheduled task;
- app permission options can vary and may include Always ask, Allow read actions, Allow low-risk actions, and Allow all actions for eligible individual apps/accounts; and
- Plugins are the primary workflow-discovery surface while apps remain the underlying external-service connections.

Current evidence sources:
- https://help.openai.com/en/articles/11145903
- https://help.openai.com/en/articles/10291617
- https://help.openai.com/en/articles/11487775
- https://help.openai.com/en/articles/20001495
- https://help.openai.com/en/articles/20001256

These observations are not permanent protocol constants. Re-check them when implementation-time product behavior matters. Public issue #13 owns generic freshness, authenticity, surface compatibility, and assurance semantics for volatile external evidence.

## Capability basis

A scheduled mutation-capability decision binds an immutable basis equivalent to `SetupCapabilityBasis` containing at least:

- stable installation/repository-scope identity;
- stable probe-namespace lineage identity and generation;
- current execution-context instance identity as evidence attached to that lineage;
- product/app/plugin/connector surface identity;
- exact GitHub repository identity and authorized installation scope;
- required action class;
- relevant account/workspace/app permission and approval disposition;
- relevant organization/repository policy basis;
- observation time and evidence identities;
- probe method/version when a probe was used;
- authority-adapter compatibility identity; and
- reducer/schema version.

Interactive capability is evidence only for the interactive basis that produced it. It cannot establish scheduled/background capability merely because the same GitHub account, repository, chat title, task title, or connector is involved.

## Scheduled-context acceptance

Before persistent scheduled agents are described as mutation-capable, obtain evidence from the scheduled/background execution context itself.

Prefer a non-destructive capability check that establishes the required action class without changing repository state.

If no adequate non-destructive check exists, a reversible probe may be used only when all of these are true:

1. the target is a disposable non-default surface;
2. `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY` remains unchanged unless the human separately changes it;
3. a stable probe identity exists before the first side effect;
4. current lineage is established;
5. current compatible mutation authority/fencing exists for the exact create/verify/cleanup actions;
6. the exact expected pre-state and resource incarnation are bound;
7. retry and acknowledgement-loss reconciliation are defined; and
8. cleanup has an exact terminal evidence path.

A probe must not widen permissions merely to make acceptance pass.

Use dispositions equivalent to:

`PASSED_CLEAN | CAPABILITY_ABSENT | ACTION_BLOCKED | AUTHORITY_UNAVAILABLE | AUTHORITY_STALE | LINEAGE_UNKNOWN | CLEANUP_REQUIRED | RECOVERY_REQUIRED | CLEANUP_FAILED | AMBIGUOUS | ERROR`

Only a successful capability result with no unresolved created resource qualifies as mutation-capable acceptance.

An approval-required scheduled action is not autonomous success. It is `ACTION_BLOCKED` or an equivalent paused state until the required approval is independently satisfied.

## Stable probe identity

Every reversible attempt receives a durable identity before mutation. It binds at least:

- repository identity;
- stable installation/probe-namespace lineage;
- current execution-context instance as lineage evidence;
- probe protocol/version;
- required action class;
- attempt generation;
- exact disposable resource kind and namespace; and
- immutable expected pre-state, including expected absence when creation requires it.

Retries of unchanged intent reuse the same identity. Changed immutable intent uses a new generation. A branch name, issue number, comment locator, task ID, chat ID, or provider-generated context ID is not sufficient proof of probe ownership or lineage.

## Stable installation and context lineage

Probe namespace continuity must survive legitimate task/context recreation without letting opaque identifier reuse transfer ownership.

A lineage record binds at least:

- stable installation/repository scope identity;
- probe namespace/protocol generation;
- current and historical context-instance identities;
- explicit predecessor/successor or alias transitions;
- evidence source and compatibility generation for each transition;
- revocation/supersession state; and
- unresolved probe generations/resources that remain reachable.

A replacement context becomes a successor only through current machine-checkable evidence under the same installation lineage. Matching title, later timestamp, equal opaque context ID, or access to the same repository is insufficient.

If context continuity is stale, conflicting, permission-limited, unsupported, or unknown, return `LINEAGE_UNKNOWN` and perform no fresh reversible mutation.

A context successor inherits discoverability of unresolved probes, not their mutation authority.

## Probe mutation authority and fencing

Probe identity, resource ownership, and lineage are not authority.

Every reversible create, edit, or cleanup effect must consume a current machine-checkable authority envelope compatible with public issue #10, either immediately at the side-effect boundary or under one trusted serialization/fence covering the complete bounded step.

The boundary proves all three independently:

1. current lineage/current execution-context continuity;
2. current authority for the exact action; and
3. exact resource incarnation/pre-state required by the action.

No one dimension upgrades another.

After authority supersession, a stale probe may perform bounded read-only reconciliation when permitted. It cannot create, edit, or delete under stale authority.

If stale work leaves a disposable resource, destructive cleanup after supersession requires explicit bounded recovery authority for the exact old operation/generation, resource incarnation, permitted cleanup action, successor principal/generation, current trust basis, and expiry/supersession semantics.

If no compatible authority adapter exists, the reversible fallback is disabled. Use non-destructive diagnostics or explicit human acceptance instead.

## Cutoff, acknowledgement loss, and cleanup

After cutoff or lost acknowledgement, reconcile before any new side effect.

Reconciliation distinguishes at least:

- no side effect occurred;
- exact owned resource exists and requires continuation;
- exact resource was already cleaned with durable terminal evidence;
- locator exists but incarnation/ownership is unprovable;
- capability/permission/authority changed;
- lineage changed or is unresolved; and
- inventory/completeness is unknown.

Current absence alone does not prove successful cleanup when provenance matters.

A later attempt must reconcile unresolved earlier generations in the same stable namespace before creating another disposable resource. Repeated cutoffs cannot accumulate an unbounded trail of abandoned probes.

## Repository-policy boundary

Product capability never overrides GitHub or Fleet-Control policy.

- connection does not bypass organization approval, provider permissions, branch protection, rulesets, required checks, or repository policy;
- a reversible setup probe never mutates the configured default branch;
- the beginner starter keeps `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY`;
- technical merge capability does not grant fleet merge authority;
- setup must not tell a beginner to weaken repository protections merely to obtain a passing result; and
- missing or incompatible evidence fails toward read-only diagnosis, not guessed mutation capability.

## Deterministic verification

`fixtures/github-setup-spec5.json` and `scripts/check-github-setup-fixtures.py` implement deterministic capability, probe-recovery, and lineage cases without network or wall-clock dependency.

The corpus covers the issue #11 spec-5 failure matrix, including:

- interactive write with scheduled read-only or missing capability;
- scheduled approval requirements and repository-scope mismatch;
- stale capability evidence and repository-policy rejection;
- default-branch probe rejection;
- cutoff before verification or cleanup;
- duplicate retry and repeated cutoff;
- cleanup permission loss and acknowledgement loss;
- locator reuse and exact-incarnation mismatch;
- concurrent current/stale generations;
- superseded-probe reconciliation and recovery authority;
- forged/self-asserted authority;
- incompatible authority adapters;
- task recreation with current successor evidence;
- opaque context-ID reuse;
- product migration without lineage evidence;
- orphan cleanup after successor admission; and
- lineage-map generation skew.

Run from repository root:

```bash
python3 Phase0/scripts/check-github-setup-fixtures.py
```

## Migration and rollback

Adopt additively. Historical capability observations and older probe-like artifacts retain only the meaning they originally proved.

Do not infer stable installation lineage from an old opaque task/context identifier. Do not infer current authority from a still-existing resource or a previously successful mutation. Unresolved older probes become reconciliation debt until current lineage, resource identity, and authority are established.

Rollback disables fresh reversible probing first while preserving evidence needed to reconcile or safely clean existing disposable resources. Read-only diagnostics remain available when separately safe.

## Package and license invariant

This setup contract, fixtures, and checker live under `Phase0/`, which the starter packaging process copies recursively. The repository root MIT `LICENSE` remains authoritative and must remain in generated/downloadable starter packages.

## Done condition

A beginner installation can connect GitHub without being promised universal write access, can independently establish or reject scheduled mutation capability for the exact execution context, and can use a reversible fallback only through stable lineage, exact resource identity, current machine-checkable authority, deterministic reconciliation, and bounded cleanup/recovery semantics.
