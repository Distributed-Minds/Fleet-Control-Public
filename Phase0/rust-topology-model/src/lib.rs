//! Synthetic, side-effect-free model of Phase0 issue #7 (specification 5).
//!
//! This crate is NOT a principal registry, trusted topology source, agent-state
//! publisher, permission checker or source of state-append authority. Callers
//! may construct all input evidence; a Candidate MUST NEVER be used as a
//! provider-side admission receipt. Production needs separately trusted
//! currentness, predecessor, registration and effect-authority adapters.

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

#[derive(Clone, Debug)]
pub struct Topology {
    pub id: String,
    pub generation: u32,
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
    SyntheticIncompatible { from: String, to: String },
    SyntheticCompatibleBuilder { from: String, to: String },
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
        || top.principals.is_empty()
        || top.principals.len() != top.declared_size
        || top.principals.iter().any(|p| p.is_empty())
    {
        return Err(Rejection::TopologyUnknown);
    }
    for (idx, p) in top.principals.iter().enumerate() {
        if top.principals[..idx].contains(p) {
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
    match p.state.phase {
        Phase::Solo => p.size == 1 && p.index == 1,
        Phase::Analytic => {
            if p.size >= 3 && p.index == 1 {
                p.state.mode == Mode::Plan
            } else if p.size >= 3 && p.index == 2 {
                if p.generation >= 2 {
                    p.state.mode == Mode::Predict
                } else {
                    matches!(p.state.mode, Mode::Plan | Mode::Predict | Mode::Audit)
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

fn advance(mut state: State, size: usize, index: usize, result: ResultClass) -> Result<State, Rejection> {
    if matches!(result, ResultClass::Observe) {
        return Ok(state);
    }
    if size == 1 {
        return Err(Rejection::MissionDependentSolo);
    }
    if size >= 3 && index <= 2 {
        if matches!(result, ResultClass::BuildProgress | ResultClass::BuildNoProgress) {
            return Err(Rejection::IncompatibleResult);
        }
        state.mode = if index == 1 { Mode::Plan } else { Mode::Predict };
        state.noop_streak = if matches!(result, ResultClass::Noop) {
            state.noop_streak.saturating_add(1)
        } else {
            0
        };
        return Ok(state);
    }
    if size == 2 && index == 1 {
        if matches!(result, ResultClass::BuildProgress | ResultClass::BuildNoProgress) {
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
    if matches!(result, ResultClass::BuildProgress | ResultClass::BuildNoProgress) {
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
                    (bootstrap(input.topology.declared_size, index), BasisDisposition::IncompatibleReset)
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
