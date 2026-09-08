# Troubleshooting

## My repository does not appear in ChatGPT

Check that:

1. the correct GitHub account is connected;
2. the GitHub app/plugin is installed for the correct personal account or organization;
3. that installation has access to the exact repository;
4. an organization administrator approved it if approval is required; and
5. you allowed time for a newly created or newly authorized repository to appear.

Installing/enabling a GitHub app or plugin and granting it access to a specific repository are separate operations.

## The agent can read GitHub but cannot change it

Do not assume that successful repository reading implies write capability, and do not assume that every ChatGPT GitHub surface is permanently read-only.

Check, in order:

1. **Exact execution context:** determine whether the failure occurred in an interactive conversation, scheduled task, event-triggered task, Work run, or another supported surface. Capability evidence from one does not automatically establish another.
2. **Capability:** confirm that the GitHub app/plugin available in that exact surface exposes the create/update/comment/branch/PR action you need.
3. **App permission/approval:** inspect the connected GitHub capability's permission state. Depending on account/workspace/app, controls may include **Always ask**, **Allow read actions**, **Allow low-risk actions**, or **Allow all actions** for an eligible individual app/account. An action may still require approval and pause a scheduled task.
4. **GitHub authorization:** confirm the provider installation has access to the exact repository and necessary provider permissions.
5. **Organization policy:** an organization may require app approval or restrict repositories/actions.
6. **Repository policy:** branch protection, rulesets, required checks, and other repository controls may reject an otherwise supported action.
7. **Fleet-Control policy:** `HUMAN_MERGE_ONLY`, ownership, planning gates, and other Phase0 rules remain authoritative even when the product can technically perform the action.

Do not widen permissions merely to make a capability check pass.

## Interactive GitHub works but a scheduled agent is read-only or blocked

That is a distinct setup result, not a contradiction.

Scheduled/background capability must be established from the scheduled/background context itself. Use `Phase0/110-GITHUB-SETUP-CAPABILITY.md`.

Prefer a non-destructive capability check. Record the exact execution context, GitHub/app/plugin surface, repository scope, permission/approval state, observation time, and required action class.

If the scheduled action is unavailable, requires unsatisfied approval, is outside repository scope, is blocked by repository policy, or cannot be established from current evidence, treat scheduled mutation capability as absent/blocked/unknown. Read-only scheduled work may continue when separately safe.

## A scheduled task pauses on a GitHub action

Current connected-app permissions and approval requirements continue to apply to scheduled tasks. If an external-data action requires approval, the task can pause until approval is supplied.

Do not record the paused action as successful autonomous mutation. Preserve it as an approval-blocked state and re-evaluate the exact capability/authority basis after approval.

## A task was recreated, migrated, or got a different context ID

Do not start a fresh reversible capability probe merely because the new task can access the same repository.

Probe continuity is tied to a stable installation/probe-namespace lineage, not to an opaque task/chat/context ID. A legitimate successor requires explicit current successor/alias evidence. Unresolved probe generations from the predecessor remain discoverable through that stable lineage and must be reconciled before fresh overlapping mutation.

If lineage evidence is missing, stale, conflicting, or incompatible, use `LINEAGE_UNKNOWN` or an equivalent fail-closed state.

## A context ID was reused

Identifier equality does not prove lineage.

If an old opaque context ID now belongs to a different task or installation, the new context must not adopt the old probe resources. Require current installation-lineage and incarnation/generation evidence. Otherwise fail closed.

## A reversible capability probe was interrupted

Do not blindly create another disposable resource.

Reconcile the stable probe identity against authoritative repository state first:

- no side effect occurred;
- exact owned resource exists;
- exact cleanup already completed with durable evidence;
- locator exists but ownership/incarnation is ambiguous;
- authority changed;
- lineage changed; or
- source inventory is incomplete/unknown.

Fresh mutation requires current lineage, current compatible authority/fencing, and exact resource/pre-state evidence.

If a stale probe leaves a resource requiring destructive cleanup, a successor needs explicit bounded recovery authority. Stale ownership or matching names are not enough.

## A reversible probe would need the default branch

Stop. The beginner acceptance path does not mutate the configured default branch.

Use a non-destructive check or a disposable non-default surface. Do not weaken `HUMAN_MERGE_ONLY`, branch protection, or rulesets merely to make setup pass.

## Several agents edit the same thing

Check the coordination issue. Every conflicting mutation should follow `INTENT → OWNED → WORKING` and should end in `HANDOFF`, `RELEASE`, or `YIELD`.

If agents bypass that protocol, verify every automation actually reads the Phase0 files at the start of each run and points at the same repository.

## Every automation behaves like the same agent

Each automation needs a different persistent ID. For five agents use A1, A2, A3, A4, and A5. Do not reuse A1 five times.

## The coordination issue is missing

An authorized bootstrap run may create it after searching for an equivalent. Expected title/body markers are configured in `Phase0/05-FLEET-CONFIG.md`. Do not hard-code an issue number in scheduler prompts.

## The fleet keeps planning and never builds

Possible causes:

- the human mission is EXPLORE or RESEARCH and does not authorize implementation;
- the implementation issue is not ready under the planning gate;
- no suitable builder is available;
- the exact execution context lacks mutation capability;
- repository permissions or policy block mutation; or
- a dependency is genuinely blocked.

Read the latest `AGENT_STATE` records and the active mission before changing policy.

## The fleet starts coding when I only wanted analysis

Correct the mission to `EXPLORE` or `RESEARCH` and use `READ_ONLY` or `ARTIFACTS` as appropriate. Explicit instructions such as “do not change code” are binding.

## An agent says work succeeded but I cannot find it

Treat the claim as unverified. Ask for the exact repository, branch, PR/issue number, and commit head. If those do not exist, the work was not durably published.

## A scheduled task cannot see my ChatGPT Project files

Do not depend on arbitrary Project-uploaded files being available to scheduled execution. Fleet-Control stores durable operating rules in GitHub and requires every persistent run to re-read them there.

## I changed FLEET_SIZE

Also update the ordered `AGENTS=` list and create/pause automations so real persistent identities match configuration. Do not leave two active automations with the same agent ID.

## I want agents to merge automatically

The starter intentionally uses `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY`. Autonomous merge authority is a governance decision and should not be enabled casually. Product capability to merge is not the same thing as Fleet-Control authority to merge.
