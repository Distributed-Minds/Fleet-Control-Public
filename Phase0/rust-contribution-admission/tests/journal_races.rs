//! Synthetic, offline-only concurrent callers for #159 spec 3 / D11.
//!
//! These tests check linearized in-memory journal outcomes when callers share
//! one explicitly locked instance. They do NOT exercise a durable database,
//! remote GitHub effects, authenticated identities, or human DCO consent.

use std::sync::{Arc, Barrier, Mutex};
use std::thread;

use free_energy_contribution_admission::{
    Admission, Claim, Context, Decision, Evidence, OperationKey, ProviderObservation, Replay,
    SimulationJournal,
};

fn fixture() -> Admission {
    Admission {
        schema_version: 1,
        repository_id: 1360059617,
        repository_incarnation: "synthetic-repo-generation-a".into(),
        source_commit: "synthetic-source-a".into(),
        intended_target_head: "synthetic-target-a".into(),
        observed_target_head: "synthetic-target-a".into(),
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

fn operation() -> OperationKey {
    OperationKey {
        repository_id: 1360059617,
        repository_incarnation: "synthetic-repo-generation-a".into(),
        operation_id: "synthetic-race-operation".into(),
        authority_generation: 8,
    }
}

/// Start all callers at one barrier, then allow the mutex to serialize each
/// journal transition. Do not mistake this test-only lock for provider fencing.
fn race(inputs: Vec<(OperationKey, Admission)>) -> (Vec<Replay>, usize) {
    assert!(!inputs.is_empty());
    let journal = Arc::new(Mutex::new(SimulationJournal::default()));
    let barrier = Arc::new(Barrier::new(inputs.len()));
    let handles: Vec<_> = inputs
        .into_iter()
        .map(|(key, input)| {
            let journal = Arc::clone(&journal);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                journal.lock().expect("journal lock").record(key, &input)
            })
        })
        .collect();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().expect("caller thread"))
        .collect();
    let effects = journal
        .lock()
        .expect("journal lock after join")
        .provider_effects_emitted();
    (results, effects)
}

fn occurrences(results: &[Replay], wanted: Replay) -> usize {
    results.iter().filter(|result| **result == wanted).count()
}

#[test]
fn concurrent_identical_retry_has_one_first_and_only_identical_replays() {
    let inputs = vec![(operation(), fixture()); 24];
    let (results, effects) = race(inputs);
    let admitted = Decision::ReviewableInSimulation;
    assert_eq!(occurrences(&results, Replay::First(admitted)), 1);
    assert_eq!(occurrences(&results, Replay::Identical(admitted)), 23);
    assert_eq!(effects, 0);
}

#[test]
fn concurrent_same_id_different_payload_cannot_adopt_other_content() {
    let a = fixture();
    let mut b = fixture();
    b.source_commit = "synthetic-source-b".into();
    let inputs = (0..24)
        .map(|index| (operation(), if index % 2 == 0 { a.clone() } else { b.clone() }))
        .collect();
    let (results, effects) = race(inputs);
    let admitted = Decision::ReviewableInSimulation;
    assert_eq!(occurrences(&results, Replay::First(admitted)), 1);
    assert_eq!(occurrences(&results, Replay::Identical(admitted)), 11);
    assert_eq!(occurrences(&results, Replay::Conflict), 12);
    assert_eq!(effects, 0);
}

#[test]
fn concurrent_changed_generation_or_incarnation_cannot_reuse_operation_id() {
    for change_incarnation in [false, true] {
        let a = fixture();
        let key_a = operation();
        let mut b = fixture();
        let mut key_b = operation();
        if change_incarnation {
            b.repository_incarnation = "synthetic-repo-generation-b".into();
            key_b.repository_incarnation = b.repository_incarnation.clone();
        } else {
            b.authority_generation = 9;
            b.current_authority_generation = 9;
            key_b.authority_generation = 9;
        }
        let inputs = (0..24)
            .map(|index| {
                if index % 2 == 0 {
                    (key_a.clone(), a.clone())
                } else {
                    (key_b.clone(), b.clone())
                }
            })
            .collect();
        let (results, effects) = race(inputs);
        let admitted = Decision::ReviewableInSimulation;
        assert_eq!(occurrences(&results, Replay::First(admitted)), 1);
        assert_eq!(occurrences(&results, Replay::Identical(admitted)), 11);
        assert_eq!(occurrences(&results, Replay::Conflict), 12);
        assert_eq!(effects, 0);
    }
}

#[test]
fn unrelated_operation_ids_remain_independently_admissible_under_race() {
    let inputs = (0..32)
        .map(|index| {
            let mut key = operation();
            key.operation_id = format!("synthetic-race-{index:02}");
            (key, fixture())
        })
        .collect();
    let (results, effects) = race(inputs);
    assert_eq!(
        occurrences(&results, Replay::First(Decision::ReviewableInSimulation)),
        32
    );
    assert_eq!(effects, 0);
}
