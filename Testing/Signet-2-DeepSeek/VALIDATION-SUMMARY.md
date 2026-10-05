# DeepSeek/OpenCode pre-contact validation summary

**Run date:** 2026-10-05  
**Model:** DeepSeek V4.1 Flash via OpenCode  
**Verdict:** `READY_AFTER_MINOR_FIXES`  
**Purpose:** record the actionable outcome of the independent validation run before maintainer contact.

This is a concise handoff summary of the completed local validation run. The full local `RESULTS.md` was not pushed by the validator.

## What passed

The validator reported:

- no material upstream strawman;
- all key upstream quotations were accurate;
- 10/10 delta-matrix upstream blob hashes verified;
- the shipped L1 vectors reproduced identically in Python and JavaScript on the committed fixtures;
- the L2 negative/generic-shim result reproduced;
- the measured resolver metrics reproduced;
- after correcting patch 0001 locally, the complete proposed docs tree built cleanly.

These results support continuing toward maintainer contact.

## Stop-ship findings

### 1. Patch 0001 applicability claim is false as committed

**Severity:** HIGH

The validator reported that `git apply --check` fails against upstream `2ddb136e` on two EOF hunks in:

`Publications/Game-Interoperability/Signet-2-Proposed-Upstream-Patches/0001-harden-draft-semantics.patch`

The existing `APPLY_CHECK = PASS` claim is therefore not reliable as shipped.

A two-line local correction reportedly made patch 0001 apply, after which 0002/0003 applied and the docs build passed.

### 2. L2 role-contract evidence is not self-reproducing

**Severity:** HIGH

At L2 head:

`a5a677f7bf3e80d2face8f2185b27fcb8e394ab3`

the validator reported:

- `role-contract.schema.json` contains an extra closing brace;
- `check.py` therefore fails during JSON parsing;
- after the one-character schema correction, the claimed Minecraft role coverage reproduces as 7/7;
- committed generated JSON/header artifacts do not match the recorded digest manifest.

Before citing the 7/7 result as reproducible, regenerate all generated artifacts and digest evidence from the corrected source and rerun the checks.

### 3. L1 has latent cross-language determinism gaps

**Severity:** MED/HIGH

At L1 head:

`b8f47a332fcb0aa641cfe676e22b49a801731628`

the committed fixtures reproduce, but the validator found edge cases not covered by them:

- Python and JavaScript semantic-reference ordering use different string-order rules for non-BMP identifiers;
- at least one rejection reason can depend on input order even where the corresponding collection is declared unordered;
- HASH05 is documented but not enforced by the current vector runner.

The next revision should define one normative comparator/canonical ordering rule and add adversarial vectors that exercise these cases in both languages.

### 4. The maintainer-facing priority is too broad

**Severity:** MED

The short response currently gives the semantic core and the full trust/supply-chain/governance program similar visual weight.

The delta matrix already implies a better priority:

- **P0:** semantic identity, deterministic negotiation/composition, abstention, executable vectors;
- **P1:** provenance/freshness and minimal trust boundaries;
- **P2+:** richer TUF/Sigstore-style distribution, lifecycle and governance machinery.

The public fork should reflect that smaller first-implementation scope.

## Interpretation

The independent run found repairable evidence/implementation defects, not a collapse of the core response.

Do not contact the maintainer until the HIGH findings are fixed and the targeted regression passes.

Do not spend another broad research cycle unless those corrections reveal a new architectural problem.
