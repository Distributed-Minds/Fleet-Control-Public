//! Deterministic advisory preflight for bounded append-only coordination logs.
//!
//! This program NEVER fetches provider state, reserves a write, acquires a lease,
//! publishes an archive, rolls over an issue, or authorizes deletion. Inputs are
//! caller-supplied upper bounds; trust and currentness must be established by
//! a separate authority-bearing adapter under Phase0/#22.
//!
//! Usage:
//!   coordination_capacity <hard-limit> <observed-count> <pending-writes-upper-bound> <reserve>
//!
//! Exit 0: headroom for the declared budget; exit 1: rollover required or full;
//! exit 2: input insufficient or invalid, fail closed. A HEADROOM verdict is
//! advisory ONLY, not permission to publish any coordination transition.

use std::env;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Capacity {
    Headroom { remaining: u64 },
    RolloverRequired { remaining: u64 },
    Exhausted,
}

fn assess(
    hard_limit: u64,
    observed_count: u64,
    pending_writes_upper_bound: u64,
    reserve: u64,
) -> Result<Capacity, &'static str> {
    if hard_limit == 0 {
        return Err("hard limit must be positive");
    }
    if observed_count > hard_limit {
        return Err("observed count exceeds hard limit");
    }
    if pending_writes_upper_bound == 0 {
        return Err("pending writes must have a nonzero upper bound");
    }
    let budget = pending_writes_upper_bound
        .checked_add(reserve)
        .ok_or("pending and reserve budget overflow")?;
    let remaining = hard_limit - observed_count;
    if remaining == 0 {
        Ok(Capacity::Exhausted)
    } else if remaining <= budget {
        Ok(Capacity::RolloverRequired { remaining })
    } else {
        Ok(Capacity::Headroom { remaining })
    }
}

fn parse_count(label: &str, input: &str) -> Result<u64, String> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{label} must be an unsigned decimal integer"));
    }
    input
        .parse::<u64>()
        .map_err(|_| format!("{label} exceeds the supported integer range"))
}

fn preflight(args: &[String]) -> Result<Capacity, String> {
    if args.len() != 4 {
        return Err(
            "usage: coordination_capacity <hard-limit> <observed-count> <pending-writes-upper-bound> <reserve>"
                .to_owned(),
        );
    }
    let hard_limit = parse_count("hard-limit", &args[0])?;
    let observed = parse_count("observed-count", &args[1])?;
    let pending = parse_count("pending-writes-upper-bound", &args[2])?;
    let reserve = parse_count("reserve", &args[3])?;
    assess(hard_limit, observed, pending, reserve).map_err(str::to_owned)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match preflight(&args) {
        Ok(Capacity::Headroom { remaining }) => {
            println!("HEADROOM remaining={remaining}; advisory only, verify provider state and ownership");
            ExitCode::SUCCESS
        }
        Ok(Capacity::RolloverRequired { remaining }) => {
            eprintln!("ROLLOVER_REQUIRED remaining={remaining}; stop new work and follow authorized rollover protocol");
            ExitCode::FAILURE
        }
        Ok(Capacity::Exhausted) => {
            eprintln!("EXHAUSTED remaining=0; coordination mutations cannot proceed here");
            ExitCode::FAILURE
        }
        Err(reason) => {
            eprintln!("CAPACITY_UNKNOWN {reason}; no write authority");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: [&str; 4]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn the_exhausted_2500_comment_incident_fails_closed() {
        assert_eq!(assess(2500, 2500, 1, 0), Ok(Capacity::Exhausted));
        assert_eq!(preflight(&args(["2500", "2500", "1", "10"])), Ok(Capacity::Exhausted));
    }

    #[test]
    fn preflight_warns_before_last_available_mutation_slot() {
        assert_eq!(
            assess(2500, 2490, 8, 2),
            Ok(Capacity::RolloverRequired { remaining: 10 })
        );
        assert_eq!(
            assess(2500, 2489, 8, 2),
            Ok(Capacity::Headroom { remaining: 11 })
        );
        assert_eq!(
            assess(2500, 2499, 1, 0),
            Ok(Capacity::RolloverRequired { remaining: 1 })
        );
    }

    #[test]
    fn unproved_or_out_of_range_inputs_never_claim_headroom() {
        assert!(assess(0, 0, 1, 0).is_err());
        assert!(assess(2500, 2501, 1, 0).is_err());
        assert!(assess(2500, 0, 0, 0).is_err());
        assert!(assess(u64::MAX, 0, u64::MAX, 1).is_err());
        for bad in ["-1", "+1", " 1", "1 ", "1.0", "0x10", "", "18446744073709551616"] {
            let invalid = args(["2500", bad, "2", "5"]);
            assert!(preflight(&invalid).is_err(), "invalid count {bad:?} admitted");
        }
        assert!(preflight(&["2500".to_owned()]).is_err());
        assert!(preflight(&[]).is_err());
    }

    #[test]
    fn additional_consumption_never_restores_headroom() {
        let mut has_become_unsafe = false;
        for observed in 0..=2500 {
            let decision = assess(2500, observed, 6, 16).unwrap();
            if decision != Capacity::Headroom { remaining: 2500 - observed } {
                has_become_unsafe = true;
            }
            if has_become_unsafe {
                assert!(
                    !matches!(decision, Capacity::Headroom { .. }),
                    "headroom reappeared after capacity was exhausted at {observed}"
                );
            }
        }
        assert!(has_become_unsafe);
    }

    #[test]
    fn increased_reserved_budget_cannot_increase_admission() {
        for observed in [0, 2000, 2460, 2479, 2499, 2500] {
            let small = assess(2500, observed, 1, 0).unwrap();
            let large = assess(2500, observed, 10, 20).unwrap();
            if !matches!(small, Capacity::Headroom { .. }) {
                assert!(!matches!(large, Capacity::Headroom { .. }));
            }
        }
    }
}
