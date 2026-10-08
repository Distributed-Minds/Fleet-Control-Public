//! Deterministic, side-effect-free candidate planning for archive-first compaction.
//!
//! Model witnesses are NOT provider attestations, mutation authorization, or
//! DELETE tokens. A trusted executor must independently revalidate every
//! receipt at the effect boundary under the installed authority protocol.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    evaluate, replay, DeleteWitness, Denial, ReplayCut, ReplayFailure, ReplayRecord, Source,
    Verdict,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRemoval {
    pub sequence: u64,
    pub source: Source,
    pub operation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionPlan {
    pub model_removals: Vec<ModelRemoval>,
    pub retained_live: Vec<ReplayRecord>,
    pub reconstructed: Vec<ReplayRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanFailure {
    UntrustedHistory(ReplayFailure),
    DuplicateCandidate,
    MissingOrAmbiguousLiveRecord,
    MissingOrAmbiguousArchivedCopy,
    InconsistentWitness,
    Ineligible(Denial),
    ReconcilePriorEffect,
    PostRemovalHistory(ReplayFailure),
    ReplayDiverged,
}

/// Plan only exact, already-archived copies of eligible live records.
///
/// The complete pre-removal replay MUST already be admissible. Every candidate
/// must appear exactly once in each tier, have identical reducer-relevant
/// record bytes, and bind the exact independent source and snapshot bases.
/// This routine never mutates the caller's slices, storage, or provider state.
pub fn plan_compaction(
    archived: &[ReplayRecord],
    live: &[ReplayRecord],
    cut: &ReplayCut,
    witnesses: &[DeleteWitness],
) -> Result<CompactionPlan, PlanFailure> {
    let reconstructed = replay(archived, live, cut).map_err(PlanFailure::UntrustedHistory)?;

    let mut live_by_id: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut archived_by_id: HashMap<&str, Vec<&ReplayRecord>> = HashMap::new();
    for (index, record) in live.iter().enumerate() {
        live_by_id.entry(&record.stable_id).or_default().push(index);
    }
    for record in archived {
        archived_by_id
            .entry(&record.stable_id)
            .or_default()
            .push(record);
    }

    let mut selected = BTreeSet::new();
    let mut operations = HashSet::new();
    let mut model_removals = Vec::with_capacity(witnesses.len());
    // A batch is admitted against one coherent archive manifest generation.
    // Individually valid witness snapshots from different generations cannot
    // be composed into one plan without a trusted cross-generation cut.
    let mut manifest_basis = None;
    for witness in witnesses {
        match &manifest_basis {
            Some(expected) if expected != &witness.manifest.basis => {
                return Err(PlanFailure::InconsistentWitness);
            }
            None => manifest_basis = Some(witness.manifest.basis.clone()),
            _ => {}
        }
        let source = &witness.observed_source;
        if !operations.insert(witness.operation_id.as_str()) {
            return Err(PlanFailure::DuplicateCandidate);
        }
        let positions = live_by_id
            .get(source.record_id.as_str())
            .ok_or(PlanFailure::MissingOrAmbiguousLiveRecord)?;
        if positions.len() != 1 {
            return Err(PlanFailure::MissingOrAmbiguousLiveRecord);
        }
        let index = positions[0];
        if !selected.insert(index) {
            return Err(PlanFailure::DuplicateCandidate);
        }
        let record = &live[index];
        if record.source_incarnation != source.incarnation
            || record.payload_digest != source.content_digest
            || record.order_basis != cut.ordering
            || witness.ordering.basis != cut.ordering
            || witness.ordering.source_incarnation != cut.source_incarnation
            || witness.snapshot.ordering != cut.ordering
            || witness.snapshot.source_incarnation != cut.source_incarnation
            || witness.snapshot.manifest != witness.manifest.basis
        {
            return Err(PlanFailure::InconsistentWitness);
        }
        let copies = archived_by_id
            .get(source.record_id.as_str())
            .ok_or(PlanFailure::MissingOrAmbiguousArchivedCopy)?;
        if copies.len() != 1 || *copies[0] != *record {
            return Err(PlanFailure::MissingOrAmbiguousArchivedCopy);
        }
        match evaluate(witness) {
            Verdict::EligibleModelOnly => {}
            Verdict::Ineligible(reason) => return Err(PlanFailure::Ineligible(reason)),
            Verdict::ReconcilePriorEffect => return Err(PlanFailure::ReconcilePriorEffect),
        }
        model_removals.push(ModelRemoval {
            sequence: record.sequence,
            source: source.clone(),
            operation_id: witness.operation_id.clone(),
        });
    }

    let retained_live: Vec<ReplayRecord> = live
        .iter()
        .enumerate()
        .filter(|(index, _)| !selected.contains(index))
        .map(|(_, record)| record.clone())
        .collect();
    let replayed =
        replay(archived, &retained_live, cut).map_err(PlanFailure::PostRemovalHistory)?;
    if replayed != reconstructed {
        return Err(PlanFailure::ReplayDiverged);
    }
    model_removals.sort_by_key(|record| record.sequence);

    Ok(CompactionPlan {
        model_removals,
        retained_live,
        reconstructed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Archive, Authority, Basis, Durability, EffectState, Horizon, Manifest, Ordering,
        Protections, Snapshot,
    };

    fn basis(identity: &str) -> Basis {
        Basis {
            identity: identity.to_owned(),
            generation: 5,
        }
    }

    fn record(id: &str, sequence: u64) -> ReplayRecord {
        ReplayRecord {
            stable_id: id.to_owned(),
            sequence,
            payload_digest: [sequence as u8; 32],
            order_basis: basis("provider-order"),
            source_incarnation: "live-v2".into(),
        }
    }

    fn cut() -> ReplayCut {
        ReplayCut {
            ordering: basis("provider-order"),
            source_incarnation: "live-v2".into(),
            first_sequence: 10,
            last_sequence: 13,
            authoritative_order: true,
            current_manifest: true,
            coherent_snapshot: true,
            complete_frontier: true,
            closing_fence_current: true,
        }
    }

    fn witness(record: &ReplayRecord) -> DeleteWitness {
        let source = Source {
            record_id: record.stable_id.clone(),
            incarnation: record.source_incarnation.clone(),
            version: format!("etag-{}", record.sequence),
            content_digest: record.payload_digest,
        };
        let manifest = basis("manifest-current");
        let horizon = basis("horizon-current");
        let operation_id = format!("delete-{}", record.stable_id);
        DeleteWitness {
            operation_id: operation_id.clone(),
            observed_source: source.clone(),
            archive: Archive {
                source: source.clone(),
                manifest: manifest.clone(),
                segment: "immutable-segment".into(),
                manifest_digest: [21; 32],
                remotely_read_digest: [21; 32],
                exact_remote_readback: true,
            },
            ordering: Ordering {
                basis: basis("provider-order"),
                source_incarnation: source.incarnation.clone(),
                authoritative: true,
                frontier_complete: true,
                current: true,
            },
            manifest: Manifest {
                basis: manifest.clone(),
                destination_incarnation: "archive-v3".into(),
                unique_current_selection: true,
                predecessor_transition_fenced: true,
            },
            snapshot: Snapshot {
                manifest: manifest.clone(),
                ordering: basis("provider-order"),
                source_incarnation: source.incarnation.clone(),
                coherent_cut: true,
                frontier_complete: true,
                closing_fence_current: true,
                stable_identity_dedup: true,
            },
            authority: Authority {
                operation_id,
                source,
                action: "DELETE_SOURCE_RECORD".into(),
                current: true,
                fence_current: true,
            },
            horizon: Horizon {
                basis: horizon.clone(),
                policy_source_incarnation: "policy-v2".into(),
                uniquely_current: true,
                transition_authorized: true,
                preserve_through_epoch: 100,
                predecessor_protects_source: false,
                exact_retirement_authorized: false,
            },
            durability: Durability {
                manifest,
                segment: "immutable-segment".into(),
                destination_incarnation: "archive-v3".into(),
                horizon,
                reconstructed_through_epoch: 100,
                current: true,
                keys_recoverable: true,
            },
            protections: Protections::default(),
            effect_state: EffectState::NotAttempted,
        }
    }

    fn histories() -> (Vec<ReplayRecord>, Vec<ReplayRecord>) {
        (
            vec![record("r10", 10), record("r11", 11), record("r12", 12)],
            vec![record("r11", 11), record("r12", 12), record("r13", 13)],
        )
    }

    #[test]
    fn plans_exact_overlaps_and_preserves_order_sensitive_history() {
        let (archived, live) = histories();
        let witnesses = [witness(&live[0]), witness(&live[1])];
        let plan = plan_compaction(&archived, &live, &cut(), &witnesses).unwrap();
        assert_eq!(plan.model_removals.len(), 2);
        assert_eq!(plan.model_removals[0].sequence, 11);
        assert_eq!(plan.model_removals[1].sequence, 12);
        assert_eq!(plan.retained_live, vec![record("r13", 13)]);
        assert_eq!(
            plan.reconstructed,
            vec![
                record("r10", 10),
                record("r11", 11),
                record("r12", 12),
                record("r13", 13),
            ]
        );
        assert_eq!(live.len(), 3);
        assert_eq!(archived.len(), 3);
    }

    #[test]
    fn no_candidates_replays_without_removing_anything() {
        let (archived, live) = histories();
        let plan = plan_compaction(&archived, &live, &cut(), &[]).unwrap();
        assert!(plan.model_removals.is_empty());
        assert_eq!(plan.retained_live, live);
    }

    #[test]
    fn missing_or_ambiguous_archive_copy_is_not_eligible() {
        let (mut archived, live) = histories();
        let candidate = witness(&live[0]);
        archived.retain(|r| r.stable_id != "r11");
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[candidate.clone()]),
            Err(PlanFailure::MissingOrAmbiguousArchivedCopy)
        );
        archived.push(record("r11", 11));
        archived.push(record("r11", 11));
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[candidate]),
            Err(PlanFailure::MissingOrAmbiguousArchivedCopy)
        );
    }

    #[test]
    fn mismatch_and_stale_order_cannot_remove_anything() {
        let (archived, live) = histories();
        let mut bad = witness(&live[0]);
        bad.observed_source.content_digest = [100; 32];
        bad.archive.source.content_digest = [100; 32];
        bad.authority.source.content_digest = [100; 32];
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[bad]),
            Err(PlanFailure::InconsistentWitness)
        );
        let mut bad = witness(&live[0]);
        bad.snapshot.ordering.generation += 1;
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[bad]),
            Err(PlanFailure::InconsistentWitness)
        );
        let mut bad_cut = cut();
        bad_cut.closing_fence_current = false;
        assert_eq!(
            plan_compaction(&archived, &live, &bad_cut, &[]),
            Err(PlanFailure::UntrustedHistory(ReplayFailure::UntrustedCut))
        );
    }

    #[test]
    fn duplicate_candidate_and_ambiguous_live_record_fail_closed() {
        let (archived, mut live) = histories();
        let first = witness(&live[0]);
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[first.clone(), first.clone()]),
            Err(PlanFailure::DuplicateCandidate)
        );
        live.push(record("r11", 11));
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[first]),
            Err(PlanFailure::MissingOrAmbiguousLiveRecord)
        );
    }

    #[test]
    fn protection_and_unknown_prior_effect_block_selection() {
        let (archived, live) = histories();
        let mut protected = witness(&live[0]);
        protected.protections.active_ownership_chain = true;
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[protected]),
            Err(PlanFailure::Ineligible(Denial::ProtectedRecord))
        );
        let mut retry = witness(&live[0]);
        retry.effect_state = EffectState::AckUnknown;
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[retry]),
            Err(PlanFailure::ReconcilePriorEffect)
        );
    }

    #[test]
    fn mixed_manifest_generations_cannot_form_a_single_compaction_plan() {
        let (archived, live) = histories();
        let first = witness(&live[0]);
        let original_second = witness(&live[1]);
        let mut moved_second = original_second.clone();

        // The second witness is internally consistent, and independently
        // eligible: only the combined plan crosses manifest generations.
        moved_second.manifest.basis.generation += 1;
        moved_second.archive.manifest.generation += 1;
        moved_second.snapshot.manifest.generation += 1;
        moved_second.durability.manifest.generation += 1;
        assert_eq!(evaluate(&first), Verdict::EligibleModelOnly);
        assert_eq!(evaluate(&moved_second), Verdict::EligibleModelOnly);

        let original_live = live.clone();
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[first.clone(), moved_second]),
            Err(PlanFailure::InconsistentWitness)
        );
        assert_eq!(live, original_live, "rejected model must not alter input");
        assert!(
            plan_compaction(&archived, &live, &cut(), &[first, original_second]).is_ok(),
            "matching manifest generation must remain plannable"
        );
    }

    #[test]
    fn non_archived_live_tail_cannot_be_selected() {
        let (archived, live) = histories();
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[witness(&live[2])]),
            Err(PlanFailure::MissingOrAmbiguousArchivedCopy)
        );
    }
}
