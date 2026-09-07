# Troubleshooting

## My repository does not appear in ChatGPT

Check that:

1. the correct GitHub account is connected;
2. the GitHub app/plugin is installed for the correct personal account or organization;
3. that app has access to the exact repository;
4. an organization administrator approved it if approval is required;
5. you waited a few minutes after creating or authorizing a new repository.

Installing/enabling a GitHub app or plugin and granting it access to a specific repository are separate operations.

## The agent can read GitHub but cannot push changes

Do not assume that successful repository reading implies write capability, and do not assume that every ChatGPT GitHub surface is permanently read-only either.

Check, in order:

1. **Capability:** confirm the GitHub app/plugin available in the ChatGPT surface you are using actually exposes the create/update/comment/branch/PR action you need. OpenAI currently has more than one GitHub-related surface, and their supported actions can differ.
2. **ChatGPT app/plugin permission:** review the connected GitHub capability's permission/approval setting. Depending on account and app, current controls can include **Always ask**, **Allow read actions**, **Allow low-risk actions**, and **Allow all actions**. Permission settings only authorize actions the app actually supports.
3. **GitHub authorization:** confirm the connected GitHub account/app installation has access to the exact repository and the required provider permissions.
4. **Organization policy:** an organization can require app approval or restrict which repositories/actions are available.
5. **Repository policy:** branch protection, rulesets, required checks, and other repository settings can reject an otherwise supported write or merge.
6. **Safety/approval:** ChatGPT may still ask for confirmation or deny an action depending on its risk and the active product/workspace policy.

Fleet-Control's beginner setup is ChatGPT-only: configuring a separate GitHub connector in Codex cloud settings can be part of repository-access plumbing when your account exposes that flow, but it does not by itself mean a Fleet-Control run launches a Codex execution session. If your current ChatGPT GitHub capability exposes only reads, use a supported action-enabled connection/surface before expecting the fleet to mutate the repository.

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

The starter intentionally uses `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY`. Autonomous merge authority is a governance decision and should not be enabled casually.
