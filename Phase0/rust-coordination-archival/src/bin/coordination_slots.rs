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
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::unix::fs::OpenOptionsExt;
use std::process::ExitCode;

const TITLE: &str = "[fleet-control] coordination";
const MARKER: &str = "FLEET_COORDINATION_V1";
// Advisory inputs can come from untrusted downloads or local paths. Do not
// consume an unbounded file or special device before validating its contents.
const MAX_BODY_BYTES: u64 = 1_048_576;

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

// The installed COORDINATION_TRUSTED_AUTHORS setting is a comma-separated
// list of GitHub logins, not a single login. Reject malformed list entries
// in their entirety instead of accepting a valid prefix from bad policy.
fn author_is_trusted(author: &str, configured: &str) -> bool {
    let mut matched = false;
    for login in configured.split(',') {
        if login.is_empty()
            || !login
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return false;
        }
        if login.eq_ignore_ascii_case(author) {
            matched = true;
        }
    }
    matched
}

fn parse_record(observation: Observation<'_>, trusted_author: &str) -> Result<SlotRecord, String> {
    if !author_is_trusted(observation.author, trusted_author) {
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
        (State::Active, State::Active) => match first.epoch.cmp(&second.epoch) {
            std::cmp::Ordering::Equal => Err("conflicting ACTIVE slots at equal epoch".to_owned()),
            std::cmp::Ordering::Greater => Ok(first),
            std::cmp::Ordering::Less => Ok(second),
        },
        (State::Active, _) if first.epoch > second.epoch => Ok(first),
        (_, State::Active) if second.epoch > first.epoch => Ok(second),
        (State::Active, _) | (_, State::Active) => {
            Err("inactive slot has equal or newer epoch than ACTIVE".to_owned())
        }
        _ => Err("no ACTIVE coordination slot".to_owned()),
    }
}

fn read_body_with_observed_metadata(path: &str, metadata: &fs::Metadata) -> Result<String, String> {
    if !metadata.file_type().is_file() {
        return Err("coordination body must be a regular, non-symlink file".to_owned());
    }
    if metadata.len() > MAX_BODY_BYTES {
        return Err("coordination body exceeds bounded input size".to_owned());
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        // Linux O_NONBLOCK stops a swapped-in FIFO from hanging the reader;
        // O_NOFOLLOW refuses symlink substitution at open time. The numeric
        // constants are Linux-specific, so do not apply them on other OSes.
        const O_NONBLOCK: i32 = 0o4000;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options.open(path).map_err(|e| e.to_string())?;
    let opened = file.metadata().map_err(|e| e.to_string())?;
    if !opened.file_type().is_file() {
        return Err("opened coordination body is not a regular file".to_owned());
    }
    #[cfg(unix)]
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err("coordination body changed between metadata check and open".to_owned());
    }
    if opened.len() > MAX_BODY_BYTES {
        return Err("coordination body exceeds bounded input size".to_owned());
    }

    // Check the open descriptor, not just the pathname; cap actual reads in
    // case another writer appends after the size check.
    let mut body = String::new();
    file.take(MAX_BODY_BYTES + 1)
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() as u64 > MAX_BODY_BYTES {
        return Err("coordination body exceeds bounded input size".to_owned());
    }
    Ok(body)
}

