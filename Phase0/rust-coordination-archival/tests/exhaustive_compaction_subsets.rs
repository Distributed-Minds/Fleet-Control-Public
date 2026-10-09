//! Exhaustive small-batch compaction selection regressions for #22.
//!
//! Unlike replay-cut tests, these enumerate the deletion candidate subsets.
//! Every successful plan must retain the exact unselected live records and
//! reconstruct the same certified history. All receipts here are synthetic
//! model inputs, NOT provider authority, fencing, or permission to delete.

use free_energy_coordination_archival::compaction::{plan_compaction, PlanFailure};
use free_energy_coordination_archival::{
    replay, Archive, Authority, Basis, DeleteWitness, Denial, Durability, EffectState, Horizon,
    Manifest, Ordering, Protections, ReplayCut, ReplayRecord, Snapshot, Source,
};

const FIRST: u64 = 40;
const OVERLAP: usize = 4;

fn basis(name: &str) -> Basis {
    Basis {
        identity: name.to_owned(),
        generation: 7,
    }
}

fn record(sequence: u64) -> ReplayRecord {
    ReplayRecord {
        stable_id: format!("opaque-{}", 100 - sequence),
        sequence,
        payload_digest: [sequence as u8; 32],
        order_basis: basis("certified-order"),
        source_incarnation: "live-incarnation-7".into(),
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: basis("certified-order"),
        source_incarnation: "live-incarnation-7".into(),
        first_sequence: FIRST,
        last_sequence: FIRST + OVERLAP as u64 + 1,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn histories() -> (Vec<ReplayRecord>, Vec<ReplayRecord>) {
    (
        (FIRST..=FIRST + OVERLAP as u64).map(record).collect(),
        (FIRST + 1..=FIRST + OVERLAP as u64 + 1)
            .map(record)
            .collect(),
    )
}

fn witness(record: &ReplayRecord) -> DeleteWitness {
    let source = Source {
        record_id: record.stable_id.clone(),
        incarnation: record.source_incarnation.clone(),
        version: format!("etag-{}", record.sequence),
        content_digest: record.payload_digest,
    };
    let manifest = basis("selected-manifest");
    let horizon = basis("authorized-horizon");
    let operation_id = format!("remove-{}", record.stable_id);
    DeleteWitness {
        operation_id: operation_id.clone(),
        observed_source: source.clone(),
        archive: Archive {
            source: source.clone(),
            manifest: manifest.clone(),
            segment: "immutable-segment".into(),
            manifest_digest: [19; 32],
            remotely_read_digest: [19; 32],
            exact_remote_readback: true,
        },
        ordering: Ordering {
            basis: basis("certified-order"),
            source_incarnation: source.incarnation.clone(),
            authoritative: true,
            frontier_complete: true,
            current: true,
        },
        manifest: Manifest {
            basis: manifest.clone(),
            destination_incarnation: "archive-incarnation-7".into(),
            unique_current_selection: true,
            predecessor_transition_fenced: true,
        },
        snapshot: Snapshot {
            manifest: manifest.clone(),
            ordering: basis("certified-order"),
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
            policy_source_incarnation: "horizon-policy-7".into(),
            uniquely_current: true,
            transition_authorized: true,
            preserve_through_epoch: 100,
            predecessor_protects_source: false,
            exact_retirement_authorized: false,
        },
        durability: Durability {
            manifest,
            segment: "immutable-segment".into(),
            destination_incarnation: "archive-incarnation-7".into(),
            horizon,
            reconstructed_through_epoch: 100,
            current: true,
            keys_recoverable: true,
        },
        protections: Protections::default(),
        effect_state: EffectState::NotAttempted,
    }
}

#[test]
fn every_candidate_subset_retains_exactly_the_unselected_live_records() {
    let expected_history: Vec<_> = (FIRST..=FIRST + OVERLAP as u64 + 1).map(record).collect();
    for mask in 0..(1_usize << OVERLAP) {
        let selected: Vec<_> = (0..OVERLAP)
            .filter(|index| mask & (1_usize << *index) != 0)
            .map(|index| FIRST + index as u64 + 1)
            .collect();
        for reversed in [false, true] {
            let (mut archived, mut live) = histories();
            let mut witnesses: Vec<_> = selected
                .iter()
                .map(|sequence| witness(&record(*sequence)))
                .collect();
            if reversed {
                archived.reverse();
                live.reverse();
                witnesses.reverse();
            }
            let original_archived = archived.clone();
            let original_live = live.clone();
            let plan = plan_compaction(&archived, &live, &cut(), &witnesses)
                .expect("exact, uniquely witnessed overlap must be model-plannable");
            let removed: Vec<_> = plan
                .model_removals
                .iter()
                .map(|candidate| candidate.sequence)
                .collect();
            assert_eq!(removed, selected, "mask={mask:#06b} reversed={reversed}");
            let survivors: Vec<_> = live
                .iter()
                .filter(|record| !selected.contains(&record.sequence))
                .cloned()
                .collect();
            assert_eq!(
                plan.retained_live, survivors,
                "mask={mask:#06b} reversed={reversed}"
            );
            assert_eq!(plan.reconstructed, expected_history);
            assert_eq!(
                replay(&archived, &plan.retained_live, &cut()),
                Ok(expected_history.clone()),
                "post-removal historical replay changed for mask={mask:#06b}"
            );
            assert_eq!(archived, original_archived);
            assert_eq!(live, original_live);
        }
    }
}

#[test]
fn every_candidate_position_can_poison_an_otherwise_valid_batch_atomically() {
    let (archived, live) = histories();
    for bad_index in 0..OVERLAP {
        let good_index = (bad_index + 1) % OVERLAP;
        let good = witness(&live[good_index]);
        let mut bad = witness(&live[bad_index]);
        bad.protections.live_tail = true;
        for proofs in [[good.clone(), bad.clone()], [bad.clone(), good.clone()]] {
            assert_eq!(
                plan_compaction(&archived, &live, &cut(), &proofs),
                Err(PlanFailure::Ineligible(Denial::ProtectedRecord)),
                "bad_index={bad_index}; must not return a partial plan"
            );
        }
        bad.protections.live_tail = false;
        bad.effect_state = EffectState::AckUnknown;
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[good, bad]),
            Err(PlanFailure::ReconcilePriorEffect),
            "unknown prior effect must not produce a fresh removal plan"
        );
    }
}
