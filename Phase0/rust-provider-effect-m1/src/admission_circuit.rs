//! Offline M1 policy bridge between a synthetic provider response and a
//! credential-group admission circuit for issue #91, spec 3.
//!
//! This is deliberately not an HTTP adapter, a trusted source of rate-limit
//! evidence, an authorization decision, a timer or an automatic retry path.
//! In a real broker the adapter must independently authenticate and bind each
//! response to the exact effective credential group and operation attempt.
use crate::credential_budget::{Error, Scheduler};
use crate::provider_response::{classify, Disposition, Response};

/// Record one externally authenticated synthetic response against its already
/// configured effective-credential budget. Ambiguous, forbidden and throttled
/// responses close the shared circuit; neither a fresh 2xx nor a refill opens
/// it. An exact current authority check is still needed to select any task.
///
/// A StalePrecondition is operation-scoped and does not automatically disable
/// unrelated work with the same credential; that operation must be reconciled.
/// This returns no effect success receipt and never sends or retries a request.
pub fn record_response(
    scheduler: &mut Scheduler,
    credential_group: &str,
    response: Response<'_>,
) -> Result<Disposition, Error> {
    // Refuse an unregistered group for *all* outcomes, even HTTP 2xx.
    // Otherwise a wrong-group positive response could be treated as recorded.
    scheduler.remaining(credential_group)?;
    let decision = classify(response);
    if matches!(
        decision,
        Disposition::Denied | Disposition::ManualHold | Disposition::Throttled { .. }
    ) {
        scheduler.set_circuit_open(credential_group, true)?;
    }
    Ok(decision)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential_budget::{Boundary, Priority, Request};
    use crate::provider_response::ProviderSignal;

    const CURRENT: Boundary = Boundary {
        domain_authority_current: true,
        provider_fence_current: true,
    };
    const STALE: Boundary = Boundary {
        domain_authority_current: false,
        provider_fence_current: true,
    };

    fn request(id: &str, repository: &str) -> Request {
        Request {
            operation_id: id.to_owned(),
            credential_group: "effective-credential-1".to_owned(),
            repository: repository.to_owned(),
            tenant: repository.to_owned(),
            target: "issue:91/comments".to_owned(),
            operation_class: "comment-create".to_owned(),
            priority: Priority::Normal,
        }
    }

    fn scheduler() -> Scheduler {
        let mut scheduler = Scheduler::new();
        scheduler
            .register_group("effective-credential-1", 4, 1, 8)
            .unwrap();
        scheduler
            .register_operation_class("effective-credential-1", "comment-create", 4)
            .unwrap();
        scheduler.enqueue(request("a", "repo-a")).unwrap();
        scheduler.enqueue(request("b", "repo-b")).unwrap();
        scheduler
    }

    fn response(status: Option<u16>, signal: ProviderSignal) -> Response<'static> {
        Response {
            status,
            retry_after: None,
            signal,
            current_authority: true,
            consecutive_throttles: 0,
        }
    }

    #[test]
    fn secondary_throttle_closes_shared_group_without_spending_or_dropping_work() {
        let mut s = scheduler();
        let mut r = response(Some(429), ProviderSignal::SecondaryThrottle);
        r.retry_after = Some("300");
        r.consecutive_throttles = 2;
        assert_eq!(
            record_response(&mut s, "effective-credential-1", r),
            Ok(Disposition::Throttled {
                minimum_wait_seconds: 300
            })
        );
        assert!(matches!(
            s.simulate_candidate("effective-credential-1", CURRENT),
            Err(Error::CircuitOpen)
        ));
        assert_eq!(s.pending("effective-credential-1"), Ok(2));
        assert_eq!(s.remaining("effective-credential-1"), Ok(4));
        // An offline window refill cannot revoke a provider/authority hold.
        s.replenish("effective-credential-1").unwrap();
        assert!(matches!(
            s.simulate_candidate("effective-credential-1", CURRENT),
            Err(Error::CircuitOpen)
        ));
        s.set_circuit_open("effective-credential-1", false).unwrap();
        let selected = s
            .simulate_candidate("effective-credential-1", CURRENT)
            .unwrap();
        assert!(selected.is_some());
        assert_eq!(s.remaining("effective-credential-1"), Ok(3));
        assert_eq!(s.pending("effective-credential-1"), Ok(1));
    }

    #[test]
    fn ambiguous_or_lost_provider_result_holds_all_same_group_tenants() {
        for (status, signal) in [
            (None, ProviderSignal::Unknown),
            (Some(403), ProviderSignal::Unknown),
            (Some(503), ProviderSignal::SecondaryThrottle),
        ] {
            let mut s = scheduler();
            assert_eq!(
                record_response(&mut s, "effective-credential-1", response(status, signal)),
                Ok(Disposition::ManualHold)
            );
            assert_eq!(
                s.simulate_candidate("effective-credential-1", CURRENT)
                    .err(),
                Some(Error::CircuitOpen)
            );
            assert_eq!(s.pending("effective-credential-1"), Ok(2));
            assert_eq!(s.remaining("effective-credential-1"), Ok(4));
        }
    }

    #[test]
    fn permission_denial_and_revocation_never_become_quota_recovery() {
        for (status, current_authority) in [(Some(401), true), (Some(200), false)] {
            let mut s = scheduler();
            let mut r = response(status, ProviderSignal::Unknown);
            r.current_authority = current_authority;
            assert_eq!(
                record_response(&mut s, "effective-credential-1", r),
                Ok(Disposition::Denied)
            );
            s.replenish("effective-credential-1").unwrap();
            assert_eq!(
                s.simulate_candidate("effective-credential-1", CURRENT)
                    .err(),
                Some(Error::CircuitOpen)
            );
            // Explicit circuit reopening is still not independent authority.
            s.set_circuit_open("effective-credential-1", false).unwrap();
            assert_eq!(
                s.simulate_candidate("effective-credential-1", STALE).err(),
                Some(Error::StaleBoundary)
            );
            assert_eq!(s.pending("effective-credential-1"), Ok(2));
        }
    }

    #[test]
    fn an_unverified_success_does_not_clear_an_existing_circuit() {
        let mut s = scheduler();
        s.set_circuit_open("effective-credential-1", true).unwrap();
        assert_eq!(
            record_response(
                &mut s,
                "effective-credential-1",
                response(Some(201), ProviderSignal::Unknown)
            ),
            Ok(Disposition::AcknowledgedUnverified)
        );
        assert_eq!(
            s.simulate_candidate("effective-credential-1", CURRENT)
                .err(),
            Some(Error::CircuitOpen)
        );
    }

    #[test]
    fn moved_precondition_is_operation_scoped_not_a_credential_group_throttle() {
        let mut s = scheduler();
        for status in [409, 412] {
            assert_eq!(
                record_response(
                    &mut s,
                    "effective-credential-1",
                    response(Some(status), ProviderSignal::Unknown)
                ),
                Ok(Disposition::StalePrecondition)
            );
        }
        // This only selects a synthetic candidate; the stale target still
        // requires exact-state reconciliation before any actual provider effect.
        assert!(s
            .simulate_candidate("effective-credential-1", CURRENT)
            .unwrap()
            .is_some());
    }

    #[test]
    fn wrong_group_cannot_accept_even_positive_ack_or_block_another_group() {
        let mut s = scheduler();
        for status in [Some(201), Some(429), None] {
            assert_eq!(
                record_response(
                    &mut s,
                    "unknown-group",
                    response(status, ProviderSignal::Unknown)
                ),
                Err(Error::UnknownGroup)
            );
        }
        assert_eq!(s.pending("effective-credential-1"), Ok(2));
        assert!(s
            .simulate_candidate("effective-credential-1", CURRENT)
            .unwrap()
            .is_some());
    }
}
