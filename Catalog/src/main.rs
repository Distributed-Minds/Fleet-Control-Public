//! Experimental offline catalog admission CLI (NOT the finished catalog).
mod render;

use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

/// Escape filesystem labels before writing error diagnostics. Paths are untrusted
/// data; embedded newline and terminal controls must never forge log records.
fn diagnostic_path(path: &Path) -> String {
    format!("{path:?}")
}

/// Refuse symlinked render output directories, including links outside the
/// project tree. This is a local input-admission check, not a race-free
/// filesystem sandbox against concurrent untrusted directory replacement.
fn validate_output_directory(parent: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.file_type().is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "catalog output directory must be a real directory (symlink prohibited)",
        ));
    }
    Ok(())
}

/// A successful regeneration check must examine a real checked-in artifact,
/// not HTML reached through a symlink (possibly outside the catalog tree).
/// Reject directories and special files before attempting to read them too.
fn read_regular_generated_page(output: &Path) -> io::Result<String> {
    let metadata = fs::symlink_metadata(output)?;
    if !metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "generated catalog page must be a regular file (symlink prohibited)",
        ));
    }
    fs::read_to_string(output)
}

/// Publish a fully written page with a same-directory rename. An interrupted
/// write must not truncate the previously generated static page. This changes
/// only the local preview file; it is not hosting or publication authority.
fn write_complete_page(output: &Path, html: &str) -> io::Result<()> {
    let parent = output
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output has no parent"))?;
    validate_output_directory(parent)?;
    let filename = output
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output has no filename"))?;
    for attempt in 0..16 {
        let temporary = parent.join(format!(
            ".{}.staging-{}-{attempt}",
            filename.to_string_lossy(),
            std::process::id()
        ));
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let written = file
            .write_all(html.as_bytes())
            .and_then(|_| file.sync_all());
        drop(file);
        let result = written.and_then(|_| fs::rename(&temporary, output));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        return result;
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no unused same-directory staging filename",
    ))
}

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
                    problems.push(format!("{path:?}: duplicate project ID: {}", record.id));
                } else {
                    accepted.push((record.id, path.to_owned()));
                }
            }
            Err(errors) => {
                problems.extend(errors.into_iter().map(|error| format!("{path:?}: {error}")));
            }
        }
    }
    if problems.is_empty() {
        Ok(accepted)
    } else {
        Err(problems)
    }
}

/// Reject symlinked or non-regular JSON inputs for the deterministic render path.
/// The explicit validate CLI accepts caller-chosen paths, but generating the
/// checked-in page must not follow an untracked filesystem symlink out of the
/// project source directory (or silently accept a JSON-named directory).
fn read_render_manifest(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{}: {error}", diagnostic_path(path)))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "{}: render source must be a regular file (symlinks prohibited)",
            diagnostic_path(path)
        ));
    }
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", diagnostic_path(path)))
}

