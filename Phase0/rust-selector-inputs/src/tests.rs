use super::*;

fn fixture() -> Snapshot {
    Snapshot {
        claim: "measured-latency".into(),
        schema: 1,
        cut: 10,
        policy_generation: 3,
        registry_generation: 4,
        complete: true,
        jointly_fenced: true,
        models: vec![
            Model {
                id: "conservative".into(),
                incarnation: "m1".into(),
            },
            Model {
                id: "optimistic".into(),
                incarnation: "m2".into(),
            },
        ],
        sources: ["registry-a", "registry-b"]
            .iter()
            .map(|name| Source {
                id: (*name).into(),
                incarnation: format!("{name}-1"),
                generation: 7,
                valid_from: 1,
                valid_until: 20,
                high_water: 10,
                dependencies: vec![],
            })
            .collect(),
        ranks: ["registry-a", "registry-b"]
            .iter()
            .flat_map(|name| {
                [("conservative", "m1", 100), ("optimistic", "m2", 900)]
                    .into_iter()
                    .map(move |(model, model_incarnation, score)| Rank {
                        model: model.into(),
                        model_incarnation: model_incarnation.into(),
                        source: (*name).into(),
                        source_incarnation: format!("{name}-1"),
                        source_generation: 7,
                        cut: 10,
                        score,
                        unit: "basis-points".into(),
                    })
            })
            .collect(),
    }
}

#[test]
fn s6_10_complete_current_simulation_selects_without_confidence_gain() {
    let selection = evaluate(&fixture()).unwrap();
    assert_eq!(selection.model, "conservative");
    assert_eq!(selection.score, 100);
    assert!(!selection.confidence_gain);
    assert!(selection.semantic_basis.starts_with("selector-input-v1|"));
}

#[test]
fn s6_01_unregistered_favorable_evidence_does_not_select() {
    let mut s = fixture();
    s.ranks.push(Rank {
        model: "optimistic".into(),
        model_incarnation: "m2".into(),
        source: "attacker".into(),
        source_incarnation: "fake".into(),
        source_generation: 1,
        cut: 10,
        score: 0,
        unit: "basis-points".into(),
    });
    assert_eq!(evaluate(&s), Err(Denial::UntrustedLineage));
}

#[test]
fn s6_02_missing_stale_and_unpaged_required_source_denied() {
    let mut s = fixture();
    s.ranks.pop();
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    s = fixture();
    s.sources[0].valid_until = 9;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    s = fixture();
    s.sources[0].high_water = 9;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
}

#[test]
fn s6_03_source_generation_change_after_preflight_is_stale() {
    let old = fixture();
    let mut newer = old.clone();
    newer.sources[0].generation = 8;
    for rank in &mut newer.ranks {
        if rank.source == "registry-a" {
            rank.source_generation = 8;
        }
    }
    assert_eq!(
        Journal::default().submit(11, &old, &newer),
        Err(Denial::EffectTimeDrift)
    );
}

#[test]
fn oversized_source_dependency_vector_is_rejected_before_graph_walk() {
    let mut s = fixture();
    // With only two registered sources no valid declaration can contain
    // thousands of dependency edges. Reject raw fanout before sorting.
    s.sources[0].dependencies = vec!["registry-b".into(); 4096];
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
}

#[test]
fn one_registered_acyclic_dependency_still_selects() {
    let mut s = fixture();
    s.sources[0].dependencies.push("registry-b".into());
    let selected = evaluate(&s).unwrap();
    assert_eq!(selected.model, "conservative");
    assert_eq!(selected.score, 100);
    assert!(!selected.confidence_gain);
}

#[test]
fn s6_04_conflicting_rank_sources_have_no_favorable_tiebreak() {
    let mut s = fixture();
    s.ranks[0].score = 999;
    assert_eq!(evaluate(&s), Err(Denial::AmbiguousEvidence));
}

#[test]
fn s6_05_direct_and_indirect_scored_claim_cycles_deny() {
    let mut s = fixture();
    let claim = s.claim.clone();
    s.sources[0].dependencies.push(claim);
    assert_eq!(evaluate(&s), Err(Denial::UntrustedLineage));
    let mut s = fixture();
    s.sources[0].dependencies.push("registry-b".into());
    s.sources[1].dependencies.push("registry-a".into());
    assert_eq!(evaluate(&s), Err(Denial::UntrustedLineage));
}

#[test]
fn s6_06_reordering_candidate_source_and_evidence_sets_is_nonsemantic() {
    let selection = evaluate(&fixture()).unwrap();
    let mut s = fixture();
    s.models.reverse();
    s.sources.reverse();
    s.ranks.reverse();
    assert_eq!(evaluate(&s), Ok(selection));
}

