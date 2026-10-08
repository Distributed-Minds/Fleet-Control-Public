//! Experimental offline catalog admission CLI (NOT the finished catalog).
mod render;

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

/// Render only after every local pilot manifest passes typed admission.
/// This does not claim complete Draft 2020-12 schema or upstream rights clearance.
fn render_command(args: Vec<std::ffi::OsString>) -> ExitCode {
    if args.len() > 1 || args.first().is_some_and(|a| a != "--check") {
        eprintln!("Usage: free-energy-catalog render [--check]");
        return ExitCode::FAILURE;
    }
    let check = !args.is_empty();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_dir = root.join("projects");
    let entries = match fs::read_dir(&project_dir) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("{}: {error}", project_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    paths.push(path);
                }
            }
            Err(error) => {
                eprintln!("project directory entry: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    paths.sort();
    if paths.is_empty() {
        eprintln!("No project JSON manifests found");
        return ExitCode::FAILURE;
    }
    let mut records = Vec::new();
    let mut ids = HashSet::new();
    let mut problems = Vec::new();
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(text) => match free_energy_catalog::validate_manifest(&text) {
                Ok(record) => {
                    if !ids.insert(record.id.clone()) {
                        problems.push(format!("{}: duplicate project ID: {}", path.display(), record.id));
                    } else {
                        records.push(record);
                    }
                }
                Err(errors) => {
                    for error in errors {
                        problems.push(format!("{}: {error}", path.display()));
                    }
                }
            },
            Err(error) => problems.push(format!("{}: {error}", path.display())),
        }
    }
    if !problems.is_empty() {
        problems.sort();
        for problem in problems {
            eprintln!("{problem}");
        }
        return ExitCode::FAILURE;
    }
    let html = render::render_catalog(&records);
    let output = root.join("site/index.html");
    if check {
        match fs::read_to_string(&output) {
            Ok(previous) if previous == html => {
                println!("Catalog HTML matches typed pilot input (not full schema/rights verification)");
                ExitCode::SUCCESS
            }
            Ok(_) => {
                eprintln!("{}: generated HTML differs; run render to regenerate", output.display());
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("{}: {error}", output.display());
                ExitCode::FAILURE
            }
        }
    } else {
        if let Some(parent) = output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("{}: {error}", parent.display());
                return ExitCode::FAILURE;
            }
        }
        match fs::write(&output, html) {
            Ok(()) => {
                println!("Generated {} from typed pilot records (draft only)", output.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{}: {error}", output.display());
                ExitCode::FAILURE
            }
        }
    }
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let command = args.next();
    if command.as_deref() == Some(std::ffi::OsStr::new("render")) {
        return render_command(args.collect());
    }
    if command.as_deref() != Some(std::ffi::OsStr::new("validate")) {
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
