---
description: Reproduces L2 resolver and real-target adapter evidence for Signet response
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

Audit only the L2 adapter/resolver evidence at the exact revision specified in `Testing/Signet-2-DeepSeek/AGENTS.md`.

Execute `Testing/Signet-2-DeepSeek/PLAN.md` section 4.

Priority:
1. independently reproduce/check resolver metrics and candidate-order leakage;
2. verify Godot evidence is no broader than `ABI_HOST_HARNESS_PASS`;
3. reproduce/check Minecraft v0 coverage and the revised 7/7 role-contract fixture;
4. try to falsify the revised abstraction or show target overfitting.

If runtime dependencies are unavailable, mark the exact test BLOCKED rather than inferring a pass/fail.

Do not push or mutate tracked evidence.

Return concise findings with exact commands/artifacts.
