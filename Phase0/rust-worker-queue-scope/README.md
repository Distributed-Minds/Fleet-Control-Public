# #280 — conservative scope admission (simulation only)

**Issue/spec:** [#280](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/280), Phase0 spec **2**. Two distinct-run readiness records are [PLAN READY](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/280#issuecomment-6097898991) and [ADVERSARIAL READY](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/280#issuecomment-6097962736), both scoped to a simulation-only Rust queue MVP.

This std-only Rust crate implements one **bounded, executable component** of the required ScopeConflictBasis: strict exact canonical branch refs, issue/PR conversation resource coalescing, complete bounded multi-resource footprints, conservative conflict detection, stale-basis refusal, deterministic alternate arrival order, and recovery-held scope preservation. It does **not** allocate actual worker assignments. Hash strings supplied by task clients are not treated as collision evidence. Path/alias/hierarchical scope classes are rejected rather than incorrectly classified as disjoint; this implementation's admitted exact-key subset is deliberately narrower than the eventual system.

## Offline verification

- `cargo +1.85.1 fmt --all --check`
- `cargo +1.85.1 test --locked --offline`
- `cargo +1.85.1 clippy --all-targets --locked --offline -- -D warnings`
- `cargo +1.85.1 run --locked --offline --bin scope_demo`

The pinned std-only Cargo package has a committed lockfile and no external dependencies. Current hosted tests cover **19 Rust cases** (scope, fixture-authenticated poll/replay and synthetic delivery); they validate simulation decisions, not a real SQLite concurrent transaction or provider mutation.

### Fixture-only ACK and result submission

`SimulatedReplay` includes bounded `acknowledge` and `submit_result` decisions keyed to the immutable, previously issued synthetic poll receipt: fixture-injected principal identity/generation, request ID, exact task and assignment generation, and current exact scope basis. Duplicate identical events reconcile without extra state; wrong principal, moved basis, missing ACK, changed result digest, revoked generation and recovery-held assignment fail closed. This is **not** an authenticated HTTP transport or a provider-verified result: a digest is merely caller-supplied fixture data, not semantic truth. No durable journal, external signature, result acceptance or GitHub mutation authority is inferred.

## Explicit incomplete acceptance

- **NO SQLite / durable state:** no transactional BEGIN IMMEDIATE, WAL, outbox, restart, crash recovery or schema migration. Production work requires an independently vetted and pinned SQLite Rust binding/Cargo.lock, dependency/license audit and compiled crash/restart tests.
- **NO authenticated worker API:** no real authentication adapter, persistent identity/replay ledger, network ACK/result endpoint, result provenance or trusted acceptance, authoritative generation fencing, worker enrollment, concurrent service or provider token access.
- **NO trusted provider resource registry:** this model works only with fixtures claiming an exact current basis. A future server must independently establish canonical identities, alias/ancestry semantics and real effect-time #43/#50/#10 fencing.
- **NO live release:** this model neither grants provider mutation capability nor satisfies issue #280, parent #70, or M1–M4 product acceptance. A green test must not be interpreted as permission to schedule a real worker.

Current Phase0 coordination and human-only default-branch integration remain unchanged.
