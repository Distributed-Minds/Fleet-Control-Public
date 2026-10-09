//! Process-level regression: untrusted fixture filenames must not forge log
//! lines or terminal control sequences on Rust oracle failure paths (#71).
//! This is output-integrity protection, not evidence of semantic parity.
#![cfg(unix)]

use std::process::{self, Command};

const PATH_REPORTING_CLIS: [(&str, &str); 7] = [
    (
        "containment",
        env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"),
    ),
    ("ad_hoc_research", env!("CARGO_BIN_EXE_ad_hoc_research")),
    ("adaptive_stress", env!("CARGO_BIN_EXE_adaptive_stress")),
    (
        "containment_capacity",
        env!("CARGO_BIN_EXE_containment_capacity"),
    ),
    (
        "coordination_history",
        env!("CARGO_BIN_EXE_coordination_history"),
    ),
    ("github_capability", env!("CARGO_BIN_EXE_github_capability")),
    (
        "merge_base_topology",
        env!("CARGO_BIN_EXE_merge_base_topology"),
    ),
];

#[test]
fn missing_fixture_path_cannot_inject_status_lines_or_terminal_controls() {
    // This non-existent pathname contains three classes of dangerous log
    // syntax. No filesystem writes are needed for the negative process test.
    let crafted = std::env::temp_dir().join(format!(
        "free-energy-escaped-path-{}-\nFORGED-PASS: all tests accepted\r\x1b[2J.json",
        process::id()
    ));
    assert!(!crafted.exists(), "crafted fixture unexpectedly exists");

    for (family, binary) in PATH_REPORTING_CLIS {
        let output = Command::new(binary)
            .arg(&crafted)
            .output()
            .expect("execute compiled fixture oracle");
        assert!(
            !output.status.success(),
            "{family}: missing fixture unexpectedly passed"
        );
        assert!(
            output.stdout.is_empty(),
            "{family}: failed fixture emitted success stdout"
        );
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 Rust diagnostic");
        assert!(
            !stderr.contains('\r') && !stderr.contains('\x1b'),
            "{family}: raw carriage return or ANSI control in diagnostic: {stderr:?}"
        );
        assert_eq!(
            stderr.lines().count(),
            1,
            "{family}: filename forged additional log lines: {stderr:?}"
        );
        assert!(
            stderr.contains("FORGED-PASS")
                && stderr.contains("\\nFORGED-PASS")
                && stderr.contains("\\r"),
            "{family}: filename not represented in escaped, auditable form: {stderr:?}"
        );
    }
}
