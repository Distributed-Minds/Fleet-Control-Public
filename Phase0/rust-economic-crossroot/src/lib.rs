//! Issue #19 spec 2: offline synthetic shared economic-capacity fixture.
//! No provider calls, payments, jobs, cryptographic authority, or actual FX.
use std::collections::BTreeMap;

pub const MAX_RECEIPTS: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub id: String,
    pub component: String,
    pub intent_digest: String,
    pub valuation_basis: String,
    pub terms_basis: String,
    pub root: String,
    pub root_generation: u64,
    pub topology_generation: u64,
    pub expected_ledger_generation: u64,
    pub units: u64,
    pub authority_current: bool,
    pub valuation_current: bool,
    pub terms_current: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Pending,
    Committed(String),
    Refunded(String),
    Debt(String),
    Reclaimed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    Invalid,
    WrongComponent,
    UnknownRoot,
    StaleRoot,
    StaleTopology,
    StaleLedger,
    MissingAuthority,
    UnknownValuation,
    ChangedTerms,
    NoCapacity,
    HistoryFull,
    Conflict,
    Unproven,
    Fenced,
    WrongState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResultKind {
    Accepted,
    Identical(State),
    Denied(Denial),
}

#[derive(Clone)]
struct Receipt {
    request: Request,
    state: State,
}

/// Separately selected current roots are synthetic constructor inputs.
/// A real system MUST consume trusted authority from #10/#16/#27, never here.
pub struct Ledger {
    component: String,
    capacity: u64,
    roots: BTreeMap<String, u64>,
    topology: u64,
    generation: u64,
    charged: u64,
    debt: u64,
    receipts: BTreeMap<String, Receipt>,
}

fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

impl Ledger {
    pub fn new(component: &str, capacity: u64, topology: u64, roots: &[(&str, u64)]) -> Self {
        assert!(token(component) && topology > 0);
        let mut trusted = BTreeMap::new();
        for &(root, generation) in roots {
            assert!(token(root) && generation > 0);
            assert!(trusted.insert(root.to_owned(), generation).is_none());
        }
        Self {
            component: component.to_owned(),
            capacity,
            roots: trusted,
            topology,
            generation: 1,
            charged: 0,
            debt: 0,
            receipts: BTreeMap::new(),
        }
    }

