# Selector-input authority reducer (issue #49, Phase0 spec 6)

**OFFLINE SYNTHETIC MODEL ONLY.** This crate is not a production evidence registry, signature verifier, independently trusted provenance source, cross-clock calibration, confidence calculator, source-selection authority or worker/provider effect gate. Its `jointly_fenced` flag is synthetic fixture input, not a trusted serialization primitive.

It enforces a closed complete model/source set, exact incarnations and generations, source validity and high-water completeness, no provenance cycle back to the scored metric, one coherent fixture cut, all required model/source rank pairs, deterministic unanimity and one unique minimal rank. Conflicts, stale evidence, malformed units, missing sources, rank overflow and ambiguous outcomes deny selection. Canonical basis identity is the full length-prefixed semantic envelope, **not** a digest or proof of trust. Every positive selection explicitly reports `confidence_gain: false`.

An in-memory journal rejects changed intent and effect-time basis movement and reconciles identical previously committed operations after lost acknowledgements. It is not crash-durable, concurrent, cryptographic or an authoritative effect service.

The Rust unit tests exercise cases S6-01 through S6-10 and additional lineage, tie, malformed input, replay, duplicate-source and change-of-incarnation controls. The earlier forty spec-5 fixture classes, real provider/source adapters, historical calibration, cohort/censoring and publication semantics are **not implemented**. Consequently issue #49 remains open and the PR is a partial draft.

From `Phase0/rust-selector-inputs` run:

```sh
cargo fmt --all --check
cargo test --locked --offline
cargo clippy --all-targets --locked --offline -- -D warnings
```

No npm or maintained Python dependencies. The public Constitution, exact-spec gates, two-slot ownership protocol and human-only default-branch merge restriction continue to apply.
