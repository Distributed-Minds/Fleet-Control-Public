//! Fail-closed, offline two-slot coordination capacity preflight.
//!
//! This advisor does NOT fetch/authenticate issue state, own a slot, reserve
//! capacity, perform a rotation, archive comments, or authorize a write.
//! All counts and epochs are caller observations; a trusted actor must obtain
//! and revalidate the actual provider snapshot and acquire ownership.
//!
//! coordination_rotation <hard-limit> <switch-at> <active-count> <active-epoch>
//!   <other-state> <other-count> <other-epoch> <standby-max>
//!   <pending-writes-upper-bound> <reserve>
//!
//! Other state: STANDBY or DRAINING. A second ACTIVE issue requires a separate
//! trusted highest-epoch election before this advisor can be called safely.

use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtherState {
    Standby,
    Draining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Observation {
    hard_limit: u64,
    switch_at: u64,
    active_count: u64,
    active_epoch: u64,
    other_state: OtherState,
    other_count: u64,
    other_epoch: u64,
    standby_max: u64,
    pending_upper_bound: u64,
    reserve: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Headroom { remaining: u64 },
    RotationReadyModelOnly { remaining: u64 },
    MaintenanceRequired { remaining: u64 },
    Exhausted,
}

/// Capacity arithmetic is conservative, deterministic and free of side effects.
/// A nonzero observed budget is mandatory: without a bounded number of
/// pending writes we cannot prove that the headroom will survive this run.
fn assess(o: Observation) -> Result<Decision, &'static str> {
    if o.hard_limit == 0 {
        return Err("hard limit must be positive");
    }
    // The installed two-slot policy permits proactive switches only in the
    // inclusive 100..=2400 window (GitHub hard limit: 2,500 comments).
    // Accepting a later caller-supplied threshold would advertise headroom
    // after the configured safe rotation window had already expired.
    if !(100..=2400).contains(&o.switch_at) || o.switch_at >= o.hard_limit {
        return Err("switch threshold must be 100..=2400 and precede the hard limit");
    }
    if o.standby_max >= o.hard_limit {
        return Err("standby maximum must be below the hard limit");
    }
    if o.standby_max >= o.switch_at {
        return Err("standby maximum must precede the switch threshold");
    }
    if o.active_count > o.hard_limit || o.other_count > o.hard_limit {
        return Err("observed count exceeds the hard limit");
    }
    if o.active_epoch <= o.other_epoch {
        return Err("active slot epoch is not newer than the other slot");
    }
    if o.active_epoch == u64::MAX {
        return Err("active slot epoch cannot advance for rotation");
    }
    if o.pending_upper_bound == 0 {
        return Err("pending writes require a nonzero upper bound");
    }
    let budget = o
        .pending_upper_bound
        .checked_add(o.reserve)
        .ok_or("pending and reserve budget overflow")?;

    let remaining = o.hard_limit - o.active_count;
    if remaining == 0 {
        return Ok(Decision::Exhausted);
    }

    // An unavailable standby must not block a safe bounded write batch on
    // the still-healthy active slot. Require successor capacity only when
    // the projected batch reaches the proactive switch or physical limit.
    let near_switch = o
        .active_count
        .checked_add(budget)
        .is_none_or(|projected| projected >= o.switch_at);
    if !near_switch && remaining > budget {
        return Ok(Decision::Headroom { remaining });
    }

    // Never advertise a rotation while the other issue is still DRAINING
    // or has not been compacted within its configured standby headroom.
    // Rotation must leave enough capacity for the entire declared write batch
    // in the successor slot, before its own proactive switch threshold.
    // Being below standby_max alone does not establish that headroom.
    let standby_ready = o.other_state == OtherState::Standby
        && o.other_count <= o.standby_max
        && o.other_count
            .checked_add(budget)
            .is_some_and(|projected| projected < o.switch_at);
    if !standby_ready {
        return Ok(Decision::MaintenanceRequired { remaining });
    }

    // The projected batch has reached a switch boundary and the successor
    // is ready. This remains advisory, never provider-side rotation authority.
    Ok(Decision::RotationReadyModelOnly { remaining })
}

fn unsigned(label: &str, input: &str) -> Result<u64, String> {
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{label} must be an unsigned decimal integer"));
    }
    input
        .parse::<u64>()
        .map_err(|_| format!("{label} exceeds the integer range"))
}

