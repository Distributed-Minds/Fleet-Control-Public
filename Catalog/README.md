# FREE ENERGY project catalog — Rust v0 implementation draft

Canonical specification: [issue #60](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/60), **Phase0 spec version 5**. Maintained executable-language/no-npm boundary: [issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70).

**Current implementation boundary (2026-10-09):** This non-default `Catalog/` directory now contains a compiled Rust crate, a committed `Cargo.lock`, typed manifest admission, several semantic checks, an offline static HTML renderer, a machine-readable `catalog_explain` binary, and compiled adversarial regression tests. The earlier statement that only declarative fixture inputs exist is obsolete. **These working components do not satisfy the whole v5 specification.** In particular, typed/handwritten structural checks are **not a complete offline JSON Schema Draft 2020-12 validator**; there is no authenticated reviewer authority, full rights clearance, playable-game validation, external asset hosting, or deployed catalog.

What each current source does:

- [`schema/project-v0.schema.json`](schema/project-v0.schema.json): published closed Draft 2020-12 structural contract. It forbids self-declared `APPROVED_FOR_SCOPE`; its simple `https` regex is not a security-grade link check. The current Rust CLI does **not** yet execute the complete schema dialect.
- [`src/lib.rs`](src/lib.rs): typed `serde` admission and executable fail-closed checks for many version, evidence/reference, URL, rights-scope, metadata and history invariants. Passing this partial boundary is not legal/source authentication.
- [`src/main.rs`](src/main.rs): `validate` manifests and `render [--check]` the generated static page. Symlink/path guards are best-effort filesystem checks, not race-free confinement against a concurrent privileged writer.
- [`src/bin/catalog_explain.rs`](src/bin/catalog_explain.rs): deterministic JSON explanation of admitted records, with an atomic whole-batch failure contract. Explanations explicitly disallow inferring FREE ENERGY redistribution authorization from contributor-supplied review strings.
- [`src/render.rs`](src/render.rs) and [`site/index.html`](site/index.html): escaped, deterministic, local **draft** static HTML; neither a production site nor a game download endpoint.
- [`tests/`](tests/) and [`fixtures/`](fixtures/): executable Rust tests and declarative adverse inputs, respectively. A fixture file's `expected_valid` field is not an executed verdict by itself.

Three deliberately conservative pilots remain:

- [`projects/luanti.json`](projects/luanti.json): `ENGINE_ONLY`; an engine without separately installed playable content.
- [`projects/openra.json`](projects/openra.json): `UPSTREAM_LINK_ONLY`; GPL engine rights do **not** license original Electronic Arts game assets.
- [`projects/veloren.json`](projects/veloren.json): canonical upstream development on GitLab, a distinct read-only GitHub mirror, and scoped soundtrack rights rather than whole-game reuse clearance.

All three are research `DRAFT` records; none claims `FREE_ENERGY_VERIFIED` play, a `TESTED` adapter, authenticated rights approval or a locally tested game. The cited upstream evidence was observed on 2026-10-08; no automatically fetched live-source/currentness verdict follows from that historical snapshot. The current tree cannot prove append-only history across Git revisions.

## Run the compiled Rust checks

The existing GitHub Actions jobs use pinned Rust **1.85.1**. On a supported development machine with Rust installed, fetch reviewed dependencies once into Cargo's cache **before** disconnected checks. The following commands run from the repository root:

```sh
rustup toolchain install 1.85.1 --profile minimal --component rustfmt --component clippy
rustup run 1.85.1 cargo fetch --manifest-path Catalog/Cargo.toml --locked

rustup run 1.85.1 cargo fmt --manifest-path Catalog/Cargo.toml -- --check
rustup run 1.85.1 cargo build --manifest-path Catalog/Cargo.toml --locked --offline
rustup run 1.85.1 cargo test --manifest-path Catalog/Cargo.toml --locked --offline
rustup run 1.85.1 cargo clippy --manifest-path Catalog/Cargo.toml --all-targets --locked --offline -- -D warnings
rustup run 1.85.1 cargo run --manifest-path Catalog/Cargo.toml --locked --offline -- validate Catalog/projects
rustup run 1.85.1 cargo run --manifest-path Catalog/Cargo.toml --locked --offline -- render --check
rustup run 1.85.1 cargo run --manifest-path Catalog/Cargo.toml --bin catalog_explain --locked --offline -- Catalog/projects/luanti.json
```

`fetch` requires reviewed network/dependency access on an uncached machine. Subsequent locked/offline commands cannot resolve uncached dependencies. Verify the GitHub Actions conclusions for the **exact candidate commit**, not for a different source head or a prior feature branch; passing tests prove their scoped assertions only. Do not treat `render --check` or typed validation as full Draft 2020-12 conformance or permission to deploy/redistribute.

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

## Remaining v5 acceptance work

1. Integrate a pinned/reviewed **offline Draft 2020-12 validator** (including the required subset of keywords and formats) and comprehensive schema-to-semantic cross-check fixtures, or explicitly keep full-schema acceptance open.
2. Complete the source-rights/evidence/currentness and record-scoping checks required by #60 without trusting manifest-authored reviewer strings. Keep the three pilot records and any public-facing HTML conservative.
3. Establish current-head integration and reproducibility evidence for the entire Rust pipeline and generated artifact after other concurrent child PRs land; resolve every failing CI job, merge conflict and stale fixture.
4. Obtain the required independent human review and authorized non-default integration. Rights clearance, actual playable builds, hosting, publication, public default-branch merges and implementation of #70's universal orchestration system are separate gates.

**Status:** compiled Rust draft, **not v5 complete**, not a published catalog, and not an authorized asset distribution. Do not treat child-branch CI successes as proof that this target branch, a release package or the public default branch contains the same tested code.
