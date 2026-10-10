//! Synthetic, side-effect-free model of Phase0 issue #7 (specification 5).
//!
//! This crate is NOT a principal registry, trusted topology source, agent-state
//! publisher, permission checker or source of state-append authority. Callers
//! may construct all input evidence; a Candidate MUST NEVER be used as a
//! provider-side admission receipt. Production needs separately trusted
//! currentness, predecessor, registration and effect-authority adapters.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plan,
    Predict,
    Audit,
    Build,
    Integrate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Solo,
    Analytic,
    Bootstrap,
    Free,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildResult {
    None,
    Progress,
    NoProgress,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub mode: Mode,
    pub phase: Phase,
    pub noop_streak: u32,
    pub last_build: BuildResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// A synthetic label for fixture data, NOT proof of real-world admission.
    SyntheticCoherent,
    Missing,
    Torn,
    Forked,
    Unauthorized,
    Revoked,
}

/// Historical transition semantics never follow topology generation.
/// This synthetic label is not trusted evidence of a real state-machine version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateMachine {
    RotatingAnalystV1,
    PermanentPredictorV2,
}

#[derive(Clone, Debug)]
pub struct Topology {
    pub id: String,
    pub generation: u32,
    pub state_machine: StateMachine,
    pub principals: Vec<String>,
    pub declared_size: usize,
    pub evidence: Evidence,
}

#[derive(Clone, Debug)]
pub struct Prior {
    pub principal: String,
    pub source_basis: String,
    pub generation: u32,
    pub size: usize,
    pub index: usize,
    pub source_machine: StateMachine,
    pub state: State,
    /// The fixture's synthetic immediate-predecessor/counter evidence.
    pub counter: Option<CounterProof>,
}

#[derive(Clone, Debug)]
pub struct CounterProof {
    pub basis: String,
    pub generation: u32,
    pub principal: String,
    pub predecessor_known: bool,
    pub transition_rule_version: u32,
}

#[derive(Clone, Debug)]
pub enum Lineage {
    /// The only simulated first-enrollment case: no prior record exists.
    FirstEnrollment,
    SameBasis,
    /// An independently-authorized real transition is NOT established by this
    /// user-constructible fixture label; this only explores its consequences.
    SyntheticIncompatible {
        from: String,
        to: String,
    },
    SyntheticCompatibleBuilder {
        from: String,
        to: String,
    },
    Unproven,
}

