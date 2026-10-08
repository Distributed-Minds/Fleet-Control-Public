//! Model-only adversarial regression matrix for archival admission (#22).
//!
//! This tests AND-composition and replay under controlled evidence loss.
//! Caller-supplied proof fields are NOT authenticated authority or permission
//! to compact a real provider's coordination history.

use free_energy_coordination_archival::*;

fn basis(identity: &str, generation: u64) -> Basis {
    Basis {
        identity: identity.into(),
        generation,
    }
}

fn eligible_witness() -> DeleteWitness {
    let source = Source {
        record_id: "record-42".into(),
        incarnation: "live-incarnation-2".into(),
        version: "etag-42".into(),
        content_digest: [42; 32],
    };
    let manifest = basis("manifest-current", 4);
    let order = basis("order-authoritative", 2);
    let horizon = basis("horizon-authorized", 3);
    DeleteWitness {
        operation_id: "delete-op-42".into(),
        observed_source: source.clone(),
        archive: Archive {
            source: source.clone(),
            manifest: manifest.clone(),
            segment: "segment-7".into(),
            manifest_digest: [7; 32],
            remotely_read_digest: [7; 32],
            exact_remote_readback: true,
        },
        ordering: Ordering {
            basis: order.clone(),
            source_incarnation: source.incarnation.clone(),
            authoritative: true,
            frontier_complete: true,
            current: true,
        },
        manifest: Manifest {
            basis: manifest.clone(),
            destination_incarnation: "archive-incarnation-3".into(),
            unique_current_selection: true,
            predecessor_transition_fenced: true,
        },
        snapshot: Snapshot {
            manifest: manifest.clone(),
            ordering: order,
            source_incarnation: source.incarnation.clone(),
            coherent_cut: true,
            frontier_complete: true,
            closing_fence_current: true,
            stable_identity_dedup: true,
        },
        authority: Authority {
            operation_id: "delete-op-42".into(),
            source,
            action: "DELETE_SOURCE_RECORD".into(),
            current: true,
            fence_current: true,
        },
        horizon: Horizon {
            basis: horizon.clone(),
            policy_source_incarnation: "policy-source-1".into(),
            uniquely_current: true,
            transition_authorized: true,
            preserve_through_epoch: 2000,
            predecessor_protects_source: false,
            exact_retirement_authorized: false,
        },
        durability: Durability {
            manifest,
            segment: "segment-7".into(),
            destination_incarnation: "archive-incarnation-3".into(),
            horizon,
            reconstructed_through_epoch: 2000,
            current: true,
            keys_recoverable: true,
        },
        protections: Protections::default(),
        effect_state: EffectState::NotAttempted,
    }
}

