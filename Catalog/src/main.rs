//! Experimental offline catalog admission CLI (NOT the finished catalog).
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("validate")) {
        eprintln!("Usage: free-energy-catalog validate <manifest.json> [manifest.json ...]");
        return ExitCode::FAILURE;
    }
    let paths: Vec<_> = args.collect();
    if paths.is_empty() {
        eprintln!("At least one manifest path is required");
        return ExitCode::FAILURE;
    }
    let mut failed = false;
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(text) => match free_energy_catalog::validate_manifest(&text) {
                Ok(record) => println!("TYPED-BOUNDARY-ONLY {}: {}", record.id, path.to_string_lossy()),
                Err(errors) => {
                    failed = true;
                    for error in errors {
                        eprintln!("{}: {error}", path.to_string_lossy());
                    }
                }
            },
            Err(error) => {
                failed = true;
                eprintln!("{}: {error}", path.to_string_lossy());
            }
        }
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
