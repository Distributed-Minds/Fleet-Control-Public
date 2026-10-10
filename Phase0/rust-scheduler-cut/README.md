# Scheduler admission cut (issue #50, spec 5)

This is an **offline synthetic Rust model**, not a scheduler, distributed lock,
production admission oracle, receipt database, identity authority or permission
to start any worker. The injected `trusted_fence` and `authority_issued`
booleans are **fixtures only** and MUST NOT be used as production proof.

The model binds an admitted output to an exact scheduler source incarnation,
policy source incarnation/generation, capacity basis, assignment and immutable
operation ID. It rejects torn current-state observations, unsupported bridges,
untrusted/incomparable transitions, changed capacity without a bridge,
effect-time movement, malformed identities and second starts. Retrying the
same committed operation returns the original historical cut rather than
granting a new start under a more favorable current policy.

Deterministic compiled tests exercise CUT01–CUT11, CUT15–CUT18 and identity
negative controls. **CUT12–CUT14 full successor-continuation semantics are
not implemented.** There is no durable crash recovery, production concurrent
CAS, cryptographic current-selection, domain authority, external effects or
provider API. Tests only prove the bounded deterministic simulation.

```sh
cd Phase0/rust-scheduler-cut
cargo +1.85.1 fmt --all --check
cargo +1.85.1 test --locked --offline
cargo +1.85.1 clippy --all-targets --locked --offline -- -D warnings
```

Read the [current canonical issue #50](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/50)
and independently gated scope before extending. Keep Rust only and no npm.
Never treat green model tests as authority to mutate or deploy.
