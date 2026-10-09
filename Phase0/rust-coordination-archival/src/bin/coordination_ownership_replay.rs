//! Bounded, offline, read-only inspection of Phase0 ownership transitions.
//!
//! Input is a *caller-supplied* tab-separated transcript, NOT an authenticated
//! GitHub snapshot. An independent trusted adapter must certify completeness,
//! order, authors, slot epoch, archived history and exact source incarnation.
//! Neither a successful exit nor MODEL_ONLY output grants mutation authority.
//!
//! Each nonempty line has exactly ten TAB-separated fields:
//! position, comment_id, run, seq, state, prev, issue, pr, branch, seam.
//! Use "-" for absent prev/issue/pr/branch. Positions are the certified
//! reducer order and cannot be inferred from comment IDs or timestamps.
//!
//! Usage: coordination_ownership_replay <transcript.tsv>
//! Exit 0 means only that the supplied model is internally consistent.
//! Exit 1 means the input or reduction is invalid. No files are modified.

use free_energy_coordination_archival::ownership::{reduce_model_only, Scope, State, Transition};
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
    process::ExitCode,
};

const MAX_INPUT_BYTES: u64 = 16 * 1024 * 1024;

fn positive(label: &str, raw: &str) -> Result<u64, String> {
    if raw.is_empty()
        || (raw.len() > 1 && raw.starts_with('0'))
        || !raw.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(format!(
            "{label}: expected canonical positive decimal integer"
        ));
    }
    match raw.parse::<u64>() {
        Ok(value) if value != 0 => Ok(value),
        _ => Err(format!("{label}: invalid or overflowing positive integer")),
    }
}

fn optional_number(label: &str, raw: &str) -> Result<Option<u64>, String> {
    if raw == "-" {
        Ok(None)
    } else {
        positive(label, raw).map(Some)
    }
}

fn parse_transition(line: &str, number: usize) -> Result<Transition, String> {
    let fields: Vec<_> = line.split('\t').collect();
    if fields.len() != 10 {
        return Err(format!(
            "line {number}: expected exactly ten TAB-separated fields"
        ));
    }
    let state = match fields[4] {
        "INTENT" => State::Intent,
        "OWNED" => State::Owned,
        "WORKING" => State::Working,
        "HANDOFF" => State::Handoff,
        "RELEASE" => State::Release,
        "YIELD" => State::Yield,
        "RECOVERED" => State::Recovered,
        _ => return Err(format!("line {number}: unknown transition state")),
    };
    if fields[2].is_empty() || fields[2] == "-" || fields[9].is_empty() || fields[9] == "-" {
        return Err(format!("line {number}: missing run or seam"));
    }
    let branch = match fields[8] {
        "-" => None,
        raw if !raw.is_empty() => Some(raw.to_owned()),
        _ => return Err(format!("line {number}: invalid empty branch")),
    };
    Ok(Transition {
        position: positive("position", fields[0])?,
        comment_id: positive("comment_id", fields[1])?,
        run: fields[2].to_owned(),
        seq: positive("seq", fields[3])?,
        state,
        prev: optional_number("prev", fields[5])?,
        scope: Scope {
            issue: optional_number("issue", fields[6])?,
            pr: optional_number("pr", fields[7])?,
            branch,
            seam: fields[9].to_owned(),
        },
    })
}

fn parse_transcript(input: &str) -> Result<Vec<Transition>, String> {
    let mut transitions = Vec::new();
    for (index, line) in input.split_terminator('\n').enumerate() {
        if line.is_empty() {
            return Err(format!("line {}: empty record", index + 1));
        }
        transitions.push(parse_transition(line, index + 1)?);
    }
    if transitions.is_empty() {
        return Err("empty transcript is not proof of a complete history".to_owned());
    }
    Ok(transitions)
}

