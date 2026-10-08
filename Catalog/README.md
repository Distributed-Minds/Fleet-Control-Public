# FREE ENERGY project catalog — v0 data seed (implementation in progress)

Canonical specification: [issue #60](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/60), **Phase0 spec version 5**. Governing executable-language/no-npm rule: [issue #70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70).

This initial non-default branch commit provides only the **reviewable, declarative JSON Schema and three conservative pilot manifests**. It is not an installable catalog product, published catalog, verified game index, validator, renderer, authorized asset collection, or completed v5 implementation.

- `schema/project-v0.schema.json`: closed Draft 2020-12 **structural** schema; `APPROVED_FOR_SCOPE` is deliberately not a permitted v0 permission decision. Its `https` regex is **not** the required security-grade URL validation.
- `projects/luanti.json`: an engine-only route; separately installed game/content required.
- `projects/openra.json`: upstream download pointer; GPL engine rights do **not** license Electronic Arts original-game assets.
- `projects/veloren.json`: GitLab canonical development; the separately pinned GitHub mirror and CC BY-NC-SA soundtrack evidence are scoped rather than a whole-game reusability claim.

All three records are `DRAFT`; no `FREE_ENERGY_VERIFIED` play status, `TESTED` adapter, `APPROVED_FOR_SCOPE` rights grant or local game/build test is asserted. Evidence dates describe the historical 2026-10-08 research snapshots. Moving upstream pages are **not** pinned repository evidence. Publisher rights statements remain subject to use-specific independent review. This format does **not** mechanically guarantee append-only claim history or authenticate a manifest's `reviewer` string.

## Remaining implementation (not yet delivered)

1. Add `Cargo.toml`, audited and committed `Cargo.lock`, typed Rust `src/lib.rs`/`src/main.rs`, and a Rust Draft 2020-12 schema validator with remote reference retrieval disabled.
2. Implement deterministic offline semantic checks: full revision pins, records/evidence/claim references, permission forgery rejection, URL/host/userinfo/port/encoded-delimiter rejection, safe scoped license conflicts, play/adapter evidence, internal provenance and currentness.
3. Add Rust fixture-based adversarial tests, escaped deterministic static `site/index.html` generator, and output regeneration check; no Python/JavaScript/Node/npm project tooling.
4. Prime audited Cargo dependencies in a separate trusted setup phase; verify `cargo fmt`, locked offline `clippy`, `build`, `test`, `validate` and `render --check` on a real Rust toolchain. **No Rust compiler or Cargo is available in this authoring environment; no such tests have been run.**

This PR must remain **draft/unmerged** until the executable Rust implementation, lockfile, fixture tests, and reproducible verification exist. No changes to `main`, release/site configuration, third-party game repos or catalog hosting are authorized.
