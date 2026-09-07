# Installation checklist

Check each item before asking the fleet to do important work.

- [ ] I have a GitHub account.
- [ ] I have a repository that I own or am authorized to modify.
- [ ] The complete `Phase0/` folder is committed in that repository.
- [ ] `Phase0/05-FLEET-CONFIG.md` matches the number of agents I will run.
- [ ] My GitHub/OpenAI connection is authorized for this exact repository.
- [ ] I checked that the ChatGPT GitHub capability I am using exposes the actions the fleet needs, not only repository reads.
- [ ] I reviewed its ChatGPT app/plugin action permission (for example Always ask, Allow read, Allow low-risk, or Allow all when available) and chose the least-permissive setting that still supports my intended workflow.
- [ ] I understand GitHub organization approval, app installation scope, branch protection, rulesets, provider permissions, and safety controls can still block an action even when ChatGPT permissions allow it.
- [ ] My ChatGPT Project instructions use the exact `OWNER/REPOSITORY` name.
- [ ] Every persistent automation uses the same target repository.
- [ ] Every automation has a unique agent ID.
- [ ] I left `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY` for the first trial.
- [ ] I understand analysis-only requests should not silently become product-code changes.
- [ ] I understand generated code still needs normal tests, permissions, CI, backups, and human judgment.
- [ ] I am starting on a repository where mistakes are recoverable.
