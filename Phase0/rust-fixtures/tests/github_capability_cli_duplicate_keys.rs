//! Compiled-process admission regressions. Successful fixture validation is not
//! proof of live provider permission or a full independent semantic oracle.

use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::{self, Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const BASELINE: &str = include_str!("../../fixtures/github-capability-spec5.json");
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

fn run_bytes(bytes: &[u8]) -> Output {
    let path = std::env::temp_dir().join(format!(
        "free-energy-ghcap-duplicate-{}-{}.json",
        process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("create unique temporary fixture");
    file.write_all(bytes).expect("write temporary fixture");
    drop(file);

    let output = Command::new(env!("CARGO_BIN_EXE_github_capability"))
        .arg(&path)
        .output()
        .expect("run compiled GitHub capability CLI");
    fs::remove_file(&path).expect("clean up temporary fixture");
    output
}

#[test]
fn compiled_cli_rejects_duplicate_decoded_json_members_at_every_depth() {
    let positive = run_bytes(BASELINE.as_bytes());
    assert!(positive.status.success(), "baseline failed: {positive:?}");

    let value: Value = serde_json::from_str(BASELINE).expect("canonical baseline JSON");
    let canonical = serde_json::to_string(&value).expect("serialize original fixture");
    assert!(canonical.contains(r#""lineage":"#));

    let top_level = format!(
        r#"{{"spec":5,{}"#,
        canonical.strip_prefix('{').expect("fixture object")
    );
    let nested = canonical.replacen(r#""lineage":"#, r#""lineage":"unknown","lineage":"#, 1);
    let escaped_alias =
        canonical.replacen(r#""lineage":"#, r#""\u006cineage":"unknown","lineage":"#, 1);

    for (label, raw) in [
        ("top-level", top_level),
        ("nested", nested),
        ("unicode-escaped-alias", escaped_alias),
    ] {
        let output = run_bytes(raw.as_bytes());
        assert!(!output.status.success(), "{label} admitted: {output:?}");
        assert!(output.stdout.is_empty(), "{label} emitted a success line");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("duplicate JSON object key"),
            "{label}: missing duplicate-member diagnostic: {output:?}"
        );
    }
}
