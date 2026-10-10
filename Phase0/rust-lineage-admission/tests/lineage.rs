//! LA6 admission and effect-cut regressions; no provider actions are exercised.
use free_energy_lineage_admission::{
    Denied, EffectInput, Eligibility, Receipt, Registry, Scope, Selection, Transition,
};
use std::collections::BTreeSet;

fn fixture() -> (Registry, Selection, Transition) {
    let scope = Scope {
        repository: 1360059617,
        repo_incarnation: "repo-A".into(),
        installation: "install-A".into(),
        tenant: "tenant-A".into(),
        namespace: "probe-A".into(),
    };
    let obligations = BTreeSet::from(["original-probe".into()]);
    let s = Selection {
        scope: scope.clone(),
        head: "h0".into(),
        generation: 0,
        policy: 2,
        obligations: obligations.clone(),
        selected_issuer: true,
        unique_current: true,
        frontier_complete: true,
        forked: false,
        revoked: false,
    };
    let t = Transition {
        id: "op1".into(),
        scope: scope.clone(),
        predecessor: "h0".into(),
        previous_generation: 0,
        policy: 2,
        successor: "worker-B".into(),
        new_head: "h1".into(),
        admitted_by_issuer: true,
    };
    (Registry::new(scope, "h0".into(), 0, 2, obligations), s, t)
}
fn request(s: &Selection, r: &Receipt) -> EffectInput {
    let mut cut = s.clone();
    cut.head = r.transition.new_head.clone();
    cut.generation = r.generation;
    EffectInput {
        cut,
        operation_id: r.transition.id.clone(),
        actor: r.transition.successor.clone(),
        authorized_action: true,
        recovery_required: true,
        recovery_authorized: true,
        observed_resource: "inc-1".into(),
        expected_resource: "inc-1".into(),
    }
}
#[test]
fn la6_10_positive_simulation_only() {
    let (mut db, s, t) = fixture();
    let receipt = db.admit(&s, t).unwrap();
    assert_eq!(db.head(), ("h1", 1));
    assert_eq!(receipt.obligations, s.obligations);
    assert_eq!(
        db.effect(&receipt, &request(&s, &receipt)),
        Ok(Eligibility::SimulationOnly)
    );
}
#[test]
fn la6_01_self_declared_issuer_is_not_selected() {
    let (mut db, mut s, t) = fixture();
    s.selected_issuer = false;
    assert_eq!(db.admit(&s, t), Err(Denied::UnknownLineage));
}
#[test]
fn la6_02_competing_successors_only_one_wins_in_both_orders() {
    for reversed in [false, true] {
        let (mut db, s, a) = fixture();
        let mut b = a.clone();
        b.id = "op2".into();
        b.successor = "worker-C".into();
        b.new_head = "h2".into();
        let (first, second) = if reversed { (b, a) } else { (a, b) };
        let receipt = db.admit(&s, first).unwrap();
        assert_eq!(db.admit(&s, second), Err(Denied::Stale));
        assert_eq!(db.head(), (receipt.transition.new_head.as_str(), 1));
        assert_eq!(db.admitted_count(), 1);
    }
}
#[test]
fn la6_03_cloned_forks_and_unselected_heads_denied() {
    for which in [0, 1] {
        let (mut db, mut s, t) = fixture();
        if which == 0 {
            s.forked = true;
        } else {
            s.unique_current = false;
        }
        assert_eq!(db.admit(&s, t), Err(Denied::UnknownLineage));
    }
}
#[test]
fn la6_04_revoked_actor_cannot_act() {
    let (mut db, s, t) = fixture();
    let receipt = db.admit(&s, t).unwrap();
    let mut effect = request(&s, &receipt);
    effect.cut.revoked = true;
    assert_eq!(db.effect(&receipt, &effect), Err(Denied::UnknownLineage));
}
#[test]
fn la6_05_permission_does_not_mint_lineage() {
    let (mut db, mut s, t) = fixture();
    s.unique_current = false;
    assert_eq!(db.admit(&s, t), Err(Denied::UnknownLineage));
    assert_eq!(db.admitted_count(), 0);
}
#[test]
fn la6_06_wrong_resource_or_missing_recovery_denied() {
    let (mut db, s, t) = fixture();
    let receipt = db.admit(&s, t).unwrap();
    let mut e = request(&s, &receipt);
    e.observed_resource = "different-incarnation".into();
    assert_eq!(db.effect(&receipt, &e), Err(Denied::WrongResource));
    e.observed_resource = "inc-1".into();
    e.recovery_authorized = false;
    assert_eq!(db.effect(&receipt, &e), Err(Denied::RecoveryHold));
    e.recovery_authorized = true;
    e.authorized_action = false;
    assert_eq!(db.effect(&receipt, &e), Err(Denied::NoActionAuthority));
}
#[test]
fn la6_07_effect_time_policy_or_head_drift_denied() {
    let (mut db, s, t) = fixture();
    let receipt = db.admit(&s, t).unwrap();
    let mut e = request(&s, &receipt);
    e.cut.policy += 1;
    assert_eq!(db.effect(&receipt, &e), Err(Denied::Stale));
    e.cut.policy -= 1;
    e.cut.head = "old-head".into();
    assert_eq!(db.effect(&receipt, &e), Err(Denied::Stale));
}
#[test]
fn la6_08_missing_prior_obligation_or_listing_denied() {
    let (mut db, mut s, t) = fixture();
    s.obligations.clear();
    assert_eq!(db.admit(&s, t.clone()), Err(Denied::UnknownLineage));
    s.obligations.insert("original-probe".into());
    s.frontier_complete = false;
    assert_eq!(db.admit(&s, t), Err(Denied::UnknownLineage));
}
#[test]
fn la6_09_reused_name_across_installations_is_not_identity() {
    let (mut db, mut s, t) = fixture();
    s.scope.installation = "new-installation".into();
    assert_eq!(db.admit(&s, t), Err(Denied::Scope));
}
#[test]
fn exact_lost_ack_replay_is_idempotent_but_changed_retry_denied() {
    let (mut db, s, t) = fixture();
    let receipt = db.admit(&s, t.clone()).unwrap();
    assert_eq!(db.admit(&s, t.clone()), Ok(receipt));
    assert_eq!(db.admitted_count(), 1);
    let mut altered = t;
    altered.successor = "attacker".into();
    assert_eq!(db.admit(&s, altered), Err(Denied::ReplayConflict));
}
#[test]
fn old_head_restore_or_issuer_denial_does_not_advance() {
    let (mut db, s, mut t) = fixture();
    t.admitted_by_issuer = false;
    assert_eq!(db.admit(&s, t.clone()), Err(Denied::InvalidTransition));
    t.admitted_by_issuer = true;
    let receipt = db.admit(&s, t).unwrap();
    let mut second = receipt.transition;
    second.id = "next".into();
    second.predecessor = "h1".into();
    second.previous_generation = 1;
    second.new_head = "h0".into();
    let mut current = s;
    current.head = "h1".into();
    current.generation = 1;
    assert_eq!(db.admit(&current, second), Err(Denied::ReplayConflict));
}
