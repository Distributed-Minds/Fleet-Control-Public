//! Offline, MODEL_ONLY conversion of native Phase0 coordination comments.
//!
//! Input is a caller-supplied TSV, NOT a trusted GitHub or archive export:
//!   certified_position<TAB>provider_comment_id<TAB>PHASE0 | seq=... | run=... | ...
//!
//! A separate trusted adapter must establish complete provider ordering (including
//! archive/live cut), comment IDs, author trust, live slot/epoch, source incarnation,
//! and freshness. This tool cannot do that and grants NO ownership/write authority.
//! A contiguous position sequence detects gaps *inside the supplied window*, not
//! an omitted prefix/suffix or fabricated positions.

use free_energy_coordination_archival::ownership::{reduce_model_only, Scope, State, Transition};
use std::{collections::HashMap, env, fs, io::Read, path::Path, process::ExitCode};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RECORDS: usize = 100_000;

fn positive(label: &str, value: &str) -> Result<u64, String> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(format!("{label}: expected canonical positive decimal"));
    }
    match value.parse::<u64>() {
        Ok(number) if number != 0 => Ok(number),
        _ => Err(format!("{label}: invalid or overflowing positive decimal")),
    }
}

fn optional_number(label: &str, value: &str) -> Result<Option<u64>, String> {
    match value {
        "none" | "-" => Ok(None),
        _ => positive(label, value).map(Some),
    }
}

fn required<'a>(fields: &'a HashMap<&str, &str>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| format!("missing {key}"))
}

fn parse_row(
    line: &str,
    row: usize,
    scopes: &mut HashMap<String, Scope>,
) -> Result<Transition, String> {
    let mut columns = line.splitn(3, '\t');
    let position = positive(
        "certified_position",
        columns.next().ok_or("missing position")?,
    )?;
    let comment_id = positive(
        "provider_comment_id",
        columns.next().ok_or("missing comment ID")?,
    )?;
    let raw = columns.next().ok_or("missing native Phase0 record")?;
    if raw.contains(['\t', '\r', '\n']) {
        return Err(format!("row {row}: invalid control in native record"));
    }
    let body = raw
        .strip_prefix("PHASE0 | ")
        .ok_or_else(|| format!("row {row}: not a native PHASE0 record"))?;
    let mut fields = HashMap::new();
    for part in body.split(" | ") {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| format!("row {row}: malformed field"))?;
        // Unknown fields are rejected, not trusted or silently reinterpreted.
        if value.is_empty()
            || !matches!(
                key,
                "seq"
                    | "run"
                    | "agent"
                    | "state"
                    | "mission"
                    | "issue"
                    | "pr"
                    | "branch"
                    | "head"
                    | "seam"
                    | "prev"
                    | "ts"
                    | "note"
                    | "outcome"
                    | "stale_run"
            )
            || fields.insert(key, value).is_some()
        {
            return Err(format!(
                "row {row}: unknown, empty or duplicate field {key:?}"
            ));
        }
    }

    let run = required(&fields, "run")?.to_owned();
    let state = match required(&fields, "state")? {
        "INTENT" => State::Intent,
        "OWNED" => State::Owned,
        "WORKING" => State::Working,
        "HANDOFF" => State::Handoff,
        "RELEASE" => State::Release,
        "YIELD" => State::Yield,
        "RECOVERED" => State::Recovered,
        _ => return Err(format!("row {row}: unsupported state")),
    };
    let terminal = matches!(state, State::Handoff | State::Release | State::Yield);
    let old = scopes.get(&run);

    // Installed terminal records can omit scope fields. Inherit only from the
    // same run's previous record; supplied fields must still match in reducer.
    let issue = match fields.get("issue") {
        Some(raw) => optional_number("issue", raw)?,
        None if terminal => old.ok_or("terminal without prior run scope")?.issue,
        None => return Err(format!("row {row}: missing issue")),
    };
    let pr = match fields.get("pr") {
        Some(raw) => optional_number("pr", raw)?,
        None if terminal => old.ok_or("terminal without prior run scope")?.pr,
        None => return Err(format!("row {row}: missing pr")),
    };
    let branch = match fields.get("branch") {
        Some(&"none" | &"-") => None,
        Some(value) if !value.is_empty() => Some((*value).to_owned()),
        Some(_) => return Err(format!("row {row}: invalid branch")),
        None if terminal => old
            .ok_or("terminal without prior run scope")?
            .branch
            .clone(),
        None => return Err(format!("row {row}: missing branch")),
    };
    let seam = match fields.get("seam") {
        Some(value) => (*value).to_owned(),
        None if terminal => old.ok_or("terminal without prior run scope")?.seam.clone(),
        None => return Err(format!("row {row}: missing seam")),
    };
    let prev = match fields.get("prev") {
        Some(raw) => optional_number("prev", raw)?,
        None => None,
    };
    let transition = Transition {
        position,
        comment_id,
        run: run.clone(),
        seq: positive("seq", required(&fields, "seq")?)?,
        state,
        prev,
        scope: Scope {
            issue,
            pr,
            branch,
            seam,
        },
    };
    scopes.insert(run, transition.scope.clone());
    Ok(transition)
}

