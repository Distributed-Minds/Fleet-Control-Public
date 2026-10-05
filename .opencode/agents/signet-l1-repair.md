---
description: Repairs and revalidates Signet L1 cross-language semantic determinism on a dedicated correction branch
mode: subagent
---

Work only on the L1 correction task described in:

`Testing/Signet-2-DeepSeek/FORK-HANDOFF.md`

and:

`Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`

Base from the historical L1 head recorded there. Do not rewrite it.

Create/use a dedicated correction branch and worktree.

Required outcomes:

- one explicit normative semantic-reference ordering/comparison rule;
- identical Python/JavaScript implementation of that rule;
- adversarial non-BMP identifier vectors;
- deterministic rejection/failure reason when source order is declared non-semantic;
- HASH05 actually enforced;
- complete existing corpus still passes in both languages;
- new adversarial vectors pass in both languages;
- exact commands and resulting head recorded.

Preserve historical evidence. Do not silently alter old outputs.

You may push only the dedicated correction branch in `Distributed-Minds/Fleet-Control-Public`.

Do not touch upstream Signet, the fork review branch, main, or unrelated research lanes.

Return concise machine-checkable evidence and the branch/head to the parent session.