type Mutation = (&'static str, fn(&mut DeleteWitness), Denial);

#[test]
fn loss_of_any_independent_admission_predicate_never_increases_eligibility() {
    use Denial::*;
    let mutations: &[Mutation] = &[
        ("archive readback unavailable", |w| w.archive.exact_remote_readback = false, ArchiveNotExact),
        ("archive hash drift", |w| w.archive.remotely_read_digest[0] ^= 1, ArchiveNotExact),
        ("archived source version changed", |w| w.archive.source.version = "etag-older".into(), ArchiveNotExact),
        ("archive segment missing", |w| w.archive.segment.clear(), ArchiveNotExact),
        ("archive manifest identity missing", |w| w.archive.manifest.identity.clear(), ArchiveNotExact),
        ("ordering untrusted", |w| w.ordering.authoritative = false, OrderNotProven),
        ("ordering frontier unknown", |w| w.ordering.frontier_complete = false, OrderNotProven),
        ("ordering expired", |w| w.ordering.current = false, OrderNotProven),
        ("ordering lineage changed", |w| w.ordering.source_incarnation = "old-live".into(), OrderNotProven),
        ("ordering identity missing", |w| w.ordering.basis.identity.clear(), OrderNotProven),
        ("manifest forked", |w| w.manifest.unique_current_selection = false, ManifestNotCurrent),
        ("manifest fence lost", |w| w.manifest.predecessor_transition_fenced = false, ManifestNotCurrent),
        ("manifest generation changed", |w| w.manifest.basis.generation += 1, ManifestNotCurrent),
        ("destination incarnation missing", |w| w.manifest.destination_incarnation.clear(), ManifestNotCurrent),
        ("snapshot mixed cut", |w| w.snapshot.coherent_cut = false, SnapshotNotCoherent),
        ("snapshot missing live frontier", |w| w.snapshot.frontier_complete = false, SnapshotNotCoherent),
        ("snapshot closing fence stale", |w| w.snapshot.closing_fence_current = false, SnapshotNotCoherent),
        ("snapshot dedup unproved", |w| w.snapshot.stable_identity_dedup = false, SnapshotNotCoherent),
        ("snapshot order generation moved", |w| w.snapshot.ordering.generation += 1, SnapshotNotCoherent),
        ("snapshot source changed", |w| w.snapshot.source_incarnation = "another-live".into(), SnapshotNotCoherent),
        ("observed source ID missing", |w| w.observed_source.record_id.clear(), ArchiveNotExact),
        ("operation ID missing", |w| w.operation_id.clear(), AuthorityNotCurrent),
        ("mutation authority revoked", |w| w.authority.current = false, AuthorityNotCurrent),
        ("mutation fence revoked", |w| w.authority.fence_current = false, AuthorityNotCurrent),
        ("mutation operation mismatch", |w| w.authority.operation_id = "other".into(), AuthorityNotCurrent),
        ("mutation action mismatch", |w| w.authority.action = "WRITE_MANIFEST".into(), AuthorityNotCurrent),
        ("mutation source mismatch", |w| w.authority.source.content_digest[0] ^= 1, AuthorityNotCurrent),
        ("horizon policy source missing", |w| w.horizon.policy_source_incarnation.clear(), HorizonNotAuthorized),
        ("horizon selection fork", |w| w.horizon.uniquely_current = false, HorizonNotAuthorized),
        ("horizon transition unauthorized", |w| w.horizon.transition_authorized = false, HorizonNotAuthorized),
        ("predecessor still protected", |w| w.horizon.predecessor_protects_source = true, HorizonNotAuthorized),
        ("horizon identity missing", |w| w.horizon.basis.identity.clear(), HorizonNotAuthorized),
        ("durability expired", |w| w.durability.current = false, DurabilityInsufficient),
        ("durability key missing", |w| w.durability.keys_recoverable = false, DurabilityInsufficient),
        ("durability incomplete horizon", |w| w.durability.reconstructed_through_epoch = 1999, DurabilityInsufficient),
        ("durability destination moved", |w| w.durability.destination_incarnation = "archive-restored".into(), DurabilityInsufficient),
        ("durability manifest stale", |w| w.durability.manifest.generation += 1, DurabilityInsufficient),
        ("durability horizon stale", |w| w.durability.horizon.generation += 1, DurabilityInsufficient),
        ("durability segment changed", |w| w.durability.segment = "segment-6".into(), DurabilityInsufficient),
        ("active owner protected", |w| w.protections.active_ownership_chain = true, ProtectedRecord),
        ("latest state protected", |w| w.protections.latest_persistent_state = true, ProtectedRecord),
        ("live tail protected", |w| w.protections.live_tail = true, ProtectedRecord),
        ("referenced record protected", |w| w.protections.authoritative_reference = true, ProtectedRecord),
    ];

    assert_eq!(evaluate(&eligible_witness()), Verdict::EligibleModelOnly);
    for (label, mutation, denial) in mutations {
        let mut witness = eligible_witness();
        mutation(&mut witness);
        assert_eq!(
            evaluate(&witness),
            Verdict::Ineligible(*denial),
            "{label} unexpectedly qualified under model admission"
        );
    }
}

#[test]
fn unknown_or_partial_prior_effects_are_reconciled_before_eligibility() {
    for state in [
        EffectState::AckUnknown,
        EffectState::PartialEffect,
        EffectState::AlreadyApplied,
    ] {
        let mut witness = eligible_witness();
        witness.effect_state = state;
        assert_eq!(evaluate(&witness), Verdict::ReconcilePriorEffect);
        witness.authority.current = false;
        assert_eq!(
            evaluate(&witness),
            Verdict::ReconcilePriorEffect,
            "stale authority never justifies blind effect replay"
        );
    }
}

fn record(id: &str, position: u64) -> ReplayRecord {
    ReplayRecord {
        stable_id: id.into(),
        sequence: position,
        payload_digest: [(position & 255) as u8; 32],
        order_basis: basis("ordered", 7),
        source_incarnation: "live-1".into(),
    }
}

fn cut(first: u64, last: u64) -> ReplayCut {
    ReplayCut {
        ordering: basis("ordered", 7),
        source_incarnation: "live-1".into(),
        first_sequence: first,
        last_sequence: last,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

#[test]
fn archive_live_reordering_and_exact_overlap_preserve_one_ordered_history() {
    let expected = vec![record("a", 10), record("b", 11), record("c", 12)];
    for (archived, live) in [
        (vec![record("a", 10), record("b", 11)], vec![record("b", 11), record("c", 12)]),
        (vec![record("b", 11), record("a", 10)], vec![record("c", 12), record("b", 11)]),
        (vec![record("c", 12)], vec![record("b", 11), record("a", 10)]),
        (vec![], vec![record("c", 12), record("a", 10), record("b", 11)]),
    ] {
        assert_eq!(replay(&archived, &live, &cut(10, 12)), Ok(expected.clone()));
    }
}

#[test]
fn replay_rejects_cross_tier_forks_gaps_and_unsupported_cut_identity() {
    let initial = vec![record("a", 10), record("b", 11)];
    let mut conflict = record("b", 11);
    conflict.payload_digest[0] ^= 1;
    assert_eq!(
        replay(&initial, &[conflict, record("c", 12)], &cut(10, 12)),
        Err(ReplayFailure::ConflictingDuplicate)
    );
    assert_eq!(
        replay(&initial, &[record("b", 12)], &cut(10, 12)),
        Err(ReplayFailure::ConflictingDuplicate)
    );
    assert_eq!(
        replay(&[record("a", 10)], &[record("c", 12)], &cut(10, 12)),
        Err(ReplayFailure::IncompleteHistory)
    );

    let mut bad_cut = cut(10, 12);
    bad_cut.current_manifest = false;
    assert_eq!(
        replay(&initial, &[record("c", 12)], &bad_cut),
        Err(ReplayFailure::UntrustedCut)
    );

    let mut mismatched = record("c", 12);
    mismatched.order_basis.generation += 1;
    assert_eq!(
        replay(&initial, &[mismatched], &cut(10, 12)),
        Err(ReplayFailure::UntrustedCut)
    );

    let mut unexpected = record("x", 13);
    unexpected.source_incarnation = "old-live".into();
    assert_eq!(
        replay(&initial, &[unexpected], &cut(10, 12)),
        Err(ReplayFailure::UntrustedCut)
    );
}

#[test]
fn terminal_u64_sequence_and_empty_frontier_are_bounded() {
    let max = u64::MAX;
    let high = [record("x", max - 2), record("y", max - 1)];
    let tail = [record("z", max)];
    let expected = vec![high[0].clone(), high[1].clone(), tail[0].clone()];
    assert_eq!(replay(&high, &tail, &cut(max - 2, max)), Ok(expected));
    assert_eq!(
        replay(&[record("x", max - 2)], &tail, &cut(max - 2, max)),
        Err(ReplayFailure::IncompleteHistory)
    );
    assert_eq!(
        replay(&[], &[], &cut(0, max)),
        Err(ReplayFailure::IncompleteHistory)
    );
    assert_eq!(
        replay(&[], &[], &cut(12, 10)),
        Err(ReplayFailure::UntrustedCut)
    );
}
