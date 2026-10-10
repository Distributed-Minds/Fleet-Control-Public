//! Conservative exact-key scope reservation simulation for FREE ENERGY #280.
//!
//! No network, credentials, provider authority, SQLite store, or production queue.
//! Real admission MUST use a current trusted resource registry, SQLite atomic
//! transaction and external #10/#43/#50 authority adapters. This module never
//! issues a runnable or provider-effect-capable assignment.

use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_RESOURCES: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeConflictBasis {
    pub version: u32,
    pub generation: u64,
    pub repository_id: u64,
    pub repository_incarnation: String,
}

impl ScopeConflictBasis {
    pub fn validate(&self) -> Result<(), AdmissionError> {
        if self.version != SCHEMA_VERSION
            || self.generation == 0
            || self.repository_id == 0
            || self.repository_incarnation.is_empty()
            || self.repository_incarnation.len() > 128
        {
            return Err(AdmissionError::InvalidBasis);
        }
        Ok(())
    }
}

/// Inputs deliberately limited to two exact resource classes. The issue and
/// pull-request API views of one GitHub number are ONE conversation resource.
/// Paths, ancestors, branch aliases and unknown resource families are refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeSelector {
    ExactBranchRef(String),
    Conversation(u64),
    Path(String),
    Alias(String),
    Hierarchical(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalResource {
    Branch {
        repository_id: u64,
        full_ref: String,
    },
    Conversation {
        repository_id: u64,
        number: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    InvalidBasis,
    StaleBasis,
    EmptyFootprint,
    TooManyResources,
    InvalidResource,
    UnsupportedScope,
    DuplicateResource,
    DuplicateTask,
    Collision { held_by_task: u64 },
    UnknownTask,
    GenerationExhausted,
}

/// Deliberately accept a strict ASCII subset of valid refs. Rejecting a valid
/// unfamiliar spelling is safer than normalizing two aliases as disjoint.
/// A trusted provider adapter must establish repository/ref identity later.
fn admitted_branch_ref(value: &str) -> bool {
    let Some(name) = value.strip_prefix("refs/heads/") else {
        return false;
    };
    if name.is_empty()
        || name.len() > 255
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.ends_with(".lock")
        || name.contains("..")
        || name.contains("//")
        || name.contains("@{")
    {
        return false;
    }
    name.split('/').all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && !segment.starts_with('.')
            && !segment.ends_with(".lock")
            && segment
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    })
}

