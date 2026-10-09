//! Black-box encoding-boundary regression for every external-file fixture CLI.
//! Invalid UTF-8 is never valid JSON, and no executable should print a success
//! claim or exit successfully when the historical fixture bytes are unreadable.
//! This tests process behavior, not production authority or Python parity.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{self, Command};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

const FILE_CLIS: [(&str, &str); 10] = [
    (
        "containment",
        env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"),
    ),
    ("ad_hoc_research", env!("CARGO_BIN_EXE_ad_hoc_research")),
    ("adaptive_stress", env!("CARGO_BIN_EXE_adaptive_stress")),
    ("authority_closure", env!("CARGO_BIN_EXE_authority_closure")),
    (
        "containment_capacity",
        env!("CARGO_BIN_EXE_containment_capacity"),
    ),
    (
        "coordination_history",
        env!("CARGO_BIN_EXE_coordination_history"),
    ),
    ("github_capability", env!("CARGO_BIN_EXE_github_capability")),
    ("merge_base_topology", env!("CARGO_BIN_EXE_merge_base_topology")),
    (
        "integration_candidate",
        env!("CARGO_BIN_EXE_integration_candidate"),
    ),
    (
        "integration_candidate_digest",
        env!("CARGO_BIN_EXE_integration_candidate_digest"),
    ),
];

#[test]
fn every_external_fixture_cli_rejects_malformed_input_without_false_success() {
    for (family, binary) in FILE_CLIS {
        // Cover all ten file-driven binaries, including merge-base-topology.
        // Reject broken encoding, incomplete JSON and trailing junk before
        // accepting any fixture as historical evidence.
        for (label, data) in [
            ("invalid-byte", &[b'{', 0xff, b'}'][..]),
            ("truncated-sequence", &[b'{', 0xc3][..]),
            ("truncated-json", &b"{\"cases\":["[..]),
            ("trailing-garbage", &b"{}unexpected"[..]),
        ] {
            let index = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "free-energy-utf8-boundary-{}-{index}-{family}-{label}.json",
                process::id()
            ));
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .expect("create unique test input");
            file.write_all(data).expect("write raw non-UTF-8 bytes");
            drop(file);

            let output = Command::new(binary)
                .arg(&path)
                .output()
                .expect("execute compiled Rust oracle");
            fs::remove_file(&path).expect("remove owned temporary fixture");

            assert!(
                !output.status.success(),
                "{family}/{label} accepted malformed fixture bytes; stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(
                output.stdout.is_empty(),
                "{family}/{label} emitted false-positive stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(
                !output.stderr.is_empty(),
                "{family}/{label} must explain the failed input boundary"
            );
        }
    }
}
