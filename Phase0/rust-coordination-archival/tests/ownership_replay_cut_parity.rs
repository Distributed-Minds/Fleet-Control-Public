//! Cross-component regression for #22's *model-only* archive/live replay.
//!
//! Unlike the toy ownership reducer fixture, this runs the actual compiled
//! Phase0 ownership transition reducer after assembling a two-tier replay.
//! Inputs are synthetic, not authenticated GitHub evidence. Successful tests
//! do NOT authorize a provider mutation or prove historical Python parity.

use free_energy_coordination_archival::ownership::{
    reduce_model_only, Scope, State, Transition,
};
use free_energy_coordination_archival::{
    replay, Basis, ReplayCut, ReplayFailure, ReplayRecord,
};

fn transition(
    position: u64,
    run: &str,
    seq: u64,
    state: State,
    prev: Option<u64>,
    branch: &str,
) -> Transition {
    Transition {
        position,
        comment_id: 1000 + position,
        run: run.to_owned(),
        seq,
        state,
        prev,
        scope: Scope {
            issue: Some(22),
            pr: None,
            branch: Some(branch.to_owned()),
            seam: branch.to_owned(),
        },
    }
}

fn transitions() -> Vec<Transition> {
    vec![
        transition(1, "a", 1, State::Intent, None, "alpha"),
        transition(2, "a", 2, State::Owned, Some(1001), "alpha"),
        transition(3, "a", 3, State::Working, Some(1002), "alpha"),
        transition(4, "b", 1, State::Intent, None, "beta"),
        transition(5, "b", 2, State::Owned, Some(1004), "beta"),
        transition(6, "a", 4, State::Release, Some(1003), "alpha"),
        transition(7, "b", 3, State::Working, Some(1005), "beta"),
        transition(8, "b", 4, State::Handoff, Some(1007), "beta"),
        transition(9, "c", 1, State::Intent, None, "alpha"),
        transition(10, "c", 2, State::Owned, Some(1009), "alpha"),
        transition(11, "d", 1, State::Intent, None, "delta"),
        transition(12, "d", 2, State::Owned, Some(1011), "delta"),
        transition(13, "c", 3, State::Working, Some(1010), "alpha"),
        transition(14, "d", 3, State::Release, Some(1012), "delta"),
    ]
}

fn ordering() -> Basis {
    Basis {
        identity: "certified-synthetic-append-order".to_owned(),
        generation: 3,
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: ordering(),
        manifest: Basis {
            identity: "synthetic-manifest-current".to_owned(),
            generation: 7,
        },
        source_incarnation: "synthetic-live-incarnation".to_owned(),
        first_sequence: 1,
        last_sequence: 14,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn record(event: &Transition) -> ReplayRecord {
    ReplayRecord {
        stable_id: format!("comment-{}", event.comment_id),
        sequence: event.position,
        // Synthetic byte marker, deliberately NOT a cryptographic digest.
        payload_digest: [(event.comment_id % 251) as u8; 32],
        order_basis: ordering(),
        source_incarnation: "synthetic-live-incarnation".to_owned(),
        source_version: format!("version-{}", event.comment_id),
    }
}

fn records(events: &[Transition]) -> Vec<ReplayRecord> {
    events.iter().map(record).collect()
}

#[test]
fn every_archive_live_boundary_preserves_the_real_ownership_reducer_result() {
    let canonical_events = transitions();
    let canonical_records = records(&canonical_events);
    let expected = reduce_model_only(&canonical_events).expect("valid canonical transitions");
    assert_eq!(expected.len(), 1);
    assert_eq!(expected[0].run, "c");
    assert_eq!(expected[0].state, State::Working);
    assert_eq!(expected[0].scope.branch.as_deref(), Some("alpha"));

    // Every possible cut, with up to three entries duplicated across tiers.
    // Reverse both page orders to ensure neither page order governs reduction.
    for boundary in 0..=canonical_records.len() {
        let overlap_start = boundary.saturating_sub(3);
        let mut archive = canonical_records[..boundary].to_vec();
        let mut live = canonical_records[overlap_start..].to_vec();
        archive.reverse();
        live.reverse();

        let reconstructed = replay(&archive, &live, &cut())
            .expect("all coherent complete model cuts should replay");
        assert_eq!(reconstructed, canonical_records, "boundary={boundary}");

        // Simulate the adapter binding each exact replay record to a typed
        // transition. This test makes NO authenticity claim about that adapter.
        let reconstructed_events: Vec<_> = reconstructed
            .iter()
            .map(|item| {
                let index = usize::try_from(item.sequence).unwrap() - 1;
                let event = &canonical_events[index];
                assert_eq!(*item, record(event));
                event.clone()
            })
            .collect();
        assert_eq!(
            reduce_model_only(&reconstructed_events),
            Ok(expected.clone()),
            "reducer changed after archive/live cut boundary={boundary}"
        );
    }
}

#[test]
fn omitted_released_owner_cannot_be_resurrected_by_a_partial_two_tier_cut() {
    let canonical = records(&transitions());
    let mut archive = canonical[..8].to_vec();
    let mut live = canonical[5..].to_vec();

    // Drop the RELEASE from both tiers. A naive stitched history could
    // incorrectly keep run a active even after c begins using branch alpha.
    archive.retain(|item| item.sequence != 6);
    live.retain(|item| item.sequence != 6);
    assert_eq!(
        replay(&archive, &live, &cut()),
        Err(ReplayFailure::IncompleteHistory)
    );

    // Restoring one valid archived copy recovers exactly one canonical event.
    archive.insert(5, canonical[5].clone());
    assert_eq!(replay(&archive, &live, &cut()), Ok(canonical));
}

#[test]
fn conflicted_overlap_cannot_feed_the_real_ownership_reducer() {
    let canonical = records(&transitions());
    let archive = canonical[..8].to_vec();
    let mut live = canonical[5..].to_vec();
    live[0].source_version = "replacement-with-same-payload".to_owned();
    assert_eq!(
        replay(&archive, &live, &cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );

    let mut live = canonical[5..].to_vec();
    live[0].payload_digest = [0x55; 32];
    assert_eq!(
        replay(&archive, &live, &cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );
}

#[test]
fn stale_manifest_order_frontier_or_snapshot_cannot_produce_an_owner() {
    let canonical = records(&transitions());
    for downgraded in 0..5 {
        let mut invalid = cut();
        match downgraded {
            0 => invalid.authoritative_order = false,
            1 => invalid.current_manifest = false,
            2 => invalid.coherent_snapshot = false,
            3 => invalid.complete_frontier = false,
            4 => invalid.closing_fence_current = false,
            _ => unreachable!(),
        }
        assert_eq!(
            replay(&canonical, &[], &invalid),
            Err(ReplayFailure::UntrustedCut),
            "downgrade={downgraded}"
        );
    }
    let mut changed_incarnation = canonical;
    changed_incarnation[0].source_incarnation = "restored-clone".to_owned();
    assert_eq!(
        replay(&changed_incarnation, &[], &cut()),
        Err(ReplayFailure::UntrustedCut)
    );
}
