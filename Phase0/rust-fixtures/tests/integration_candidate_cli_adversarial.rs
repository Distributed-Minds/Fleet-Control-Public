//! Real process-boundary negative controls for the Rust integration-candidate
//! oracle (#71). These exercise computed fixture semantics, not just deserialization
//! or string comparison with the fixture's claimed expected dispositions.
//! This does not establish canonical SHA-256 identity or historical Python parity.

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BASELINE: &str = include_str!("../../fixtures/integration-candidate-v1.json");
const EXECUTABLE: &str = env!("CARGO_BIN_EXE_integration_candidate");

static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

struct TemporaryFixture {
    path: PathBuf,
}

impl TemporaryFixture {
    fn create(value: &Value) -> Self {
        let index = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-candidate-negative-{}-{index}.json",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("create isolated fixture");
        let bytes = serde_json::to_vec(value).expect("serialize controlled mutation");
        file.write_all(&bytes).expect("write controlled fixture");
        Self { path }
    }
}

impl Drop for TemporaryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn invoke(fixture: &Value) -> Output {
    let input = TemporaryFixture::create(fixture);
    Command::new(EXECUTABLE)
        .arg(&input.path)
        .output()
        .expect("execute compiled Rust candidate validator")
}

fn baseline() -> Value {
    serde_json::from_str(BASELINE).expect("parse checked-in historical fixture")
}

fn assert_rejected(label: &str, input: &Value) {
    let output = invoke(input);
    assert!(
        !output.status.success(),
        "{label}: accepted mutated semantic input: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "{label}: a rejected fixture emitted a positive oracle result"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("FAIL:"),
        "{label}: missing deterministic failure marker: {output:?}"
    );
}

#[test]
fn exact_historical_input_passes_without_claiming_hash_parity() {
    let output = invoke(&baseline());
    assert!(
        output.status.success(),
        "historical fixture rejected: {output:?}"
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 CLI result");
    assert!(stdout.contains("6 passed"), "{stdout}");
    assert!(
        stdout.contains("SHA-256 identity parity NOT checked"),
        "{stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected diagnostics: {output:?}"
    );
}

#[test]
fn semantic_mutations_fail_at_the_compiled_cli_boundary() {
    let original = baseline();

    let mut changed = original.clone();
    changed["cases"][0]["candidate"]["parents"][0] =
        json!("2222222222222222222222222222222222222222");
    assert_rejected("normal parent no longer matches target", &changed);

    let mut changed = original.clone();
    changed["cases"][1]["candidate"]["parents"][0] =
        json!("1111111111111111111111111111111111111111");
    assert_rejected("reversal silently destroyed", &changed);

    let mut changed = original.clone();
    changed["cases"][2]["constructor_support"]
        .as_array_mut()
        .expect("constructor support")
        .push(json!(3));
    assert_rejected("unsupported parent count becomes supported", &changed);

    let mut changed = original.clone();
    changed["cases"][3]["candidate"]["metadata"]["message"] = json!("different bytes\n");
    assert_rejected("compatibility claimed after metadata drift", &changed);

    let mut changed = original.clone();
    changed["stale_head_cases"][0]["live_target"] =
        changed["stale_head_cases"][0]["predicted_target"].clone();
    assert_rejected("moved target falsely treated as stale", &changed);

    let mut changed = original.clone();
    changed["stale_head_cases"][1]["live_source"] =
        changed["stale_head_cases"][1]["predicted_source"].clone();
    assert_rejected("moved source falsely treated as stale", &changed);

    let mut changed = original.clone();
    changed["cases"][0]["candidate"]["target_commit"] =
        json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    assert_rejected("uppercase Git object ID", &changed);

    let mut changed = original.clone();
    changed["cases"][0]["candidate"]["constructor_version"] = json!("");
    assert_rejected("blank constructor identity", &changed);

    let mut changed = original.clone();
    changed["cases"][0]["expect"] = json!("UNSUPPORTED_PARENT_CARDINALITY");
    assert_rejected(
        "expected output rewritten without semantic change",
        &changed,
    );
}

#[test]
fn fixture_structure_and_cli_errors_cannot_return_success() {
    let original = baseline();
    let mut changed = original.clone();
    changed["cases"].as_array_mut().expect("cases").remove(0);
    assert_rejected("required baseline removed", &changed);

    let mut changed = original.clone();
    changed["cases"][1]["name"] = changed["cases"][0]["name"].clone();
    assert_rejected("duplicate case name", &changed);

    let mut changed = original.clone();
    changed["cases"][0]["candidate"]["unexpected_authority"] = json!(true);
    assert_rejected("unknown candidate field", &changed);

    let mut changed = original;
    changed["digest"] = json!("sha1");
    assert_rejected("wrong digest declaration", &changed);

    let output = Command::new(EXECUTABLE)
        .args(["missing.json", "unexpected.json"])
        .output()
        .expect("execute invalid invocation");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
