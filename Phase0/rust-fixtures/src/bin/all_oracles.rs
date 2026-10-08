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

// A zero exit code is not a fixture PASS unless the child emitted its
// documented, family-specific successful result. Arbitrary stdout such as an
// error banner or an empty digest set must not be upgraded to verification.
// Preserve stderr separately: valid digest fixtures print a scope caveat there.
fn has_verification_output(oracle: &Oracle, stdout: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(stdout) else {
        return false;
    };
    let text = text.trim_end_matches(|c| c == '\n' || c == '\r');

    if oracle.name == "integration_candidate_digest" {
        let mut observed = 0usize;
        for line in text.lines() {
            let Some((name, hex)) = line.rsplit_once(": ") else {
                return false;
            };
            if name.trim().is_empty()
                || hex.len() != 64
                || !hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return false;
            }
            observed += 1;
        }
        return observed > 0;
    }

    let (prefix, suffix) = match oracle.name {
        "ad_hoc_research" => ("ad-hoc research fixtures (Rust): ", " passed"),
        "adaptive_stress" => ("adaptive-stress semantic fixtures (Rust): ", " passed"),
        "authority_closure" => ("authority closure Rust semantic fixtures: ", " cases passed"),
        "containment" => ("containment fixtures (Rust): ", " passed"),
        "containment_capacity" => ("containment-capacity fixtures (Rust): ", " passed"),
        "coordination_history" => (
            "PASS: ",
            " independently evaluated coordination-history cases",
        ),
        "github_capability" => ("GitHub capability invariant fixtures (Rust): ", " checked"),
        "integration_candidate" => (
            "integration candidate fixture envelope: ",
            " passed; SHA-256 identity parity NOT checked",
        ),
        "merge_base_topology" => ("merge-base-topology: ", " read-only model fixtures PASS"),
        _ => return false,
    };
    let Some(count) = text
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
    else {
        return false;
    };
    !count.is_empty()
        && count.bytes().all(|byte| byte.is_ascii_digit())
        && count.parse::<usize>().is_ok_and(|number| number > 0)
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
            Ok(output) if output.status.success() && has_verification_output(&oracle, &output.stdout) => {
                println!("PASS {}", oracle.name);
            }
            Ok(output) if output.status.success() => failed.push(format!(
                "{}: child exited successfully without a recognizable verification result",
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
    fn only_recognizable_nonzero_family_results_qualify_as_verification() {
        let valid = [
            ("ad_hoc_research", "ad-hoc research fixtures (Rust): 1 passed"),
            ("adaptive_stress", "adaptive-stress semantic fixtures (Rust): 2 passed"),
            ("authority_closure", "authority closure Rust semantic fixtures: 3 cases passed"),
            ("containment", "containment fixtures (Rust): 35 passed"),
            ("containment_capacity", "containment-capacity fixtures (Rust): 4 passed"),
            ("coordination_history", "PASS: 5 independently evaluated coordination-history cases"),
            ("github_capability", "GitHub capability invariant fixtures (Rust): 6 checked"),
            ("integration_candidate", "integration candidate fixture envelope: 7 passed; SHA-256 identity parity NOT checked"),
            ("merge_base_topology", "merge-base-topology: 12 read-only model fixtures PASS"),
        ];
        for (name, output) in valid {
            let oracle = ORACLES.iter().find(|oracle| oracle.name == name).unwrap();
            assert!(has_verification_output(oracle, output.as_bytes()), "{name}");
            for bogus in [
                "".as_bytes(),
                b" \n\t",
                b"PASS",
                b"FAIL: child skipped verification",
                b"error: no fixture evaluated",
                b"\xff",
                b"containment fixtures (Rust): 0 passed",
                b"containment fixtures (Rust): 35 passed\nFAIL: later error",
            ] {
                assert!(!has_verification_output(oracle, bogus), "{name}: {bogus:?}");
            }
        }

        let digest = ORACLES
            .iter()
            .find(|oracle| oracle.name == "integration_candidate_digest")
            .unwrap();
        let valid_digest = format!("original-case: {}\n", "a".repeat(64));
        assert!(has_verification_output(digest, valid_digest.as_bytes()));
        for bogus in [
            "".to_owned(),
            "PASS digest".to_owned(),
            "digest: abc".to_owned(),
            format!("digest: {}\nFAIL: bad input", "a".repeat(64)),
            format!("digest: {}\n", "0".repeat(63)),
        ] {
            assert!(!has_verification_output(digest, bogus.as_bytes()), "{bogus}");
        }
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
