//! Synthetic scheduler/policy admission cut for FREE ENERGY issue #50 spec 5.
//!
//! OFFLINE MODEL ONLY. The trusted_fence and authority_issued flags below are
//! test-fixture assertions, NOT signatures, locks, provider receipts or a real
//! source of scheduler/worker execution authority.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub identity: String,
    pub incarnation: String,
    pub generation: u64,
}

impl Source {
    fn valid(&self) -> bool {
        !self.identity.is_empty()
            && !self.incarnation.is_empty()
            && self.generation != 0
            && self.identity.len() <= 128
            && self.incarnation.len() <= 128
    }
}

/// Immutable simulation evidence of what was *actually admitted*.
/// A production implementation must obtain a jointly authoritative snapshot
/// or a causally ordered, provider-fenced admission-compatible transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    pub schema: u32,
    pub assignment: u64,
    pub operation: u64,
    pub scheduler: Source,
    pub policy: Source,
    pub output_generation: u64,
    pub admission_policy_generation: u64,
    pub capacity_generation: u64,
    pub admitted: bool,
}

impl Cut {
    fn valid(&self) -> bool {
        self.schema == 1
            && self.assignment != 0
            && self.operation != 0
            && self.scheduler.valid()
            && self.policy.valid()
            && self.output_generation != 0
            && self.capacity_generation != 0
            && self.admission_policy_generation == self.policy.generation
    }
}

