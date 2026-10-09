//! Black-box exit-status and output contracts for the bounded capacity advisor.
//! The CLI is model-only: an exit-0 HEADROOM is never provider write authority.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_coordination_capacity"))
        .args(args)
        .output()
        .expect("run compiled coordination_capacity binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout must be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr must be UTF-8")
}

#[test]
fn headroom_requires_budget_and_has_one_unambiguous_advisory_record() {
    let out = run(&["2500", "2489", "8", "2"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out),
        "HEADROOM remaining=11; advisory only, verify provider state and ownership\n"
    );
    assert!(stderr(&out).is_empty());
}

#[test]
fn reserve_boundary_and_exhaustion_have_no_success_stdout() {
    for (observed, expected) in [
        (
            "2490",
            "ROLLOVER_REQUIRED remaining=10; stop new work and follow authorized rollover protocol\n",
        ),
        (
            "2499",
            "ROLLOVER_REQUIRED remaining=1; stop new work and follow authorized rollover protocol\n",
        ),
        (
            "2500",
            "EXHAUSTED remaining=0; coordination mutations cannot proceed here\n",
        ),
    ] {
        let out = run(&["2500", observed, "8", "2"]);
        assert_eq!(out.status.code(), Some(1), "count={observed}");
        assert!(stdout(&out).is_empty(), "count={observed}");
        assert_eq!(stderr(&out), expected, "count={observed}");
    }
}

#[test]
fn malformed_and_missing_arguments_fail_closed_without_success_receipts() {
    for args in [
        vec![],
        vec!["2500"],
        vec!["2500", "1000", "8", "2", "extra"],
        vec!["2500", "1000", "0", "2"],
        vec!["0", "0", "1", "0"],
        vec!["2500", "2501", "1", "0"],
        vec!["2500", "1000", "18446744073709551615", "1"],
        vec!["2500", "1000", "18446744073709551616", "1"],
    ] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2), "args={args:?}");
        assert!(stdout(&out).is_empty(), "args={args:?}");
        assert!(
            stderr(&out).starts_with("CAPACITY_UNKNOWN "),
            "args={args:?}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn noncanonical_unsigned_decimal_inputs_are_rejected_at_process_boundary() {
    for wrong in [
        "-1",
        "+1",
        " 2",
        "2 ",
        "1.0",
        "0x10",
        "",
        "18446744073709551616",
    ] {
        let out = run(&["2500", wrong, "8", "2"]);
        assert_eq!(out.status.code(), Some(2), "observed={wrong:?}");
        assert!(stdout(&out).is_empty(), "observed={wrong:?}");
        assert!(
            stderr(&out).contains("CAPACITY_UNKNOWN "),
            "observed={wrong:?}"
        );
    }
}

#[test]
fn increasing_reserve_never_recovers_success_after_rollover_threshold() {
    for reserve in ["2", "3", "10", "500", "18446744073709551615"] {
        let out = run(&["2500", "2490", "8", reserve]);
        assert!(!out.status.success(), "reserve={reserve}");
        assert!(stdout(&out).is_empty(), "reserve={reserve}");
        assert!(
            stderr(&out).contains("ROLLOVER_REQUIRED") || stderr(&out).contains("CAPACITY_UNKNOWN"),
            "reserve={reserve}: {}",
            stderr(&out)
        );
    }
}
