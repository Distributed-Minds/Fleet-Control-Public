# Locked third-party crates — Phase0 Rust fixture prototype

**Inventory basis:** [`Cargo.lock`](Cargo.lock), Git blob `5f4611b68e5c0bdee7d235cd9ad7d31e9870e9f6`, captured 8 October 2026 for [draft PR #82](https://github.com/Distributed-Minds/Fleet-Control-Public/pull/82) / [issue #71](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/71). This is an evidence-scoped license and dependency **inventory**, not a release, distribution approval, binary scan, completed dependency-security audit, or legal clearance.

The lockfile contains the first-party `free-energy-phase0-fixtures 0.1.0` package and exactly **11 crates.io packages**, each with a registry source and SHA-256 package checksum. The table records version-specific crate license declarations or upstream license notices; the linked source directories are the review entry points. A registry checksum pins fetched bytes when verified by Cargo: it does **not** authenticate the maintainer, review executable build scripts, or guarantee a license entitlement.

| Exact locked crate | Declared SPDX expression / upstream notice |
| --- | --- |
| [`itoa` 1.0.18](https://docs.rs/crate/itoa/1.0.18/source/) | `MIT OR Apache-2.0` |
| [`memchr` 2.8.3](https://docs.rs/crate/memchr/2.8.3/source/) | `Unlicense OR MIT` |
| [`proc-macro2` 1.0.107](https://docs.rs/crate/proc-macro2/1.0.107/source/) | `MIT OR Apache-2.0` |
| [`quote` 1.0.47](https://docs.rs/crate/quote/1.0.47/source/) | `MIT OR Apache-2.0` |
| [`ryu` 1.0.23](https://docs.rs/crate/ryu/1.0.23/source/) | `Apache-2.0 OR BSL-1.0` |
| [`serde` 1.0.228](https://docs.rs/crate/serde/1.0.228/source/) | `MIT OR Apache-2.0` |
| [`serde_core` 1.0.228](https://docs.rs/crate/serde_core/1.0.228/source/) | `MIT OR Apache-2.0` |
| [`serde_derive` 1.0.228](https://docs.rs/crate/serde_derive/1.0.228/source/) | `MIT OR Apache-2.0` |
| [`serde_json` 1.0.145](https://docs.rs/crate/serde_json/1.0.145/source/) | `MIT OR Apache-2.0` |
| [`syn` 2.0.119](https://docs.rs/crate/syn/2.0.119/source/) | `MIT OR Apache-2.0` |
| [`unicode-ident` 1.0.26](https://docs.rs/crate/unicode-ident/1.0.26/source/) | `(MIT OR Apache-2.0) AND Unicode-3.0` |

## Important non-MIT-only cases

- **`unicode-ident 1.0.26`:** the crate uses Unicode Character Database-derived tables. Its authors distinguish ordinary source under **MIT OR Apache-2.0** from generated Unicode-derived data under **Unicode-3.0**, and require compliance with both applicable terms. See the [versioned crate overview](https://docs.rs/crate/unicode-ident/1.0.26) and [Unicode License](https://www.unicode.org/faq/unicode_license.html). Do **not** omit the Unicode copyright/permission notice merely because the FREE ENERGY root license is MIT.
- **`ryu 1.0.23`:** its offered alternatives are **Apache-2.0 OR BSL-1.0**, not MIT. Its [exact-version manifest](https://docs.rs/crate/ryu/1.0.23/source/Cargo.toml) and included [Apache / Boost license texts](https://docs.rs/crate/ryu/1.0.23/source/) govern attribution/redistribution choices.
- **`memchr 2.8.3`:** offers **Unlicense OR MIT**, rather than the `MIT OR Apache-2.0` pair common elsewhere. See its [exact-version manifest](https://docs.rs/crate/memchr/2.8.3/source/Cargo.toml.orig).

## What this does and does not verify

1. **Checked here:** inventory of package names/versions and registry checksum presence in the committed lockfile; versioned upstream licensing declarations/references. The project is a read-only, non-default Rust fixture candidate; it is **not** published as a binary or incorporated into the historical v0.1.2 ZIP.
2. **Still required before packaging or integration approval:** independently inspect the exact downloaded archives and their included `LICENSE*` texts, any attribution and notices required for the **actual distribution form**, crate provenance, known advisories, feature/dependency graph, and build-time execution. `serde`, `serde_core`, and `serde_json` declare build scripts; `serde_derive` is a procedural macro using `proc-macro2`, `quote`, and `syn`. Version pinning alone does not establish build-time safety.
3. **Scope of CI evidence:** current [Rust candidate workflow](../../.github/workflows/phase0-rust-containment.yml) fetches pinned Cargo dependencies with network access, then compiles/tests using `--locked --offline`. That is not a cold-cache network-free reproducibility proof, audit of crate sources, or a release-rights check.
4. **Review cadence:** refresh this inventory against the *actual* `Cargo.lock` before any merge, binary distribution, release, or dependency update; changes to locked package names, versions or integrity require a new exact-archive rights/provenance review. Do not silently carry these historical version claims forward to future crates.

This document does **not** vendor third-party source, logos, game assets, copyrighted game material, or license texts from external repositories. A future distribution must make an explicit, accurate choice of licensing/notice obligations for the shipped artifact; the repository's root MIT license does not relicense dependencies or games.
