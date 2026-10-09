//! Fail-closed, offline selection of the writable Phase0 coordination slot.
//!
//! This is an advisory interpretation of caller-supplied observations. It does
//! not authenticate GitHub responses, establish a coherent provider snapshot,
//! acquire ownership, append comments, rotate slots, or authorize any write.
//!
//! Usage:
//! coordination_slots <trusted-author> <id1> <author1> <title1> <body1-path> <id2> <author2> <title2> <body2-path>

use std::env;
use std::fs;
use std::process::ExitCode;

const TITLE: &str = "[fleet-control] coordination";
const MARKER: &str = "FLEET_COORDINATION_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    A,
    B,
}

impl Slot {
    fn label(self) -> &'static str {
        match self {
            Slot::A => "A",
            Slot::B => "B",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Active,
    Draining,
    Standby,
}

#[derive(Clone, Copy)]
struct Observation<'a> {
    issue_number: u64,
    author: &'a str,
    title: &'a str,
    body: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SlotRecord {
    issue_number: u64,
    slot: Slot,
    state: State,
    epoch: u64,
}

fn decimal(value: &str) -> Result<u64, &'static str> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected unsigned decimal integer");
    }
    value.parse::<u64>().map_err(|_| "integer overflow")
}

fn parse_record(observation: Observation<'_>, trusted_author: &str) -> Result<SlotRecord, String> {
    if trusted_author.is_empty() || observation.author != trusted_author {
        return Err("coordination issue author is not trusted".to_owned());
    }
    if observation.title != TITLE {
        return Err("coordination issue title mismatch".to_owned());
    }
    if observation.issue_number == 0 {
        return Err("invalid coordination issue number".to_owned());
    }

    let mut lines = observation.body.lines();
    if lines.next() != Some(MARKER) {
        return Err("missing exact first-line coordination marker".to_owned());
    }

    let slot = match lines
        .next()
        .and_then(|line| line.strip_prefix("COORDINATION_SLOT="))
    {
        Some("A") => Slot::A,
        Some("B") => Slot::B,
        _ => return Err("missing or invalid COORDINATION_SLOT".to_owned()),
    };
    let state = match lines
        .next()
        .and_then(|line| line.strip_prefix("COORDINATION_STATE="))
    {
        Some("ACTIVE") => State::Active,
        Some("DRAINING") => State::Draining,
        Some("STANDBY") => State::Standby,
        _ => return Err("missing or invalid COORDINATION_STATE".to_owned()),
    };
    let epoch = lines
        .next()
        .and_then(|line| line.strip_prefix("COORDINATION_EPOCH="))
        .ok_or("missing COORDINATION_EPOCH")
        .and_then(decimal)
        .map_err(str::to_owned)?;

    // Duplicating authority-bearing fields later in prose is ambiguous. The
    // installed format has exactly one line for each of the three keys.
    if lines.any(|line| {
        [
            "COORDINATION_SLOT=",
            "COORDINATION_STATE=",
            "COORDINATION_EPOCH=",
        ]
        .iter()
        .any(|key| line.starts_with(key))
    }) {
        return Err("duplicate coordination header field".to_owned());
    }

    Ok(SlotRecord {
        issue_number: observation.issue_number,
        slot,
        state,
        epoch,
    })
}