fn read_body(path: &str) -> Result<String, String> {
    // A leaf-only lstat follows symlinked parent directories, even when the
    // leaf itself is a real file. Do not admit such advisory input paths.
    // This is a preflight guard, not an atomic directory-handle sandbox.
    let mut examined = std::path::PathBuf::new();
    if let Some(parent) = std::path::Path::new(path).parent() {
        for component in parent.components() {
            examined.push(component.as_os_str());
            let kind = fs::symlink_metadata(&examined)
                .map_err(|error| format!("cannot inspect {examined:?}: {error}"))?;
            if kind.file_type().is_symlink() {
                return Err("coordination body path contains symlink component".to_owned());
            }
        }
    }
    let observed = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    read_body_with_observed_metadata(path, &observed)
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
        read_body(&args[4]).map_err(|error| format!("first body unavailable: {error}"))?;
    let second_body =
        read_body(&args[8]).map_err(|error| format!("second body unavailable: {error}"))?;
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
    // Untrusted Unix argv may contain non-UTF-8 bytes. env::args() panics,
    // bypassing the CLI's explicit non-authoritative SLOT_UNKNOWN disposition.
    let args: Vec<String> = match env::args_os()
        .skip(1)
        .map(|arg| arg.into_string())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(_) => {
            eprintln!(
                "SLOT_UNKNOWN arguments must be valid UTF-8; no coordination write authority"
            );
            return ExitCode::FAILURE;
        }
    };
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
    fn configured_trusted_author_list_accepts_either_author() {
        let mut entries = observed(A, B);
        entries[1].author = "alice";
        assert_eq!(
            select_active(entries, "geromet,alice").unwrap().slot,
            Slot::A
        );
        entries[0].author = "ALICE";
        assert_eq!(
            select_active(entries, "geromet,alice").unwrap().slot,
            Slot::A
        );
        assert!(select_active(entries, "geromet").is_err());
    }

    #[test]
    fn malformed_trusted_author_lists_and_substrings_fail_closed() {
        for policy in [
            "",
            "geromet,",
            ",geromet",
            "geromet,,alice",
            "geromet, alice",
            "geromet,alice\\n",
            "geromet|alice",
        ] {
            assert!(
                select_active(observed(A, B), policy).is_err(),
                "malformed trusted-author configuration accepted: {policy:?}"
            );
        }
        let mut entries = observed(A, B);
        entries[1].author = "alice-malicious";
        assert!(select_active(entries, "geromet,alice").is_err());
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

    #[test]
    fn bounded_body_reader_accepts_real_files_and_rejects_oversize_or_special_inputs() {
        let scratch = std::env::temp_dir().join(format!(
            "free-energy-coordination-slots-bounded-{}",
            std::process::id()
        ));
        fs::create_dir_all(&scratch).expect("create isolated test directory");
        let good = scratch.join("valid-issue-body");
        fs::write(&good, A).expect("write valid coordination body");
        assert_eq!(read_body(good.to_str().unwrap()).unwrap(), A);

        let too_large = scratch.join("oversized-issue-body");
        fs::write(&too_large, vec![b'X'; MAX_BODY_BYTES as usize + 1])
            .expect("write oversized file");
        assert!(read_body(too_large.to_str().unwrap())
            .unwrap_err()
            .contains("bounded input size"));

        let directory = scratch.join("a-directory-not-an-issue-body");
        fs::create_dir(&directory).expect("create non-regular path");
        assert!(read_body(directory.to_str().unwrap())
            .unwrap_err()
            .contains("regular, non-symlink"));

        #[cfg(unix)]
        {
            let alias = scratch.join("symlink-to-issue-body");
            std::os::unix::fs::symlink(&good, &alias).expect("create symlink");
            assert!(read_body(alias.to_str().unwrap())
                .unwrap_err()
                .contains("regular, non-symlink"));
        }
        fs::remove_dir_all(&scratch).expect("remove isolated test directory");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_ancestor_cannot_supply_a_trusted_coordination_body() {
        use std::os::unix::fs::symlink;
        let scratch = std::env::temp_dir().join(format!(
            "free-energy-slot-ancestor-path-{}",
            std::process::id()
        ));
        fs::create_dir_all(&scratch).expect("create isolated test root");
        let real = scratch.join("real");
        fs::create_dir(&real).expect("create regular parent");
        let body = real.join("slot-body");
        fs::write(&body, A).expect("write valid body");
        assert_eq!(read_body(body.to_str().unwrap()).unwrap(), A);

        let alias = scratch.join("alias");
        symlink(&real, &alias).expect("create symlinked parent directory");
        let path_through_alias = alias.join("slot-body");
        assert_eq!(
            read_body(path_through_alias.to_str().unwrap()).unwrap_err(),
            "coordination body path contains symlink component"
        );
        fs::remove_dir_all(&scratch).expect("remove isolated test root");
    }

    #[cfg(unix)]
    #[test]
    fn renamed_regular_file_after_observation_is_not_trusted() {
        let scratch =
            std::env::temp_dir().join(format!("free-energy-slot-open-swap-{}", std::process::id()));
        fs::create_dir_all(&scratch).expect("create isolated test directory");
        let source = scratch.join("slot-body");
        let replacement = scratch.join("replacement-body");
        fs::write(&source, A).expect("write original observed body");
        fs::write(&replacement, B).expect("write replacement body");
        let original = fs::symlink_metadata(&source).expect("observe original identity");
        fs::rename(&replacement, &source).expect("replace body between check and open");
        assert!(
            read_body_with_observed_metadata(source.to_str().unwrap(), &original)
                .unwrap_err()
                .contains("changed between metadata check and open"),
            "a swapped same-path regular file must never be accepted"
        );

        let prior = fs::symlink_metadata(&source).expect("observe second regular file");
        let outside = scratch.join("outside");
        fs::write(&outside, A).expect("create symlink destination");
        fs::remove_file(&source).expect("remove prior file");
        std::os::unix::fs::symlink(&outside, &source).expect("swap in symlink");
        assert!(
            read_body_with_observed_metadata(source.to_str().unwrap(), &prior).is_err(),
            "an open-time symlink swap cannot be trusted"
        );
        fs::remove_dir_all(&scratch).expect("remove isolated test directory");
    }

    #[test]
    fn bounded_reader_failure_cannot_select_an_active_slot() {
        let scratch = std::env::temp_dir().join(format!(
            "free-energy-slots-process-boundary-{}",
            std::process::id()
        ));
        fs::create_dir_all(&scratch).expect("create isolated test directory");
        let good = scratch.join("slot-a");
        let too_large = scratch.join("slot-b-oversized");
        fs::write(&good, A).unwrap();
        fs::write(&too_large, vec![b'Z'; MAX_BODY_BYTES as usize + 1]).unwrap();
        let args = [
            "geromet".to_owned(),
            "168".to_owned(),
            "geromet".to_owned(),
            TITLE.to_owned(),
            good.to_str().unwrap().to_owned(),
            "186".to_owned(),
            "geromet".to_owned(),
            TITLE.to_owned(),
            too_large.to_str().unwrap().to_owned(),
        ];
        assert!(execute(&args)
            .unwrap_err()
            .contains("second body unavailable"));
        fs::remove_dir_all(&scratch).unwrap();
    }
}
