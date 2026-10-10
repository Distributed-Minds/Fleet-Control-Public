use free_energy_contribution_admission::{
    canonical_bytes, canonical_digest, evaluate, Admission, Claim, Context, Decision, Evidence,
    OperationKey, ProviderObservation, Reason, Replay, SimulationJournal, MAX_SIMULATED_RECEIPTS,
};

fn good() -> Admission {
    Admission {
        schema_version: 1,
        repository_id: 1360059617,
        repository_incarnation: "synthetic-repo-generation-a".into(),
        source_commit: "synthetic-source-1".into(),
        intended_target_head: "synthetic-target-1".into(),
        observed_target_head: "synthetic-target-1".into(),
        input_digest: "synthetic-sha256-input-a".into(),
        output_digest: "synthetic-sha256-output-a".into(),
        reviewed_output_digest: "synthetic-sha256-output-a".into(),
        terms_version: 3,
        current_terms_version: 3,
        authority_generation: 8,
        current_authority_generation: 8,
        transformation_occurred: false,
        credential_identity_ambiguous: false,
        context: Context::OfflineSimulation,
        provider_observation: ProviderObservation::NoEffectAttempted,
        evidence: Evidence {
            human_act: Claim::SyntheticFixture,
            dco_declaration: Claim::SyntheticFixture,
            terms_assent: Claim::SyntheticFixture,
            code_rights: Claim::SyntheticFixture,
            asset_rights: Claim::SyntheticFixture,
            employer_rights: Claim::SyntheticFixture,
            reviewer_decision: Claim::SyntheticFixture,
            task_authority: Claim::SyntheticFixture,
            transformation_map: Claim::Absent,
            historical_lineage: Claim::SyntheticFixture,
        },
        github_signature_verified: false,
        dco_bot_exempt: false,
    }
}

#[test]
fn a01_only_synthetic_review_eligibility_never_publication() {
    assert_eq!(evaluate(&good()), Decision::ReviewableInSimulation);
}

#[test]
fn a02_explicit_transformation_map_still_only_reviewable() {
    let mut x = good();
    x.transformation_occurred = true;
    x.evidence.transformation_map = Claim::SyntheticFixture;
    assert_eq!(evaluate(&x), Decision::ReviewableInSimulation);
}

#[test]
fn d01_verified_github_actor_cannot_replace_human_dco() {
    let mut x = good();
    x.evidence.dco_declaration = Claim::Absent;
    x.github_signature_verified = true;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MissingDco));
}

#[test]
fn d02_bot_copied_human_assent_is_only_a_caller_claim() {
    let mut x = good();
    x.evidence.human_act = Claim::CallerClaim;
    x.evidence.dco_declaration = Claim::CallerClaim;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MissingHumanAct));
}

#[test]
fn d03_coauthor_only_without_dco_fails() {
    let mut x = good();
    x.evidence.dco_declaration = Claim::Absent;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MissingDco));
}

#[test]
fn d04_superseded_terms_require_fresh_human_review() {
    let mut x = good();
    x.current_terms_version += 1;
    assert_eq!(
        evaluate(&x),
        Decision::HumanReviewRequired(Reason::StaleTerms)
    );
}

#[test]
fn d05_changed_reviewed_tree_invalidates_old_approval() {
    let mut x = good();
    x.output_digest = "changed-output".into();
    assert_eq!(
        evaluate(&x),
        Decision::Blocked(Reason::ChangedReviewedContent)
    );
}

#[test]
fn d06_disputed_employer_or_asset_rights_require_review() {
    for mutator in 0..2 {
        let mut x = good();
        if mutator == 0 {
            x.evidence.employer_rights = Claim::Disputed;
        } else {
            x.evidence.asset_rights = Claim::Disputed;
        }
        assert_eq!(
            evaluate(&x),
            Decision::HumanReviewRequired(Reason::UnclearRights)
        );
    }
}

#[test]
fn d07_shared_github_credential_is_not_individual_assent() {
    let mut x = good();
    x.credential_identity_ambiguous = true;
    assert_eq!(evaluate(&x), Decision::Unknown(Reason::AmbiguousCredential));
}

#[test]
fn d08_revoked_authority_or_moved_head_blocks() {
    let mut x = good();
    x.current_authority_generation += 1;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::StaleAuthority));
    let mut x = good();
    x.observed_target_head = "moved".into();
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MovedTarget));
}

