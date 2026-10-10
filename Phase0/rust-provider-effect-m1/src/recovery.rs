//! Offline reconstruction of a synthetic #91 M1 outbox journal.
//!
//! This is NOT durable storage or an authenticated event log. Input records
//! and historical Boundary booleans are not trusted provider/authority proofs.
//! Replay never sends a provider call, issues an assignment or grants authority.
//! An in-flight dispatch after a simulated restart becomes EFFECT_UNKNOWN;
//! an unused reservation is released. Neither condition is a retry license.
use crate::outbox::{Action, Boundary, Envelope, Error, Outbox, Status};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JournalRecord {
    Submitted(Envelope),
    Applied {
        operation_id: String,
        expected_version: u64,
        action: Action,
        historical_boundary: Boundary,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    DuplicateSubmission(usize),
    InvalidRecord(usize, Error),
}

pub struct ReplayOutcome {
    pub outbox: Outbox,
    /// Number of pre-effect reservations released to QUEUED.
    pub released_reservations: usize,
    /// Number of in-flight attempts demoted to EFFECT_UNKNOWN.
    pub newly_ambiguous_attempts: usize,
}

/// Replay a synthetic, ordered, complete journal and conservatively model a
/// process cutoff. The caller must independently establish journal integrity,
/// provenance, source completeness and version compatibility before ANY real
/// recovery decision. This function proves none of those prerequisites.
pub fn reconstruct_after_cutoff(records: &[JournalRecord]) -> Result<ReplayOutcome, ReplayError> {
    let mut outbox = Outbox::new();
    let mut operation_ids = Vec::new();

    for (index, record) in records.iter().enumerate() {
        match record {
            JournalRecord::Submitted(envelope) => match outbox.submit(envelope.clone()) {
                Ok(true) => operation_ids.push(envelope.operation_id.clone()),
                Ok(false) => return Err(ReplayError::DuplicateSubmission(index)),
                Err(error) => return Err(ReplayError::InvalidRecord(index, error)),
            },
            JournalRecord::Applied {
                operation_id,
                expected_version,
                action,
                historical_boundary,
            } => {
                outbox
                    .apply(
                        operation_id,
                        *expected_version,
                        *action,
                        *historical_boundary,
                    )
                    .map_err(|error| ReplayError::InvalidRecord(index, error))?;
            }
        }
    }

    let mut released_reservations = 0;
    let mut newly_ambiguous_attempts = 0;
    for id in operation_ids {
        let entry = outbox.entry(&id).expect("submitted entry must exist");
        let (version, action) = match entry.status() {
            Status::Reserved => {
                released_reservations += 1;
                (entry.version(), Action::ReleaseReservation)
            }
            Status::Dispatching => {
                newly_ambiguous_attempts += 1;
                (entry.version(), Action::MarkEffectUnknown)
            }
            _ => continue,
        };

        // Recovery here is a synthetic state transition, not an effect.
        // A historical authority flag is deliberately NOT reused as current.
        outbox
            .apply(
                &id,
                version,
                action,
                Boundary {
                    authority_current: false,
                    provider_fence_current: false,
                },
            )
            .expect("model's recovery transition must be valid");
    }
    Ok(ReplayOutcome {
        outbox,
        released_reservations,
        newly_ambiguous_attempts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: Boundary = Boundary {
        authority_current: true,
        provider_fence_current: true,
    };

    fn envelope(operation_id: &str) -> Envelope {
        Envelope {
            operation_id: operation_id.into(),
            repository_incarnation: "repo-91".into(),
            target: "issue:91/comments".into(),
            payload_digest: "sha256:test".into(),
            assignment_id: "assignment-91".into(),
            authority_generation: 3,
        }
    }

    fn submitted(id: &str) -> JournalRecord {
        JournalRecord::Submitted(envelope(id))
    }

    fn applied(id: &str, version: u64, action: Action) -> JournalRecord {
        JournalRecord::Applied {
            operation_id: id.into(),
            expected_version: version,
            action,
            historical_boundary: CURRENT,
        }
    }

    #[test]
    fn empty_synthetic_history_has_no_invented_attempt() {
        let replay = reconstruct_after_cutoff(&[]).unwrap();
        assert!(replay.outbox.entry("unknown").is_none());
        assert_eq!(replay.released_reservations, 0);
        assert_eq!(replay.newly_ambiguous_attempts, 0);
    }

    #[test]
    fn pre_effect_reservation_releases_without_dispatch() {
        let records = vec![
            submitted("one"),
            applied("one", 0, Action::Admit),
            applied("one", 1, Action::Reserve),
        ];
        let replay = reconstruct_after_cutoff(&records).unwrap();
        let entry = replay.outbox.entry("one").unwrap();
        assert_eq!(entry.status(), Status::Queued);
        assert_eq!(entry.version(), 3);
        assert_eq!(entry.history().len(), 4);
        assert!(!entry.attempted());
        assert!(!entry.automatic_retry_allowed());
        assert_eq!(replay.released_reservations, 1);
        assert_eq!(replay.newly_ambiguous_attempts, 0);
    }

    #[test]
    fn cutoff_after_begin_attempt_is_unknown_not_retried_or_proven() {
        let records = vec![
            submitted("one"),
            applied("one", 0, Action::Admit),
            applied("one", 1, Action::Reserve),
            applied("one", 2, Action::BeginAttempt),
        ];
        let mut replay = reconstruct_after_cutoff(&records).unwrap();
        assert_eq!(replay.released_reservations, 0);
        assert_eq!(replay.newly_ambiguous_attempts, 1);
        let entry = replay.outbox.entry("one").unwrap();
        assert_eq!(entry.status(), Status::EffectUnknown);
        assert!(entry.attempted());
        assert_eq!(entry.version(), 4);
        assert!(!entry.automatic_retry_allowed());
        assert!(matches!(
            replay.outbox.apply("one", 4, Action::BeginAttempt, CURRENT),
            Err(Error::InvalidTransition)
        ));
        assert_eq!(
            replay.outbox.entry("one").unwrap().status(),
            Status::EffectUnknown
        );
    }

    #[test]
    fn multiple_operations_recover_independently_without_success_invention() {
        let records = vec![
            submitted("dispatched"),
            submitted("reserved"),
            submitted("held"),
            applied("dispatched", 0, Action::Admit),
            applied("dispatched", 1, Action::Reserve),
            applied("dispatched", 2, Action::BeginAttempt),
            applied("reserved", 0, Action::Admit),
            applied("reserved", 1, Action::Reserve),
            applied("held", 0, Action::Admit),
            applied("held", 1, Action::Reserve),
            applied("held", 2, Action::BeginAttempt),
            applied("held", 3, Action::MarkEffectUnknown),
            applied("held", 4, Action::Hold),
        ];
        let replay = reconstruct_after_cutoff(&records).unwrap();
        assert_eq!(replay.newly_ambiguous_attempts, 1);
        assert_eq!(replay.released_reservations, 1);
        assert_eq!(
            replay.outbox.entry("dispatched").unwrap().status(),
            Status::EffectUnknown
        );
        assert_eq!(
            replay.outbox.entry("reserved").unwrap().status(),
            Status::Queued
        );
        assert_eq!(
            replay.outbox.entry("held").unwrap().status(),
            Status::ManualHold
        );
        for id in ["dispatched", "reserved", "held"] {
            assert_ne!(
                replay.outbox.entry(id).unwrap().status(),
                Status::RemoteProven
            );
        }
    }

    #[test]
    fn prepared_and_queued_reconstruct_without_dispatch_or_implicit_admission() {
        let records = vec![
            submitted("prepared"),
            submitted("queued"),
            applied("queued", 0, Action::Admit),
        ];
        let replay = reconstruct_after_cutoff(&records).unwrap();
        let prepared = replay.outbox.entry("prepared").unwrap();
        let queued = replay.outbox.entry("queued").unwrap();
        assert_eq!(prepared.status(), Status::Prepared);
        assert_eq!(queued.status(), Status::Queued);
        assert_eq!(prepared.version(), 0);
        assert_eq!(queued.version(), 1);
        assert!(!prepared.attempted());
        assert!(!queued.attempted());
        assert_eq!(replay.released_reservations, 0);
        assert_eq!(replay.newly_ambiguous_attempts, 0);
    }

    #[test]
    fn replay_order_and_versions_are_semantic() {
        let out_of_order = vec![applied("one", 0, Action::Admit), submitted("one")];
        assert_eq!(
            reconstruct_after_cutoff(&out_of_order).err(),
            Some(ReplayError::InvalidRecord(0, Error::UnknownOperation))
        );

        let stale_version = vec![
            submitted("one"),
            applied("one", 0, Action::Admit),
            applied("one", 0, Action::Reserve),
        ];
        assert_eq!(
            reconstruct_after_cutoff(&stale_version).err(),
            Some(ReplayError::InvalidRecord(2, Error::StaleVersion))
        );
    }

    #[test]
    fn duplicate_or_changed_submission_is_not_silently_deduplicated() {
        assert_eq!(
            reconstruct_after_cutoff(&[submitted("one"), submitted("one")]).err(),
            Some(ReplayError::DuplicateSubmission(1))
        );
        let mut changed = envelope("one");
        changed.payload_digest = "sha256:different".into();
        let changed_records = vec![submitted("one"), JournalRecord::Submitted(changed)];
        assert_eq!(
            reconstruct_after_cutoff(&changed_records).err(),
            Some(ReplayError::InvalidRecord(
                1,
                Error::OperationIdentityConflict
            ))
        );
    }

    #[test]
    fn historical_denial_is_not_rewritten_as_allowed() {
        let denied = vec![
            submitted("one"),
            applied("one", 0, Action::Admit),
            JournalRecord::Applied {
                operation_id: "one".into(),
                expected_version: 1,
                action: Action::Reserve,
                historical_boundary: Boundary {
                    authority_current: false,
                    provider_fence_current: true,
                },
            },
        ];
        assert_eq!(
            reconstruct_after_cutoff(&denied).err(),
            Some(ReplayError::InvalidRecord(2, Error::StaleAuthority))
        );
    }

    #[test]
    fn replay_is_deterministic_and_requires_explicit_reconciliation() {
        let records = vec![
            submitted("one"),
            applied("one", 0, Action::Admit),
            applied("one", 1, Action::Reserve),
            applied("one", 2, Action::BeginAttempt),
            applied("one", 3, Action::MarkEffectUnknown),
            applied("one", 4, Action::StartReconciliation),
        ];
        let first = reconstruct_after_cutoff(&records).unwrap();
        let second = reconstruct_after_cutoff(&records).unwrap();
        let a = first.outbox.entry("one").unwrap();
        let b = second.outbox.entry("one").unwrap();
        assert_eq!(a.history(), b.history());
        assert_eq!(a.status(), Status::Reconciling);
        assert!(a.attempted());
        assert!(!a.automatic_retry_allowed());
        assert_eq!(first.newly_ambiguous_attempts, 0);
    }
}
