# Provider-effect M1 offline feasibility model

Issue: public #91, Phase0 specification v3. This is an intentionally partial, non-operational Rust work package targeting the non-default preview only. It is not a GitHub mutation broker, server, publisher, provider adapter or release.

The strict TSV v1 capability matrix records effect family and transport separately, distinguishing positive acknowledgements, lost-acknowledgement attribution, automatic replay, and target CAS. Every listed actual GitHub path is UNKNOWN or UNSUPPORTED absent transport-specific evidence. A manifest row is never authorization.

The std-only typed Rust model shows the essential lost-acknowledgement safety invariant: a complete listing containing one identical same-credential artifact cannot establish that this attempt created it. Only an internal, exact-attempt-bound receipt (no public constructor exists yet) can demonstrate attribution in the offline model; a future real adapter must implement and independently verify its trust boundary. Unknown results become terminal MANUAL_HOLD for automatic replay, even with a matching remote object. Unit tests cover specific C13–C17 denial/positive subsets, and the compiled manifest tests cover the C18 support split; this is NOT full C1–C18 conformance or persisted crash/restart.

Tests:
    cargo fmt --all --check
    cargo test --locked --offline
    cargo clippy --all-targets --locked --offline -- -D warnings

Remaining M1 acceptance: real durable outbox and typed receipt storage/restart, other C1–C12 fault-injection transitions, externally fenced transport-specific proof adapters, support-matrix evidence and completeness, semantics for all operation families, independent CI and review. No producer of TrustedAttemptReceipt is exported until a verified adapter exists. No production provider side effects are performed.
