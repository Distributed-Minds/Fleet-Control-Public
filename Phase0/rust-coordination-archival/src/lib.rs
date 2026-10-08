//! Pure coordination-history archive admission and coherent replay model.
//!
//! This module deliberately has NO provider client and NEVER grants mutation
//! authority. Its inputs must be populated by independently trusted, current
//! and appropriately fenced adapters (Phase0 #10 / #22). Booleans in a caller
//! owned struct, local files, GitHub comments, or this model's test fixtures
//! are NOT authority receipts. An Eligible model decision is not a DELETE token.

use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub record_id: String,
    pub incarnation: String,
    pub version: String,
    pub content_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Basis {
    pub identity: String,
    pub generation: u64,
}

#[derive(Debug, Clone)]
pub struct Archive {
    pub source: Source,
    pub manifest: Basis,
    pub segment: String,
    pub manifest_digest: [u8; 32],
    pub remotely_read_digest: [u8; 32],
    pub exact_remote_readback: bool,
}

#[derive(Debug, Clone)]
pub struct Ordering {
    pub basis: Basis,
    pub source_incarnation: String,
    pub authoritative: bool,
    pub frontier_complete: bool,
    pub current: bool,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub basis: Basis,
    pub destination_incarnation: String,
    pub unique_current_selection: bool,
    pub predecessor_transition_fenced: bool,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub manifest: Basis,
    pub ordering: Basis,
    pub source_incarnation: String,
    pub coherent_cut: bool,
    pub frontier_complete: bool,
    pub closing_fence_current: bool,
    pub stable_identity_dedup: bool,
}

#[derive(Debug, Clone)]
pub struct Authority {
    pub operation_id: String,
    pub source: Source,
    pub action: String,
    pub current: bool,
    pub fence_current: bool,
}

#[derive(Debug, Clone)]
pub struct Horizon {
    pub basis: Basis,
    pub policy_source_incarnation: String,
    pub uniquely_current: bool,
    pub transition_authorized: bool,
    pub preserve_through_epoch: u64,
    // A successor policy may not silently release predecessor-protected data.
    pub predecessor_protects_source: bool,
    pub exact_retirement_authorized: bool,
}

