//! Compiled-process regressions for the all_oracles diagnostic trust boundary.
//! Untrusted CLI arguments may not forge PASS lines or terminal output.

use std::process::Command;

const RUNNER: &str = env!("CARGO_BIN_EXE_all_oracles");

#[test]
fn malicious_arguments_are_escaped_in_real_stderr() {
    let injected = "bad\nPASS forged\r\u{1b}[2K\u{202e}";
    for args in [vec!["--family", injected], vec![injected]] {
        let output = Command::new(RUNNER)
            .args(&args)
            .output()
            .expect("run compiled fixture aggregate");
        assert!(
            !output.status.success(),
            "malicious CLI argument was accepted"
        );
        assert!(output.stdout.is_empty(), "failed CLI emitted PASS stdout");
        let diagnostic = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
        assert!(diagnostic.starts_with("FAIL: "), "{diagnostic:?}");
        assert!(diagnostic.contains(r"\nPASS forged"), "{diagnostic:?}");
        assert!(diagnostic.contains(r"\r"), "{diagnostic:?}");
        assert!(diagnostic.contains(r"\u{1b}"), "{diagnostic:?}");
        assert!(diagnostic.contains(r"\u{202e}"), "{diagnostic:?}");
        assert!(!diagnostic.contains("\nPASS forged"));
        assert!(!diagnostic.contains('\r'));
        assert!(!diagnostic.contains('\u{1b}'));
        assert!(!diagnostic.contains('\u{202e}'));
    }
}

#[test]
fn oversized_argument_does_not_leak_an_unbounded_diagnostic() {
    let attack = format!("--unknown-{}-FORGED_SUCCESS", "x".repeat(2048));
    let output = Command::new(RUNNER)
        .arg(attack)
        .output()
        .expect("run compiled fixture aggregate with long argument");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
    assert!(diagnostic.contains("unknown or repeated argument"));
    assert!(!diagnostic.contains("FORGED_SUCCESS"), "{diagnostic:?}");
    assert!(
        diagnostic.len() < 1024,
        "unbounded stderr: {}",
        diagnostic.len()
    );
}

#[cfg(unix)]
#[test]
fn invalid_utf8_os_arguments_fail_without_a_panic_or_success_output() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let invalid = OsString::from_vec(vec![b'x', 0xff, b'y']);
    for args in [
        vec![invalid.clone()],
        vec![OsString::from("--family"), invalid.clone()],
        vec![OsString::from("--root"), invalid.clone()],
    ] {
        let output = Command::new(RUNNER)
            .args(&args)
            .output()
            .expect("execute compiled runner with non-UTF-8 OS arguments");
        assert!(!output.status.success(), "invalid OS argv was accepted");
        assert!(output.stdout.is_empty(), "failed argv emitted PASS stdout");
        let diagnostic = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
        assert!(
            diagnostic.starts_with("FAIL: non-UTF-8 CLI argument\n"),
            "{diagnostic:?}"
        );
        assert!(!diagnostic.contains("panicked"), "{diagnostic:?}");
        assert_eq!(diagnostic.lines().count(), 2, "{diagnostic:?}");
    }
}
