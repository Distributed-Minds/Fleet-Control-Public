//! Large, deterministic, model-only compaction planner regression (#22).
//! No proof in this test grants provider mutation authority or a DELETE token.

use free_energy_coordination_archival::compaction::{plan_compaction, PlanFailure};
use free_energy_coordination_archival::{
    replay, Archive, Authority, Basis, DeleteWitness, Denial, Durability, EffectState, Horizon,
    Manifest, Ordering, Protections, ReplayCut, ReplayRecord, Snapshot, Source,
};

fn basis(name: &str) -> Basis {
    Basis {
        identity: name.to_owned(),
        generation: 9,
    }
}

fn record(sequence: u64) -> ReplayRecord {
    ReplayRecord {
        // Reversing opaque identity order ensures that lexical order is not
        // accidentally mistaken for the provider-certified reducer order.
        stable_id: format!("opaque-{:04}", 1999 - sequence),
        sequence,
        payload_digest: [sequence as u8; 32],
        order_basis: basis("authoritative-order"),
        source_incarnation: "live-v9".to_owned(),
        source_version: "fixture-version-v1".to_owned(),
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: basis("authoritative-order"),
        source_incarnation: "live-v9".to_owned(),
        manifest: basis("current-manifest"),
        first_sequence: 0,
        last_sequence: 1999,
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
        version: record.source_version.clone(),
        content_digest: record.payload_digest,
    };
    let manifest = basis("current-manifest");
    let horizon = basis("current-horizon");
    let operation_id = format!("remove-{}", record.stable_id);
    DeleteWitness {
        operation_id: operation_id.clone(),
        observed_source: source.clone(),
        archive: Archive {
            source: source.clone(),
            manifest: manifest.clone(),
            segment: "immutable-segment".into(),
            manifest_digest: [17; 32],
            remotely_read_digest: [17; 32],
            exact_remote_readback: true,
        },
        ordering: Ordering {
            basis: basis("authoritative-order"),
            source_incarnation: "live-v9".into(),
            authoritative: true,
            frontier_complete: true,
            current: true,
        },
        manifest: Manifest {
            basis: manifest.clone(),
            destination_incarnation: "archive-v9".into(),
            unique_current_selection: true,
            predecessor_transition_fenced: true,
        },
        snapshot: Snapshot {
            manifest: manifest.clone(),
            ordering: basis("authoritative-order"),
            source_incarnation: "live-v9".into(),
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
            policy_source_incarnation: "policy-v9".into(),
            uniquely_current: true,
            transition_authorized: true,
            preserve_through_epoch: 500,
            predecessor_protects_source: false,
            exact_retirement_authorized: false,
        },
        durability: Durability {
            manifest,
            segment: "immutable-segment".into(),
            destination_incarnation: "archive-v9".into(),
            horizon,
            reconstructed_through_epoch: 500,
            current: true,
            keys_recoverable: true,
        },
        protections: Protections::default(),
        effect_state: EffectState::NotAttempted,
    }
}

fn shuffle<T>(values: &mut [T], mut seed: usize) {
    for index in (1..values.len()).rev() {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        values.swap(index, seed % (index + 1));
    }
}

fn histories() -> (Vec<ReplayRecord>, Vec<ReplayRecord>) {
    (
        (0..=1599).map(record).collect(),
        (1400..=1999).map(record).collect(),
    )
}

fn candidates() -> Vec<DeleteWitness> {
    (1400..1600)
        .step_by(2)
        .map(|sequence| witness(&record(sequence)))
        .collect()
}

#[test]
fn two_thousand_positions_have_the_same_safe_compaction_plan_under_page_reordering() {
    let expected: Vec<_> = (0..=1999).map(record).collect();
    let selected: Vec<u64> = (1400..1600).step_by(2).collect();
    for seed in [1, 7, 31, 101, 2027, 99991] {
        let (mut archived, mut live) = histories();
        let mut proofs = candidates();
        shuffle(&mut archived, seed);
        shuffle(&mut live, seed + 13);
        shuffle(&mut proofs, seed + 29);
        let archived_before = archived.clone();
        let live_before = live.clone();

        let plan = plan_compaction(&archived, &live, &cut(), &proofs)
            .expect("all witnesses exactly identify already archived positions");
        let removed: Vec<_> = plan
            .model_removals
            .iter()
            .map(|candidate| candidate.sequence)
            .collect();
        assert_eq!(
            removed, selected,
            "untrusted provider page order seed {seed}"
        );
        assert_eq!(plan.reconstructed, expected);
        assert_eq!(plan.retained_live.len(), 500);
        assert_eq!(
            replay(&archived, &plan.retained_live, &cut()),
            Ok(expected.clone())
        );
        for survivor in &plan.retained_live {
            assert!(
                survivor.sequence >= 1600 || survivor.sequence % 2 == 1,
                "planner retained an eligible even-numbered overlap"
            );
        }
        // The model must propose removals without modifying its inputs.
        assert_eq!(archived, archived_before);
        assert_eq!(live, live_before);
    }
}

#[test]
fn a_single_revoked_predicate_prevents_any_partial_plan() {
    let (archived, live) = histories();
    let original = live.clone();
    let mut proofs = candidates();
    proofs[99].protections.live_tail = true;
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &proofs),
        Err(PlanFailure::Ineligible(Denial::ProtectedRecord))
    );
    assert_eq!(live, original, "denied planning cannot mutate live history");

    let mut proofs = candidates();
    proofs[99].durability.reconstructed_through_epoch = 499;
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &proofs),
        Err(PlanFailure::Ineligible(Denial::DurabilityInsufficient))
    );
    assert_eq!(live, original);
}

#[test]
fn unarchived_live_tail_is_not_a_removal_candidate() {
    let (archived, live) = histories();
    let wrong = witness(&record(1999));
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &[wrong]),
        Err(PlanFailure::MissingOrAmbiguousArchivedCopy)
    );
}

#[test]
fn globally_sparse_github_comment_ids_require_exact_archived_copy() {
    // Synthetic records use real-sized, globally sparse GitHub ID boundaries,
    // not actual comment payloads or current archival authority.
    let first = "6072460160";
    let second = "6072460161";
    let last = "6086765381";
    let unarchived_in_range = "6080000000";
    assert_eq!(
        last.parse::<u64>().unwrap() - first.parse::<u64>().unwrap() + 1,
        14_305_222
    );

    let mut history: Vec<_> = (0..4).map(record).collect();
    for (row, id) in history
        .iter_mut()
        .zip([first, second, last, unarchived_in_range])
    {
        row.stable_id = id.to_owned();
    }
    let archived = history[..3].to_vec();
    let live = history[1..].to_vec();
    let mut scoped_cut = cut();
    scoped_cut.first_sequence = 0;
    scoped_cut.last_sequence = 3;

    let witnesses = [witness(&live[0]), witness(&live[1])];
    let plan = plan_compaction(&archived, &live, &scoped_cut, &witnesses)
        .expect("only the two exact archived live identities are eligible");
    assert_eq!(
        plan.model_removals
            .iter()
            .map(|removal| removal.source.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![second, last]
    );
    assert_eq!(plan.retained_live, vec![live[2].clone()]);
    assert_eq!(plan.reconstructed, history);

    // This ID lies between the same numeric segment boundaries but does not
    // occur in the archived records. Numeric-span membership grants nothing.
    assert_eq!(
        plan_compaction(&archived, &live, &scoped_cut, &[witness(&live[2])]),
        Err(PlanFailure::MissingOrAmbiguousArchivedCopy)
    );
}
