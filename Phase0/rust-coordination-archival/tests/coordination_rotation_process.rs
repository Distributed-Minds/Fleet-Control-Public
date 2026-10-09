//! Black-box contracts for the advisory two-slot rotation executable (#22).
//!
//! Exit 0 is reserved for a bounded headroom advisory, never a provider
//! authorization. Every rollover, malformed observation, ambiguous election,
//! draining standby, or exhausted slot must keep success stdout empty.

use std::process::{Command, Output};

const ROTATION: &str = env!("CARGO_BIN_EXE_coordination_rotation");

fn run(args: &[&str]) -> Output {
    Command::new(ROTATION)
        .args(args)
        .output()
        .expect("execute compiled coordination_rotation")
}

fn snapshot<'a>(
    active_count: &'a str,
    other_state: &'a str,
    other_count: &'a str,
    pending: &'a str,
    reserve: &'a str,
) -> [&'a str; 10] {
    [
        "2500",
        "2000",
        active_count,
        "3",
        other_state,
        other_count,
        "2",
        "500",
        pending,
        reserve,
    ]
}

fn assert_exact(args: &[&str], exit: i32, stdout: &str, stderr: &str) {
    let result = run(args);
    assert_eq!(result.status.code(), Some(exit), "args={args:?}");
    assert_eq!(
        String::from_utf8(result.stdout).expect("UTF-8 stdout"),
        stdout,
        "args={args:?}"
    );
    assert_eq!(
        String::from_utf8(result.stderr).expect("UTF-8 stderr"),
        stderr,
        "args={args:?}"
    );
}

fn assert_denied(args: &[&str], exit: i32, diagnostic: &str) {
    let result = run(args);
    assert_eq!(result.status.code(), Some(exit), "args={args:?}");
    assert!(
        result.stdout.is_empty(),
        "unsafe success output for {args:?}"
    );
    let stderr = String::from_utf8(result.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.starts_with(diagnostic),
        "wrong diagnostic for {args:?}: {stderr:?}"
    );
    assert!(
        stderr.contains("no coordination write authority")
            || stderr.contains("ADVISORY ONLY")
            || stderr.contains("do not rotate")
            || stderr.contains("no further writes"),
        "denial must not masquerade as authorization: {stderr:?}"
    );
}

#[test]
fn bounded_headroom_prints_exactly_one_non_authoritative_success_record() {
    assert_exact(
        &snapshot("1800", "STANDBY", "300", "8", "2"),
        0,
        "HEADROOM remaining=700; ADVISORY ONLY; revalidate provider state and ownership\n",
        "",
    );
}

#[test]
fn rotation_and_exhaustion_require_explicit_nonzero_dispositions() {
    assert_exact(
        &snapshot("1990", "STANDBY", "300", "8", "2"),
        1,
        "",
        "ROTATION_CANDIDATE remaining=510; ADVISORY ONLY; obtain trusted slot/currentness and rotation authority\n",
    );
    assert_exact(
        &snapshot("2500", "STANDBY", "300", "8", "2"),
        1,
        "",
        "EXHAUSTED remaining=0; no further writes to this slot\n",
    );
}

#[test]
fn draining_or_overfull_standby_never_emits_a_rotation_receipt() {
    for (state, count) in [("DRAINING", "300"), ("STANDBY", "501")] {
        assert_exact(
            &snapshot("1990", state, count, "8", "2"),
            1,
            "",
            "STANDBY_NOT_READY remaining=510; do not rotate; preserve capacity and request authorized archival maintenance\n",
        );
    }

    // A nominal STANDBY below standby_max still cannot accept a batch that
    // reaches the successor's proactive switch threshold.
    let mut almost_full = snapshot("1990", "STANDBY", "1990", "8", "2");
    almost_full[7] = "1999";
    assert_denied(&almost_full, 1, "STANDBY_NOT_READY remaining=510;");
}

#[test]
fn two_active_slots_and_stale_epochs_fail_without_stdout() {
    let mut ambiguous = snapshot("1990", "ACTIVE", "300", "8", "2");
    assert_denied(
        &ambiguous,
        2,
        "CAPACITY_UNKNOWN two ACTIVE slots require trusted highest-epoch election first;",
    );
    ambiguous[4] = "STANDBY";
    ambiguous[6] = "3";
    assert_denied(
        &ambiguous,
        2,
        "CAPACITY_UNKNOWN active slot epoch is not newer than the other slot;",
    );
}

#[test]
fn malformed_arguments_and_overflow_fail_at_real_process_boundary() {
    assert_denied(&[], 2, "CAPACITY_UNKNOWN usage:");
    assert_denied(&["2500"], 2, "CAPACITY_UNKNOWN usage:");
    let mut extra = snapshot("1800", "STANDBY", "300", "8", "2").to_vec();
    extra.push("untrusted");
    assert_denied(&extra, 2, "CAPACITY_UNKNOWN usage:");

    for malformed in ["-1", "+1", " 1", "1 ", "1.0", "0x10", ""] {
        let mut args = snapshot("1800", "STANDBY", "300", "8", "2");
        args[8] = malformed;
        assert_denied(&args, 2, "CAPACITY_UNKNOWN pending-upper-bound");
    }
    let mut overflow = snapshot("1800", "STANDBY", "300", "8", "2");
    overflow[8] = "18446744073709551616";
    assert_denied(&overflow, 2, "CAPACITY_UNKNOWN pending-upper-bound exceeds");
    overflow[8] = "18446744073709551615";
    assert_denied(
        &overflow,
        2,
        "CAPACITY_UNKNOWN pending and reserve budget overflow;",
    );
}

#[test]
fn raising_the_declared_batch_budget_never_recovers_success() {
    for pending in ["8", "10", "100", "2000", "18446744073709551615"] {
        let args = snapshot("1990", "STANDBY", "300", pending, "2");
        let result = run(&args);
        assert!(
            !result.status.success(),
            "budget restored success: {pending}"
        );
        assert!(
            result.stdout.is_empty(),
            "budget emitted success text: {pending}"
        );
    }
}
