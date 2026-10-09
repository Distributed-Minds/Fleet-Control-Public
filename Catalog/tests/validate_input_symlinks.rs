//! Black-box regression for catalog validator symlink input admission.
//! No claim of race-free filesystem confinement or source-rights clearance.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "free-energy-validate-symlinks-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create scratch directory");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(paths: &[&Path]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"));
    cmd.arg("validate");
    cmd.args(paths);
    cmd.output().expect("run compiled catalog validator")
}

fn rejected_without_success(output: &Output, diagnostic: &str) {
    assert!(
        !output.status.success(),
        "symlink unexpectedly admitted: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "partial success record leaked: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "missing {diagnostic:?} diagnostic: {output:?}"
    );
}

#[test]
fn valid_regular_file_and_directory_still_admit_existing_pilot() {
    let scratch = Scratch::new();
    let folder = scratch.child("projects");
    fs::create_dir(&folder).expect("create real project directory");
    let regular = folder.join("luanti.json");
    fs::write(&regular, include_str!("../projects/luanti.json"))
        .expect("write reviewed pilot fixture");

    for input in [&regular as &Path, &folder as &Path] {
        let output = run(&[input]);
        assert!(output.status.success(), "real input rejected: {output:?}");
        assert!(
            output.stderr.is_empty(),
            "unexpected diagnostics: {output:?}"
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 1);
    }
}

#[test]
fn symlinked_manifest_cannot_pass_even_inside_otherwise_valid_batch() {
    let scratch = Scratch::new();
    let original = scratch.child("original.json");
    let alias = scratch.child("alias.json");
    fs::write(&original, include_str!("../projects/luanti.json"))
        .expect("write valid source manifest");
    symlink(&original, &alias).expect("create symlink to legitimate source");

    rejected_without_success(&run(&[&original, &alias]), "symlinked manifest prohibited");
    assert!(
        original.exists(),
        "rejection must not mutate original manifest"
    );
}

#[test]
fn symlinked_project_directory_cannot_redirect_catalog_scan() {
    let scratch = Scratch::new();
    let real = scratch.child("real-projects");
    let alias = scratch.child("redirected-projects");
    fs::create_dir(&real).expect("create real project directory");
    let original = real.join("luanti.json");
    fs::write(&original, include_str!("../projects/luanti.json"))
        .expect("write valid external source manifest");
    symlink(&real, &alias).expect("create directory symlink");

    rejected_without_success(&run(&[&alias]), "symlinked project directory prohibited");
    assert!(
        original.exists(),
        "rejection must not delete source manifest"
    );
}

#[test]
fn direct_symlinked_child_in_real_directory_is_rejected_atomically() {
    let scratch = Scratch::new();
    let dir = scratch.child("projects");
    fs::create_dir(&dir).expect("create real project directory");
    let original = dir.join("a-luanti.json");
    let alias = dir.join("z-alias.json");
    fs::write(&original, include_str!("../projects/luanti.json"))
        .expect("write valid source manifest");
    symlink(&original, &alias).expect("create symlinked direct JSON child");

    rejected_without_success(&run(&[&dir]), "symlinked manifest prohibited");
    assert!(original.exists(), "rejection must not alter source");
}
