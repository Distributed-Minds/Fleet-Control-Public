# Offline synthetic cross-root economic capacity model

P0 #19 spec 2, bounded synthetic Rust-only model. No provider adapter, money movement, human-hiring endpoint, production trust root, durable ledger, real valuation or real external effects. The roots and proof flags are externally supplied fixtures; they do not grant actual spending authority.

Exercises one shared risk component across separate authority roots, exact CAS generation, immutable replay receipts, bounded history, outcome-uncertain reservations, no-late-effect reclamation fencing, conservative refund finality and post-refund reconciliation debt. Zero actual provider effects.

Independent seam from the concurrent rust-economic-terms package. This crate does not implement merchant-offer comparison.

Checks: cargo +1.85.1 fmt --all --check; cargo +1.85.1 test --locked --offline; cargo +1.85.1 clippy --all-targets --locked --offline -- -D warnings.

Unimplemented prerequisites: #10/#13/#16/#27 trusted authority and atomic effect fences, durable storage, human legal/DCO consent, provider authentication, real FX and physical commitments. Non-default draft PR only; human owns any integration.