#[derive(Clone, Copy, Debug)]
pub enum ResultClass {
    Observe,
    Material,
    Noop,
    BuildProgress,
    BuildNoProgress,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub topology: Topology,
    pub principal: String,
    pub prior: Option<Prior>,
    pub lineage: Lineage,
    pub result: ResultClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    TopologyUnknown,
    InvalidHistory,
    UnprovenLineage,
    CounterUnknown,
    IncompatibleResult,
    MissionDependentSolo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasisDisposition {
    FirstEnrollment,
    Same,
    IncompatibleReset,
    CompatibleBuilderCarry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub state: State,
    pub basis: BasisDisposition,
    /// ALWAYS false: a model prediction cannot mint provider mutation authority.
    pub provider_append_authorized: bool,
}

fn member_index(top: &Topology, principal: &str) -> Result<usize, Rejection> {
    if top.evidence != Evidence::SyntheticCoherent
        || top.id.is_empty()
        || top.generation == 0
        || top.state_machine != StateMachine::PermanentPredictorV2
        || top.principals.is_empty()
        || top.principals.len() != top.declared_size
        || top.principals.iter().any(|p| p.is_empty())
    {
        return Err(Rejection::TopologyUnknown);
    }
    // Preserve principal ordering while checking each identity only once.
    let mut seen = BTreeSet::new();
    for p in &top.principals {
        if !seen.insert(p.as_str()) {
            return Err(Rejection::TopologyUnknown);
        }
    }
    top.principals
        .iter()
        .position(|p| p == principal)
        .map(|i| i + 1)
        .ok_or(Rejection::TopologyUnknown)
}

fn is_builder(size: usize, index: usize) -> bool {
    (size == 2 && index == 2) || (size >= 3 && index >= 3 && index <= size)
}

fn valid_historical_state(p: &Prior) -> bool {
    if p.size == 0 || p.index == 0 || p.index > p.size || p.generation == 0 {
        return false;
    }
    // These are unreachable under the transition function: analytic and
    // bootstrap roles never report a BUILD outcome, and every second NOOP in
    // an exhausting role moves the mode and resets the counter to zero.
    if matches!(p.state.phase, Phase::Analytic | Phase::Bootstrap)
        && p.state.last_build != BuildResult::None
    {
        return false;
    }
    // The V1 index-2 analyst rotated after every second NOOP, whereas the
    // V2 permanent predictor deliberately retains an unbounded observation
    // streak. Reject V1 history that could not have been produced by its
    // transition law without rejecting valid long-lived V2 predictor state.
    let rotating_v1_analyst = p.state.phase == Phase::Analytic
        && p.size >= 3
        && p.index == 2
        && p.source_machine == StateMachine::RotatingAnalystV1;
    if (p.state.phase == Phase::Bootstrap || (p.size == 2 && p.index == 1) || rotating_v1_analyst)
        && p.state.noop_streak > 1
    {
        return false;
    }
    // The FREE reactive loop always clears the NOOP counter.
    if p.state.phase == Phase::Free && p.state.noop_streak != 0 {
        return false;
    }
    // INTEGRATE can only follow BUILD progress, and AUDIT can only follow
    // BUILD no-progress. Neither may appear with a missing/mismatched outcome.
    // BUILD itself can retain either outcome after a reactive return, or None
    // immediately after the bootstrap-to-FREE transition.
    if p.state.phase == Phase::Free {
        if p.state.mode == Mode::Integrate && p.state.last_build != BuildResult::Progress {
            return false;
        }
        if p.state.mode == Mode::Audit && p.state.last_build != BuildResult::NoProgress {
            return false;
        }
    }
    match p.state.phase {
        // The single-agent model only bootstraps PLAN and permits an
        // observational no-op. Non-observational results require a separate
        // mission-specific decision (not modelled by this transition graph).
        // A historical SOLO build/audit mode or accumulated NOOP is therefore
        // unreachable, not a compatible state to carry into a new topology.
        Phase::Solo => {
            p.size == 1
                && p.index == 1
                && p.state.mode == Mode::Plan
                && p.state.noop_streak == 0
                && p.state.last_build == BuildResult::None
        }
        Phase::Analytic => {
            if p.size >= 3 && p.index == 1 {
                p.state.mode == Mode::Plan
            } else if p.size >= 3 && p.index == 2 {
                match p.source_machine {
                    StateMachine::PermanentPredictorV2 => p.state.mode == Mode::Predict,
                    StateMachine::RotatingAnalystV1 => {
                        matches!(p.state.mode, Mode::Plan | Mode::Predict | Mode::Audit)
                    }
                }
            } else {
                p.size == 2
                    && p.index == 1
                    && matches!(p.state.mode, Mode::Plan | Mode::Predict | Mode::Audit)
            }
        }
        Phase::Bootstrap => {
            is_builder(p.size, p.index)
                && matches!(p.state.mode, Mode::Plan | Mode::Predict | Mode::Audit)
        }
        Phase::Free => {
            is_builder(p.size, p.index)
                && matches!(p.state.mode, Mode::Build | Mode::Audit | Mode::Integrate)
        }
    }
}

fn counter_proven(p: &Prior) -> bool {
    matches!(&p.counter, Some(c) if c.basis == p.source_basis
        && c.generation == p.generation
        && c.principal == p.principal
        && c.predecessor_known
        && c.transition_rule_version == 1)
}

fn bootstrap(size: usize, index: usize) -> State {
    let (mode, phase) = if size == 1 {
        (Mode::Plan, Phase::Solo)
    } else if size >= 3 && index == 1 {
        (Mode::Plan, Phase::Analytic)
    } else if size >= 3 && index == 2 {
        (Mode::Predict, Phase::Analytic)
    } else if size == 2 && index == 1 {
        (Mode::Plan, Phase::Analytic)
    } else {
        (Mode::Plan, Phase::Bootstrap)
    };
    State {
        mode,
        phase,
        noop_streak: 0,
        last_build: BuildResult::None,
    }
}

fn advance(
    mut state: State,
    size: usize,
    index: usize,
    result: ResultClass,
) -> Result<State, Rejection> {
    if matches!(result, ResultClass::Observe) {
        return Ok(state);
    }
    if size == 1 {
        return Err(Rejection::MissionDependentSolo);
    }
    if size >= 3 && index <= 2 {
        if matches!(
            result,
            ResultClass::BuildProgress | ResultClass::BuildNoProgress
        ) {
            return Err(Rejection::IncompatibleResult);
        }
        state.mode = if index == 1 {
            Mode::Plan
        } else {
            Mode::Predict
        };
        state.noop_streak = if matches!(result, ResultClass::Noop) {
            state.noop_streak.saturating_add(1)
        } else {
            0
        };
        return Ok(state);
    }
    if size == 2 && index == 1 {
        if matches!(
            result,
            ResultClass::BuildProgress | ResultClass::BuildNoProgress
        ) {
            return Err(Rejection::IncompatibleResult);
        }
        state.noop_streak = if matches!(result, ResultClass::Noop) {
            state.noop_streak.saturating_add(1)
        } else {
            0
        };
        if state.noop_streak >= 2 {
            state.mode = match state.mode {
                Mode::Plan => Mode::Predict,
                Mode::Predict => Mode::Audit,
                Mode::Audit => Mode::Plan,
                _ => return Err(Rejection::InvalidHistory),
            };
            state.noop_streak = 0;
        }
        return Ok(state);
    }
    if state.phase == Phase::Free {
        state.noop_streak = 0;
        match (state.mode, result) {
            (Mode::Build, ResultClass::BuildProgress) => {
                state.mode = Mode::Integrate;
                state.last_build = BuildResult::Progress;
            }
            (Mode::Build, ResultClass::BuildNoProgress) => {
                state.mode = Mode::Audit;
                state.last_build = BuildResult::NoProgress;
            }
            (Mode::Audit | Mode::Integrate, ResultClass::Material | ResultClass::Noop) => {
                state.mode = Mode::Build;
            }
            _ => return Err(Rejection::IncompatibleResult),
        }
        return Ok(state);
    }
    if matches!(
        result,
        ResultClass::BuildProgress | ResultClass::BuildNoProgress
    ) {
        return Err(Rejection::IncompatibleResult);
    }
    if matches!(result, ResultClass::Material) {
        state.noop_streak = 0;
        return Ok(state);
    }
    state.noop_streak = state.noop_streak.saturating_add(1);
    if state.noop_streak >= 2 {
        state.mode = match state.mode {
            Mode::Plan => Mode::Predict,
            Mode::Predict => Mode::Audit,
            Mode::Audit => {
                state.phase = Phase::Free;
                Mode::Build
            }
            _ => return Err(Rejection::InvalidHistory),
        };
        state.noop_streak = 0;
    }
    Ok(state)
}

/// Predict a synthetic next state without creating, authorizing or applying an
/// AGENT_STATE record. In particular, labels inside Input cannot prove actual
/// TopologyBasis currentness, registration, ownership or provider authority.
pub fn predict(input: &Input) -> Result<Candidate, Rejection> {
    let index = member_index(&input.topology, &input.principal)?;
    let (state, basis) = match (&input.prior, &input.lineage) {
        (None, Lineage::FirstEnrollment) => (
            bootstrap(input.topology.declared_size, index),
            BasisDisposition::FirstEnrollment,
        ),
        (Some(p), lineage) => {
            if p.principal != input.principal
                || p.source_basis.is_empty()
                || !valid_historical_state(p)
            {
                return Err(Rejection::InvalidHistory);
            }
            match lineage {
                Lineage::SameBasis => {
                    if p.source_basis != input.topology.id
                        || p.source_machine != input.topology.state_machine
                        || p.generation != input.topology.generation
                        || p.size != input.topology.declared_size
                        || p.index != index
                    {
                        return Err(Rejection::UnprovenLineage);
                    }
                    if !counter_proven(p) {
                        return Err(Rejection::CounterUnknown);
                    }
                    (p.state.clone(), BasisDisposition::Same)
                }
                Lineage::SyntheticIncompatible { from, to }
                    if from == &p.source_basis
                        && to == &input.topology.id
                        && (p.source_basis != input.topology.id
                            || p.generation != input.topology.generation) =>
                {
                    (
                        bootstrap(input.topology.declared_size, index),
                        BasisDisposition::IncompatibleReset,
                    )
                }
                Lineage::SyntheticCompatibleBuilder { from, to }
                    if from == &p.source_basis
                        && to == &input.topology.id
                        && is_builder(p.size, p.index)
                        && is_builder(input.topology.declared_size, index)
                        && p.state.phase == Phase::Free =>
                {
                    if !counter_proven(p) {
                        return Err(Rejection::CounterUnknown);
                    }
                    (p.state.clone(), BasisDisposition::CompatibleBuilderCarry)
                }
                _ => return Err(Rejection::UnprovenLineage),
            }
        }
        _ => return Err(Rejection::UnprovenLineage),
    };
    let next = advance(state, input.topology.declared_size, index, input.result)?;
    Ok(Candidate {
        state: next,
        basis,
        provider_append_authorized: false,
    })
}

#[cfg(test)]
mod historical_state_validation_tests {
    use super::*;