#[test]
fn d09_lost_ack_reconciles_without_duplicate_effect() {
    let mut x = good();
    x.provider_observation = ProviderObservation::AcknowledgementLost;
    let result = evaluate(&x);
    assert_eq!(result, Decision::ReconcileOriginalAttempt);
    let mut journal = SimulationJournal::default();
    let key = key();
    assert_eq!(journal.record(key.clone(), &x), Replay::First(result));
    assert_eq!(journal.record(key, &x), Replay::Identical(result));
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn d10_conflicting_remote_resource_is_not_our_effect() {
    let mut x = good();
    x.provider_observation = ProviderObservation::ConflictingObject;
    assert_eq!(
        evaluate(&x),
        Decision::Blocked(Reason::ConflictingRemoteEffect)
    );
    x.provider_observation = ProviderObservation::MatchingObjectUnattributed;
    assert_eq!(evaluate(&x), Decision::ReconcileOriginalAttempt);
}

fn key() -> OperationKey {
    OperationKey {
        repository_id: 1360059617,
        repository_incarnation: "synthetic-repo-generation-a".into(),
        operation_id: "synthetic-attempt-001".into(),
        authority_generation: 8,
    }
}

#[test]
fn d11_two_simulated_claimants_cannot_replace_original_receipt() {
    let mut journal = SimulationJournal::default();
    let admitted = good();
    let mut changed = admitted.clone();
    changed.evidence.human_act = Claim::CallerClaim;
    assert_eq!(
        journal.record(key(), &admitted),
        Replay::First(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.record(key(), &changed), Replay::Conflict);
    assert_eq!(
        journal.record(key(), &admitted),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn d12_unreviewed_transformation_requires_new_review() {
    let mut x = good();
    x.transformation_occurred = true;
    assert_eq!(
        evaluate(&x),
        Decision::HumanReviewRequired(Reason::UnreviewedTransformation)
    );
}

#[test]
fn d13_bot_check_exemption_does_not_create_dco_proof() {
    let mut x = good();
    x.dco_bot_exempt = true;
    x.evidence.dco_declaration = Claim::Absent;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MissingDco));
}

#[test]
fn d14_old_history_without_clearance_is_not_retroactively_signed() {
    let mut x = good();
    x.evidence.historical_lineage = Claim::Absent;
    assert_eq!(evaluate(&x), Decision::Unknown(Reason::UnverifiedHistory));
}

#[test]
fn d15_interactive_permissions_cannot_prove_scheduled_context() {
    let mut x = good();
    x.context = Context::InteractiveOnly;
    assert_eq!(evaluate(&x), Decision::Unknown(Reason::UnverifiedContext));
}

#[test]
fn d16_throttled_response_stays_reconciliation_only() {
    let mut x = good();
    x.provider_observation = ProviderObservation::RateLimited;
    assert_eq!(evaluate(&x), Decision::ReconcileOriginalAttempt);
}

#[test]
fn negative_one_predicate_from_positive_never_grants_review_eligibility() {
    let mut x = good();
    x.evidence.human_act = Claim::CallerClaim;
    assert_eq!(evaluate(&x), Decision::Blocked(Reason::MissingHumanAct));
}

#[test]
fn unknown_schema_and_missing_resource_identity_fail_closed() {
    let mut x = good();
    x.schema_version = 2;
    assert_eq!(evaluate(&x), Decision::Unknown(Reason::MalformedIdentity));
    let mut x = good();
    x.repository_id = 0;
    assert_eq!(evaluate(&x), Decision::Unknown(Reason::MalformedIdentity));
}

#[test]
fn journal_cannot_promote_original_blocked_payload_by_replay() {
    let mut journal = SimulationJournal::default();
    let mut denied = good();
    denied.evidence.human_act = Claim::CallerClaim;
    let disposition = Decision::Blocked(Reason::MissingHumanAct);
    assert_eq!(journal.record(key(), &denied), Replay::First(disposition));
    assert_eq!(
        journal.record(key(), &good()),
        Replay::Conflict,
        "a different envelope cannot upgrade the original blocked decision"
    );
    assert_eq!(
        journal.record(key(), &denied),
        Replay::Identical(disposition)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn journal_rejects_key_substitution_before_creating_receipt() {
    let admission = good();
    let mut journal = SimulationJournal::default();
    let mut wrong_repo = key();
    wrong_repo.repository_id += 1;
    assert_eq!(journal.record(wrong_repo, &admission), Replay::Conflict);
    let mut wrong_incarnation = key();
    wrong_incarnation.repository_incarnation = "other-repo".into();
    assert_eq!(
        journal.record(wrong_incarnation, &admission),
        Replay::Conflict
    );
    let mut wrong_generation = key();
    wrong_generation.authority_generation += 1;
    assert_eq!(
        journal.record(wrong_generation, &admission),
        Replay::Conflict
    );
    let mut missing_operation = key();
    missing_operation.operation_id.clear();
    assert_eq!(
        journal.record(missing_operation, &admission),
        Replay::Conflict
    );
    assert_eq!(
        journal.record(key(), &admission),
        Replay::First(Decision::ReviewableInSimulation)
    );
}

#[test]
fn journal_rejects_reused_operation_id_after_authority_generation_change() {
    let mut journal = SimulationJournal::default();
    let original = good();
    assert_eq!(
        journal.record(key(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );
    let mut next = original.clone();
    next.authority_generation += 1;
    next.current_authority_generation += 1;
    let mut recycled_key = key();
    recycled_key.authority_generation += 1;
    assert_eq!(journal.record(recycled_key, &next), Replay::Conflict);
    assert_eq!(
        journal.record(key(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn journal_rejects_reused_operation_after_repository_reincarnation() {
    let mut journal = SimulationJournal::default();
    let original = good();
    assert_eq!(
        journal.record(key(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );

    let mut reincarnated = original.clone();
    reincarnated.repository_incarnation = "synthetic-repo-generation-b".into();
    let mut recycled_key = key();
    recycled_key.repository_incarnation = reincarnated.repository_incarnation.clone();
    assert_eq!(
        journal.record(recycled_key, &reincarnated),
        Replay::Conflict
    );
    assert_eq!(
        journal.record(key(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );

    let mut independent = original.clone();
    independent.repository_id += 1;
    let mut independent_key = key();
    independent_key.repository_id = independent.repository_id;
    assert_eq!(
        journal.record(independent_key, &independent),
        Replay::First(Decision::ReviewableInSimulation),
        "independent repositories have distinct operation namespaces"
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn canonical_encoding_is_unicode_safe_field_separated_and_sensitive() {
    let mut a = good();
    let base = canonical_digest(&a);
    assert_eq!(base.len(), 64);
    a.source_commit = "é".into();
    let unicode = canonical_bytes(&a);
    assert_ne!(canonical_digest(&a), base);
    a.source_commit = "e\u{301}".into();
    assert_ne!(canonical_bytes(&a), unicode);
    let mut a = good();
    a.source_commit = "ab".into();
    a.intended_target_head = "c".into();
    let mut b = good();
    b.source_commit = "a".into();
    b.intended_target_head = "bc".into();
    assert_ne!(canonical_bytes(&a), canonical_bytes(&b));
    assert_ne!(canonical_digest(&a), canonical_digest(&b));
}

#[test]
fn malformed_synthetic_resource_tokens_and_zero_generations_fail_closed() {
    let invalid = [
        " ".to_owned(),
        "has space".to_owned(),
        "line\nfeed".to_owned(),
        "bidi\u{202e}reversal".to_owned(),
        "path/segment".to_owned(),
        "x".repeat(257),
    ];
    for value in invalid {
        for field in 0..7 {
            let mut x = good();
            match field {
                0 => x.repository_incarnation = value.clone(),
                1 => x.source_commit = value.clone(),
                2 => x.intended_target_head = value.clone(),
                3 => x.observed_target_head = value.clone(),
                4 => x.input_digest = value.clone(),
                5 => x.output_digest = value.clone(),
                6 => x.reviewed_output_digest = value.clone(),
                _ => unreachable!(),
            }
            assert_eq!(
                evaluate(&x),
                Decision::Unknown(Reason::MalformedIdentity),
                "field {field}, value {value:?}"
            );
        }
    }

    for field in 0..4 {
        let mut x = good();
        match field {
            0 => x.terms_version = 0,
            1 => x.current_terms_version = 0,
            2 => x.authority_generation = 0,
            3 => x.current_authority_generation = 0,
            _ => unreachable!(),
        }
        assert_eq!(evaluate(&x), Decision::Unknown(Reason::MalformedIdentity));
    }

    let mut boundary = good();
    boundary.source_commit = "a".repeat(256);
    assert_eq!(evaluate(&boundary), Decision::ReviewableInSimulation);
    boundary.source_commit.push('a');
    assert_eq!(
        evaluate(&boundary),
        Decision::Unknown(Reason::MalformedIdentity)
    );
}

#[test]
fn malformed_operation_id_cannot_create_or_replace_synthetic_receipt() {
    let original = good();
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.record(key(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );
    for invalid in [
        " ".to_owned(),
        "a\nb".to_owned(),
        "spoof\u{202e}".to_owned(),
        "a/b".to_owned(),
        "o".repeat(257),
    ] {
        let mut wrong = key();
        wrong.operation_id = invalid;
        assert_eq!(journal.record(wrong, &original), Replay::Conflict);
    }
    assert_eq!(
        journal.record(key(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn journal_rejects_unbounded_payload_fields_before_digest_or_receipt_insertion() {
    let original = good();
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.record(key(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );

    // Even an otherwise valid replay operation must not hash caller-controlled
    // unbounded text. Check every field consumed by canonical_bytes.
    let oversized = "a".repeat(1024 * 1024);
    for index in 0..7 {
        let mut input = original.clone();
        match index {
            0 => input.repository_incarnation = oversized.clone(),
            1 => input.source_commit = oversized.clone(),
            2 => input.intended_target_head = oversized.clone(),
            3 => input.observed_target_head = oversized.clone(),
            4 => input.input_digest = oversized.clone(),
            5 => input.output_digest = oversized.clone(),
            6 => input.reviewed_output_digest = oversized.clone(),
            _ => unreachable!(),
        }
        assert_eq!(journal.record(key(), &input), Replay::Conflict);
    }

    // Rejected oversized attempts cannot replace the historical receipt.
    assert_eq!(
        journal.record(key(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn journal_capacity_fails_closed_without_eviction_or_replay_downgrade() {
    let original = good();
    let mut journal = SimulationJournal::default();
    for index in 0..MAX_SIMULATED_RECEIPTS {
        let mut operation = key();
        operation.operation_id = format!("synthetic-{index:04}");
        assert_eq!(
            journal.record(operation, &original),
            Replay::First(Decision::ReviewableInSimulation)
        );
    }

    let mut over_capacity = key();
    over_capacity.operation_id = "new-operation".into();
    assert_eq!(
        journal.record(over_capacity.clone(), &original),
        Replay::CapacityExhausted
    );
    // No failed request was inserted, including on a repeated attempt.
    assert_eq!(
        journal.record(over_capacity, &original),
        Replay::CapacityExhausted
    );

    let mut first_operation = key();
    first_operation.operation_id = "synthetic-0000".into();
    assert_eq!(
        journal.record(first_operation.clone(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    let mut changed = original.clone();
    changed.output_digest = "different-reviewed-payload".into();
    assert_eq!(journal.record(first_operation, &changed), Replay::Conflict);
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn canonical_envelope_fingerprint_binds_every_synthetic_field() {
    let original = good();
    let original_bytes = canonical_bytes(&original);
    let original_digest = canonical_digest(&original);

    // Each v1 field must independently alter the canonical encoding and hash.
    // Some variants become invalid admissions; this tests byte identity only,
    // not real consent or permission to publish.
    for field in 0..29 {
        let mut changed = original.clone();
        match field {
            0 => changed.repository_incarnation.push('x'),
            1 => changed.source_commit.push('x'),
            2 => changed.intended_target_head.push('x'),
            3 => changed.observed_target_head.push('x'),
            4 => changed.input_digest.push('x'),
            5 => changed.output_digest.push('x'),
            6 => changed.reviewed_output_digest.push('x'),
            7 => changed.schema_version += 1,
            8 => changed.repository_id += 1,
            9 => changed.terms_version += 1,
            10 => changed.current_terms_version += 1,
            11 => changed.authority_generation += 1,
            12 => changed.current_authority_generation += 1,
            13 => changed.transformation_occurred = true,
            14 => changed.credential_identity_ambiguous = true,
            15 => changed.context = Context::InteractiveOnly,
            16 => changed.provider_observation = ProviderObservation::RateLimited,
            17 => changed.evidence.human_act = Claim::Disputed,
            18 => changed.evidence.dco_declaration = Claim::Disputed,
            19 => changed.evidence.terms_assent = Claim::Disputed,
            20 => changed.evidence.code_rights = Claim::Disputed,
            21 => changed.evidence.asset_rights = Claim::Disputed,
            22 => changed.evidence.employer_rights = Claim::Disputed,
            23 => changed.evidence.reviewer_decision = Claim::Disputed,
            24 => changed.evidence.task_authority = Claim::Disputed,
            25 => changed.evidence.transformation_map = Claim::Disputed,
            26 => changed.evidence.historical_lineage = Claim::Disputed,
            27 => changed.github_signature_verified = true,
            28 => changed.dco_bot_exempt = true,
            _ => unreachable!(),
        }
        assert_ne!(canonical_bytes(&changed), original_bytes, "field {field}");
        assert_ne!(canonical_digest(&changed), original_digest, "field {field}");
    }
}

#[test]
fn journal_rejects_equal_decision_with_changed_synthetic_evidence_bytes() {
    let original = good();
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.record(key(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );

    // These changes preserve evaluate()'s simulation decision. The journal
    // must nevertheless bind the original exact input, not just its result.
    for field in 0..3 {
        let mut changed = original.clone();
        match field {
            0 => changed.input_digest.push('x'),
            1 => changed.github_signature_verified = true,
            2 => changed.dco_bot_exempt = true,
            _ => unreachable!(),
        }
        assert_eq!(evaluate(&changed), Decision::ReviewableInSimulation);
        assert_eq!(journal.record(key(), &changed), Replay::Conflict);
    }
    assert_eq!(
        journal.record(key(), &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}
