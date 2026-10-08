//! Opt-in Python/Rust differential for the archived 47-case ad-hoc research oracle.
//! The historical Python runner is only an independent migration baseline.
//! The maintained Rust CLI and normal cargo test path do not need Python.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/ad-hoc-research-spec1.json");
static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

fn historical_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 directory")
        .join("scripts/check-ad-hoc-research-fixtures.py")
}

fn fixture_file(fixture: &Value) -> PathBuf {
    let ordinal = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "free-energy-ad-hoc-differential-{}-{ordinal}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create exclusive temporary fixture");
    file.write_all(
        serde_json::to_string(fixture)
            .expect("serialize fixture")
            .as_bytes(),
    )
    .expect("write complete fixture");
    path
}

fn output(command: &mut Command, path: &Path) -> Output {
    command
        .arg(path)
        .output()
        .expect("execute independently maintained fixture runner")
}

fn check_both(fixture: &Value, expected_pass: bool, label: &str) {
    let file = fixture_file(fixture);
    let python = output(&mut Command::new("python3").arg(historical_script()), &file);
    let rust = output(
        &mut Command::new(env!("CARGO_BIN_EXE_ad_hoc_research")),
        &file,
    );
    fs::remove_file(&file).expect("remove temporary input");

    for (implementation, result) in [("historical Python", python), ("compiled Rust", rust)] {
        assert_eq!(
            result.status.success(),
            expected_pass,
            "{implementation} {label}: stdout={} stderr={}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        if expected_pass {
            assert!(
                String::from_utf8_lossy(&result.stdout).contains("47 passed"),
                "{implementation} omitted full-family success: {label}"
            );
        } else {
            assert!(
                !String::from_utf8_lossy(&result.stdout).contains("47 passed"),
                "{implementation} emitted a false 47-case success: {label}"
            );
        }
    }
}

fn case_mut<'a>(fixture: &'a mut Value, group: &str, id: &str) -> &'a mut Value {
    fixture[group]
        .as_array_mut()
        .expect("historical family")
        .iter_mut()
        .find(|case| case["id"] == id)
        .expect("historical named case")
}

/// Call the unchanged historical Python reducer on semantic inputs, not on
/// the fixture's expected result. This answer is used as a new oracle below.
fn python_answer(fixture: &Value, group: &str, id: &str) -> Value {
    const SCRIPT: &str = r#"
import json, runpy, sys
model = runpy.run_path(sys.argv[1])
data = json.load(sys.stdin)
group, case_id = sys.argv[2], sys.argv[3]
handlers = {
    "publication_cases": "publication",
    "identity_cases": "identity",
    "source_cases": "source",
    "recovery_cases": "recovery",
    "concurrency_cases": "concurrent_publication",
    "packet_field_cases": "packet_fields",
}
case = next(c for c in data[group] if c["id"] == case_id)
expanded = model["expand_packet_refs"](case, data["packet_templates"])
answer = model[handlers[group]](expanded)
json.dump(answer, sys.stdout, sort_keys=True, ensure_ascii=False)
"#;
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(SCRIPT)
        .arg(historical_script())
        .arg(group)
        .arg(id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("historical Python 3 required for opt-in differential");
    child
        .stdin
        .take()
        .expect("Python stdin")
        .write_all(
            serde_json::to_string(fixture)
                .expect("serialize facts")
                .as_bytes(),
        )
        .expect("provide historical semantic facts");
    let result = child
        .wait_with_output()
        .expect("historical model completes");
    assert!(
        result.status.success(),
        "historical reducer {group}/{id} failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).expect("historical reducer produces JSON")
}

#[test]
#[ignore = "opt-in historical Python 3 baseline; Rust tooling remains Python-free"]
fn forty_seven_cases_and_six_causal_mutations_match_historical_semantics() {
    let original: Value = serde_json::from_str(BASELINE).expect("source fixture JSON");
    check_both(&original, true, "unaltered 47-case baseline");

    let mutations = [
        (
            "publication_cases",
            "complete-absent-current-authority",
            "authority_current",
        ),
        (
            "identity_cases",
            "changed-content-new-identity",
            "retry_packet_ref",
        ),
        (
            "source_cases",
            "default-authority-first",
            "default_head_resolved",
        ),
        (
            "recovery_cases",
            "cutoff-before-create",
            "authority_current",
        ),
        ("concurrency_cases", "two-empty-blind-create", "strategy"),
        (
            "packet_field_cases",
            "required-handoff-fields-preserved",
            "discovery_vocabulary",
        ),
    ];
    for (group, id, field) in mutations {
        let mut changed = original.clone();
        let case = case_mut(&mut changed, group, id);
        let prior = case["expected"].clone();
        if group == "packet_field_cases" {
            case.as_object_mut().expect("case object").remove(field);
        } else {
            case[field] = match group {
                "publication_cases" | "recovery_cases" | "source_cases" => json!(false),
                "identity_cases" => json!("base"),
                "concurrency_cases" => json!("serialized"),
                _ => unreachable!("fixed mutation families"),
            };
        }
        let independently_computed = python_answer(&changed, group, id);
        assert_ne!(
            independently_computed, prior,
            "changed semantic fact must change the historical decision: {group}/{id}"
        );
        check_both(&changed, false, id);
        case_mut(&mut changed, group, id)["expected"] = independently_computed;
        check_both(&changed, true, id);
    }
}
