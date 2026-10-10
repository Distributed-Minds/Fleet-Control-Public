//! Offline classification of GitHub-style provider responses for #91 M1.
//!
//! This is a deterministic policy model, NOT an HTTP adapter, throttle detector,
//! authenticated header parser, timer, retry loop or external-effect receipt.
//! The caller must establish whether rate-limit evidence is trustworthy.
//! No verdict here permits an automatic mutation or proves a remote effect.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderSignal {
    /// No trusted differentiation between permission, quota, and ambiguity.
    Unknown,
    /// Provider-authenticated denial, not a retryable quota condition.
    PermissionDenied,
    /// Provider-authenticated primary exhaustion with a reset-relative wait.
    PrimaryExhaustion { reset_after_seconds: u64 },
    /// Provider-authenticated secondary or abuse-limit throttle.
    SecondaryThrottle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Response<'a> {
    /// None models timeout, connection loss or missing response.
    pub status: Option<u16>,
    /// A separately authenticated HTTP Retry-After field, if present.
    /// HTTP-date form needs trusted wall-clock context and is not parsed here.
    pub retry_after: Option<&'a str>,
    pub signal: ProviderSignal,
    /// Caller-supplied synthetic boundary: never external authority evidence.
    pub current_authority: bool,
    /// Number of previous consecutive throttled outcomes (zero-based).
    pub consecutive_throttles: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    /// Even a successful HTTP acknowledgement needs exact effect readback.
    AcknowledgedUnverified,
    /// Revoked authority or trusted non-retryable permission denial.
    Denied,
    /// Expected head/resource state changed; requires fresh reconciliation.
    StalePrecondition,
    /// A *minimum* cooldown hint; does not reopen a circuit or grant retry.
    Throttled { minimum_wait_seconds: u64 },
    /// Effect may have happened or evidence is contradictory/incomplete.
    ManualHold,
}

const MAX_INTERPRETABLE_DELAY_SECONDS: u64 = 86_400;

/// Parse only bounded decimal delta-seconds. Do not silently convert an
/// unparseable date, overflow, signed value or absurdly large wait to zero.
/// Refusing interpretation is safer than shortening a provider instruction.
fn retry_after_seconds(value: Option<&str>) -> Result<Option<u64>, ()> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_empty() || raw.len() > 18 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    let seconds = raw.parse::<u64>().map_err(|_| ())?;
    if seconds > MAX_INTERPRETABLE_DELAY_SECONDS {
        return Err(());
    }
    Ok(Some(seconds))
}

/// Conservative 60-second exponential floor capped at one hour. Host-owned
/// jitter, real clocks, primary reset authority and circuit state are outside
/// this model: no sleep or circuit reopening happens here.
fn backoff_floor(previous_throttles: u32) -> u64 {
    let exponent = previous_throttles.min(6);
    60_u64.saturating_mul(1_u64 << exponent).min(3_600)
}

fn throttle(response: Response<'_>, primary_reset: Option<u64>) -> Disposition {
    let explicit = match retry_after_seconds(response.retry_after) {
        Ok(seconds) => seconds,
        Err(()) => return Disposition::ManualHold,
    };
    if matches!(primary_reset, Some(0)) {
        // Zero does not establish a meaningful primary quota-reset window.
        return Disposition::ManualHold;
    }
    let minimum = backoff_floor(response.consecutive_throttles)
        .max(explicit.unwrap_or(0))
        .max(primary_reset.unwrap_or(0));
    Disposition::Throttled {
        minimum_wait_seconds: minimum,
    }
}

