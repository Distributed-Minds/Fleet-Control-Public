# ContributionAdmissionV1 — synthetic offline Rust model

Issue: [#159](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/159), Phase0 specification v3. Exact-spec PLAN READY: [6098025604](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/159#issuecomment-6098025604); independent adversarial READY: [6098162040](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/159#issuecomment-6098162040).

This is a deliberately **simulation-only, nonpublishing** standard-library-only Rust model. It does not capture human assent, sign commits, validate actual DCO declarations, verify GitHub identities, certify legal rights, use credentials, access providers, or implement the #70/#91 authority boundary. `Claim::SyntheticFixture` is explicitly synthetic: it **never proves real consent**. The strongest positive decision is `ReviewableInSimulation`, never provider-write authority or `PUBLISHED_VERIFIED`.

The reducer independently checks versioned resource identity, terms, human act, DCO evidence, rights by class, current task authority, exact reviewed output, history clearance, transformed-output review and context attribution. Unsafe ambiguity yields typed `Blocked`, `HumanReviewRequired`, `ReconcileOriginalAttempt` or `Unknown`. The in-memory `SimulationJournal` rejects same-operation changed-content replays and reports zero real provider effects. **Persistence, crash durability, legal actor verification, concurrency across processes, post-effect reconciliation and production provider adapter remain unimplemented.**

The deterministic v1 canonical envelope is length-prefixed UTF-8 plus fixed-width numeric/evidence fields, hashed using a local offline SHA-256 implementation verified against published empty/abc test vectors. That digest is a content identifier, not a signature or authority proof. Any format change needs a new schema and migration review.

Compiled test coverage: A01/A02 and D01–D16 synthetic positive/negative controls, near-positive single-predicate rejection, unknown-schema/resource rejection, canonical framing and SHA-256 vectors.

Verification from this crate directory:

```bash
cargo +1.85.1 fmt --all --check
cargo +1.85.1 test --locked --offline
cargo +1.85.1 clippy --all-targets --locked --offline -- -D warnings
```

This PR must stay non-default and draft until exact-head CI, review, full issue acceptance and human policy boundaries are satisfied. No DCO `Signed-off-by` is fabricated.
