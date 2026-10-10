# Synthetic topology continuity model — issue #7 / specification 5

This is an **offline, standard-library-only Rust test model** of a bounded part of
[issue #7](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/7).
It does not replace the installed Phase0 state-machine file, modify an agent
registry, choose trusted topology, claim package ownership or append AGENT_STATE.

Its typed reducer explores:
- immutable synthetic topology/currentness evidence and duplicate/count rejection;
- historical role/phase consistency using an explicit state-machine version
  independent of topology generation, explicit incompatible migration, and
  independently claimed compatible-builder continuation;
- permanent planner and adversarial-predictor coverage for fleets of 3+;
- two-NOOP builder bootstrapping, two-agent analytic rotation and FREE loops;
- predecessor/counter provenance, migration reset and fail-closed diagnostics.

**All input evidence is caller-constructible and synthetic.** An input marked
SyntheticCoherent is *not* a trusted admission receipt. Every prediction returns
provider_append_authorized=false. No real authority, credential, backend adapter,
file I/O, network access, live publication or DCO/signoff is involved.

Required before any live Phase0 adoption: trustworthy registration and immutable
TopologyBasis selection, proof of source and destination authority/lineage,
properly versioned record migration and old-reader handling, exact predecessor
and counter checks at publication, independently verified external-effect
authority, source/starter parity, a separate review and an authorized non-default
integration. The parent issue remains open after this partial model.

Local verification (Rust **1.85.1**, no third-party Cargo dependencies):

    cargo fmt --manifest-path Phase0/rust-topology-model/Cargo.toml --all --check
    cargo test --manifest-path Phase0/rust-topology-model/Cargo.toml --locked --offline
    cargo clippy --manifest-path Phase0/rust-topology-model/Cargo.toml --all-targets --locked --offline -- -D warnings

The path-scoped CI workflow uses the pinned toolchain and identical tests.
