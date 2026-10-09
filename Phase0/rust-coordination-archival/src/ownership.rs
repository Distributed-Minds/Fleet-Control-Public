//! Advisory reduction of certified Phase0 coordination transitions.
//!
//! Input order, comment IDs, trusted authorship and the complete archive/live
//! frontier MUST already be proven by a trusted adapter. This module neither
//! authenticates provider data nor acquires, recovers or grants ownership.
//! Its output is a consistency check, NEVER an authorization to write.

use std::collections::{BTreeMap, HashMap, HashSet};

// Coordination records are also reviewed by humans. Unicode format controls
// can make distinct authority identities appear identical in logs or diffs.
// Reject these at the advisory reducer boundary; do not normalize/merge them.
fn ambiguous_identity_scalar(ch: char) -> bool {
    ch.is_control()
        || matches!(
            ch,
            '\u{00AD}'
                | '\u{034F}'
                | '\u{061C}'
                | '\u{180E}'
                | '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2060}'..='\u{206F}'
                | '\u{FE00}'..='\u{FE0F}'
                | '\u{FEFF}'
                | '\u{E0100}'..='\u{E01EF}'
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Intent,
    Owned,
    Working,
    Handoff,
    Release,
    Yield,
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub issue: Option<u64>,
    pub pr: Option<u64>,
    pub branch: Option<String>,
    pub seam: String,
}

impl Scope {
    fn valid(&self) -> bool {
        !self.seam.trim().is_empty()
            && self.seam == self.seam.trim()
            && !self.seam.chars().any(ambiguous_identity_scalar)
            && self.issue.is_none_or(|n| n != 0)
            && self.pr.is_none_or(|n| n != 0)
            && self.branch.as_ref().is_none_or(|b| {
                !b.trim().is_empty() && b.trim() == b && !b.chars().any(ambiguous_identity_scalar)
            })
    }

    /// Same issue alone does not collide when concrete seams differ.
    /// Sharing a branch, PR or explicit seam always does.
    fn overlaps(&self, other: &Self) -> bool {
        (self.pr.is_some() && self.pr == other.pr)
            || (self.branch.is_some() && self.branch == other.branch)
            || self.seam == other.seam
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    /// Certified reducer order, NOT inferred from a timestamp or comment ID.
    pub position: u64,
    /// Provider-supplied GitHub comment ID, used by the installed INTENT election.
    pub comment_id: u64,
    pub run: String,
    pub seq: u64,
    pub state: State,
    pub prev: Option<u64>,
    pub scope: Scope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLease {
    pub run: String,
    pub latest_comment_id: u64,
    pub state: State,
    pub scope: Scope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionFailure {
    InvalidRecord,
    UnorderedOrIncomplete,
    DuplicateComment,
    InvalidChain,
    InvalidTransition,
    EarlierCompetingIntent,
    OverlappingOwner,
}

#[derive(Clone)]
struct Last {
    id: u64,
    seq: u64,
    state: State,
    scope: Scope,
}

/// Replay one already-certified, complete, chronological coordination history.
///
/// A historical orphan INTENT is not automatically expired. If qualification
/// cannot be proven, a later competing OWNED record fails closed rather than
/// retroactively electing its writer. RECOVERED never evicts an incumbent:
/// trusted external recovery evidence must settle the previous owner first.
///
/// The model is deliberately stricter than a permissive best-effort log parser.
/// A failure means the caller must investigate the history, not skip records.
pub fn reduce_model_only(transitions: &[Transition]) -> Result<Vec<ActiveLease>, ReductionFailure> {
    use ReductionFailure::*;
    use State::*;

    let mut seen_ids = HashSet::new();
    let mut previous_position = None;
    let mut last_by_run: HashMap<&str, Last> = HashMap::new();
    let mut active: BTreeMap<&str, ActiveLease> = BTreeMap::new();

    for event in transitions {
        if event.position == 0
            || event.comment_id == 0
            || event.seq == 0
            || event.run.is_empty()
            || event.run.trim() != event.run
            || event.run.chars().any(ambiguous_identity_scalar)
            || !event.scope.valid()
        {
            return Err(InvalidRecord);
        }
        if previous_position.is_some_and(|last| event.position <= last) {
            return Err(UnorderedOrIncomplete);
        }
        previous_position = Some(event.position);
        if !seen_ids.insert(event.comment_id) {
            return Err(DuplicateComment);
        }

        let previous = last_by_run.get(event.run.as_str());
        match previous {
            None => {
                if event.seq != 1
                    || event.prev.is_some()
                    || !matches!(event.state, Intent | Recovered)
                {
                    return Err(InvalidChain);
                }
            }
            Some(old) if matches!(old.state, Handoff | Release | Yield) => {
                // A run may release package A and start unrelated package B.
                if event.seq != 1 || event.prev.is_some() || event.state != Intent {
                    return Err(InvalidChain);
                }
            }
            Some(old) if old.state == Recovered => {
                // The installed recovery record and subsequent INTENT can
                // both use seq=1. Recovery is evidence, not a lease transfer.
                if event.seq != 1 || event.prev.is_some() || event.state != Intent {
                    return Err(InvalidChain);
                }
            }
            Some(old) => {
                if old.seq.checked_add(1) != Some(event.seq)
                    || event.prev != Some(old.id)
                    || event.scope != old.scope
                {
                    return Err(InvalidChain);
                }
            }
        }

        match event.state {
            Intent => {
                if previous.is_some_and(|old| matches!(old.state, Intent | Owned | Working)) {
                    return Err(InvalidTransition);
                }
            }
            Recovered => {
                // Recovery is an initial evidence record for a fresh run,
                // never a mid-chain transition that can reset a live lease.
                if previous.is_some() {
                    return Err(InvalidTransition);
                }
            }
            Owned => {
                if !previous.is_some_and(|old| old.state == Intent) {
                    return Err(InvalidTransition);
                }
                // GitHub's configured election compares actual comment IDs
                // among qualified, conflicting intents, not reducer positions.
                if last_by_run.iter().any(|(run, old)| {
                    *run != event.run
                        && old.state == Intent
                        && old.id < previous.expect("checked").id
                        && old.scope.overlaps(&event.scope)
                }) {
                    return Err(EarlierCompetingIntent);
                }
                if active
                    .values()
                    .any(|lease| lease.scope.overlaps(&event.scope))
                {
                    return Err(OverlappingOwner);
                }
                active.insert(
                    event.run.as_str(),
                    ActiveLease {
                        run: event.run.clone(),
                        latest_comment_id: event.comment_id,
                        state: Owned,
                        scope: event.scope.clone(),
                    },
                );
            }
            Working => {
                if !previous.is_some_and(|old| matches!(old.state, Owned | Working)) {
                    return Err(InvalidTransition);
                }
                let lease = active
                    .get_mut(event.run.as_str())
                    .ok_or(InvalidTransition)?;
                lease.state = Working;
                lease.latest_comment_id = event.comment_id;
            }
            Handoff | Release => {
                if !previous.is_some_and(|old| matches!(old.state, Owned | Working))
                    || active.remove(event.run.as_str()).is_none()
                {
                    return Err(InvalidTransition);
                }
            }
            Yield => {
                if !previous.is_some_and(|old| matches!(old.state, Intent | Owned | Working)) {
                    return Err(InvalidTransition);
                }
                active.remove(event.run.as_str());
            }
        }

        last_by_run.insert(
            event.run.as_str(),
            Last {
                id: event.comment_id,
                seq: event.seq,
                state: event.state,
                scope: event.scope.clone(),
            },
        );
    }
    Ok(active.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(
        position: u64,
        id: u64,
        run: &str,
        seq: u64,
        state: State,
        prev: Option<u64>,
        branch: &str,
    ) -> Transition {
        Transition {
            position,
            comment_id: id,
            run: run.to_owned(),
            seq,
            state,
            prev,
            scope: Scope {
                issue: Some(22),
                pr: None,
                branch: Some(branch.to_owned()),
                seam: branch.to_owned(),
            },
        }
    }

    #[test]
    fn invisible_identity_scalars_never_create_apparently_valid_leases() {
        // These chars are not all Rust control characters. Keep expected
        // transitions unchanged while mutating only one identifier at a time.
        for hidden in [
            "\u{00AD}", "\u{034F}", "\u{061C}", "\u{200B}",
            "\u{202E}", "\u{2060}", "\u{FE0F}", "\u{FEFF}",
            "\u{E0100}",
        ] {
            let base = [
                event(1, 101, "actor", 1, State::Intent, None, "ref-a"),
                event(2, 102, "actor", 2, State::Owned, Some(101), "ref-a"),
            ];
            let mut fake_run = base.clone();
            for record in &mut fake_run {
                record.run = format!("actor{hidden}");
            }
            assert_eq!(
                reduce_model_only(&fake_run),
                Err(ReductionFailure::InvalidRecord),
                "hidden run identity {hidden:?} accepted"
            );
            let mut fake_branch = base.clone();
            for record in &mut fake_branch {
                record.scope.branch = Some(format!("ref-a{hidden}"));
            }
            assert_eq!(
                reduce_model_only(&fake_branch),
                Err(ReductionFailure::InvalidRecord),
                "hidden branch identity {hidden:?} accepted"
            );
            let mut fake_seam = base;
            for record in &mut fake_seam {
                record.scope.seam = format!("ref-a{hidden}");
            }
            assert_eq!(
                reduce_model_only(&fake_seam),
                Err(ReductionFailure::InvalidRecord),
                "hidden seam identity {hidden:?} accepted"
            );
        }

        // Non-ASCII printable letters remain valid: this is not an ASCII-only
        // ban, normalization, a GitHub identity proof, or Unicode confusable audit.
        let mut legitimate = [
            event(1, 101, "actor", 1, State::Intent, None, "ref-a"),
            event(2, 102, "actor", 2, State::Owned, Some(101), "ref-a"),
        ];
        for record in &mut legitimate {
            record.run = "réparer-β".to_owned();
            record.scope.branch = Some("travail-β".to_owned());
            record.scope.seam = "travail-β".to_owned();
        }
        assert_eq!(reduce_model_only(&legitimate).unwrap().len(), 1);
    }

    #[test]
    fn legitimate_ownership_working_and_release_do_not_leave_a_lease() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 103, "a", 2, State::Owned, Some(101), "ref-a"),
            event(3, 105, "a", 3, State::Working, Some(103), "ref-a"),
            event(4, 107, "a", 4, State::Handoff, Some(105), "ref-a"),
        ];
        assert!(reduce_model_only(&records).unwrap().is_empty());
    }

    #[test]
    fn independent_branches_can_be_owned_concurrently_under_one_issue() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 102, "b", 1, State::Intent, None, "ref-b"),
            event(3, 103, "b", 2, State::Owned, Some(102), "ref-b"),
            event(4, 104, "a", 2, State::Owned, Some(101), "ref-a"),
        ];
        let active = reduce_model_only(&records).unwrap();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].run, "a");
        assert_eq!(active[1].run, "b");
    }

    #[test]
    fn competing_intent_with_lower_comment_id_must_win_election() {
        let records = [
            event(1, 102, "later", 1, State::Intent, None, "shared"),
            event(2, 101, "earlier", 1, State::Intent, None, "shared"),
            event(3, 103, "later", 2, State::Owned, Some(102), "shared"),
        ];
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::EarlierCompetingIntent)
        );
    }

    #[test]
    fn incumbent_blocks_late_ownership_even_without_pending_intents() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "shared"),
            event(2, 102, "a", 2, State::Owned, Some(101), "shared"),
            event(3, 103, "b", 1, State::Intent, None, "shared"),
            event(4, 104, "b", 2, State::Owned, Some(103), "shared"),
        ];
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::OverlappingOwner)
        );
    }

    #[test]
    fn malformed_chains_and_scope_drift_are_rejected() {
        let mut records = vec![
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 102, "a", 2, State::Owned, Some(101), "ref-a"),
        ];
        records[1].prev = Some(999);
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::InvalidChain)
        );
        records[1].prev = Some(101);
        records[1].scope.branch = Some("other-branch".to_owned());
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::InvalidChain)
        );
        records[1].scope.branch = Some("ref-a".to_owned());
        records[1].seq = 4;
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::InvalidChain)
        );
    }

    #[test]
    fn certified_positions_are_not_comment_id_order() {
        let records = [
            event(1, 500, "a", 1, State::Intent, None, "ref-a"),
            event(2, 400, "a", 2, State::Owned, Some(500), "ref-a"),
        ];
        assert_eq!(reduce_model_only(&records).unwrap().len(), 1);
    }

    #[test]
    fn duplicate_ids_and_reordered_positions_fail_closed() {
        let mut records = vec![
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 101, "b", 1, State::Intent, None, "ref-b"),
        ];
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::DuplicateComment)
        );
        records[1].comment_id = 102;
        records[1].position = 1;
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::UnorderedOrIncomplete)
        );
    }

    #[test]
    fn terminal_package_allows_next_intent_with_same_run() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 102, "a", 2, State::Owned, Some(101), "ref-a"),
            event(3, 103, "a", 3, State::Release, Some(102), "ref-a"),
            event(4, 104, "a", 1, State::Intent, None, "ref-b"),
            event(5, 105, "a", 2, State::Owned, Some(104), "ref-b"),
        ];
        let leases = reduce_model_only(&records).unwrap();
        assert_eq!(leases.len(), 1);
        assert_eq!(leases[0].scope.branch.as_deref(), Some("ref-b"));
    }

    #[test]
    fn recovery_cannot_steal_the_existing_lease() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "shared"),
            event(2, 102, "a", 2, State::Owned, Some(101), "shared"),
            event(3, 103, "b", 1, State::Recovered, None, "shared"),
            event(4, 104, "b", 1, State::Intent, None, "shared"),
            event(5, 105, "b", 2, State::Owned, Some(104), "shared"),
        ];
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::OverlappingOwner)
        );
    }

    #[test]
    fn yield_releases_scope_only_if_already_owned() {
        let records = [
            event(1, 101, "a", 1, State::Intent, None, "shared"),
            event(2, 102, "a", 2, State::Owned, Some(101), "shared"),
            event(3, 103, "a", 3, State::Yield, Some(102), "shared"),
            event(4, 104, "b", 1, State::Intent, None, "shared"),
            event(5, 105, "b", 2, State::Owned, Some(104), "shared"),
        ];
        assert_eq!(reduce_model_only(&records).unwrap()[0].run, "b");
    }

    #[test]
    fn recovered_cannot_reset_a_nonterminal_run_chain() {
        let prefixes = [
            vec![event(1, 101, "a", 1, State::Intent, None, "ref-a")],
            vec![
                event(1, 101, "a", 1, State::Intent, None, "ref-a"),
                event(2, 102, "a", 2, State::Owned, Some(101), "ref-a"),
            ],
            vec![
                event(1, 101, "a", 1, State::Intent, None, "ref-a"),
                event(2, 102, "a", 2, State::Owned, Some(101), "ref-a"),
                event(3, 103, "a", 3, State::Working, Some(102), "ref-a"),
            ],
        ];
        for mut records in prefixes {
            let last = records.last().expect("nonempty prefix");
            let invalid = event(
                last.position + 1,
                last.comment_id + 1,
                "a",
                last.seq + 1,
                State::Recovered,
                Some(last.comment_id),
                "ref-a",
            );
            records.push(invalid);
            assert_eq!(
                reduce_model_only(&records),
                Err(ReductionFailure::InvalidTransition),
                "recovery must not reset an active or pending run"
            );
        }

        // The old reducer accepted this chain and silently replaced the
        // still-owned ref-a scope with ref-b under the same run identity.
        let illicit_second_package = [
            event(1, 101, "a", 1, State::Intent, None, "ref-a"),
            event(2, 102, "a", 2, State::Owned, Some(101), "ref-a"),
            event(3, 103, "a", 3, State::Recovered, Some(102), "ref-a"),
            event(4, 104, "a", 1, State::Intent, None, "ref-b"),
            event(5, 105, "a", 2, State::Owned, Some(104), "ref-b"),
        ];
        assert_eq!(
            reduce_model_only(&illicit_second_package),
            Err(ReductionFailure::InvalidTransition)
        );
    }

    #[test]
    fn fresh_run_recovery_still_allows_intent_and_ownership() {
        let records = [
            event(1, 101, "new-run", 1, State::Recovered, None, "ref-a"),
            event(2, 102, "new-run", 1, State::Intent, None, "ref-a"),
            event(3, 103, "new-run", 2, State::Owned, Some(102), "ref-a"),
        ];
        let leases = reduce_model_only(&records).expect("valid recovery prefix");
        assert_eq!(leases.len(), 1);
        assert_eq!(leases[0].run, "new-run");
        assert_eq!(leases[0].scope.branch.as_deref(), Some("ref-a"));
    }

    #[test]
    fn incomplete_run_prefix_is_not_inferred_from_the_tail() {
        let records = [event(2, 102, "a", 2, State::Owned, Some(101), "shared")];
        assert_eq!(
            reduce_model_only(&records),
            Err(ReductionFailure::InvalidChain)
        );
    }
}