/// Collect the complete manifest set only from a real project directory.
/// A symlink at the directory boundary could otherwise redirect the preview
/// renderer to unrelated JSON files before per-file admission takes place.
fn collect_render_manifests(project_dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    validate_output_directory(project_dir)
        .map_err(|error| format!("{}: {error}", diagnostic_path(project_dir)))?;
    let entries = fs::read_dir(project_dir)
        .map_err(|error| format!("{}: {error}", diagnostic_path(project_dir)))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("project directory entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err("No project JSON manifests found".to_owned());
    }
    Ok(paths)
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
    let paths = match collect_render_manifests(&project_dir) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut records = Vec::new();
    let mut ids = HashSet::new();
    let mut problems = Vec::new();
    for path in paths {
        match read_render_manifest(&path) {
            Ok(text) => match free_energy_catalog::validate_manifest(&text) {
                Ok(record) => {
                    if !ids.insert(record.id.clone()) {
                        problems.push(format!(
                            "{}: duplicate project ID: {}",
                            diagnostic_path(&path),
                            record.id
                        ));
                    } else {
                        records.push(record);
                    }
                }
                Err(errors) => {
                    for error in errors {
                        problems.push(format!("{}: {error}", diagnostic_path(&path)));
                    }
                }
            },
            Err(error) => problems.push(format!("{}: {error}", diagnostic_path(&path))),
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
        if let Err(error) =
            validate_output_directory(output.parent().expect("catalog output always has a parent"))
        {
            eprintln!("{}: {error}", diagnostic_path(&output));
            return ExitCode::FAILURE;
        }
        match read_regular_generated_page(&output) {
            Ok(previous) if previous == html => {
                println!(
                    "Catalog HTML matches typed pilot input (not full schema/rights verification)"
                );
                ExitCode::SUCCESS
            }
            Ok(_) => {
                eprintln!(
                    "{}: generated HTML differs; run render to regenerate",
                    diagnostic_path(&output)
                );
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("{}: {error}", diagnostic_path(&output));
                ExitCode::FAILURE
            }
        }
    } else {
        if let Some(parent) = output.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!("{}: {error}", diagnostic_path(parent));
                return ExitCode::FAILURE;
            }
        }
        match write_complete_page(&output, &html) {
            Ok(()) => {
                println!(
                    "Generated {} from typed pilot records (draft only)",
                    output.display()
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{}: {error}", diagnostic_path(&output));
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
        eprintln!("Usage: free-energy-catalog validate <manifest.json|project-directory> [more paths ...]");
        return ExitCode::FAILURE;
    }
    let paths: Vec<_> = args.collect();
    if paths.is_empty() {
        eprintln!("At least one manifest path is required");
        return ExitCode::FAILURE;
    }

    // Expand direct JSON children of a project directory. Sorting makes
    // success records and diagnostics independent of filesystem order.
    let mut manifest_paths = Vec::new();
    let mut errors = Vec::new();
    for input in paths {
        let path = std::path::PathBuf::from(input);
        if path.is_dir() {
            // is_dir() follows symlinks. Reject a supplied alias before
            // enumerating files outside the chosen project directory.
            if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
                errors.push(format!(
                    "{}: symlinked project directory prohibited",
                    diagnostic_path(&path)
                ));
                continue;
            }
            let mut count = 0;
            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                let child = entry.path();
                                if child.extension().and_then(|ext| ext.to_str()) == Some("json") {
                                    manifest_paths.push(child);
                                    count += 1;
                                }
                            }
                            Err(error) => errors.push(format!(
                                "{}: directory entry: {error}",
                                diagnostic_path(&path)
                            )),
                        }
                    }
                    if count == 0 {
                        errors.push(format!(
                            "{}: no JSON manifests found",
                            diagnostic_path(&path)
                        ));
                    }
                }
                Err(error) => errors.push(format!("{}: {error}", diagnostic_path(&path))),
            }
        } else {
            manifest_paths.push(path);
        }
    }
    manifest_paths.sort();

    let mut loaded = Vec::new();
    for path in manifest_paths {
        // Explicit manifests and directory children use the same pre-read
        // admission rule. This does not prevent concurrent path replacement.
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                errors.push(format!(
                    "{}: symlinked manifest prohibited",
                    diagnostic_path(&path)
                ));
                continue;
            }
            Ok(metadata) if !metadata.file_type().is_file() => {
                errors.push(format!(
                    "{}: manifest is not a regular file",
                    diagnostic_path(&path)
                ));
                continue;
            }
            Err(error) => {
                errors.push(format!("{}: {error}", diagnostic_path(&path)));
                continue;
            }
            Ok(_) => {}
        }
        match fs::read_to_string(&path) {
            Ok(text) => loaded.push((path.to_string_lossy().into_owned(), text)),
            Err(error) => errors.push(format!("{}: {error}", diagnostic_path(&path))),
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
                // A pathname is untrusted data: newlines must not forge extra
                // positive CLI records. JSON quoting is reversible for consumers.
                let path_json =
                    serde_json::to_string(&path).expect("serialize admitted input path");
                println!("TYPED-BOUNDARY-ONLY {id}: {path_json}");
            }
            ExitCode::SUCCESS
        }
        Ok(_) => {
            errors.sort();
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::FAILURE
        }
        Err(mut validation_errors) => {
            errors.append(&mut validation_errors);
            errors.sort();
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
                .any(|error| error.contains("\"two.json\": duplicate project ID:")),
            "{failures:?}"
        );
    }

    #[test]
    fn malformed_later_record_does_not_return_partial_batch_success() {
        assert!(validate_loaded([("good.json", OPENRA), ("bad.json", "{}")]).is_err());
    }
}

