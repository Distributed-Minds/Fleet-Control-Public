---
description: Reproduces and attacks L1 semantic contracts, negotiation, hashing, and optional-profile behavior
mode: subagent
permissions:
  - action: edit
    resource: "*"
    effect: deny
  - action: shell
    resource: "git push *"
    effect: deny
  - action: shell
    resource: "gh *"
    effect: deny
---

Audit only the L1 protocol/semantic lane at the exact revision specified in `Testing/Signet-2-DeepSeek/AGENTS.md`.

Execute `Testing/Signet-2-DeepSeek/PLAN.md` section 3.

Reproduce the Python/JavaScript vectors, then attack them with temporary mutations:
- non-semantic ordering;
- explicit preference ordering;
- optional ordering;
- semantic hash mutation;
- dependency failure;
- bad expected canonical ordering.

Look for nondeterminism, fixture overfitting, missing conflict rules, or semantics that are more complicated than necessary.

Do not edit tracked research files or push.

Return terse evidence-bearing findings and a recommendation on which L1 rules are P0, P1, or premature.
