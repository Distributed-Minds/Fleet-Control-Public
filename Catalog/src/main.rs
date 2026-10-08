//! Experimental offline catalog admission CLI (NOT the finished catalog).
use std::{collections::HashSet, env, fs, process::ExitCode};

/// Validate readable manifests as one unit. Rejected input cannot emit
/// a partial positive catalog admission result.
fn validate_loaded<'a, I>(sources: I) -> Result<Vec<(String, String)>, Vec<String>>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut ids = HashSet::new();
    let mut accepted = Vec::new();
    let mut problems = Vec::new();
    for (path, json) in sources {
        match free_energy_catalog::validate_manifest(json) {
            Ok(record) => {
                if !ids.insert(record.id.clone()) {
                    problems.push(format!("{path}: duplicate project ID: {}", record.id));
                } else {
                    accepted.push((record.id, path.to_owned()));
                }
            }
            Err(errors) => {
                problems.extend(errors.into_iter().map(|error| format!("{path}: {error}")));
            }
        }
    }
    if problems.is_empty() {
        Ok(accepted)
    } else {
        Err(problems)
    }
}

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

    let mut loaded = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(text) => loaded.push((path.to_string_lossy().into_owned(), text)),
            Err(error) => errors.push(format!("{}: {error}", path.to_string_lossy())),
        }
    }

    let result = validate_loaded(
        loaded
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str())),
    );
    match result {
        Ok(records) if errors.is_empty() => {
            for (id, path) in records {
                println!("TYPED-BOUNDARY-ONLY {id}: {path}");
            }
            ExitCode::SUCCESS
        }
        Ok(_) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
        Err(mut validation_errors) => {
            errors.append(&mut validation_errors);
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LUANTI: &str = include_str!("../projects/luanti.json");
    const OPENRA: &str = include_str!("../projects/openra.json");

    #[test]
    fn distinct_manifest_ids_are_admitted_as_one_batch() {
        let result = validate_loaded([("luanti.json", LUANTI), ("openra.json", OPENRA)]).unwrap();
        assert_eq!(result.len(), 2);
        assert_ne!(result[0].0, result[1].0);
    }

    #[test]
    fn duplicate_project_ids_are_rejected_across_files() {
        let failures = validate_loaded([("one.json", LUANTI), ("two.json", LUANTI)]).unwrap_err();
        assert!(
            failures
                .iter()
                .any(|error| error.contains("two.json: duplicate project ID:")),
            "{failures:?}"
        );
    }

    #[test]
    fn malformed_later_record_does_not_return_partial_batch_success() {
        assert!(validate_loaded([("good.json", OPENRA), ("bad.json", "{}")]).is_err());
    }
}
