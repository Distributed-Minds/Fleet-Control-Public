//! Offline-only bounded journal capacity fixtures for P0 #11 spec 6.
//! The in-memory selection/issuer claims below are never real provider authority.
use free_energy_lineage_admission::{
    Denied, EffectInput, Eligibility, Registry, Scope, Selection, Transition,
    MAX_SYNTHETIC_LINEAGE_RECEIPTS,
};
use std::collections::BTreeSet;

#[test]
fn bounded_receipt_journal_fails_closed_but_preserves_old_replay() {
    let scope = Scope {
        repository: 1360059617,
        repo_incarnation: "synthetic-repo".into(),
        installation: "synthetic-installation".into(),
        tenant: "synthetic-tenant".into(),
        namespace: "synthetic-namespace".into(),
    };
    let obligations = BTreeSet::from(["unresolved-original".into()]);
    let mut selected = Selection {
        scope: scope.clone(),
        head: "head-0".into(),
        generation: 0,
        policy: 2,
        obligations: obligations.clone(),
        selected_issuer: true,
        unique_current: true,
        frontier_complete: true,
        forked: false,
        revoked: false,
    };
    let mut registry = Registry::new(scope.clone(), selected.head.clone(), 0, 2, obligations);
    let initial_selection = selected.clone();
    let mut first_operation = None;
    let mut latest_receipt = None;

    for index in 0..MAX_SYNTHETIC_LINEAGE_RECEIPTS {
        let transition = Transition {
            id: format!("operation-{index}"),
            scope: scope.clone(),
            predecessor: selected.head.clone(),
            previous_generation: selected.generation,
            policy: selected.policy,
            successor: "synthetic-worker".into(),
            new_head: format!("head-{}", index + 1),
            admitted_by_issuer: true,
        };
        let receipt = registry.admit(&selected, transition.clone()).unwrap();
        if index == 0 {
            first_operation = Some((transition, receipt.clone()));
        }
        selected.head = receipt.transition.new_head.clone();
        selected.generation = receipt.generation;
        latest_receipt = Some(receipt);
    }
    assert_eq!(registry.admitted_count(), MAX_SYNTHETIC_LINEAGE_RECEIPTS);
    let full_head = selected.head.clone();
    let full_generation = selected.generation;

    let overflow = Transition {
        id: "first-overflow".into(),
        scope,
        predecessor: full_head.clone(),
        previous_generation: full_generation,
        policy: selected.policy,
        successor: "synthetic-worker".into(),
        new_head: "head-overflow".into(),
        admitted_by_issuer: true,
    };
    assert_eq!(registry.admit(&selected, overflow.clone()), Err(Denied::CapacityExhausted));
    assert_eq!(registry.admit(&selected, overflow), Err(Denied::CapacityExhausted));
    assert_eq!(registry.head(), (full_head.as_str(), full_generation));
    assert_eq!(registry.admitted_count(), MAX_SYNTHETIC_LINEAGE_RECEIPTS);

    // Old receipts remain accessible with the original historical selection,
    // even though that selection is no longer authorized for any NEW action.
    let (original, original_receipt) = first_operation.unwrap();
    assert_eq!(registry.admit(&initial_selection, original.clone()), Ok(original_receipt.clone()));
    let mut replay_forgery = original;
    replay_forgery.successor = "other-worker".into();
    assert_eq!(registry.admit(&initial_selection, replay_forgery), Err(Denied::ReplayConflict));

    // The newest previously admitted operation remains reviewable in the
    // bounded offline simulation; capacity does not revoke its own receipt.
    let last_receipt = latest_receipt.unwrap();
    let effect = EffectInput {
        cut: selected,
        operation_id: last_receipt.transition.id.clone(),
        actor: last_receipt.transition.successor.clone(),
        authorized_action: true,
        recovery_required: true,
        recovery_authorized: true,
        observed_resource: "synthetic-incarnation".into(),
        expected_resource: "synthetic-incarnation".into(),
    };
    assert_eq!(registry.effect(&last_receipt, &effect), Ok(Eligibility::SimulationOnly));
    assert_eq!(registry.head(), (full_head.as_str(), full_generation));
}
