//! Synthetic #91 M1 burst fixture derived from the 2026-10-10 public #186
//! coordination snapshot (226 comments in the observed busiest rolling hour).
//!
//! This is NOT an exact timestamp replay, GitHub rate-limit forecast, live
//! broker, transport call, real-identity admission, or authority to coalesce
//! the currently installed GitHub coordination transitions.
use free_energy_provider_effect_m1::credential_budget::{
    Boundary, Error, Priority, Request, Scheduler,
};

const GROUP: &str = "shared-effective-credential";
const CURRENT: Boundary = Boundary {
    domain_authority_current: true,
    provider_fence_current: true,
};
const OBSERVED_BURST_COUNT: usize = 226;

fn event(index: usize) -> Request {
    let repository = if index % 2 == 0 { "public-one" } else { "public-two" };
    Request {
        operation_id: format!("coord-transition-{index:03}"),
        credential_group: GROUP.into(),
        repository: repository.into(),
        tenant: format!("agent-{:02}", index % 13),
        target: "issue:186/comments".into(),
        operation_class: "comment-create".into(),
        priority: Priority::Normal,
    }
}

fn burst_scheduler() -> Scheduler {
    let mut scheduler = Scheduler::new();
    // Fixture-only budgets: NOT assertions about an effective GitHub quota.
    scheduler.register_group(GROUP, 48, 8, 250).unwrap();
    scheduler.register_operation_class(GROUP, "comment-create", 48).unwrap();
    for index in 0..OBSERVED_BURST_COUNT {
        assert_eq!(scheduler.enqueue(event(index)), Ok(true));
    }
    scheduler
}

#[test]
fn public_burst_preserves_pending_transitions_and_reserved_recovery_budget() {
    let mut scheduler = burst_scheduler();
    assert_eq!(scheduler.pending(GROUP), Ok(OBSERVED_BURST_COUNT));
    assert_eq!(scheduler.remaining(GROUP), Ok(48));

    // The same effective credential is shared by both synthetic repositories.
    // Normal candidates cannot eat the reserved eight slots, even though 226
    // distinct transitions were presented within one synthetic window.
    let mut served = 0;
    while let Some(candidate) = scheduler.simulate_candidate(GROUP, CURRENT).unwrap() {
        assert_eq!(candidate.request.priority, Priority::Normal);
        assert!(candidate.request.repository.starts_with("public-"));
        served += 1;
    }
    assert_eq!(served, 40);
    assert_eq!(scheduler.remaining(GROUP), Ok(8));
    assert_eq!(scheduler.pending(GROUP), Ok(186));

    // Retry of an already admitted operation is a no-op; rewriting its effect
    // identity fails without replacing or silently appending a second request.
    assert_eq!(scheduler.enqueue(event(0)), Ok(false));
    let mut collision = event(0);
    collision.target = "issue:168/comments".into();
    assert_eq!(
        scheduler.enqueue(collision),
        Err(Error::OperationIdentityConflict)
    );
    assert_eq!(scheduler.pending(GROUP), Ok(186));

    let mut recovery = event(999);
    recovery.priority = Priority::Recovery;
    assert_eq!(scheduler.enqueue(recovery), Ok(true));
    assert_eq!(
        scheduler
            .simulate_candidate(GROUP, CURRENT)
            .unwrap()
            .unwrap()
            .request
            .priority,
        Priority::Recovery
    );
    assert_eq!(scheduler.remaining(GROUP), Ok(7));
    assert_eq!(scheduler.pending(GROUP), Ok(186));
    // Candidate selection never performs a provider call; no HTTP/effect proof.
}

#[test]
fn burst_throttle_and_revocation_leave_all_pending_ids_recoverable() {
    let mut scheduler = burst_scheduler();
    for boundary in [
        Boundary {
            domain_authority_current: false,
            provider_fence_current: true,
        },
        Boundary {
            domain_authority_current: true,
            provider_fence_current: false,
        },
    ] {
        assert!(matches!(
            scheduler.simulate_candidate(GROUP, boundary),
            Err(Error::StaleBoundary)
        ));
    }
    assert_eq!(scheduler.pending(GROUP), Ok(OBSERVED_BURST_COUNT));
    assert_eq!(scheduler.remaining(GROUP), Ok(48));

    scheduler.set_circuit_open(GROUP, true).unwrap();
    assert!(matches!(
        scheduler.simulate_candidate(GROUP, CURRENT),
        Err(Error::CircuitOpen)
    ));
    scheduler.replenish(GROUP).unwrap();
    assert!(matches!(
        scheduler.simulate_candidate(GROUP, CURRENT),
        Err(Error::CircuitOpen)
    ));
    assert_eq!(scheduler.pending(GROUP), Ok(OBSERVED_BURST_COUNT));
    scheduler.set_circuit_open(GROUP, false).unwrap();
    assert!(scheduler.simulate_candidate(GROUP, CURRENT).unwrap().is_some());
    assert_eq!(scheduler.pending(GROUP), Ok(OBSERVED_BURST_COUNT - 1));
}
