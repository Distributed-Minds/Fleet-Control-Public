//! Compiled CLI regression: symlinked ancestor rejection must be atomic (#60).
//! Test-only input directories; no provider operations or release authority.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_catalog_explain");
const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "free-energy-ancestor-cli-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("exclusive scratch directory");
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn invoke(root: &Path, paths: &[&Path]) -> Output {
    Command::new(BIN)
        .current_dir(root)
        .args(paths)
        .output()
        .expect("run compiled catalog CLI")
}

#[test]
fn symlinked_ancestor_is_rejected_without_partial_json() {
    let scratch = Scratch::new();
    let real = scratch.0.join("real");
    fs::create_dir(&real).expect("real input directory");
    fs::write(real.join("luanti.json"), LUANTI).expect("valid Luanti manifest");
    fs::write(scratch.0.join("openra.json"), OPENRA).expect("valid OpenRA manifest");
    symlink(&real, scratch.0.join("alias")).expect("symlinked parent");
    let valid = invoke(
        &scratch.0,
        &[Path::new("openra.json"), Path::new("real/luanti.json")],
    );
    assert!(
        valid.status.success(),
        "positive control failed: {}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let projects: serde_json::Value =
        serde_json::from_slice(&valid.stdout).expect("valid full JSON");
    assert_eq!(projects.as_array().map(Vec::len), Some(2));

    // An older leaf-only symlink_metadata guard would accept both aliases:
    // they end at a regular file, but their parent is a directory symlink.
    for tail in ["alias/luanti.json", "alias/../real/luanti.json"] {
        let failure = invoke(&scratch.0, &[Path::new("openra.json"), Path::new(tail)]);
        assert!(!failure.status.success(), "accepted {tail}");
        assert!(failure.stdout.is_empty(), "partial success JSON for {tail}");
        assert!(
            String::from_utf8_lossy(&failure.stderr).contains("symlink path component"),
            "wrong diagnostic for {tail}: {}",
            String::from_utf8_lossy(&failure.stderr)
        );
    }
}
