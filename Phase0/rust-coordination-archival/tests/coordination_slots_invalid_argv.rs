//! Process-level argv acceptance for the read-only Phase0 slot selector.
//! Invalid native bytes must fail through SLOT_UNKNOWN, never a Rust panic.

#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_coordination_slots");
const SLOT_A: &str =
    "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=A\nCOORDINATION_STATE=ACTIVE\nCOORDINATION_EPOCH=1\n";
const SLOT_B: &str =
    "FLEET_COORDINATION_V1\nCOORDINATION_SLOT=B\nCOORDINATION_STATE=STANDBY\nCOORDINATION_EPOCH=0\n";

#[test]
fn non_utf8_in_every_slot_argument_fails_closed_without_panic() {
    let scratch = std::env::temp_dir().join(format!(
        "free-energy-slot-argv-{}",
        std::process::id()
    ));
    fs::create_dir_all(&scratch).expect("create isolated fixture directory");
    let first = scratch.join("slot-a-body");
    let second = scratch.join("slot-b-body");
    fs::write(&first, SLOT_A).expect("write valid first slot");
    fs::write(&second, SLOT_B).expect("write valid second slot");

    let baseline: Vec<OsString> = vec![
        "geromet".into(),
        "168".into(),
        "geromet".into(),
        "[fleet-control] coordination".into(),
        first.as_os_str().to_owned(),
        "186".into(),
        "geromet".into(),
        "[fleet-control] coordination".into(),
        second.as_os_str().to_owned(),
    ];

    let positive = Command::new(BIN)
        .args(&baseline)
        .output()
        .expect("execute valid selector");
    assert_eq!(positive.status.code(), Some(0), "{positive:?}");
    assert!(positive.stderr.is_empty(), "{positive:?}");
    assert_eq!(
        String::from_utf8(positive.stdout).expect("UTF-8 advisory"),
        "ACTIVE_SLOT=A issue=168 epoch=1; ADVISORY ONLY; verify authoritative current provider state and acquire ownership\n"
    );

    for index in 0..baseline.len() {
        let mut malformed = baseline.clone();
        malformed[index] = OsString::from_vec(vec![b'1', 0xff, b'2']);
        let denied = Command::new(BIN)
            .args(&malformed)
            .output()
            .expect("execute selector with non-UTF-8 argv");
        assert_eq!(denied.status.code(), Some(1), "index={index}: {denied:?}");
        assert!(denied.stdout.is_empty(), "index={index}: {denied:?}");
        assert_eq!(
            String::from_utf8(denied.stderr).expect("UTF-8 denial"),
            "SLOT_UNKNOWN arguments must be valid UTF-8; no coordination write authority\n",
            "index={index}"
        );
    }
    fs::remove_dir_all(&scratch).expect("clean fixture directory");
}
