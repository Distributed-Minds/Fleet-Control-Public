# Autonomous Signet fork handoff

You are now implementing the maintainer-facing Signet 2 review artifact.

The human does **not** need to be kept in the decision loop. Make reasonable engineering decisions autonomously, document them, and stop only at the explicit contact boundary below.

## Read first

1. `Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`
2. `Publications/Game-Interoperability/SIGNET-2-FORK-DELIVERY-PLAN.md`
3. `Publications/Game-Interoperability/SIGNET-2-RESPONSE-HANDOFF.md`
4. `Publications/Game-Interoperability/SIGNET-2-PROPOSAL-DELTA-MATRIX.md`

Use the full response paper only when deeper rationale/evidence is needed.

## Mission

Produce a public, buildable, maintainer-friendly fork:

`geromet/signetprotocol`

with branch:

`review/signet2-interoperability`

The branch itself is the proposal.

Do **not** make the old patch bundle the delivery artifact.

## Authority

You may autonomously:

- create the public GitHub fork under `geromet` if it does not already exist;
- clone/fetch both repositories;
- create local worktrees;
- create dedicated correction branches in `Distributed-Minds/Fleet-Control-Public` for the known L1/L2 evidence defects;
- push those dedicated correction branches;
- create and push `review/signet2-interoperability` to `geromet/signetprotocol`;
- edit Signet docs directly in the fork;
- simplify/restructure our proposal when doing so preserves the demonstrated invariants;
- run builds/tests;
- update exact evidence links to corrected branch heads;
- write final local/branch validation notes.

You must not:

- mutate `kian-cx/signetprotocol`;
- open an upstream PR/issue/Discussion;
- message/contact the maintainer;
- merge into upstream;
- rewrite historical L1/L2 branch history;
- claim tests passed if they were not executed.

## Parallelize

Prefer independent subagents for:

1. L1 correction + adversarial vectors;
2. L2 correction + regeneration/reproduction;
3. fork docs implementation/build;
4. final simplification/upstream-fidelity review.

Use the project repair/build subagents if available. Otherwise create equivalent built-in subagent tasks.

Do not duplicate the full corpus into every subagent context.

## Required corrections

### L1

Fix the DeepSeek-reported determinism gaps on a dedicated correction branch:

- normative comparator/order rule shared by Python and JavaScript;
- non-BMP tests;
- deterministic rejection reason for unordered input;
- HASH05 enforcement;
- complete cross-language regression.

Do not rewrite the original L1 head.

### L2

Fix on a dedicated correction branch:

- malformed role-contract schema;
- regenerate all generated artifacts from source;
- regenerate digest/evidence manifest;
- rerun role-contract coverage;
- rerun deterministic generation checks.

Do not hand-edit generated output as the final fix.

Do not rewrite the original L2 head.

## Fork implementation

Start from current upstream Signet after inspecting whether the draft moved since `2ddb136e`.

Apply our proposal as normal direct edits, not as patch files.

Prioritize:

### P0
- semantic identity/versioning;
- modular semantic profiles;
- deterministic negotiated session contract;
- required/optional semantics;
- typed fallback;
- explicit session preferences;
- deterministic optional activation;
- profile authority classes;
- publisher baseline/participant overlay;
- resolver abstention;
- canonical composition/test vectors.

### P1
- mapping provenance/freshness;
- effective profile identity;
- minimal integration/trust evidence boundaries.

### P2+
Keep future-looking and clearly non-blocking.

Do not make mature supply-chain/governance infrastructure appear mandatory for the first Signet 2 implementation.

## Verify

At minimum:

- build the fork docs using upstream's own documented commands;
- verify all new internal links/routes;
- rerun corrected L1;
- rerun corrected L2;
- inspect the fork as a skeptical upstream maintainer;
- compare the fork against current upstream, not only the old snapshot;
- ensure no internal Fleet-Control operational artifacts leaked into the fork.

If practical, use another fresh subagent for the final adversarial review.

## Final artifacts

In Fleet-Control-Public, write/update a concise record containing:

- corrected L1 branch + head;
- corrected L2 branch + head;
- fork URL;
- review branch URL;
- upstream base SHA;
- fork review-branch head;
- exact commands/tests run;
- PASS/PARTIAL/BLOCKED state;
- any remaining limitations.

In the fork branch, add only maintainer-useful documentation. Do not copy the Fleet-Control validation harness.

## Contact boundary

**Do not contact the maintainer.**

Stop when all of the following exist:

- public `geromet/signetprotocol` fork;
- pushed `review/signet2-interoperability` branch;
- docs build/test evidence;
- repaired supporting evidence;
- no remaining HIGH stop-ship finding;
- a short Discord message draft containing the branch URL.

Then report completion to the human.

The human's only remaining action should be sending the Discord message.