    #[test]
    fn large_unique_topology_preserves_indexes_and_rejects_late_duplicates() {
        let principals: Vec<String> = (0..2_048).map(|i| format!("agent-{i:04}")).collect();
        let top = Topology {
            id: "large-synthetic".into(),
            generation: 1,
            state_machine: StateMachine::PermanentPredictorV2,
            declared_size: principals.len(),
            principals: principals.clone(),
            evidence: Evidence::SyntheticCoherent,
        };
        assert_eq!(member_index(&top, &principals[0]), Ok(1));
        assert_eq!(member_index(&top, &principals[1_023]), Ok(1_024));
        assert_eq!(member_index(&top, &principals[2_047]), Ok(2_048));

        let mut duplicate = top.clone();
        duplicate.principals[2_047] = principals[1_023].clone();
        assert_eq!(
            member_index(&duplicate, &principals[0]),
            Err(Rejection::TopologyUnknown)
        );
    }

    fn prior(phase: Phase, mode: Mode, index: usize, noop_streak: u32) -> Prior {
        Prior {
            principal: "P".into(),
            source_basis: "basis".into(),
            generation: 1,
            size: 3,
            index,
            source_machine: StateMachine::PermanentPredictorV2,
            state: State {
                mode,
                phase,
                noop_streak,
                last_build: BuildResult::None,
            },
            counter: None,
        }
    }

