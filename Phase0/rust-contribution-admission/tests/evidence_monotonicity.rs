//! Offline synthetic #159 spec-3 trust-boundary regressions.
//!
//! These fixtures do not prove DCO assent, legal rights, provider effects or
//! production publication authority. All positive decisions remain simulation-only.
use std::collections::BTreeSet;

use free_energy_contribution_admission::{
    canonical_digest, evaluate, Admission, Claim, Context, Decision, Evidence, OperationKey,
    ProviderObservation, Replay, SimulationJournal,
};

fn positive() -> Admission {
    Admission {
        schema_version: 1,
        repository_id: 1360059617,
        repository_incarnation: "synthetic-incarnation-a".into(),
        source_commit: "synthetic-source-a".into(),
        intended_target_head: "synthetic-target-a".into(),
        observed_target_head: "synthetic-target-a".into(),
        input_digest: "synthetic-input-a".into(),
        output_digest: "synthetic-output-a".into(),
        reviewed_output_digest: "synthetic-output-a".into(),
        terms_version: 4,
        current_terms_version: 4,
        authority_generation: 8,
        current_authority_generation: 8,
        transformation_occurred: true,
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
            transformation_map: Claim::SyntheticFixture,
            historical_lineage: Claim::SyntheticFixture,
        },
        github_signature_verified: false,
        dco_bot_exempt: false,
    }
}

fn degrade(admission: &mut Admission, field: usize, state: Claim) {
    match field {
        0 => admission.evidence.human_act = state,
        1 => admission.evidence.dco_declaration = state,
        2 => admission.evidence.terms_assent = state,
        3 => admission.evidence.code_rights = state,
        4 => admission.evidence.asset_rights = state,
        5 => admission.evidence.employer_rights = state,
        6 => admission.evidence.reviewer_decision = state,
        7 => admission.evidence.task_authority = state,
        8 => admission.evidence.transformation_map = state,
        9 => admission.evidence.historical_lineage = state,
        _ => unreachable!(),
    }
}

fn key(operation_id: String) -> OperationKey {
    OperationKey {
        repository_id: 1360059617,
        repository_incarnation: "synthetic-incarnation-a".into(),
        operation_id,
        authority_generation: 8,
    }
}

#[test]
fn all_single_proof_degradations_fail_closed_even_with_provider_badges() {
    let approved = positive();
    assert_eq!(evaluate(&approved), Decision::ReviewableInSimulation);
    let mut seen = BTreeSet::new();
    let states = [Claim::Absent, Claim::CallerClaim, Claim::Disputed, Claim::Revoked];
    let mut checked = 0;
    for field in 0..10 {
        for state in states {
            let mut input = approved.clone();
            degrade(&mut input, field, state);
            // GitHub verification and bot exemptions are not human assent.
            for verified in [false, true] {
                for bot_exempt in [false, true] {
                    input.github_signature_verified = verified;
                    input.dco_bot_exempt = bot_exempt;
                    let outcome = evaluate(&input);
                    assert_ne!(
                        outcome,
                        Decision::ReviewableInSimulation,
                        "field={field} state={state:?} verified={verified} bot={bot_exempt}"
                    );
                    checked += 1;
                }
            }
            // Every differently degraded *field/state* has a distinct payload
            // identity; a caller cannot swap evidence without changing digest.
            let digest = canonical_digest(&input);
            assert!(seen.insert(digest), "canonical proof positions must not alias");
        }
    }
    assert_eq!(checked, 160);
    assert_eq!(seen.len(), 40);
}

#[test]
fn changed_evidence_cannot_replace_a_reviewable_original_receipt() {
    let approved = positive();
    let mut journal = SimulationJournal::default();
    let original_key = key("synthetic-sealed-proof".into());
    assert_eq!(
        journal.record(original_key.clone(), &approved),
        Replay::First(Decision::ReviewableInSimulation)
    );

    // Same operation ID and changed proof must conflict, not inherit the
    // original reviewable decision or silently replace the initial receipt.
    for field in 0..10 {
        for state in [Claim::Absent, Claim::CallerClaim, Claim::Disputed, Claim::Revoked] {
            let mut degraded = approved.clone();
            degrade(&mut degraded, field, state);
            assert_eq!(
                journal.record(original_key.clone(), &degraded),
                Replay::Conflict,
                "field={field} state={state:?}"
            );
        }
    }
    assert_eq!(
        journal.record(original_key, &approved),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}

#[test]
fn independently_denied_receipts_stay_denied_after_replay() {
    let approved = positive();
    let mut journal = SimulationJournal::default();
    let states = [Claim::Absent, Claim::CallerClaim, Claim::Disputed, Claim::Revoked];
    let mut checked = 0;
    for field in 0..10 {
        for (index, state) in states.into_iter().enumerate() {
            let mut degraded = approved.clone();
            degrade(&mut degraded, field, state);
            let operation = key(format!("synthetic-denied-{field}-{index}"));
            let original = evaluate(&degraded);
            assert_ne!(original, Decision::ReviewableInSimulation);
            assert_eq!(
                journal.record(operation.clone(), &degraded),
                Replay::First(original)
            );
            assert_eq!(
                journal.record(operation.clone(), &degraded),
                Replay::Identical(original)
            );
            assert_eq!(journal.record(operation, &approved), Replay::Conflict);
            checked += 1;
        }
    }
    assert_eq!(checked, 40);
    assert_eq!(journal.provider_effects_emitted(), 0);
}
