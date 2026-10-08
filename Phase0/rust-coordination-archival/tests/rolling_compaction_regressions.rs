//! Consecutive, purely model-level archive/live cutover regressions (#22).
//! No fixture or model verdict proves provider authority or permits deletion.

use free_energy_coordination_archival::{replay, Basis, ReplayCut, ReplayFailure, ReplayRecord};

fn record(sequence: u64) -> ReplayRecord {
    ReplayRecord {
        // Deliberately different from sortable sequence text.
        stable_id: format!("opaque-event-{}", sequence.wrapping_mul(37) % 997),
        sequence,
        payload_digest: [sequence as u8; 32],
        order_basis: Basis {
            identity: "provider-certified-order".into(),
            generation: 9,
        },
        source_incarnation: "live-store-incarnation-4".into(),
    }
}

fn cut() -> ReplayCut {
    ReplayCut {
        ordering: Basis {
            identity: "provider-certified-order".into(),
            generation: 9,
        },
        source_incarnation: "live-store-incarnation-4".into(),
        first_sequence: 1,
        last_sequence: 128,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn records(first: u64, last: u64) -> Vec<ReplayRecord> {
    (first..=last).map(record).collect()
}

#[test]
fn successive_archive_first_cutovers_preserve_exact_reducer_order() {
    let expected = records(1, 128);
    // Model consecutive archival generations: old live data can overlap the
    // newly archived prefix while the durable live frontier remains complete.
    for archive_end in [0_u64, 1, 2, 32, 64, 95, 127, 128] {
        let mut archived = records(1, archive_end);
        let live_start = archive_end.saturating_sub(3).max(1);
        let mut live = records(live_start, 128);
        // Neither tier's retrieval/page order is an ordering authority.
        archived.reverse();
        live.reverse();
        assert_eq!(
            replay(&archived, &live, &cut()),
            Ok(expected.clone()),
            "coherent cutover at archive sequence {archive_end}"
        );
    }
}

#[test]
fn mixed_archive_live_cut_fails_even_with_individually_complete_tiers() {
    // One observer sees the old archived frontier, another sees a later live
    // frontier following deletion. The assembled view omits positions 65..67.
    let archived = records(1, 64);
    let live = records(68, 128);
    assert_eq!(
        replay(&archived, &live, &cut()),
        Err(ReplayFailure::IncompleteHistory)
    );
    // A claimed current manifest or coherent snapshot flag cannot synthesize
    // missing reducer positions from a stale cross-tier observation.
    let mut old_manifest = cut();
    old_manifest.current_manifest = false;
    assert_eq!(
        replay(&archived, &live, &old_manifest),
        Err(ReplayFailure::UntrustedCut)
    );
}

#[test]
fn overlapping_cutover_rejects_conflicting_bytes_or_identity() {
    let archived = records(1, 64);
    let mut live = records(62, 128);
    // The same reducer position has different bytes after archive-first
    // publication: do not choose whichever copy arrived last.
    live[1].payload_digest[0] ^= 1;
    assert_eq!(
        replay(&archived, &live, &cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );

    let mut live = records(62, 128);
    live[1].stable_id = "different-incarnation-of-the-event".into();
    assert_eq!(
        replay(&archived, &live, &cut()),
        Err(ReplayFailure::ConflictingDuplicate)
    );
}

#[test]
fn cutover_does_not_justify_stale_order_or_reused_source_incarnation() {
    let archived = records(1, 64);
    let mut live = records(62, 128);
    live[0].order_basis.generation += 1;
    assert_eq!(
        replay(&archived, &live, &cut()),
        Err(ReplayFailure::UntrustedCut)
    );
    let mut live = records(62, 128);
    live[0].source_incarnation = "restored-live-store".into();
    assert_eq!(
        replay(&archived, &live, &cut()),
        Err(ReplayFailure::UntrustedCut)
    );
}
