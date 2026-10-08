# DeepSeek validation plan

## 0. Establish exact state

Record:
- OpenCode version;
- active model/provider identifier;
- OS;
- Python/Node/Rust/Godot versions when relevant;
- synthesis branch/head;
- upstream head and exact tested upstream revision;
- L1-L4 heads.

Confirm whether the current repository state still matches the expected revisions in AGENTS.md.

## 1. Cold-read and upstream fidelity

Read in this order:

1. `Publications/Game-Interoperability/SIGNET-2-RESPONSE-HANDOFF.md`
2. `Publications/Game-Interoperability/SIGNET-2-UPSTREAM-PROTOCOL-PROPOSAL-DRAFT.md`
3. `Publications/Game-Interoperability/SIGNET-2-PROPOSAL-DELTA-MATRIX.md`
4. `Publications/Game-Interoperability/Signet-2-Proposed-Upstream-Patches/README.md`
5. only then the full response paper.

Independently inspect upstream Signet 2 / Forge.

Find:
- strawmen;
- claims upstream already solves;
- incorrect quotations/characterizations;
- proposal vs implemented-behavior confusion;
- overclaims.

Critical invariants to challenge:

```text
SHARED_LABELS != SHARED_SEMANTICS
PINNED_CHOICE != CORRECT_CHOICE
SERVER_AUTHORITY != CHEAT_IMPOSSIBILITY
DECLARED_PERMISSION != ENFORCED_PERMISSION
VALID_SIGNATURE != CURRENT_AUTHORIZATION
PACKAGE_BYTES != EFFECTIVE_PROFILE
MODEL_SUGGESTION != AUTHORITY
LOCAL_APPROVAL != DATASET_CONTRIBUTION
O(n) mapping topology != O(n) empirical validation demand
```

## 2. Patch-series test

Against an expendable checkout/worktree pinned to:

`kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2`

test, in order:

```text
0001-harden-draft-semantics.patch
0002-semantic-contracts-and-profile-composition.patch
0003-translator-trust-and-validation.patch
```

Run `git apply --check`, then apply them.

Also test patch 1 by itself.

Verify changed files match the patch README.

Then inspect upstream's own documented build/check commands and run the relevant documentation build/checks after patch application.

Separate:
- patch applies;
- docs syntax/build passes;
- CI passes;
- maintainer accepts.

Never infer later claims from earlier ones.

## 3. L1 semantics — executable reproduction

Use exact L1 head from AGENTS.md.

Locate and run the reference harness. Independently verify Python and JavaScript agreement for:
- hash vector;
- composition vector;
- negotiation vector;
- optional negotiation vector;
- contract vector;
- additional positive vectors;
- negative vectors.

Then mutate temporary copies:
- reorder non-semantic sets;
- reorder explicit preference lists;
- reorder optional preference lists;
- change one semantic hash;
- break one dependency;
- corrupt one expected canonical ordering.

Check that behavior changes only where the proposed semantics say it should.

Key properties:
- deterministic same-input result;
- non-semantic order stability;
- explicit preference order remains semantic;
- required baseline cannot be rescued by an optional;
- skipped optionals retain exact reasons.

## 4. L2 adapter/resolver evidence

Use exact L2 head.

### Resolver
Reproduce or independently calculate the reported metrics.

Audit:
- candidate ordering leakage;
- `NO_MATCH` cases;
- perturbation methodology;
- metric calculations;
- whether conclusions exceed dataset size.

Try at least one deliberately weak alternative baseline.

### Godot
Verify that the evidence supports exactly:

`ABI_HOST_HARNESS_PASS`

and not a broader runtime claim.

If a compatible Godot runtime is unavailable, mark runtime reproduction BLOCKED.

### Minecraft RCON
Verify the original generic-shim result:

`NO=5, PARTIAL=2, FAIL_EXPECTED`

Then verify whether the role model:

`HOST / IMPORTER / WORLD / INPUT / PRESENTATION / AUTHORITY`

really covers the documented fixture 7/7.

Challenge whether it is overfit to one target.

## 5. L3 trust audit

Use exact L3 head.

Check that the response consistently separates:
- role contract;
- sandbox/enforcement;
- signature;
- current publisher authorization;
- provenance;
- reproducibility;
- protocol conformance;
- game compatibility;
- effective profile;
- integration mode.

Classify each trust recommendation:

`MUST_HAVE_FOR_V2 | SHOULD_DESIGN_FOR_NOW | CAN_DEFER | OVERENGINEERED`

Pay particular attention to whether a young protocol is being burdened with mechanisms that can safely remain future work.

## 6. L4 validation audit

Use exact L4 head.

Attack:

`O(n) mapping topology != O(n) empirical validation demand`

For major claims determine:
- local conformance sufficient?
- independent implementation needed?
- pairwise target test needed?
- periodic freshness needed?

Try to reduce the validation burden while preserving falsifiability.

## 7. Simplicity attack

Assume you are Signet's single maintainer and want to ship.

Classify recommendations:

`P0 | P1 | P2 | P3 | REMOVE`

Where:
- P0 = needed before implementation;
- P1 = design for now;
- P2 = useful later;
- P3 = research-only/premature;
- REMOVE = unjustified.

Design the smallest Signet 2 architecture that preserves the strongest demonstrated invariants.

Explicitly report anything our proposal overengineers.

## 8. Independent alternative

Create a compact alternative design under these fixed constraints:
- preserve game -> shared meaning -> game;
- authoritative server;
- deterministic runtime;
- model not required per tick;
- multiple game genres;
- participant-owned presentation.

Compare it against our proposal.

This is a falsification test, not a request to generate another large architecture.

## 9. Cross-artifact consistency

Compare:
- maintainer handoff;
- protocol proposal draft;
- delta matrix;
- patch series;
- full paper.

Find contradictions in:
- terminology;
- evidence level;
- proposed priority;
- experimental numbers;
- upstream revision references.

## 10. Final readiness

Produce `RESULTS.md` from REPORT-TEMPLATE.md.

A send-ready result requires, at minimum:
- no material upstream strawman;
- patch series applies;
- documentation build passes or failures are clearly unrelated;
- L1 vectors reproduce;
- key L2 claims reproduce or are explicitly BLOCKED;
- evidence labels stay narrow;
- no high-severity contradiction across artifacts;
- complexity attack finds no obviously simpler design that preserves the same demonstrated guarantees.

Do not require model agreement. Require factual and reproducibility robustness.
