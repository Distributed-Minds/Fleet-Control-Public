//! Complete one-coordinate canonical-identity matrix for issue #159 spec 3.
//!
//! This is an offline synthetic regression test only. A matching hash or
//! ReviewableInSimulation does not attest legal assent, rights or provider effects.
use std::collections::BTreeSet;

use free_energy_contribution_admission::{
    canonical_digest, Admission, Claim, Context, Decision, Evidence, OperationKey,
    ProviderObservation, Replay, SimulationJournal,
};

fn fixture() -> Admission {
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
        terms_version: 3,
        current_terms_version: 3,
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

fn variants(original: &Admission) -> Vec<(&'static str, Admission)> {
    let mut cases = Vec::new();
    // One independently changed field per case. Preserve the original so the
    // digest and the journal both have an identical immutable replay basis.
    macro_rules! change {
        ($name:literal, evidence.$field:ident, $value:expr) => {{
            let mut candidate = original.clone();
            candidate.evidence.$field = $value;
            cases.push(($name, candidate));
        }};
        ($name:literal, $field:ident, $value:expr) => {{
            let mut candidate = original.clone();
            candidate.$field = $value;
            cases.push(($name, candidate));
        }};
    }
    change!("schema_version", schema_version, 2);
    change!("repository_id", repository_id, 1360059618);
    change!(
        "repository_incarnation",
        repository_incarnation,
        "synthetic-incarnation-b".into()
    );
    change!("source_commit", source_commit, "synthetic-source-b".into());
    change!(
        "intended_target_head",
        intended_target_head,
        "synthetic-target-b".into()
    );
    change!(
        "observed_target_head",
        observed_target_head,
        "synthetic-target-b".into()
    );
    change!("input_digest", input_digest, "synthetic-input-b".into());
    change!("output_digest", output_digest, "synthetic-output-b".into());
    change!(
        "reviewed_output_digest",
        reviewed_output_digest,
        "synthetic-output-b".into()
    );
    change!("terms_version", terms_version, 4);
    change!("current_terms_version", current_terms_version, 4);
    change!("authority_generation", authority_generation, 9);
    change!(
        "current_authority_generation",
        current_authority_generation,
        9
    );
    change!("transformation_occurred", transformation_occurred, false);
    change!(
        "credential_identity_ambiguous",
        credential_identity_ambiguous,
        true
    );
    change!("context", context, Context::InteractiveOnly);
    change!(
        "provider_observation",
        provider_observation,
        ProviderObservation::AcknowledgementLost
    );
    change!("human_act", evidence.human_act, Claim::Absent);
    change!("dco_declaration", evidence.dco_declaration, Claim::Absent);
    change!("terms_assent", evidence.terms_assent, Claim::Absent);
    change!("code_rights", evidence.code_rights, Claim::Absent);
    change!("asset_rights", evidence.asset_rights, Claim::Absent);
    change!("employer_rights", evidence.employer_rights, Claim::Absent);
    change!(
        "reviewer_decision",
        evidence.reviewer_decision,
        Claim::Absent
    );
    change!("task_authority", evidence.task_authority, Claim::Absent);
    change!(
        "transformation_map",
        evidence.transformation_map,
        Claim::Absent
    );
    change!(
        "historical_lineage",
        evidence.historical_lineage,
        Claim::Absent
    );
    change!("github_signature_verified", github_signature_verified, true);
    change!("dco_bot_exempt", dco_bot_exempt, true);
    cases
}

#[test]
fn every_envelope_field_changes_digest_and_conflicts_with_original_receipt() {
    let original = fixture();
    let unchanged_digest = canonical_digest(&original);
    let modified = variants(&original);
    assert_eq!(modified.len(), 29, "cover every Admission coordinate");

    let key = OperationKey {
        repository_id: original.repository_id,
        repository_incarnation: original.repository_incarnation.clone(),
        operation_id: "synthetic-envelope-matrix".into(),
        authority_generation: original.authority_generation,
    };
    let mut journal = SimulationJournal::default();
    assert_eq!(
        journal.record(key.clone(), &original),
        Replay::First(Decision::ReviewableInSimulation)
    );

    let mut unique_digests = BTreeSet::new();
    for (field, candidate) in modified {
        let digest = canonical_digest(&candidate);
        assert_ne!(digest, unchanged_digest, "omitted canonical field: {field}");
        assert!(
            unique_digests.insert(digest),
            "distinct field changes aliased: {field}"
        );
        assert_eq!(
            journal.record(key.clone(), &candidate),
            Replay::Conflict,
            "same operation cannot silently replace changed {field}"
        );
    }
    assert_eq!(unique_digests.len(), 29);
    assert_eq!(
        journal.record(key, &original),
        Replay::Identical(Decision::ReviewableInSimulation)
    );
    assert_eq!(journal.provider_effects_emitted(), 0);
}
