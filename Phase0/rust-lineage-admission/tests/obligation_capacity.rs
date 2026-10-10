//! P0 #11 spec-6 synthetic-only regression: one retained receipt clones
//! inherited obligations, so bounding receipt count alone cannot bound memory.
use free_energy_lineage_admission::{
    Denied, Registry, Scope, Selection, Transition, MAX_SYNTHETIC_LINEAGE_OBLIGATIONS,
};
use std::collections::BTreeSet;

fn fixture(obligation_count: usize) -> (Registry, Selection, Transition) {
    let scope = Scope {
        repository: 1360059617,
        repo_incarnation: "synthetic-repo".into(),
        installation: "synthetic-install".into(),
        tenant: "synthetic-tenant".into(),
        namespace: "synthetic-lineage".into(),
    };
    let obligations: BTreeSet<String> = (0..obligation_count)
        .map(|index| format!("unresolved-{index}"))
        .collect();
    let selection = Selection {
        scope: scope.clone(),
        head: "head-0".into(),
        generation: 0,
        policy: 1,
        obligations: obligations.clone(),
        selected_issuer: true,
        unique_current: true,
        frontier_complete: true,
        forked: false,
        revoked: false,
    };
    let registry = Registry::new(scope.clone(), "head-0".into(), 0, 1, obligations);
    let transition = Transition {
        id: "synthetic-operation".into(),
        scope,
        predecessor: "head-0".into(),
        previous_generation: 0,
        policy: 1,
        successor: "synthetic-successor".into(),
        new_head: "head-1".into(),
        admitted_by_issuer: true,
    };
    (registry, selection, transition)
}

#[test]
fn excess_inherited_obligations_cannot_multiply_into_receipts() {
    let (mut registry, selection, transition) = fixture(MAX_SYNTHETIC_LINEAGE_OBLIGATIONS + 1);
    assert_eq!(
        registry.admit(&selection, transition.clone()),
        Err(Denied::CapacityExhausted)
    );
    assert_eq!(
        registry.admit(&selection, transition),
        Err(Denied::CapacityExhausted)
    );
    assert_eq!(registry.admitted_count(), 0);
    assert_eq!(registry.head(), ("head-0", 0));
}

#[test]
fn exactly_at_limit_admits_and_exact_replay_preserves_one_receipt() {
    let (mut registry, selection, transition) = fixture(MAX_SYNTHETIC_LINEAGE_OBLIGATIONS);
    let receipt = registry.admit(&selection, transition.clone()).unwrap();
    assert_eq!(receipt.obligations.len(), MAX_SYNTHETIC_LINEAGE_OBLIGATIONS);
    assert_eq!(registry.admit(&selection, transition), Ok(receipt));
    assert_eq!(registry.admitted_count(), 1);
    assert_eq!(registry.head(), ("head-1", 1));
}

#[test]
fn oversized_caller_selection_is_not_admitted_to_bounded_registry() {
    let (mut registry, mut selection, transition) = fixture(MAX_SYNTHETIC_LINEAGE_OBLIGATIONS);
    selection.obligations.insert("forged-extra".into());
    assert_eq!(
        registry.admit(&selection, transition),
        Err(Denied::CapacityExhausted)
    );
    assert_eq!(registry.admitted_count(), 0);
    assert_eq!(registry.head(), ("head-0", 0));
}
