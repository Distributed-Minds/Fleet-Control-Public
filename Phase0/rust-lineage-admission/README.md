# Rust synthetic lineage admission (#11, spec 6)

This dependency-free crate is a deterministic OFFLINE model, not an actual
trusted registry, provider authorization source, durable transaction system,
scheduled GitHub capability probe or cleanup utility. All inputs (including
issuer selection and the effect-time cut) are caller-provided simulation
hypotheses. A positive verdict is always named SimulationOnly. There are
no provider effects or filesystem IO.

The in-process compare-and-swap model captures exact predecessor/generation,
successor incarnation, one-winner competition, immutable operation replay,
inherited obligations, namespace scope and independent resource/action/recovery
checks at an effect cut. Compiled tests exercise LA6-01 through LA6-10 and
replay/rollback variations. A real system still requires an independent
nonforkable registry, provider-backed currentness, exact resource incarnation,
current action fencing and scheduled-context capability evidence. Without
that evidence production must fail closed to LINEAGE_UNKNOWN.

Run cargo fmt --all --check, cargo test --locked --offline and
cargo clippy --all-targets --locked --offline -- -D warnings.

This partial implementation is NOT issue #11 acceptance or a default-branch
merge. It is gated only as a bounded offline Rust simulation.
