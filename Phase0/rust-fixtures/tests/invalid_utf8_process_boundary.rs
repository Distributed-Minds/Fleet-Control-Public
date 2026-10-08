//! Black-box encoding-boundary regression for every external-file fixture CLI.
//! Invalid UTF-8 is never valid JSON, and no executable should print a success
//! claim or exit successfully when the historical fixture bytes are unreadable.
//! This tests process behavior, not production authority or Python parity.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{self, Command};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

const FILE_CLIS: [(&str, &str); 8] = [
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
    (
        "integration_candidate",
        env!("CARGO_BIN_EXE_integration_candidate"),
    ),
];

#[test]
fn all_eight_external_fixture_clis_reject_non_utf8_input_without_success_output() {
    for (family, binary) in FILE_CLIS {
        // A raw invalid byte and an incomplete multibyte sequence must each
        // fail before a semantic fixture can be accepted as historical proof.
        for (label, data) in [
            ("invalid-byte", &[b'{', 0xff, b'}'][..]),
            ("truncated-sequence", &[b'{', 0xc3][..]),
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
                "{family}/{label} accepted invalid UTF-8; stdout: {}",
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