/// Explicitly binds both ends of an *assignment-specific* transition. In this
/// model proof booleans are injected by the fixture; they are never sufficient
/// for real current-selection, authority or effect-time execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compatibility {
    pub predecessor: Cut,
    pub successor: Cut,
    pub authority_issued: bool,
    pub causally_ordered: bool,
    pub assignment_preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartAttempt {
    pub original: Cut,
    pub observed_current: Cut,
    pub effect_current: Cut,
    pub bridge: Option<Compatibility>,
    /// Synthetic stand-in for a cross-domain, non-bypassable effect-time fence.
    pub trusted_fence: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Denial {
    MalformedCut,
    NotAdmitted,
    NoJointFence,
    EffectTimeDrift,
    UnprovedTransition,
    ExistingOperationDifferentIntent,
    AssignmentAlreadyStarted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub assignment: u64,
    pub operation: u64,
    pub historical_cut: Cut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Committed(Receipt),
    Reconciled(Receipt),
}

/// Single-threaded, in-memory testing ledger. No database, crash durability,
/// live authorization, network provider adapter or concurrency guarantee.
#[derive(Default)]
pub struct Simulation {
    committed: BTreeMap<u64, Receipt>,
    by_assignment: BTreeMap<u64, u64>,
    emitted_starts: usize,
}

impl Simulation {
    pub fn start(&mut self, attempt: &StartAttempt) -> Result<Outcome, Denial> {
        // Reconciliation precedes evaluating *new* policy. Retries keep the
        // original historical cut and cannot manufacture a second operation.
        if let Some(original) = self.committed.get(&attempt.original.operation) {
            if original.historical_cut == attempt.original {
                return Ok(Outcome::Reconciled(original.clone()));
            }
            return Err(Denial::ExistingOperationDifferentIntent);
        }

        if !attempt.original.valid()
            || !attempt.observed_current.valid()
            || !attempt.effect_current.valid()
            || attempt.original.assignment != attempt.observed_current.assignment
            || attempt.original.operation != attempt.observed_current.operation
        {
            return Err(Denial::MalformedCut);
        }
        if !attempt.original.admitted || !attempt.observed_current.admitted {
            return Err(Denial::NotAdmitted);
        }
        if !attempt.trusted_fence {
            return Err(Denial::NoJointFence);
        }
        if attempt.observed_current != attempt.effect_current {
            return Err(Denial::EffectTimeDrift);
        }
        if attempt.original != attempt.effect_current {
            let valid_bridge = attempt.bridge.as_ref().is_some_and(|bridge| {
                bridge.predecessor == attempt.original
                    && bridge.successor == attempt.effect_current
                    && bridge.authority_issued
                    && bridge.causally_ordered
                    && bridge.assignment_preserved
            });
            if !valid_bridge {
                return Err(Denial::UnprovedTransition);
            }
        }
        if self
            .by_assignment
            .contains_key(&attempt.original.assignment)
        {
            return Err(Denial::AssignmentAlreadyStarted);
        }

        let receipt = Receipt {
            assignment: attempt.original.assignment,
            operation: attempt.original.operation,
            historical_cut: attempt.original.clone(),
        };
        self.committed
            .insert(attempt.original.operation, receipt.clone());
        self.by_assignment
            .insert(attempt.original.assignment, attempt.original.operation);
        self.emitted_starts += 1;
        Ok(Outcome::Committed(receipt))
    }

    pub fn reconcile(&self, operation: u64) -> Option<&Receipt> {
        self.committed.get(&operation)
    }

    pub fn emitted_starts(&self) -> usize {
        self.emitted_starts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(name: &str, generation: u64) -> Source {
        Source {
            identity: name.into(),
            incarnation: format!("{name}-inc-1"),
            generation,
        }
    }

    fn original() -> Cut {
        Cut {
            schema: 1,
            assignment: 12,
            operation: 81,
            scheduler: source("scheduler", 1),
            policy: source("policy", 1),
            output_generation: 3,
            admission_policy_generation: 1,
            capacity_generation: 1,
            admitted: true,
        }
    }

    fn attempt() -> StartAttempt {
        StartAttempt {
            original: original(),
            observed_current: original(),
            effect_current: original(),
            bridge: None,
            trusted_fence: true,
        }
    }

    fn bridge(from: &Cut, to: &Cut) -> Compatibility {
        Compatibility {
            predecessor: from.clone(),
            successor: to.clone(),
            authority_issued: true,
            causally_ordered: true,
            assignment_preserved: true,
        }
    }

    #[test]
    fn cut01_joint_fenced_state_starts_exactly_once() {
        let mut simulation = Simulation::default();
        let first = simulation.start(&attempt()).unwrap();
        assert!(matches!(first, Outcome::Committed(_)));
        assert_eq!(simulation.emitted_starts(), 1);
        assert_eq!(
            simulation.start(&attempt()).unwrap(),
            Outcome::Reconciled(match first {
                Outcome::Committed(receipt) => receipt,
                Outcome::Reconciled(_) => unreachable!(),
            })
        );
        assert_eq!(simulation.emitted_starts(), 1);
    }

    #[test]
    fn cut02_03_individually_current_policy_is_not_joint_admission() {
        let mut a = attempt();
        a.observed_current.policy.generation = 2;
        a.observed_current.admission_policy_generation = 2;
        a.effect_current = a.observed_current.clone();
        let mut simulation = Simulation::default();
        assert_eq!(simulation.start(&a), Err(Denial::UnprovedTransition));
        assert_eq!(simulation.emitted_starts(), 0);
    }

    #[test]
    fn cut04_assignment_specific_ordered_transition_can_be_modelled() {
        let mut a = attempt();
        a.effect_current.policy.generation = 2;
        a.effect_current.admission_policy_generation = 2;
        a.observed_current = a.effect_current.clone();
        a.bridge = Some(bridge(&a.original, &a.effect_current));
        let mut simulation = Simulation::default();
        assert!(matches!(simulation.start(&a), Ok(Outcome::Committed(_))));
        assert_eq!(simulation.reconcile(81).unwrap().historical_cut, original());
    }

    #[test]
    fn cut05_and_cut18_worker_asserted_or_unordered_bridge_denied() {
        let mut a = attempt();
        a.effect_current.policy.generation = 2;
        a.effect_current.admission_policy_generation = 2;
        a.observed_current = a.effect_current.clone();
        for mode in 0..3 {
            let mut bad = bridge(&a.original, &a.effect_current);
            match mode {
                0 => bad.authority_issued = false,
                1 => bad.causally_ordered = false,
                _ => bad.assignment_preserved = false,
            }
            a.bridge = Some(bad);
            let mut book = Simulation::default();
            assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
            assert_eq!(book.emitted_starts(), 0);
        }
    }

    #[test]
    fn cut06_07_dual_handoff_needs_one_exact_ordered_bridge() {
        let mut a = attempt();
        a.effect_current.scheduler.generation += 1;
        a.effect_current.policy.generation += 1;
        a.effect_current.admission_policy_generation += 1;
        a.effect_current.output_generation += 1;
        a.observed_current = a.effect_current.clone();
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
        a.bridge = Some(bridge(&a.original, &a.effect_current));
        assert!(matches!(book.start(&a), Ok(Outcome::Committed(_))));
        assert_eq!(book.emitted_starts(), 1);
    }

    #[test]
    fn cut08_source_reincarnation_is_not_same_basis() {
        let mut a = attempt();
        a.effect_current.scheduler.incarnation = "restored-copy".into();
        a.observed_current = a.effect_current.clone();
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
    }

    #[test]
    fn cut09_effect_movement_after_observation_denied() {
        let mut a = attempt();
        a.effect_current.capacity_generation += 1;
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::EffectTimeDrift));
        assert_eq!(book.emitted_starts(), 0);
    }

    #[test]
    fn cut10_committed_lost_ack_reconciles_historical_cut() {
        let mut book = Simulation::default();
        book.start(&attempt()).unwrap();
        let mut retry = attempt();
        retry.observed_current.policy.generation += 1;
        retry.observed_current.admission_policy_generation += 1;
        retry.effect_current = retry.observed_current.clone();
        assert!(matches!(book.start(&retry), Ok(Outcome::Reconciled(_))));
        assert_eq!(book.reconcile(81).unwrap().historical_cut, original());
        assert_eq!(book.emitted_starts(), 1);
    }

    #[test]
    fn cut11_uncommitted_retry_cannot_rebuild_changed_policy() {
        let mut a = attempt();
        a.observed_current.policy.generation += 1;
        a.observed_current.admission_policy_generation += 1;
        a.effect_current = a.observed_current.clone();
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
        assert_eq!(book.reconcile(81), None);
    }

    #[test]
    fn cut15_missing_authoritative_joint_fence_denied() {
        let mut a = attempt();
        a.trusted_fence = false;
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::NoJointFence));
        assert_eq!(book.emitted_starts(), 0);
    }

    #[test]
    fn cut16_two_operations_same_assignment_cannot_both_start() {
        let mut book = Simulation::default();
        book.start(&attempt()).unwrap();
        let mut rival = attempt();
        rival.original.operation = 82;
        rival.observed_current.operation = 82;
        rival.effect_current.operation = 82;
        assert_eq!(book.start(&rival), Err(Denial::AssignmentAlreadyStarted));
        assert_eq!(book.emitted_starts(), 1);
    }

    #[test]
    fn cut17_independent_capacity_change_is_not_ignored() {
        let mut a = attempt();
        a.observed_current.capacity_generation += 1;
        a.effect_current = a.observed_current.clone();
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
    }

    #[test]
    fn identity_schema_and_missing_admission_fail_closed() {
        let mut a = attempt();
        a.original.schema = 2;
        let mut book = Simulation::default();
        assert_eq!(book.start(&a), Err(Denial::MalformedCut));
        a = attempt();
        a.observed_current.admitted = false;
        a.effect_current = a.observed_current.clone();
        assert_eq!(book.start(&a), Err(Denial::NotAdmitted));
        assert_eq!(book.emitted_starts(), 0);
    }

    #[test]
    fn changed_original_operation_identity_does_not_reconcile() {
        let mut book = Simulation::default();
        book.start(&attempt()).unwrap();
        let mut forged = attempt();
        forged.original.output_generation += 1;
        assert_eq!(
            book.start(&forged),
            Err(Denial::ExistingOperationDifferentIntent)
        );
        assert_eq!(book.emitted_starts(), 1);
    }
}