fn preflight(args: &[String]) -> Result<Decision, String> {
    if args.len() != 10 {
        return Err(
            "usage: coordination_rotation <hard-limit> <switch-at> <active-count> \
             <active-epoch> <other-state> <other-count> <other-epoch> \
             <standby-max> <pending-upper-bound> <reserve>"
                .to_owned(),
        );
    }
    let other_state = match args[4].as_str() {
        "STANDBY" => OtherState::Standby,
        "DRAINING" => OtherState::Draining,
        "ACTIVE" => {
            return Err("two ACTIVE slots require trusted highest-epoch election first".to_owned());
        }
        _ => return Err("invalid other slot state".to_owned()),
    };
    assess(Observation {
        hard_limit: unsigned("hard-limit", &args[0])?,
        switch_at: unsigned("switch-at", &args[1])?,
        active_count: unsigned("active-count", &args[2])?,
        active_epoch: unsigned("active-epoch", &args[3])?,
        other_state,
        other_count: unsigned("other-count", &args[5])?,
        other_epoch: unsigned("other-epoch", &args[6])?,
        standby_max: unsigned("standby-max", &args[7])?,
        pending_upper_bound: unsigned("pending-upper-bound", &args[8])?,
        reserve: unsigned("reserve", &args[9])?,
    })
    .map_err(str::to_owned)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match preflight(&args) {
        Ok(Decision::Headroom { remaining }) => {
            println!(
                "HEADROOM remaining={remaining}; ADVISORY ONLY; revalidate provider state and ownership"
            );
            ExitCode::SUCCESS
        }
        Ok(Decision::RotationReadyModelOnly { remaining }) => {
            eprintln!(
                "ROTATION_CANDIDATE remaining={remaining}; ADVISORY ONLY; \
                 obtain trusted slot/currentness and rotation authority"
            );
            ExitCode::FAILURE
        }
        Ok(Decision::MaintenanceRequired { remaining }) => {
            eprintln!(
                "STANDBY_NOT_READY remaining={remaining}; do not rotate; \
                 preserve capacity and request authorized archival maintenance"
            );
            ExitCode::FAILURE
        }
        Ok(Decision::Exhausted) => {
            eprintln!("EXHAUSTED remaining=0; no further writes to this slot");
            ExitCode::FAILURE
        }
        Err(reason) => {
            eprintln!("CAPACITY_UNKNOWN {reason}; no coordination write authority");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(active_count: u64) -> Observation {
        Observation {
            hard_limit: 2500,
            switch_at: 2000,
            active_count,
            active_epoch: 3,
            other_state: OtherState::Standby,
            other_count: 300,
            other_epoch: 2,
            standby_max: 500,
            pending_upper_bound: 8,
            reserve: 2,
        }
    }

    fn argv(values: [&str; 10]) -> Vec<String> {
        values.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn installed_defaults_have_headroom_and_proactive_switch_warning() {
        assert_eq!(
            assess(snapshot(1800)),
            Ok(Decision::Headroom { remaining: 700 })
        );
        assert_eq!(
            assess(snapshot(1990)),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );
        assert_eq!(
            assess(snapshot(1991)),
            Ok(Decision::RotationReadyModelOnly { remaining: 509 })
        );
        assert_eq!(assess(snapshot(2500)), Ok(Decision::Exhausted));
    }

    #[test]
    fn standby_must_be_a_bounded_standby_not_a_draining_slot() {
        let mut o = snapshot(1990);
        o.other_count = 501;
        assert_eq!(
            assess(o),
            Ok(Decision::MaintenanceRequired { remaining: 510 })
        );
        o.other_count = 300;
        o.other_state = OtherState::Draining;
        assert_eq!(
            assess(o),
            Ok(Decision::MaintenanceRequired { remaining: 510 })
        );
        o.other_state = OtherState::Standby;
        assert_eq!(
            assess(o),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );
    }

    #[test]
    fn unavailable_standby_does_not_block_safe_active_slot_headroom() {
        let mut o = snapshot(1800);
        o.other_state = OtherState::Draining;
        o.other_count = 2400;
        assert_eq!(assess(o), Ok(Decision::Headroom { remaining: 700 }));

        o.active_count = 1989;
        assert_eq!(assess(o), Ok(Decision::Headroom { remaining: 511 }));

        // The next proposed batch crosses the switch threshold.
        o.active_count = 1990;
        assert_eq!(
            assess(o),
            Ok(Decision::MaintenanceRequired { remaining: 510 })
        );

        o.other_state = OtherState::Standby;
        o.other_count = 300;
        assert_eq!(
            assess(o),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );
    }

    #[test]
    fn stale_epoch_and_invalid_configuration_fail_closed() {
        let mut o = snapshot(1000);
        o.active_epoch = 2;
        assert!(assess(o).is_err());
        o.active_epoch = 1;
        assert!(assess(o).is_err());
        o.active_epoch = 3;
        o.switch_at = 2500;
        assert!(assess(o).is_err());
        o.switch_at = 0;
        assert!(assess(o).is_err());
        o.switch_at = 2000;
        o.standby_max = 2500;
        assert!(assess(o).is_err());
        o.standby_max = 500;
        o.other_count = 2501;
        assert!(assess(o).is_err());
        o.other_count = 300;
        o.pending_upper_bound = 0;
        assert!(assess(o).is_err());
    }

    #[test]
    fn no_budget_overflow_and_no_unsafe_count_admission() {
        let mut o = snapshot(0);
        o.pending_upper_bound = u64::MAX;
        o.reserve = 1;
        assert!(assess(o).is_err());
        o.pending_upper_bound = 8;
        o.reserve = 2;
        o.active_count = 2501;
        assert!(assess(o).is_err());
        o.active_count = 2500;
        assert_eq!(assess(o), Ok(Decision::Exhausted));
    }

    #[test]
    fn malformed_cli_and_two_active_handoff_do_not_claim_rotation() {
        let valid = [
            "2500", "2000", "1990", "3", "STANDBY", "300", "2", "500", "8", "2",
        ];
        assert_eq!(
            preflight(&argv(valid)),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );
        for wrong_state in ["ACTIVE", "unknown", "standby", ""] {
            let mut input = valid;
            input[4] = wrong_state;
            assert!(preflight(&argv(input)).is_err());
        }
        for invalid in ["-1", "+1", " 2", "2 ", "1.0", "", "18446744073709551616"] {
            let mut input = valid;
            input[6] = invalid;
            assert!(preflight(&argv(input)).is_err());
        }
        assert!(preflight(&[]).is_err());
    }

    #[test]
    fn installed_switch_window_rejects_unsafe_caller_thresholds() {
        // The provider limit is not a license to defer the configured
        // switch beyond the installed policy's 2,400-comment maximum.
        for threshold in [0, 1, 99, 2401, 2499, u64::MAX] {
            let mut o = snapshot(0);
            o.switch_at = threshold;
            o.standby_max = 0;
            o.other_count = 0;
            assert!(
                assess(o).is_err(),
                "out-of-policy switch threshold {threshold} was admitted"
            );
        }

        // Both inclusive policy boundaries remain valid with a suitably
        // empty successor slot and a bounded write batch.
        for threshold in [100, 2400] {
            let mut o = snapshot(0);
            o.switch_at = threshold;
            o.standby_max = 0;
            o.other_count = 0;
            o.pending_upper_bound = 1;
            o.reserve = 0;
            assert_eq!(assess(o), Ok(Decision::Headroom { remaining: 2500 }));
        }

        // The provider's observed hard limit must still exceed the switch.
        let mut o = snapshot(0);
        o.hard_limit = 100;
        o.switch_at = 100;
        o.standby_max = 0;
        o.other_count = 0;
        assert!(assess(o).is_err());
    }

    #[test]
    fn remaining_capacity_never_recovers_when_active_count_rises() {
        let mut crossed = false;
        for count in 0..=2500 {
            let result = assess(snapshot(count)).unwrap();
            if !matches!(result, Decision::Headroom { .. }) {
                crossed = true;
            }
            if crossed {
                assert!(!matches!(result, Decision::Headroom { .. }));
            }
        }
    }

    #[test]
    fn rotation_requires_successor_epoch_and_budget_safe_standby() {
        let mut o = snapshot(1990);

        // A standby inside a permissive standby_max may already be unable
        // to accept the pending batch without reaching the switch threshold.
        o.standby_max = 1999;
        o.other_count = 1990;
        assert_eq!(
            assess(o),
            Ok(Decision::MaintenanceRequired { remaining: 510 })
        );
        o.other_count = 1989;
        assert_eq!(
            assess(o),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );

        // The same invariant applies when the configured standby is small
        // but the declared write budget is unusually large.
        o.other_count = 300;
        o.pending_upper_bound = 1700;
        o.reserve = 0;
        assert_eq!(
            assess(o),
            Ok(Decision::MaintenanceRequired { remaining: 510 })
        );
        o.pending_upper_bound = 1699;
        assert_eq!(
            assess(o),
            Ok(Decision::RotationReadyModelOnly { remaining: 510 })
        );

        // Rotating into a slot above the switch threshold cannot be safe
        // merely because the provider's hard limit is larger.
        o.standby_max = 2000;
        assert!(assess(o).is_err());

        // No strictly newer epoch can be issued after the maximum value.
        o.standby_max = 1999;
        o.active_epoch = u64::MAX;
        assert!(assess(o).is_err());
    }

    #[test]
    fn larger_unacknowledged_write_budget_never_restores_headroom() {
        for count in [0, 1980, 1989, 1990, 2490, 2499, 2500] {
            let baseline = assess(snapshot(count)).unwrap();
            let mut larger = snapshot(count);
            larger.pending_upper_bound = 20;
            larger.reserve = 15;
            let grown = assess(larger).unwrap();
            if !matches!(baseline, Decision::Headroom { .. }) {
                assert!(!matches!(grown, Decision::Headroom { .. }));
            }
        }
    }
}
