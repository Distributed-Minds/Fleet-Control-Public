//! Offline, fake-identity request replay model for issue #280, specification 2.
//!
//! The trusted test harness supplies "authenticated" principals here. This is
//! NOT an authentication adapter, SQLite journal, HTTP API or effect authority.
//! Receipts and principal state vanish on restart; no worker can execute them.

use crate::{
    compile_footprint, AdmissionError, CanonicalResource, ScopeConflictBasis, ScopeSelector,
    SimulationBook, SimulationReceipt, SimulationState,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPoll {
    pub request_id: String,
    pub task_id: u64,
    pub selectors: Vec<ScopeSelector>,
    // Synthetic caller inputs, never a grant of external capability.
    pub protocol_version: u32,
    pub context_generation: u64,
    pub capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidPrincipal,
    StalePrincipal,
    RevokedPrincipal,
    InvalidRequestId,
    UnsupportedProtocol,
    InvalidContext,
    InvalidCapabilities,
    ConflictingReplay,
    CapacityExhausted,
    RecoveryRequired,
    Scope(AdmissionError),
}

/// Model-only acceptance of a duplicate or first delivery event; neither grants
/// authorization to start a worker, mutate GitHub or trust result contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Recorded,
    Reconciled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryError {
    UnknownPrincipal,
    StalePrincipal,
    RevokedPrincipal,
    StaleBasis,
    UnknownRequest,
    AssignmentMismatch,
    RecoveryRequired,
    MissingAcknowledgement,
    InvalidDigest,
    ConflictingResult,
}

impl From<AdmissionError> for ReplayError {
    fn from(error: AdmissionError) -> Self {
        Self::Scope(error)
    }
}

#[derive(Debug, Clone)]
struct TestPrincipal {
    generation: u64,
    active: bool,
}

#[derive(Debug, Clone)]
struct RecordedRequest {
    task_id: u64,
    resources: BTreeSet<CanonicalResource>,
    protocol_version: u32,
    context_generation: u64,
    capabilities: BTreeSet<String>,
    receipt: SimulationReceipt,
}

/// Bound synthetic immutable request history. Never evict old receipts to
/// admit another request: historical replay/ACK/result fences must survive.
/// This is a fixture capacity limit, not a production quota or cleanup policy.
pub const MAX_SIMULATED_REQUESTS: usize = 1024;

/// A deliberately non-durable, single-threaded model. Unlike production
/// authentication, the caller here is the trusted *test harness* itself.
/// Tests must not pass a client-supplied identity off as server authentication.
#[derive(Debug)]
pub struct SimulatedReplay {
    basis: ScopeConflictBasis,
    book: SimulationBook,
    principals: BTreeMap<u64, TestPrincipal>,
    requests: BTreeMap<(u64, u64, String), RecordedRequest>,
    // Synthetic receipts only; no trusted worker transport or effect execution.
    acknowledgements: BTreeMap<(u64, u64, String), u64>,
    results: BTreeMap<(u64, u64, String), (u64, String)>,
}

impl SimulatedReplay {
    pub fn new(basis: ScopeConflictBasis) -> Result<Self, AdmissionError> {
        let book = SimulationBook::new(basis.clone())?;
        Ok(Self {
            basis,
            book,
            principals: BTreeMap::new(),
            requests: BTreeMap::new(),
            acknowledgements: BTreeMap::new(),
            results: BTreeMap::new(),
        })
    }

    // Principal-generation movement cannot silently keep older assignments
    // active. A recovery hold preserves both the immutable receipts and scopes.
    fn hold_principal_reservations(&mut self, principal_id: u64) -> Result<(), ReplayError> {
        let task_ids: BTreeSet<u64> = self
            .requests
            .iter()
            .filter(|((id, _, _), _)| *id == principal_id)
            .map(|(_, record)| record.task_id)
            .collect();
        for task_id in task_ids {
            self.book.require_recovery(task_id)?;
        }
        Ok(())
    }

    /// Only a privileged fake test fixture can enroll a principal. Generation
    /// reuse or rollback cannot resurrect previously revoked test identities.
    pub fn enroll_test_principal(
        &mut self,
        principal_id: u64,
        generation: u64,
    ) -> Result<(), ReplayError> {
        if principal_id == 0 || generation == 0 {
            return Err(ReplayError::InvalidPrincipal);
        }
        if let Some(existing) = self.principals.get(&principal_id) {
            if generation <= existing.generation {
                return Err(ReplayError::StalePrincipal);
            }
            // Rotating directly to a newer generation must fence old leases
            // even when a separate revoke event was never observed.
            self.hold_principal_reservations(principal_id)?;
        }
        self.principals.insert(
            principal_id,
            TestPrincipal {
                generation,
                active: true,
            },
        );
        Ok(())
    }

    /// Revoke the fake principal without destroying immutable replay records.
    /// Its previously held scopes become RECOVERY_REQUIRED, never free.
    pub fn revoke_test_principal(&mut self, principal_id: u64) -> Result<(), ReplayError> {
        let principal = self
            .principals
            .get_mut(&principal_id)
            .ok_or(ReplayError::InvalidPrincipal)?;
        principal.active = false;
        self.hold_principal_reservations(principal_id)
    }

    /// Treat the supplied principal as a fixture-authenticated identity.
    /// A repeated request is matched against its canonical COMPLETE footprint,
    /// not a caller-selected digest or the order of its resource selectors.
    /// Authorization and current basis are rechecked BEFORE returning history.
    pub fn poll(
        &mut self,
        authenticated_principal_id: u64,
        authenticated_generation: u64,
        request: &TestPoll,
        current_basis: &ScopeConflictBasis,
    ) -> Result<SimulationReceipt, ReplayError> {
        let principal = self
            .principals
            .get(&authenticated_principal_id)
            .ok_or(ReplayError::InvalidPrincipal)?;
        if authenticated_generation != principal.generation {
            return Err(ReplayError::StalePrincipal);
        }
        if !principal.active {
            return Err(ReplayError::RevokedPrincipal);
        }
        if request.request_id.is_empty()
            || request.request_id.len() > 128
            || !request
                .request_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(ReplayError::InvalidRequestId);
        }

        let footprint = compile_footprint(&request.selectors, &self.basis, current_basis)?;
        let key = (
            authenticated_principal_id,
            authenticated_generation,
            request.request_id.clone(),
        );
        if let Some(existing) = self.requests.get(&key) {
            // Reusing a request ID cannot borrow an old assignment under
            // a different protocol, context or claimed capability set.
            if request.task_id != existing.task_id
                || footprint != existing.resources
                || request.protocol_version != existing.protocol_version
                || request.context_generation != existing.context_generation
                || request.capabilities != existing.capabilities
            {
                return Err(ReplayError::ConflictingReplay);
            }
            if self.book.reservation(existing.task_id).map(|r| r.state)
                != Some(SimulationState::Active)
            {
                return Err(ReplayError::RecoveryRequired);
            }
            return Ok(existing.receipt.clone());
        }

        // Admit only this fixture protocol. No synthetic capability or context
        // identifier establishes real identity, authority or provider effects.
        if request.protocol_version != crate::SCHEMA_VERSION {
            return Err(ReplayError::UnsupportedProtocol);
        }
        if request.context_generation == 0 {
            return Err(ReplayError::InvalidContext);
        }
        if request.capabilities.is_empty()
            || request.capabilities.len() > 32
            || request.capabilities.iter().any(|name| {
                name.is_empty()
                    || name.len() > 64
                    || !name.bytes().all(|b| {
                        b.is_ascii_lowercase()
                            || b.is_ascii_digit()
                            || matches!(b, b'-' | b'_' | b'.')
                    })
            })
        {
            return Err(ReplayError::InvalidCapabilities);
        }

        // Check capacity only after looking up an existing immutable request:
        // exact retries and conflicting replay checks remain available at cap.
        // No reservation may be allocated for a rejected new request.
        if self.requests.len() >= MAX_SIMULATED_REQUESTS {
            return Err(ReplayError::CapacityExhausted);
        }

        let receipt = self
            .book
            .reserve(request.task_id, &request.selectors, current_basis)?
            .clone();
        self.requests.insert(
            key,
            RecordedRequest {
                task_id: request.task_id,
                resources: footprint,
                protocol_version: request.protocol_version,
                context_generation: request.context_generation,
                capabilities: request.capabilities.clone(),
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    /// A missing cutoff proof denies replay as an active assignment.
    pub fn require_recovery(&mut self, task_id: u64) -> Result<(), ReplayError> {
        self.book.require_recovery(task_id)?;
        Ok(())
    }

    /// Validate a fixture-supplied principal, request ID and exact assignment.
    /// The trusted test harness injects identity; no client JSON can authenticate
    /// itself by invoking this helper in a real execution context.
    fn delivery_key(
        &self,
        principal_id: u64,
        principal_generation: u64,
        request_id: &str,
        task_id: u64,
        assignment_generation: u64,
        current_basis: &ScopeConflictBasis,
    ) -> Result<(u64, u64, String), DeliveryError> {
        let principal = self
            .principals
            .get(&principal_id)
            .ok_or(DeliveryError::UnknownPrincipal)?;
        if principal_generation != principal.generation {
            return Err(DeliveryError::StalePrincipal);
        }
        if !principal.active {
            return Err(DeliveryError::RevokedPrincipal);
        }
        if current_basis != &self.basis {
            return Err(DeliveryError::StaleBasis);
        }
        let key = (principal_id, principal_generation, request_id.to_owned());
        // Do not expose another principal's receipt in denial diagnostics.
        let recorded = self
            .requests
            .get(&key)
            .ok_or(DeliveryError::UnknownRequest)?;
        if recorded.task_id != task_id
            || recorded.receipt.assignment_generation != assignment_generation
        {
            return Err(DeliveryError::AssignmentMismatch);
        }
        let live = self
            .book
            .reservation(task_id)
            .ok_or(DeliveryError::RecoveryRequired)?;
        if live.state != SimulationState::Active
            || live.assignment_generation != assignment_generation
        {
            return Err(DeliveryError::RecoveryRequired);
        }
        Ok(key)
    }

    /// One synthetic ACK per immutable issued request. Duplicate exact ACK is
    /// reconciled; revocation or recovery never re-activates an old assignment.
    pub fn acknowledge(
        &mut self,
        principal_id: u64,
        principal_generation: u64,
        request_id: &str,
        task_id: u64,
        assignment_generation: u64,
        current_basis: &ScopeConflictBasis,
    ) -> Result<DeliveryOutcome, DeliveryError> {
        let key = self.delivery_key(
            principal_id,
            principal_generation,
            request_id,
            task_id,
            assignment_generation,
            current_basis,
        )?;
        if self.acknowledgements.contains_key(&key) {
            return Ok(DeliveryOutcome::Reconciled);
        }
        self.acknowledgements.insert(key, assignment_generation);
        Ok(DeliveryOutcome::Recorded)
    }

    /// Exact semantic result identity is constrained by the previously issued
    /// assignment and its ACK. A digest is an *untrusted fixture label*: it is
    /// not a signature, provenance proof, domain-acceptance or provider effect.
    pub fn submit_result(
        &mut self,
        principal_id: u64,
        principal_generation: u64,
        request_id: &str,
        assignment: (u64, u64),
        payload_digest: &str,
        current_basis: &ScopeConflictBasis,
    ) -> Result<DeliveryOutcome, DeliveryError> {
        // The tuple retains the exact immutable (task, assignment-generation)
        // pair. It is never an authorization token by itself.
        let (task_id, assignment_generation) = assignment;
        let key = self.delivery_key(
            principal_id,
            principal_generation,
            request_id,
            task_id,
            assignment_generation,
            current_basis,
        )?;
        if payload_digest.is_empty()
            || payload_digest.len() > 128
            || !payload_digest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(DeliveryError::InvalidDigest);
        }
        if self.acknowledgements.get(&key) != Some(&assignment_generation) {
            return Err(DeliveryError::MissingAcknowledgement);
        }
        if let Some((saved_assignment, saved_digest)) = self.results.get(&key) {
            return if *saved_assignment == assignment_generation && saved_digest == payload_digest {
                Ok(DeliveryOutcome::Reconciled)
            } else {
                Err(DeliveryError::ConflictingResult)
            };
        }
        self.results
            .insert(key, (assignment_generation, payload_digest.to_owned()));
        Ok(DeliveryOutcome::Recorded)
    }

    pub fn acknowledgement_count(&self) -> usize {
        self.acknowledgements.len()
    }

    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn reservation_count(&self) -> usize {
        self.book.reservation_count()
    }

    pub fn request_count(&self) -> usize {
        self.requests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SCHEMA_VERSION;

    fn basis() -> ScopeConflictBasis {
        ScopeConflictBasis {
            version: SCHEMA_VERSION,
            generation: 1,
            repository_id: 1360059617,
            repository_incarnation: "fixture-only".to_owned(),
        }
    }

    fn poll(request_id: &str, task_id: u64, resources: &[u64]) -> TestPoll {
        TestPoll {
            request_id: request_id.into(),
            task_id,
            selectors: resources
                .iter()
                .map(|number| ScopeSelector::Conversation(*number))
                .collect(),
            protocol_version: SCHEMA_VERSION,
            context_generation: 1,
            capabilities: BTreeSet::from(["rust".to_owned()]),
        }
    }

    #[test]
    fn full_request_ledger_refuses_new_scope_without_eviction_or_replay_loss() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let original = poll("first", 1, &[1]);
        let first = state.poll(1, 1, &original, &basis()).unwrap();
        for index in 2..=MAX_SIMULATED_REQUESTS {
            let n = index as u64;
            let request = poll(&format!("request-{index}"), n, &[n]);
            state.poll(1, 1, &request, &basis()).unwrap();
        }
        assert_eq!(state.request_count(), MAX_SIMULATED_REQUESTS);
        assert_eq!(state.reservation_count(), MAX_SIMULATED_REQUESTS);

        let overflow = poll("new-after-cap", 9001, &[9001]);
        assert_eq!(
            state.poll(1, 1, &overflow, &basis()),
            Err(ReplayError::CapacityExhausted)
        );
        assert_eq!(state.request_count(), MAX_SIMULATED_REQUESTS);
        assert_eq!(state.reservation_count(), MAX_SIMULATED_REQUESTS);

        // The ledger stays useful for reconciliation even when full.
        assert_eq!(state.poll(1, 1, &original, &basis()), Ok(first.clone()));
        assert_eq!(
            state.poll(1, 1, &poll("first", 1, &[9001]), &basis()),
            Err(ReplayError::ConflictingReplay)
        );
        assert_eq!(
            state.acknowledge(1, 1, "first", 1, first.assignment_generation, &basis()),
            Ok(DeliveryOutcome::Recorded)
        );
        assert_eq!(
            state.submit_result(
                1,
                1,
                "first",
                (1, first.assignment_generation),
                "digest",
                &basis()
            ),
            Ok(DeliveryOutcome::Recorded)
        );
        assert_eq!(
            state.submit_result(
                1,
                1,
                "first",
                (1, first.assignment_generation),
                "digest",
                &basis()
            ),
            Ok(DeliveryOutcome::Reconciled)
        );
        assert_eq!(state.acknowledgement_count(), 1);
        assert_eq!(state.result_count(), 1);
    }

    #[test]
    fn same_request_cannot_replay_changed_protocol_context_or_capabilities() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let initial = poll("immutable", 40, &[10, 20]);
        let original = state.poll(1, 1, &initial, &basis()).unwrap();

        let mut changed = initial.clone();
        changed.protocol_version += 1;
        assert_eq!(
            state.poll(1, 1, &changed, &basis()),
            Err(ReplayError::ConflictingReplay)
        );
        changed = initial.clone();
        changed.context_generation += 1;
        assert_eq!(
            state.poll(1, 1, &changed, &basis()),
            Err(ReplayError::ConflictingReplay)
        );
        changed = initial.clone();
        changed.capabilities.insert("write".to_owned());
        assert_eq!(
            state.poll(1, 1, &changed, &basis()),
            Err(ReplayError::ConflictingReplay)
        );

        assert_eq!(state.request_count(), 1);
        assert_eq!(state.reservation_count(), 1);
        assert_eq!(state.poll(1, 1, &initial, &basis()), Ok(original));
    }

    #[test]
    fn invalid_first_poll_protocol_context_and_capabilities_do_not_reserve() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let original = poll("new", 50, &[10]);
        for version in [0, SCHEMA_VERSION + 1] {
            let mut changed = original.clone();
            changed.protocol_version = version;
            assert_eq!(
                state.poll(1, 1, &changed, &basis()),
                Err(ReplayError::UnsupportedProtocol)
            );
        }
        let mut changed = original.clone();
        changed.context_generation = 0;
        assert_eq!(
            state.poll(1, 1, &changed, &basis()),
            Err(ReplayError::InvalidContext)
        );

        for invalid in ["", "CAP", "bad space", "bad\nline", "e\u{202e}vil"] {
            let mut changed = original.clone();
            changed.capabilities = BTreeSet::from([invalid.to_owned()]);
            assert_eq!(
                state.poll(1, 1, &changed, &basis()),
                Err(ReplayError::InvalidCapabilities)
            );
        }
        let mut changed = original.clone();
        changed.capabilities = (0..33).map(|i| format!("cap-{i}")).collect();
        assert_eq!(
            state.poll(1, 1, &changed, &basis()),
            Err(ReplayError::InvalidCapabilities)
        );
        assert_eq!(state.request_count(), 0);
        assert_eq!(state.reservation_count(), 0);
        assert_eq!(state.poll(1, 1, &original, &basis()).unwrap().task_id, 50);
    }

    #[test]
    fn identical_retry_returns_immutable_receipt_without_second_allocation() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 7).unwrap();
        let request = poll("poll-1", 40, &[11, 12]);
        let first = state.poll(1, 7, &request, &basis()).unwrap();
        assert_eq!(state.poll(1, 7, &request, &basis()), Ok(first.clone()));
        assert_eq!(state.reservation_count(), 1);
        assert_eq!(state.request_count(), 1);
        assert_eq!(first.assignment_generation, 1);
    }

    #[test]
    fn canonical_selector_order_does_not_manufacture_replay_conflicts() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let first = state
            .poll(1, 1, &poll("a", 2, &[10, 20]), &basis())
            .unwrap();
        assert_eq!(
            state.poll(1, 1, &poll("a", 2, &[20, 10]), &basis()),
            Ok(first)
        );
        assert_eq!(state.reservation_count(), 1);
    }

    #[test]
    fn same_request_id_with_changed_material_scope_or_task_is_rejected() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        state.poll(1, 1, &poll("a", 2, &[10]), &basis()).unwrap();
        assert_eq!(
            state.poll(1, 1, &poll("a", 2, &[11]), &basis()),
            Err(ReplayError::ConflictingReplay)
        );
        assert_eq!(
            state.poll(1, 1, &poll("a", 3, &[10]), &basis()),
            Err(ReplayError::ConflictingReplay)
        );
        assert_eq!(state.reservation_count(), 1);
        assert_eq!(state.request_count(), 1);
    }

    #[test]
    fn principal_namespace_isolation_and_revocation_preserve_fences() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 3).unwrap();
        state.enroll_test_principal(2, 3).unwrap();
        state
            .poll(1, 3, &poll("same", 11, &[100]), &basis())
            .unwrap();
        state
            .poll(2, 3, &poll("same", 12, &[200]), &basis())
            .unwrap();
        assert_eq!(state.reservation_count(), 2);
        state.revoke_test_principal(1).unwrap();
        assert_eq!(
            state.poll(1, 3, &poll("same", 11, &[100]), &basis()),
            Err(ReplayError::RevokedPrincipal)
        );
        assert_eq!(
            state.poll(2, 3, &poll("new", 13, &[100]), &basis()),
            Err(ReplayError::Scope(AdmissionError::Collision {
                held_by_task: 11
            }))
        );
        assert_eq!(
            state
                .poll(2, 3, &poll("same", 12, &[200]), &basis())
                .unwrap()
                .task_id,
            12
        );
    }

    #[test]
    fn generation_rotation_requires_new_generation_and_never_revives_old_receipt() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        state.poll(1, 1, &poll("same", 11, &[1]), &basis()).unwrap();
        state.revoke_test_principal(1).unwrap();
        assert_eq!(
            state.enroll_test_principal(1, 1),
            Err(ReplayError::StalePrincipal)
        );
        state.enroll_test_principal(1, 2).unwrap();
        assert_eq!(
            state.poll(1, 1, &poll("same", 11, &[1]), &basis()),
            Err(ReplayError::StalePrincipal)
        );
        assert_eq!(
            state.poll(1, 2, &poll("same", 12, &[1]), &basis()),
            Err(ReplayError::Scope(AdmissionError::Collision {
                held_by_task: 11
            }))
        );
        state.poll(1, 2, &poll("same", 12, &[2]), &basis()).unwrap();
    }

    #[test]
    fn direct_generation_rotation_fences_old_leases_without_harming_other_principals() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        state.enroll_test_principal(2, 1).unwrap();
        let old_request = poll("old", 11, &[10]);
        state.poll(1, 1, &old_request, &basis()).unwrap();
        let other_request = poll("other", 21, &[20]);
        let other_receipt = state.poll(2, 1, &other_request, &basis()).unwrap();

        // No explicit revoke: the new identity generation must still fence
        // every old assignment and preserve its immutable replay history.
        state.enroll_test_principal(1, 2).unwrap();
        assert_eq!(
            state.book.reservation(11).unwrap().state,
            SimulationState::RecoveryRequired
        );
        assert_eq!(
            state.book.reservation(21).unwrap().state,
            SimulationState::Active
        );
        assert_eq!(state.request_count(), 2);
        assert_eq!(
            state.poll(1, 1, &old_request, &basis()),
            Err(ReplayError::StalePrincipal)
        );
        assert_eq!(
            state.poll(1, 2, &poll("new", 12, &[10]), &basis()),
            Err(ReplayError::Scope(AdmissionError::Collision {
                held_by_task: 11
            }))
        );
        assert_eq!(
            state.poll(2, 1, &other_request, &basis()),
            Ok(other_receipt)
        );
        state.poll(1, 2, &poll("new", 12, &[30]), &basis()).unwrap();
        assert_eq!(state.reservation_count(), 3);
    }

    #[test]
    fn recovery_and_moved_basis_block_replay_without_reallocation() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let request = poll("x", 10, &[1]);
        state.poll(1, 1, &request, &basis()).unwrap();
        let mut moved = basis();
        moved.generation += 1;
        assert_eq!(
            state.poll(1, 1, &request, &moved),
            Err(ReplayError::Scope(AdmissionError::StaleBasis))
        );
        state.require_recovery(10).unwrap();
        assert_eq!(
            state.poll(1, 1, &request, &basis()),
            Err(ReplayError::RecoveryRequired)
        );
        assert_eq!(state.reservation_count(), 1);
    }

    #[test]
    fn malformed_identity_or_request_is_denied_without_state_mutation() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        assert_eq!(
            state.enroll_test_principal(0, 1),
            Err(ReplayError::InvalidPrincipal)
        );
        state.enroll_test_principal(1, 1).unwrap();
        assert_eq!(
            state.poll(2, 1, &poll("a", 4, &[1]), &basis()),
            Err(ReplayError::InvalidPrincipal)
        );
        for name in ["", "bad id", "bad\nline", "x/invalid"] {
            assert_eq!(
                state.poll(1, 1, &poll(name, 4, &[1]), &basis()),
                Err(ReplayError::InvalidRequestId)
            );
        }
        let large = "x".repeat(129);
        assert_eq!(
            state.poll(1, 1, &poll(&large, 4, &[1]), &basis()),
            Err(ReplayError::InvalidRequestId)
        );
        assert_eq!(state.reservation_count(), 0);
        assert_eq!(state.request_count(), 0);
    }

    #[test]
    fn synthetic_ack_and_result_are_idempotent_and_generation_bound() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 7).unwrap();
        state.enroll_test_principal(2, 7).unwrap();
        let issued = state.poll(1, 7, &poll("r1", 40, &[11]), &basis()).unwrap();
        let generation = issued.assignment_generation;
        assert_eq!(
            state.acknowledge(2, 7, "r1", 40, generation, &basis()),
            Err(DeliveryError::UnknownRequest)
        );
        assert_eq!(
            state.acknowledge(1, 6, "r1", 40, generation, &basis()),
            Err(DeliveryError::StalePrincipal)
        );
        assert_eq!(
            state.acknowledge(1, 7, "r1", 40, generation + 1, &basis()),
            Err(DeliveryError::AssignmentMismatch)
        );
        assert_eq!(
            state.submit_result(1, 7, "r1", (40, generation), "digest1", &basis()),
            Err(DeliveryError::MissingAcknowledgement)
        );
        assert_eq!(
            state.acknowledge(1, 7, "r1", 40, generation, &basis()),
            Ok(DeliveryOutcome::Recorded)
        );
        assert_eq!(
            state.acknowledge(1, 7, "r1", 40, generation, &basis()),
            Ok(DeliveryOutcome::Reconciled)
        );
        assert_eq!(
            state.submit_result(1, 7, "r1", (40, generation), "digest1", &basis()),
            Ok(DeliveryOutcome::Recorded)
        );
        assert_eq!(
            state.submit_result(1, 7, "r1", (40, generation), "digest1", &basis()),
            Ok(DeliveryOutcome::Reconciled)
        );
        assert_eq!(
            state.submit_result(1, 7, "r1", (40, generation), "digest2", &basis()),
            Err(DeliveryError::ConflictingResult)
        );
        assert_eq!(state.acknowledgement_count(), 1);
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.request_count(), 1);
    }

    #[test]
    fn revoked_and_recovery_held_requests_never_accept_late_result() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let first = state
            .poll(1, 1, &poll("first", 10, &[10]), &basis())
            .unwrap();
        state
            .acknowledge(1, 1, "first", 10, first.assignment_generation, &basis())
            .unwrap();
        state.revoke_test_principal(1).unwrap();
        assert_eq!(
            state.submit_result(
                1,
                1,
                "first",
                (10, first.assignment_generation),
                "digest",
                &basis()
            ),
            Err(DeliveryError::RevokedPrincipal)
        );
        state.enroll_test_principal(1, 2).unwrap();
        assert_eq!(
            state.submit_result(
                1,
                1,
                "first",
                (10, first.assignment_generation),
                "digest",
                &basis()
            ),
            Err(DeliveryError::StalePrincipal)
        );
        let second = state
            .poll(1, 2, &poll("second", 20, &[20]), &basis())
            .unwrap();
        state
            .acknowledge(1, 2, "second", 20, second.assignment_generation, &basis())
            .unwrap();
        state.require_recovery(20).unwrap();
        assert_eq!(
            state.submit_result(
                1,
                2,
                "second",
                (20, second.assignment_generation),
                "digest",
                &basis()
            ),
            Err(DeliveryError::RecoveryRequired)
        );
        assert_eq!(state.acknowledgement_count(), 2);
        assert_eq!(state.result_count(), 0);
        assert_eq!(state.reservation_count(), 2);
    }

    #[test]
    fn result_controls_reject_wrong_assignment_stale_basis_and_malformed_digest() {
        let mut state = SimulatedReplay::new(basis()).unwrap();
        state.enroll_test_principal(1, 1).unwrap();
        let receipt = state
            .poll(1, 1, &poll("result", 12, &[7]), &basis())
            .unwrap();
        let generation = receipt.assignment_generation;
        let mut moved = basis();
        moved.generation += 1;
        assert_eq!(
            state.acknowledge(1, 1, "result", 12, generation, &moved),
            Err(DeliveryError::StaleBasis)
        );
        assert_eq!(
            state.acknowledge(1, 1, "result", 13, generation, &basis()),
            Err(DeliveryError::AssignmentMismatch)
        );
        state
            .acknowledge(1, 1, "result", 12, generation, &basis())
            .unwrap();
        for digest in ["", "bad digest", "bad\nvalue"] {
            assert_eq!(
                state.submit_result(1, 1, "result", (12, generation), digest, &basis()),
                Err(DeliveryError::InvalidDigest)
            );
        }
        assert_eq!(
            state.submit_result(1, 1, "result", (13, generation), "valid", &basis()),
            Err(DeliveryError::AssignmentMismatch)
        );
        assert_eq!(
            state.submit_result(1, 1, "result", (12, generation), "valid", &moved),
            Err(DeliveryError::StaleBasis)
        );
        assert_eq!(state.result_count(), 0);
        assert_eq!(state.acknowledgement_count(), 1);
    }
}
