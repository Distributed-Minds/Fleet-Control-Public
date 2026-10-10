//! Exercise the actual digest binary, not just its fixture model.

use std::process::Command;

const BINARY: &str = env!("CARGO_BIN_EXE_integration_candidate_digest");

#[test]
fn historical_fixture_still_emits_four_sha256_identities() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/integration-candidate-v1.json");
    let output = Command::new(BINARY)
        .arg(path)
        .output()
        .expect("run compiled digest binary");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 identities");
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 4, "{stdout:?}");
    for line in lines {
        let (name, digest) = line.split_once(": ").expect("named SHA-256 identity");
        assert!(!name.is_empty());
        assert_eq!(digest.len(), 64, "{line:?}");
        assert!(digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    }
    assert_eq!(
        String::from_utf8(output.stderr).expect("UTF-8 advisory"),
        "Digest identity only; does not authorize Git mutations\n"
    );
}

#[cfg(unix)]
#[test]
fn malformed_os_arguments_fail_without_panic_or_identity_output() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let invalid = OsString::from_vec(vec![b'x', 0xff, b'y']);
    for args in [
        vec![invalid.clone()],
        vec![OsString::from("valid.json"), invalid.clone()],
    ] {
        let output = Command::new(BINARY)
            .args(&args)
            .output()
            .expect("run compiled digest with malformed OS argv");
        assert!(!output.status.success(), "invalid argv succeeded");
        assert!(output.stdout.is_empty(), "invalid argv printed SHA-256 identities");
        let stderr = String::from_utf8(output.stderr).expect("safe UTF-8 diagnostic");
        assert_eq!(stderr, "FAIL: non-UTF-8 CLI argument\n");
        assert!(!stderr.contains("panicked"));
    }
}
