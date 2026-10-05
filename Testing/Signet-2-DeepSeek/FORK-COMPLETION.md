# Signet 2 autonomous fork-delivery completion record

**Run date:** 2026-10-05
**Agent:** DeepSeek V4.1 Flash via OpenCode
**Status:** COMPLETE — public review branch ready; no remaining HIGH stop-ship finding.

## Delivered artifacts

| Artifact | Value |
|---|---|
| Public fork | https://github.com/geromet/signetprotocol |
| Review branch | `review/signet2-interoperability` |
| Review branch head | `ab313ff9e3005ac52451d985b6574e7f74b41fce` |
| Compare view | https://github.com/kian-cx/signetprotocol/compare/main...geromet:review/signet2-interoperability |
| Upstream base SHA | `2ddb136ee941705d3e1c020eaddad93be65026f2` (upstream `main` unmoved) |
| Corrected L1 branch | `correction/l1-cross-language-determinism` @ `f109e15ed1c73b0c76876524ec3c126e34bd63ae` |
| Corrected L2 branch | `correction/l2-role-contract-reproducibility` @ `ba65b131559a28cf0cb7d3416d492dbd1d91b316` |

The fork branch is docs-only: 11 files, all under `docs/content/`, +530/−15
(vs `2ddb136`). No patch files, no Fleet-Control machinery, no OpenCode agents,
no research-lane trees were copied into the fork.

## Corrections completed

### L1 — cross-language semantic determinism (`f109e15`)

- One normative ordering rule added (`HASH-2a` / `SEM-ORDER-1`): semantic
  reference comparison over `(id, definition_hash)` by UTF-16 code units,
  implemented identically in Python and JavaScript.
- Adversarial non-BMP composition vector added (previously diverged across
  languages).
- Deterministic rejection reason for declared-unordered input (`COMP-1a`, fixed
  phase precedence); previously reason depended on source order.
- HASH05 enforced (`SET_ARRAY_NOT_CANONICAL`).
- Full existing corpus plus new vectors pass in both languages.
- Historical L1 head and outputs preserved.

### L2 — role-contract reproduction (`ba65b13`)

- Malformed `role-contract.schema.json` fixed.
- All generated JSON/C/Rust artifacts regenerated from the canonical generator.
- Digest/evidence manifest regenerated from the regenerated outputs.
- `check.py` reproduces `minecraft-responsibility-coverage: 7/7` from source.
- Deterministic-generation checks rerun (2 clean runs byte-identical, C/Rust
  compile, negative descriptors rejected ×3).
- Resolver `lookup` relabelled `gold_oracle` and documented as an oracle upper
  bound, not resolver performance.
- Historical broken evidence preserved under `historical/`; original L2 head
  unchanged.

### Fork review branch corrections

- Corrected the over-broad "nobody can cheat" sentence, the common-subset /
  "ignore or approximate" wording, pinned-vs-current staleness, resolver
  `NO_MATCH`, and local-approval-vs-dataset-contribution in upstream docs.
- Added two focused proposal pages (`semantic-contracts.mdx` P0-primary,
  `translator-trust-and-validation.mdx` P1-secondary with P2+ marked
  non-blocking) and updated `proposals/_meta.js` nav order.
- Pinned the L1/L2 evidence links to the corrected commits (no placeholders).
- Final-audit fixes: replaced non-existent reason codes
  (`DEPENDENCY_VERSION_MISMATCH`, `NOT_OFFERED`) with valid enum members,
  split a union evidence claim per commit, repointed resolver metrics to the
  pinned L2 notebook, and inline-labelled the two proposal sentences in forge
  pages.

## Exact commands and results

| Command | Result |
|---|---|
| L1: `bash reference/run-all.sh` (Python + Node) | PASS — all existing + new vectors; py output == js output |
| L2: `python3 run_generation_validation.py` | PASS — deterministic generation, C + Rust compile, 3 negative descriptors rejected |
| L2: `python3 check.py` | PASS — schema PASS, `7/7`, role-contract PASS |
| L2: `python3 benchmark.py` | PASS — `results.json` reproduced |
| Fork: `cd docs && npm ci` | PASS — 438 packages |
| Fork: `cd docs && npm run build` | PASS — compiled, 43/43 routes, 40 pages indexed, exit 0 |
| Fork: internal link check | PASS — 0 broken links on changed pages |
| Fork: forbidden-content scan | PASS — no harness/agents/patches/coordination |
| Final adversarial audit (fresh subagent, ×2) | PASS — no HIGH; contact-ready |

## Final audit verdict

`CONTACT-READY` — the second pass confirmed the prior HIGH/MED findings are
resolved and introduced none. Residual LOW items only (see limitations).

## Remaining limitations

- **Godot 4.7.2 runtime:** BLOCKED (binary not materializable in this
  environment). `ABI_HOST_HARNESS_PASS` stands and is not broadened to
  `GODOT_RUNTIME_PASS`.
- **Resolver pilot:** six-case seed; `gold_oracle` is an oracle upper bound,
  not a resolver benchmark. Only `lexical`/`first` are resolver-shaped controls.
- **HASH05 scope:** enforced on hashed normative definitions and the contract's
  `profiles` set array; composition *inputs* remain runtime-canonicalized per
  `COMP-1a` (documented decision that keeps the existing corpus valid).
- Upstream's generated `/signet-2-architecture.pdf` diverges from the corrected
  pages; docs are the source of truth.
- The public notebook name and internal lane paths (`Fleet-Control-Public`,
  `Signet-Frontier-2026-10/lanes/L1`) are visible to an upstream reader; the
  repository is intentionally public evidence.
- Original L2 environment lacked `rustc`; the correction environment had it, so
  the regenerated manifest records Rust compilation as an actual run.

## Discord message draft (to send)

> Hi — I've put my feedback on a review branch in my fork rather than editing upstream: https://github.com/geromet/signetprotocol/tree/review/signet2-interoperability
>
> It's docs-only, based on `2ddb136`. The main proposal argues for versioned semantic identity, capability offers that negotiate a deterministic session contract, typed fallback, resolver `NO_MATCH`/`ABSTAIN`, and executable conformance vectors; a smaller page covers evidence boundaries and explicitly non-blocking lifecycle work. The L1/L2 experiments are pinned to exact commits in a public notebook, and the docs build clean (40 pages).
>
> I'd genuinely welcome being told where this is wrong — especially if the semantic core is already implied by the current draft, or if the P0/P1/P2 framing is off. Happy to split, shrink, or drop anything.
