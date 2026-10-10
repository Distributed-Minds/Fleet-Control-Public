//! Compiled-process input boundary regressions for the model-only capability CLI.
//! Passing these fixtures does not authorize live GitHub mutations.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BASELINE: &str = include_str!("../../fixtures/github-capability-spec5.json");
const LIMIT: usize = 8 * 1024 * 1024;
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

fn unique_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "free-energy-ghcap-input-{}-{}-{label}",
        process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ))
}

fn write_file(label: &str, bytes: &[u8]) -> PathBuf {
    let path = unique_path(label);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique temporary fixture");
    file.write_all(bytes).expect("write temporary fixture");
    path
}

fn execute(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_github_capability"))
        .arg(path)
        .output()
        .expect("execute compiled GitHub capability CLI")
}

fn assert_rejected(path: &Path, reason: &str) {
    let output = execute(path);
    assert!(!output.status.success(), "unexpected success: {output:?}");
    assert!(output.stdout.is_empty(), "failure emitted success output");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(reason),
        "expected {reason:?}, got {output:?}"
    );
}

#[test]
fn external_fixture_is_bounded_and_regular_without_losing_historical_baseline() {
    let valid = write_file("valid.json", BASELINE.as_bytes());
    let original = execute(&valid);
    assert!(original.status.success(), "baseline failed: {original:?}");
    assert!(
        String::from_utf8_lossy(&original.stdout).contains("28 checked"),
        "baseline success count absent"
    );

    // A file exactly at the documented cap remains acceptable; an extra byte
    // fails without ever parsing or reporting fixture success.
    let mut boundary = BASELINE.as_bytes().to_vec();
    assert!(boundary.len() < LIMIT);
    boundary.resize(LIMIT, b' ');
    let at_limit = write_file("at-limit.json", &boundary);
    let exact = execute(&at_limit);
    assert!(exact.status.success(), "exact limit rejected: {exact:?}");
    fs::remove_file(&at_limit).expect("remove exact-bound fixture");

    let too_large = unique_path("too-large.json");
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&too_large)
        .expect("create oversized sparse fixture");
    file.set_len((LIMIT + 1) as u64)
        .expect("extend sparse fixture");
    drop(file);
    assert_rejected(&too_large, "exceeds");
    fs::remove_file(&too_large).expect("remove oversized fixture");

    let directory = unique_path("directory");
    fs::create_dir(&directory).expect("create test directory");
    assert_rejected(&directory, "regular file");
    fs::remove_dir(&directory).expect("remove directory");

    #[cfg(unix)]
    {
        let link = unique_path("symlink");
        std::os::unix::fs::symlink(&valid, &link).expect("create symlink leaf");
        assert_rejected(&link, "regular file");
        fs::remove_file(&link).expect("remove symlink");
    }

    fs::remove_file(&valid).expect("remove valid fixture");
}

#[test]
fn malformed_fixture_path_cannot_inject_a_forged_success_line() {
    let path = write_file("bad\nPASS-28-checked.json", b"{");
    assert_rejected(&path, "invalid fixture");
    let output = execute(&path);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "unescaped path split diagnostic");
    assert!(stderr.contains(r"\n"), "newline must be debug-escaped");
    fs::remove_file(&path).expect("remove malicious path fixture");
}
