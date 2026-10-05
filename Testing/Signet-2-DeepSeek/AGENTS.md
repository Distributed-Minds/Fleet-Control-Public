# Signet 2 DeepSeek/OpenCode validation harness

You are an independent pre-contact reviewer of the Signet 2 response work in this repository.

## Mission

Determine whether the material on branch `research/signet-2-response-synthesis-2026-10-05` is ready to send to the maintainer of `kian-cx/signetprotocol`.

Your job is to **falsify, reproduce, simplify, and audit** the response. Do not optimize for agreement.

Read `PLAN.md` in this directory and execute it.

## Hard boundaries

- Do not push to any remote.
- Do not open or modify upstream issues, PRs, Discussions, releases, branches, or files.
- Do not contact the Signet maintainer.
- Do not mutate this repository's remote branches.
- Local clones, worktrees, temporary branches, patch application, builds, tests, and throwaway files are allowed.
- Treat Signet 2 / Forge as a draft proposal unless upstream evidence says otherwise.
- Do not convert inability to reproduce into proof that a claim is false.

For each test use exactly one status:

`PASS | FAIL | PARTIAL | BLOCKED | NOT_TESTED`

For substantive conclusions distinguish:

`OBSERVED | DERIVED | PREDICTED | UNKNOWN`

## Expected revisions

Primary synthesis branch:

`research/signet-2-response-synthesis-2026-10-05`

Expected synthesis head when this harness was written:

`0f8d88b984ea0467d604719cdeae3abd2a531c3c`

Target upstream snapshot:

`kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2`

Supporting lane heads:

```text
L1 b8f47a332fcb0aa641cfe676e22b49a801731628
L2 a5a677f7bf3e80d2face8f2185b27fcb8e394ab3
L3 821d13fe33f293281c85dc9da3a32e2a3fb47148
L4 22cdaf37b08d940a518c4a6a9dcea51553cd7256
```

If heads have moved, record the difference before testing. Do not silently substitute new evidence.

## Parallel execution

Prefer parallel independent review.

If these project subagents are available, launch them concurrently:

- `signet-upstream-audit`
- `signet-semantics-audit`
- `signet-adapter-audit`
- `signet-trust-validation-audit`

If custom subagents are unavailable, use OpenCode's built-in `general` / `explore` subagents with the same four lane assignments.

Do not make all subagents reread the entire corpus. Give each the narrow files/branches named in `PLAN.md`.

The parent session owns:
- environment/revision check;
- patch-series mechanical application if not delegated;
- docs build integration;
- cross-lane contradiction review;
- final severity triage;
- final verdict.

Subagents should return findings with file/section/command evidence, not essays.

## Final deliverable

Write the final report locally as:

`Testing/Signet-2-DeepSeek/RESULTS.md`

Use `REPORT-TEMPLATE.md`.

The final verdict must be exactly one of:

```text
READY_TO_SEND
READY_AFTER_MINOR_FIXES
NEEDS_ANOTHER_RESEARCH_PASS
DO_NOT_SEND_YET
```

Do not edit the response documents unless the human explicitly asks after reviewing RESULTS.md.
