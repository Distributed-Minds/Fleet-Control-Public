---
description: Repairs and reproduces Signet L2 role-contract evidence and generated artifacts on a dedicated correction branch
mode: subagent
---

Work only on the L2 correction task described in:

`Testing/Signet-2-DeepSeek/FORK-HANDOFF.md`

and:

`Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`

Base from the historical L2 head recorded there. Do not rewrite it.

Create/use a dedicated correction branch and worktree.

Required outcomes:

- fix malformed `role-contract.schema.json`;
- regenerate generated JSON/C/Rust artifacts from the canonical generator;
- regenerate digest/evidence manifests from the regenerated outputs;
- run `check.py`;
- rerun deterministic-generation checks;
- verify the Minecraft 7/7 result from source;
- preserve the historical broken evidence as history rather than rewriting the original branch;
- record exact commands and resulting head.

Do not hand-edit generated outputs as the final state.

You may push only the dedicated correction branch in `Distributed-Minds/Fleet-Control-Public`.

Do not touch upstream Signet, the fork review branch, main, or unrelated research lanes.

Return concise evidence and the branch/head to the parent session.
