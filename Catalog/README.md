# FREE ENERGY project catalog — v0 data seed (implementation in progress)

Canonical specification: [issue #60](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/60), **Phase0 spec version 5**. Governing executable-language/no-npm rule: [issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70).

This non-default draft currently provides a **reviewable, declarative JSON Schema, three conservative pilot manifests, and several authored fixture files**. The fixture expectations are inputs for a future compiled runner, not passing executable tests. This is not an installable catalog product, published catalog, verified game index, validator, renderer, authorized asset collection, or completed v5 implementation.

- `schema/project-v0.schema.json`: closed Draft 2020-12 **structural** schema; `APPROVED_FOR_SCOPE` is deliberately not a permitted v0 permission decision. Its `https` regex is **not** the required security-grade URL validation.
- `projects/luanti.json`: an engine-only route; separately installed game/content required.
- `projects/openra.json`: upstream download pointer; GPL engine rights do **not** license Electronic Arts original-game assets.
- `projects/veloren.json`: GitLab canonical development; the separately pinned GitHub mirror and CC BY-NC-SA soundtrack evidence are scoped rather than a whole-game reusability claim.

All three records are `DRAFT`; no `FREE_ENERGY_VERIFIED` play status, `TESTED` adapter, `APPROVED_FOR_SCOPE` rights grant or local game/build test is asserted. Evidence dates describe the historical 2026-10-08 research snapshots. Moving upstream pages are **not** pinned repository evidence. Publisher rights statements remain subject to use-specific independent review. This format does **not** mechanically guarantee append-only claim history or authenticate a manifest's `reviewer` string.

## Verification fixture snapshot — 847af19d (not executed)

The seven files below were inspected at [source commit `847af19d`](https://github.com/Distributed-Minds/Fleet-Control-Public/commit/847af19d3fd9a58403b902e24772af98b61bb064). This is a **historical snapshot, not an exhaustive live directory inventory**: other parallel operators have since added fixture files. Check the [current fixture directory](fixtures/) for newer inputs. These are source-controlled test inputs/expected verdicts, **not executed Rust tests** or proof of Draft 2020-12 or semantic validator conformance. Paths below are relative to `Catalog/`.

| Fixture | Input contract | Declared cases |
| --- | --- | ---: |
| [`fixtures/evidence-reference-shape-v0.json`](fixtures/evidence-reference-shape-v0.json) | Play/adapter evidence-reference shape; not proof of actual evidence | 10 |
| [`fixtures/not-authorized-review-metadata-v0.json`](fixtures/not-authorized-review-metadata-v0.json) | Denial-review metadata shape, not authenticated reviewer authority | 10 |
| [`fixtures/pinned-evidence-structure-v0.json`](fixtures/pinned-evidence-structure-v0.json) | Immutable repository-file coordinates and normalized relative paths | 27 |
| [`fixtures/rights-path-scope-v0.json`](fixtures/rights-path-scope-v0.json) | Rights PATH/PREFIX structure and normalized path segments | 23 |
| [`fixtures/tested-adapter-target-v0.json`](fixtures/tested-adapter-target-v0.json) | TESTED adapter target-ID shape, not evidence of adapter execution | 13 |
| [`fixtures/unpinned-upstream-reason-v0.json`](fixtures/unpinned-upstream-reason-v0.json) | Required explanation for an unpinned upstream revision | 8 |
| [`fixtures/forged-approval-v0.json`](fixtures/forged-approval-v0.json) | One **complete negative manifest**, not a fragment suite: a forged human-review record attempts forbidden `APPROVED_FOR_SCOPE` | 1 record |

At that source snapshot, the first six files declared **91 fragment cases** (26 expected valid, 65 expected invalid) using `base`/`base_*`, `target_pointer`, case overrides and `expected_valid`; they need an actual compatible schema runner to resolve the target `$defs`, apply overrides, and compare outcomes. At the same source snapshot, the seventh file was a separate full-manifest negative to reject regardless of self-asserted review evidence. These counts describe authored expectations only; **none of those 92 snapshot inputs constitutes an observed test PASS**. Structural acceptance would not prove source authenticity, safe URL destinations, rights clearance, a tested adapter, or a working downloadable game.

## Remaining implementation (not yet delivered)

1. Add `Cargo.toml`, audited and committed `Cargo.lock`, typed Rust `src/lib.rs`/`src/main.rs`, and a Rust Draft 2020-12 schema validator with remote reference retrieval disabled.
2. Implement deterministic offline semantic checks: full revision pins, records/evidence/claim references, permission forgery rejection, URL/host/userinfo/port/encoded-delimiter rejection, safe scoped license conflicts, play/adapter evidence, internal provenance and currentness.
3. Add Rust fixture-based adversarial tests, escaped deterministic static `site/index.html` generator, and output regeneration check; no Python/JavaScript/Node/npm project tooling.
4. Prime audited Cargo dependencies in a separate trusted setup phase; verify `cargo fmt`, locked offline `clippy`, `build`, `test`, `validate` and `render --check` on a real Rust toolchain. **No Rust compiler or Cargo is available in this authoring environment; no such tests have been run.**

This PR must remain **draft/unmerged** until the executable Rust implementation, lockfile, fixture tests, and reproducible verification exist. No changes to `main`, release/site configuration, third-party game repos or catalog hosting are authorized.
