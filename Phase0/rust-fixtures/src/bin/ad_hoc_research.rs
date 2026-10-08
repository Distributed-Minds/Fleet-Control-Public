//! Standalone offline CLI for the existing ad-hoc research semantic fixture oracle.
//!
//! Issue #71: expose the 47-case Rust semantic reducer as an executable without
//! introducing Python/Node toolchain requirements or duplicating its logic.
//! Packet SHA-256 canonical identity parity remains an independently open gate.

mod oracle {
    // Reuse the existing independently computed semantic reducers verbatim.
    // The integration-test source also supplies the negative unit tests when
    // this binary is built with `cargo test --all-targets`.
    include!("../../tests/ad_hoc_research.rs");

    pub(super) fn verify(data: &serde_json::Value) -> Result<usize, Vec<String>> {
        validate(data)
    }
}

use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn run() -> Result<usize, String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/ad-hoc-research-spec1.json"));
    if args.next().is_some() {
        return Err("usage: ad_hoc_research [fixture.json]".to_owned());
    }
    let bytes = fs::read_to_string(&input)
        .map_err(|error| format!("cannot read {}: {error}", input.display()))?;
    let value: Value = serde_json::from_str(&bytes)
        .map_err(|error| format!("invalid fixture {}: {error}", input.display()))?;
    oracle::verify(&value).map_err(|errors| errors.join("\n"))
}

fn main() {
    match run() {
        Ok(count) => println!("ad-hoc research fixtures (Rust): {count} passed"),
        Err(error) => {
            eprintln!("FAIL: {error}");
            process::exit(1);
        }
    }
}
