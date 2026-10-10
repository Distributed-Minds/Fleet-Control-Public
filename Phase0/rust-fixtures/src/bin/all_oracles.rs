//! Offline Rust fixture-oracle runner for issue #71.
//!
//! Executes only a fixed allowlist of compiled sibling binaries, never commands
//! or paths supplied by fixture data. This is a developer verification runner:
//! passing fixtures is not Python semantic parity or runtime authorization.

use std::collections::HashSet;
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

// Aggregate status messages must not replay terminal-control characters from
// untrusted arguments or child stderr. Bound raw scalar count before escaping.
fn log_safe(raw: &str) -> String {
    raw.chars()
        .take(256)
        .flat_map(char::escape_default)
        .collect()
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
                            return Err(format!("unknown fixture family: {}", log_safe(value)));
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
            unknown => {
                return Err(format!(
                    "unknown or repeated argument: {}",
                    log_safe(unknown)
                ))
            }
        }
        i += 1;
    }
    Ok((selection, root))
}

// A zero exit code or arbitrary nonempty stdout is not a verified oracle PASS.
// Each fixed sibling has an explicit success-output contract. Preserve stderr:
// some successful oracles intentionally print non-authority disclaimers there.
// These oracles verify a fixed historical fixture corpus, not an arbitrary
// positive number of cases. A child that skips cases must not mint PASS.
fn exact_count(line: &str, prefix: &str, suffix: &str, expected: usize) -> bool {
    line == format!("{prefix}{expected}{suffix}")
}

