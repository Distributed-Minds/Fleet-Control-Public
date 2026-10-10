use free_energy_topology_model::{
    predict, BasisDisposition, BuildResult, Candidate, CounterProof, Evidence, Input, Lineage,
    Mode, Phase, Prior, Rejection, ResultClass, State, Topology,
};

fn topology(principals: &[&str], id: &str, generation: u32) -> Topology {
    Topology {
        id: id.into(),
        generation,
        principals: principals.iter().map(|v| (*v).into()).collect(),
        declared_size: principals.len(),
        evidence: Evidence::SyntheticCoherent,
    }
}

fn state(mode: Mode, phase: Phase, noop_streak: u32) -> State {
    State {
        mode,
        phase,
        noop_streak,
        last_build: BuildResult::None,
    }
}

fn previous(
    who: &str,
    basis: &str,
    generation: u32,
    size: usize,
    index: usize,
    state: State,
) -> Prior {
    Prior {
        principal: who.into(),
        source_basis: basis.into(),
        generation,
        size,
        index,
        state,
        counter: Some(CounterProof {
            basis: basis.into(),
            generation,
            principal: who.into(),
            predecessor_known: true,
            transition_rule_version: 1,
        }),
    }
}

fn check(
    topology: &Topology,
    who: &str,
    prior: Option<Prior>,
    lineage: Lineage,
    result: ResultClass,
) -> Result<Candidate, Rejection> {
    predict(&Input {
        topology: topology.clone(),
        principal: who.into(),
        prior,
        lineage,
        result,
    })
}

#[test]
fn fixed_planner_and_predictor_survive_long_arbitrary_material_noop_sequences() {
    for size in [3, 4, 5, 8] {
        let names: Vec<String> = (0..size).map(|i| format!("P{i}")).collect();
        let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
        let top = topology(&borrowed, "corrected", 2);
        for (index, expected) in [(1, Mode::Plan), (2, Mode::Predict)] {
            let who = &names[index - 1];
            let mut candidate = check(&top, who, None, Lineage::FirstEnrollment, ResultClass::Observe)
                .expect("first-ever synthetic bootstrap");
            for iteration in 0..256 {
                assert_eq!(candidate.state.mode, expected, "size {size} iteration {iteration}");
                assert!(!candidate.provider_append_authorized);
                let result = if iteration % 7 < 4 {
                    ResultClass::Noop
                } else {
                    ResultClass::Material
                };
                candidate = check(
                    &top,
                    who,
                    Some(previous(who, "corrected", 2, size, index, candidate.state)),
                    Lineage::SameBasis,
                    result,
                )
                .expect("same-basis predecessor chain");
            }
        }
    }
}

