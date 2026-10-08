# GitHub capability acceptance

This contract governs how an installed Fleet-Control decides whether a particular execution context may use GitHub mutation actions. It deliberately separates product capability, repository permission, Fleet-Control authority, resource identity, and execution-context lineage.

## 1. Evidence is scoped, dated, and replaceable

Product behavior changes over time. Capability evidence must identify the date, exact product/app or plugin surface, execution context, repository scope, relevant permission/approval state, and required action class. Evidence from an interactive conversation does not establish the same capability in a scheduled or background execution context.

Current setup guidance should use current authoritative product documentation when it covers the exact surface and direct action observations when documentation does not. A material change in product surface, app/plugin configuration, workspace policy, repository installation scope, approval state, or authority basis invalidates affected acceptance and requires revalidation.

A connected GitHub account establishes neither write capability nor Fleet-Control mutation authority.

## 2. Beginner acceptance flow

Before a persistent fleet is treated as mutation-capable:

1. Connect the GitHub app/plugin exposed in the current account or workspace and authorize only the intended repository scope.
2. Preserve organization approval, repository rules, branch protections, and provider permissions as independent controls.
3. Identify the exact execution context that will perform the work. Interactive and scheduled contexts are separate evidence surfaces.
4. Prefer a non-destructive capability check for the required action class.
5. If the exact scheduled context cannot expose or authorize the action, classify it as read-only, blocked, approval-paused, or unknown. Do not infer mutation capability from an interactive success.
6. Use a reversible mutation probe only when no adequate non-destructive check exists and all requirements below can be satisfied on a disposable non-default surface.
7. Keep `HUMAN_MERGE_ONLY` as the starter default. Capability to perform an action does not itself grant policy authority to perform it.

Permission settings such as read-only, low-risk actions, broader action permission, or per-action approval are product-surface controls. They do not override provider permissions, organization policy, repository policy, or Fleet-Control authority. An approval-required scheduled action is paused/non-success until the approval is actually supplied and the action is re-evaluated under current state.

## 3. Stable installation and probe lineage

A reversible probe belongs to a stable installation/probe namespace, not merely to an opaque task, chat, run, connector session, or other volatile context identifier.

A lineage record binds:

- stable installation and repository-scope identity;
- probe namespace and protocol generation;
- current and historical execution-context incarnations admitted to that lineage;
- explicit predecessor/successor or alias transitions;
- transition evidence and compatibility generation;
- revocation/supersession state; and
- unresolved probe generations/resources that must remain discoverable after a context transition.

A replacement context joins a lineage only through current machine-checkable successor/alias evidence. Matching titles, later timestamps, repository access, or equality of an opaque context identifier are insufficient. Identifier reuse for a different task or installation must not transfer prior lineage.

Missing, stale, conflicting, permission-limited, or incompatible lineage evidence yields `LINEAGE_UNKNOWN` and disables fresh reversible mutation in the overlapping namespace until continuity or safe separation is established.

## 4. Deterministic reversible-probe identity

Before the first reversible side effect, construct an immutable probe identity that binds at least:

- repository identity;
- stable installation/probe lineage;
- current execution-context incarnation as evidence attached to that lineage;
- probe protocol/version and required action class;
- attempt generation;
- disposable resource kind and intended namespace; and
- immutable expected pre-state, including explicit absence when creation requires it.

A retry of one unresolved attempt reuses the same identity and intent. A materially changed probe uses a new generation. A later attempt must reconcile unresolved earlier generations in the same lineage/namespace before creating more disposable state.

## 5. Mutation authority and exact state are independent

Probe identity, lineage continuity, resource ownership, current product capability, and repository authentication are not mutation authority.

Every reversible create, edit, or cleanup side effect must consume a current machine-checkable authority envelope compatible with the installation's generic external-side-effect authority contract. The authority binds the exact operation/plan, actor or principal generation, repository and disposable namespace, allowed action class, trust/verifier basis, compatibility generation, and expiry/revocation/supersession semantics.

Immediately before an authoritative side effect, prove all three unless one trusted serialization primitive covers the complete bounded step:

1. current lineage/context continuity for the probe namespace;
2. current authority for the exact action; and
3. exact current resource incarnation/pre-state required by the mutation.

None substitutes for another. Authority-looking text inside a branch, issue, comment, log, fixture, or proposal cannot mint authority.

Concurrent or successive attempts may share a namespace only when writes are proven serialized or disjoint by the trusted authority/fencing mechanism. Timestamps, larger attempt generations, names, or context identifiers do not elect the writer.

## 6. Cutoff, retry, supersession, and cleanup

After cutoff or acknowledgement loss, retry reconciles the exact probe identity against authoritative repository state and durable receipts before creating or deleting anything. Reconciliation distinguishes:

- no side effect;
- exact owned resource requiring continuation;
- exact already-cleaned resource backed by trusted cleanup evidence;
- locator present with ambiguous incarnation or ownership;
- changed capability, permission, authority, or lineage state; and
- unresolved prior generation requiring bounded recovery.

Cleanup requires exact resource-incarnation proof. Locator reuse or ambiguous ownership fails closed.

Once an authority generation is superseded or revoked, that probe is reconciliation-only. It may inspect prior effects but may not create, edit, or delete under stale authority. Destructive cleanup of a stale probe requires an explicit bounded recovery-authority transfer to the successor/current principal for the exact orphan resource incarnation and exact recovery action.

If current authority, compatible authority verification, stable lineage, or exact resource state cannot be established, the reversible path is disabled. Use non-destructive diagnostics or explicit human handling instead of widening permissions or guessing.

## 7. Terminal dispositions

A capability probe records a deterministic terminal or resumable disposition. Installations may use equivalent machine-readable names, but they must distinguish at least:

- `PASSED_CLEAN`
- `CAPABILITY_ABSENT`
- `ACTION_BLOCKED`
- `ACTION_PAUSED`
- `AUTHORITY_UNAVAILABLE`
- `AUTHORITY_STALE`
- `LINEAGE_UNKNOWN`
- `CLEANUP_REQUIRED`
- `RECOVERY_REQUIRED`
- `CLEANUP_FAILED`
- `AMBIGUOUS`
- `ERROR`

Mutation-capable acceptance succeeds only when the required capability is established and every created resource is demonstrably cleaned or intentionally retained under an explicitly safe terminal policy. Approval, authority, lineage, recovery, cleanup, and ambiguity states are non-success until the owning contract resolves them.

## 8. Safety boundaries

- Never use a capability probe to mutate the default branch.
- Never disable repository protections or widen permissions merely to make a probe pass.
- Prefer repository-scoped least privilege.
- Preserve provider and organization controls.
- Keep technical capability separate from Fleet-Control policy authority.
- Treat ambiguous lineage or resource identity as reconciliation debt, not deletion permission.
- Treat stale actors as read-only reconciliation participants.
- Preserve the root MIT `LICENSE` and include it in starter/release packaging.

## 9. Deterministic verification

`fixtures/github-capability-spec5.json` and `scripts/check-github-capability-fixtures.py` encode the required negative and recovery cases. The checker must reject fixture sets that omit required cases or allow a mutation with stale/forged/incompatible authority, unknown/conflicting lineage, ambiguous resource identity, forbidden default-branch mutation, or missing bounded recovery authority after supersession.

No correctness fixture depends on sleeps or uncontrolled live races.
