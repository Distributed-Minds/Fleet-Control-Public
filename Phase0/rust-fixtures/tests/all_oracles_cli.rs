//! Process-level regression checks for the fixed allowlisted offline Rust runner.
//! These are real compiled binary subprocesses, not expected-label echoes.
//! Successful fixture execution does not prove migration parity or real authority.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const RUNNER: &str = env!("CARGO_BIN_EXE_all_oracles");
// Require Cargo to build every actual sibling executable used by --all.
const SIBLINGS: [&str; 10] = [
    env!("CARGO_BIN_EXE_ad_hoc_research"),
    env!("CARGO_BIN_EXE_adaptive_stress"),
    env!("CARGO_BIN_EXE_authority_closure"),
    env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"),
    env!("CARGO_BIN_EXE_containment_capacity"),
    env!("CARGO_BIN_EXE_coordination_history"),
    env!("CARGO_BIN_EXE_github_capability"),
    env!("CARGO_BIN_EXE_integration_candidate"),
    env!("CARGO_BIN_EXE_integration_candidate_digest"),
    env!("CARGO_BIN_EXE_merge_base_topology"),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0")
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn invoke(args: &[&str]) -> Output {
    Command::new(RUNNER)
        .args(args)
        .current_dir(root())
        .output()
        .expect("run compiled all_oracles")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 runner output")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 diagnostic")
}

#[test]
fn every_fixed_sibling_is_a_compiled_binary() {
    for path in SIBLINGS {
        assert!(Path::new(path).is_file(), "required sibling not built: {path}");
    }
}

#[test]
fn all_families_execute_real_oracles_and_report_only_actual_successes() {
    let output = invoke(&["--all"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
    let lines: Vec<_> = stdout(&output).lines().map(str::to_owned).collect();
    assert_eq!(
        lines,
        [
            "PASS ad_hoc_research",
            "PASS adaptive_stress",
            "PASS authority_closure",
            "PASS containment",
            "PASS containment_capacity",
            "PASS coordination_history",
            "PASS github_capability",
            "PASS integration_candidate",
            "PASS integration_candidate_digest",
            "PASS merge_base_topology",
        ]
    );
}

#[test]
fn one_family_does_not_claim_other_families_passed() {
    let output = invoke(&["--family", "authority_closure", "--root", root().to_str().unwrap()]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "PASS authority_closure\n");
}

#[test]
fn list_is_stable_and_does_not_require_checkout_or_executable_children() {
    let output = Command::new(RUNNER)
        .arg("--list")
        .current_dir("/")
        .output()
        .expect("list names from compiled binary");
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output).lines().count(), 10);
    assert!(stdout(&output).contains("integration_candidate_digest\n"));
}

#[test]
fn missing_root_and_unknown_family_fail_without_false_pass() {
    for args in [
        vec!["--root", "/nonexistent/free-energy-fixtures-test"],
        vec!["--family", "/bin/true"],
        vec!["--family", "not-a-known-oracle"],
        vec!["--all", "--family", "authority_closure"],
    ] {
        let output = invoke(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(stderr(&output).contains("FAIL:"), "{args:?}");
    }
}