fn has_verification_output(oracle: &Oracle, stdout: &[u8]) -> bool {
    let Ok(output) = std::str::from_utf8(stdout) else {
        return false;
    };
    let Some(line) = output.strip_suffix('\n') else {
        return false;
    };
    if line.is_empty() || line.contains('\r') {
        return false;
    }
    match oracle.name {
        "ad_hoc_research" => exact_count(line, "ad-hoc research fixtures (Rust): ", " passed", 47),
        "adaptive_stress" => exact_count(
            line,
            "adaptive-stress semantic fixtures (Rust): ",
            " passed",
            23,
        ),
        "authority_closure" => exact_count(
            line,
            "authority closure Rust semantic fixtures: ",
            " cases passed",
            26,
        ),
        "containment" => exact_count(line, "containment fixtures (Rust): ", " passed", 35),
        "containment_capacity" => exact_count(
            line,
            "containment-capacity fixtures (Rust): ",
            " passed",
            17,
        ),
        "coordination_history" => exact_count(
            line,
            "PASS: ",
            " independently evaluated coordination-history cases",
            18,
        ),
        "github_capability" => exact_count(
            line,
            "GitHub capability invariant fixtures (Rust): ",
            " checked",
            28,
        ),
        "integration_candidate" => exact_count(
            line,
            "integration candidate fixture envelope: ",
            " passed; SHA-256 identity parity NOT checked",
            6,
        ),
        "integration_candidate_digest" => {
            // An arbitrary or truncated list of plausible SHA-256 strings is
            // not evidence that all four historical candidate identities ran.
            // Bind the aggregate receipt to the digest CLI's fixed case set
            // and stable order, without asserting Git mutation authority.
            const REQUIRED: [&str; 4] = [
                "normal-two-parent",
                "reversed-parents-same-tree",
                "unsupported-three-parent",
                "compatible-constructor-migration",
            ];
            let entries: Vec<&str> = line.split('\n').collect();
            // The four fixed historical candidate envelopes differ in parent
            // order, parent cardinality or constructor identity. Four copies
            // of one plausible SHA-256 string cannot be a complete receipt.
            let mut seen_digests = HashSet::new();
            entries.len() == REQUIRED.len()
                && entries.iter().zip(REQUIRED).all(|(entry, expected_name)| {
                    let Some((name, digest)) = entry.split_once(": ") else {
                        return false;
                    };
                    name == expected_name
                        && digest.len() == 64
                        && digest
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                        && seen_digests.insert(digest)
                })
        }
        "merge_base_topology" => exact_count(
            line,
            "merge-base-topology: ",
            " read-only model fixtures PASS",
            12,
        ),
        _ => false,
    }
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
            Ok(output)
                if output.status.success() && has_verification_output(&oracle, &output.stdout) =>
            {
                passed.push(oracle.name);
            }
            Ok(output) if output.status.success() => failed.push(format!(
                "{}: child exited successfully without verification output",
                oracle.name
            )),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let detail = log_safe(stderr.trim());
                failed.push(format!(
                    "{}: child exited {}: {}",
                    oracle.name, output.status, detail
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
    // OS argv is untrusted; env::args() panics on Unix non-UTF-8 bytes and
    // bypasses the normal deterministic FAIL diagnostic for rejected input.
    let result = env::args_os()
        .skip(1)
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "non-UTF-8 CLI argument".to_owned())
        })
        .collect::<Result<Vec<String>, String>>()
        .and_then(|args| parse_args(&args))
        .and_then(|(selection, root)| run_oracles(selection, &root));
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
    fn only_family_specific_positive_output_qualifies_as_verification() {
        let expected = [
            ("ad_hoc_research", "ad-hoc research fixtures (Rust): 47 passed\n"),
            (
                "adaptive_stress",
                "adaptive-stress semantic fixtures (Rust): 23 passed\n",
            ),
            (
                "authority_closure",
                "authority closure Rust semantic fixtures: 26 cases passed\n",
            ),
            ("containment", "containment fixtures (Rust): 35 passed\n"),
            (
                "containment_capacity",
                "containment-capacity fixtures (Rust): 17 passed\n",
            ),
            (
                "coordination_history",
                "PASS: 18 independently evaluated coordination-history cases\n",
            ),
            (
                "github_capability",
                "GitHub capability invariant fixtures (Rust): 28 checked\n",
            ),
            (
                "integration_candidate",
                "integration candidate fixture envelope: 6 passed; SHA-256 identity parity NOT checked\n",
            ),
            (
                "merge_base_topology",
                "merge-base-topology: 12 read-only model fixtures PASS\n",
            ),
        ];
        for (name, output) in expected {
            let oracle = ORACLES.iter().find(|oracle| oracle.name == name).unwrap();
            assert!(has_verification_output(oracle, output.as_bytes()), "{name}");
        }
        let digest_oracle = ORACLES
            .iter()
            .find(|oracle| oracle.name == "integration_candidate_digest")
            .unwrap();
        let digest_output = format!(
            "normal-two-parent: {}\nreversed-parents-same-tree: {}\nunsupported-three-parent: {}\ncompatible-constructor-migration: {}\n",
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64)
        );
        assert!(has_verification_output(
            digest_oracle,
            digest_output.as_bytes()
        ));
    }

    #[test]
    fn nonempty_false_success_and_malformed_digest_output_fail_closed() {
        for oracle in ORACLES {
            for output in [
                "",
                " ",
                "PASS\n",
                "OK\n",
                "FAIL: the child did not verify anything\n",
                "0 passed\n",
                "1 passed\n",
                "containment fixtures (Rust): 0 passed\n",
                "containment fixtures (Rust): 35 passed\nEXTRA\n",
                "containment fixtures (Rust): 35 passed\r\n",
                "containment fixtures (Rust): 35 passed",
                "PASS integration_candidate\n",
                "PASS: 0 independently evaluated coordination-history cases\n",
            ] {
                assert!(
                    !has_verification_output(&oracle, output.as_bytes()),
                    "{} accepted unverified stdout {output:?}",
                    oracle.name
                );
            }
        }
        let digest_oracle = ORACLES
            .iter()
            .find(|oracle| oracle.name == "integration_candidate_digest")
            .unwrap();
        for output in [
            "case: not-a-digest\n",
            "case: \n",
            ": abcdef\n",
            "case: 1234\n",
            "case: 1234\n\n",
        ] {
            assert!(!has_verification_output(digest_oracle, output.as_bytes()));
        }
        let good = format!(
            "normal-two-parent: {}\nreversed-parents-same-tree: {}\nunsupported-three-parent: {}\ncompatible-constructor-migration: {}\n",
            "a".repeat(64),
            "b".repeat(64),
            "c".repeat(64),
            "d".repeat(64)
        );
        for repeated in ["b", "c", "d"] {
            let forged = good.replacen(&repeated.repeat(64), &"a".repeat(64), 1);
            assert!(
                !has_verification_output(digest_oracle, forged.as_bytes()),
                "accepted repeated SHA-256 digest for distinct historical envelopes"
            );
        }
        for invalid in [
            // Previously a single plausible digest, repeated names, swapped
            // order and undeclared names all qualified as aggregate success.
            format!("normal-two-parent: {}\n", "a".repeat(64)),
            good.replacen("reversed-parents-same-tree", "normal-two-parent", 1),
            good.replacen("normal-two-parent", "unknown-case", 1),
            good.replacen("normal-two-parent: ", "normal-two-parent: 0", 1),
            format!("{good}extra-case: 1234\n"),
            good.replacen("normal-two-parent: ", "normal-two-parent : ", 1),
            good.replacen("normal-two-parent: ", "normal-two-parent: ABC", 1),
            good.replacen("normal-two-parent: ", "reversed-parents-same-tree: ", 1),
        ] {
            assert!(
                !has_verification_output(digest_oracle, invalid.as_bytes()),
                "accepted malformed or incomplete digest receipt: {invalid:?}"
            );
        }
    }

    #[test]
    fn partial_or_forged_nonzero_counts_never_qualify_as_completed_fixture_families() {
        let expected = [
            (
                "ad_hoc_research",
                "ad-hoc research fixtures (Rust): ",
                " passed",
                47,
            ),
            (
                "adaptive_stress",
                "adaptive-stress semantic fixtures (Rust): ",
                " passed",
                23,
            ),
            (
                "authority_closure",
                "authority closure Rust semantic fixtures: ",
                " cases passed",
                26,
            ),
            (
                "containment",
                "containment fixtures (Rust): ",
                " passed",
                35,
            ),
            (
                "containment_capacity",
                "containment-capacity fixtures (Rust): ",
                " passed",
                17,
            ),
            (
                "coordination_history",
                "PASS: ",
                " independently evaluated coordination-history cases",
                18,
            ),
            (
                "github_capability",
                "GitHub capability invariant fixtures (Rust): ",
                " checked",
                28,
            ),
            (
                "integration_candidate",
                "integration candidate fixture envelope: ",
                " passed; SHA-256 identity parity NOT checked",
                6,
            ),
            (
                "merge_base_topology",
                "merge-base-topology: ",
                " read-only model fixtures PASS",
                12,
            ),
        ];
        for (name, prefix, suffix, count) in expected {
            let oracle = ORACLES.iter().find(|oracle| oracle.name == name).unwrap();
            for incomplete in [1, count - 1, count + 1] {
                let output = format!("{prefix}{incomplete}{suffix}\n");
                assert!(
                    !has_verification_output(oracle, output.as_bytes()),
                    "{name} accepted incorrect count {incomplete}"
                );
            }
            let padded = format!("{prefix}0{count}{suffix}\n");
            assert!(
                !has_verification_output(oracle, padded.as_bytes()),
                "{name}"
            );
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
    fn aggregate_errors_escape_untrusted_control_characters() {
        let injected = "bad\nPASS forged\r\u{1b}[2K\u{202e}name";
        let rendered = log_safe(injected);
        assert!(rendered.contains(r"\nPASS forged"));
        assert!(rendered.contains(r"\r"));
        assert!(rendered.contains(r"\u{1b}"));
        assert!(rendered.contains(r"\u{202e}"));
        assert!(!rendered.chars().any(char::is_control));

        let error =
            options(&["--unknown\nPASS forged\r\u{1b}"]).expect_err("malicious option must fail");
        assert!(error.contains(r"\nPASS forged"));
        assert!(!error.contains('\n'));
        assert!(!error.contains('\r'));
        assert!(!error.contains('\u{1b}'));

        let family_error =
            options(&["--family", "bogus\nPASS forged"]).expect_err("unknown family must fail");
        assert!(family_error.contains(r"\nPASS forged"));
        assert!(!family_error.contains('\n'));
        assert_eq!(log_safe(&"X".repeat(400)).len(), 256);
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
