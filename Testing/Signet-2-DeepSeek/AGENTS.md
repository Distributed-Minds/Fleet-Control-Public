# Signet 2 DeepSeek/OpenCode autonomous fork-delivery harness

The independent validation phase is complete.

Historical validation result:

`READY_AFTER_MINOR_FIXES`

The active mission is now to repair the known reproducibility defects and produce the public maintainer-facing Signet fork.

## Start here

The fork exists and is contact-ready technically. The active task is now **presentation simplification** so the maintainer does not have to dig through the docs tree.

Read, in order:

1. `Testing/Signet-2-DeepSeek/FORK-PRESENTATION-HANDOFF.md`
2. `Testing/Signet-2-DeepSeek/FORK-COMPLETION.md`
3. `Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`
4. `Publications/Game-Interoperability/SIGNET-2-FORK-DELIVERY-PLAN.md`

The older `PLAN.md` is retained as completed validation history. Do **not** rerun the entire original validation plan unless a targeted regression points to a new systemic problem.

## Mission

Improve the **maintainer-facing presentation** of the existing public branch:

`geromet/signetprotocol@review/signet2-interoperability`

Required result:

- root-level `SIGNET2-REVIEW.md` that explains the whole proposal in 2–4 minutes;
- a prominent README banner linking to it;
- detailed docs remain drill-down material;
- docs build still passes;
- a fresh reviewer can understand the P0 proposal and evidence without opening Fleet-Control;
- no contact with upstream yet.

The human does not need to approve intermediate engineering decisions.

## Hard contact boundary

You may create and push correction branches and the public fork review branch.

You must **not**:

- mutate `kian-cx/signetprotocol`;
- open an upstream PR, issue, or Discussion;
- contact the Signet maintainer;
- merge into upstream.

Stop with the branch ready and the Discord message drafted.

## Parallel execution

Prefer these independent project subagents:

- `signet-l1-repair`
- `signet-l2-repair`
- `signet-fork-builder`

Run the final:

- `signet-final-audit`

after the correction heads and fork branch exist.

If custom subagents are unavailable, create equivalent built-in subagent tasks.

Do not make every subagent reread the full research corpus.

## Evidence rules

For each executed test use:

`PASS | FAIL | PARTIAL | BLOCKED | NOT_TESTED`

For substantive conclusions distinguish:

`OBSERVED | DERIVED | PREDICTED | UNKNOWN`

Do not broaden narrow evidence labels.

Examples:

`ABI_HOST_HARNESS_PASS != GODOT_RUNTIME_PASS`

`PINNED_CHOICE != CORRECT_CHOICE`

`VALID_SIGNATURE != CURRENT_AUTHORIZATION`

## Historical revisions

Original synthesis snapshot used for the completed validation:

`0f8d88b984ea0467d604719cdeae3abd2a531c3c`

Historical L1 head:

`b8f47a332fcb0aa641cfe676e22b49a801731628`

Historical L2 head:

`a5a677f7bf3e80d2face8f2185b27fcb8e394ab3`

Historical L3 head:

`821d13fe33f293281c85dc9da3a32e2a3fb47148`

Historical L4 head:

`22cdaf37b08d940a518c4a6a9dcea51553cd7256`

Historical upstream Signet snapshot:

`2ddb136ee941705d3e1c020eaddad93be65026f2`

Before doing fork work, inspect current upstream state and record any movement.

## Final report

Write/update a concise completion record in Fleet-Control-Public containing:

- corrected L1 branch/head;
- corrected L2 branch/head;
- upstream base SHA;
- public fork URL;
- review branch URL/head;
- build/test commands and results;
- remaining limitations;
- final audit verdict;
- Discord message draft.

Do not copy the internal validation harness into the Signet fork.