    #[test]
    fn analytic_and_bootstrap_states_cannot_carry_build_outcomes() {
        for (phase, mode, index) in [
            (Phase::Analytic, Mode::Plan, 1),
            (Phase::Analytic, Mode::Predict, 2),
            (Phase::Bootstrap, Mode::Plan, 3),
        ] {
            let mut previous = prior(phase, mode, index, 0);
            assert!(valid_historical_state(&previous));
            for outcome in [BuildResult::Progress, BuildResult::NoProgress] {
                previous.state.last_build = outcome;
                assert!(!valid_historical_state(&previous));
            }
        }
    }

    #[test]
    fn free_review_modes_require_consistent_build_outcomes() {
        let mut integrate = prior(Phase::Free, Mode::Integrate, 3, 0);
        integrate.state.last_build = BuildResult::Progress;
        assert!(valid_historical_state(&integrate));
        for invalid in [BuildResult::None, BuildResult::NoProgress] {
            integrate.state.last_build = invalid;
            assert!(!valid_historical_state(&integrate));
        }

        let mut audit = prior(Phase::Free, Mode::Audit, 3, 0);
        audit.state.last_build = BuildResult::NoProgress;
        assert!(valid_historical_state(&audit));
        for invalid in [BuildResult::None, BuildResult::Progress] {
            audit.state.last_build = invalid;
            assert!(!valid_historical_state(&audit));
        }

        // A freshly bootstrapped BUILD has no result; a reactive return from
        // either review mode preserves its last BUILD outcome.
        let mut build = prior(Phase::Free, Mode::Build, 3, 0);
        for reachable in [
            BuildResult::None,
            BuildResult::Progress,
            BuildResult::NoProgress,
        ] {
            build.state.last_build = reachable;
            assert!(valid_historical_state(&build));
        }
    }

