# Coordination archival admission — issue #22, spec 5

Pure Rust, standard-library-only **model component**. It evaluates independent, typed eligibility receipts for one archival source deletion and reconstructs a coherent ordered archive/live history with exact overlap deduplication, identity checks and gap rejection.

**No destructive side effects.** This component cannot prove that a provider-issued receipt is authentic, current, or sufficiently trusted; nor can a test fixture, comment, boolean or successful model evaluation grant deletion authority. Production integration requires independently trusted providers for #10 effect-time authorization/fencing, authoritative reducer order, unique manifest selection, coherent snapshot/cutover, exact live identity, authoritative reconstruction horizon, and current durability/key custody. All effect-time revalidation and actual compaction remain unimplemented.

Commands from this directory with Rust 1.85.1:

    cargo fmt --all -- --check
    cargo test --locked --offline
    cargo clippy --all-targets --locked --offline -- -D warnings

This is an independent tested **slice**, not satisfaction of the full #22 acceptance (provider adapters, large synthetic histories, permanent archival storage, runtime integration, migration, production cleanup). Older history lacks these proofs and must not be upgraded by this model.
