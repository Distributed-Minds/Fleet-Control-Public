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

#[test]
fn invalid_registry_scopes_cannot_admit_a_successor() {
    for field in 0..7 {
        let (_, mut selected, mut transition) = fixture();
        let mut scope = selected.scope.clone();
        match field {
            0 => scope.repository = 0,
            1 => scope.repo_incarnation.clear(),
            2 => scope.installation = " ".into(),
            3 => scope.tenant = "\n".into(),
            4 => scope.namespace.clear(),
            5 => scope.installation = "valid\nspoof".into(),
            6 => scope.repo_incarnation = "\u{0007}".into(),
            _ => unreachable!(),
        }
        selected.scope = scope.clone();
        transition.scope = scope.clone();
        let mut db = Registry::new(scope, "h0".into(), 0, 2, selected.obligations.clone());
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::UnknownLineage),
            "invalid scope field {field}"
        );
        assert_eq!(db.admitted_count(), 0);
    }
}

#[test]
fn blank_registry_head_or_obligation_denies_even_selected_issuer() {
    for invalid in ["", " ", "\n"] {
        let (_, mut selected, mut transition) = fixture();
        selected.head = invalid.into();
        transition.predecessor = invalid.into();
        let mut db = Registry::new(
            selected.scope.clone(),
            invalid.into(),
            0,
            2,
            selected.obligations.clone(),
        );
        assert_eq!(db.admit(&selected, transition), Err(Denied::UnknownLineage));
    }

    let (_, mut selected, transition) = fixture();
    selected.obligations.insert(String::new());
    let mut db = Registry::new(
        selected.scope.clone(),
        "h0".into(),
        0,
        2,
        selected.obligations.clone(),
    );
    assert_eq!(db.admit(&selected, transition), Err(Denied::UnknownLineage));
}

#[test]
fn blank_or_control_bearing_successor_identifiers_never_advance() {
    for invalid in ["", " ", "\n", "valid\u{0007}spoof"] {
        let (mut db, selected, mut transition) = fixture();
        transition.id = invalid.into();
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::InvalidTransition)
        );

        let (mut db, selected, mut transition) = fixture();
        transition.successor = invalid.into();
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::InvalidTransition)
        );

        let (mut db, selected, mut transition) = fixture();
        transition.new_head = invalid.into();
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::InvalidTransition)
        );
        assert_eq!(db.admitted_count(), 0);
    }
}

#[test]
fn unbounded_scope_head_and_obligation_identifiers_cannot_admit() {
    let oversized = "x".repeat(257);
    for field in 0..4 {
        let (_, mut selected, mut transition) = fixture();
        let mut scope = selected.scope.clone();
        match field {
            0 => scope.repo_incarnation = oversized.clone(),
            1 => scope.installation = oversized.clone(),
            2 => scope.tenant = oversized.clone(),
            3 => scope.namespace = oversized.clone(),
            _ => unreachable!(),
        }
        selected.scope = scope.clone();
        transition.scope = scope.clone();
        let mut db = Registry::new(scope, "h0".into(), 0, 2, selected.obligations.clone());
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::UnknownLineage),
            "oversized scope field {field}"
        );
        assert_eq!(db.admitted_count(), 0);
    }

    let (_, mut selected, mut transition) = fixture();
    selected.head = oversized.clone();
    transition.predecessor = oversized.clone();
    let mut db = Registry::new(
        selected.scope.clone(),
        oversized.clone(),
        0,
        2,
        selected.obligations.clone(),
    );
    assert_eq!(db.admit(&selected, transition), Err(Denied::UnknownLineage));
    assert_eq!(db.admitted_count(), 0);

    let (_, mut selected, transition) = fixture();
    selected.obligations.insert(oversized);
    let mut db = Registry::new(
        selected.scope.clone(),
        "h0".into(),
        0,
        2,
        selected.obligations.clone(),
    );
    assert_eq!(db.admit(&selected, transition), Err(Denied::UnknownLineage));
    assert_eq!(db.admitted_count(), 0);
}

#[test]
fn unbounded_transition_identifiers_cannot_create_receipts() {
    let oversized = "x".repeat(257);
    for field in 0..3 {
        let (mut db, selected, mut transition) = fixture();
        match field {
            0 => transition.id = oversized.clone(),
            1 => transition.successor = oversized.clone(),
            2 => transition.new_head = oversized.clone(),
            _ => unreachable!(),
        }
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::InvalidTransition),
            "oversized transition field {field}"
        );
        assert_eq!(db.admitted_count(), 0);
        assert_eq!(db.head(), ("h0", 0));
    }
}