/// Fail-closed, side-effect-free response classifier. Neither a time window
/// expiring nor an HTTP 2xx acknowledges causal remote state. Authority must
/// be independently revalidated at every real provider effect boundary.
pub fn classify(response: Response<'_>) -> Disposition {
    use Disposition::*;
    use ProviderSignal::*;

    if !response.current_authority {
        return Denied;
    }

    match response.status {
        None => ManualHold,
        Some(200..=299) => {
            if response.signal == Unknown && response.retry_after.is_none() {
                AcknowledgedUnverified
            } else {
                ManualHold
            }
        }
        Some(401) => Denied,
        Some(403) => match response.signal {
            PermissionDenied => Denied,
            PrimaryExhaustion {
                reset_after_seconds,
            } => throttle(response, Some(reset_after_seconds)),
            SecondaryThrottle => throttle(response, None),
            Unknown => ManualHold,
        },
        Some(429) => match response.signal {
            PermissionDenied => ManualHold,
            PrimaryExhaustion {
                reset_after_seconds,
            } => throttle(response, Some(reset_after_seconds)),
            SecondaryThrottle | Unknown => throttle(response, None),
        },
        Some(409 | 412) => StalePrecondition,
        // 5xx and other transport/provider errors might follow a committed
        // write. They are NOT a license for a second attempt.
        Some(_) => ManualHold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn positive_ack_is_not_remote_effect_proof() {
        assert_eq!(
            classify(response(Some(201), ProviderSignal::Unknown)),
            Disposition::AcknowledgedUnverified
        );
        assert_eq!(
            classify(response(Some(204), ProviderSignal::SecondaryThrottle)),
            Disposition::ManualHold
        );
    }

    #[test]
    fn ambiguous_or_lost_ack_must_hold_without_duplicate_create() {
        for status in [None, Some(500), Some(502), Some(503), Some(404)] {
            assert_eq!(
                classify(response(status, ProviderSignal::Unknown)),
                Disposition::ManualHold
            );
        }
    }

    #[test]
    fn distinguish_permission_primary_secondary_and_unknown_403() {
        let cases = [
            (ProviderSignal::PermissionDenied, Disposition::Denied),
            (
                ProviderSignal::PrimaryExhaustion {
                    reset_after_seconds: 120,
                },
                Disposition::Throttled {
                    minimum_wait_seconds: 120,
                },
            ),
            (
                ProviderSignal::SecondaryThrottle,
                Disposition::Throttled {
                    minimum_wait_seconds: 60,
                },
            ),
            (ProviderSignal::Unknown, Disposition::ManualHold),
        ];
        for (signal, expected) in cases {
            assert_eq!(classify(response(Some(403), signal)), expected);
        }
    }

    #[test]
    fn retry_after_is_minimum_not_permission_or_a_success_receipt() {
        let mut r = response(Some(429), ProviderSignal::Unknown);
        r.retry_after = Some("120");
        assert_eq!(
            classify(r),
            Disposition::Throttled {
                minimum_wait_seconds: 120
            }
        );
        r.status = Some(403);
        assert_eq!(classify(r), Disposition::ManualHold);
        r.signal = ProviderSignal::SecondaryThrottle;
        assert_eq!(
            classify(r),
            Disposition::Throttled {
                minimum_wait_seconds: 120
            }
        );
    }

    #[test]
    fn primary_reset_and_explicit_retry_after_use_longer_current_wait() {
        let mut r = response(
            Some(403),
            ProviderSignal::PrimaryExhaustion {
                reset_after_seconds: 180,
            },
        );
        r.retry_after = Some("300");
        assert_eq!(
            classify(r),
            Disposition::Throttled {
                minimum_wait_seconds: 300
            }
        );
        r.retry_after = Some("10");
        assert_eq!(
            classify(r),
            Disposition::Throttled {
                minimum_wait_seconds: 180
            }
        );
    }

    #[test]
    fn missing_or_zero_primary_reset_fails_closed() {
        let r = response(
            Some(403),
            ProviderSignal::PrimaryExhaustion {
                reset_after_seconds: 0,
            },
        );
        assert_eq!(classify(r), Disposition::ManualHold);
    }

    #[test]
    fn malformed_or_uninterpretable_retry_after_never_shortens_wait() {
        let mut r = response(Some(429), ProviderSignal::Unknown);
        for bad in [
            "", "-1", "+1", "1.5", " 60 ", "Sun, 11 Oct 2026 12:00:00 GMT",
            "999999999999999999", "86401",
        ] {
            r.retry_after = Some(bad);
            assert_eq!(classify(r), Disposition::ManualHold, "{bad}");
        }
        r.retry_after = Some("86400");
        assert_eq!(
            classify(r),
            Disposition::Throttled {
                minimum_wait_seconds: 86_400
            }
        );
    }

    #[test]
    fn bounded_exponential_backoff_never_auto_recovers() {
        for (attempt, minimum) in [
            (0, 60), (1, 120), (2, 240), (3, 480),
            (6, 3_600), (u32::MAX, 3_600),
        ] {
            let mut r = response(Some(429), ProviderSignal::Unknown);
            r.consecutive_throttles = attempt;
            assert_eq!(
                classify(r),
                Disposition::Throttled {
                    minimum_wait_seconds: minimum
                }
            );
        }
    }

    #[test]
    fn revoked_boundary_overrides_success_and_rate_hint() {
        for status in [None, Some(200), Some(403), Some(429), Some(503)] {
            let mut r = response(status, ProviderSignal::SecondaryThrottle);
            r.retry_after = Some("180");
            r.current_authority = false;
            assert_eq!(classify(r), Disposition::Denied);
        }
    }

    #[test]
    fn moved_precondition_never_replays_as_a_throttle() {
        for status in [409, 412] {
            assert_eq!(
                classify(response(Some(status), ProviderSignal::Unknown)),
                Disposition::StalePrecondition
            );
        }
    }

    #[test]
    fn contradictory_throttle_and_permission_evidence_holds() {
        assert_eq!(
            classify(response(Some(429), ProviderSignal::PermissionDenied)),
            Disposition::ManualHold
        );
    }
}