fn read_bounded(path: &Path) -> Result<String, String> {
    // A byte limit does not protect an offline CLI from opening a FIFO, device,
    // directory or caller-controlled symlink. This is not provider admission.
    let before = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {path:?}: {error}"))?;
    if !before.file_type().is_file() {
        return Err("transcript must be a regular non-symlink file".to_owned());
    }
    if before.len() > MAX_INPUT_BYTES {
        return Err("transcript exceeds the 16 MiB model limit".to_owned());
    }

    let file = File::open(path).map_err(|error| format!("cannot open {path:?}: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("cannot inspect opened {path:?}: {error}"))?;
    if !opened.is_file() {
        return Err("opened transcript is not a regular file".to_owned());
    }
    // An attacker may replace the directory entry after the preflight. Bind
    // the opened descriptor to the observed inode on Unix, failing closed on
    // replacement. This is not a general atomic/no-follow open guarantee.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err("transcript identity changed between inspection and open".to_owned());
        }
    }

    let mut bytes = Vec::new();
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {path:?}: {error}"))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err("transcript exceeds the 16 MiB model limit".to_owned());
    }
    String::from_utf8(bytes).map_err(|_| "transcript must be UTF-8".to_owned())
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: coordination_ownership_replay <transcript.tsv>");
        return ExitCode::FAILURE;
    };
    if args.next().is_some() {
        eprintln!("usage: coordination_ownership_replay <transcript.tsv>");
        return ExitCode::FAILURE;
    }
    let input = match read_bounded(Path::new(&path)) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("MODEL_UNKNOWN {error}");
            return ExitCode::FAILURE;
        }
    };
    let transitions = match parse_transcript(&input) {
        Ok(records) => records,
        Err(error) => {
            eprintln!("MODEL_REJECTED {error}");
            return ExitCode::FAILURE;
        }
    };
    let leases = match reduce_model_only(&transitions) {
        Ok(leases) => leases,
        Err(error) => {
            eprintln!("MODEL_REJECTED {error:?}; investigate certified history");
            return ExitCode::FAILURE;
        }
    };
    // Nothing is printed on stdout until every supplied transition is accepted.
    println!(
        "MODEL_ONLY records={} active={}",
        transitions.len(),
        leases.len()
    );
    for lease in leases {
        println!(
            "MODEL_ACTIVE run={:?} state={:?} comment_id={} issue={:?} pr={:?} branch={:?} seam={:?}",
            lease.run,
            lease.state,
            lease.latest_comment_id,
            lease.scope.issue,
            lease.scope.pr,
            lease.scope.branch,
            lease.scope.seam
        );
    }
    println!("MODEL_ONLY not_authenticated not_authoritative no_write_permission");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use free_energy_coordination_archival::ownership::ReductionFailure;

    const TRANSCRIPT: &str = concat!(
        "1\t101\trun-a\t1\tINTENT\t-\t22\t-\tbranch-a\tseam-a\n",
        "2\t102\trun-a\t2\tOWNED\t101\t22\t-\tbranch-a\tseam-a\n",
        "3\t103\trun-a\t3\tWORKING\t102\t22\t-\tbranch-a\tseam-a\n",
    );

    #[test]
    fn complete_chain_yields_only_model_active_state() {
        let events = parse_transcript(TRANSCRIPT).unwrap();
        let result = reduce_model_only(&events).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].run, "run-a");
        assert_eq!(result[0].state, State::Working);
        assert_eq!(result[0].latest_comment_id, 103);
    }

    #[test]
    fn terminal_release_removes_active_lease() {
        let transcript =
            format!("{TRANSCRIPT}4\t104\trun-a\t4\tRELEASE\t103\t22\t-\tbranch-a\tseam-a\n");
        assert!(reduce_model_only(&parse_transcript(&transcript).unwrap())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn conflicting_owner_is_rejected_not_displayed_as_second_lease() {
        let transcript = format!(
            "{TRANSCRIPT}4\t104\trun-b\t1\tINTENT\t-\t22\t-\tbranch-b\tseam-a\n\
             5\t105\trun-b\t2\tOWNED\t104\t22\t-\tbranch-b\tseam-a\n"
        );
        assert_eq!(
            reduce_model_only(&parse_transcript(&transcript).unwrap()),
            Err(ReductionFailure::OverlappingOwner)
        );
    }

    #[test]
    fn malformed_and_ambiguous_rows_never_enter_reduction() {
        for transcript in [
            "",
            "\n",
            "1\t101\trun-a\t1\tINTENT\t-\t22\t-\tbranch-a\n",
            "01\t101\trun-a\t1\tINTENT\t-\t22\t-\tbranch-a\tseam-a\n",
            "1\t101\trun-a\t1\tINTENT\t-\t22\t-\tbranch-a\tseam-a\tignored\n",
            "1\t101\trun-a\t1\tOWNED\t-\t22\t-\tbranch-a\tseam-a\n\n",
        ] {
            if let Ok(events) = parse_transcript(transcript) {
                assert!(
                    reduce_model_only(&events).is_err(),
                    "invalid chain unexpectedly admitted: {transcript:?}"
                );
            }
        }
    }

    #[test]
    fn incomplete_order_or_duplicate_comment_is_not_authoritative() {
        let repeated_position =
            format!("{TRANSCRIPT}3\t104\trun-b\t1\tINTENT\t-\t22\t-\tbranch-b\tseam-b\n");
        assert_eq!(
            reduce_model_only(&parse_transcript(&repeated_position).unwrap()),
            Err(ReductionFailure::UnorderedOrIncomplete)
        );
        let repeated_id =
            format!("{TRANSCRIPT}4\t103\trun-b\t1\tINTENT\t-\t22\t-\tbranch-b\tseam-b\n");
        assert_eq!(
            reduce_model_only(&parse_transcript(&repeated_id).unwrap()),
            Err(ReductionFailure::DuplicateComment)
        );
    }
    #[test]
    fn reader_admits_only_bounded_regular_transcripts() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "free-energy-ownership-replay-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("temporary directory");
        let regular = directory.join("valid.tsv");
        fs::write(&regular, TRANSCRIPT).expect("regular transcript");
        assert_eq!(
            read_bounded(&regular).expect("regular bounded transcript"),
            TRANSCRIPT
        );
        let events = parse_transcript(&read_bounded(&regular).unwrap()).unwrap();
        assert_eq!(reduce_model_only(&events).unwrap().len(), 1);

        assert!(
            read_bounded(&directory)
                .expect_err("directory must be denied")
                .contains("regular non-symlink")
        );

        let oversized = directory.join("oversized.tsv");
        File::create(&oversized)
            .expect("sparse fixture")
            .set_len(MAX_INPUT_BYTES + 1)
            .expect("oversized sparse fixture");
        assert!(
            read_bounded(&oversized)
                .expect_err("oversized file must be denied")
                .contains("16 MiB")
        );

        #[cfg(unix)]
        {
            let alias = directory.join("alias.tsv");
            std::os::unix::fs::symlink(&regular, &alias).expect("symlink fixture");
            assert!(
                read_bounded(&alias)
                    .expect_err("symlink must be denied")
                    .contains("regular non-symlink")
            );
        }
        fs::remove_dir_all(directory).expect("cleanup temporary fixtures");
    }

}
