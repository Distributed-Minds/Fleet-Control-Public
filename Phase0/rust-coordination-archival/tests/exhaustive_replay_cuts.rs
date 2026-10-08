//! Exhaustive small-history archival cut tests for #22 (specification v5).
//!
//! Every record independently belongs to neither tier, archive only, live
//! only, or both. This enumerates all 4^7 layouts; the expected outcome is
//! derived from the input allocation, not the model's own verdict labels.
//! Pure Rust model tests: no provider, deletion, or authority execution.

use free_energy_coordination_archival::{replay, Basis, ReplayCut, ReplayFailure, ReplayRecord};

const FIRST: u64 = 20;
const COUNT: usize = 7;
const LAYOUTS: usize = 1 << (COUNT * 2);

fn basis() -> Basis {
    Basis {
        identity: "certified-reducer-order-17".to_owned(),
        generation: 17,
    }
}

fn record(index: usize) -> ReplayRecord {
    ReplayRecord {
        // Deliberately not sorted in reducer sequence order.
        stable_id: format!("opaque-{}", (index * 13 + 7) % 97),
        sequence: FIRST + index as u64,
        payload_digest: [(index + 1) as u8; 32],
        order_basis: basis(),
        source_incarnation: "source-generation-9".to_owned(),
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: basis(),
        source_incarnation: "source-generation-9".to_owned(),
        first_sequence: FIRST,
        last_sequence: FIRST + COUNT as u64 - 1,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn inputs(layout: usize) -> (Vec<ReplayRecord>, Vec<ReplayRecord>, bool) {
    let mut archived = Vec::new();
    let mut live = Vec::new();
    let mut complete = true;
    for index in 0..COUNT {
        let disposition = (layout >> (index * 2)) & 3;
        if disposition & 1 != 0 {
            archived.push(record(index));
        }
        if disposition & 2 != 0 {
            live.push(record(index));
        }
        if disposition == 0 {
            complete = false;
        }
    }
    // Provider pages are not the reducer's ordering authority.
    archived.reverse();
    if layout & 1 != 0 {
        live.reverse();
    }
    (archived, live, complete)
}

#[test]
fn all_seven_position_archive_live_allocations_have_the_predicted_replay_result() {
    let expected: Vec<_> = (0..COUNT).map(record).collect();
    let mut reconstructed = 0;
    let mut rejected_gaps = 0;
    for layout in 0..LAYOUTS {
        let (archived, live, complete) = inputs(layout);
        let actual = replay(&archived, &live, &cut());
        if complete {
            assert_eq!(
                actual,
                Ok(expected.clone()),
                "complete placement {layout:#06x} changed certified history"
            );
            reconstructed += 1;
        } else {
            assert_eq!(
                actual,
                Err(ReplayFailure::IncompleteHistory),
                "missing position passed layout {layout:#06x}"
            );
            rejected_gaps += 1;
        }
    }
    // Each of seven positions has three non-missing placements:
    // archive only, live only, or byte-identical in both.
    assert_eq!(reconstructed, 3_usize.pow(COUNT as u32));
    assert_eq!(reconstructed + rejected_gaps, LAYOUTS);
}

#[test]
fn every_exact_overlap_rejects_one_byte_of_forked_payload_in_either_tier() {
    let canonical: Vec<_> = (0..COUNT).map(record).collect();
    for position in 0..COUNT {
        for corrupt_archived in [false, true] {
            let mut archived = canonical.clone();
            let mut live = canonical.clone();
            if corrupt_archived {
                archived[position].payload_digest[0] ^= 1;
            } else {
                live[position].payload_digest[0] ^= 1;
            }
            archived.reverse();
            assert_eq!(
                replay(&archived, &live, &cut()),
                Err(ReplayFailure::ConflictingDuplicate),
                "forked overlap at position {position}, archived={corrupt_archived}"
            );
        }
    }
}

#[test]
fn one_identity_reused_across_different_positions_cannot_erase_history() {
    let canonical: Vec<_> = (0..COUNT).map(record).collect();
    for reused_at in 1..COUNT {
        let mut live = canonical.clone();
        live[reused_at].stable_id = live[0].stable_id.clone();
        assert_eq!(
            replay(&canonical, &live, &cut()),
            Err(ReplayFailure::ConflictingDuplicate),
            "opaque identity reused at index {reused_at}"
        );
    }
}

#[test]
fn stale_provenance_is_rejected_even_when_all_payloads_and_positions_match() {
    let canonical: Vec<_> = (0..COUNT).map(record).collect();
    for stale_at in 0..COUNT {
        let mut live = canonical.clone();
        live[stale_at].order_basis.generation += 1;
        assert_eq!(replay(&canonical, &live, &cut()), Err(ReplayFailure::UntrustedCut));
        live[stale_at] = canonical[stale_at].clone();
        live[stale_at].source_incarnation = "stale-store-incarnation".into();
        assert_eq!(replay(&canonical, &live, &cut()), Err(ReplayFailure::UntrustedCut));
    }
}
