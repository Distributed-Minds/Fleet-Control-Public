# Troubleshooting

## My repository does not appear in ChatGPT

Check that:

1. the correct GitHub account is connected;
2. the GitHub app/plugin is installed for the correct personal account or organization;
3. that app has access to the exact repository;
4. an organization administrator approved it if approval is required;
5. the current ChatGPT surface and workspace allow that app/plugin; and
6. you allowed time for a newly created or newly authorized repository to become visible.

Installing a GitHub connection and granting it access to a specific repository are separate operations.

## The agent can read GitHub but cannot mutate it

Treat read/search capability and mutation capability as different action classes. A connected GitHub account does not prove write access, and an interactive success does not prove that a scheduled/background execution context exposes the same action or approval path.

Check the exact execution context, current app/plugin permissions, repository installation scope, organization approval, repository permissions, branch protections/rulesets, and Fleet-Control policy authority. If the action requires approval, a scheduled run may pause rather than complete autonomously.

For persistent-agent setup, follow `Phase0/95-GITHUB-CAPABILITY-ACCEPTANCE.md`. Prefer a non-destructive scheduled-context check. If capability is absent, blocked, approval-paused, or unknown, keep that context read-only or route the action to explicit human handling rather than widening permissions or guessing.

## A capability check created disposable state and then stopped

Do not delete by locator name alone and do not start a fresh probe blindly. Reconcile the exact durable probe identity, stable installation/probe lineage, current execution-context continuity, resource incarnation, and current authority first.

A stale or superseded probe is reconciliation-only. Destructive cleanup after supersession requires an explicit bounded recovery-authority transfer for the exact orphan resource. Unknown lineage, ambiguous ownership, stale authority, or lost cleanup permission is recovery debt, not deletion permission.

## A scheduled context changed identity after recreation or migration

An opaque task/context identifier does not define durable probe lineage. Use current machine-checkable predecessor/successor or alias evidence to attach the replacement context to the same stable installation/probe namespace. Reconcile unresolved older generations before fresh mutation.

If continuity is stale, conflicting, unavailable, or based only on matching names/IDs/repository access, classify it as `LINEAGE_UNKNOWN` and fail closed for reversible mutation until continuity or safe separation is established.

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
- builder agents are still in analytical bootstrap;
- no agent has reached BUILD;
- repository permissions prevent mutation;
- the scheduled execution context has not established the required GitHub action class;
- current lineage or mutation authority is unavailable;
- a dependency is genuinely blocked.

Read the latest `AGENT_STATE` records and the active mission before changing policy.

## The fleet starts coding when I only wanted analysis

Correct the mission to `EXPLORE` or `RESEARCH` and use `READ_ONLY` or `ARTIFACTS` as appropriate. Explicit instructions such as “do not change code” are binding.

## An agent says work succeeded but I cannot find it

Treat the claim as unverified. Ask for the exact repository, branch, PR/issue number, and commit head. If those do not exist, the work was not durably published.

## A scheduled task cannot see my ChatGPT Project files

That can be expected. Scheduled ChatGPT tasks may not have access to files uploaded directly to a Project. Fleet-Control therefore stores durable operating rules in GitHub and requires every persistent run to re-read them from there.

## I changed FLEET_SIZE

Also update the ordered `AGENTS=` list and create/pause automations so real persistent identities match configuration. Do not leave two active automations with the same agent ID.

## I want agents to merge automatically

The starter intentionally uses `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY`. Technical ability to call a merge action is not authority to merge. Autonomous integration authority is a governance decision and should not be enabled casually.
