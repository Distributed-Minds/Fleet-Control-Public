# Offline economic terms commit oracle (public #19, spec 2)

This **synthetic-only** Rust crate exercises one dangerous economic/paid-human
admission seam: a correct client-side offer precheck can be invalidated before
provider commitment. The modeled adapter accepts only an **atomic provider-side
conditional match on every material accepted term**, including the offer and
provider incarnations, not merely price. A missing real provider capability
would block; a changed quote would not create a simulated acceptance.

No live provider, payments, worker dispatch, credentials, trusted human mandate,
independent authority issuer, actual provider-side CAS, real #27 shared ledger,
#16 cutoff, #13 quote verifier, settlement or publication is implemented. The
`Evidence::Synthetic` label is an injected test assertion, never a production
proof. `FakeProvider::simulated_accepted_count()` counts only in-memory fake
acceptances. `SimulationJournal` is neither durable nor distributed; its
single-process behavior is not a production fencing or replay guarantee.

Run in this directory with pinned Rust 1.85.1:

```sh
cargo +1.85.1 fmt --all --check
cargo +1.85.1 test --locked --offline
cargo +1.85.1 clippy --all-targets --locked --offline -- -D warnings
```

The associated PR-only CI workflow uses no JavaScript or Node checkout actions.
The tests cover precheck and effect-time drifts in 12 material term fields,
unknown conditional capability, exact authority/valuation/source receipt
currentness, lost acknowledgement, immutable idempotency receipts, capacity
exhaustion and positive simulation-only compatibility. This is a **partial
regression slice**, not full coverage of issue #19's 32 mandatory fixtures.

No successful test here authorizes purchase, contracting, paid-human delegation,
production use, human DCO signoff, or merging to any default branch.
