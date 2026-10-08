# Phase0 Rust fixture oracle — partial migration under #71

This is a non-default **source candidate**, not the implemented universal
FREE ENERGY dispatcher, a release, or runtime containment authority. It replaces
**one** of the eight Python fixture-checker families semantically: containment
specification v3 (17 decisions, 5 recovery cases, 13 traces = 35 cases).
The original Python checker and JSON fixture remain untouched until all
families have proven executable parity and the migration can integrate safely.

## Run from the repository root

With a Rust/Cargo toolchain installed:

    cargo test --manifest-path Phase0/rust-fixtures/Cargo.toml
    cargo run --manifest-path Phase0/rust-fixtures/Cargo.toml -- Phase0/fixtures/containment-spec3.json

The CLI exits nonzero on an invalid schema/type, absent fixture, repeated ID,
missing required semantic input or disagreement between its independently
computed verdict and the fixture's expected outcome. Unit tests explicitly
mutate authority, expectations, dependencies, types and identities.

## Admission limits

- The directly required serde and serde_json versions are exact in Cargo.toml,
  but the transitive dependency lockfile is NOT YET committed. Generate,
  inspect and commit Cargo.lock, review dependency provenance and licenses, and
  run locked/offline builds before any integration approval.
- Execute cargo fmt, clippy, locked tests and actual differential tests against
  the unchanged historical Python runner. Source-only review is NOT a PASS.
- Other seven fixture families, canonical JSON/SHA-256 identity, and Git
  merge-base executable proofs remain OUTSTANDING in issue #71.
- The current preview remains on phase0/public-v0. Do not change main,
  publish this prototype, or remove Python based on this one-family patch.
