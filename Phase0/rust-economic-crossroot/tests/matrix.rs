//! Deterministic interleaving/credit controls for #19's offline fixture.
//! These checks have no marketplace, money, identity, or provider side effects.
use free_energy_economic_crossroot::{Denial, Ledger, Request, ResultKind, State};

fn attempt(id: &str, root: &str, units: u64, expected: u64) -> Request {
    Request {
        id: id.to_owned(),
        root: root.to_owned(),
        root_generation: 1,
        topology_generation: 1,
        expected_ledger_generation: expected,
        units,
        authority_current: true,
        valuation_current: true,
        terms_current: true,
    }
}

#[test]
fn every_bounded_two_root_order_rejects_stale_cas_and_overspend() {
    let amounts = [1, 7, 25, 50, 80, 100];
    for &left in &amounts {
        for &right in &amounts {
            for swap in [false, true] {
                let mut ledger =
                    Ledger::new("shared-domain", 100, 1, &[("root-a", 1), ("root-b", 1)]);
                let (first_root, second_root) = if swap {
                    ("root-b", "root-a")
                } else {
                    ("root-a", "root-b")
                };
                let first = attempt("first", first_root, left, 1);
                assert_eq!(ledger.reserve(first.clone()), ResultKind::Accepted);
                assert_eq!(
                    ledger.reserve(attempt("second", second_root, right, 1)),
                    ResultKind::Denied(Denial::StaleLedger)
                );
                let latest = attempt("second", second_root, right, ledger.generation());
                let expected = if left + right <= 100 {
                    ResultKind::Accepted
                } else {
                    ResultKind::Denied(Denial::NoCapacity)
                };
                assert_eq!(ledger.reserve(latest), expected);
                assert_eq!(ledger.reserve(first), ResultKind::Identical(State::Pending));
                assert!(ledger.charged() <= 100);
                assert_eq!(ledger.debt(), 0);
                assert_eq!(ledger.provider_effects(), 0);
            }
        }
    }
}

#[test]
fn refund_invalidation_matrix_preserves_accepted_exposure_and_debt() {
    for first in [1, 20, 60, 80, 100] {
        for second in [1, 20, 60, 80, 100] {
            let mut ledger = Ledger::new("shared-domain", 100, 1, &[("root-a", 1), ("root-b", 1)]);
            let original = attempt("first", "root-a", first, 1);
            assert_eq!(ledger.reserve(original.clone()), ResultKind::Accepted);
            assert_eq!(
                ledger.observe_commit("first", ledger.generation(), "effect-a", true),
                ResultKind::Accepted
            );
            assert_eq!(
                ledger.refund("first", ledger.generation(), true),
                ResultKind::Accepted
            );
            assert_eq!(
                ledger.reserve(attempt("second", "root-b", second, ledger.generation())),
                ResultKind::Accepted
            );
            assert_eq!(
                ledger.invalidate_refund("first", ledger.generation(), true),
                ResultKind::Accepted
            );
            assert_eq!(ledger.charged(), second);
            assert_eq!(ledger.debt(), first);
            assert_eq!(ledger.headroom(), 100_u64.saturating_sub(first + second));
            assert_eq!(
                ledger.reserve(original),
                ResultKind::Identical(State::Debt("effect-a".to_owned()))
            );
            if first + second >= 100 {
                assert_eq!(
                    ledger.reserve(attempt("third", "root-a", 1, ledger.generation())),
                    ResultKind::Denied(Denial::NoCapacity)
                );
            }
            assert_eq!(ledger.provider_effects(), 0);
        }
    }
}

#[test]
fn malformed_and_zero_size_attempts_never_create_receipts() {
    let mut ledger = Ledger::new("shared", 100, 1, &[("root-a", 1)]);
    for invalid in ["", "with space", "line\nbreak", "bad/slash"] {
        assert_eq!(
            ledger.reserve(attempt(invalid, "root-a", 20, 1)),
            ResultKind::Denied(Denial::Invalid)
        );
    }
    assert_eq!(
        ledger.reserve(attempt("zero", "root-a", 0, 1)),
        ResultKind::Denied(Denial::Invalid)
    );
    assert_eq!(ledger.generation(), 1);
    assert_eq!(ledger.charged(), 0);
    assert_eq!(ledger.provider_effects(), 0);
}
