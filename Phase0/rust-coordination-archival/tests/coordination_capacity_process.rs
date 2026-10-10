//! Process-level regressions for the installed coordination-capacity model (#22).
//!
//! These invoke the actual compiled Rust CLI. Unit-level `assess` results are
//! insufficient if exit codes or stdout incorrectly signal authorization.
//! All inputs remain caller-supplied: even a successful advisory response is
//! NOT evidence of trusted GitHub state, an ownership lease, or write authority.

use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_coordination_capacity");

fn invoke(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("execute the compiled coordination_capacity binary")
}

fn expect_success(args: &[&str], expected: &str) {
    let output = invoke(args);
    assert_eq!(output.status.code(), Some(0), "args={args:?}: {output:?}");
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("CLI stdout is UTF-8");
    assert_eq!(stdout, format!("{expected}\n"));
    assert!(
        stdout.contains("advisory only, verify provider state and ownership"),
        "a success must not read like a provider authorization"
    );
}

fn expect_failure(args: &[&str], exit: i32, diagnostic_prefix: &str) {
    let output = invoke(args);
    assert_eq!(
        output.status.code(),
        Some(exit),
        "wrong disposition for args={args:?}: {output:?}"
    );
    assert!(
        output.stdout.is_empty(),
        "failure leaked a machine-readable success record: {output:?}"
    );
    let stderr = String::from_utf8(output.stderr).expect("CLI stderr is UTF-8");
    assert!(
        stderr.starts_with(diagnostic_prefix),
        "missing diagnostic {diagnostic_prefix:?} for args={args:?}: {stderr:?}"
    );
}

#[test]
fn safe_capacity_is_advisory_only_and_has_no_stderr() {
    expect_success(
        &["2500", "2489", "8", "2"],
        "HEADROOM remaining=11; advisory only, verify provider state and ownership",
    );
    expect_success(
        &["2500", "0", "1", "0"],
        "HEADROOM remaining=2500; advisory only, verify provider state and ownership",
    );
}

#[test]
fn rollover_threshold_and_exhaustion_fail_without_stdout() {
    expect_failure(
        &["2500", "2490", "8", "2"],
        1,
        "ROLLOVER_REQUIRED remaining=10;",
    );
    expect_failure(
        &["2500", "2499", "1", "0"],
        1,
        "ROLLOVER_REQUIRED remaining=1;",
    );
    expect_failure(&["2500", "2500", "1", "0"], 1, "EXHAUSTED remaining=0;");
}

#[test]
fn invalid_inputs_never_masquerade_as_rollover_or_success() {
    for args in [
        ["0", "0", "1", "0"],
        ["2500", "2501", "1", "0"],
        ["2500", "0", "0", "0"],
        ["18446744073709551615", "0", "18446744073709551615", "1"],
    ] {
        expect_failure(&args, 2, "CAPACITY_UNKNOWN ");
    }
}

#[test]
fn malformed_unsigned_numbers_fail_closed_at_the_real_argv_boundary() {
    for bad in [
        "-1",
        "+1",
        " 1",
        "1 ",
        "1.0",
        "0x10",
        "",
        "18446744073709551616",
    ] {
        expect_failure(&["2500", bad, "2", "5"], 2, "CAPACITY_UNKNOWN ");
    }
    expect_failure(&[], 2, "CAPACITY_UNKNOWN usage:");
    expect_failure(&["2500"], 2, "CAPACITY_UNKNOWN usage:");
    expect_failure(
        &["2500", "0", "1", "0", "untrusted-extra-argument"],
        2,
        "CAPACITY_UNKNOWN usage:",
    );
}

#[test]
fn adding_consumption_never_recovers_successful_capacity() {
    // Model monotonicity already has exhaustive unit coverage. Verify the
    // subprocess status transition at meaningful adjacent boundary values.
    expect_success(
        &["2500", "2477", "6", "16"],
        "HEADROOM remaining=23; advisory only, verify provider state and ownership",
    );
    expect_failure(
        &["2500", "2478", "6", "16"],
        1,
        "ROLLOVER_REQUIRED remaining=22;",
    );
    expect_failure(&["2500", "2500", "6", "16"], 1, "EXHAUSTED remaining=0;");
}

#[cfg(unix)]
#[test]
fn non_utf8_argv_is_a_typed_denial_in_every_numeric_position() {
    use std::os::unix::ffi::OsStringExt;

    let baseline = ["2500", "0", "1", "0"];
    for index in 0..baseline.len() {
        let mut argv: Vec<std::ffi::OsString> = baseline
            .iter()
            .map(|value| std::ffi::OsString::from(*value))
            .collect();
        argv[index] = std::ffi::OsString::from_vec(vec![b'1', 0xff, b'2']);
        let output = Command::new(BIN)
            .args(&argv)
            .output()
            .expect("execute compiled advisor with non-UTF-8 argv");

        assert_eq!(output.status.code(), Some(2), "index={index}");
        assert!(output.stdout.is_empty(), "index={index}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stderr).expect("UTF-8 diagnostic"),
            "CAPACITY_UNKNOWN arguments must be valid UTF-8; no write authority\n",
            "index={index}"
        );
    }
}
