//! Deterministic reducer-relevant ordering controls for issue #22 / spec 5.
//!
//! This is a SYNTHETIC state reducer over model records. It proves that the
//! compiled replay preserves order-sensitive results across a coherent
//! archive/live overlap, not parity with a real Fleet reducer or authority to
//! delete a provider record. An untrusted cut must fail before reduction.

use free_energy_coordination_archival::{replay, Basis, ReplayCut, ReplayFailure, ReplayRecord};

fn order_basis() -> Basis {
    Basis {
        identity: "provider-certified-order".to_owned(),
        generation: 7,
    }
}

// Payload bytes below encode toy transition kinds only, not real event digests.
// 1 = A acquires; 2 = B acquires; 3 = A releases; 4 = B releases.
fn event(position: u64, kind: u8) -> ReplayRecord {
    ReplayRecord {
        stable_id: format!("opaque-event-{position}"),
        sequence: position,
        payload_digest: [kind; 32],
        order_basis: order_basis(),
        source_incarnation: "live-provider-incarnation-4".to_owned(),
        source_version: "fixture-version-v1".to_owned(),
    }
}

fn history() -> Vec<ReplayRecord> {
    vec![
        event(100, 1),
        event(101, 2),
        event(102, 3),
        event(103, 1),
        event(104, 3),
        event(105, 2),
    ]
}

fn coherent_cut() -> ReplayCut {
    ReplayCut {
        ordering: order_basis(),
        source_incarnation: "live-provider-incarnation-4".to_owned(),
        manifest: order_basis(),
        first_sequence: 100,
        last_sequence: 105,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

// A deliberately order-sensitive synthetic reducer: releasing a non-owner
// must not revoke the actual current owner. Neither source page order nor
// opaque identity lexicography may choose the winner.
fn synthetic_current_owner(records: &[ReplayRecord]) -> Option<char> {
    let mut owner = None;
    for record in records {
        match record.payload_digest[0] {
            1 => owner = Some('A'),
            2 => owner = Some('B'),
            3 if owner == Some('A') => owner = None,
            4 if owner == Some('B') => owner = None,
            _ => {}
        }
    }
    owner
}

#[test]
fn toy_ownership_is_observably_sensitive_to_event_order() {
    let chronological = history();
    let reversed: Vec<_> = chronological.iter().cloned().rev().collect();
    assert_eq!(synthetic_current_owner(&chronological), Some('B'));
    assert_eq!(synthetic_current_owner(&reversed), Some('A'));
}

#[test]
fn coherent_archive_live_replay_preserves_synthetic_reducer_outcome() {
    let canonical = history();
    for (mut archive, mut live) in [
        (canonical[..4].to_vec(), canonical[2..].to_vec()),
        (canonical[..2].to_vec(), canonical[1..].to_vec()),
        (canonical.clone(), Vec::new()),
        (Vec::new(), canonical.clone()),
        (canonical[..5].to_vec(), canonical[3..].to_vec()),
    ] {
        // Deliberately make provider read/page order disagree with authoritative
        // positions; exact overlap must be deduplicated, not applied twice.
        archive.reverse();
        live.reverse();
        let restored = replay(&archive, &live, &coherent_cut())
            .expect("coherent and complete model histories must replay");
        assert_eq!(restored, canonical);
        assert_eq!(synthetic_current_owner(&restored), Some('B'));
    }
}

#[test]
fn omitted_or_forked_transition_cannot_manufacture_a_reducer_winner() {
    let canonical = history();
    let missing_release: Vec<_> = canonical
        .iter()
        .filter(|record| record.sequence != 104)
        .cloned()
        .collect();
    assert_eq!(
        replay(&missing_release, &[], &coherent_cut()),
        Err(ReplayFailure::IncompleteHistory)
    );

    let mut forged = canonical[3].clone();
    forged.payload_digest = [2; 32];
    assert_eq!(
        replay(&canonical[..4], &[forged], &coherent_cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );

    let mut reused_identity = canonical[0].clone();
    reused_identity.sequence = 105;
    assert_eq!(
        replay(&canonical[..5], &[reused_identity], &coherent_cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );
}

#[test]
fn an_incoherent_or_stale_cut_cannot_reach_the_synthetic_reducer() {
    let canonical = history();
    for downgrade in 0..5 {
        let mut cut = coherent_cut();
        match downgrade {
            0 => cut.authoritative_order = false,
            1 => cut.current_manifest = false,
            2 => cut.coherent_snapshot = false,
            3 => cut.complete_frontier = false,
            4 => cut.closing_fence_current = false,
            _ => unreachable!(),
        }
        assert_eq!(
            replay(&canonical, &[], &cut),
            Err(ReplayFailure::UntrustedCut),
            "untrusted cut variant {downgrade} must stop before replay"
        );
    }

    let mut stale = canonical;
    stale[0].order_basis.generation += 1;
    assert_eq!(
        replay(&stale, &[], &coherent_cut()),
        Err(ReplayFailure::UntrustedCut)
    );
}

#[test]
fn blank_identity_or_moved_source_rejects_a_complete_toy_history() {
    let mut blank = history();
    blank[0].stable_id = " ".to_owned();
    assert_eq!(
        replay(&blank, &[], &coherent_cut()),
        Err(ReplayFailure::UntrustedCut)
    );

    let mut moved = history();
    moved[0].source_incarnation = "restored-provider-instance".to_owned();
    assert_eq!(
        replay(&moved, &[], &coherent_cut()),
        Err(ReplayFailure::UntrustedCut)
    );
}
