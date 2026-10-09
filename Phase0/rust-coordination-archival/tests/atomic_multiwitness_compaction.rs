//! #22: model-only multi-candidate compaction preflight must be atomic.
//! A batch containing one invalid witness must never yield a successful
//! partial removal plan. These locally manufactured receipts are NOT trusted
//! provider attestations, deletion permission, or a live compactor.

use free_energy_coordination_archival::compaction::{plan_compaction, PlanFailure};
use free_energy_coordination_archival::*;

fn basis(name: &str) -> Basis {
    Basis {
        identity: name.into(),
        generation: 4,
    }
}

fn record(id: &str, sequence: u64) -> ReplayRecord {
    ReplayRecord {
        stable_id: id.into(),
        sequence,
        payload_digest: [sequence as u8; 32],
        order_basis: basis("authoritative-order"),
        source_incarnation: "live-generation-2".into(),
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: basis("authoritative-order"),
        source_incarnation: "live-generation-2".into(),
        manifest: basis("unique-manifest"),
        first_sequence: 10,
        last_sequence: 13,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn history() -> (Vec<ReplayRecord>, Vec<ReplayRecord>) {
    (
        vec![record("r10", 10), record("r11", 11), record("r12", 12)],
        vec![record("r11", 11), record("r12", 12), record("r13", 13)],
    )
}

fn witness(record: &ReplayRecord) -> DeleteWitness {
    let source = Source {
        record_id: record.stable_id.clone(),
        incarnation: record.source_incarnation.clone(),
        version: format!("etag-{}", record.sequence),
        content_digest: record.payload_digest,
    };
    let manifest = basis("unique-manifest");
    let horizon = basis("authorized-horizon");
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
            basis: basis("authoritative-order"),
            source_incarnation: source.incarnation.clone(),
            authoritative: true,
            frontier_complete: true,
            current: true,
        },
        manifest: Manifest {
            basis: manifest.clone(),
            destination_incarnation: "archive-store-3".into(),
            unique_current_selection: true,
            predecessor_transition_fenced: true,
        },
        snapshot: Snapshot {
            manifest: manifest.clone(),
            ordering: basis("authoritative-order"),
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
            policy_source_incarnation: "current-policy".into(),
            uniquely_current: true,
            transition_authorized: true,
            preserve_through_epoch: 100,
            predecessor_protects_source: false,
            exact_retirement_authorized: false,
        },
        durability: Durability {
            manifest,
            segment: "immutable-segment".into(),
            destination_incarnation: "archive-store-3".into(),
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
fn exact_batch_is_sorted_by_authoritative_position_not_witness_order() {
    let (archived, live) = history();
    let first = witness(&live[0]);
    let second = witness(&live[1]);

    let forward = plan_compaction(&archived, &live, &cut(), &[first.clone(), second.clone()])
        .expect("both exact copies are model eligible");
    let reverse = plan_compaction(&archived, &live, &cut(), &[second, first])
        .expect("candidate page ordering does not grant new authority");

    assert_eq!(forward, reverse);
    assert_eq!(
        forward
            .model_removals
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        [11, 12]
    );
    assert_eq!(forward.retained_live, [record("r13", 13)]);
    assert_eq!(
        forward.reconstructed,
        [
            record("r10", 10),
            record("r11", 11),
            record("r12", 12),
            record("r13", 13)
        ]
    );
}

#[test]
fn one_failed_rights_or_durability_guard_poison_the_entire_batch_in_any_position() {
    let (archived, live) = history();
    let good = witness(&live[0]);
    let mut bad = witness(&live[1]);
    bad.horizon.predecessor_protects_source = true;
    for candidates in [[good.clone(), bad.clone()], [bad.clone(), good.clone()]] {
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &candidates),
            Err(PlanFailure::Ineligible(Denial::HorizonNotAuthorized))
        );
    }

    bad.horizon.predecessor_protects_source = false;
    bad.durability.keys_recoverable = false;
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &[good, bad]),
        Err(PlanFailure::Ineligible(Denial::DurabilityInsufficient))
    );

    assert_eq!(
        archived,
        [record("r10", 10), record("r11", 11), record("r12", 12)]
    );
    assert_eq!(
        live,
        [record("r11", 11), record("r12", 12), record("r13", 13)]
    );
}

#[test]
fn operation_reuse_across_distinct_exact_sources_is_not_a_second_delete_grant() {
    let (archived, live) = history();
    let first = witness(&live[0]);
    let mut other = witness(&live[1]);
    // The source, archiver and authority receipts individually remain
    // coherent. Reusing a token for a different source is a batch conflict.
    other.operation_id = first.operation_id.clone();
    other.authority.operation_id = first.operation_id.clone();
    assert_eq!(evaluate(&other), Verdict::EligibleModelOnly);
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &[first, other]),
        Err(PlanFailure::DuplicateCandidate)
    );
}

#[test]
fn replay_cut_manifest_must_match_a_self_consistent_compaction_batch() {
    let (archived, live) = history();
    let first = witness(&live[0]);
    let second = witness(&live[1]);
    assert_eq!(evaluate(&first), Verdict::EligibleModelOnly);
    assert_eq!(evaluate(&second), Verdict::EligibleModelOnly);

    let mut previous_cut = cut();
    previous_cut.manifest.generation -= 1;
    assert_eq!(
        plan_compaction(&archived, &live, &previous_cut, &[first, second]),
        Err(PlanFailure::InconsistentWitness)
    );
    assert_eq!(
        plan_compaction(&archived, &live, &cut(), &[witness(&live[0]), witness(&live[1])])
            .expect("same manifest is coherent").model_removals.len(),
        2
    );
}

#[test]
fn unknown_prior_effect_fails_the_full_batch_without_blind_retry() {
    let (archived, live) = history();
    let good = witness(&live[0]);
    let mut retry = witness(&live[1]);
    for state in [
        EffectState::AckUnknown,
        EffectState::PartialEffect,
        EffectState::AlreadyApplied,
    ] {
        retry.effect_state = state;
        assert_eq!(
            plan_compaction(&archived, &live, &cut(), &[good.clone(), retry.clone()]),
            Err(PlanFailure::ReconcilePriorEffect)
        );
    }
}
