//! Compiled process regressions for malformed Unix OS argv in the
//! standalone integration-candidate CLI. Invalid argv must never panic,
//! run a fixture, or emit canonical bytes.

#![cfg(unix)]

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

const RUNNER: &str = env!("CARGO_BIN_EXE_integration_candidate");

#[test]
fn invalid_utf8_os_arguments_fail_without_panic_or_output() {
    let invalid = OsString::from_vec(vec![b'x', 0xff, b'y']);
    for args in [
        vec![invalid.clone()],
        vec![OsString::from("--emit-canonical"), invalid.clone()],
        vec![
            OsString::from("--emit-canonical"),
            OsString::from("normal-two-parent"),
            invalid.clone(),
        ],
    ] {
        let output = Command::new(RUNNER)
            .args(&args)
            .output()
            .expect("run compiled candidate CLI with malformed OS argv");
        assert_eq!(output.status.code(), Some(2), "malformed OS argv exit");
        assert!(output.stdout.is_empty(), "invalid argv emitted fixture/canonical bytes");
        let stderr = String::from_utf8(output.stderr).expect("stable UTF-8 failure diagnostic");
        assert_eq!(stderr, "FAIL: non-UTF-8 CLI argument\n");
        assert!(!stderr.contains("panicked"), "{stderr:?}");
    }
}
