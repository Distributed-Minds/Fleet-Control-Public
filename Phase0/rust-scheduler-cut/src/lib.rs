//! Synthetic scheduler/policy admission cut for FREE ENERGY issue #50 spec 5.
//!
//! OFFLINE MODEL ONLY. The trusted_fence and authority_issued flags below are
//! test-fixture assertions, NOT signatures, locks, provider receipts or a real
//! source of scheduler/worker execution authority.

pub mod continuation;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub identity: String,
    pub incarnation: String,
    pub generation: u64,
}

/// Strict offline-model identity shape; not provider-authenticated provenance.
/// Prevent format controls, invisible aliases and unbounded opaque source keys.
fn bounded_source_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|b| b.is_ascii_graphic())
}

impl Source {
    fn valid(&self) -> bool {
        self.generation != 0
            && bounded_source_id(&self.identity)
            && bounded_source_id(&self.incarnation)
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
    CapacityExhausted,
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

/// Maximum receipts retained by this offline model. Existing receipts are never
/// evicted to free capacity: exact retries must retain their original outcome.
pub const MAX_SIMULATED_START_RECEIPTS: usize = 1024;

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
        // An explicitly supplied bridge is part of the asserted evidence,
        // even when the observed cut is unchanged. Never accept a valid
        // direct-cut start by silently ignoring a contradictory bridge.
        if attempt.original != attempt.effect_current || attempt.bridge.is_some() {
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

        if self.committed.len() >= MAX_SIMULATED_START_RECEIPTS {
            return Err(Denial::CapacityExhausted);
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
    fn cut01_supplied_inconsistent_bridge_does_not_get_ignored() {
        // The cut itself is unchanged, so an absent bridge is admissible.
        // A *supplied* contradictory bridge is not ignorable evidence.
        for case in 0..5 {
            let mut a = attempt();
            let mut proof = bridge(&a.original, &a.effect_current);
            match case {
                0 => proof.predecessor.assignment += 1,
                1 => proof.successor.operation += 1,
                2 => proof.authority_issued = false,
                3 => proof.causally_ordered = false,
                _ => proof.assignment_preserved = false,
            }
            a.bridge = Some(proof);
            let mut book = Simulation::default();
            assert_eq!(book.start(&a), Err(Denial::UnprovedTransition));
            assert_eq!(book.emitted_starts(), 0);
        }
    }

    #[test]
    fn cut01_matching_explicit_bridge_preserves_single_start() {
        let mut a = attempt();
        a.bridge = Some(bridge(&a.original, &a.effect_current));
        let mut book = Simulation::default();
        assert!(matches!(book.start(&a), Ok(Outcome::Committed(_))));
        assert_eq!(book.emitted_starts(), 1);
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
    #[test]
    fn authority_source_identifiers_reject_format_controls_and_unbounded_aliases() {
        for bad in [
            String::new(),
            " ".to_owned(),
            "line\nbreak".to_owned(),
            "spoof\u{202e}value".to_owned(),
            "é".to_owned(),
            "x".repeat(129),
        ] {
            for cut_index in 0..3 {
                for source_index in 0..2 {
                    for field_index in 0..2 {
                        let mut candidate = attempt();
                        let cut = match cut_index {
                            0 => &mut candidate.original,
                            1 => &mut candidate.observed_current,
                            _ => &mut candidate.effect_current,
                        };
                        let source = if source_index == 0 {
                            &mut cut.scheduler
                        } else {
                            &mut cut.policy
                        };
                        if field_index == 0 {
                            source.identity = bad.clone();
                        } else {
                            source.incarnation = bad.clone();
                        }
                        let mut simulation = Simulation::default();
                        assert_eq!(simulation.start(&candidate), Err(Denial::MalformedCut));
                        assert_eq!(simulation.emitted_starts(), 0);
                    }
                }
            }
        }

        let at_limit = Source {
            identity: "s".repeat(128),
            incarnation: "i".repeat(128),
            generation: 1,
        };
        assert!(at_limit.valid());
        let mut above_limit = at_limit.clone();
        above_limit.incarnation.push('i');
        assert!(!above_limit.valid());
    }

    #[test]
    fn start_receipt_capacity_preserves_historical_reconciliation() {
        let mut book = Simulation::default();
        for index in 0..MAX_SIMULATED_START_RECEIPTS {
            let mut a = attempt();
            a.original.operation += index as u64;
            a.original.assignment += index as u64;
            a.observed_current = a.original.clone();
            a.effect_current = a.original.clone();
            assert!(matches!(book.start(&a), Ok(Outcome::Committed(_))));
        }

        let mut overflow = attempt();
        overflow.original.operation += MAX_SIMULATED_START_RECEIPTS as u64;
        overflow.original.assignment += MAX_SIMULATED_START_RECEIPTS as u64;
        overflow.observed_current = overflow.original.clone();
        overflow.effect_current = overflow.original.clone();
        assert_eq!(book.start(&overflow), Err(Denial::CapacityExhausted));
        assert_eq!(book.start(&overflow), Err(Denial::CapacityExhausted));

        assert!(matches!(book.start(&attempt()), Ok(Outcome::Reconciled(_))));
        let mut conflicting_replay = attempt();
        conflicting_replay.original.output_generation += 1;
        assert_eq!(
            book.start(&conflicting_replay),
            Err(Denial::ExistingOperationDifferentIntent)
        );
        assert_eq!(book.emitted_starts(), MAX_SIMULATED_START_RECEIPTS);
        assert_eq!(book.reconcile(81).unwrap().historical_cut, original());
    }}