#[cfg(test)]
mod atomic_render_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    fn sandbox() -> std::path::PathBuf {
        let number = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!(
            "free-energy-catalog-atomic-{}-{number}",
            std::process::id()
        ));
        fs::create_dir(&dir).expect("create unique test output directory");
        dir
    }

    #[test]
    fn complete_page_replaces_old_contents_without_staging_debris() {
        let dir = sandbox();
        let target = dir.join("index.html");
        fs::write(&target, "previous complete page").expect("seed page");

        write_complete_page(&target, "<html>replacement & safe</html>")
            .expect("publish whole replacement");
        assert_eq!(
            fs::read_to_string(&target).expect("read published page"),
            "<html>replacement & safe</html>"
        );
        let names: Vec<_> = fs::read_dir(&dir)
            .expect("read output directory")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        assert_eq!(names, [std::ffi::OsString::from("index.html")]);
        fs::remove_dir_all(&dir).expect("clean test output");
    }

    #[test]
    fn failed_rename_preserves_destination_and_cleans_staging() {
        let dir = sandbox();
        let target = dir.join("index.html");
        fs::create_dir(&target).expect("directory cannot be replaced by file");
        assert!(write_complete_page(&target, "new page").is_err());
        assert!(target.is_dir(), "failed write changed existing destination");
        assert_eq!(
            fs::read_dir(&dir).expect("read directory").count(),
            1,
            "failed publish leaked partial staging output"
        );
        fs::remove_dir_all(&dir).expect("clean test output");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_output_directory_cannot_redirect_render_to_unrelated_files() {
        use std::os::unix::fs::symlink;

        let dir = sandbox();
        let outside = dir.join("unrelated-directory");
        let alias = dir.join("site");
        fs::create_dir(&outside).expect("prepare unrelated output directory");
        fs::write(outside.join("index.html"), "original").expect("seed unrelated file");
        symlink(&outside, &alias).expect("redirect site directory outside project");

        let target = alias.join("index.html");
        let error = write_complete_page(&target, "replacement")
            .expect_err("symlinked output directory must fail closed");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("symlink prohibited"));
        assert_eq!(
            fs::read_to_string(outside.join("index.html")).unwrap(),
            "original"
        );
        assert!(validate_output_directory(&alias).is_err());
        let entries: Vec<_> = fs::read_dir(&outside)
            .expect("read unrelated directory")
            .map(|entry| entry.expect("entry").file_name())
            .collect();
        assert_eq!(entries, [std::ffi::OsString::from("index.html")]);
        fs::remove_dir_all(&dir).expect("clean isolated sandbox");
    }

    #[test]
    fn check_requires_regular_existing_generated_page() {
        let dir = sandbox();
        let target = dir.join("index.html");
        assert_eq!(
            read_regular_generated_page(&target).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        fs::create_dir(&target).expect("seed invalid directory output");
        assert_eq!(
            read_regular_generated_page(&target).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        fs::remove_dir(&target).expect("remove invalid directory output");
        fs::write(&target, "<html>expected</html>").expect("seed real generated page");
        assert_eq!(
            read_regular_generated_page(&target).unwrap(),
            "<html>expected</html>"
        );
        fs::remove_dir_all(&dir).expect("clean isolated sandbox");
    }

    #[cfg(unix)]
    #[test]
    fn check_cannot_accept_matching_html_via_output_file_symlink() {
        use std::os::unix::fs::symlink;

        let dir = sandbox();
        let outside = dir.join("external.html");
        let target = dir.join("index.html");
        fs::write(&outside, "<html>expected</html>").expect("seed unrelated HTML");
        symlink(&outside, &target).expect("replace checked page with symlink");
        let error = read_regular_generated_page(&target)
            .expect_err("symlink to matching HTML must never pass regeneration check");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("symlink prohibited"));
        assert_eq!(
            fs::read_to_string(&outside).unwrap(),
            "<html>expected</html>",
            "check must never mutate the symlink target"
        );
        fs::remove_dir_all(&dir).expect("clean isolated sandbox");
    }

    #[cfg(unix)]
    #[test]
    fn replacing_existing_link_does_not_overwrite_its_target() {
        use std::os::unix::fs::symlink;

        let dir = sandbox();
        let outside = dir.join("unrelated.txt");
        let target = dir.join("index.html");
        fs::write(&outside, "unrelated original").expect("seed unrelated file");
        symlink(&outside, &target).expect("seed existing link");

        write_complete_page(&target, "new complete page").expect("replace link itself");
        assert_eq!(fs::read_to_string(&target).unwrap(), "new complete page");
        assert_eq!(fs::read_to_string(&outside).unwrap(), "unrelated original");
        fs::remove_dir_all(&dir).expect("clean test output");
    }
}

