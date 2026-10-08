# Installation checklist

Check each item before asking the fleet to do important work.

- [ ] I have a GitHub account.
- [ ] I have a repository that I own or am authorized to modify.
- [ ] The complete `Phase0/` folder is committed in that repository.
- [ ] My repository's **default branch** contains `Phase0/05-FLEET-CONFIG.md` (a default-branch-only fork of the current upstream `main` does not).
- [ ] The copied `Phase0/` folder retains its MIT copyright and permission notice in `Phase0/LICENSE`. For the historical v0.1.2 ZIP, add this file from the [public Phase0 MIT notice](https://github.com/Distributed-Minds/Fleet-Control-Public/blob/phase0/public-v0/LICENSE) before committing; do not overwrite my project's existing root `LICENSE` or assume Phase0's MIT license also covers unrelated project files.
- [ ] `Phase0/05-FLEET-CONFIG.md` matches the number of agents I will run.
- [ ] My GitHub connection is authorized for this exact repository.
- [ ] My ChatGPT Project instructions use the exact `OWNER/REPOSITORY` name.
- [ ] Every persistent automation uses the same target repository.
- [ ] Every automation has a unique agent ID.
- [ ] I established the required GitHub action classes from the scheduled/background execution context itself, following `Phase0/95-GITHUB-CAPABILITY-ACCEPTANCE.md`.
- [ ] I did not treat interactive capability, connection state, matching context IDs, or provider authentication as proof of scheduled mutation authority.
- [ ] Any reversible capability probe is disposable and non-default, has stable installation/probe lineage, exact resource-incarnation evidence, current bounded authority, cutoff-safe reconciliation, and bounded recovery semantics.
- [ ] Approval-paused, blocked, read-only, stale-authority, unknown-lineage, and ambiguous-resource outcomes fail closed rather than widening permissions.
- [ ] I left `DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY` for the first trial.
- [ ] I understand analysis-only requests should not silently become product-code changes.
- [ ] I understand generated code still needs normal tests, permissions, CI, backups, and human judgment.
- [ ] I am starting on a repository where mistakes are recoverable.