#[derive(Debug, Clone)]
pub struct Durability {
    pub manifest: Basis,
    pub segment: String,
    pub destination_incarnation: String,
    pub horizon: Basis,
    pub reconstructed_through_epoch: u64,
    pub current: bool,
    pub keys_recoverable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Protections {
    pub active_ownership_chain: bool,
    pub latest_persistent_state: bool,
    pub live_tail: bool,
    pub authoritative_reference: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectState {
    NotAttempted,
    AckUnknown,
    PartialEffect,
    AlreadyApplied,
}

#[derive(Debug, Clone)]
pub struct DeleteWitness {
    pub operation_id: String,
    pub observed_source: Source,
    pub archive: Archive,
    pub ordering: Ordering,
    pub manifest: Manifest,
    pub snapshot: Snapshot,
    pub authority: Authority,
    pub horizon: Horizon,
    pub durability: Durability,
    pub protections: Protections,
    pub effect_state: EffectState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Denial {
    ArchiveNotExact,
    OrderNotProven,
    ManifestNotCurrent,
    SnapshotNotCoherent,
    SourceMoved,
    AuthorityNotCurrent,
    HorizonNotAuthorized,
    DurabilityInsufficient,
    ProtectedRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    EligibleModelOnly,
    Ineligible(Denial),
    ReconcilePriorEffect,
}

fn named(id: &str) -> bool {
    // Provider IDs are opaque, but they appear in operator receipts and logs.
    // C0/C1 controls, Unicode whitespace, bidi and default-ignorable
    // formatting may spoof identity in receipts or operator logs.
    // Reject such characters without normalizing opaque provider IDs.
    !id.trim().is_empty()
        && !id.chars().any(|ch| {
            ch.is_control()
                || (!ch.is_ascii() && ch.is_whitespace())
                || matches!(
                    ch,
                    '\u{00ad}'
                        | '\u{034f}'
                        | '\u{061c}'
                        | '\u{180e}'
                        | '\u{200b}'..='\u{200f}'
                        | '\u{202a}'..='\u{202e}'
                        | '\u{2060}'..='\u{206f}'
                        | '\u{feff}'
                )
        })
}

/// Deterministic, side-effect-free AND composition of independent bases.
/// The caller must still perform exact effect-time revalidation or execute
/// under one trusted serialization primitive. NEVER interpret this result
/// as permitting a live provider-side deletion by itself.
pub fn evaluate(w: &DeleteWitness) -> Verdict {
    use Denial::*;
    if w.effect_state != EffectState::NotAttempted {
        return Verdict::ReconcilePriorEffect;
    }
    if !named(&w.archive.segment)
        || !named(&w.archive.manifest.identity)
        || !w.archive.exact_remote_readback
        || w.archive.source != w.observed_source
        || w.archive.manifest_digest != w.archive.remotely_read_digest
    {
        return Verdict::Ineligible(ArchiveNotExact);
    }
    if !named(&w.ordering.basis.identity)
        || !w.ordering.authoritative
        || !w.ordering.frontier_complete
        || !w.ordering.current
        || w.ordering.source_incarnation != w.observed_source.incarnation
    {
        return Verdict::Ineligible(OrderNotProven);
    }
    if !named(&w.manifest.basis.identity)
        || !named(&w.manifest.destination_incarnation)
        || !w.manifest.unique_current_selection
        || !w.manifest.predecessor_transition_fenced
        || w.archive.manifest != w.manifest.basis
    {
        return Verdict::Ineligible(ManifestNotCurrent);
    }
    if w.snapshot.manifest != w.manifest.basis
        || w.snapshot.ordering != w.ordering.basis
        || w.snapshot.source_incarnation != w.observed_source.incarnation
        || !w.snapshot.coherent_cut
        || !w.snapshot.frontier_complete
        || !w.snapshot.closing_fence_current
        || !w.snapshot.stable_identity_dedup
    {
        return Verdict::Ineligible(SnapshotNotCoherent);
    }
    if !named(&w.observed_source.record_id)
        || !named(&w.observed_source.incarnation)
        || !named(&w.observed_source.version)
    {
        return Verdict::Ineligible(SourceMoved);
    }
    if !named(&w.operation_id)
        || !w.authority.current
        || !w.authority.fence_current
        || w.authority.action != "DELETE_SOURCE_RECORD"
        || w.authority.operation_id != w.operation_id
        || w.authority.source != w.observed_source
    {
        return Verdict::Ineligible(AuthorityNotCurrent);
    }
    if !named(&w.horizon.basis.identity)
        || !named(&w.horizon.policy_source_incarnation)
        || !w.horizon.uniquely_current
        || !w.horizon.transition_authorized
        || (w.horizon.predecessor_protects_source && !w.horizon.exact_retirement_authorized)
    {
        return Verdict::Ineligible(HorizonNotAuthorized);
    }
    if !w.durability.current
        || !w.durability.keys_recoverable
        || w.durability.manifest != w.manifest.basis
        || w.durability.segment != w.archive.segment
        || w.durability.destination_incarnation != w.manifest.destination_incarnation
        || w.durability.horizon != w.horizon.basis
        || w.durability.reconstructed_through_epoch < w.horizon.preserve_through_epoch
    {
        return Verdict::Ineligible(DurabilityInsufficient);
    }
    let p = &w.protections;
    if p.active_ownership_chain
        || p.latest_persistent_state
        || p.live_tail
        || p.authoritative_reference
    {
        return Verdict::Ineligible(ProtectedRecord);
    }
    Verdict::EligibleModelOnly
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRecord {
    pub stable_id: String,
    pub sequence: u64,
    pub payload_digest: [u8; 32],
    pub order_basis: Basis,
    pub source_incarnation: String,
}

#[derive(Debug, Clone)]
pub struct ReplayCut {
    pub ordering: Basis,
    pub source_incarnation: String,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub authoritative_order: bool,
    pub current_manifest: bool,
    pub coherent_snapshot: bool,
    pub complete_frontier: bool,
    pub closing_fence_current: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFailure {
    UntrustedCut,
    ConflictingDuplicate,
    IncompleteHistory,
}

/// Order comes ONLY from certified reducer positions, not opaque IDs,
/// timestamps, archive order or the order returned by a provider. The caller
/// must prove a coherent archive/live cut and complete sequence frontier.
/// Exact overlap is harmless; a conflicting duplicate or gap fails closed.
pub fn replay(
    archived: &[ReplayRecord],
    live: &[ReplayRecord],
    cut: &ReplayCut,
) -> Result<Vec<ReplayRecord>, ReplayFailure> {
    if !cut.authoritative_order
        || !cut.current_manifest
        || !cut.coherent_snapshot
        || !cut.complete_frontier
        || !cut.closing_fence_current
        || !named(&cut.ordering.identity)
        || !named(&cut.source_incarnation)
        || cut.first_sequence > cut.last_sequence
    {
        return Err(ReplayFailure::UntrustedCut);
    }

    let mut by_sequence = BTreeMap::<u64, ReplayRecord>::new();
    let mut by_identity = HashMap::<&str, u64>::new();
    for record in archived.iter().chain(live) {
        if !named(&record.stable_id)
            || record.order_basis != cut.ordering
            || record.source_incarnation != cut.source_incarnation
        {
            return Err(ReplayFailure::UntrustedCut);
        }
        if record.sequence < cut.first_sequence || record.sequence > cut.last_sequence {
            return Err(ReplayFailure::IncompleteHistory);
        }
        if let Some(old_sequence) = by_identity.insert(&record.stable_id, record.sequence) {
            if old_sequence != record.sequence {
                return Err(ReplayFailure::ConflictingDuplicate);
            }
        }
        match by_sequence.get(&record.sequence) {
            Some(old) if old != record => return Err(ReplayFailure::ConflictingDuplicate),
            Some(_) => {}
            None => {
                by_sequence.insert(record.sequence, record.clone());
            }
        }
    }

    // Check gaps using observed neighbors, never iterate an untrusted massive
    // first..last range. This is bounded by actual retained records.
    let mut previous: Option<u64> = None;
    for sequence in by_sequence.keys().copied() {
        if let Some(prev) = previous {
            if prev.checked_add(1) != Some(sequence) {
                return Err(ReplayFailure::IncompleteHistory);
            }
        } else if sequence != cut.first_sequence {
            return Err(ReplayFailure::IncompleteHistory);
        }
        previous = Some(sequence);
    }
    if previous != Some(cut.last_sequence) {
        return Err(ReplayFailure::IncompleteHistory);
    }
    Ok(by_sequence.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis(id: &str) -> Basis {
        Basis {
            identity: id.to_owned(),
            generation: 4,
        }
    }

    fn fixture() -> DeleteWitness {
        let source = Source {
            record_id: "R42".into(),
            incarnation: "live-1".into(),
            version: "etag-2".into(),
            content_digest: [7; 32],
        };
        let manifest = basis("manifest-M2");
        let horizon = basis("horizon-H2");
        DeleteWitness {
            operation_id: "delete-R42".into(),
            observed_source: source.clone(),
            archive: Archive {
                source: source.clone(),
                manifest: manifest.clone(),
                segment: "segment-4".into(),
                manifest_digest: [8; 32],
                remotely_read_digest: [8; 32],
                exact_remote_readback: true,
            },
            ordering: Ordering {
                basis: basis("order-O1"),
                source_incarnation: "live-1".into(),
                authoritative: true,
                frontier_complete: true,
                current: true,
            },
            manifest: Manifest {
                basis: manifest.clone(),
                destination_incarnation: "archive-store-1".into(),
                unique_current_selection: true,
                predecessor_transition_fenced: true,
            },
            snapshot: Snapshot {
                manifest: manifest.clone(),
                ordering: basis("order-O1"),
                source_incarnation: "live-1".into(),
                coherent_cut: true,
                frontier_complete: true,
                closing_fence_current: true,
                stable_identity_dedup: true,
            },
            authority: Authority {
                operation_id: "delete-R42".into(),
                source: source.clone(),
                action: "DELETE_SOURCE_RECORD".into(),
                current: true,
                fence_current: true,
            },
            horizon: Horizon {
                basis: horizon.clone(),
                policy_source_incarnation: "policy-root-1".into(),
                uniquely_current: true,
                transition_authorized: true,
                preserve_through_epoch: 100,
                predecessor_protects_source: false,
                exact_retirement_authorized: false,
            },
            durability: Durability {
                manifest,
                segment: "segment-4".into(),
                destination_incarnation: "archive-store-1".into(),
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
    fn control_bytes_cannot_admit_archival_or_authority_identifiers() {
        for bad in [
            "record\n42",
            "record\r42",
            "record\0id",
            "record\u{007f}id",
            "record\u{0085}id",
        ] {
            // Make *every* copy of the source ID agree to prove that this
            // rejection is about unsafe identifiers, not a version mismatch.
            let mut witness = fixture();
            witness.observed_source.record_id = bad.into();
            witness.archive.source.record_id = bad.into();
            witness.authority.source.record_id = bad.into();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::SourceMoved),
                "unsafe source ID admitted: {bad:?}"
            );

            let mut witness = fixture();
            witness.operation_id = bad.into();
            witness.authority.operation_id = bad.into();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::AuthorityNotCurrent),
                "unsafe operation ID admitted: {bad:?}"
            );

            let mut witness = fixture();
            witness.ordering.basis.identity = bad.into();
            witness.snapshot.ordering.identity = bad.into();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::OrderNotProven),
                "unsafe ordering ID admitted: {bad:?}"
            );
        }
    }

    #[test]
    fn invisible_formatting_cannot_spoof_archival_and_replay_identifiers() {
        // These characters are not C0/C1 controls, and several are not bidi
        // controls, yet an ID with one embedded can be visually indistinguishable
        // from a different exact source, manifest or operation identifier.
        for marker in [
            '\u{00ad}', '\u{034f}', '\u{180e}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}',
            '\u{2064}', '\u{feff}',
        ] {
            let bad = format!("R{marker}42");

            // Keep every independent source witness byte-identical: rejection
            // must come from unsafe presentation, not a mismatched version.
            let mut witness = fixture();
            witness.observed_source.record_id = bad.clone();
            witness.archive.source.record_id = bad.clone();
            witness.authority.source.record_id = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::SourceMoved),
                "invisible source ID admitted: {marker:?}"
            );

            let mut witness = fixture();
            witness.operation_id = bad.clone();
            witness.authority.operation_id = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::AuthorityNotCurrent),
                "invisible operation ID admitted: {marker:?}"
            );

            let mut witness = fixture();
            witness.archive.segment = bad.clone();
            witness.durability.segment = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::ArchiveNotExact),
                "invisible archive segment admitted: {marker:?}"
            );

            let mut witness = fixture();
            witness.horizon.basis.identity = bad.clone();
            witness.durability.horizon.identity = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::HorizonNotAuthorized),
                "invisible horizon basis admitted: {marker:?}"
            );

            let mut record = item("a", 10);
            record.stable_id = bad;
            assert_eq!(
                replay(&[record, item("b", 11)], &[item("c", 12)], &cut()),
                Err(ReplayFailure::UntrustedCut),
                "invisible replay identity admitted: {marker:?}"
            );
        }

        // Human-readable non-ASCII is not rejected merely for being Unicode.
        assert_eq!(evaluate(&fixture()), Verdict::EligibleModelOnly);
        let ordinary = item("histórico-λ", 10);
        assert_eq!(
            replay(&[ordinary.clone(), item("b", 11)], &[item("c", 12)], &cut()).unwrap()[0],
            ordinary
        );
    }

    #[test]
    fn bidi_formatting_cannot_spoof_replay_or_admission_identifiers() {
        for marker in [
            '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
            '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
        ] {
            let bad = format!("record{marker}42");
            // Identical forged source IDs in all independent inputs must
            // still fail; a mere mismatch test would not prove the guard.
            let mut witness = fixture();
            witness.observed_source.record_id = bad.clone();
            witness.archive.source.record_id = bad.clone();
            witness.authority.source.record_id = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::SourceMoved),
                "bidi source identity admitted: {marker:?}"
            );

            let mut witness = fixture();
            witness.operation_id = bad.clone();
            witness.authority.operation_id = bad.clone();
            assert_eq!(
                evaluate(&witness),
                Verdict::Ineligible(Denial::AuthorityNotCurrent),
                "bidi operation identity admitted: {marker:?}"
            );

            let mut forged = item("a", 10);
            forged.stable_id = bad;
            assert_eq!(
                replay(&[forged, item("b", 11)], &[item("c", 12)], &cut()),
                Err(ReplayFailure::UntrustedCut),
                "bidi replay record identity admitted: {marker:?}"
            );
        }
    }

    #[test]
    fn coherent_replay_rejects_control_bytes_in_stable_record_identity() {
        let mut archive = item("a", 10);
        archive.stable_id = "a\nforged".into();
        assert_eq!(
            replay(&[archive, item("b", 11)], &[item("c", 12)], &cut()),
            Err(ReplayFailure::UntrustedCut)
        );
        // Non-ASCII ordinary opaque identities remain valid.
        let ordinary = item("μ-opaque", 10);
        assert_eq!(
            replay(&[ordinary.clone(), item("b", 11)], &[item("c", 12)], &cut()).unwrap()[0],
            ordinary
        );
    }

    #[test]
    fn complete_consistent_model_is_eligible_only_as_model() {
        assert_eq!(evaluate(&fixture()), Verdict::EligibleModelOnly);
    }

    #[test]
    fn each_independent_base_is_required() {
        type NegativeCase = (fn(&mut DeleteWitness), Denial);
        let cases: [NegativeCase; 9] = [
            (
                |w| w.archive.exact_remote_readback = false,
                Denial::ArchiveNotExact,
            ),
            (
                |w| w.ordering.frontier_complete = false,
                Denial::OrderNotProven,
            ),
            (
                |w| w.manifest.unique_current_selection = false,
                Denial::ManifestNotCurrent,
            ),
            (
                |w| w.snapshot.coherent_cut = false,
                Denial::SnapshotNotCoherent,
            ),
            (
                |w| w.observed_source.version = "etag-3".into(),
                Denial::ArchiveNotExact,
            ),
            (|w| w.authority.current = false, Denial::AuthorityNotCurrent),
            (
                |w| w.horizon.uniquely_current = false,
                Denial::HorizonNotAuthorized,
            ),
            (
                |w| w.durability.keys_recoverable = false,
                Denial::DurabilityInsufficient,
            ),
            (
                |w| w.protections.active_ownership_chain = true,
                Denial::ProtectedRecord,
            ),
        ];
        for (change, reason) in cases {
            let mut w = fixture();
            change(&mut w);
            assert_eq!(evaluate(&w), Verdict::Ineligible(reason));
        }
    }

    #[test]
    fn cross_basis_and_effect_boundary_movement_fail_closed() {
        let cases: [fn(&mut DeleteWitness); 7] = [
            |w| w.snapshot.manifest.generation += 1,
            |w| w.snapshot.ordering.identity = "stale-order".into(),
            |w| w.durability.horizon.identity = "old-horizon".into(),
            |w| w.durability.destination_incarnation = "restored-store".into(),
            |w| w.authority.operation_id = "other-operation".into(),
            |w| w.ordering.source_incarnation = "old-live-store".into(),
            |w| w.manifest.predecessor_transition_fenced = false,
        ];
        for change in cases {
            let mut w = fixture();
            change(&mut w);
            assert_ne!(evaluate(&w), Verdict::EligibleModelOnly);
        }
    }

    #[test]
    fn unauthorized_horizon_downgrade_cannot_free_predecessor_records() {
        let mut w = fixture();
        w.horizon.predecessor_protects_source = true;
        assert_eq!(
            evaluate(&w),
            Verdict::Ineligible(Denial::HorizonNotAuthorized)
        );
        w.horizon.exact_retirement_authorized = true;
        assert_eq!(evaluate(&w), Verdict::EligibleModelOnly);
        w.horizon.uniquely_current = false;
        assert_eq!(
            evaluate(&w),
            Verdict::Ineligible(Denial::HorizonNotAuthorized)
        );
    }

    #[test]
    fn durability_must_reach_actual_current_authorized_horizon() {
        let mut w = fixture();
        w.horizon.preserve_through_epoch = 101;
        assert_eq!(
            evaluate(&w),
            Verdict::Ineligible(Denial::DurabilityInsufficient)
        );
        w.durability.reconstructed_through_epoch = 101;
        assert_eq!(evaluate(&w), Verdict::EligibleModelOnly);
        w.durability.current = false;
        assert_eq!(
            evaluate(&w),
            Verdict::Ineligible(Denial::DurabilityInsufficient)
        );
    }

    #[test]
    fn all_nonclean_retry_states_reconcile_instead_of_repeating_delete() {
        for state in [
            EffectState::AckUnknown,
            EffectState::PartialEffect,
            EffectState::AlreadyApplied,
        ] {
            let mut w = fixture();
            w.effect_state = state;
            assert_eq!(evaluate(&w), Verdict::ReconcilePriorEffect);
        }
    }

    #[test]
    fn unicode_whitespace_cannot_spoof_archive_or_replay_identifiers() {
        for marker in [
            '\u{00a0}', '\u{1680}', '\u{2000}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{3000}',
        ] {
            let bad = format!("record{marker}split");
            // Make every independent copy agree: this must fail because the
            // identifier is unsafe, not because witness copies disagree.
            let mut w = fixture();
            w.observed_source.record_id = bad.clone();
            w.archive.source.record_id = bad.clone();
            w.authority.source.record_id = bad.clone();
            assert_eq!(
                evaluate(&w),
                Verdict::Ineligible(Denial::SourceMoved),
                "unicode-space source ID admitted: {marker:?}"
            );

            let mut w = fixture();
            w.archive.segment = bad.clone();
            w.durability.segment = bad.clone();
            assert_eq!(
                evaluate(&w),
                Verdict::Ineligible(Denial::ArchiveNotExact),
                "unicode-space segment ID admitted: {marker:?}"
            );

            let mut w = fixture();
            w.operation_id = bad.clone();
            w.authority.operation_id = bad.clone();
            assert_eq!(
                evaluate(&w),
                Verdict::Ineligible(Denial::AuthorityNotCurrent),
                "unicode-space operation ID admitted: {marker:?}"
            );

            let mut record = item("a", 10);
            record.stable_id = bad;
            assert_eq!(
                replay(&[record, item("b", 11)], &[item("c", 12)], &cut()),
                Err(ReplayFailure::UntrustedCut),
                "unicode-space replay identity admitted: {marker:?}"
            );
        }

        // Opaque IDs may contain harmless Unicode, and internal ASCII spaces
        // remain valid; the guard is specific to display-spoofing whitespace.
        assert!(named("source id"));
        assert!(named("source-λ"));
    }

    fn item(id: &str, sequence: u64) -> ReplayRecord {
        ReplayRecord {
            stable_id: id.into(),
            sequence,
            payload_digest: [sequence as u8; 32],
            order_basis: basis("order-O1"),
            source_incarnation: "live-1".into(),
        }
    }

    fn cut() -> ReplayCut {
        ReplayCut {
            ordering: basis("order-O1"),
            source_incarnation: "live-1".into(),
            first_sequence: 10,
            last_sequence: 12,
            authoritative_order: true,
            current_manifest: true,
            coherent_snapshot: true,
            complete_frontier: true,
            closing_fence_current: true,
        }
    }

    #[test]
    fn archived_live_overlap_is_deduplicated_by_exact_semantic_identity() {
        let records = replay(
            &[item("a", 10), item("b", 11)],
            &[item("b", 11), item("c", 12)],
            &cut(),
        )
        .unwrap();
        assert_eq!(
            records.iter().map(|r| r.sequence).collect::<Vec<_>>(),
            vec![10, 11, 12]
        );
    }

    #[test]
    fn mixed_cut_omission_is_not_repaired_by_guessing_missing_history() {
        assert_eq!(
            replay(&[item("a", 10)], &[item("c", 12)], &cut()),
            Err(ReplayFailure::IncompleteHistory)
        );
    }

    #[test]
    fn conflicting_duplicate_and_reordered_identity_are_denied() {
        let mut changed = item("b", 11);
        changed.payload_digest = [99; 32];
        assert_eq!(
            replay(
                &[item("a", 10), item("b", 11)],
                &[changed, item("c", 12)],
                &cut()
            ),
            Err(ReplayFailure::ConflictingDuplicate)
        );
        assert_eq!(
            replay(&[item("a", 10), item("b", 11)], &[item("a", 12)], &cut()),
            Err(ReplayFailure::ConflictingDuplicate)
        );
    }

    #[test]
    fn untrusted_ordering_or_moved_snapshot_never_replays() {
        let mut snapshot = cut();
        snapshot.authoritative_order = false;
        assert_eq!(
            replay(&[], &[], &snapshot),
            Err(ReplayFailure::UntrustedCut)
        );
        snapshot = cut();
        snapshot.coherent_snapshot = false;
        assert_eq!(
            replay(&[], &[], &snapshot),
            Err(ReplayFailure::UntrustedCut)
        );
        snapshot = cut();
        let mut moved = item("a", 10);
        moved.order_basis.generation += 1;
        assert_eq!(
            replay(&[moved, item("b", 11), item("c", 12)], &[], &snapshot),
            Err(ReplayFailure::UntrustedCut)
        );
    }

    #[test]
    fn enormous_untrusted_sequence_range_does_not_trigger_large_iteration() {
        let mut snapshot = cut();
        snapshot.first_sequence = 0;
        snapshot.last_sequence = u64::MAX;
        assert_eq!(
            replay(&[], &[], &snapshot),
            Err(ReplayFailure::IncompleteHistory)
        );
    }
}

pub mod compaction;
