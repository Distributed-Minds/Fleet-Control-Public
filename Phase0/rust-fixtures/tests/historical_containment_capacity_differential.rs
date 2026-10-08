//! Issue #71: historical containment-capacity Python/Rust differential evidence.
//! The maintained acceptance path is compiled Rust. Python is invoked ONLY by
//! an explicitly ignored archival comparison, never by ordinary Rust tests.
//! This is an offline fixture oracle, not live containment authority.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/containment-capacity-spec2.json");
const RUST_BINARY: &str = env!("CARGO_BIN_EXE_containment_capacity");
const PYTHON_WRAPPER: &str = r#"
import importlib.util
import json
import pathlib
import sys
spec = importlib.util.spec_from_file_location("archived_capacity", sys.argv[1])
model = importlib.util.module_from_spec(spec)
spec.loader.exec_module(model)
model.DATA = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
model.main()
"#;
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Input(PathBuf);

impl Input {
    fn new(value: &Value) -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-capacity-historical-{}-{id}.json",
            process::id()
        ));
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("reserve exclusive fixture path");
        output
            .write_all(
                serde_json::to_string(value)
                    .expect("serializable fixture")
                    .as_bytes(),
            )
            .expect("write complete fixture");
        drop(output);
        Self(path)
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("tracked 17-case fixture parses")
}

fn rust(path: &Path) -> Output {
    Command::new(RUST_BINARY)
        .arg(path)
        .output()
        .expect("execute compiled Rust capacity oracle")
}

fn python(path: &Path) -> Output {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate in Phase0")
        .join("scripts/check-containment-capacity-fixtures.py");
    Command::new("python3")
        .arg("-c")
        .arg(PYTHON_WRAPPER)
        .arg(script)
        .arg(path)
        .output()
        .expect("execute opt-in archived Python fixture oracle")
}

fn named_case<'a>(root: &'a mut Value, family: &str, id: &str) -> &'a mut Value {
    root[family]
        .as_array_mut()
        .expect("tracked family array")
        .iter_mut()
        .find(|case| case["id"] == id)
        .expect("tracked scenario identity")
}

fn assert_rejected(output: &Output, label: &str) {
    assert!(
        !output.status.success(),
        "{label}: unexpectedly accepted stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "{label}: invalid fixture produced positive stdout"
    );
}

fn changed_causal_facts() -> Vec<(&'static str, Value)> {
    let mut decision = baseline();
    named_case(&mut decision, "decision_cases", "backlog-cannot-renew")
        ["renewal_evidence_current"] = json!(true);

    let mut workload = baseline();
    named_case(&mut workload, "workload_cases", "ordinary-load")
        ["service_capacity"][0] = json!(0);

    let mut planning = baseline();
    named_case(
        &mut planning,
        "planning_cases",
        "rare-event-false-positive-heavy",
    )["false_positive_num"] = json!(0);

    vec![
        ("renewal evidence changed", decision),
        ("service capacity changed", workload),
        ("false positive rate changed", planning),
    ]
}

#[test]
fn compiled_rust_admits_baseline_but_rejects_changed_causal_facts() {
    let original = Input::new(&baseline());
    let result = rust(&original.0);
    assert!(
        result.status.success(),
        "17-case Rust baseline failed: {result:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).trim(),
        "containment-capacity fixtures (Rust): 17 passed"
    );

    for (label, value) in changed_causal_facts() {
        let input = Input::new(&value);
        assert_rejected(&rust(&input.0), label);
    }
}

#[test]
fn rust_closes_historical_sparse_expected_and_identity_loopholes() {
    let mut sparse = baseline();
    named_case(&mut sparse, "workload_cases", "ordinary-load")
        ["expected"]
        .as_object_mut()
        .expect("expected workload metrics")
        .remove("final_adjudication_backlog");
    let input = Input::new(&sparse);
    assert_rejected(&rust(&input.0), "sparse expected workload");

    let mut renamed = baseline();
    named_case(&mut renamed, "decision_cases", "backlog-cannot-renew")["id"] =
        json!("invented-historical-case");
    let input = Input::new(&renamed);
    assert_rejected(&rust(&input.0), "replaced historical case");
}

#[test]
#[ignore = "explicit archival Python 3 comparison; never an installed runtime dependency"]
fn archived_python_and_compiled_rust_agree_on_causal_changes_and_record_deltas() {
    let original = Input::new(&baseline());
    assert!(rust(&original.0).status.success(), "Rust historical baseline");
    assert!(
        python(&original.0).status.success(),
        "Python historical baseline"
    );

    // Both independently compute the consequence; expected labels stay fixed.
    for (label, value) in changed_causal_facts() {
        let input = Input::new(&value);
        assert_rejected(&rust(&input.0), label);
        assert_rejected(&python(&input.0), label);
    }

    // The original Python runner checked only provided expected fields; the
    // Rust migration intentionally requires essential workload measurements.
    let mut sparse = baseline();
    named_case(&mut sparse, "workload_cases", "ordinary-load")
        ["expected"]
        .as_object_mut()
        .expect("workload metrics")
        .remove("final_adjudication_backlog");
    let input = Input::new(&sparse);
    assert_rejected(&rust(&input.0), "sparse expected workload");
    assert!(
        python(&input.0).status.success(),
        "archived checker unexpectedly enforced missing expected field"
    );

    // Python also permitted replacing historical scenario names while still
    // reporting a passing same-count family. Rust must preserve identities.
    let mut renamed = baseline();
    named_case(&mut renamed, "decision_cases", "backlog-cannot-renew")["id"] =
        json!("invented-historical-case");
    let input = Input::new(&renamed);
    assert_rejected(&rust(&input.0), "replaced historical scenario");
    assert!(
        python(&input.0).status.success(),
        "archived checker unexpectedly rejected renamed scenario"
    );
}
