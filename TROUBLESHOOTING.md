# Troubleshooting

## My fork shows the FREE ENERGY landing README but no `Phase0/`

The upstream `main` branch currently has a project landing page, **not** the installable Phase0 preview. A fork copied from only the default branch can therefore look correct at first but lack the operating files. **Do not enable fleet automations** until `Phase0/05-FLEET-CONFIG.md` is visible on **your repository's default branch**.

1. In **your fork**, use the branch selector to check whether `phase0/public-v0` exists.
2. **If that branch exists:** with administrator access to **your fork**, open **Settings → Default branch** and select `phase0/public-v0`. Return to the fork's default view and verify `Phase0/05-FLEET-CONFIG.md`. Do **not** change the upstream repository's `main` branch.
3. **If that branch is missing:** a default-only fork did not copy the preview. Either make a new fork with **Copy the DEFAULT branch only** unchecked and then follow step 2, or install the [Phase0 starter ZIP](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/tag/v0.1.2-phase0-preview) into a repository you control and commit its `Phase0/` directory on that repository's default branch. **For the historical v0.1.2 ZIP, add the [public Phase0 MIT notice](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/LICENSE) as `Phase0/LICENSE` before committing; do not overwrite your project's root `LICENSE` or assume unrelated files are MIT-licensed.** A fork that copies all branches also contains historical/research branches; they are not the installed fleet.

   **If using the historical ZIP:** Download both the [v0.1.2 starter ZIP](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip) and its [published SHA-256 sidecar](https://github.com/Distributed-Minds/Fleet-Control-Public/releases/download/v0.1.2-phase0-preview/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip.sha256). **Before extracting or copying**, run `sha256sum FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip` on Linux (or `Get-FileHash -Algorithm SHA256 .\FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip` in PowerShell) and compare its hexadecimal digest with the one in the sidecar. **Stop if they differ.** A match establishes only consistency with that same published release; it is **not** an independent authenticity signature, a safety or rights audit, or proof the archived setup text is current. This check is for the release ZIP, not the fork/preview-source route.

4. Follow the [current Getting Started instructions](GETTING-STARTED.md#2-install-the-actual-phase0-preview-in-a-repository-you-control) for both routes. The published v0.1.2 ZIP contains **older setup instructions** and is not a snapshot of the latest preview branch. Inspect the files and configure your own target repository and distinct agent IDs before scheduling runs.

If the verified default branch still has no `Phase0/05-FLEET-CONFIG.md`, **stop**: changing ChatGPT connections or creating more scheduled tasks will not repair the missing installation.

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