#[test]
fn matching_malformed_resource_identifiers_do_not_authorize_synthetic_effects() {
    let (mut db, selected, transition) = fixture();
    let receipt = db.admit(&selected, transition).unwrap();
    for invalid in ["", " ", "\n", "valid\u{0007}spoof"] {
        let mut effect = request(&selected, &receipt);
        effect.observed_resource = invalid.into();
        effect.expected_resource = invalid.into();
        assert_eq!(db.effect(&receipt, &effect), Err(Denied::WrongResource));
    }
    let mut effect = request(&selected, &receipt);
    effect.observed_resource = "x".repeat(257);
    effect.expected_resource = effect.observed_resource.clone();
    assert_eq!(db.effect(&receipt, &effect), Err(Denied::WrongResource));

    let mut valid = request(&selected, &receipt);
    valid.observed_resource = "resource-incarnation:1".into();
    valid.expected_resource = valid.observed_resource.clone();
    assert_eq!(db.effect(&receipt, &valid), Ok(Eligibility::SimulationOnly));
    assert_eq!(db.admitted_count(), 1);
}

#[test]
fn unresolved_obligations_require_recovery_even_if_caller_waives_flag() {
    let (mut db, selected, transition) = fixture();
    let receipt = db.admit(&selected, transition).unwrap();
    let mut effect = request(&selected, &receipt);
    effect.recovery_required = false;
    effect.recovery_authorized = false;
    assert_eq!(db.effect(&receipt, &effect), Err(Denied::RecoveryHold));

    // Recovery evidence is still only synthetic input to this model.
    effect.recovery_authorized = true;
    assert_eq!(
        db.effect(&receipt, &effect),
        Ok(Eligibility::SimulationOnly)
    );
}

#[test]
fn empty_obligations_preserve_nonrecovery_effect_and_explicit_hold() {
    let (_, mut selected, transition) = fixture();
    selected.obligations.clear();
    let mut db = Registry::new(selected.scope.clone(), "h0".into(), 0, 2, BTreeSet::new());
    let receipt = db.admit(&selected, transition).unwrap();
    let mut effect = request(&selected, &receipt);
    effect.recovery_required = false;
    effect.recovery_authorized = false;
    assert_eq!(
        db.effect(&receipt, &effect),
        Ok(Eligibility::SimulationOnly)
    );

    effect.recovery_required = true;
    assert_eq!(db.effect(&receipt, &effect), Err(Denied::RecoveryHold));
}

#[test]
fn invisible_or_padded_synthetic_scope_and_obligations_fail_closed() {
    // These are machine tokens, never display names or human legal identities.
    for invalid in [
        " leading",
        "trailing ",
        "two words",
        "worker\u{202e}B",
        "work\u{200b}er",
        "worker\u{180f}",
        "caf\u{e9}",
    ] {
        let (_, mut selected, mut transition) = fixture();
        selected.scope.installation = invalid.into();
        transition.scope = selected.scope.clone();
        let mut db = Registry::new(
            selected.scope.clone(),
            "h0".into(),
            0,
            2,
            selected.obligations.clone(),
        );
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::UnknownLineage),
            "invalid installation {invalid:?}"
        );
        assert_eq!(db.admitted_count(), 0);

        let (_, mut selected, transition) = fixture();
        selected.obligations.insert(invalid.into());
        let mut db = Registry::new(
            selected.scope.clone(),
            "h0".into(),
            0,
            2,
            selected.obligations.clone(),
        );
        assert_eq!(
            db.admit(&selected, transition),
            Err(Denied::UnknownLineage),
            "invalid inherited obligation {invalid:?}"
        );
        assert_eq!(db.admitted_count(), 0);
    }
}

#[test]
fn invisible_or_padded_synthetic_transition_ids_fail_closed() {
    for invalid in [" x", "x ", "a b", "a\u{202e}b", "a\u{200b}b", "a\u{180f}b"] {
        for field in 0..3 {
            let (mut db, selected, mut transition) = fixture();
            match field {
                0 => transition.id = invalid.into(),
                1 => transition.successor = invalid.into(),
                2 => transition.new_head = invalid.into(),
                _ => unreachable!(),
            }
            assert_eq!(
                db.admit(&selected, transition),
                Err(Denied::InvalidTransition),
                "field {field}, value {invalid:?}"
            );
            assert_eq!(db.admitted_count(), 0);
            assert_eq!(db.head(), ("h0", 0));
        }
    }
}