fn select_active(records: [Observation<'_>; 2], trusted: &str) -> Result<SlotRecord, String> {
    let first = parse_record(records[0], trusted)?;
    let second = parse_record(records[1], trusted)?;
    if first.issue_number == second.issue_number {
        return Err("the two coordination observations refer to one issue".to_owned());
    }
    if first.slot == second.slot {
        return Err("expected exactly one slot A and one slot B".to_owned());
    }

    match (first.state, second.state) {
        (State::Active, State::Active) => {
            if first.epoch == second.epoch {
                Err("conflicting ACTIVE slots at equal epoch".to_owned())
            } else if first.epoch > second.epoch {
                Ok(first)
            } else {
                Ok(second)
            }
        }
        (State::Active, _) if first.epoch > second.epoch => Ok(first),
        (_, State::Active) if second.epoch > first.epoch => Ok(second),
        (State::Active, _) | (_, State::Active) => {
            Err("inactive slot has equal or newer epoch than ACTIVE".to_owned())
        }
        _ => Err("no ACTIVE coordination slot".to_owned()),
    }
}

fn execute(args: &[String]) -> Result<SlotRecord, String> {
    if args.len() != 9 {
        return Err(
            "usage: coordination_slots <trusted-author> <id1> <author1> <title1> <body1-path> <id2> <author2> <title2> <body2-path>"
                .to_owned(),
        );
    }
    let first_id = decimal(&args[1]).map_err(str::to_owned)?;
    let second_id = decimal(&args[5]).map_err(str::to_owned)?;
    let first_body =
        fs::read_to_string(&args[4]).map_err(|error| format!("first body unavailable: {error}"))?;
    let second_body = fs::read_to_string(&args[8])
        .map_err(|error| format!("second body unavailable: {error}"))?;
    select_active(
        [
            Observation {
                issue_number: first_id,
                author: &args[2],
                title: &args[3],
                body: &first_body,
            },
            Observation {
                issue_number: second_id,
                author: &args[6],
                title: &args[7],
                body: &second_body,
            },
        ],
        &args[0],
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match execute(&args) {
        Ok(active) => {
            println!(
                "ACTIVE_SLOT={} issue={} epoch={}; ADVISORY ONLY; verify authoritative current provider state and acquire ownership",
                active.slot.label(),
                active.issue_number,
                active.epoch
            );
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("SLOT_UNKNOWN {reason}; no coordination write authority");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=A\nCOORDINATION_STATE=ACTIVE\nCOORDINATION_EPOCH=1\n\nLive issue\n";
    const B: &str = "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=0\n";

    fn observed<'a>(first_body: &'a str, second_body: &'a str) -> [Observation<'a>; 2] {
        [
            Observation {
                issue_number: 168,
                author: "geromet",
                title: TITLE,
                body: first_body,
            },
            Observation {
                issue_number: 186,
                author: "geromet",
                title: TITLE,
                body: second_body,
            },
        ]
    }

    #[test]
    fn installed_active_and_standby_select_first_slot() {
        let active = select_active(observed(A, B), "geromet").unwrap();
        assert_eq!(active.slot, Slot::A);
        assert_eq!(active.issue_number, 168);
        assert_eq!(active.epoch, 1);
    }

    #[test]
    fn two_active_during_flip_choose_higher_epoch_independent_of_input_order() {
        let new_active =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=ACTIVE\nCOORDINATION_EPOCH=2\n";
        let entries = observed(A, new_active);
        let selected = select_active(entries, "geromet").unwrap();
        assert_eq!(selected.slot, Slot::B);
        assert_eq!(
            select_active([entries[1], entries[0]], "geromet").unwrap(),
            selected
        );
    }

    #[test]
    fn contradictory_epochs_and_equal_active_epochs_fail_closed() {
        let equal_active =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=ACTIVE\nCOORDINATION_EPOCH=1\n";
        assert!(select_active(observed(A, equal_active), "geromet").is_err());
        let newer_standby =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=2\n";
        assert!(select_active(observed(A, newer_standby), "geromet").is_err());
        let equal_standby =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=1\n";
        assert!(select_active(observed(A, equal_standby), "geromet").is_err());
    }

    #[test]
    fn no_active_slot_is_not_a_write_target() {
        let inactive =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=A\nCOORDINATION_STATE=DRAINING\nCOORDINATION_EPOCH=1\n";
        assert!(select_active(observed(inactive, B), "geromet").is_err());
    }

    #[test]
    fn untrusted_author_wrong_title_and_reused_issue_id_are_rejected() {
        let mut entries = observed(A, B);
        entries[0].author = "attacker";
        assert!(select_active(entries, "geromet").is_err());
        entries = observed(A, B);
        entries[1].title = "[fleet-control] other";
        assert!(select_active(entries, "geromet").is_err());
        entries = observed(A, B);
        entries[1].issue_number = entries[0].issue_number;
        assert!(select_active(entries, "geromet").is_err());
        assert!(select_active(observed(A, B), "").is_err());
    }

    #[test]
    fn duplicate_slot_and_header_replay_are_rejected() {
        let another_a =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=A\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=0\n";
        assert!(select_active(observed(A, another_a), "geromet").is_err());
        let repeated =
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=0\nCOORDINATION_STATE=ACTIVE\n";
        assert!(select_active(observed(A, repeated), "geromet").is_err());
    }

    #[test]
    fn malformed_headers_and_integer_overflow_fail_closed() {
        for broken in [
            "COORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=0\n",
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=UNKNOWN\nCOORDINATION_EPOCH=0\n",
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=-1\n",
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=18446744073709551616\n",
            "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\n",
        ] {
            assert!(select_active(observed(A, broken), "geromet").is_err());
        }
        assert!(decimal("+1").is_err());
        assert!(decimal("").is_err());
        assert!(decimal("18446744073709551616").is_err());
    }
}
