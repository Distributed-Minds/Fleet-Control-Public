//! Offline Rust fixture-oracle runner for issue #71.
//!
//! Executes only a fixed allowlist of compiled sibling binaries, never commands
//! or paths supplied by fixture data. This is a developer verification runner:
//! passing fixtures is not Python semantic parity or runtime authorization.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

#[derive(Clone, Copy)]
struct Oracle {
    name: &'static str,
    executable: &'static str,
    fixture: Option<&'static str>,
}

const ORACLES: [Oracle; 10] = [
    Oracle {
        name: "ad_hoc_research",
        executable: "ad_hoc_research",
        fixture: None,
    },
    Oracle {
        name: "adaptive_stress",
        executable: "adaptive_stress",
        fixture: None,
    },
    Oracle {
        name: "authority_closure",
        executable: "authority_closure",
        fixture: Some("Phase0/fixtures/authority-closure-spec2.json"),
    },
    Oracle {
        name: "containment",
        executable: "free-energy-phase0-fixtures",
        fixture: None,
    },
    Oracle {
        name: "containment_capacity",
        executable: "containment_capacity",
        fixture: None,
    },
    Oracle {
        name: "coordination_history",
        executable: "coordination_history",
        fixture: None,
    },
    Oracle {
        name: "github_capability",
        executable: "github_capability",
        fixture: None,
    },
    Oracle {
        name: "integration_candidate",
        executable: "integration_candidate",
        fixture: None,
    },
    Oracle {
        name: "integration_candidate_digest",
        executable: "integration_candidate_digest",
        fixture: None,
    },
    Oracle {
        name: "merge_base_topology",
        executable: "merge_base_topology",
        fixture: None,
    },
];

#[derive(Debug, PartialEq, Eq)]
enum Selection {
    All,
    List,
    Family(String),
}

fn parse_args(args: &[String]) -> Result<(Selection, PathBuf), String> {
    let mut selection = Selection::All;
    let mut selection_seen = false;
    let mut root = PathBuf::from(".");
    let mut root_seen = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--all" | "--list" | "--family" if !selection_seen => {
                selection_seen = true;
                selection = match args[i].as_str() {
                    "--all" => Selection::All,
                    "--list" => Selection::List,
                    _ => {
                        i += 1;
                        let value = args.get(i).ok_or("--family requires a known name")?;
                        if !ORACLES.iter().any(|oracle| oracle.name == value) {
                            return Err(format!("unknown fixture family: {value}"));
                        }
                        Selection::Family(value.clone())
                    }
                };
            }
            "--root" if !root_seen => {
                root_seen = true;
                i += 1;
                let value = args.get(i).ok_or("--root requires a directory")?;
                if value.is_empty() {
                    return Err("--root requires a nonempty directory".to_owned());
                }
                root = PathBuf::from(value);
            }
            unknown => return Err(format!("unknown or repeated argument: {unknown}")),
        }
        i += 1;
    }
    Ok((selection, root))
}

// A zero exit code without an actual verification result is not evidence of
// passing a fixture. Preserve stderr: some valid oracles print warnings there.
fn has_verification_output(stdout: &[u8]) -> bool {
    stdout.iter().any(|byte| !byte.is_ascii_whitespace())
}

fn run_oracles(selection: Selection, root: &Path) -> Result<(), String> {
    if selection == Selection::List {
        for oracle in ORACLES {
            println!("{}", oracle.name);
        }
        return Ok(());
    }

    let root = fs::canonicalize(root)
        .map_err(|error| format!("cannot resolve repository root: {error}"))?;
    if !root.join("Phase0/rust-fixtures/Cargo.toml").is_file() {
        return Err("root is missing Phase0/rust-fixtures/Cargo.toml".to_owned());
    }
    let executable =
        env::current_exe().map_err(|error| format!("cannot locate runner executable: {error}"))?;
    let bin_dir = executable
        .parent()
        .ok_or("runner executable has no parent directory")?;
    let mut failed = Vec::new();
    let mut passed = Vec::new();
    for oracle in ORACLES {
        if let Selection::Family(ref chosen) = selection {
            if chosen != oracle.name {
                continue;
            }
        }
        let target = bin_dir.join(oracle.executable);
        if !target.is_file() {
            failed.push(format!(
                "{}: compiled sibling executable is missing",
                oracle.name
            ));
            continue;
        }
        let mut command = Command::new(target);
        command.current_dir(&root);
        if let Some(fixture) = oracle.fixture {
            command.arg(root.join(fixture));
        }
        match command.output() {
            Ok(output) if output.status.success() && has_verification_output(&output.stdout) => {
                passed.push(oracle.name);
            }
            Ok(output) if output.status.success() => failed.push(format!(
                "{}: child exited successfully without verification output",
                oracle.name
            )),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail = stderr.trim().chars().take(256).collect::<String>();
                failed.push(format!(
                    "{}: child exited {}: {}",
                    oracle.name,
                    output.status,
                    detail.replace('\n', "\\n")
                ));
            }
            Err(error) => failed.push(format!("{}: could not execute: {error}", oracle.name)),
        }
    }
    if failed.is_empty() {
        // Aggregate success is atomic: never emit PASS for a failed --all run.
        for name in passed {
            println!("PASS {name}");
        }
        Ok(())
    } else {
        Err(failed.join("\nFAIL: "))
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = parse_args(&args).and_then(|(selection, root)| run_oracles(selection, &root));
    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        eprintln!("usage: all_oracles [--all | --list | --family NAME] [--root REPOSITORY]");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(input: &[&str]) -> Result<(Selection, PathBuf), String> {
        parse_args(
            &input
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn silent_or_whitespace_only_child_output_never_qualifies_as_verification() {
        for output in [b"".as_slice(), b" ", b"\n\t\r "] {
            assert!(!has_verification_output(output));
        }
        assert!(has_verification_output(
            b"containment fixtures (Rust): 35 passed\n"
        ));
        assert!(has_verification_output(b"PASS integration_candidate\n"));
    }

    #[test]
    fn fixed_family_selection_and_deterministic_inventory() {
        assert_eq!(ORACLES.len(), 10);
        assert_eq!(
            options(&["--family", "authority_closure"]).unwrap().0,
            Selection::Family("authority_closure".to_owned())
        );
        assert_eq!(options(&["--list"]).unwrap().0, Selection::List);
        for pair in ORACLES.windows(2) {
            assert!(pair[0].name < pair[1].name, "family order must be stable");
        }
    }

    #[test]
    fn no_external_command_or_repeated_mode_can_be_injected() {
        for arguments in [
            vec!["--family", "/bin/true"],
            vec!["--family", "all_oracles"],
            vec!["--family"],
            vec!["--root"],
            vec!["--all", "--list"],
            vec!["--root", ".", "--root", "."],
            vec!["--family", "authority_closure", "--all"],
            vec!["--unknown"],
        ] {
            assert!(options(&arguments).is_err(), "{arguments:?}");
        }
    }

    #[test]
    fn wrong_checkout_root_fails_before_any_child_execution() {
        assert!(run_oracles(Selection::All, Path::new("/nonexistent/free-energy-root")).is_err());
        assert!(run_oracles(Selection::Family("containment".into()), Path::new("/")).is_err());
    }
}