#[test]
fn s6_07_torn_source_cut_and_missing_joint_fence_deny() {
    let mut s = fixture();
    s.ranks[0].cut = 9;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    let mut s = fixture();
    s.jointly_fenced = false;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
}

#[test]
fn s6_08_invalid_unit_duplicate_evidence_and_false_incarnation_deny() {
    let mut s = fixture();
    s.ranks[0].unit = "milliseconds".into();
    assert_eq!(evaluate(&s), Err(Denial::Malformed));
    let mut s = fixture();
    s.ranks.push(s.ranks[0].clone());
    assert_eq!(evaluate(&s), Err(Denial::AmbiguousEvidence));
    let mut s = fixture();
    s.ranks[0].source_incarnation = "restored".into();
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
}

#[test]
fn s6_09_ack_loss_reconciles_exact_original_and_rejects_retargeting() {
    let old = fixture();
    let mut journal = Journal::default();
    assert!(matches!(
        journal.submit(81, &old, &old),
        Ok(Outcome::Committed(_))
    ));
    let mut moved = old.clone();
    moved.registry_generation += 1;
    assert_eq!(
        journal.submit(81, &old, &moved),
        Ok(Outcome::Reconciled(evaluate(&old).unwrap()))
    );
    assert_eq!(
        journal.submit(81, &moved, &moved),
        Err(Denial::ConflictingReplay)
    );
    assert_eq!(
        Journal::default().submit(81, &old, &moved),
        Err(Denial::EffectTimeDrift)
    );
}

#[test]
fn unresolved_best_score_tie_is_not_success() {
    let mut s = fixture();
    for rank in &mut s.ranks {
        rank.score = 100;
    }
    assert_eq!(evaluate(&s), Err(Denial::NoUniqueWinner));
}

#[test]
fn invalid_rank_bounds_and_unknown_upstream_cannot_gain_confidence() {
    let mut s = fixture();
    s.ranks[0].score = i64::MAX;
    assert_eq!(evaluate(&s), Err(Denial::Malformed));
    let mut s = fixture();
    s.sources[0].dependencies.push("missing-origin".into());
    assert_eq!(evaluate(&s), Err(Denial::UntrustedLineage));
}

#[test]
fn identity_movement_invalidates_basis_without_changing_numeric_winner() {
    let original = evaluate(&fixture()).unwrap();
    let mut s = fixture();
    s.sources[0].incarnation = "registry-a-second-incarnation".into();
    for rank in &mut s.ranks {
        if rank.source == "registry-a" {
            rank.source_incarnation = "registry-a-second-incarnation".into();
        }
    }
    let moved = evaluate(&s).unwrap();
    assert_eq!(original.model, moved.model);
    assert_eq!(original.score, moved.score);
    assert_ne!(original.semantic_basis, moved.semantic_basis);
}

#[test]
fn registry_incomplete_or_duplicate_members_fail_closed() {
    let mut s = fixture();
    s.complete = false;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    let mut s = fixture();
    s.sources.push(s.sources[0].clone());
    assert_eq!(evaluate(&s), Err(Denial::Malformed));
    let mut s = fixture();
    s.models.push(s.models[0].clone());
    assert_eq!(evaluate(&s), Err(Denial::Malformed));
}

#[test]
fn unknown_policy_or_schema_and_zero_operation_not_accepted() {
    let mut s = fixture();
    s.schema = 2;
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    let s = fixture();
    assert_eq!(Journal::default().submit(0, &s, &s), Err(Denial::Malformed));
}

#[test]
fn stale_candidate_incarnation_cannot_reuse_prior_rank_evidence() {
    let baseline = evaluate(&fixture()).unwrap();
    let mut s = fixture();
    s.models[1].incarnation = "m3".into();
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));

    for rank in &mut s.ranks {
        if rank.model == "optimistic" {
            rank.model_incarnation = "m3".into();
        }
    }
    let refreshed = evaluate(&s).unwrap();
    assert_eq!(refreshed.model, baseline.model);
    assert_eq!(refreshed.score, baseline.score);
    assert_ne!(refreshed.semantic_basis, baseline.semantic_basis);
    assert!(!refreshed.confidence_gain);
}

#[test]
fn forged_rank_model_incarnation_is_denied_without_state_change() {
    let mut s = fixture();
    s.ranks[0].model_incarnation = "m2".into();
    assert_eq!(evaluate(&s), Err(Denial::Incomplete));
    assert_eq!(evaluate(&fixture()).unwrap().model, "conservative");
}
