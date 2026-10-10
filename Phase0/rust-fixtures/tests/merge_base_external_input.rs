//! Process-boundary checks for the model-only merge-base topology oracle (#71).
//! These test its actual CLI and computed semantics, not just in-process labels.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
const BASELINE: &str = include_str!("../../fixtures/merge-base-topology-spec2.json");

fn run_input(content: &[u8]) -> Output {
    let serial = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-merge-base-cli-{}-{serial}.json",
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("unique exclusive fixture path");
    file.write_all(content)
        .expect("write external fixture bytes");
    drop(file);
    let result = Command::new(env!("CARGO_BIN_EXE_merge_base_topology"))
        .arg(&path)
        .output()
        .expect("run compiled topology oracle");
    fs::remove_file(&path).expect("remove our fixture");
    result
}

fn run_json(value: &Value) -> Output {
    run_input(&serde_json::to_vec(value).expect("serialize test fixture"))
}

fn assert_rejected(output: &Output, reason: &str) {
    assert!(
        !output.status.success(),
        "{reason}: malformed input unexpectedly passed"
    );
    assert!(
        output.stdout.is_empty(),
        "{reason}: rejected input emitted a success record: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !output.stderr.is_empty(),
        "{reason}: rejection must have a diagnostic"
    );
}

#[test]
fn bundled_and_external_baseline_both_execute_the_real_oracle() {
    let program = env!("CARGO_BIN_EXE_merge_base_topology");
    let bundled = Command::new(program)
        .output()
        .expect("execute bundled baseline");
    let external = run_input(BASELINE.as_bytes());
    assert!(bundled.status.success(), "bundled fixture failed");
    assert!(
        external.status.success(),
        "external fixture failed: {}",
        String::from_utf8_lossy(&external.stderr)
    );
    assert_eq!(bundled.stdout, external.stdout);
    assert!(String::from_utf8_lossy(&bundled.stdout).contains("12 read-only model fixtures PASS"));
}

#[test]
fn changing_topology_inputs_without_changing_labels_is_rejected() {
    let mut cases: Value = serde_json::from_str(BASELINE).expect("historical cases");
    let case = cases
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|case| case["name"] == "unique-complete")
        .expect("unique-complete baseline");
    case["complete"] = json!(false);
    assert_rejected(&run_json(&cases), "incomplete history claiming unique base");

    let mut cases: Value = serde_json::from_str(BASELINE).expect("historical cases");
    let case = cases
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|case| case["name"] == "unique-complete")
        .expect("unique-complete baseline");
    case["bases"] = json!([]);
    assert_rejected(&run_json(&cases), "no common ancestor claiming unique base");

    let mut cases: Value = serde_json::from_str(BASELINE).expect("historical cases");
    let case = cases
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|case| case["name"] == "multiple-complete")
        .expect("multiple-complete baseline");
    case["bases"].as_array_mut().unwrap().reverse();
    let output = run_json(&cases);
    assert!(
        output.status.success(),
        "equivalent reordered bases should be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn external_cli_rejects_corrupt_bytes_and_wrong_shape_without_success() {
    for (label, input) in [
        ("truncated JSON", &b"["[..]),
        ("invalid UTF-8", &b"[\xff]"[..]),
        ("wrong root type", &b"{}"[..]),
        ("empty case collection", &b"[]"[..]),
        ("unknown case member", &b"[{\"unexpected\":true}]"[..]),
    ] {
        assert_rejected(&run_input(input), label);
    }
}

#[test]
fn external_cli_rejects_self_referential_cross_case_witnesses() {
    let baseline: Value = serde_json::from_str(BASELINE).expect("historical fixture");
    for (name, field) in [
        ("multiple-reordered", "same_set_as"),
        ("multi-virtual-a-reordered", "same_computation_as"),
        ("multi-virtual-version-drift", "different_computation_from"),
    ] {
        let mut cases = baseline.clone();
        let case = cases
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|case| case["name"] == name)
            .expect("cross-case historical fixture");
        // Preserve the expected verdict and all evidence except its witness.
        case[field] = json!(name);
        let output = run_json(&cases);
        assert_rejected(&output, "self-referential cross-case witness");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("requires a distinct witness case"),
            "{name}/{field}: {:?}",
            output.stderr
        );
    }
}

#[test]
fn external_cli_rejects_missing_file_and_extra_arguments() {
    let program = env!("CARGO_BIN_EXE_merge_base_topology");
    let output = Command::new(program)
        .arg("does-not-exist-topology-fixture-728590.json")
        .output()
        .expect("run compiled topology oracle");
    assert_rejected(&output, "missing fixture file");

    let output = Command::new(program)
        .args(["fixture-one.json", "fixture-two.json"])
        .output()
        .expect("run compiled topology oracle");
    assert_rejected(&output, "unexpected second CLI argument");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("usage:"),
        "extra argument should explain usage"
    );
}