fn parse_native(input: &str) -> Result<Vec<Transition>, String> {
    let mut scopes = HashMap::new();
    let mut parsed: Vec<Transition> = Vec::new();
    for (index, line) in input.split_terminator('\n').enumerate() {
        if line.is_empty() || parsed.len() >= MAX_RECORDS {
            return Err(format!("row {}: blank or too many records", index + 1));
        }
        let event = parse_row(line, index + 1, &mut scopes)?;
        if let Some(previous) = parsed.last() {
            if previous.position.checked_add(1) != Some(event.position) {
                return Err(format!("row {}: gap in certified positions", index + 1));
            }
        }
        parsed.push(event);
    }
    if parsed.is_empty() {
        return Err("empty input is not a complete coordination history".to_owned());
    }
    Ok(parsed)
}

// Leaf-only lstat and O_NOFOLLOW do not reject symlinked parent directories.
// This bounded advisory preflight follows the existing coordination_slots model;
// it is NOT an atomic directory-handle sandbox against concurrent renames.
fn reject_symlinked_ancestors(path: &Path) -> Result<(), String> {
    let mut inspected = std::path::PathBuf::new();
    if let Some(parent) = path.parent() {
        for component in parent.components() {
            inspected.push(component.as_os_str());
            let metadata = fs::symlink_metadata(&inspected)
                .map_err(|error| format!("cannot inspect {inspected:?}: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err("source path contains symlink component".to_owned());
            }
        }
    }
    Ok(())
}

fn read_bounded(path: &Path) -> Result<String, String> {
    reject_symlinked_ancestors(path)?;
    let before =
        fs::symlink_metadata(path).map_err(|error| format!("cannot stat {path:?}: {error}"))?;
    if !before.file_type().is_file() {
        return Err("source must be a non-symlink regular file".to_owned());
    }
    if before.len() > MAX_BYTES {
        return Err("source exceeds 16 MiB".to_owned());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // No symlink traversal and no blocking on a swapped-in FIFO.
        options.custom_flags(0o4000 | 0o400000); // O_NONBLOCK | O_NOFOLLOW
    }
    let file = options
        .open(path)
        .map_err(|error| format!("cannot open {path:?}: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("cannot stat opened source: {error}"))?;
    if !opened.file_type().is_file() || opened.len() > MAX_BYTES {
        return Err("opened source is nonregular or over 16 MiB".to_owned());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err("source changed between preflight and open".to_owned());
        }
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read source: {error}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("source grew beyond 16 MiB".to_owned());
    }
    String::from_utf8(bytes).map_err(|_| "source is not UTF-8".to_owned())
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(path), None) = (args.next(), args.next()) else {
        eprintln!("usage: coordination_phase0_records <certified-comments.tsv>");
        return ExitCode::FAILURE;
    };
    let result = read_bounded(Path::new(&path))
        .and_then(|text| parse_native(&text))
        .and_then(|events| {
            reduce_model_only(&events)
                .map(|leases| (events.len(), leases))
                .map_err(|error| format!("{error:?}"))
        });
    match result {
        Err(error) => {
            eprintln!("MODEL_REJECTED {error}");
            ExitCode::FAILURE
        }
        Ok((records, active)) => {
            println!("MODEL_ONLY records={records} active={}", active.len());
            for lease in active {
                println!(
                    "MODEL_ACTIVE run={:?} state={:?} comment_id={} branch={:?} seam={:?}",
                    lease.run,
                    lease.state,
                    lease.latest_comment_id,
                    lease.scope.branch,
                    lease.scope.seam
                );
            }
            println!("MODEL_ONLY untrusted_input no_provider_authentication no_mutation_authority");
            ExitCode::SUCCESS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        concat!(
            "10\t101\tPHASE0 | seq=1 | run=run-a | agent=agent | state=INTENT | mission=none | issue=22 | pr=none | branch=branch-a | head=abc | seam=fixture-native\n",
            "11\t102\tPHASE0 | seq=2 | run=run-a | agent=agent | state=OWNED | mission=none | issue=22 | pr=none | branch=branch-a | head=abc | seam=fixture-native | prev=101\n",
            "12\t103\tPHASE0 | seq=3 | run=run-a | agent=agent | state=WORKING | mission=none | issue=22 | pr=none | branch=branch-a | head=abc | seam=fixture-native | prev=102\n",
        )
        .to_owned()
    }

    #[test]
    fn parses_actual_phased_record_shape_and_honors_terminal_scope_inheritance() {
        let input = sample();
        let active = reduce_model_only(&parse_native(&input).unwrap()).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].latest_comment_id, 103);
        let ended = format!(
            "{input}13\t104\tPHASE0 | seq=4 | run=run-a | agent=agent | state=HANDOFF | mission=none | head=abc | prev=103 | outcome=verified\n"
        );
        assert!(reduce_model_only(&parse_native(&ended).unwrap())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn rejects_gaps_duplicate_keys_spoofed_fields_and_scope_drift() {
        for invalid in [
            sample().replace("11\t102", "13\t102"),
            sample().replace("run=run-a | agent", "run=run-a | run=forged | agent"),
            sample().replace("seam=fixture-native", "seam=fixture-native | power=grant"),
            sample().replace("11\t102", "11\t101"),
            sample().replace("11\t102", "11\tnot-a-number"),
        ] {
            let result = parse_native(&invalid)
                .and_then(|r| reduce_model_only(&r).map_err(|error| format!("{error:?}")));
            assert!(result.is_err(), "unexpectedly admitted {invalid:?}");
        }
        let drift = sample().replace(
            "state=OWNED | mission=none | issue=22",
            "state=OWNED | mission=none | issue=23",
        );
        assert!(reduce_model_only(&parse_native(&drift).unwrap()).is_err());
    }

    #[test]
    fn missing_terminal_predecessor_and_blank_history_fail_closed() {
        assert!(parse_native("").is_err());
        assert!(parse_native("\n").is_err());
        let lone = "1\t101\tPHASE0 | seq=1 | run=x | state=HANDOFF | prev=100\n";
        assert!(parse_native(lone).is_err());
    }

    #[test]
    fn regular_file_bound_and_missing_file_are_not_model_success() {
        let missing = std::env::temp_dir().join(format!(
            "nonexistent-free-energy-native-comments-{}",
            std::process::id()
        ));
        assert!(read_bounded(&missing).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn native_coordination_reader_rejects_symlinked_parent_and_leaf_paths() {
        use std::os::unix::fs::symlink;

        let scratch = std::env::temp_dir().join(format!(
            "free-energy-native-records-ancestor-{}",
            std::process::id()
        ));
        fs::create_dir_all(&scratch).expect("create isolated scratch root");
        let real = scratch.join("real");
        fs::create_dir(&real).expect("create real input parent");
        let input = real.join("records.tsv");
        fs::write(&input, sample()).expect("write legitimate records");
        assert_eq!(read_bounded(&input).unwrap(), sample());

        let alias = scratch.join("linked-parent");
        symlink(&real, &alias).expect("create symlink ancestor");
        let unsafe_path = alias.join("records.tsv");
        assert!(
            read_bounded(&unsafe_path)
                .unwrap_err()
                .contains("symlink component"),
            "a symlinked parent must not supply an apparently valid history"
        );

        let leaf_alias = scratch.join("linked-leaf.tsv");
        symlink(&input, &leaf_alias).expect("create symlink leaf");
        assert!(
            read_bounded(&leaf_alias).is_err(),
            "existing leaf protection must stay fail-closed"
        );

        fs::remove_dir_all(&scratch).expect("remove isolated scratch root");
    }

}
