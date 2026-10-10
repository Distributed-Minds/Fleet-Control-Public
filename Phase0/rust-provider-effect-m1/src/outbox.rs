//! Offline M1 outbox lifecycle model for #91. In-memory only: no persistence,
//! clock, credentials, provider adapter, trusted receipts, or write authority.
//! These transitions must not be used to authorize a real external effect.
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub operation_id: String,
    pub repository_incarnation: String,
    pub target: String,
    pub payload_digest: String,
    pub assignment_id: String,
    pub authority_generation: u64,
}

impl Envelope {
    fn valid(&self) -> bool {
        // Zero is not a meaningful authority generation, even in this
        // untrusted, in-memory simulation. Do not admit it to the journal.
        if self.authority_generation == 0 {
            return false;
        }
        [
            &self.operation_id,
            &self.repository_incarnation,
            &self.target,
            &self.payload_digest,
            &self.assignment_id,
        ]
        .iter()
        .all(|value| crate::bounded_machine_id(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Prepared,
    Queued,
    Reserved,
    Dispatching,
    EffectUnknown,
    Reconciling,
    RemoteProven,
    Rejected,
    Cancelled,
    ManualHold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Admit,
    Reserve,
    ReleaseReservation,
    BeginAttempt,
    MarkEffectUnknown,
    StartReconciliation,
    Hold,
    Reject,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidEnvelope,
    OperationIdentityConflict,
    UnknownOperation,
    StaleVersion,
    StaleAuthority,
    InvalidTransition,
    VersionExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// Synthetic input, NOT evidence of authorization or provider currentness.
    pub authority_current: bool,
    pub provider_fence_current: bool,
}

impl Boundary {
    fn current(self) -> bool {
        self.authority_current && self.provider_fence_current
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub version: u64,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    envelope: Envelope,
    status: Status,
    version: u64,
    attempted: bool,
    history: Vec<Step>,
}

impl Entry {
    pub fn envelope(&self) -> &Envelope {
        &self.envelope
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn attempted(&self) -> bool {
        self.attempted
    }

    pub fn history(&self) -> &[Step] {
        &self.history
    }

    /// No automatic replay exists, including after an ambiguous acknowledgement.
    pub fn automatic_retry_allowed(&self) -> bool {
        false
    }
}

/// Deterministic, single-threaded, *non-durable* model. A production outbox
/// needs trusted admission, a transactional journal, locks/fences and replay
/// reconciliation; this type deliberately provides none of those effects.
#[derive(Default)]
pub struct Outbox {
    entries: BTreeMap<String, Entry>,
}

impl Outbox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Exact semantic duplicates are idempotent. Reusing an operation ID for a
    /// different payload, target, assignment or authority is an error.
    pub fn submit(&mut self, envelope: Envelope) -> Result<bool, Error> {
        if !envelope.valid() {
            return Err(Error::InvalidEnvelope);
        }
        if let Some(existing) = self.entries.get(&envelope.operation_id) {
            return if existing.envelope == envelope {
                Ok(false)
            } else {
                Err(Error::OperationIdentityConflict)
            };
        }
        self.entries.insert(
            envelope.operation_id.clone(),
            Entry {
                envelope,
                status: Status::Prepared,
                version: 0,
                attempted: false,
                history: vec![Step {
                    version: 0,
                    status: Status::Prepared,
                }],
            },
        );
        Ok(true)
    }

    pub fn entry(&self, operation_id: &str) -> Option<&Entry> {
        self.entries.get(operation_id)
    }

    /// A CAS-style transition in one in-memory model. 'Boundary' is synthetic;
    /// it cannot prove a current provider capability, lease or actual effect.
    /// There is intentionally no RemoteProven action or trusted proof constructor.
    pub fn apply(
        &mut self,
        operation_id: &str,
        expected_version: u64,
        action: Action,
        boundary: Boundary,
    ) -> Result<&Entry, Error> {
        let entry = self
            .entries
            .get_mut(operation_id)
            .ok_or(Error::UnknownOperation)?;
        if entry.version != expected_version {
            return Err(Error::StaleVersion);
        }
        let next = match (entry.status, action) {
            (Status::Prepared, Action::Admit) => Status::Queued,
            (Status::Prepared, Action::Reject) => Status::Rejected,
            (Status::Queued, Action::Reserve) => Status::Reserved,
            (Status::Queued, Action::Cancel) => Status::Cancelled,
            (Status::Reserved, Action::ReleaseReservation) => Status::Queued,
            (Status::Reserved, Action::Reject) => Status::Rejected,
            (Status::Reserved, Action::BeginAttempt) => Status::Dispatching,
            (Status::Dispatching, Action::MarkEffectUnknown) => Status::EffectUnknown,
            (Status::EffectUnknown, Action::StartReconciliation) => Status::Reconciling,
            (Status::EffectUnknown, Action::Hold) | (Status::Reconciling, Action::Hold) => {
                Status::ManualHold
            }
            _ => return Err(Error::InvalidTransition),
        };
        if matches!(action, Action::Reserve | Action::BeginAttempt) && !boundary.current() {
            return Err(Error::StaleAuthority);
        }
        let next_version = entry
            .version
            .checked_add(1)
            .ok_or(Error::VersionExhausted)?;
        if action == Action::BeginAttempt {
            // This flag models a write-ahead boundary, not provider transmission.
            entry.attempted = true;
        }
        entry.status = next;
        entry.version = next_version;
        entry.history.push(Step {
            version: next_version,
            status: next,
        });
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> Envelope {
        Envelope {
            operation_id: "op-123".into(),
            repository_incarnation: "repo-2026".into(),
            target: "issue:91/comments".into(),
            payload_digest: "sha256:abc".into(),
            assignment_id: "assignment-7".into(),
            authority_generation: 4,
        }
    }

    const CURRENT: Boundary = Boundary {
        authority_current: true,
        provider_fence_current: true,
    };

    fn step(outbox: &mut Outbox, version: u64, action: Action) -> Status {
        outbox
            .apply("op-123", version, action, CURRENT)
            .unwrap()
            .status()
    }

    #[test]
    fn queued_reserved_dispatch_unknown_reconcile_hold_is_exact() {
        let mut outbox = Outbox::new();
        assert_eq!(outbox.submit(envelope()), Ok(true));
        assert_eq!(step(&mut outbox, 0, Action::Admit), Status::Queued);
        assert_eq!(step(&mut outbox, 1, Action::Reserve), Status::Reserved);
        assert_eq!(
            step(&mut outbox, 2, Action::BeginAttempt),
            Status::Dispatching
        );
        assert_eq!(
            step(&mut outbox, 3, Action::MarkEffectUnknown),
            Status::EffectUnknown
        );
        assert_eq!(
            step(&mut outbox, 4, Action::StartReconciliation),
            Status::Reconciling
        );
        assert_eq!(step(&mut outbox, 5, Action::Hold), Status::ManualHold);
        let entry = outbox.entry("op-123").unwrap();
        assert!(entry.attempted());
        assert!(!entry.automatic_retry_allowed());
        assert_eq!(entry.version(), 6);
        assert_eq!(
            entry
                .history()
                .iter()
                .map(|step| step.status)
                .collect::<Vec<_>>(),
            vec![
                Status::Prepared,
                Status::Queued,
                Status::Reserved,
                Status::Dispatching,
                Status::EffectUnknown,
                Status::Reconciling,
                Status::ManualHold
            ]
        );
    }

    #[test]
    fn exact_duplicate_is_noop_but_changed_semantics_reject() {
        let mut outbox = Outbox::new();
        assert_eq!(outbox.submit(envelope()), Ok(true));
        assert_eq!(outbox.submit(envelope()), Ok(false));
        step(&mut outbox, 0, Action::Admit);
        assert_eq!(outbox.submit(envelope()), Ok(false));
        for field in 0..5 {
            let mut altered = envelope();
            match field {
                0 => altered.target = "issue:92/comments".into(),
                1 => altered.payload_digest = "sha256:different".into(),
                2 => altered.assignment_id = "assignment-8".into(),
                3 => altered.repository_incarnation = "repo-restored".into(),
                _ => altered.authority_generation += 1,
            }
            assert_eq!(
                outbox.submit(altered),
                Err(Error::OperationIdentityConflict)
            );
        }
        assert_eq!(outbox.entry("op-123").unwrap().version(), 1);
    }

    #[test]
    fn stale_version_and_fence_cannot_mutate_or_dispatch() {
        let mut outbox = Outbox::new();
        outbox.submit(envelope()).unwrap();
        step(&mut outbox, 0, Action::Admit);
        assert!(matches!(
            outbox.apply("op-123", 0, Action::Reserve, CURRENT),
            Err(Error::StaleVersion)
        ));
        for stale in [
            Boundary {
                authority_current: false,
                provider_fence_current: true,
            },
            Boundary {
                authority_current: true,
                provider_fence_current: false,
            },
        ] {
            assert!(matches!(
                outbox.apply("op-123", 1, Action::Reserve, stale),
                Err(Error::StaleAuthority)
            ));
        }
        step(&mut outbox, 1, Action::Reserve);
        assert!(matches!(
            outbox.apply(
                "op-123",
                2,
                Action::BeginAttempt,
                Boundary {
                    authority_current: false,
                    provider_fence_current: false
                }
            ),
            Err(Error::StaleAuthority)
        ));
        let entry = outbox.entry("op-123").unwrap();
        assert!(!entry.attempted());
        assert_eq!(entry.version(), 2);
        assert_eq!(entry.history().len(), 3);
    }

    #[test]
    fn invalid_skips_and_terminal_states_cannot_resume() {
        let mut outbox = Outbox::new();
        outbox.submit(envelope()).unwrap();
        assert!(matches!(
            outbox.apply("op-123", 0, Action::BeginAttempt, CURRENT),
            Err(Error::InvalidTransition)
        ));
        step(&mut outbox, 0, Action::Admit);
        step(&mut outbox, 1, Action::Cancel);
        assert_eq!(outbox.entry("op-123").unwrap().status(), Status::Cancelled);
        for action in [Action::Admit, Action::Reserve, Action::BeginAttempt] {
            assert!(matches!(
                outbox.apply("op-123", 2, action, CURRENT),
                Err(Error::InvalidTransition)
            ));
        }
        assert_eq!(outbox.entry("op-123").unwrap().history().len(), 3);
    }

    #[test]
    fn reservation_release_is_not_a_provider_retry() {
        let mut outbox = Outbox::new();
        outbox.submit(envelope()).unwrap();
        step(&mut outbox, 0, Action::Admit);
        step(&mut outbox, 1, Action::Reserve);
        step(&mut outbox, 2, Action::ReleaseReservation);
        assert_eq!(outbox.entry("op-123").unwrap().status(), Status::Queued);
        assert!(!outbox.entry("op-123").unwrap().attempted());
        step(&mut outbox, 3, Action::Reserve);
        step(&mut outbox, 4, Action::BeginAttempt);
        assert!(matches!(
            outbox.apply("op-123", 5, Action::BeginAttempt, CURRENT),
            Err(Error::InvalidTransition)
        ));
        assert_eq!(outbox.entry("op-123").unwrap().version(), 5);
    }

    #[test]
    fn untrusted_or_empty_identity_never_enters_journal() {
        let mut outbox = Outbox::new();
        for value in ["", "  ", "op\nspoof"] {
            let mut invalid = envelope();
            invalid.operation_id = value.into();
            assert_eq!(outbox.submit(invalid), Err(Error::InvalidEnvelope));
        }
        assert!(outbox.entry("op-123").is_none());
        assert!(matches!(
            outbox.apply("missing", 0, Action::Admit, CURRENT),
            Err(Error::UnknownOperation)
        ));
    }

    #[test]
    fn zero_authority_generation_is_denied_before_journaling_or_dispatch() {
        let mut outbox = Outbox::new();
        let mut invalid = envelope();
        invalid.authority_generation = 0;
        assert_eq!(outbox.submit(invalid), Err(Error::InvalidEnvelope));
        assert!(outbox.entry("op-123").is_none());
        assert_eq!(
            outbox.apply("op-123", 0, Action::Admit, CURRENT),
            Err(Error::UnknownOperation)
        );

        // Preserve the positive synthetic path for a nonzero generation.
        assert_eq!(outbox.submit(envelope()), Ok(true));
        assert_eq!(step(&mut outbox, 0, Action::Admit), Status::Queued);
        assert_eq!(step(&mut outbox, 1, Action::Reserve), Status::Reserved);
        assert_eq!(
            step(&mut outbox, 2, Action::BeginAttempt),
            Status::Dispatching
        );
        assert_eq!(outbox.entry("op-123").unwrap().version(), 3);
    }

    #[test]
    fn ambiguous_effect_never_becomes_success_or_automatically_replays() {
        let mut outbox = Outbox::new();
        outbox.submit(envelope()).unwrap();
        step(&mut outbox, 0, Action::Admit);
        step(&mut outbox, 1, Action::Reserve);
        step(&mut outbox, 2, Action::BeginAttempt);
        step(&mut outbox, 3, Action::MarkEffectUnknown);
        assert!(matches!(
            outbox.apply("op-123", 4, Action::BeginAttempt, CURRENT),
            Err(Error::InvalidTransition)
        ));
        step(&mut outbox, 4, Action::Hold);
        let entry = outbox.entry("op-123").unwrap();
        assert_eq!(entry.status(), Status::ManualHold);
        assert!(!entry.automatic_retry_allowed());
        assert!(!entry
            .history()
            .iter()
            .any(|event| event.status == Status::RemoteProven));
    }
    #[test]
    fn opaque_envelope_fields_are_bounded_and_cannot_hide_format_controls() {
        let mut outbox = Outbox::new();
        for bad in [
            "with space".to_owned(),
            "a\tb".to_owned(),
            "hidden\u{202e}suffix".to_owned(),
            "é".to_owned(),
            "x".repeat(513),
        ] {
            for field in 0..5 {
                let mut invalid = envelope();
                match field {
                    0 => invalid.operation_id = bad.clone(),
                    1 => invalid.repository_incarnation = bad.clone(),
                    2 => invalid.target = bad.clone(),
                    3 => invalid.payload_digest = bad.clone(),
                    _ => invalid.assignment_id = bad.clone(),
                }
                assert_eq!(outbox.submit(invalid), Err(Error::InvalidEnvelope));
            }
        }
        assert!(outbox.entry("op-123").is_none());

        let mut maximum = envelope();
        maximum.operation_id = "x".repeat(512);
        assert_eq!(outbox.submit(maximum), Ok(true));
    }
}
