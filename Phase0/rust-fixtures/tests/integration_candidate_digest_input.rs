//! End-to-end admission tests for the compiled SHA-256 candidate-digest CLI.
//! These check process exit/output, not merely calls to its private reader.
//! Passing them does not establish provider authority or full Python/Rust parity.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_integration_candidate_digest");
const MAX_FIXTURE_BYTES: usize = 8 * 1024 * 1024;
static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Phase0 parent")
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn historical() -> PathBuf {
    root().join("Phase0/fixtures/integration-candidate-v1.json")
}

fn invoke(path: Option<&Path>) -> Output {
    let mut command = Command::new(BIN);
    command.current_dir(root());
    if let Some(path) = path {
        command.arg(path);
    }
    command.output().expect("run compiled digest executable")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_denied(output: &Output) {
    assert!(!output.status.success(), "unexpected admission success");
    assert!(
        output.stdout.is_empty(),
        "failed digest input emitted candidate receipts: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stderr(output).contains("FAIL:"),
        "failure must have an explicit diagnostic: {}",
        stderr(output)
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let generation = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-digest-cli-{}-{}",
            std::process::id(),
            generation
        ));
        fs::create_dir(&path).expect("create isolated fixture directory");
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn compiled_cli_preserves_exact_historical_receipts_at_eight_mib_boundary() {
    let original = invoke(None);
    assert!(original.status.success(), "{}", stderr(&original));
    assert_eq!(original.stdout.iter().filter(|&&byte| byte == b'\n').count(), 4);
    assert!(stderr(&original).contains("does not authorize Git mutations"));

    let direct = invoke(Some(&historical()));
    assert!(direct.status.success(), "{}", stderr(&direct));
    assert_eq!(direct.stdout, original.stdout);

    let scratch = Scratch::new();
    let mut padded = fs::read(historical()).expect("read checked-in historical fixture");
    assert!(padded.len() < MAX_FIXTURE_BYTES);
    padded.resize(MAX_FIXTURE_BYTES, b' ');
    let at_limit = scratch.path("exactly-eight-mib.json");
    fs::write(&at_limit, &padded).expect("write exact boundary input");
    let boundary = invoke(Some(&at_limit));
    assert!(boundary.status.success(), "{}", stderr(&boundary));
    assert_eq!(boundary.stdout, original.stdout);
}

#[test]
fn compiled_cli_denies_oversized_and_malformed_input_without_partial_receipts() {
    let scratch = Scratch::new();
    let oversized = scratch.path("over-limit.json");
    fs::File::create(&oversized)
        .expect("create sparse test file")
        .set_len(MAX_FIXTURE_BYTES as u64 + 1)
        .expect("extend sparse file beyond the bound");
    let oversized_result = invoke(Some(&oversized));
    assert_denied(&oversized_result);
    assert!(stderr(&oversized_result).contains("maximum input size"));

    let invalid_utf8 = scratch.path("invalid-utf8.json");
    fs::write(&invalid_utf8, [0xff, 0xfe]).expect("write invalid UTF-8");
    assert_denied(&invoke(Some(&invalid_utf8)));

    let malformed = scratch.path("trailing-garbage.json");
    let mut bytes = fs::read(historical()).expect("read historical fixture");
    bytes.extend_from_slice(b"unexpected");
    fs::write(&malformed, bytes).expect("write non-JSON tail");
    assert_denied(&invoke(Some(&malformed)));
}

#[test]
fn compiled_cli_denies_nonregular_path_and_wrong_arity() {
    let scratch = Scratch::new();
    let directory_result = invoke(Some(&scratch.0));
    assert_denied(&directory_result);
    assert!(stderr(&directory_result).contains("regular non-symlink"));

    #[cfg(unix)]
    {
        let alias = scratch.path("alias.json");
        std::os::unix::fs::symlink(historical(), &alias).expect("create symlink");
        let symlink_result = invoke(Some(&alias));
        assert_denied(&symlink_result);
        assert!(stderr(&symlink_result).contains("regular non-symlink"));
    }

    let extra = Command::new(BIN)
        .args([historical().as_os_str(), historical().as_os_str()])
        .current_dir(root())
        .output()
        .expect("run compiled digest with two positional inputs");
    assert_eq!(extra.status.code(), Some(2));
    assert!(extra.stdout.is_empty());
    assert!(stderr(&extra).contains("usage:"));
}
