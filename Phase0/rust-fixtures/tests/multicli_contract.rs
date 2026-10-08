//! Process-level regression checks for every staged external-file Rust fixture CLI.
//! These are historical fixture tests, not parity or runtime enforcement evidence.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

const CLIS: [(&str, &str, &str); 8] = [
    (
        "containment",
        env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"),
        "containment-spec3.json",
    ),
    (
        "ad_hoc_research",
        env!("CARGO_BIN_EXE_ad_hoc_research"),
        "ad-hoc-research-spec1.json",
    ),
    (
        "adaptive_stress",
        env!("CARGO_BIN_EXE_adaptive_stress"),
        "adaptive-stress-spec2.json",
    ),
    (
        "authority_closure",
        env!("CARGO_BIN_EXE_authority_closure"),
        "authority-closure-spec2.json",
    ),
    (
        "containment_capacity",
        env!("CARGO_BIN_EXE_containment_capacity"),
        "containment-capacity-spec2.json",
    ),
    (
        "coordination_history",
        env!("CARGO_BIN_EXE_coordination_history"),
        "coordination-history-spec5.json",
    ),
    (
        "github_capability",
        env!("CARGO_BIN_EXE_github_capability"),
        "github-capability-spec5.json",
    ),
    (
        "integration_candidate",
        env!("CARGO_BIN_EXE_integration_candidate"),
        "integration-candidate-v1.json",
    ),
];

fn historical_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 parent")
        .join("fixtures")
        .join(name)
}

fn invoke(binary: &str, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .output()
        .expect("compiled Rust fixture binary is executable")
}

fn temporary_input(contents: &str) -> PathBuf {
    let counter = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-multicli-{}-{counter}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("unique temporary fixture path");
    file.write_all(contents.as_bytes())
        .expect("write exact temporary fixture");
    path
}

fn invoke_temporary(binary: &str, contents: &str) -> Output {
    let path = temporary_input(contents);
    let output = invoke(binary, &[path.to_str().expect("UTF-8 temp path")]);
    fs::remove_file(path).expect("remove temporary fixture");
    output
}

fn assert_denied(output: &Output, family: &str) {
    assert!(
        !output.status.success(),
        "{family} incorrectly accepted a negative input: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stdout.is_empty(),
        "{family} emitted a success line on failure: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !output.stderr.is_empty(),
        "{family} did not provide a failure diagnostic"
    );
}

#[test]
fn every_staged_cli_accepts_its_checked_in_historical_fixture() {
    for (family, binary, filename) in CLIS {
        let file = historical_fixture(filename);
        let output = invoke(binary, &[file.to_str().expect("UTF-8 repository path")]);
        assert!(
            output.status.success(),
            "{family} rejected historical fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !output.stdout.is_empty(),
            "{family} exited successfully without an observable result"
        );
    }

    let embedded = invoke(env!("CARGO_BIN_EXE_merge_base_topology"), &[]);
    assert!(
        embedded.status.success(),
        "embedded merge-base topology model failed: {}",
        String::from_utf8_lossy(&embedded.stderr)
    );
    assert!(!embedded.stdout.is_empty());
}

#[test]
fn every_external_file_cli_rejects_malformed_json_and_extra_arguments() {
    for (family, binary, filename) in CLIS {
        assert_denied(&invoke_temporary(binary, r#"{"cases":"#), family);
        let file = historical_fixture(filename);
        let valid_path = file.to_str().expect("UTF-8 repository path");
        assert_denied(
            &invoke(binary, &[valid_path, "unexpected-extra-argument"]),
            family,
        );
    }
}

#[test]
fn every_external_file_cli_rejects_a_mutated_historical_semantic_input() {
    for (family, binary, filename) in CLIS {
        let original =
            fs::read_to_string(historical_fixture(filename)).expect("read historical fixture");
        let mut fixture: Value = serde_json::from_str(&original).expect("parse historical fixture");
        match family {
            "ad_hoc_research" => {
                fixture["publication_cases"][0]["authority_current"] = json!(false);
            }
            "containment" => fixture["decision_cases"][0]["authority_current"] = json!(false),
            "adaptive_stress" => fixture["cases"][0]["expected"] = json!("FORGED_SUCCESS"),
            "authority_closure" => fixture["cases"][0]["name"] = json!("forged-unknown-case"),
            "containment_capacity" => {
                fixture["decision_cases"][0]["expected"] = json!("FORGED_SUCCESS");
            }
            "coordination_history" => {
                fixture["cases"][0]["facts"]["authority_current"] = json!(false);
            }
            "github_capability" => fixture["cases"][0]["id"] = json!(0),
            "integration_candidate" => {
                fixture["cases"][0]["expect"] = json!("FORGED_SUCCESS");
            }
            _ => unreachable!("complete staged CLI list"),
        }
        let modified = serde_json::to_string(&fixture).expect("serialize mutated fixture");
        assert_denied(&invoke_temporary(binary, &modified), family);
    }
}

#[test]
fn every_staged_external_file_cli_fails_closed_on_missing_fixture_path() {
    // A missing fixture is not an empty successful oracle evaluation. Check
    // process exit, no positive stdout and an actionable error for all eight.
    let never_created = std::env::temp_dir().join(format!(
        "free-energy-fixture-intentionally-absent-{}-{}.json",
        process::id(),
        NEXT_INPUT.fetch_add(1, Ordering::Relaxed)
    ));
    assert!(!never_created.exists(), "negative input unexpectedly exists");
    for (family, binary, _) in CLIS {
        let output = invoke(
            binary,
            &[never_created.to_str().expect("UTF-8 missing path")],
        );
        assert_denied(&output, family);
    }
}
