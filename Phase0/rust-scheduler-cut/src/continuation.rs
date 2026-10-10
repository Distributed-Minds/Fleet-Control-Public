//! CUT12–CUT14: deterministic *offline* continuation admission model for #50.
//!
//! A historical scheduler start and its continuation policy are recorded together.
//! The booleans used to model trusted currentness and compatible transitions are
//! fixture inputs, never provider authority, a persistent receipt, or a live fence.

use std::collections::BTreeMap;

use crate::{Compatibility, Cut, Denial, Outcome, Receipt, Simulation, StartAttempt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationPolicy {
    /// Scheduler policy admitted with the historical start allows a bounded
    /// number of current-run steps, even when later configuration changes.
    BoundedCurrentRun { max_steps: u32 },
    /// Every next step requires its own coherent successor cut and historical
    /// bridge. The allowed step count is fixed at semantic start.
    SuccessorReAdmission { max_steps: u32 },
}

impl ContinuationPolicy {
    fn max_steps(&self) -> u32 {
        match self {
            Self::BoundedCurrentRun { max_steps } | Self::SuccessorReAdmission { max_steps } => {
                *max_steps
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalStart {
    pub receipt: Receipt,
    pub continuation_policy: ContinuationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartContractDenial {
    InvalidHistoricalContract,
    ChangedHistoricalContract,
    OperationAlreadyUsed,
    Start(Denial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationAttempt {
    pub start_operation: u64,
    pub operation: u64,
    pub step: u32,
    pub observed_current: Option<Cut>,
    pub effect_current: Option<Cut>,
    pub bridge: Option<Compatibility>,
    /// Synthetic assertion; not a trusted real-world effect-time fence.
    pub trusted_fence: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationReceipt {
    pub start_operation: u64,
    pub operation: u64,
    pub step: u32,
    pub historical_start_cut: Cut,
    pub authorized_cut: Cut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationOutcome {
    Committed(ContinuationReceipt),
    Reconciled(ContinuationReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationDenial {
    UnknownHistoricalStart,
    InvalidOperation,
    InvalidStep,
    NoJointFence,
    ContractBoundaryExceeded,
    OutOfOrderStep,
    StepAlreadyCommitted,
    ExistingOperationDifferentIntent,
    UnexpectedSuccessor,
    MissingSuccessor,
    InvalidSuccessorCut,
    EffectTimeDrift,
    UnprovedSuccessorTransition,
    CapacityExhausted,
}

/// Bound synthetic continuation receipts without evicting previously committed
/// operation/step identities; a full model still reconciles existing receipts.
pub const MAX_SIMULATED_CONTINUATION_RECEIPTS: usize = 1024;

/// In-memory, single-threaded fixture ledger. No crash persistence, concurrent
/// transaction, scheduler service, domain-policy authorization or provider API.
#[derive(Default)]
pub struct ContinuationSimulation {
    starts: Simulation,
    historical: BTreeMap<u64, HistoricalStart>,
    continuations: BTreeMap<u64, (ContinuationAttempt, ContinuationReceipt)>,
    steps: BTreeMap<(u64, u32), u64>,
}

impl ContinuationSimulation {
    /// Atomic only within this deterministic in-memory model: a successful
    /// semantic start fixes both the historical cut and its immutable rule.
    pub fn start(
        &mut self,
        attempt: &StartAttempt,
        policy: ContinuationPolicy,
    ) -> Result<Outcome, StartContractDenial> {
        if policy.max_steps() == 0 {
            return Err(StartContractDenial::InvalidHistoricalContract);
        }
        let id = attempt.original.operation;
        if self.continuations.contains_key(&id) {
            return Err(StartContractDenial::OperationAlreadyUsed);
        }
        if let Some(existing) = self.historical.get(&id) {
            if existing.receipt.historical_cut != attempt.original
                || existing.continuation_policy != policy
            {
                return Err(StartContractDenial::ChangedHistoricalContract);
            }
            return self
                .starts
                .start(attempt)
                .map_err(StartContractDenial::Start);
        }

        let result = self
            .starts
            .start(attempt)
            .map_err(StartContractDenial::Start)?;
        if let Outcome::Committed(receipt) = &result {
            self.historical.insert(
                id,
                HistoricalStart {
                    receipt: receipt.clone(),
                    continuation_policy: policy,
                },
            );
        }
        Ok(result)
    }

    pub fn historical(&self, start_operation: u64) -> Option<&HistoricalStart> {
        self.historical.get(&start_operation)
    }

    pub fn emitted_starts(&self) -> usize {
        self.starts.emitted_starts()
    }

    pub fn emitted_continuations(&self) -> usize {
        self.continuations.len()
    }

    pub fn continue_run(
        &mut self,
        attempt: &ContinuationAttempt,
    ) -> Result<ContinuationOutcome, ContinuationDenial> {
        // Reconcile an exact previously committed operation before reading
        // any newly current policy. Changed replay intent is never accepted.
        if let Some((original, receipt)) = self.continuations.get(&attempt.operation) {
            return if original == attempt {
                Ok(ContinuationOutcome::Reconciled(receipt.clone()))
            } else {
                Err(ContinuationDenial::ExistingOperationDifferentIntent)
            };
        }
        if attempt.operation == 0
            || attempt.operation == attempt.start_operation
            || self.historical.contains_key(&attempt.operation)
        {
            return Err(ContinuationDenial::InvalidOperation);
        }
        let historic = self
            .historical
            .get(&attempt.start_operation)
            .ok_or(ContinuationDenial::UnknownHistoricalStart)?;
        if attempt.step == 0 {
            return Err(ContinuationDenial::InvalidStep);
        }
        if attempt.step > historic.continuation_policy.max_steps() {
            return Err(ContinuationDenial::ContractBoundaryExceeded);
        }
        if self
            .steps
            .contains_key(&(attempt.start_operation, attempt.step))
        {
            return Err(ContinuationDenial::StepAlreadyCommitted);
        }
        if attempt.step > 1
            && !self
                .steps
                .contains_key(&(attempt.start_operation, attempt.step - 1))
        {
            return Err(ContinuationDenial::OutOfOrderStep);
        }
        if !attempt.trusted_fence {
            return Err(ContinuationDenial::NoJointFence);
        }

        let authorized_cut = match historic.continuation_policy {
            ContinuationPolicy::BoundedCurrentRun { .. } => {
                // The admitted historical mode, not current P2, governs this
                // already-started bounded execution. Reject injected successor
                // evidence rather than silently changing execution modes.
                if attempt.observed_current.is_some()
                    || attempt.effect_current.is_some()
                    || attempt.bridge.is_some()
                {
                    return Err(ContinuationDenial::UnexpectedSuccessor);
                }
                historic.receipt.historical_cut.clone()
            }
            ContinuationPolicy::SuccessorReAdmission { .. } => {
                let observed = attempt
                    .observed_current
                    .as_ref()
                    .ok_or(ContinuationDenial::MissingSuccessor)?;
                let effect = attempt
                    .effect_current
                    .as_ref()
                    .ok_or(ContinuationDenial::MissingSuccessor)?;
                if !observed.valid()
                    || !effect.valid()
                    || !observed.admitted
                    || !effect.admitted
                    || observed.assignment != historic.receipt.assignment
                    || observed.operation != attempt.operation
                {
                    return Err(ContinuationDenial::InvalidSuccessorCut);
                }
                if observed != effect {
                    return Err(ContinuationDenial::EffectTimeDrift);
                }

                // Each subsequent step links to the previously committed
                // successor, not back to an unrelated/stale original cut.
                let predecessor = if attempt.step == 1 {
                    &historic.receipt.historical_cut
                } else {
                    let previous_id = self.steps[&(attempt.start_operation, attempt.step - 1)];
                    &self.continuations[&previous_id].1.authorized_cut
                };
                let valid_bridge = attempt.bridge.as_ref().is_some_and(|bridge| {
                    bridge.predecessor.eq(predecessor)
                        && bridge.successor.eq(effect)
                        && bridge.authority_issued
                        && bridge.causally_ordered
                        && bridge.assignment_preserved
                });
                if !valid_bridge {
                    return Err(ContinuationDenial::UnprovedSuccessorTransition);
                }
                effect.clone()
            }
        };

        if self.continuations.len() >= MAX_SIMULATED_CONTINUATION_RECEIPTS {
            return Err(ContinuationDenial::CapacityExhausted);
        }

        let receipt = ContinuationReceipt {
            start_operation: attempt.start_operation,
            operation: attempt.operation,
            step: attempt.step,
            historical_start_cut: historic.receipt.historical_cut.clone(),
            authorized_cut,
        };
        self.steps
            .insert((attempt.start_operation, attempt.step), attempt.operation);
        self.continuations
            .insert(attempt.operation, (attempt.clone(), receipt.clone()));
        Ok(ContinuationOutcome::Committed(receipt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Source;

    fn source(name: &str, generation: u64) -> Source {
        Source {
            identity: name.into(),
            incarnation: format!("{name}-inc-1"),
            generation,
        }
    }

    fn cut() -> Cut {
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

    fn start() -> StartAttempt {
        StartAttempt {
            original: cut(),
            observed_current: cut(),
            effect_current: cut(),
            bridge: None,
            trusted_fence: true,
        }
    }

    fn continuation(operation: u64, step: u32) -> ContinuationAttempt {
        ContinuationAttempt {
            start_operation: 81,
            operation,
            step,
            observed_current: None,
            effect_current: None,
            bridge: None,
            trusted_fence: true,
        }
    }

    fn successor(attempt: &mut ContinuationAttempt, from: &Cut) {
        let mut next = from.clone();
        next.operation = attempt.operation;
        next.scheduler.generation += 1;
        next.policy.generation += 1;
        next.admission_policy_generation += 1;
        next.output_generation += 1;
        attempt.observed_current = Some(next.clone());
        attempt.effect_current = Some(next.clone());
        attempt.bridge = Some(Compatibility {
            predecessor: from.clone(),
            successor: next,
            authority_issued: true,
            causally_ordered: true,
            assignment_preserved: true,
        });
    }

    #[test]
    fn cut12_historical_current_run_is_bounded_and_not_rewritten_by_p2() {
        let mut book = ContinuationSimulation::default();
        book.start(
            &start(),
            ContinuationPolicy::BoundedCurrentRun { max_steps: 1 },
        )
        .unwrap();
        let one = continuation(82, 1);
        let receipt = match book.continue_run(&one).unwrap() {
            ContinuationOutcome::Committed(r) => r,
            _ => unreachable!(),
        };
        assert_eq!(receipt.historical_start_cut, cut());
        assert_eq!(receipt.authorized_cut, cut());
        assert_eq!(
            book.continue_run(&one),
            Ok(ContinuationOutcome::Reconciled(receipt))
        );
        let mut later_p2 = start();
        later_p2.observed_current.policy.generation = 2;
        later_p2.observed_current.admission_policy_generation = 2;
        later_p2.effect_current = later_p2.observed_current.clone();
        assert!(matches!(
            book.start(
                &later_p2,
                ContinuationPolicy::BoundedCurrentRun { max_steps: 1 }
            ),
            Ok(Outcome::Reconciled(_))
        ));
        assert_eq!(
            book.continue_run(&continuation(83, 2)),
            Err(ContinuationDenial::ContractBoundaryExceeded)
        );
        assert_eq!(book.emitted_starts(), 1);
        assert_eq!(book.emitted_continuations(), 1);
        assert_eq!(book.historical(81).unwrap().receipt.historical_cut, cut());
    }

    #[test]
    fn cut13_incompatible_p2_or_untrusted_bridge_cannot_continue() {
        let mut book = ContinuationSimulation::default();
        book.start(
            &start(),
            ContinuationPolicy::SuccessorReAdmission { max_steps: 1 },
        )
        .unwrap();
        let mut candidate = continuation(82, 1);
        successor(&mut candidate, &cut());
        candidate.bridge = None;
        assert_eq!(
            book.continue_run(&candidate),
            Err(ContinuationDenial::UnprovedSuccessorTransition)
        );
        successor(&mut candidate, &cut());
        candidate.bridge.as_mut().unwrap().authority_issued = false;
        assert_eq!(
            book.continue_run(&candidate),
            Err(ContinuationDenial::UnprovedSuccessorTransition)
        );
        successor(&mut candidate, &cut());
        candidate.effect_current.as_mut().unwrap().policy.generation = 3;
        candidate
            .effect_current
            .as_mut()
            .unwrap()
            .admission_policy_generation = 3;
        assert_eq!(
            book.continue_run(&candidate),
            Err(ContinuationDenial::EffectTimeDrift)
        );
        assert_eq!(book.emitted_continuations(), 0);
    }

    #[test]
    fn cut14_authorized_successor_is_exact_and_replay_safe() {
        let mut book = ContinuationSimulation::default();
        book.start(
            &start(),
            ContinuationPolicy::SuccessorReAdmission { max_steps: 2 },
        )
        .unwrap();
        let mut first = continuation(82, 1);
        successor(&mut first, &cut());
        let first_receipt = match book.continue_run(&first).unwrap() {
            ContinuationOutcome::Committed(r) => r,
            _ => unreachable!(),
        };
        assert_eq!(first_receipt.historical_start_cut, cut());
        assert_eq!(
            book.continue_run(&first),
            Ok(ContinuationOutcome::Reconciled(first_receipt.clone()))
        );
        let mut changed = first.clone();
        changed.bridge.as_mut().unwrap().causally_ordered = false;
        assert_eq!(
            book.continue_run(&changed),
            Err(ContinuationDenial::ExistingOperationDifferentIntent)
        );
        let mut second = continuation(83, 2);
        successor(&mut second, &first_receipt.authorized_cut);
        assert!(matches!(
            book.continue_run(&second),
            Ok(ContinuationOutcome::Committed(_))
        ));
        assert_eq!(
            book.continue_run(&continuation(84, 3)),
            Err(ContinuationDenial::ContractBoundaryExceeded)
        );
        assert_eq!(book.emitted_starts(), 1);
        assert_eq!(book.emitted_continuations(), 2);
        assert_eq!(book.historical(81).unwrap().receipt.historical_cut, cut());
    }

    #[test]
    fn continuation_negative_controls_reject_cross_step_and_changed_policy() {
        let mut book = ContinuationSimulation::default();
        let a = continuation(82, 1);
        assert_eq!(
            book.continue_run(&a),
            Err(ContinuationDenial::UnknownHistoricalStart)
        );
        book.start(
            &start(),
            ContinuationPolicy::BoundedCurrentRun { max_steps: 2 },
        )
        .unwrap();
        assert_eq!(
            book.start(
                &start(),
                ContinuationPolicy::SuccessorReAdmission { max_steps: 2 }
            ),
            Err(StartContractDenial::ChangedHistoricalContract)
        );
        assert_eq!(
            book.continue_run(&continuation(83, 2)),
            Err(ContinuationDenial::OutOfOrderStep)
        );
        let mut no_fence = a.clone();
        no_fence.trusted_fence = false;
        assert_eq!(
            book.continue_run(&no_fence),
            Err(ContinuationDenial::NoJointFence)
        );
        book.continue_run(&a).unwrap();
        assert_eq!(
            book.continue_run(&continuation(84, 1)),
            Err(ContinuationDenial::StepAlreadyCommitted)
        );
        let mut forged = continuation(83, 2);
        successor(&mut forged, &cut());
        assert_eq!(
            book.continue_run(&forged),
            Err(ContinuationDenial::UnexpectedSuccessor)
        );
        assert_eq!(book.emitted_continuations(), 1);
    }

    #[test]
    fn continuation_capacity_denies_new_steps_without_erasing_replay_evidence() {
        let mut book = ContinuationSimulation::default();
        book.start(
            &start(),
            ContinuationPolicy::BoundedCurrentRun {
                max_steps: MAX_SIMULATED_CONTINUATION_RECEIPTS as u32 + 1,
            },
        )
        .unwrap();

        for step in 1..=(MAX_SIMULATED_CONTINUATION_RECEIPTS as u32) {
            let candidate = continuation(81 + u64::from(step), step);
            assert!(matches!(
                book.continue_run(&candidate),
                Ok(ContinuationOutcome::Committed(_))
            ));
        }

        let beyond = continuation(
            82 + MAX_SIMULATED_CONTINUATION_RECEIPTS as u64,
            MAX_SIMULATED_CONTINUATION_RECEIPTS as u32 + 1,
        );
        assert_eq!(
            book.continue_run(&beyond),
            Err(ContinuationDenial::CapacityExhausted)
        );
        assert_eq!(
            book.continue_run(&beyond),
            Err(ContinuationDenial::CapacityExhausted)
        );
        assert!(matches!(
            book.continue_run(&continuation(82, 1)),
            Ok(ContinuationOutcome::Reconciled(_))
        ));
        let mut changed = continuation(82, 1);
        changed.trusted_fence = false;
        assert_eq!(
            book.continue_run(&changed),
            Err(ContinuationDenial::ExistingOperationDifferentIntent)
        );
        assert_eq!(
            book.emitted_continuations(),
            MAX_SIMULATED_CONTINUATION_RECEIPTS
        );
        assert_eq!(book.emitted_starts(), 1);
     }
}
