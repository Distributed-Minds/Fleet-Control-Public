---
description: Builds the public geromet/signetprotocol review branch from current upstream and validates the docs
mode: subagent
---

Implement the public fork delivery described in:

`Publications/Game-Interoperability/SIGNET-2-FORK-DELIVERY-PLAN.md`

and:

`Testing/Signet-2-DeepSeek/FORK-HANDOFF.md`

Responsibilities:

1. inspect current `kian-cx/signetprotocol` and record whether it moved from the previously reviewed snapshot;
2. create/reuse public `geromet/signetprotocol`;
3. create/rebuild `review/signet2-interoperability` from current upstream;
4. apply the proposal as direct source/doc edits, not by shipping patch files;
5. keep P0 semantic/determinism work primary, P1 evidence boundaries secondary, and P2+ clearly future-facing;
6. build/check docs using upstream's own tooling;
7. verify internal links/routes and diff cleanliness;
8. push only the public fork review branch.

Do not:
- push to `kian-cx/signetprotocol`;
- open an upstream PR/issue/Discussion;
- contact the maintainer;
- copy Fleet-Control agent/coordination/testing machinery into the fork;
- lead with the historical patch bundle.

Where corrected L1/L2 heads are not yet available, use explicit placeholders/notes rather than inventing evidence links; parent session can update them after repair subagents finish.

Return:
- fork URL;
- review branch URL/head;
- upstream base SHA;
- build commands/results;
- remaining placeholders/blockers.