#[cfg(test)]
mod render_manifest_file_admission_tests {
    use super::read_render_manifest;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    const PILOT: &str = include_str!("../projects/luanti.json");

    struct Sandbox(PathBuf);

    impl Sandbox {
        fn new() -> Self {
            let number = NEXT.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "free-energy-render-manifest-admission-{}-{number}",
                std::process::id()
            ));
            fs::create_dir(&directory).expect("unique local sandbox");
            Self(directory)
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove isolated fixture");
        }
    }

    #[test]
    fn only_json_children_of_a_real_project_directory_are_sorted() {
        let sandbox = Sandbox::new();
        let projects = sandbox.0.join("projects");
        fs::create_dir(&projects).expect("create project source");
        fs::write(projects.join("z.json"), PILOT).expect("write last manifest");
        fs::write(projects.join("a.json"), PILOT).expect("write first manifest");
        fs::write(projects.join("notes.txt"), "not a manifest").expect("write unrelated file");
        let files = super::collect_render_manifests(&projects).expect("admit real directory");
        assert_eq!(
            files,
            vec![projects.join("a.json"), projects.join("z.json")]
        );
    }

    #[test]
    fn missing_empty_or_nondirectory_project_sources_fail_closed() {
        let sandbox = Sandbox::new();
        let missing = sandbox.0.join("missing");
        assert!(super::collect_render_manifests(&missing).is_err());
        let regular_file = sandbox.0.join("projects");
        fs::write(&regular_file, PILOT).expect("create file instead of directory");
        assert!(super::collect_render_manifests(&regular_file).is_err());
        fs::remove_file(&regular_file).expect("remove file");
        fs::create_dir(&regular_file).expect("create empty directory");
        assert!(super::collect_render_manifests(&regular_file).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn project_directory_symlink_cannot_redirect_render_inputs() {
        use std::os::unix::fs::symlink;

        let sandbox = Sandbox::new();
        let external = sandbox.0.join("unrelated-projects");
        fs::create_dir(&external).expect("create outside source directory");
        fs::write(external.join("project.json"), PILOT).expect("seed outside data");
        let project_alias = sandbox.0.join("projects");
        symlink(&external, &project_alias).expect("redirect project directory");

        let error = super::collect_render_manifests(&project_alias)
            .expect_err("symlinked directory must never supply rendered projects");
        assert!(error.contains("symlink prohibited"), "{error}");
        assert_eq!(
            fs::read_to_string(external.join("project.json")).unwrap(),
            PILOT,
            "rejected input must remain unchanged"
        );
    }

    #[test]
    fn regular_json_manifest_remains_readable() {
        let sandbox = Sandbox::new();
        let project = sandbox.0.join("luanti.json");
        fs::write(&project, PILOT).expect("write local pilot");
        assert_eq!(read_render_manifest(&project).unwrap(), PILOT);
    }

    #[test]
    fn directory_with_json_extension_is_rejected() {
        let sandbox = Sandbox::new();
        let not_a_file = sandbox.0.join("directory.json");
        fs::create_dir(&not_a_file).expect("create disguised directory");
        let diagnosis = read_render_manifest(&not_a_file).unwrap_err();
        assert!(diagnosis.contains("regular file"), "{diagnosis}");
    }

    #[test]
    fn missing_input_is_not_silently_accepted() {
        let sandbox = Sandbox::new();
        assert!(read_render_manifest(&sandbox.0.join("missing.json")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_import_external_manifests_or_alias_local_ones() {
        use std::os::unix::fs::symlink;

        let sandbox = Sandbox::new();
        let projects = sandbox.0.join("projects");
        fs::create_dir(&projects).expect("project fixture directory");
        let external = sandbox.0.join("outside-projects.json");
        fs::write(&external, PILOT).expect("write outside project source");
        let external_link = projects.join("external.json");
        symlink(&external, &external_link).expect("link to external data");

        let internal = projects.join("internal.json");
        fs::write(&internal, PILOT).expect("write real manifest");
        let alias = projects.join("alias.json");
        symlink(&internal, &alias).expect("link to in-tree manifest");

        for path in [&external_link, &alias] {
            let diagnosis = read_render_manifest(path).unwrap_err();
            assert!(diagnosis.contains("symlinks prohibited"), "{diagnosis}");
        }
        assert_eq!(read_render_manifest(&internal).unwrap(), PILOT);
    }
}
