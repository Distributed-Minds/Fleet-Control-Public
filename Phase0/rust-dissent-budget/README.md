# Issue #17 — offline dissent amendment budget oracle

This package is a dependency-free, deterministic Rust **synthetic model**, not a
publisher, authority gate, production CAS implementation or permission to keep a
revoked principal operating. It only models the bounded slice accepted by the
independent exact-spec-3 PLAN (#6102132422) and ADVERSARIAL (#6102182426)
readiness records on public issue #17.

The ledger keeps one immutable principal-generation, shutdown decision, cutoff
and dissent lineage; exact CAS observation generations serialize reservations;
pending and outcome-unknown reservations consume finite capacity; identical
replay reuses the old receipt; conflicting payloads fail closed; uncertain
materiality is not assumed new; only a mock independent grant extends capacity.
Reclamation requires BOTH mock no-effect proof and mock late-worker fencing,
then retains an immutable tombstone. Every successful state transition checks
capacity conservation and unique simulated receipts. The principal authority
marker and cutoff fence have no mutation path.

EffectBasis booleans, grant approval and ReclaimProof are deliberately SYNTHETIC.
They are not evidence of actual human authority, semantic equivalence,
cryptographic issuer provenance, durable external fencing, provider readback
or disclosure permission. A production adapter MUST establish those through
independent trusted mechanisms and cannot reuse the boolean fixture inputs.

Commands from this directory (requires a real Rust toolchain):

    cargo fmt --all -- --check
    cargo test --locked --offline
    cargo clippy --all-targets --locked --offline -- -D warnings

Tests cover last-slot CAS collision, replay after acknowledgement loss,
crash/pending capacity, canonical semantic duplicates, stale policy and lineage,
serial successor generations, reclaim-fenced late worker, capability/channel
drift, immutable committed receipt, duplicate extension, unsafe reclaim,
duplicate external receipt, incarnation drift, and 729 short interleavings.

**Review gate:** This is an unmerged non-default-branch proposal targeting
phase0/public-v0. Native Rust, hosted CI, formatter, signing and all provider
effects need their own exact-head evidence before any completion claim.
No npm, third-party dependencies, credentials, public private-repo content,
real dissent emissions, scheduler edits or default-branch integration.

Issue: https://github.com/Distributed-Minds/Fleet-Control-Public/issues/17