/// Validate the entire proposed footprint BEFORE inspecting conflicting leases.
/// In this simulation a caller supplies the claimed basis; no caller-provided
/// hash/label is treated as evidence of disjointness or provider authority.
pub fn compile_footprint(
    selectors: &[ScopeSelector],
    claimed_basis: &ScopeConflictBasis,
    current_basis: &ScopeConflictBasis,
) -> Result<BTreeSet<CanonicalResource>, AdmissionError> {
    current_basis.validate()?;
    claimed_basis.validate()?;
    if claimed_basis != current_basis {
        return Err(AdmissionError::StaleBasis);
    }
    if selectors.is_empty() {
        return Err(AdmissionError::EmptyFootprint);
    }
    if selectors.len() > MAX_RESOURCES {
        return Err(AdmissionError::TooManyResources);
    }
    let mut keys = BTreeSet::new();
    for selector in selectors {
        let key = match selector {
            ScopeSelector::ExactBranchRef(name) if admitted_branch_ref(name) => {
                CanonicalResource::Branch {
                    repository_id: current_basis.repository_id,
                    full_ref: name.clone(),
                }
            }
            ScopeSelector::ExactBranchRef(_) => return Err(AdmissionError::InvalidResource),
            ScopeSelector::Conversation(number) if *number > 0 => {
                CanonicalResource::Conversation {
                    repository_id: current_basis.repository_id,
                    number: *number,
                }
            }
            ScopeSelector::Conversation(_) => return Err(AdmissionError::InvalidResource),
            ScopeSelector::Path(_)
            | ScopeSelector::Alias(_)
            | ScopeSelector::Hierarchical(_) => return Err(AdmissionError::UnsupportedScope),
        };
        if !keys.insert(key) {
            return Err(AdmissionError::DuplicateResource);
        }
    }
    Ok(keys)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    Active,
    RecoveryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationReceipt {
    pub task_id: u64,
    pub assignment_generation: u64,
    pub state: SimulationState,
    pub resources: BTreeSet<CanonicalResource>,
}

/// A purely deterministic test model, NOT an allocator suitable for workers.
/// Storage is in-memory and offers no concurrency, persistence, authorization,
/// lease expiry, cancellation, ACK, rollback or GitHub effect semantics.
#[derive(Debug)]
pub struct SimulationBook {
    basis: ScopeConflictBasis,
    next_generation: u64,
    reservations: BTreeMap<u64, SimulationReceipt>,
}

impl SimulationBook {
    pub fn new(basis: ScopeConflictBasis) -> Result<Self, AdmissionError> {
        basis.validate()?;
        Ok(Self {
            basis,
            next_generation: 0,
            reservations: BTreeMap::new(),
        })
    }

    pub fn reserve(
        &mut self,
        task_id: u64,
        selectors: &[ScopeSelector],
        current_basis: &ScopeConflictBasis,
    ) -> Result<&SimulationReceipt, AdmissionError> {
        if current_basis != &self.basis {
            return Err(AdmissionError::StaleBasis);
        }
        let resources = compile_footprint(selectors, &self.basis, current_basis)?;
        if task_id == 0 {
            return Err(AdmissionError::InvalidResource);
        }
        if self.reservations.contains_key(&task_id) {
            return Err(AdmissionError::DuplicateTask);
        }
        for old in self.reservations.values() {
            if !old.resources.is_disjoint(&resources) {
                return Err(AdmissionError::Collision {
                    held_by_task: old.task_id,
                });
            }
        }
        let generation = self
            .next_generation
            .checked_add(1)
            .ok_or(AdmissionError::GenerationExhausted)?;
        self.next_generation = generation;
        self.reservations.insert(
            task_id,
            SimulationReceipt {
                task_id,
                assignment_generation: generation,
                state: SimulationState::Active,
                resources,
            },
        );
        Ok(self.reservations.get(&task_id).expect("inserted task"))
    }

    /// Lost cutoff evidence causes a permanent hold in this model. No public
    /// release shortcut converts expired time into a safe reallocation.
    pub fn require_recovery(&mut self, task_id: u64) -> Result<(), AdmissionError> {
        let Some(receipt) = self.reservations.get_mut(&task_id) else {
            return Err(AdmissionError::UnknownTask);
        };
        receipt.state = SimulationState::RecoveryRequired;
        Ok(())
    }

    pub fn reservation(&self, task_id: u64) -> Option<&SimulationReceipt> {
        self.reservations.get(&task_id)
    }

    pub fn reservation_count(&self) -> usize {
        self.reservations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basis() -> ScopeConflictBasis {
        ScopeConflictBasis {
            version: SCHEMA_VERSION,
            generation: 7,
            repository_id: 1360059617,
            repository_incarnation: "repo-incarnation-1".to_owned(),
        }
    }

    fn branch(name: &str) -> ScopeSelector {
        ScopeSelector::ExactBranchRef(format!("refs/heads/{name}"))
    }

    #[test]
    fn same_canonical_branch_collides_even_with_different_task_ids() {
        let mut book = SimulationBook::new(basis()).unwrap();
        book.reserve(1, &[branch("feature/a")], &basis()).unwrap();
        assert_eq!(
            book.reserve(2, &[branch("feature/a")], &basis()),
            Err(AdmissionError::Collision { held_by_task: 1 })
        );
    }

    #[test]
    fn multi_key_second_resource_collision_is_atomic() {
        let mut book = SimulationBook::new(basis()).unwrap();
        book.reserve(4, &[branch("shared")], &basis()).unwrap();
        assert_eq!(
            book.reserve(5, &[branch("free"), branch("shared")], &basis()),
            Err(AdmissionError::Collision { held_by_task: 4 })
        );
        assert_eq!(book.reservation_count(), 1);
        assert!(book.reservation(5).is_none());
        book.reserve(6, &[branch("free")], &basis()).unwrap();
    }

    #[test]
    fn disjoint_tasks_and_reversed_arrival_are_deterministic() {
        for first in [10, 20] {
            let mut book = SimulationBook::new(basis()).unwrap();
            let second = if first == 10 { 20 } else { 10 };
            let r1 = book.reserve(first, &[branch("a")], &basis()).unwrap();
            assert_eq!(r1.assignment_generation, 1);
            let r2 = book.reserve(second, &[branch("b")], &basis()).unwrap();
            assert_eq!(r2.assignment_generation, 2);
            assert_eq!(book.reservation_count(), 2);
        }
    }

    #[test]
    fn recovery_hold_continues_to_reserve_scope() {
        let mut book = SimulationBook::new(basis()).unwrap();
        book.reserve(1, &[ScopeSelector::Conversation(7)], &basis())
            .unwrap();
        book.require_recovery(1).unwrap();
        assert_eq!(
            book.reservation(1).unwrap().state,
            SimulationState::RecoveryRequired
        );
        assert_eq!(
            book.reserve(2, &[ScopeSelector::Conversation(7)], &basis()),
            Err(AdmissionError::Collision { held_by_task: 1 })
        );
    }

    #[test]
    fn stale_basis_and_reincarnated_repository_are_rejected() {
        let mut book = SimulationBook::new(basis()).unwrap();
        let mut changed = basis();
        changed.generation += 1;
        assert_eq!(
            book.reserve(1, &[branch("a")], &changed),
            Err(AdmissionError::StaleBasis)
        );
        changed = basis();
        changed.repository_incarnation = "restored-repo".into();
        assert_eq!(
            book.reserve(1, &[branch("a")], &changed),
            Err(AdmissionError::StaleBasis)
        );
        assert_eq!(book.reservation_count(), 0);
    }

    #[test]
    fn aliases_hierarchies_paths_and_malformed_refs_fail_closed() {
        for selector in [
            ScopeSelector::Path("src/".into()),
            ScopeSelector::Alias("HEAD".into()),
            ScopeSelector::Hierarchical("refs/heads/*".into()),
            branch("../main"),
            branch("a//b"),
            branch("a.lock"),
            ScopeSelector::ExactBranchRef("main".into()),
            ScopeSelector::Conversation(0),
        ] {
            assert!(compile_footprint(&[selector], &basis(), &basis()).is_err());
        }
    }

    #[test]
    fn partial_footprint_never_reserves_any_keys() {
        let mut book = SimulationBook::new(basis()).unwrap();
        let result = book.reserve(
            2,
            &[branch("free"), ScopeSelector::Alias("unknown".into())],
            &basis(),
        );
        assert_eq!(result, Err(AdmissionError::UnsupportedScope));
        assert_eq!(book.reservation_count(), 0);
        book.reserve(3, &[branch("free")], &basis()).unwrap();
    }

    #[test]
    fn duplicate_resource_and_unknown_schema_fail_closed() {
        assert_eq!(
            compile_footprint(&[branch("a"), branch("a")], &basis(), &basis()),
            Err(AdmissionError::DuplicateResource)
        );
        let mut bad = basis();
        bad.version = 2;
        assert_eq!(
            compile_footprint(&[branch("a")], &bad, &basis()),
            Err(AdmissionError::InvalidBasis)
        );
    }
}