#[test]
fn invisible_resource_identity_never_admits_even_when_both_sides_match() {
    let (mut db, selected, transition) = fixture();
    let receipt = db.admit(&selected, transition).unwrap();
    for invalid in [" x", "x ", "a b", "a\u{202e}b", "a\u{200b}b", "a\u{180f}b"] {
        let mut effect = request(&selected, &receipt);
        effect.observed_resource = invalid.into();
        effect.expected_resource = invalid.into();
        assert_eq!(
            db.effect(&receipt, &effect),
            Err(Denied::WrongResource),
            "matching but unsafe resource identity {invalid:?}"
        );
    }
    let mut valid = request(&selected, &receipt);
    valid.observed_resource = "refs/heads/demo:inc-1".into();
    valid.expected_resource = valid.observed_resource.clone();
    assert_eq!(db.effect(&receipt, &valid), Ok(Eligibility::SimulationOnly));
    assert_eq!(db.admitted_count(), 1);
}


#[test]
fn late_ack_is_historical_receipt_not_current_effect_authority() {
    let (mut registry, original_cut, first_transition) = fixture();
    let first = registry.admit(&original_cut, first_transition.clone()).unwrap();

    let mut successor_cut = original_cut.clone();
    successor_cut.head = first.transition.new_head.clone();
    successor_cut.generation = first.generation;
    let mut second_transition = first_transition.clone();
    second_transition.id = "op2".into();
    second_transition.predecessor = first.transition.new_head.clone();
    second_transition.previous_generation = first.generation;
    second_transition.successor = "worker-C".into();
    second_transition.new_head = "h2".into();
    let second = registry.admit(&successor_cut, second_transition).unwrap();

    // A lost ACK can be reconciled later without creating a new admission.
    // It returns a historical receipt; it must never restore its effect right.
    assert_eq!(
        registry.admit(&original_cut, first_transition),
        Ok(first.clone())
    );
    assert_eq!(registry.admitted_count(), 2);
    assert_eq!(registry.head(), ("h2", 2));

    let stale_effect = request(&original_cut, &first);
    assert_eq!(
        registry.effect(&first, &stale_effect),
        Err(Denied::Stale)
    );

    // Even copying the current selection cut onto the old receipt cannot
    // turn its former successor into the currently admitted successor.
    let mut lifted = stale_effect;
    lifted.cut.head = second.transition.new_head.clone();
    lifted.cut.generation = second.generation;
    assert_eq!(registry.effect(&first, &lifted), Err(Denied::Stale));
    lifted.actor = second.transition.successor.clone();
    assert_eq!(registry.effect(&first, &lifted), Err(Denied::Stale));

    let latest = request(&successor_cut, &second);
    assert_eq!(
        registry.effect(&second, &latest),
        Ok(Eligibility::SimulationOnly)
    );

    // Inherited unresolved work still requires recovery permission at the
    // later generation; a late ACK never clears the obligation frontier.
    let mut without_recovery = latest;
    without_recovery.recovery_required = false;
    without_recovery.recovery_authorized = false;
    assert_eq!(
        registry.effect(&second, &without_recovery),
        Err(Denied::RecoveryHold)
    );
}

#[test]
fn cross_generation_conflicting_replays_preserve_the_new_head() {
    let (mut registry, original_cut, first_transition) = fixture();
    let first = registry.admit(&original_cut, first_transition.clone()).unwrap();
    let mut successor_cut = original_cut.clone();
    successor_cut.head = first.transition.new_head.clone();
    successor_cut.generation = first.generation;

    let mut second_transition = first_transition.clone();
    second_transition.id = "op2".into();
    second_transition.predecessor = successor_cut.head.clone();
    second_transition.previous_generation = successor_cut.generation;
    second_transition.new_head = "h2".into();
    second_transition.successor = "worker-C".into();
    let second = registry
        .admit(&successor_cut, second_transition.clone())
        .unwrap();

    // Reusing an earlier operation ID with altered parameters cannot rewrite
    // an acknowledged historical entry, even after a subsequent admission.
    let mut conflicting_old = first_transition.clone();
    conflicting_old.successor = "forged-worker".into();
    assert_eq!(
        registry.admit(&original_cut, conflicting_old),
        Err(Denied::ReplayConflict)
    );

    let mut conflicting_new = second_transition.clone();
    conflicting_new.new_head = "h3".into();
    assert_eq!(
        registry.admit(&successor_cut, conflicting_new),
        Err(Denied::ReplayConflict)
    );

    // A different operation trying to reuse the previous generation is stale.
    let mut stale_cas = second_transition.clone();
    stale_cas.id = "op3".into();
    stale_cas.new_head = "h3".into();
    assert_eq!(
        registry.admit(&successor_cut, stale_cas),
        Err(Denied::Stale)
    );

    assert_eq!(
        registry.admit(&original_cut, first_transition),
        Ok(first)
    );
    assert_eq!(
        registry.admit(&successor_cut, second_transition),
        Ok(second)
    );
    assert_eq!(registry.head(), ("h2", 2));
    assert_eq!(registry.admitted_count(), 2);
}
