//! #91 spec-3 M1: isolation between two *synthetically configured* credential groups.
//!
//! Group labels in this offline fixture are assumptions, NOT evidence that two
//! GitHub accounts, installations or tokens have independent provider quotas.
//! This suite performs no provider calls or irreversible side effects.
use free_energy_provider_effect_m1::credential_budget::{
    Boundary, Error, Priority, Request, Scheduler,
};

const A: &str = "fixture-credential-a";
const B: &str = "fixture-credential-b";
const CURRENT: Boundary = Boundary {
    domain_authority_current: true,
    provider_fence_current: true,
};

fn request(id: &str, group: &str, class: &str, priority: Priority) -> Request {
    Request {
        operation_id: id.into(),
        credential_group: group.into(),
        repository: format!("synthetic-repo-{group}"),
        tenant: "synthetic-tenant".into(),
        target: "issue:91/comments".into(),
        operation_class: class.into(),
        priority,
    }
}

fn scheduler() -> Scheduler {
    let mut s = Scheduler::new();
    s.register_group(A, 3, 1, 8).unwrap();
    s.register_group(B, 2, 0, 8).unwrap();
    s.register_operation_class(A, "comment-create", 3).unwrap();
    s.register_operation_class(B, "comment-create", 2).unwrap();
    s
}

#[test]
fn circuit_refill_and_recovery_reserve_are_isolated_per_group() {
    let mut s = scheduler();
    for id in ["a-normal-0", "a-normal-1", "a-normal-2"] {
        assert_eq!(
            s.enqueue(request(id, A, "comment-create", Priority::Normal)),
            Ok(true)
        );
    }
    for id in ["b-normal-0", "b-normal-1"] {
        assert_eq!(
            s.enqueue(request(id, B, "comment-create", Priority::Normal)),
            Ok(true)
        );
    }

    // A retains one recovery slot after two normal selections.
    for _ in 0..2 {
        assert_eq!(
            s.simulate_candidate(A, CURRENT)
                .unwrap()
                .unwrap()
                .request
                .priority,
            Priority::Normal
        );
    }
    assert_eq!(s.remaining(A), Ok(1));
    assert_eq!(s.pending(A), Ok(1));
    assert!(s.simulate_candidate(A, CURRENT).unwrap().is_none());

    // B has its own assumed capacity; exhausting B does not spend A's reserve.
    for _ in 0..2 {
        assert!(s.simulate_candidate(B, CURRENT).unwrap().is_some());
    }
    assert_eq!(s.remaining(B), Ok(0));
    assert_eq!(s.remaining(A), Ok(1));

    assert_eq!(
        s.enqueue(request(
            "a-recovery",
            A,
            "comment-create",
            Priority::Recovery
        )),
        Ok(true)
    );
    assert_eq!(
        s.simulate_candidate(A, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "a-recovery"
    );
    assert_eq!(s.remaining(A), Ok(0));
    assert_eq!(s.pending(A), Ok(1));

    s.replenish(B).unwrap();
    assert_eq!(s.remaining(B), Ok(2));
    assert_eq!(s.remaining(A), Ok(0));
    s.set_circuit_open(A, true).unwrap();
    s.replenish(A).unwrap();
    assert_eq!(s.remaining(A), Ok(3));
    assert_eq!(s.simulate_candidate(A, CURRENT), Err(Error::CircuitOpen));
    assert_eq!(s.pending(A), Ok(1));

    // An open A circuit neither blocks B nor silently reopens on A's refill.
    s.enqueue(request("b-new", B, "comment-create", Priority::Normal))
        .unwrap();
    assert_eq!(
        s.simulate_candidate(B, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "b-new"
    );
    assert_eq!(s.remaining(B), Ok(1));
    assert_eq!(s.simulate_candidate(A, CURRENT), Err(Error::CircuitOpen));
    s.set_circuit_open(A, false).unwrap();
    assert_eq!(
        s.simulate_candidate(A, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "a-normal-2"
    );
    assert_eq!(s.remaining(A), Ok(2));
    assert_eq!(s.remaining(B), Ok(1));
}

#[test]
fn class_quota_and_replay_identity_remain_distinct_across_groups() {
    let mut s = scheduler();
    s.register_operation_class(A, "ref-update", 1).unwrap();
    s.register_operation_class(B, "ref-update", 1).unwrap();

    for id in ["a-ref-0", "a-ref-1"] {
        s.enqueue(request(id, A, "ref-update", Priority::Normal))
            .unwrap();
    }
    for id in ["b-ref-0", "b-ref-1"] {
        s.enqueue(request(id, B, "ref-update", Priority::Normal))
            .unwrap();
    }
    s.enqueue(request("b-comment", B, "comment-create", Priority::Normal))
        .unwrap();

    assert_eq!(
        s.simulate_candidate(A, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "a-ref-0"
    );
    assert_eq!(s.remaining_class(A, "ref-update"), Ok(0));
    assert_eq!(s.pending(A), Ok(1));
    assert!(s.simulate_candidate(A, CURRENT).unwrap().is_none());

    assert_eq!(
        s.simulate_candidate(B, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "b-ref-0"
    );
    assert_eq!(s.remaining_class(B, "ref-update"), Ok(0));
    // B's exhausted ref class must not block its eligible comment class.
    assert_eq!(
        s.simulate_candidate(B, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "b-comment"
    );
    assert_eq!(s.pending(B), Ok(1));
    assert_eq!(s.remaining_class(A, "ref-update"), Ok(0));

    s.replenish(B).unwrap();
    assert_eq!(
        s.simulate_candidate(B, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "b-ref-1"
    );
    assert_eq!(s.remaining_class(B, "ref-update"), Ok(0));
    assert_eq!(s.remaining_class(A, "ref-update"), Ok(0));
    assert_eq!(s.pending(A), Ok(1));

    // Operation IDs are journal-wide: changing only the group is NOT replay.
    let mut forged = request("a-ref-0", B, "ref-update", Priority::Normal);
    forged.repository = "synthetic-repo-fixture-credential-b".into();
    assert_eq!(s.enqueue(forged), Err(Error::OperationIdentityConflict));
    assert_eq!(s.pending(B), Ok(0));

    s.enqueue(request("b-later", B, "comment-create", Priority::Normal))
        .unwrap();
    let revoked = Boundary {
        domain_authority_current: false,
        provider_fence_current: true,
    };
    assert_eq!(s.simulate_candidate(B, revoked), Err(Error::StaleBoundary));
    assert_eq!(s.pending(B), Ok(1));
    assert_eq!(s.pending(A), Ok(1));
    assert_eq!(
        s.simulate_candidate(B, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .operation_id,
        "b-later"
    );
    assert!(s.simulate_candidate(A, CURRENT).unwrap().is_none());
}
