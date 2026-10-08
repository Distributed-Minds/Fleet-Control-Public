//! Large, deterministic, offline archival/live replay stress tests (issue #22).
//!
//! These test the compiled pure model, not provider authority, durable storage,
//! mutation/compaction permission, or production reducer equivalence.

use free_energy_coordination_archival::{replay, Basis, ReplayCut, ReplayFailure, ReplayRecord};

fn basis() -> Basis {
    Basis {
        identity: "authoritative-order-opaque".to_owned(),
        generation: 12,
    }
}

fn record(sequence: u64) -> ReplayRecord {
    // Opaque IDs deliberately do not have the same lexicographic order as
    // authoritative positions. Their contents cannot supply reducer order.
    ReplayRecord {
        stable_id: format!("opaque-{:020}", sequence.wrapping_mul(31) % 100_003),
        sequence,
        payload_digest: [sequence as u8; 32],
        order_basis: basis(),
        source_incarnation: "live-incarnation-9".to_owned(),
    }
}

fn cut(first: u64, last: u64) -> ReplayCut {
    ReplayCut {
        ordering: basis(),
        source_incarnation: "live-incarnation-9".to_owned(),
        first_sequence: first,
        last_sequence: last,
        authoritative_order: true,
        current_manifest: true,
        coherent_snapshot: true,
        complete_frontier: true,
        closing_fence_current: true,
    }
}

fn streams() -> (Vec<ReplayRecord>, Vec<ReplayRecord>) {
    // 5,000 distinct positions, with 200 exact duplicates across a coherent
    // archive/live overlap. Neither provider may be assumed to return sorted data.
    let archive = (10..3510).map(record).collect();
    let live = (3310..=5009).map(record).collect();
    (archive, live)
}

fn shuffle<T>(items: &mut [T], mut seed: usize) {
    for i in (1..items.len()).rev() {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        items.swap(i, seed % (i + 1));
    }
}

#[test]
fn five_thousand_records_reconstruct_identically_despite_provider_page_order() {
    let expected: Vec<_> = (10..=5009).map(record).collect();
    for seed in [1, 2, 3, 5, 89, 98765, 7654321, 99991] {
        let (mut archive, mut live) = streams();
        shuffle(&mut archive, seed);
        shuffle(&mut live, seed + 17);
        let reconstructed = replay(&archive, &live, &cut(10, 5009))
            .expect("same coherent exact history must reconstruct");
        assert_eq!(
            reconstructed, expected,
            "provider page ordering seed {seed}"
        );
    }
}

#[test]
fn a_single_missing_authoritative_position_fails_even_in_large_history() {
    for missing in [10, 11, 1999, 3200, 3509, 3510, 5008, 5009] {
        let (mut archive, mut live) = streams();
        archive.retain(|r| r.sequence != missing);
        live.retain(|r| r.sequence != missing);
        shuffle(&mut archive, 31);
        shuffle(&mut live, 97);
        assert_eq!(
            replay(&archive, &live, &cut(10, 5009)),
            Err(ReplayFailure::IncompleteHistory),
            "missing authoritative position {missing}"
        );
    }
}

#[test]
fn overlapping_conflicting_bytes_and_duplicate_ids_cannot_hide_in_reordered_pages() {
    let (mut archive, mut live) = streams();
    // 3400 appears in BOTH archive and live. A stale/forked copy cannot be
    // silently resolved by provider order or a convenient winning source.
    let conflicting = live.iter_mut().find(|r| r.sequence == 3400).unwrap();
    conflicting.payload_digest = [255; 32];
    shuffle(&mut archive, 123);
    shuffle(&mut live, 456);
    assert_eq!(
        replay(&archive, &live, &cut(10, 5009)),
        Err(ReplayFailure::ConflictingDuplicate)
    );

    let (archive, mut live) = streams();
    live.iter_mut()
        .find(|r| r.sequence == 3400)
        .unwrap()
        .stable_id = "a-different-record-at-the-same-position".to_owned();
    assert_eq!(
        replay(&archive, &live, &cut(10, 5009)),
        Err(ReplayFailure::ConflictingDuplicate)
    );
}

#[test]
fn stale_order_basis_or_source_incarnation_is_not_healed_by_complete_contents() {
    let (mut archive, live) = streams();
    archive[1].order_basis.generation += 1;
    assert_eq!(
        replay(&archive, &live, &cut(10, 5009)),
        Err(ReplayFailure::UntrustedCut)
    );
    let (mut archive, live) = streams();
    archive[1].source_incarnation = "restored-incarnation".to_owned();
    assert_eq!(
        replay(&archive, &live, &cut(10, 5009)),
        Err(ReplayFailure::UntrustedCut)
    );
}

#[test]
fn near_maximum_u64_frontier_replays_without_wrap_or_guessed_positions() {
    let first = u64::MAX - 15;
    let full: Vec<_> = (first..=u64::MAX).map(record).collect();
    let reconstructed = replay(&full[..9], &full[7..], &cut(first, u64::MAX))
        .expect("exact contiguous high positions must be reconstructible");
    assert_eq!(reconstructed, full);

    let mut missing = full;
    missing.remove(11);
    assert_eq!(
        replay(&missing, &[], &cut(first, u64::MAX)),
        Err(ReplayFailure::IncompleteHistory)
    );
}

#[test]
fn an_enormous_declared_frontier_cannot_manufacture_unobserved_records() {
    let (archive, live) = streams();
    assert_eq!(
        replay(&archive, &live, &cut(0, u64::MAX)),
        Err(ReplayFailure::IncompleteHistory)
    );
}