    #[test]
    fn exhausted_noop_streaks_and_free_streaks_fail_closed() {
        let mut bootstrap = prior(Phase::Bootstrap, Mode::Predict, 3, 1);
        assert!(valid_historical_state(&bootstrap));
        bootstrap.state.noop_streak = 2;
        assert!(!valid_historical_state(&bootstrap));

        let mut free = prior(Phase::Free, Mode::Build, 3, 0);
        assert!(valid_historical_state(&free));
        free.state.noop_streak = 1;
        assert!(!valid_historical_state(&free));

        let permanent = prior(Phase::Analytic, Mode::Predict, 2, 9);
        assert!(valid_historical_state(&permanent));

        let mut size_two_analyst = prior(Phase::Analytic, Mode::Plan, 1, 1);
        size_two_analyst.size = 2;
        assert!(valid_historical_state(&size_two_analyst));
        size_two_analyst.state.noop_streak = 2;
        assert!(!valid_historical_state(&size_two_analyst));
    }
    #[test]
    fn solo_history_rejects_unreachable_modes_streaks_and_build_outcomes() {
        let mut solo = prior(Phase::Solo, Mode::Plan, 1, 0);
        solo.size = 1;
        assert!(valid_historical_state(&solo));

        for unreachable in [Mode::Predict, Mode::Audit, Mode::Build, Mode::Integrate] {
            solo.state.mode = unreachable;
            assert!(!valid_historical_state(&solo));
        }
        solo.state.mode = Mode::Plan;

        for outcome in [BuildResult::Progress, BuildResult::NoProgress] {
            solo.state.last_build = outcome;
            assert!(!valid_historical_state(&solo));
        }
        solo.state.last_build = BuildResult::None;

        solo.state.noop_streak = 1;
        assert!(!valid_historical_state(&solo));
        solo.state.noop_streak = 0;
        assert!(valid_historical_state(&solo));

        // Reaching a builder slot later must not carry an invented SOLO
        // history through an otherwise plausible incompatible reset.
        solo.state.mode = Mode::Build;
        let input = Input {
            topology: Topology {
                id: "next".into(),
                generation: 2,
                state_machine: StateMachine::PermanentPredictorV2,
                principals: vec!["A".into(), "B".into(), "P".into()],
                declared_size: 3,
                evidence: Evidence::SyntheticCoherent,
            },
            principal: "P".into(),
            prior: Some(solo),
            lineage: Lineage::SyntheticIncompatible {
                from: "basis".into(),
                to: "next".into(),
            },
            result: ResultClass::Observe,
        };
        assert_eq!(predict(&input), Err(Rejection::InvalidHistory));
    }
}
