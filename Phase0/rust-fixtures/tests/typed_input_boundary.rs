//! Process-level negative acceptance for Rust Phase0 fixture CLI inputs.
//!
//! Unlike malformed-syntax smoke tests, these are well-formed JSON documents
//! with structurally incorrect top-level types and missing required fields.
//! Every staged external-input family must reject them without a success line.
//! These tests do not establish full historical Python/Rust semantic parity.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INPUT: AtomicUsize = AtomicUsize::new(0);

const FAMILY_BINARIES: [(&str, &str); 9] = [
    (
        "containment",
        env!("CARGO_BIN_EXE_free-energy-phase0-fixtures"),
    ),
    ("adaptive_stress", env!("CARGO_BIN_EXE_adaptive_stress")),
    ("ad_hoc_research", env!("CARGO_BIN_EXE_ad_hoc_research")),
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
    (
        "integration_candidate_digest",
        env!("CARGO_BIN_EXE_integration_candidate_digest"),
    ),
];

fn with_fixture_input(input: &str, f: impl FnOnce(&str) -> Output) -> Output {
    let serial = NEXT_INPUT.fetch_add(1, Ordering::Relaxed);
    let path: PathBuf = std::env::temp_dir().join(format!(
        "free-energy-typed-input-{}-{serial}.json",
        process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique fixture input");
    file.write_all(input.as_bytes())
        .expect("write complete fixture bytes");
    drop(file);
    let result = f(path.to_str().expect("UTF-8 temporary path"));
    fs::remove_file(path).expect("remove fixture after execution");
    result
}

fn invoke(binary: &str, contents: &str) -> Output {
    with_fixture_input(contents, |path| {
        Command::new(binary)
            .arg(path)
            .output()
            .expect("execute compiled fixture CLI")
    })
}

fn assert_rejected_without_success(family: &str, input: &str) {
    let (label, binary) = FAMILY_BINARIES
        .iter()
        .find(|(name, _)| *name == family)
        .expect("known CLI family");
    let result = invoke(binary, input);
    assert!(
        !result.status.success(),
        "{label} accepted structurally invalid input {input:?}: {result:?}"
    );
    assert!(
        result.stdout.is_empty(),
        "{label} printed misleading positive output for {input:?}: {result:?}"
    );
    assert!(
        !result.stderr.is_empty(),
        "{label} failed without diagnostic for {input:?}"
    );
}

#[test]
fn all_external_fixture_clis_reject_wrong_root_json_types() {
    // All compiled fixture CLIs, including the digest identity helper,
    // must interpret these as invalid input, not an empty successful fixture.
    for (family, _) in FAMILY_BINARIES {
        for input in ["null", "[]", "0", "true", "\"fixture\"", "[{}]"] {
            assert_rejected_without_success(family, input);
        }
    }
}

#[test]
fn all_external_fixture_clis_reject_empty_and_wrong_shape_objects() {
    // Well-formed objects can still be dangerously incomplete; validators
    // must demand their canonical family and required-case identities.
    for (family, _) in FAMILY_BINARIES {
        for input in ["{}", "{\"cases\":[]}", "{\"unexpected\":\"only\"}"] {
            assert_rejected_without_success(family, input);
        }
    }
}
