//! Historical process-level differential for merge-base topology migration (#71).
//!
//! The preserved Python script remains evidence, not a Rust runtime dependency.
//! Shared decisions must agree; stricter Rust input admission is recorded as an
//! explicit divergence rather than silently called semantic parity.

use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/merge-base-topology-spec2.json");
const RUST_BIN: &str = env!("CARGO_BIN_EXE_merge_base_topology");
const PYTHON_SHIM: &str = r#"
import pathlib
import runpy
import sys
source, fixture = sys.argv[1:3]
module = runpy.run_path(source, run_name='archived_merge_base_model')
module['main'].__globals__['FIXTURES'] = pathlib.Path(fixture)
module['main']()
"#;
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

struct FixtureFile(PathBuf);

impl FixtureFile {
    fn new(value: &Value) -> Self {
        let number = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-merge-base-differential-{}-{number}.json",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("exclusive temporary fixture input");
        file.write_all(
            serde_json::to_string(value)
                .expect("serialize fixture")
                .as_bytes(),
        )
        .expect("write fixture before running checkers");
        Self(path)
    }
}

impl Drop for FixtureFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn clean_git_environment(command: &mut Command) {
    // The historical checker constructs actual Git DAGs. Inherited host Git
    // object/index/config variables must not redirect its disposable repos.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
}

fn run_rust(path: &Path) -> Output {
    let mut command = Command::new(RUST_BIN);
    command.arg(path);
    clean_git_environment(&mut command);
    command.output().expect("run compiled Rust topology oracle")
}

fn run_python(path: &Path) -> Output {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 directory")
        .join("scripts/check-merge-base-topology-fixtures.py");
    let mut command = Command::new(OsStr::new("python3"));
    command.arg("-c").arg(PYTHON_SHIM).arg(script).arg(path);
    clean_git_environment(&mut command);
    command
        .output()
        .expect("run preserved historical Python checker")
}

fn fixture() -> Value {
    serde_json::from_str(BASELINE).expect("tracked twelve-case topology fixture")
}

fn case<'a>(root: &'a mut Value, name: &str) -> &'a mut Value {
    root.as_array_mut()
        .expect("top-level cases")
        .iter_mut()
        .find(|value| value["name"] == name)
        .expect("historical case")
}

fn assert_exit(
    label: &str,
    name: &str,
    result: &Output,
    expected_success: bool,
    expected_count: usize,
) {
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(
        result.status.success(),
        expected_success,
        "{label}: {name} unexpected exit: stdout={stdout} stderr={stderr}"
    );
    let marker = if name == "Python" {
        format!("PASS: {expected_count} modeled cases + 4 real Git topology probes")
    } else {
        "merge-base-topology: 12 read-only model fixtures PASS".to_owned()
    };
    assert_eq!(
        stdout.contains(&marker),
        expected_success,
        "{label}: {name} success marker does not match status: {stdout}"
    );
}

fn check_rust(label: &str, value: &Value, accepted: bool) {
    let input = FixtureFile::new(value);
    let count = value.as_array().expect("cases").len();
    assert_exit(label, "Rust", &run_rust(&input.0), accepted, count);
}

fn check_both(label: &str, value: &Value, python_accept: bool, rust_accept: bool) {
    let input = FixtureFile::new(value);
    let count = value.as_array().expect("cases").len();
    assert_exit(label, "Python", &run_python(&input.0), python_accept, count);
    assert_exit(label, "Rust", &run_rust(&input.0), rust_accept, count);
}

#[test]
fn rust_recomputes_topology_and_rejects_mutated_witnesses() {
    let original = fixture();
    check_rust("baseline", &original, true);

    let mut wrong_topology = original.clone();
    case(&mut wrong_topology, "unique-complete")["expect"] = json!("NONE");
    check_rust("wrong topology", &wrong_topology, false);

    let mut wrong_virtual = original.clone();
    case(&mut wrong_virtual, "multi-no-virtual")["expect_virtual"] = json!("SUPPORTED");
    check_rust("wrong virtual-base result", &wrong_virtual, false);

    let mut bad_same_set = original.clone();
    case(&mut bad_same_set, "multiple-reordered")["same_set_as"] = json!("unique-complete");
    check_rust("wrong set identity", &bad_same_set, false);

    let mut bad_same_computation = original.clone();
    case(&mut bad_same_computation, "multi-virtual-a-reordered")["same_computation_as"] =
        json!("multi-virtual-version-drift");
    check_rust("wrong computation identity", &bad_same_computation, false);

    let mut bad_drift = original.clone();
    case(&mut bad_drift, "same-heads-replaced-history")["stale_against"] =
        json!({"history_view": "hv:replace:b", "bases": ["b1"]});
    check_rust("false history drift", &bad_drift, false);

    let mut duplicate_base = original.clone();
    case(&mut duplicate_base, "multiple-complete")["bases"] = json!(["b2", "b1", "b1"]);
    check_rust("duplicate base identity", &duplicate_base, false);

    let mut truncated = original;
    truncated.as_array_mut().expect("cases").pop();
    check_rust("truncated historical family", &truncated, false);
}

#[test]
#[ignore = "opt-in historical Python/Rust process comparison; Python is not a Rust runtime dependency"]
fn historical_python_and_rust_expose_shared_semantics_and_strict_admission_differences() {
    let original = fixture();
    check_both("historical baseline", &original, true, true);

    let mut wrong_topology = original.clone();
    case(&mut wrong_topology, "unique-complete")["expect"] = json!("NONE");
    check_both("wrong topology", &wrong_topology, false, false);

    let mut wrong_virtual = original.clone();
    case(&mut wrong_virtual, "multi-no-virtual")["expect_virtual"] = json!("SUPPORTED");
    check_both("wrong virtual result", &wrong_virtual, false, false);

    let mut bad_same_set = original.clone();
    case(&mut bad_same_set, "multiple-reordered")["same_set_as"] = json!("unique-complete");
    check_both("wrong set witness", &bad_same_set, false, false);

    let mut bad_same_computation = original.clone();
    case(&mut bad_same_computation, "multi-virtual-a-reordered")["same_computation_as"] =
        json!("multi-virtual-version-drift");
    check_both(
        "wrong computation witness",
        &bad_same_computation,
        false,
        false,
    );

    let mut false_stale = original.clone();
    case(&mut false_stale, "same-heads-replaced-history")["stale_against"] =
        json!({"history_view": "hv:replace:b", "bases": ["b1"]});
    check_both("false stale-history witness", &false_stale, false, false);

    // Historical Python deduplicates bases and never pins case cardinality.
    // Rust deliberately rejects these while retaining the original valid suite.
    let mut duplicate_base = original.clone();
    case(&mut duplicate_base, "multiple-complete")["bases"] = json!(["b2", "b1", "b1"]);
    check_both("duplicate base", &duplicate_base, true, false);

    let mut truncated = original.clone();
    truncated.as_array_mut().expect("cases").pop();
    check_both("truncated case family", &truncated, true, false);

    // Python's self-comparison succeeds mechanically; Rust requires a distinct
    // provenance witness, rather than accepting self-attestation as evidence.
    let mut self_witness = original;
    case(&mut self_witness, "multiple-reordered")["same_set_as"] = json!("multiple-reordered");
    check_both("self-witness", &self_witness, true, false);
}
