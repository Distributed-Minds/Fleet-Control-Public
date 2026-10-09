//! Process-level contracts for the offline catalog explanation CLI (#60).
//!
//! These tests execute the compiled Rust binary rather than reusing its helper
//! functions. An unsuccessful multi-file invocation must not leak a partial
//! machine-readable success payload to downstream automation.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");
const VELOREN: &str = include_str!("../projects/veloren.json");
const BIN: &str = env!("CARGO_BIN_EXE_catalog_explain");
static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let index = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-catalog-explain-process-{}-{index}",
            process::id()
        ));
        fs::create_dir(&path).expect("exclusive test directory");
        Self(path)
    }

    fn file(&self, name: &str, data: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, data).expect("write isolated manifest");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn invoke(paths: &[&Path]) -> Output {
    Command::new(BIN)
        .args(paths)
        .output()
        .expect("execute compiled catalog_explain")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn exact_one_mib_valid_input_and_ordinary_pilot_are_admitted() {
    let root = Scratch::new();
    let valid = root.file("small.json", LUANTI);
    let baseline = invoke(&[&valid]);
    assert!(baseline.status.success(), "{}", stderr(&baseline));

    // JSON may contain insignificant trailing whitespace. The declared exact
    // byte limit must remain admissible, not accidentally off by one.
    let exact = format!("{LUANTI}{}", " ".repeat(1024 * 1024 - LUANTI.len()));
    let padded = root.file("exact.json", &exact);
    let output = invoke(&[&padded]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
    let projects: Value = serde_json::from_slice(&output.stdout).expect("valid explanation JSON");
    assert_eq!(projects[0]["project_id"], "engine/luanti");
}

#[test]
fn oversized_manifest_suppresses_success_for_entire_explain_batch() {
    let root = Scratch::new();
    let valid = root.file("valid.json", OPENRA);
    let huge = root.0.join("oversized.json");
    let file = fs::File::create(&huge).expect("create sparse fixture");
    file.set_len(1024 * 1024 + 1).expect("set oversized length");
    drop(file);

    let output = invoke(&[&valid, &huge]);
    assert!(
        !output.status.success(),
        "oversized file accepted: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "oversized tail leaked a success record"
    );
    assert!(stderr(&output).contains("manifest exceeds 1 MiB input limit"));
}

#[test]
fn invalid_utf8_manifest_does_not_leak_partial_batch_explanation() {
    let root = Scratch::new();
    let valid = root.file("valid.json", VELOREN);
    let invalid = root.0.join("invalid-utf8.json");
    fs::write(&invalid, [0xff, 0xfe, 0x00]).expect("write invalid UTF-8 fixture");
    let output = invoke(&[&valid, &invalid]);
    assert!(!output.status.success(), "invalid UTF-8 accepted");
    assert!(
        output.stdout.is_empty(),
        "invalid UTF-8 tail leaked positive output"
    );
    assert!(stderr(&output).contains("invalid utf-8 sequence"));
}

#[test]
fn accepted_batch_is_order_independent_and_does_not_upgrade_rights() {
    let root = Scratch::new();
    let luanti = root.file("luanti.json", LUANTI);
    let openra = root.file("openra.json", OPENRA);
    let veloren = root.file("veloren.json", VELOREN);

    let first = invoke(&[&veloren, &luanti, &openra]);
    let second = invoke(&[&openra, &veloren, &luanti]);
    assert!(first.status.success(), "{}", stderr(&first));
    assert!(second.status.success(), "{}", stderr(&second));
    assert!(first.stderr.is_empty(), "{}", stderr(&first));
    assert_eq!(first.stdout, second.stdout, "manifest order changed JSON");
    let projects: Value = serde_json::from_slice(&first.stdout).expect("one JSON array");
    let array = projects
        .as_array()
        .expect("JSON array, not per-line fragments");
    assert_eq!(array.len(), 3);
    assert_eq!(array[0]["project_id"], "engine/luanti");
    assert_eq!(array[1]["project_id"], "engine/openra");
    assert_eq!(array[2]["project_id"], "game/veloren");
    for project in array {
        assert_eq!(
            project["explanation_schema"],
            "free-energy.catalog-explanation/v1"
        );
        assert_eq!(
            project["rights"]["free_energy_redistribution_authorized"],
            false
        );
        assert_eq!(project["play"]["free_energy_hosted_download"], false);
        assert_eq!(project["record_review"]["status"], "DRAFT");
    }
}

#[test]
fn later_invalid_manifest_suppresses_every_success_record() {
    let root = Scratch::new();
    let valid = root.file("good.json", LUANTI);
    let invalid = root.file("bad.json", "{}");
    let baseline = invoke(&[&valid]);
    assert!(
        baseline.status.success(),
        "positive control: {}",
        stderr(&baseline)
    );
    assert!(!baseline.stdout.is_empty());

    let result = invoke(&[&valid, &invalid]);
    assert!(!result.status.success(), "invalid tail was accepted");
    assert!(result.stdout.is_empty(), "partial JSON escaped on stdout");
    assert!(!stderr(&result).is_empty(), "missing validation diagnosis");
}

#[test]
fn duplicate_identity_and_unreadable_tail_both_fail_atomically() {
    let root = Scratch::new();
    let first = root.file("first.json", OPENRA);
    let copy = root.file("duplicate.json", OPENRA);
    let duplicate = invoke(&[&first, &copy]);
    assert!(!duplicate.status.success());
    assert!(
        duplicate.stdout.is_empty(),
        "duplicate emitted partial JSON"
    );
    assert!(stderr(&duplicate).contains("duplicate project ID"));

    let absent = root.0.join("missing.json");
    let missing = invoke(&[&first, &absent]);
    assert!(!missing.status.success());
    assert!(
        missing.stdout.is_empty(),
        "missing tail emitted partial JSON"
    );
    assert!(!stderr(&missing).is_empty());

    let directory = invoke(&[&first, &root.0]);
    assert!(!directory.status.success());
    assert!(
        directory.stdout.is_empty(),
        "directory tail emitted partial JSON"
    );
    assert!(stderr(&directory).contains("expected a regular non-symlink JSON file"));
}

#[cfg(unix)]
#[test]
fn symlinked_manifest_does_not_bypass_atomic_failure() {
    use std::os::unix::fs::symlink;
    let root = Scratch::new();
    let valid = root.file("source.json", VELOREN);
    let link = root.0.join("alias.json");
    symlink(&valid, &link).expect("create input symlink");
    let result = invoke(&[&valid, &link]);
    assert!(!result.status.success());
    assert!(
        result.stdout.is_empty(),
        "symlinked tail emitted partial JSON"
    );
    assert!(stderr(&result).contains("symlink path component"));
}
