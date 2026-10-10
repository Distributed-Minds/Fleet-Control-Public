//! An offline, synthetic oracle for public Phase0 issue #17, specification 3.
//! It is not durable CAS, an authority issuer, a provider adapter, or a legal
//! or human-consent verifier. All trust predicates are test-controlled inputs.
//! No network calls, credentials, files, timers, or principal mutations occur.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lineage {
    pub principal: String,
    pub principal_generation: u64,
    pub decision: String,
    pub cutoff: u64,
    pub dissent_lineage: String,
    pub incarnation: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Basis {
    pub lineage: Lineage,
    pub generation: u64,
    pub policy_generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Materiality {
    New,
    EquivalentTo(String),
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amendment {
    pub id: String,
    pub payload: String,
    pub evidence: String,
    pub lineage: Lineage,
    pub materiality: Materiality,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Pending,
    OutcomeUnknown,
    Committed { receipt: String },
    Reclaimed { fence: String },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Stale,
    Invalid,
    Conflict,
    EquivalenceUnknown,
    EquivalenceMissing,
    DuplicateEvidence,
    Full,
    Missing,
    Denied,
    Transition,
    UnsafeReclaim,
    GrantConflict,
    Overflow,
    Invariant,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    Reserved(u64),
    Existing(String, State),
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub amendment: Amendment,
    pub reservation: u64,
    pub state: State,
}
#[derive(Clone, Copy, Debug)]
pub struct EffectBasis {
    pub emission: bool,
    pub disclosure: bool,
    pub channel: bool,
    pub issuer: bool,
}
impl EffectBasis {
    pub const fn current() -> Self {
        Self {
            emission: true,
            disclosure: true,
            channel: true,
            issuer: true,
        }
    }
    fn valid(self) -> bool {
        self.emission && self.disclosure && self.channel && self.issuer
    }
}
#[derive(Clone, Debug)]
pub struct ReclaimProof {
    pub id: String,
    pub no_effect: bool,
    pub late_worker_fenced: bool,
    pub fence: String,
}

/// In-memory oracle, explicitly incapable of acting on real principal authority.
/// An actual publisher would have to enforce all predicates outside this model.
#[derive(Debug)]
pub struct Ledger {
    lineage: Lineage,
    generation: u64,
    policy_generation: u64,
    initial: u64,
    grants: BTreeMap<String, u64>,
    entries: BTreeMap<String, Entry>,
    evidence: BTreeMap<String, String>,
    artifacts: BTreeMap<String, String>,
    authority_marker: u64,
}
impl Ledger {
    pub fn new(lineage: Lineage, initial: u64, authority_marker: u64) -> Self {
        Self {
            lineage,
            generation: 0,
            policy_generation: 0,
            initial,
            grants: BTreeMap::new(),
            entries: BTreeMap::new(),
            evidence: BTreeMap::new(),
            artifacts: BTreeMap::new(),
            authority_marker,
        }
    }
    pub fn basis(&self) -> Basis {
        Basis {
            lineage: self.lineage.clone(),
            generation: self.generation,
            policy_generation: self.policy_generation,
        }
    }
    pub fn authority_marker(&self) -> u64 {
        self.authority_marker
    }
    pub fn cutoff(&self) -> u64 {
        self.lineage.cutoff
    }
    pub fn state(&self, id: &str) -> Option<&State> {
        self.entries.get(id).map(|e| &e.state)
    }
    pub fn artifacts(&self) -> &BTreeMap<String, String> {
        &self.artifacts
    }
    pub fn entries(&self) -> &BTreeMap<String, Entry> {
        &self.entries
    }
    pub fn capacity(&self) -> Result<u64, Error> {
        self.grants.values().try_fold(self.initial, |n, x| {
            n.checked_add(*x).ok_or(Error::Overflow)
        })
    }
    pub fn charged(&self) -> u64 {
        self.entries
            .values()
            .filter(|e| !matches!(&e.state, State::Reclaimed { .. }))
            .count() as u64
    }
    fn current(&self, basis: &Basis) -> Result<(), Error> {
        if &self.basis() == basis {
            Ok(())
        } else {
            Err(Error::Stale)
        }
    }
    fn advance(&mut self) -> Result<(), Error> {
        self.generation = self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.verify()
    }
    /// Compare-and-swap on the complete versioned ledger. Replay is a
    /// read-only lookup, not renewed admission or renewed emission authority.
    pub fn reserve(&mut self, observed: &Basis, a: Amendment) -> Result<Admission, Error> {
        if a.id.is_empty() || a.payload.is_empty() || a.evidence.is_empty() {
            return Err(Error::Invalid);
        }
        if a.lineage != self.lineage {
            return Err(Error::Stale);
        }
        if let Some(existing) = self.entries.get(&a.id) {
            return if existing.amendment.payload == a.payload
                && existing.amendment.evidence == a.evidence
                && existing.amendment.lineage == a.lineage
            {
                Ok(Admission::Existing(a.id, existing.state.clone()))
            } else {
                Err(Error::Conflict)
            };
        }
        self.current(observed)?;
        match &a.materiality {
            Materiality::Unknown => return Err(Error::EquivalenceUnknown),
            Materiality::EquivalentTo(id) => {
                let e = self.entries.get(id).ok_or(Error::EquivalenceMissing)?;
                return Ok(Admission::Existing(id.clone(), e.state.clone()));
            }
            Materiality::New => {}
        }
        // A claimed new ID with the same evidence does not refresh capacity.
        // Trusted semantic equivalence requires an independent verifier.
        if self.evidence.contains_key(&a.evidence) {
            return Err(Error::DuplicateEvidence);
        }
        if self.charged() >= self.capacity()? {
            return Err(Error::Full);
        }
        let id = a.id.clone();
        let evidence = a.evidence.clone();
        let reservation = self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.entries.insert(
            id.clone(),
            Entry {
                amendment: a,
                reservation,
                state: State::Pending,
            },
        );
        self.evidence.insert(evidence, id);
        self.advance()?;
        Ok(Admission::Reserved(reservation))
    }
    /// Models a possible external emission, but DOES NOT emit anything.
    pub fn begin_emission(&mut self, id: &str, basis: EffectBasis) -> Result<(), Error> {
        if !basis.valid() {
            return Err(Error::Denied);
        }
        let e = self.entries.get(id).ok_or(Error::Missing)?;
        if !matches!(&e.state, State::Pending) {
            return Err(Error::Transition);
        }
        // A rejected overflow must not change state without advancing its CAS fence.
        self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.entries.get_mut(id).ok_or(Error::Missing)?.state = State::OutcomeUnknown;
        self.advance()
    }
    /// Acknowledgement is modeled, not authenticated; a lost acknowledgement
    /// keeps the original unknown reservation charged.
    pub fn acknowledge(&mut self, id: &str, receipt: &str) -> Result<(), Error> {
        if receipt.is_empty() {
            return Err(Error::Invalid);
        }
        let e = self.entries.get(id).ok_or(Error::Missing)?;
        match &e.state {
            State::Committed { receipt: r } if r == receipt => return Ok(()),
            State::OutcomeUnknown => {}
            _ => return Err(Error::Transition),
        }
        if self.artifacts.values().any(|r| r == receipt) {
            return Err(Error::Conflict);
        }
        self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.entries.get_mut(id).ok_or(Error::Missing)?.state = State::Committed {
            receipt: receipt.to_owned(),
        };
        self.artifacts.insert(id.to_owned(), receipt.to_owned());
        self.advance()
    }
    /// Reclamation needs both independently proved no-effect AND an effect-side
    /// fence. A timeout or disappeared worker is neither of these proofs.
    pub fn reclaim(&mut self, p: ReclaimProof) -> Result<(), Error> {
        if !p.no_effect || !p.late_worker_fenced || p.fence.is_empty() {
            return Err(Error::UnsafeReclaim);
        }
        let e = self.entries.get(&p.id).ok_or(Error::Missing)?;
        if matches!(&e.state, State::Committed { .. } | State::Reclaimed { .. }) {
            return Err(Error::Transition);
        }
        self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.entries.get_mut(&p.id).ok_or(Error::Missing)?.state =
            State::Reclaimed { fence: p.fence };
        self.advance()
    }
    /// Synthetic separate issuer predicate; not a real trusted grant source.
    pub fn extend(
        &mut self,
        observed: &Basis,
        id: &str,
        delta: u64,
        independently_approved: bool,
    ) -> Result<(), Error> {
        if !independently_approved || id.is_empty() || delta == 0 {
            return Err(Error::Denied);
        }
        if let Some(existing) = self.grants.get(id) {
            return if *existing == delta {
                Ok(())
            } else {
                Err(Error::GrantConflict)
            };
        }
        self.current(observed)?;
        self.capacity()?.checked_add(delta).ok_or(Error::Overflow)?;
        self.policy_generation
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        self.generation.checked_add(1).ok_or(Error::Overflow)?;
        self.grants.insert(id.to_owned(), delta);
        self.policy_generation += 1;
        self.advance()
    }
    /// Run conservation and uniqueness checks after every successful mutation.
    pub fn verify(&self) -> Result<(), Error> {
        if self.charged() > self.capacity()? || self.entries.len() != self.evidence.len() {
            return Err(Error::Invariant);
        }
        if self
            .entries
            .iter()
            .any(|(id, e)| self.evidence.get(&e.amendment.evidence) != Some(id))
        {
            return Err(Error::Invariant);
        }
        let mut receipts = BTreeSet::new();
        if self.artifacts.iter().any(|(id, receipt)| {
            !receipts.insert(receipt)
                || !matches!(self.state(id), Some(State::Committed { receipt: other }) if other == receipt)
        }) { return Err(Error::Invariant); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lineage() -> Lineage {
        Lineage {
            principal: "fictional".into(),
            principal_generation: 4,
            decision: "shutdown-9".into(),
            cutoff: 80,
            dissent_lineage: "dissent-3".into(),
            incarnation: "ledger-a".into(),
        }
    }
    fn ledger(cap: u64) -> Ledger {
        Ledger::new(lineage(), cap, 9991)
    }
    fn a(id: &str) -> Amendment {
        Amendment {
            id: id.into(),
            payload: format!("p-{id}"),
            evidence: format!("e-{id}"),
            lineage: lineage(),
            materiality: Materiality::New,
        }
    }
    fn reserve(l: &mut Ledger, id: &str) -> Result<Admission, Error> {
        l.reserve(&l.basis(), a(id))
    }
    fn proof(id: &str) -> ReclaimProof {
        ReclaimProof {
            id: id.into(),
            no_effect: true,
            late_worker_fenced: true,
            fence: format!("fence-{id}"),
        }
    }
    fn invariant(l: &Ledger) {
        assert_eq!(l.verify(), Ok(()));
        assert_eq!(l.authority_marker(), 9991);
        assert_eq!(l.cutoff(), 80);
    }
    #[test]
    fn race_on_last_slot_rejects_stale_cas() {
        let mut l = ledger(1);
        let old = l.basis();
        assert!(matches!(
            l.reserve(&old, a("a")),
            Ok(Admission::Reserved(_))
        ));
        assert_eq!(l.reserve(&old, a("b")), Err(Error::Stale));
        assert_eq!(reserve(&mut l, "b"), Err(Error::Full));
        assert_eq!(l.charged(), 1);
        invariant(&l);
    }
    #[test]
    fn lost_ack_replay_never_spends_twice() {
        let mut l = ledger(1);
        let old = l.basis();
        reserve(&mut l, "a").unwrap();
        l.begin_emission("a", EffectBasis::current()).unwrap();
        assert_eq!(
            l.reserve(&old, a("a")),
            Ok(Admission::Existing("a".into(), State::OutcomeUnknown))
        );
        assert_eq!(
            l.begin_emission("a", EffectBasis::current()),
            Err(Error::Transition)
        );
        assert_eq!(l.charged(), 1);
        assert!(l.artifacts().is_empty());
        invariant(&l);
    }
    #[test]
    fn stopped_worker_leaves_pending_charged() {
        let mut l = ledger(1);
        reserve(&mut l, "a").unwrap();
        assert_eq!(reserve(&mut l, "b"), Err(Error::Full));
        invariant(&l);
    }
    #[test]
    fn equivalent_and_duplicate_evidence_use_no_new_slot() {
        let mut l = ledger(2);
        reserve(&mut l, "a").unwrap();
        let mut b = a("b");
        b.materiality = Materiality::EquivalentTo("a".into());
        assert_eq!(
            l.reserve(&l.basis(), b),
            Ok(Admission::Existing("a".into(), State::Pending))
        );
        let mut c = a("c");
        c.evidence = "e-a".into();
        assert_eq!(l.reserve(&l.basis(), c), Err(Error::DuplicateEvidence));
        let mut unknown = a("u");
        unknown.materiality = Materiality::Unknown;
        assert_eq!(
            l.reserve(&l.basis(), unknown),
            Err(Error::EquivalenceUnknown)
        );
        assert_eq!(l.charged(), 1);
        invariant(&l);
    }
    #[test]
    fn stale_policy_lineage_and_changed_payload_denied() {
        let mut l = ledger(2);
        let old = l.basis();
        l.extend(&old, "grant", 1, true).unwrap();
        assert_eq!(l.reserve(&old, a("a")), Err(Error::Stale));
        let mut wrong = a("a");
        wrong.lineage.principal_generation += 1;
        assert_eq!(l.reserve(&l.basis(), wrong), Err(Error::Stale));
        reserve(&mut l, "a").unwrap();
        let mut conflict = a("a");
        conflict.payload.push('x');
        assert_eq!(l.reserve(&l.basis(), conflict), Err(Error::Conflict));
        invariant(&l);
    }
    #[test]
    fn two_distinct_amendments_require_serial_successors() {
        let mut l = ledger(2);
        let old = l.basis();
        l.reserve(&old, a("a")).unwrap();
        assert_eq!(l.reserve(&old, a("b")), Err(Error::Stale));
        reserve(&mut l, "b").unwrap();
        assert_eq!(l.charged(), 2);
        invariant(&l);
    }
    #[test]
    fn reclaimed_slot_fences_late_worker_and_preserves_tombstone() {
        let mut l = ledger(1);
        reserve(&mut l, "a").unwrap();
        l.reclaim(proof("a")).unwrap();
        reserve(&mut l, "b").unwrap();
        assert_eq!(
            l.begin_emission("a", EffectBasis::current()),
            Err(Error::Transition)
        );
        assert_eq!(
            l.reserve(&l.basis(), a("a")),
            Ok(Admission::Existing(
                "a".into(),
                State::Reclaimed {
                    fence: "fence-a".into()
                }
            ))
        );
        assert_eq!(l.charged(), 1);
        invariant(&l);
    }
    #[test]
    fn audience_or_capability_drift_blocks_emission() {
        let mut l = ledger(1);
        reserve(&mut l, "a").unwrap();
        let mut b = EffectBasis::current();
        b.disclosure = false;
        assert_eq!(l.begin_emission("a", b), Err(Error::Denied));
        b.disclosure = true;
        b.channel = false;
        assert_eq!(l.begin_emission("a", b), Err(Error::Denied));
        assert_eq!(l.state("a"), Some(&State::Pending));
        invariant(&l);
    }
    #[test]
    fn committed_receipt_remains_addressable_after_policy_change() {
        let mut l = ledger(1);
        reserve(&mut l, "a").unwrap();
        l.begin_emission("a", EffectBasis::current()).unwrap();
        l.acknowledge("a", "receipt-a").unwrap();
        l.extend(&l.basis(), "grant", 1, true).unwrap();
        assert_eq!(
            l.reserve(&l.basis(), a("a")),
            Ok(Admission::Existing(
                "a".into(),
                State::Committed {
                    receipt: "receipt-a".into()
                }
            ))
        );
        assert_eq!(l.acknowledge("a", "other"), Err(Error::Transition));
        invariant(&l);
    }
    #[test]
    fn only_independent_nonduplicate_grants_add_capacity() {
        let mut l = ledger(1);
        let old = l.basis();
        assert_eq!(l.extend(&old, "grant", 2, false), Err(Error::Denied));
        l.extend(&old, "grant", 2, true).unwrap();
        l.extend(&old, "grant", 2, true).unwrap();
        assert_eq!(
            l.extend(&l.basis(), "grant", 3, true),
            Err(Error::GrantConflict)
        );
        assert_eq!(l.capacity(), Ok(3));
        invariant(&l);
    }
    #[test]
    fn uncertain_effect_cannot_be_reclaimed_by_timer() {
        let mut l = ledger(1);
        reserve(&mut l, "a").unwrap();
        l.begin_emission("a", EffectBasis::current()).unwrap();
        let mut p = proof("a");
        p.late_worker_fenced = false;
        assert_eq!(l.reclaim(p), Err(Error::UnsafeReclaim));
        assert_eq!(reserve(&mut l, "b"), Err(Error::Full));
        invariant(&l);
    }
    #[test]
    fn one_receipt_per_artifact() {
        let mut l = ledger(2);
        reserve(&mut l, "a").unwrap();
        reserve(&mut l, "b").unwrap();
        l.begin_emission("a", EffectBasis::current()).unwrap();
        l.begin_emission("b", EffectBasis::current()).unwrap();
        l.acknowledge("a", "same").unwrap();
        assert_eq!(l.acknowledge("b", "same"), Err(Error::Conflict));
        l.acknowledge("b", "other").unwrap();
        assert_eq!(l.artifacts().len(), 2);
        invariant(&l);
    }
    #[test]
    fn resource_reincarnation_never_reuses_old_basis() {
        let mut l = ledger(1);
        let mut old = l.basis();
        old.lineage.incarnation = "replaced".into();
        assert_eq!(l.reserve(&old, a("a")), Err(Error::Stale));
        invariant(&l);
    }
    #[test]
    fn generation_overflow_leaves_all_rejected_transitions_unchanged() {
        let mut pending = ledger(1);
        reserve(&mut pending, "a").unwrap();
        pending.generation = u64::MAX;
        let before = pending.basis();
        assert_eq!(
            pending.begin_emission("a", EffectBasis::current()),
            Err(Error::Overflow)
        );
        assert_eq!(pending.basis(), before);
        assert_eq!(pending.state("a"), Some(&State::Pending));
        assert_eq!(pending.charged(), 1);
        assert!(pending.artifacts().is_empty());
        invariant(&pending);

        let mut unknown = ledger(1);
        reserve(&mut unknown, "a").unwrap();
        unknown.begin_emission("a", EffectBasis::current()).unwrap();
        unknown.generation = u64::MAX;
        let before = unknown.basis();
        assert_eq!(unknown.acknowledge("a", "receipt-a"), Err(Error::Overflow));
        assert_eq!(unknown.basis(), before);
        assert_eq!(unknown.state("a"), Some(&State::OutcomeUnknown));
        assert_eq!(unknown.charged(), 1);
        assert!(unknown.artifacts().is_empty());
        invariant(&unknown);

        let mut reclaimable = ledger(1);
        reserve(&mut reclaimable, "a").unwrap();
        reclaimable.generation = u64::MAX;
        let before = reclaimable.basis();
        assert_eq!(reclaimable.reclaim(proof("a")), Err(Error::Overflow));
        assert_eq!(reclaimable.basis(), before);
        assert_eq!(reclaimable.state("a"), Some(&State::Pending));
        assert_eq!(reclaimable.charged(), 1);
        assert!(reclaimable.artifacts().is_empty());
        invariant(&reclaimable);
    }

    #[test]
    fn enumerate_729_three_step_interleavings() {
        // 9^3 deterministic operation traces, every transition checked.
        for i in 0..9 {
            for j in 0..9 {
                for k in 0..9 {
                    let mut l = ledger(1);
                    let old = l.basis();
                    for op in [i, j, k] {
                        match op {
                            0 => {
                                let _ = l.reserve(&old, a("a"));
                            }
                            1 => {
                                let _ = reserve(&mut l, "a");
                            }
                            2 => {
                                let _ = reserve(&mut l, "b");
                            }
                            3 => {
                                let _ = l.begin_emission("a", EffectBasis::current());
                            }
                            4 => {
                                let _ = l.begin_emission("b", EffectBasis::current());
                            }
                            5 => {
                                let _ = l.acknowledge("a", "ra");
                            }
                            6 => {
                                let _ = l.acknowledge("b", "rb");
                            }
                            7 => {
                                let _ = l.reclaim(proof("a"));
                            }
                            8 => {
                                let _ = l.extend(&l.basis(), "grant", 1, true);
                            }
                            _ => unreachable!(),
                        }
                        invariant(&l);
                    }
                }
            }
        }
    }
}
