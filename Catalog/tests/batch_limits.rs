//! Compiled-process batch admission regressions for the offline catalog CLI.
//! All fixtures are disposable local files; this never writes the repository.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const PER_MANIFEST_LIMIT: usize = 1024 * 1024;
const PILOT: &str = include_str!("../projects/luanti.json");
static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

struct Sandbox(PathBuf);

impl Sandbox {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-catalog-batch-{label}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create disposable project directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn add(&self, index: usize, pad_to_limit: bool) {
        let key = "\"id\": \"engine/luanti\"";
        assert!(PILOT.contains(key), "pilot schema unexpectedly changed");
        let mut json = PILOT.replace(key, &format!("\"id\": \"engine/pilot-{index:03}\""));
        if pad_to_limit {
            assert!(json.len() < PER_MANIFEST_LIMIT);
            json.push_str(&" ".repeat(PER_MANIFEST_LIMIT - json.len()));
        }
        fs::write(self.0.join(format!("{index:03}.json")), json)
            .expect("write disposable valid manifest");
    }

    fn validate(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_free-energy-catalog"))
            .arg("validate")
            .arg(self.path())
            .output()
            .expect("execute compiled catalog validator")
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn validate_rejects_257th_manifest_without_any_partial_success_stdout() {
    let dir = Sandbox::new("count");
    for index in 0..257 {
        dir.add(index, false);
    }
    let output = dir.validate();
    assert!(!output.status.success(), "257 manifests must fail");
    assert!(
        output.stdout.is_empty(),
        "invalid batch emitted success rows"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("256 manifests"),
        "missing count-limit diagnostic: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn validate_admits_16_mib_then_rejects_17_mib_without_partial_success() {
    let dir = Sandbox::new("bytes");
    for index in 0..16 {
        dir.add(index, true);
    }
    let accepted = dir.validate();
    assert!(
        accepted.status.success(),
        "exact 16 MiB should pass: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert_eq!(
        accepted
            .stdout
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
        16
    );

    dir.add(16, true);
    let rejected = dir.validate();
    assert!(!rejected.status.success(), "17 MiB must fail");
    assert!(
        rejected.stdout.is_empty(),
        "invalid batch emitted success rows"
    );
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("16 MiB"),
        "missing total-input diagnostic: {}",
        String::from_utf8_lossy(&rejected.stderr)
    );
}