    pub fn component(&self) -> &str {
        &self.component
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn charged(&self) -> u64 {
        self.charged
    }

    pub fn debt(&self) -> u64 {
        self.debt
    }

    pub fn headroom(&self) -> u64 {
        self.capacity
            .saturating_sub(self.charged.saturating_add(self.debt))
    }

    pub fn provider_effects(&self) -> usize {
        0
    }

    pub fn receipt(&self, id: &str) -> Option<State> {
        self.receipts.get(id).map(|record| record.state.clone())
    }

    fn next(&self, expected: u64) -> Option<u64> {
        if self.generation != expected {
            return None;
        }
        expected.checked_add(1)
    }

    /// One exact immutable attempt consumes capacity once. A later identical
    /// retry reads the original receipt even when its CAS head is historical.
    pub fn reserve(&mut self, r: Request) -> ResultKind {
        if !token(&r.id)
            || !token(&r.component)
            || !token(&r.intent_digest)
            || !token(&r.valuation_basis)
            || !token(&r.terms_basis)
            || !token(&r.root)
            || r.units == 0
        {
            return ResultKind::Denied(Denial::Invalid);
        }
        if let Some(old) = self.receipts.get(&r.id) {
            return if old.request == r {
                ResultKind::Identical(old.state.clone())
            } else {
                ResultKind::Denied(Denial::Conflict)
            };
        }
        if r.component != self.component {
            return ResultKind::Denied(Denial::WrongComponent);
        }
        if self.receipts.len() >= MAX_RECEIPTS {
            return ResultKind::Denied(Denial::HistoryFull);
        }
        let Some(current_root) = self.roots.get(&r.root) else {
            return ResultKind::Denied(Denial::UnknownRoot);
        };
        if *current_root != r.root_generation {
            return ResultKind::Denied(Denial::StaleRoot);
        }
        if r.topology_generation != self.topology {
            return ResultKind::Denied(Denial::StaleTopology);
        }
        if !r.authority_current {
            return ResultKind::Denied(Denial::MissingAuthority);
        }
        if !r.valuation_current {
            return ResultKind::Denied(Denial::UnknownValuation);
        }
        if !r.terms_current {
            return ResultKind::Denied(Denial::ChangedTerms);
        }
        let Some(next) = self.next(r.expected_ledger_generation) else {
            return ResultKind::Denied(Denial::StaleLedger);
        };
        let Some(charged) = self.charged.checked_add(r.units) else {
            return ResultKind::Denied(Denial::NoCapacity);
        };
        let Some(exposure) = charged.checked_add(self.debt) else {
            return ResultKind::Denied(Denial::NoCapacity);
        };
        if exposure > self.capacity {
            return ResultKind::Denied(Denial::NoCapacity);
        }
        self.charged = charged;
        self.generation = next;
        self.receipts.insert(
            r.id.clone(),
            Receipt {
                request: r,
                state: State::Pending,
            },
        );
        ResultKind::Accepted
    }

    /// Represents observation of an externally independently proved effect,
    /// not an effect itself. An unproved acknowledgement retains PENDING.
    pub fn observe_commit(
        &mut self,
        id: &str,
        expected: u64,
        effect: &str,
        proved: bool,
    ) -> ResultKind {
        if !proved || !token(effect) {
            return ResultKind::Denied(Denial::Unproven);
        }
        let Some(old) = self.receipts.get(id) else {
            return ResultKind::Denied(Denial::Invalid);
        };
        match &old.state {
            State::Committed(s) | State::Refunded(s) | State::Debt(s) if s == effect => {
                return ResultKind::Identical(old.state.clone());
            }
            State::Reclaimed => return ResultKind::Denied(Denial::Fenced),
            State::Pending => {}
            _ => return ResultKind::Denied(Denial::Conflict),
        }
        let Some(next) = self.next(expected) else {
            return ResultKind::Denied(Denial::StaleLedger);
        };
        self.receipts.get_mut(id).unwrap().state = State::Committed(effect.to_owned());
        self.generation = next;
        ResultKind::Accepted
    }

    /// Release a failed PENDING only with evidence no late effect is possible.
    pub fn reclaim(&mut self, id: &str, expected: u64, no_late_effect_fence: bool) -> ResultKind {
        if !no_late_effect_fence {
            return ResultKind::Denied(Denial::Unproven);
        }
        let Some(old) = self.receipts.get(id) else {
            return ResultKind::Denied(Denial::Invalid);
        };
        if old.state == State::Reclaimed {
            return ResultKind::Identical(State::Reclaimed);
        }
        if old.state != State::Pending {
            return ResultKind::Denied(Denial::WrongState);
        }
        let amount = old.request.units;
        let Some(next) = self.next(expected) else {
            return ResultKind::Denied(Denial::StaleLedger);
        };
        self.charged -= amount;
        self.receipts.get_mut(id).unwrap().state = State::Reclaimed;
        self.generation = next;
        ResultKind::Accepted
    }

    /// Only an authoritatively final refund can restore synthetic headroom.
    pub fn refund(&mut self, id: &str, expected: u64, finality_proved: bool) -> ResultKind {
        if !finality_proved {
            return ResultKind::Denied(Denial::Unproven);
        }
        let Some(old) = self.receipts.get(id) else {
            return ResultKind::Denied(Denial::Invalid);
        };
        let effect = match &old.state {
            State::Committed(s) => s.clone(),
            State::Refunded(_) => return ResultKind::Identical(old.state.clone()),
            _ => return ResultKind::Denied(Denial::WrongState),
        };
        let amount = old.request.units;
        let Some(next) = self.next(expected) else {
            return ResultKind::Denied(Denial::StaleLedger);
        };
        self.charged -= amount;
        self.receipts.get_mut(id).unwrap().state = State::Refunded(effect);
        self.generation = next;
        ResultKind::Accepted
    }

    /// Reopening an apparently final refund creates debt instead of hiding
    /// already admitted commitments that consumed its released headroom.
    pub fn invalidate_refund(&mut self, id: &str, expected: u64, proved: bool) -> ResultKind {
        if !proved {
            return ResultKind::Denied(Denial::Unproven);
        }
        let Some(old) = self.receipts.get(id) else {
            return ResultKind::Denied(Denial::Invalid);
        };
        let effect = match &old.state {
            State::Refunded(s) => s.clone(),
            State::Debt(_) => return ResultKind::Identical(old.state.clone()),
            _ => return ResultKind::Denied(Denial::WrongState),
        };
        let amount = old.request.units;
        let Some(next) = self.next(expected) else {
            return ResultKind::Denied(Denial::StaleLedger);
        };
        let Some(debt) = self.debt.checked_add(amount) else {
            return ResultKind::Denied(Denial::NoCapacity);
        };
        self.debt = debt;
        self.receipts.get_mut(id).unwrap().state = State::Debt(effect);
        self.generation = next;
        ResultKind::Accepted
    }

    /// Fixture-only trusted topology movement, not a real authority issuer.
    pub fn synthetic_move_topology(&mut self, topology: u64, roots: &[(&str, u64)]) {
        assert!(topology > self.topology);
        let mut new_roots = BTreeMap::new();
        for &(root, generation) in roots {
            assert!(token(root) && generation > 0);
            assert!(new_roots.insert(root.to_owned(), generation).is_none());
        }
        self.roots = new_roots;
        self.topology = topology;
        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared() -> Ledger {
        Ledger::new("shared-risk", 100, 1, &[("root-a", 1), ("root-b", 1)])
    }

    fn request(id: &str, root: &str, units: u64, expected: u64) -> Request {
        request_with_component("shared-risk", id, root, units, expected)
    }

    fn request_with_component(component: &str, id: &str, root: &str, units: u64, expected: u64) -> Request {
        Request {
            id: id.into(),
            component: component.into(),
            intent_digest: "intent-basis-v1".into(),
            valuation_basis: "valuation-v1".into(),
            terms_basis: "terms-v1".into(),
            root: root.into(),
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
    fn two_roots_may_not_oversubscribe_one_component() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("a", "root-a", 80, 1)),
            ResultKind::Accepted
        );
        assert_eq!(
            l.reserve(request("b", "root-b", 30, 2)),
            ResultKind::Denied(Denial::NoCapacity)
        );
        assert_eq!(l.headroom(), 20);
    }

    #[test]
    fn genuinely_disjoint_components_have_separate_headroom() {
        let mut a = Ledger::new("domain-a", 100, 1, &[("root-a", 1)]);
        let mut b = Ledger::new("domain-b", 100, 1, &[("root-b", 1)]);
        assert_eq!(
            a.reserve(request_with_component("domain-a", "a", "root-a", 95, 1)),
            ResultKind::Accepted
        );
        assert_eq!(
            b.reserve(request_with_component("domain-b", "b", "root-b", 95, 1)),
            ResultKind::Accepted
        );
        assert_ne!(a.component(), b.component());
    }

    #[test]
    fn stale_cas_loses_but_original_request_replays_idempotently() {
        let mut l = shared();
        let first = request("x", "root-a", 60, 1);
        assert_eq!(l.reserve(first.clone()), ResultKind::Accepted);
        assert_eq!(
            l.reserve(request("y", "root-b", 10, 1)),
            ResultKind::Denied(Denial::StaleLedger)
        );
        assert_eq!(l.reserve(first), ResultKind::Identical(State::Pending));
        assert_eq!(l.charged(), 60);
    }

    #[test]
    fn different_payload_same_id_is_never_an_identical_replay() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "root-a", 30, 1)),
            ResultKind::Accepted
        );
        assert_eq!(
            l.reserve(request("x", "root-a", 31, 1)),
            ResultKind::Denied(Denial::Conflict)
        );
    }

    #[test]
    fn untrusted_root_authority_rate_and_terms_are_independent_blocks() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "unknown", 10, 1)),
            ResultKind::Denied(Denial::UnknownRoot)
        );
        let mut r = request("x", "root-a", 10, 1);
        r.authority_current = false;
        assert_eq!(l.reserve(r), ResultKind::Denied(Denial::MissingAuthority));
        let mut r = request("x", "root-a", 10, 1);
        r.valuation_current = false;
        assert_eq!(l.reserve(r), ResultKind::Denied(Denial::UnknownValuation));
        let mut r = request("x", "root-a", 10, 1);
        r.terms_current = false;
        assert_eq!(l.reserve(r), ResultKind::Denied(Denial::ChangedTerms));
        assert_eq!(l.charged(), 0);
    }

    #[test]
    fn topology_and_root_generation_move_fence_old_admission() {
        let mut l = shared();
        let original = request("a", "root-a", 10, 1);
        assert_eq!(l.reserve(original.clone()), ResultKind::Accepted);
        l.synthetic_move_topology(2, &[("root-a", 2), ("root-b", 1)]);
        assert_eq!(l.reserve(original), ResultKind::Identical(State::Pending));
        assert_eq!(
            l.reserve(request("b", "root-b", 10, 3)),
            ResultKind::Denied(Denial::StaleTopology)
        );
        let mut r = request("b", "root-a", 10, 3);
        r.topology_generation = 2;
        assert_eq!(l.reserve(r), ResultKind::Denied(Denial::StaleRoot));
    }

    #[test]
    fn lost_ack_does_not_spend_twice_or_change_effect_identity() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "root-a", 50, 1)),
            ResultKind::Accepted
        );
        assert_eq!(
            l.observe_commit("x", 2, "effect-a", false),
            ResultKind::Denied(Denial::Unproven)
        );
        assert_eq!(l.receipt("x"), Some(State::Pending));
        assert_eq!(
            l.observe_commit("x", 2, "effect-a", true),
            ResultKind::Accepted
        );
        assert_eq!(
            l.observe_commit("x", 2, "effect-a", true),
            ResultKind::Identical(State::Committed("effect-a".into()))
        );
        assert_eq!(
            l.observe_commit("x", 3, "effect-b", true),
            ResultKind::Denied(Denial::Conflict)
        );
        assert_eq!(l.charged(), 50);
    }

    #[test]
    fn reclaim_needs_fence_and_late_effect_stays_rejected() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "root-a", 95, 1)),
            ResultKind::Accepted
        );
        assert_eq!(
            l.reclaim("x", 2, false),
            ResultKind::Denied(Denial::Unproven)
        );
        assert_eq!(l.reclaim("x", 2, true), ResultKind::Accepted);
        assert_eq!(
            l.observe_commit("x", 3, "late", true),
            ResultKind::Denied(Denial::Fenced)
        );
        assert_eq!(
            l.reserve(request("y", "root-b", 100, 3)),
            ResultKind::Accepted
        );
    }

    #[test]
    fn uncertain_refund_retains_charge() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "root-a", 80, 1)),
            ResultKind::Accepted
        );
        assert_eq!(l.observe_commit("x", 2, "fx", true), ResultKind::Accepted);
        assert_eq!(
            l.refund("x", 3, false),
            ResultKind::Denied(Denial::Unproven)
        );
        assert_eq!(l.headroom(), 20);
        assert_eq!(l.refund("x", 3, true), ResultKind::Accepted);
        assert_eq!(l.headroom(), 100);
    }

    #[test]
    fn invalidated_refund_preserves_debt_and_blocks_new_admission() {
        let mut l = shared();
        assert_eq!(
            l.reserve(request("x", "root-a", 80, 1)),
            ResultKind::Accepted
        );
        assert_eq!(l.observe_commit("x", 2, "fx", true), ResultKind::Accepted);
        assert_eq!(l.refund("x", 3, true), ResultKind::Accepted);
        assert_eq!(
            l.reserve(request("y", "root-b", 40, 4)),
            ResultKind::Accepted
        );
        assert_eq!(l.invalidate_refund("x", 5, true), ResultKind::Accepted);
        assert_eq!(l.debt(), 80);
        assert_eq!(l.headroom(), 0);
        assert_eq!(
            l.reserve(request("z", "root-a", 1, 6)),
            ResultKind::Denied(Denial::NoCapacity)
        );
        assert_eq!(
            l.invalidate_refund("x", 5, true),
            ResultKind::Identical(State::Debt("fx".into()))
        );
        assert_eq!(l.provider_effects(), 0);
    }

    #[test]
    fn canonical_component_and_basis_digests_are_part_of_immutable_replay() {
        let mut ledger = shared();
        let original = request("same-operation", "root-a", 20, 1);
        assert_eq!(ledger.reserve(original.clone()), ResultKind::Accepted);
        for which in 0..4 {
            let mut changed = original.clone();
            match which {
                0 => changed.component = "other-domain".into(),
                1 => changed.intent_digest = "different-intent".into(),
                2 => changed.valuation_basis = "different-valuation".into(),
                3 => changed.terms_basis = "different-terms".into(),
                _ => unreachable!(),
            }
            assert_eq!(ledger.reserve(changed), ResultKind::Denied(Denial::Conflict));
        }
        let mut fresh = request("fresh", "root-b", 10, 2);
        fresh.component = "other-domain".into();
        assert_eq!(ledger.reserve(fresh), ResultKind::Denied(Denial::WrongComponent));
        assert_eq!(ledger.reserve(original), ResultKind::Identical(State::Pending));
        assert_eq!(ledger.charged(), 20);
    }

    #[test]
    fn full_receipt_history_preserves_old_replay_and_blocks_new_ids() {
        let mut l = Ledger::new("bounded", 2000, 1, &[("root-a", 1)]);
        for i in 0..MAX_RECEIPTS {
            assert_eq!(
                l.reserve(request_with_component("bounded", &format!("id-{i}"), "root-a", 1, l.generation())),
                ResultKind::Accepted
            );
        }
        assert_eq!(
            l.reserve(request_with_component("bounded", "new", "root-a", 1, l.generation())),
            ResultKind::Denied(Denial::HistoryFull)
        );
        assert_eq!(
            l.reserve(request_with_component("bounded", "id-0", "root-a", 1, 1)),
            ResultKind::Identical(State::Pending)
        );
    }
}
