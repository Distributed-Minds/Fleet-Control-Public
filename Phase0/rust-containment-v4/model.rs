//! Issue #23, spec 4: deliberately non-authoritative, offline-only model.
//! This module cannot call providers, issue containment orders, or restore privileges.
//! In particular, SyntheticOnly is never an external authorization receipt.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    Read,
    Write,
    Delete,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Resource {
    pub principal: String,
    pub incarnation: u64,
    pub namespace: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Interpretation {
    Incriminating,
    Exculpatory,
    Neutral,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Observation {
    pub id: String,
    pub digest: String,
    pub lineage: String,
    pub interpretation: Interpretation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frontier {
    pub policy_generation: u64,
    pub partition_generation: u64,
    pub complete: bool,
    pub observations: Vec<Observation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    IncompleteEvidence,
    ConflictingEvidence,
    UnprovedControl,
    IncomparableControl,
    IncompleteActiveSet,
    WrongIncarnation,
    StaleGeneration,
    ConflictingReplay,
    ExistingCause,
    MissingCause,
    UnknownDependency,
    StaleDecision,
    GenerationExhausted,
}

impl Frontier {
    /// Structural canonicalization, not a proof that the supplied source is trusted.
    pub fn normalize(mut self) -> Result<Self, Error> {
        if !self.complete || self.policy_generation == 0 || self.partition_generation == 0 {
            return Err(Error::IncompleteEvidence);
        }
        self.observations.sort();
        self.observations.dedup();
        if self.observations.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(Error::ConflictingEvidence);
        }
        Ok(self)
    }

    /// Correlated reports sharing a lineage never become independent votes.
    /// Reject malformed snapshots even when queried without prior normalization.
    pub fn adverse_lineages(&self) -> Result<usize, Error> {
        let canonical = self.clone().normalize()?;
        Ok(canonical
            .observations
            .iter()
            .filter(|o| o.interpretation == Interpretation::Incriminating)
            .map(|o| &o.lineage)
            .collect::<BTreeSet<_>>()
            .len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Control {
    pub name: String,
    pub denied: BTreeSet<Action>,
    pub harm: [u16; 3], // Comparable dimensions only under the SAME policy.
    pub comparison_policy: u64,
    // Caller-supplied immutable snapshot identities. They are compared, NOT
    // authenticated by this offline model. Real authority must prove currentness.
    pub risk_scope_basis: String,
    pub effectiveness_basis: String,
    pub harm_basis: String,
    pub capability_basis: String,
    pub policy_basis: String,
    pub independently_verified: bool,
    pub effective: bool,
    pub available: bool,
}

/// Canonical complete candidate catalog: all options matter, not only the
/// selected name. No duplicate names, unverified candidates or missing bases.
fn canonical_controls(controls: &[Control]) -> Result<Vec<Control>, Error> {
    let mut entries = controls.to_vec();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    if entries.is_empty()
        || entries.iter().any(|c| {
            c.name.is_empty()
                || c.comparison_policy == 0
                || c.risk_scope_basis.is_empty()
                || c.effectiveness_basis.is_empty()
                || c.harm_basis.is_empty()
                || c.capability_basis.is_empty()
                || c.policy_basis.is_empty()
                || !c.independently_verified
        })
        || entries.windows(2).any(|pair| pair[0].name == pair[1].name)
    {
        return Err(Error::UnprovedControl);
    }
    Ok(entries)
}

/// Returns a *synthetic choice*, never an operational containment instruction.
pub fn choose_control(controls: &[Control]) -> Result<String, Error> {
    let catalog = canonical_controls(controls)?;
    let first = &catalog[0];
    if catalog.iter().any(|c| c.comparison_policy != first.comparison_policy) {
        // An unverified narrower option cannot be discarded to justify a broad one.
        return Err(Error::UnprovedControl);
    }
    // Equal numeric harm scores cannot be ordered across incompatible incident,
    // harm, capability or policy bases. A separately certified conversion could
    // admit other cases; this offline model has no such compatibility witness.
    // Compare the full candidate catalog, including unavailable alternatives.
    if catalog.iter().any(|c| {
        c.risk_scope_basis.as_str() != first.risk_scope_basis.as_str()
            || c.harm_basis.as_str() != first.harm_basis.as_str()
            || c.capability_basis.as_str() != first.capability_basis.as_str()
            || c.policy_basis.as_str() != first.policy_basis.as_str()
    }) {
        return Err(Error::IncomparableControl);
    }
    let eligible: Vec<&Control> = catalog
        .iter()
        .filter(|c| c.available && c.effective)
        .collect();
    let mut winners: Vec<&Control> = eligible
        .iter()
        .copied()
        .filter(|candidate| {
            eligible.iter().all(|other| {
                candidate.denied.is_subset(&other.denied)
                    && candidate
                        .harm
                        .iter()
                        .zip(other.harm.iter())
                        .all(|(x, y)| x <= y)
            })
        })
        .collect();
    if winners.is_empty() {
        return Err(Error::IncomparableControl);
    }
    winners.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(winners[0].name.clone())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeKind {
    Add { cause: String, denied: BTreeSet<Action> },
    Recover { cause: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Change {
    pub operation_id: String,
    pub resource: Resource,
    pub expected_generation: u64,
    pub kind: ChangeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Apply {
    New,
    Replayed,
}

#[derive(Clone, Debug)]
pub struct ActiveSet {
    pub resource: Resource,
    pub generation: u64,
    pub complete: bool,
    pub base_allowed: BTreeSet<Action>,
    restrictions: BTreeMap<String, BTreeSet<Action>>,
    receipts: BTreeMap<String, Change>,
}

impl ActiveSet {
    pub fn new(resource: Resource, base_allowed: BTreeSet<Action>) -> Self {
        Self {
            resource,
            generation: 1,
            complete: true,
            base_allowed,
            restrictions: BTreeMap::new(),
            receipts: BTreeMap::new(),
        }
    }

    /// Local simulation only. The caller does NOT confer base-authority legitimacy.
    pub fn model_base_revocation(
        &mut self,
        expected_generation: u64,
        action: Action,
    ) -> Result<(), Error> {
        self.check(expected_generation, &self.resource)?;
        self.bump()?;
        self.base_allowed.remove(&action);
        Ok(())
    }

    fn check(&self, expected: u64, resource: &Resource) -> Result<(), Error> {
        if !self.complete {
            return Err(Error::IncompleteActiveSet);
        }
        if &self.resource != resource {
            return Err(Error::WrongIncarnation);
        }
        if expected != self.generation {
            return Err(Error::StaleGeneration);
        }
        Ok(())
    }

    fn bump(&mut self) -> Result<(), Error> {
        self.generation = self.generation.checked_add(1).ok_or(Error::GenerationExhausted)?;
        Ok(())
    }

    /// Replay identity is checked before generation, so an exact lost ACK is idempotent.
    pub fn apply(&mut self, command: Change) -> Result<Apply, Error> {
        if let Some(previous) = self.receipts.get(&command.operation_id) {
            return if previous == &command {
                Ok(Apply::Replayed)
            } else {
                Err(Error::ConflictingReplay)
            };
        }
        self.check(command.expected_generation, &command.resource)?;
        if self.generation == u64::MAX {
            return Err(Error::GenerationExhausted);
        }
        match &command.kind {
            ChangeKind::Add { cause, denied } => {
                if self.restrictions.contains_key(cause) {
                    return Err(Error::ExistingCause);
                }
                self.restrictions.insert(cause.clone(), denied.clone());
            }
            ChangeKind::Recover { cause } => {
                if self.restrictions.remove(cause).is_none() {
                    return Err(Error::MissingCause);
                }
            }
        }
        self.bump()?;
        self.receipts.insert(command.operation_id.clone(), command);
        Ok(Apply::New)
    }

    /// Read-only conservative permission evaluation, NOT an external effect gate.
    pub fn synthetic_allowed(&self, resource: &Resource, action: Action) -> Result<bool, Error> {
        if !self.complete {
            return Err(Error::IncompleteActiveSet);
        }
        if resource != &self.resource {
            // No assumed alias or namespace mapping.
            return Err(Error::WrongIncarnation);
        }
        Ok(self.base_allowed.contains(&action)
            && self.restrictions.values().all(|denied| !denied.contains(&action)))
    }

    pub fn causes(&self) -> usize {
        self.restrictions.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionBasis {
    pub resource: Resource,
    pub frontier: Frontier,
    pub chosen_control: String,
    // Full snapshot of all compared alternatives, canonically ordered by name.
    // Binding only chosen_control would miss name-stable policy/harm movement.
    pub control_catalog: Vec<Control>,
    pub active_generation: u64,
    pub dependency_bases: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Review {
    SyntheticOnly,
}

/// Rechecks each declared basis against one injected snapshot. It CANNOT prove
/// source-coherent acquisition, #10/#13/#16 authority or provider atomicity.
pub fn review(
    state: &ActiveSet,
    bound: &DecisionBasis,
    live_frontier: Frontier,
    live_controls: &[Control],
    live_dependencies: &BTreeMap<String, String>,
) -> Result<Review, Error> {
    if !state.complete {
        return Err(Error::IncompleteActiveSet);
    }
    if state.resource != bound.resource {
        return Err(Error::WrongIncarnation);
    }
    if state.generation != bound.active_generation {
        return Err(Error::StaleGeneration);
    }
    // These are the three dependency slots required by the spec-4 offline
    // review envelope. Equality of incomplete maps is not completeness proof.
    const REQUIRED_DEPS: [&str; 3] = ["#10", "#13", "#16"];
    if REQUIRED_DEPS.iter().any(|key| {
        !matches!(bound.dependency_bases.get(*key), Some(value) if !value.is_empty())
            || !matches!(live_dependencies.get(*key), Some(value) if !value.is_empty())
    })
        || bound.dependency_bases.values().any(String::is_empty)
        || live_dependencies.values().any(String::is_empty)
    {
        return Err(Error::UnknownDependency);
    }
    let bound_controls = canonical_controls(&bound.control_catalog)?;
    let current_controls = canonical_controls(live_controls)?;
    if bound.frontier != live_frontier.normalize()?
        || bound_controls != current_controls
        || bound.chosen_control != choose_control(&current_controls)?
        || bound.dependency_bases != *live_dependencies
    {
        return Err(Error::StaleDecision);
    }
    Ok(Review::SyntheticOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource() -> Resource {
        Resource { principal: "actor".into(), incarnation: 7, namespace: "repo/a".into() }
    }
    fn set(actions: &[Action]) -> BTreeSet<Action> {
        actions.iter().copied().collect()
    }
    fn state() -> ActiveSet {
        ActiveSet::new(resource(), set(&[Action::Read, Action::Write, Action::Delete]))
    }
    fn command(id: &str, generation: u64, kind: ChangeKind) -> Change {
        Change { operation_id: id.into(), resource: resource(), expected_generation: generation, kind }
    }
    fn add(id: &str, cause: &str, generation: u64, denied: &[Action]) -> Change {
        command(id, generation, ChangeKind::Add { cause: cause.into(), denied: set(denied) })
    }
    fn recover(id: &str, cause: &str, generation: u64) -> Change {
        command(id, generation, ChangeKind::Recover { cause: cause.into() })
    }
    fn observation(id: &str, lineage: &str, interpretation: Interpretation) -> Observation {
        Observation { id: id.into(), digest: id.into(), lineage: lineage.into(), interpretation }
    }
    fn frontier() -> Frontier {
        Frontier {
            policy_generation: 1,
            partition_generation: 1,
            complete: true,
            observations: vec![
                observation("E2", "L1", Interpretation::Incriminating),
                observation("E1", "L1", Interpretation::Incriminating),
            ],
        }.normalize().unwrap()
    }
    fn control(name: &str, denied: &[Action], harm: [u16; 3]) -> Control {
        Control {
            name: name.into(), denied: set(denied), harm,
            comparison_policy: 1,
            risk_scope_basis: "risk/subject/scope@1".into(),
            effectiveness_basis: "independent-effectiveness@1".into(),
            harm_basis: "independent-harm@1".into(),
            capability_basis: "complete-inventory@1".into(),
            policy_basis: "comparison-policy-content@1".into(),
            independently_verified: true, effective: true, available: true,
        }
    }
    fn deps() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("#10".into(), "authority-v1@exact".into()),
            ("#13".into(), "evidence-v1@exact".into()),
            ("#16".into(), "closure-v1@exact".into()),
        ])
    }
    fn basis(s: &ActiveSet) -> DecisionBasis {
        DecisionBasis {
            resource: resource(), frontier: frontier(), chosen_control: "narrow".into(),
            control_catalog: vec![control("narrow", &[Action::Write], [1, 1, 1])],
            active_generation: s.generation, dependency_bases: deps(),
        }
    }

    #[test]
    fn evidence_order_and_correlation() {
        let mut reversed = frontier();
        reversed.observations.reverse();
        assert_eq!(frontier(), reversed.normalize().unwrap());
        assert_eq!(frontier().adverse_lineages(), Ok(1));
    }

    #[test]
    fn older_newly_eligible_evidence_changes_complete_frontier() {
        let initial = frontier();
        let mut changed = initial.clone();
        changed.observations.push(observation("E0", "L2", Interpretation::Exculpatory));
        assert_ne!(initial, changed.normalize().unwrap());
        // High-water E2 did not change; membership did.
    }

    #[test]
    fn lineage_reclassification_changes_basis() {
        let mut changed = frontier();
        changed.observations[0].lineage = "independent".into();
        assert_ne!(changed.normalize().unwrap(), frontier());
    }

    #[test]
    fn incomplete_and_conflicting_evidence_fail_closed() {
        let mut f = frontier();
        f.complete = false;
        assert_eq!(f.normalize(), Err(Error::IncompleteEvidence));
        let mut f = frontier();
        let mut conflicting = f.observations[0].clone();
        conflicting.digest = "other".into();
        f.observations.push(conflicting);
        assert_eq!(f.normalize(), Err(Error::ConflictingEvidence));
    }

    #[test]
    fn direct_lineage_query_rejects_conflicting_observation_identity() {
        let f = frontier();
        assert_eq!(f.adverse_lineages(), Ok(1));
        let mut conflicted = f.clone();
        let mut changed = conflicted.observations[0].clone();
        changed.lineage = "distinct-lineage".into();
        conflicted.observations.push(changed);
        assert_eq!(conflicted.clone().normalize(), Err(Error::ConflictingEvidence));
        // RED before the fix: the public method counts both lineages anyway.
        assert_eq!(conflicted.adverse_lineages(), Err(Error::ConflictingEvidence));
    }

    #[test]
    fn direct_lineage_query_rejects_invalid_frontier_generations() {
        let f = frontier();
        assert_eq!(f.adverse_lineages(), Ok(1));

        let mut policy_unknown = f.clone();
        policy_unknown.policy_generation = 0;
        assert_eq!(policy_unknown.adverse_lineages(), Err(Error::IncompleteEvidence));

        let mut partition_unknown = f;
        partition_unknown.partition_generation = 0;
        assert_eq!(partition_unknown.adverse_lineages(), Err(Error::IncompleteEvidence));
    }

    #[test]
    fn independent_comparable_narrow_control_wins() {
        let narrow = control("narrow", &[Action::Write], [1, 1, 1]);
        let broad = control("broad", &[Action::Write, Action::Delete], [2, 3, 2]);
        assert_eq!(choose_control(&[broad, narrow]), Ok("narrow".into()));
    }

    #[test]
    fn unproven_narrow_option_cannot_force_broad_escalation() {
        let mut narrow = control("narrow", &[Action::Write], [1, 1, 1]);
        narrow.independently_verified = false;
        assert_eq!(choose_control(&[narrow, control("broad", &[Action::Write, Action::Delete], [2, 2, 2])]), Err(Error::UnprovedControl));
    }

    #[test]
    fn incomparable_harm_is_not_scalarized() {
        let a = control("a", &[Action::Write], [1, 3, 1]);
        let b = control("b", &[Action::Write], [3, 1, 1]);
        assert_eq!(choose_control(&[a, b]), Err(Error::IncomparableControl));
    }

    #[test]
    fn overlapping_recovery_never_clears_other_causes() {
        let mut s = state();
        assert_eq!(s.apply(add("a", "C1", 1, &[Action::Write])), Ok(Apply::New));
        assert_eq!(s.apply(add("b", "C2", 2, &[Action::Write, Action::Delete])), Ok(Apply::New));
        assert_eq!(s.apply(recover("c", "C1", 3)), Ok(Apply::New));
        assert_eq!(s.causes(), 1);
        assert_eq!(s.synthetic_allowed(&resource(), Action::Write), Ok(false));
        assert_eq!(s.synthetic_allowed(&resource(), Action::Delete), Ok(false));
        assert_eq!(s.synthetic_allowed(&resource(), Action::Read), Ok(true));
    }

    #[test]
    fn independent_base_revocation_survives_all_recoveries() {
        let mut s = state();
        s.apply(add("a", "C1", 1, &[Action::Write])).unwrap();
        s.model_base_revocation(2, Action::Write).unwrap();
        s.apply(recover("b", "C1", 3)).unwrap();
        assert_eq!(s.causes(), 0);
        assert_eq!(s.synthetic_allowed(&resource(), Action::Write), Ok(false));
    }

    #[test]
    fn opposite_recovery_orders_preserve_remaining_intersection() {
        for reverse in [false, true] {
            let mut s = state();
            s.apply(add("a", "C1", 1, &[Action::Write])).unwrap();
            s.apply(add("b", "C2", 2, &[Action::Delete])).unwrap();
            let (first, second) = if reverse { ("C2", "C1") } else { ("C1", "C2") };
            s.apply(recover("c", first, 3)).unwrap();
            let still_denied = if first == "C1" { Action::Delete } else { Action::Write };
            assert_eq!(s.synthetic_allowed(&resource(), still_denied), Ok(false));
            s.apply(recover("d", second, 4)).unwrap();
            assert_eq!(s.synthetic_allowed(&resource(), Action::Write), Ok(true));
            assert_eq!(s.synthetic_allowed(&resource(), Action::Delete), Ok(true));
        }
    }

    #[test]
    fn generation_cas_rejects_second_writer_on_same_prestate() {
        let mut s = state();
        s.apply(add("a", "C1", 1, &[Action::Write])).unwrap();
        assert_eq!(s.apply(add("b", "C2", 1, &[Action::Delete])), Err(Error::StaleGeneration));
        assert_eq!(s.causes(), 1);
    }

    #[test]
    fn exact_ack_loss_replay_is_idempotent_but_changed_payload_conflicts() {
        let mut s = state();
        let first = add("a", "C1", 1, &[Action::Write]);
        assert_eq!(s.apply(first.clone()), Ok(Apply::New));
        assert_eq!(s.apply(first), Ok(Apply::Replayed));
        assert_eq!(s.generation, 2);
        assert_eq!(s.apply(add("a", "C1", 1, &[Action::Delete])), Err(Error::ConflictingReplay));
    }

    #[test]
    fn replaced_resource_and_incomplete_inventory_never_restore() {
        let mut s = state();
        let mut replacement = resource();
        replacement.incarnation += 1;
        assert_eq!(s.synthetic_allowed(&replacement, Action::Write), Err(Error::WrongIncarnation));
        let cmd = Change { resource: replacement, ..add("a", "C1", 1, &[Action::Write]) };
        assert_eq!(s.apply(cmd), Err(Error::WrongIncarnation));
        s.complete = false;
        assert_eq!(s.synthetic_allowed(&resource(), Action::Read), Err(Error::IncompleteActiveSet));
        assert_eq!(s.apply(add("b", "C1", 1, &[Action::Write])), Err(Error::IncompleteActiveSet));
    }

    #[test]
    fn joint_model_basis_detects_evidence_dependency_and_set_movement() {
        let mut s = state();
        let bound = basis(&s);
        let controls = [control("narrow", &[Action::Write], [1, 1, 1])];
        assert_eq!(review(&s, &bound, frontier(), &controls, &deps()), Ok(Review::SyntheticOnly));
        let mut moved = frontier();
        moved.observations.push(observation("E0", "L0", Interpretation::Exculpatory));
        assert_eq!(review(&s, &bound, moved, &controls, &deps()), Err(Error::StaleDecision));
        let mut moved_deps = deps();
        moved_deps.insert("#13".into(), "evidence-v2".into());
        assert_eq!(review(&s, &bound, frontier(), &controls, &moved_deps), Err(Error::StaleDecision));
        s.apply(add("a", "C1", 1, &[Action::Write])).unwrap();
        assert_eq!(review(&s, &bound, frontier(), &controls, &deps()), Err(Error::StaleGeneration));
    }

    #[test]
    fn no_provider_basis_means_no_even_synthetic_candidate() {
        let s = state();
        let bound = basis(&s);
        let controls = [control("narrow", &[Action::Write], [1, 1, 1])];
        assert_eq!(review(&s, &bound, frontier(), &controls, &BTreeMap::new()), Err(Error::UnknownDependency));
    }

    #[test]
    fn no_restrictions_returns_only_current_base_permission() {
        let mut s = state();
        s.model_base_revocation(1, Action::Delete).unwrap();
        assert_eq!(s.synthetic_allowed(&resource(), Action::Read), Ok(true));
        assert_eq!(s.synthetic_allowed(&resource(), Action::Delete), Ok(false));
    }

    #[test]
    fn exhaustive_three_cause_add_recovery_interleavings_never_widen_early() {
        let permutations = [
            [0usize, 1, 2], [0, 2, 1], [1, 0, 2],
            [1, 2, 0], [2, 0, 1], [2, 1, 0],
        ];
        let bases = [
            set(&[Action::Read, Action::Write, Action::Delete]),
            set(&[Action::Read, Action::Write]),
            set(&[Action::Read]),
        ];
        let denials = [
            set(&[Action::Write]),
            set(&[Action::Write, Action::Delete]),
            set(&[Action::Delete]),
        ];
        let actions = [Action::Read, Action::Write, Action::Delete];
        let mut compared = 0usize;
        for base in bases {
            for addition in permutations {
                for recovery in permutations {
                    let mut state = ActiveSet::new(resource(), base.clone());
                    let mut active = [false; 3];
                    for index in addition {
                        state.apply(add(
                            &format!("add-{index}"),
                            &format!("cause-{index}"),
                            state.generation,
                            &denials[index].iter().copied().collect::<Vec<_>>(),
                        )).unwrap();
                        active[index] = true;
                        for action in actions {
                            let permitted = base.contains(&action)
                                && !denials.iter().enumerate().any(|(i, blocked)| {
                                    active[i] && blocked.contains(&action)
                                });
                            assert_eq!(state.synthetic_allowed(&resource(), action), Ok(permitted));
                            compared += 1;
                        }
                    }
                    for index in recovery {
                        state.apply(recover(
                            &format!("recover-{index}"),
                            &format!("cause-{index}"),
                            state.generation,
                        )).unwrap();
                        active[index] = false;
                        for action in actions {
                            let permitted = base.contains(&action)
                                && !denials.iter().enumerate().any(|(i, blocked)| {
                                    active[i] && blocked.contains(&action)
                                });
                            assert_eq!(state.synthetic_allowed(&resource(), action), Ok(permitted));
                            compared += 1;
                        }
                    }
                    assert_eq!(state.causes(), 0);
                }
            }
        }
        assert_eq!(compared, 1944);
    }

    #[test]
    fn same_control_name_cannot_hide_changed_harm_scope_or_proof_basis() {
        let s = state();
        let bound = basis(&s);
        let original = bound.control_catalog[0].clone();
        assert_eq!(
            review(&s, &bound, frontier(), &[original.clone()], &deps()),
            Ok(Review::SyntheticOnly)
        );
        let mut changed = Vec::new();
        let mut x = original.clone();
        x.harm = [9, 9, 9];
        changed.push(x);
        let mut x = original.clone();
        x.denied.insert(Action::Delete);
        changed.push(x);
        let mut x = original.clone();
        x.risk_scope_basis = "another-subject-or-risk".into();
        changed.push(x);
        let mut x = original.clone();
        x.effectiveness_basis = "new-effectiveness-proof".into();
        changed.push(x);
        let mut x = original.clone();
        x.harm_basis = "new-harm-proof".into();
        changed.push(x);
        let mut x = original.clone();
        x.capability_basis = "new-capability-inventory".into();
        changed.push(x);
        let mut x = original.clone();
        x.policy_basis = "same-generation-but-different-policy-content".into();
        changed.push(x);
        let mut x = original.clone();
        x.comparison_policy = 2;
        changed.push(x);
        assert_eq!(changed.len(), 8);
        for candidate in changed {
            assert_eq!(candidate.name, "narrow");
            assert_eq!(choose_control(&[candidate.clone()]), Ok("narrow".into()));
            assert_eq!(
                review(&s, &bound, frontier(), &[candidate], &deps()),
                Err(Error::StaleDecision)
            );
        }
    }

    #[test]
    fn all_candidates_are_bound_but_permutation_does_not_move_basis() {
        let s = state();
        let narrow = control("narrow", &[Action::Write], [1, 1, 1]);
        let broad = control("broad", &[Action::Write, Action::Delete], [2, 3, 2]);
        let mut bound = basis(&s);
        bound.control_catalog = vec![narrow.clone(), broad.clone()];
        assert_eq!(
            review(&s, &bound, frontier(), &[broad.clone(), narrow.clone()], &deps()),
            Ok(Review::SyntheticOnly)
        );
        assert_eq!(
            review(&s, &bound, frontier(), &[narrow.clone()], &deps()),
            Err(Error::StaleDecision)
        );
        assert_eq!(
            review(&s, &bound, frontier(), &[narrow.clone(), broad.clone(), broad], &deps()),
            Err(Error::UnprovedControl)
        );
    }

    #[test]
    fn incomplete_control_identity_never_becomes_synthetic_acceptance() {
        let s = state();
        let bound = basis(&s);
        let original = bound.control_catalog[0].clone();
        for missing in 0..5 {
            let mut changed = original.clone();
            match missing {
                0 => changed.risk_scope_basis.clear(),
                1 => changed.effectiveness_basis.clear(),
                2 => changed.harm_basis.clear(),
                3 => changed.capability_basis.clear(),
                4 => changed.policy_basis.clear(),
                _ => unreachable!(),
            }
            assert_eq!(
                review(&s, &bound, frontier(), &[changed], &deps()),
                Err(Error::UnprovedControl)
            );
        }
        let mut changed = original;
        changed.independently_verified = false;
        assert_eq!(
            review(&s, &bound, frontier(), &[changed], &deps()),
            Err(Error::UnprovedControl)
        );
    }

    #[test]
    fn cross_candidate_comparison_bases_must_be_coherent() {
        let narrow = control("narrow", &[Action::Write], [1, 1, 1]);
        let broad = control("broad", &[Action::Write, Action::Delete], [2, 3, 2]);
        assert_eq!(
            choose_control(&[narrow.clone(), broad.clone()]),
            Ok("narrow".into())
        );
        let mut checked = 0;
        for field in 0..4 {
            let mut incompatible = broad.clone();
            match field {
                0 => incompatible.risk_scope_basis.push_str("/other-incident"),
                1 => incompatible.harm_basis.push_str("/other-harm-law"),
                2 => incompatible.capability_basis.push_str("/other-inventory"),
                3 => incompatible.policy_basis.push_str("/other-policy"),
                _ => unreachable!(),
            }
            for unavailable in [false, true] {
                let mut changed = incompatible.clone();
                changed.available = !unavailable;
                for reverse in [false, true] {
                    let candidates = if reverse {
                        vec![changed.clone(), narrow.clone()]
                    } else {
                        vec![narrow.clone(), changed.clone()]
                    };
                    assert_eq!(
                        choose_control(&candidates),
                        Err(Error::IncomparableControl),
                        "field={field} unavailable={unavailable} reverse={reverse}"
                    );
                    checked += 1;
                }
            }
            let state = state();
            let mut decision = basis(&state);
            decision.control_catalog = vec![narrow.clone(), incompatible.clone()];
            assert_eq!(
                review(
                    &state,
                    &decision,
                    frontier(),
                    &decision.control_catalog,
                    &deps()
                ),
                Err(Error::IncomparableControl),
                "review must not accept incomparable field {field}"
            );
        }
        assert_eq!(checked, 16);

        // Candidate-specific effectiveness evidence may legitimately differ;
        // unlike incident/harm/policy basis, it is not an implicit common unit.
        let mut independently_attested = broad.clone();
        independently_attested.effectiveness_basis = "different-control-proof".into();
        assert_eq!(
            choose_control(&[narrow.clone(), independently_attested]),
            Ok("narrow".into())
        );
        let mut different_comparison_policy = broad;
        different_comparison_policy.comparison_policy = 2;
        assert_eq!(
            choose_control(&[narrow, different_comparison_policy]),
            Err(Error::UnprovedControl)
        );
    }

    #[test]
    fn dependency_tuple_requires_all_three_exact_bases() {
        let s = state();
        let controls = [control("narrow", &[Action::Write], [1, 1, 1])];
        let complete = deps();
        let mut bound = basis(&s);
        assert_eq!(
            review(&s, &bound, frontier(), &controls, &complete),
            Ok(Review::SyntheticOnly)
        );
        for missing in ["#10", "#13", "#16"] {
            let mut partial = complete.clone();
            partial.remove(missing);
            bound.dependency_bases = partial.clone();
            assert_eq!(
                review(&s, &bound, frontier(), &controls, &partial),
                Err(Error::UnknownDependency),
                "same incomplete map lacks {missing}"
            );
            bound.dependency_bases = complete.clone();
            assert_eq!(
                review(&s, &bound, frontier(), &controls, &partial),
                Err(Error::UnknownDependency),
                "live map lacks {missing}"
            );
            bound.dependency_bases = partial;
            assert_eq!(
                review(&s, &bound, frontier(), &controls, &complete),
                Err(Error::UnknownDependency),
                "bound map lacks {missing}"
            );
        }
        let unrelated = BTreeMap::from([("other".into(), "unrelated-token".into())]);
        bound.dependency_bases = unrelated.clone();
        assert_eq!(
            review(&s, &bound, frontier(), &controls, &unrelated),
            Err(Error::UnknownDependency)
        );
        let mut with_extra = complete.clone();
        with_extra.insert("#extra".into(), "independent-v1".into());
        bound.dependency_bases = with_extra.clone();
        assert_eq!(
            review(&s, &bound, frontier(), &controls, &with_extra),
            Ok(Review::SyntheticOnly)
        );
        let mut changed = with_extra.clone();
        changed.insert("#13".into(), "changed-evidence-basis".into());
        assert_eq!(
            review(&s, &bound, frontier(), &controls, &changed),
            Err(Error::StaleDecision)
        );
        let mut malformed = with_extra;
        malformed.insert("#optional".into(), String::new());
        bound.dependency_bases = malformed.clone();
        assert_eq!(
            review(&s, &bound, frontier(), &controls, &malformed),
            Err(Error::UnknownDependency)
        );
    }

}