#[test]
fn builder_bootstrap_reaches_free_then_reactive_build_cycle() {
    let top = topology(&["A", "B", "C"], "current", 2);
    let mut c = check(&top, "C", None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap();
    assert_eq!((c.state.mode, c.state.phase), (Mode::Plan, Phase::Bootstrap));
    for (step, wanted) in [
        (1, Mode::Plan),
        (2, Mode::Predict),
        (3, Mode::Predict),
        (4, Mode::Audit),
        (5, Mode::Audit),
        (6, Mode::Build),
    ] {
        c = check(
            &top,
            "C",
            Some(previous("C", "current", 2, 3, 3, c.state)),
            Lineage::SameBasis,
            ResultClass::Noop,
        )
        .unwrap();
        assert_eq!(c.state.mode, wanted, "bootstrap step {step}");
    }
    assert_eq!(c.state.phase, Phase::Free);
    c = check(
        &top,
        "C",
        Some(previous("C", "current", 2, 3, 3, c.state)),
        Lineage::SameBasis,
        ResultClass::BuildProgress,
    )
    .unwrap();
    assert_eq!(c.state.mode, Mode::Integrate);
    c = check(
        &top,
        "C",
        Some(previous("C", "current", 2, 3, 3, c.state)),
        Lineage::SameBasis,
        ResultClass::Material,
    )
    .unwrap();
    assert_eq!(c.state.mode, Mode::Build);
    c = check(
        &top,
        "C",
        Some(previous("C", "current", 2, 3, 3, c.state)),
        Lineage::SameBasis,
        ResultClass::BuildNoProgress,
    )
    .unwrap();
    assert_eq!((c.state.mode, c.state.last_build), (Mode::Audit, BuildResult::NoProgress));
}

#[test]
fn prior_permanent_planner_reordered_to_builder_resets_bootstrap() {
    let new = topology(&["A3", "A2", "A1"], "v2", 2);
    let old = previous("A1", "v1", 1, 3, 1, state(Mode::Plan, Phase::Analytic, 7));
    let result = check(&new, "A1", Some(old), Lineage::SyntheticIncompatible {
        from: "v1".into(),
        to: "v2".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(result.basis, BasisDisposition::IncompatibleReset);
    assert_eq!(result.state, state(Mode::Plan, Phase::Bootstrap, 0));
}

#[test]
fn prior_permanent_adversary_reordered_to_builder_resets_bootstrap() {
    let new = topology(&["A1", "A3", "A2"], "v2", 2);
    let old = previous("A2", "v1", 1, 3, 2, state(Mode::Audit, Phase::Analytic, 1));
    let result = check(&new, "A2", Some(old), Lineage::SyntheticIncompatible {
        from: "v1".into(),
        to: "v2".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(result.state, state(Mode::Plan, Phase::Bootstrap, 0));
}

#[test]
fn old_free_builder_can_move_to_permanent_predictor_only_after_reset() {
    let old = previous("A3", "v1", 1, 3, 3, state(Mode::Integrate, Phase::Free, 0));
    let new = topology(&["A1", "A3", "A2"], "v2", 2);
    let result = check(&new, "A3", Some(old), Lineage::SyntheticIncompatible {
        from: "v1".into(),
        to: "v2".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(result.state, state(Mode::Predict, Phase::Analytic, 0));
}

#[test]
fn shrinking_to_one_or_two_normalizes_current_role() {
    let source = previous("C", "old", 1, 3, 3, state(Mode::Build, Phase::Free, 0));
    let one = topology(&["C"], "solo", 2);
    let c = check(&one, "C", Some(source.clone()), Lineage::SyntheticIncompatible {
        from: "old".into(),
        to: "solo".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(c.state, state(Mode::Plan, Phase::Solo, 0));
    let two = topology(&["A", "C"], "duo", 2);
    let d = check(&two, "C", Some(source), Lineage::SyntheticIncompatible {
        from: "old".into(),
        to: "duo".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(d.state, state(Mode::Plan, Phase::Bootstrap, 0));
}

#[test]
fn old_rotating_predictor_streak_does_not_count_as_new_second_noop() {
    let top = topology(&["A", "B", "C"], "new", 2);
    let old = previous("B", "old", 1, 3, 2, state(Mode::Predict, Phase::Analytic, 1));
    let c = check(&top, "B", Some(old), Lineage::SyntheticIncompatible {
        from: "old".into(),
        to: "new".into(),
    }, ResultClass::Noop).unwrap();
    assert_eq!((c.state.mode, c.state.noop_streak), (Mode::Predict, 1));
}

#[test]
fn same_basis_analytical_role_contradictions_are_rejected() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    let bad_predictor = previous("B", "v2", 2, 3, 2, state(Mode::Build, Phase::Free, 0));
    assert_eq!(check(&top, "B", Some(bad_predictor), Lineage::SameBasis, ResultClass::Observe).unwrap_err(), Rejection::InvalidHistory);
    let bad_builder = previous("C", "v2", 2, 3, 3, state(Mode::Predict, Phase::Analytic, 0));
    assert_eq!(check(&top, "C", Some(bad_builder), Lineage::SameBasis, ResultClass::Observe).unwrap_err(), Rejection::InvalidHistory);
}

#[test]
fn torn_forked_revoked_and_unadmitted_topologies_fail_closed() {
    let base = topology(&["A", "B", "C"], "v2", 2);
    for evidence in [Evidence::Missing, Evidence::Torn, Evidence::Forked, Evidence::Unauthorized, Evidence::Revoked] {
        let mut top = base.clone();
        top.evidence = evidence;
        assert_eq!(check(&top, "A", None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap_err(), Rejection::TopologyUnknown);
    }
    let mut mismatched = base.clone();
    mismatched.declared_size = 4;
    assert_eq!(check(&mismatched, "A", None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap_err(), Rejection::TopologyUnknown);
    let duplicate = topology(&["A", "B", "B"], "v2", 2);
    assert_eq!(check(&duplicate, "A", None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap_err(), Rejection::TopologyUnknown);
    assert_eq!(check(&base, "X", None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap_err(), Rejection::TopologyUnknown);
}

#[test]
fn missing_or_unproved_source_lineage_cannot_be_fresh_enrollment() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    let p = previous("B", "v1", 1, 3, 2, state(Mode::Audit, Phase::Analytic, 0));
    assert_eq!(check(&top, "B", Some(p.clone()), Lineage::FirstEnrollment, ResultClass::Observe).unwrap_err(), Rejection::UnprovenLineage);
    assert_eq!(check(&top, "B", Some(p), Lineage::Unproven, ResultClass::Observe).unwrap_err(), Rejection::UnprovenLineage);
    assert_eq!(check(&top, "B", None, Lineage::SameBasis, ResultClass::Observe).unwrap_err(), Rejection::UnprovenLineage);
}

#[test]
fn missing_forked_or_mismatched_counter_proof_cannot_carry_streak() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    let mut p = previous("C", "v2", 2, 3, 3, state(Mode::Plan, Phase::Bootstrap, 1));
    p.counter = None;
    assert_eq!(check(&top, "C", Some(p.clone()), Lineage::SameBasis, ResultClass::Noop).unwrap_err(), Rejection::CounterUnknown);
    p.counter = Some(CounterProof {
        basis: "v2".into(), generation: 2, principal: "C".into(),
        predecessor_known: false, transition_rule_version: 1,
    });
    assert_eq!(check(&top, "C", Some(p), Lineage::SameBasis, ResultClass::Noop).unwrap_err(), Rejection::CounterUnknown);
}

#[test]
fn counter_exact_second_noop_transitions_only_in_nonpermanent_role() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    let builder = previous("C", "v2", 2, 3, 3, state(Mode::Plan, Phase::Bootstrap, 1));
    let c = check(&top, "C", Some(builder), Lineage::SameBasis, ResultClass::Noop).unwrap();
    assert_eq!((c.state.mode, c.state.noop_streak), (Mode::Predict, 0));
    let perm = previous("B", "v2", 2, 3, 2, state(Mode::Predict, Phase::Analytic, 1));
    let d = check(&top, "B", Some(perm), Lineage::SameBasis, ResultClass::Noop).unwrap();
    assert_eq!(d.state.mode, Mode::Predict);
}

#[test]
fn compatible_free_builder_can_continue_but_incompatible_resets() {
    let new = topology(&["A", "B", "C", "D"], "new", 2);
    let prior = previous("C", "old", 1, 3, 3, state(Mode::Build, Phase::Free, 0));
    let carried = check(&new, "C", Some(prior.clone()), Lineage::SyntheticCompatibleBuilder {
        from: "old".into(), to: "new".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(carried.basis, BasisDisposition::CompatibleBuilderCarry);
    assert_eq!(carried.state.phase, Phase::Free);
    let reset = check(&new, "C", Some(prior), Lineage::SyntheticIncompatible {
        from: "old".into(), to: "new".into(),
    }, ResultClass::Observe).unwrap();
    assert_eq!(reset.state, state(Mode::Plan, Phase::Bootstrap, 0));
}

#[test]
fn missing_build_evidence_is_not_a_valid_free_build_transition() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    let prior = previous("C", "v2", 2, 3, 3, state(Mode::Build, Phase::Free, 0));
    assert_eq!(check(&top, "C", Some(prior), Lineage::SameBasis, ResultClass::Material).unwrap_err(), Rejection::IncompatibleResult);
}

#[test]
fn size_two_analyst_cycles_without_becoming_a_builder() {
    let top = topology(&["A", "B"], "duo", 2);
    let prior = previous("A", "duo", 2, 2, 1, state(Mode::Audit, Phase::Analytic, 1));
    let c = check(&top, "A", Some(prior), Lineage::SameBasis, ResultClass::Noop).unwrap();
    assert_eq!((c.state.mode, c.state.phase, c.state.noop_streak), (Mode::Plan, Phase::Analytic, 0));
}

#[test]
fn solo_run_needs_external_mission_decision() {
    let top = topology(&["A"], "solo", 2);
    assert_eq!(check(&top, "A", None, Lineage::FirstEnrollment, ResultClass::Noop).unwrap_err(), Rejection::MissionDependentSolo);
}

#[test]
fn even_positive_predictions_never_grant_provider_append() {
    let top = topology(&["A", "B", "C"], "v2", 2);
    for who in ["A", "B", "C"] {
        let result = check(&top, who, None, Lineage::FirstEnrollment, ResultClass::Observe).unwrap();
        assert!(!result.provider_append_authorized);
    }
}
