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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    InvalidPrincipal,
    StalePrincipal,
    RevokedPrincipal,
    InvalidRequestId,
    ConflictingReplay,
    RecoveryRequired,
    Scope(AdmissionError),
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
    receipt: SimulationReceipt,
}

/// A deliberately non-durable, single-threaded model. Unlike production
/// authentication, the caller here is the trusted *test harness* itself.
/// Tests must not pass a client-supplied identity off as server authentication.
#[derive(Debug)]
pub struct SimulatedReplay {
    basis: ScopeConflictBasis,
    book: SimulationBook,
    principals: BTreeMap<u64, TestPrincipal>,
    requests: BTreeMap<(u64, u64, String), RecordedRequest>,
}

impl SimulatedReplay {
    pub fn new(basis: ScopeConflictBasis) -> Result<Self, AdmissionError> {
        let book = SimulationBook::new(basis.clone())?;
        Ok(Self {
            basis,
            book,
            principals: BTreeMap::new(),
            requests: BTreeMap::new(),
        })
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
        let task_ids: Vec<u64> = self
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
            if request.task_id != existing.task_id || footprint != existing.resources {
                return Err(ReplayError::ConflictingReplay);
            }
            if self.book.reservation(existing.task_id).map(|r| r.state)
                != Some(SimulationState::Active)
            {
                return Err(ReplayError::RecoveryRequired);
            }
            return Ok(existing.receipt.clone());
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
        }
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
        let first = state.poll(1, 1, &poll("a", 2, &[10, 20]), &basis()).unwrap();
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
        state.poll(1, 3, &poll("same", 11, &[100]), &basis()).unwrap();
        state.poll(2, 3, &poll("same", 12, &[200]), &basis()).unwrap();
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
            state.poll(2, 3, &poll("same", 12, &[200]), &basis())
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
        assert_eq!(state.enroll_test_principal(0, 1), Err(ReplayError::InvalidPrincipal));
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
}
