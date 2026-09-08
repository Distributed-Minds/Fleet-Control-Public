# Installation checklist

Check each item before asking the fleet to do important mutation-capable work.

- [ ] I have a GitHub account.
- [ ] I have a repository that I own or am authorized to modify.
- [ ] The complete `Phase0/` folder is committed in that repository.
- [ ] `Phase0/05-FLEET-CONFIG.md` matches the number of agents I will run.
- [ ] My GitHub/OpenAI connection is authorized for this exact repository.
- [ ] I identified the exact ChatGPT execution context my persistent agents will use rather than assuming interactive and scheduled capabilities are identical.
- [ ] I checked that the GitHub capability in that exact scheduled/background context exposes the actions the fleet needs.
- [ ] I recorded the relevant app/plugin permission and approval state and chose the least-permissive setting that supports my intended workflow.
- [ ] I understand GitHub organization approval, installation scope, provider permissions, branch protection, rulesets, required checks, workspace policy, and safety controls can still block an action.
- [ ] I used `Phase0/110-GITHUB-SETUP-CAPABILITY.md` to establish scheduled mutation capability or accepted a read-only/blocked/unknown result.
- [ ] If a reversible capability probe was necessary, it used a disposable non-default surface with stable probe identity, stable installation/probe lineage, exact resource incarnation, current compatible mutation authority/fencing, deterministic retry reconciliation, and exact cleanup/recovery evidence.
- [ ] I did not widen permissions or weaken repository protections merely to make capability acceptance pass.
- [ ] My ChatGPT Project instructions use the exact `OWNER/REPOSITORY` name.
- [ ] Every persistent automation uses the same intended target repository.
- [ ] Every automation has a unique agent ID.
- [ ] I left `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY` for the first trial.
- [ ] I understand analysis-only requests should not silently become product-code changes.
- [ ] I understand generated code still needs normal tests, permissions, CI, backups, and human judgment.
- [ ] I am starting on a repository where mistakes are recoverable.
