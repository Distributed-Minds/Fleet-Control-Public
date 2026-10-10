//! Offline synthetic lineage admission for issue #11 spec 6.
//! Fixture "trust" is caller data, never proof of real provider authority.
use std::collections::{BTreeMap, BTreeSet};

// Offline fixture identifiers must not silently collapse to an absent scope.
// This is synthetic admission only, not verification of external identity.
fn valid_identity(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

impl Scope {
    fn is_valid(&self) -> bool {
        self.repository != 0
            && valid_identity(&self.repo_incarnation)
            && valid_identity(&self.installation)
            && valid_identity(&self.tenant)
            && valid_identity(&self.namespace)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub repository: u64,
    pub repo_incarnation: String,
    pub installation: String,
    pub tenant: String,
    pub namespace: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub scope: Scope,
    pub head: String,
    pub generation: u64,
    pub policy: u64,
    pub obligations: BTreeSet<String>,
    pub selected_issuer: bool,
    pub unique_current: bool,
    pub frontier_complete: bool,
    pub forked: bool,
    pub revoked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub id: String,
    pub scope: Scope,
    pub predecessor: String,
    pub previous_generation: u64,
    pub policy: u64,
    pub successor: String,
    pub new_head: String,
    pub admitted_by_issuer: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub transition: Transition,
    pub generation: u64,
    pub obligations: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct EffectInput {
    pub cut: Selection,
    pub operation_id: String,
    pub actor: String,
    pub authorized_action: bool,
    pub recovery_required: bool,
    pub recovery_authorized: bool,
    pub observed_resource: String,
    pub expected_resource: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denied {
    UnknownLineage,
    Scope,
    Stale,
    ReplayConflict,
    InvalidTransition,
    NoActionAuthority,
    WrongResource,
    RecoveryHold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eligibility {
    SimulationOnly,
}

/// In-process fake CAS: no persistence, authentic issuer or external side effects.
pub struct Registry {
    scope: Scope,
    head: String,
    generation: u64,
    policy: u64,
    obligations: BTreeSet<String>,
    seen_heads: BTreeSet<String>,
    receipts: BTreeMap<String, Receipt>,
}

impl Registry {
    pub fn new(
        scope: Scope,
        head: String,
        generation: u64,
        policy: u64,
        obligations: BTreeSet<String>,
    ) -> Self {
        let mut seen_heads = BTreeSet::new();
        seen_heads.insert(head.clone());
        Self {
            scope,
            head,
            generation,
            policy,
            obligations,
            seen_heads,
            receipts: BTreeMap::new(),
        }
    }

    fn selection_ok(&self, s: &Selection) -> Result<(), Denied> {
        if !self.scope.is_valid()
            || !valid_identity(&self.head)
            || self.obligations.iter().any(|id| !valid_identity(id))
        {
            return Err(Denied::UnknownLineage);
        }
        if !s.selected_issuer || !s.unique_current || !s.frontier_complete || s.forked || s.revoked
        {
            return Err(Denied::UnknownLineage);
        }
        if s.scope != self.scope {
            return Err(Denied::Scope);
        }
        if s.head != self.head || s.generation != self.generation || s.policy != self.policy {
            return Err(Denied::Stale);
        }
        if s.obligations != self.obligations {
            return Err(Denied::UnknownLineage);
        }
        Ok(())
    }

    /// Exact-operation retry returns historical receipt, not fresh authority.
    pub fn admit(
        &mut self,
        selected: &Selection,
        transition: Transition,
    ) -> Result<Receipt, Denied> {
        if let Some(previous) = self.receipts.get(&transition.id) {
            return if previous.transition == transition {
                Ok(previous.clone())
            } else {
                Err(Denied::ReplayConflict)
            };
        }
        self.selection_ok(selected)?;
        if transition.scope != self.scope {
            return Err(Denied::Scope);
        }
        if !transition.admitted_by_issuer
            || !valid_identity(&transition.id)
            || !valid_identity(&transition.successor)
            || !valid_identity(&transition.new_head)
            || transition.new_head == transition.predecessor
        {
            return Err(Denied::InvalidTransition);
        }
        if transition.predecessor != self.head
            || transition.previous_generation != self.generation
            || transition.policy != self.policy
        {
            return Err(Denied::Stale);
        }
        if self.seen_heads.contains(&transition.new_head) {
            return Err(Denied::ReplayConflict);
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(Denied::InvalidTransition)?;
        let receipt = Receipt {
            transition: transition.clone(),
            generation,
            obligations: self.obligations.clone(),
        };
        self.generation = generation;
        self.head = transition.new_head;
        self.seen_heads.insert(self.head.clone());
        self.receipts.insert(transition.id, receipt.clone());
        Ok(receipt)
    }

    /// Read-only synthetic decision. Never issue an actual provider command.
    pub fn effect(&self, receipt: &Receipt, request: &EffectInput) -> Result<Eligibility, Denied> {
        self.selection_ok(&request.cut)?;
        if request.operation_id != receipt.transition.id
            || self.receipts.get(&request.operation_id) != Some(receipt)
            || receipt.transition.new_head != self.head
            || receipt.generation != self.generation
            || receipt.obligations != self.obligations
            || request.actor != receipt.transition.successor
        {
            return Err(Denied::Stale);
        }
        if !request.authorized_action {
            return Err(Denied::NoActionAuthority);
        }
        if request.observed_resource.is_empty()
            || request.observed_resource != request.expected_resource
        {
            return Err(Denied::WrongResource);
        }
        if request.recovery_required && !request.recovery_authorized {
            return Err(Denied::RecoveryHold);
        }
        Ok(Eligibility::SimulationOnly)
    }

    pub fn head(&self) -> (&str, u64) {
        (&self.head, self.generation)
    }

    pub fn admitted_count(&self) -> usize {
        self.receipts.len()
    }
}
