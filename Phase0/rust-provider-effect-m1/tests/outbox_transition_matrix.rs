//! Test-only, offline transition-matrix regression for public #91 M1.
//!
//! Enumerates all currently reachable outbox states and all actions against
//! four independent synthetic authority/provider-fence inputs. It proves
//! that denied transitions cannot alter state, attempt provenance, versions
//! or history. This is not a durable journal, live authorization or provider
//! effect. RemoteProven has intentionally no public transition constructor.
use free_energy_provider_effect_m1::outbox::{
    Action, Boundary, Entry, Envelope, Error, Outbox, Status,
};

const OP: &str = "op-91-transition-matrix";

const ALL_ACTIONS: [Action; 9] = [
    Action::Admit,
    Action::Reserve,
    Action::ReleaseReservation,
    Action::BeginAttempt,
    Action::MarkEffectUnknown,
    Action::StartReconciliation,
    Action::Hold,
    Action::Reject,
    Action::Cancel,
];

const REACHABLE_STATES: [Status; 9] = [
    Status::Prepared,
    Status::Queued,
    Status::Reserved,
    Status::Dispatching,
    Status::EffectUnknown,
    Status::Reconciling,
    Status::ManualHold,
    Status::Rejected,
    Status::Cancelled,
];

const BOUNDARIES: [Boundary; 4] = [
    Boundary {
        authority_current: true,
        provider_fence_current: true,
    },
    Boundary {
        authority_current: false,
        provider_fence_current: true,
    },
    Boundary {
        authority_current: true,
        provider_fence_current: false,
    },
    Boundary {
        authority_current: false,
        provider_fence_current: false,
    },
];

fn envelope() -> Envelope {
    Envelope {
        operation_id: OP.into(),
        repository_incarnation: "repo-1".into(),
        target: "issue:91".into(),
        payload_digest: "sha256:offline-matrix".into(),
        assignment_id: "assignment-91".into(),
        authority_generation: 1,
    }
}

fn actions_to(state: Status) -> &'static [Action] {
    match state {
        Status::Prepared => &[],
        Status::Queued => &[Action::Admit],
        Status::Reserved => &[Action::Admit, Action::Reserve],
        Status::Dispatching => &[Action::Admit, Action::Reserve, Action::BeginAttempt],
        Status::EffectUnknown => &[
            Action::Admit,
            Action::Reserve,
            Action::BeginAttempt,
            Action::MarkEffectUnknown,
        ],
        Status::Reconciling => &[
            Action::Admit,
            Action::Reserve,
            Action::BeginAttempt,
            Action::MarkEffectUnknown,
            Action::StartReconciliation,
        ],
        Status::ManualHold => &[
            Action::Admit,
            Action::Reserve,
            Action::BeginAttempt,
            Action::MarkEffectUnknown,
            Action::Hold,
        ],
        Status::Rejected => &[Action::Reject],
        Status::Cancelled => &[Action::Admit, Action::Cancel],
        Status::RemoteProven => panic!("no public model action can prove a remote effect"),
    }
}

fn at_state(state: Status) -> Outbox {
    let mut outbox = Outbox::new();
    assert_eq!(outbox.submit(envelope()), Ok(true));
    for &action in actions_to(state) {
        let previous_version = outbox.entry(OP).unwrap().version();
        outbox
            .apply(OP, previous_version, action, BOUNDARIES[0])
            .unwrap();
    }
    assert_eq!(outbox.entry(OP).unwrap().status(), state);
    outbox
}

fn is_semantically_allowed(state: Status, action: Action) -> bool {
    matches!(
        (state, action),
        (Status::Prepared, Action::Admit | Action::Reject)
            | (Status::Queued, Action::Reserve | Action::Cancel)
            | (
                Status::Reserved,
                Action::ReleaseReservation | Action::Reject | Action::BeginAttempt
            )
            | (Status::Dispatching, Action::MarkEffectUnknown)
            | (Status::EffectUnknown, Action::StartReconciliation | Action::Hold)
            | (Status::Reconciling, Action::Hold)
    )
}

fn snapshot(outbox: &Outbox) -> Entry {
    outbox.entry(OP).unwrap().clone()
}

#[test]
fn all_rejected_transitions_are_atomic_and_preserve_attempt_provenance() {
    let mut checks = 0;
    let mut accepted = 0;
    let mut rejected = 0;

    for state in REACHABLE_STATES {
        for action in ALL_ACTIONS {
            for boundary in BOUNDARIES {
                let mut outbox = at_state(state);
                let before = snapshot(&outbox);
                let version = before.version();
                let semantically_allowed = is_semantically_allowed(state, action);
                let boundary_required = matches!(action, Action::Reserve | Action::BeginAttempt);
                let authorized = boundary.authority_current && boundary.provider_fence_current;
                let should_accept = semantically_allowed && (!boundary_required || authorized);

                match outbox.apply(OP, version, action, boundary) {
                    Ok(after) => {
                        assert!(
                            should_accept,
                            "unexpected allowance: {state:?}, {action:?}, {boundary:?}"
                        );
                        assert_eq!(after.version(), version + 1);
                        assert_eq!(after.history().len(), before.history().len() + 1);
                        assert_eq!(
                            after.attempted(),
                            before.attempted() || action == Action::BeginAttempt
                        );
                        assert_ne!(after.status(), Status::RemoteProven);
                        accepted += 1;
                    }
                    Err(error) => {
                        let expected = if !semantically_allowed {
                            Error::InvalidTransition
                        } else if !authorized && boundary_required {
                            Error::StaleAuthority
                        } else {
                            panic!("a modeled legal transition was denied: {state:?} {action:?}")
                        };
                        assert_eq!(error, expected);
                        assert_eq!(
                            snapshot(&outbox),
                            before,
                            "denied action mutated state: {state:?} {action:?} {boundary:?}"
                        );
                        rejected += 1;
                    }
                }
                checks += 1;
            }
        }
    }

    assert_eq!(checks, 324);
    assert_eq!(accepted, 38);
    assert_eq!(rejected, 286);
}

#[test]
fn stale_version_is_side_effect_free_for_every_reachable_state_and_action() {
    let mut checks = 0;
    for state in REACHABLE_STATES {
        for action in ALL_ACTIONS {
            let mut outbox = at_state(state);
            let before = snapshot(&outbox);
            assert_eq!(
                outbox.apply(OP, before.version() + 1, action, BOUNDARIES[0]),
                Err(Error::StaleVersion),
                "stale CAS did not fail: {state:?} {action:?}"
            );
            assert_eq!(snapshot(&outbox), before);
            checks += 1;
        }
    }
    assert_eq!(checks, 81);
}
