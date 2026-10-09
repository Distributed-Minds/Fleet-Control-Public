//! Experimental offline catalog admission CLI (NOT the finished catalog).
mod render;

use std::{
    collections::HashSet,
    env, fs,
    io::{self, Read, Write},
    path::Path,
    process::ExitCode,
};

/// Escape filesystem labels before writing error diagnostics. Paths are untrusted
/// data; embedded newline and terminal controls must never forge log records.
fn diagnostic_path(path: &Path) -> String {
    format!("{path:?}")
}

/// Refuse a pathname whose directory ancestors include a symlink. Checking
/// only the leaf allows a real file or directory to escape the intended tree
/// through a linked parent. This is not protection against concurrent swaps.
fn reject_symlinked_ancestors(path: &Path) -> io::Result<()> {
    let mut prefix = std::path::PathBuf::new();
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        prefix.push(component.as_os_str());
        if components.peek().is_none() {
            break; // The caller checks the leaf's required file/directory type.
        }
        let metadata = fs::symlink_metadata(&prefix)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "symlinked ancestor prohibited: {}",
                    diagnostic_path(&prefix)
                ),
            ));
        }
    }
    Ok(())
}

/// Refuse symlinked render output directories, including links outside the
/// project tree. This is a local input-admission check, not a race-free
/// filesystem sandbox against concurrent untrusted directory replacement.
fn validate_output_directory(parent: &Path) -> io::Result<()> {
    reject_symlinked_ancestors(parent)?;
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
/// Compare a generated preview without permitting a damaged or attacker-grown
/// checked-in page to allocate an unbounded String. The canonical newly rendered
/// page is already available, so its exact byte length is the natural read cap.
/// This remains a best-effort local filesystem check, not a race-free sandbox.
fn read_regular_generated_page(output: &Path, expected_len: usize) -> io::Result<String> {
    reject_symlinked_ancestors(output)?;
    let before = fs::symlink_metadata(output)?;
    if !before.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "generated catalog page must be a regular file (symlink prohibited)",
        ));
    }
    let limit = u64::try_from(expected_len).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "expected HTML length overflow")
    })?;
    if before.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "generated catalog page exceeds expected output length",
        ));
    }

    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // Reject swapped-in symlinks and avoid blocking on a swapped-in FIFO
        // before the opened-file type/identity checks can run.
        options.custom_flags(0o400000 | 0o4000); // O_NOFOLLOW | O_NONBLOCK
    }
    let file = options.open(output)?;
    let opened = file.metadata()?;
    if !opened.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "opened generated catalog page is not a regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "generated catalog page changed between preflight and open",
            ));
        }
    }
    if opened.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "generated catalog page exceeds expected output length",
        ));
    }

    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "generated catalog page grew beyond expected output length",
        ));
    }
    String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
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

/// Limit every manifest read before parsing untrusted JSON. A large sparse file
/// must not allocate its declared size; a growing file cannot bypass the cap.
/// The inode check on Unix detects replacement of the preflighted regular
/// file with a different open file. This is not a race-free filesystem sandbox.
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_BATCH_MANIFESTS: usize = 256;
const MAX_BATCH_BYTES: usize = 16 * 1024 * 1024;

/// Keep bounded reads bounded as a batch too. Both render and validate
/// must fail before emitting a partial accepted catalog.
#[derive(Default)]
struct ManifestBatchBudget {
    files: usize,
    bytes: usize,
}

impl ManifestBatchBudget {
    fn note_file(&mut self) -> io::Result<()> {
        if self.files >= MAX_BATCH_MANIFESTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "catalog batch exceeds 256 manifests",
            ));
        }
        self.files += 1;
        Ok(())
    }

    fn note_bytes(&mut self, next: usize) -> io::Result<()> {
        let total = self.bytes.checked_add(next).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "catalog batch size overflow")
        })?;
        if total > MAX_BATCH_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "catalog batch exceeds 16 MiB total manifest input",
            ));
        }
        self.bytes = total;
        Ok(())
    }
}

fn read_bounded_manifest(path: &Path) -> io::Result<String> {
    let before = fs::symlink_metadata(path)?;
    read_bounded_manifest_observed(path, &before)
}

/// Check the exact preflighted inode after opening. Keeping preflight and
/// opening separate also permits deterministic swap regressions.
fn read_bounded_manifest_observed(path: &Path, before: &fs::Metadata) -> io::Result<String> {
    if !before.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "manifest must be a regular file (symlinks prohibited)",
        ));
    }
    if before.len() > MAX_MANIFEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "manifest exceeds 1 MiB input limit",
        ));
    }

    // The path can change after lstat. A swapped-in FIFO would block a
    // regular read-only open indefinitely; a swapped-in symlink could
    // transiently open an unrelated target before the inode check. Apply
    // Linux's open-time guards before inspecting the opened descriptor.
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NONBLOCK: i32 = 0o4000;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options.open(path)?;
    let opened = file.metadata()?;
    if !opened.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "opened manifest is not a regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != opened.dev() || before.ino() != opened.ino() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "manifest changed between preflight and open",
            ));
        }
    }
    if opened.len() > MAX_MANIFEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "manifest exceeds 1 MiB input limit",
        ));
    }

    let mut text = String::new();
    file.take(MAX_MANIFEST_BYTES + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "manifest exceeds 1 MiB input limit",
        ));
    }
    Ok(text)
}

/// Reject symlinked or non-regular JSON inputs for the deterministic render path.
/// The explicit validate CLI accepts caller-chosen paths, but generating the
/// checked-in page must not follow an untracked filesystem symlink out of the
/// project source directory (or silently accept a JSON-named directory).
fn read_render_manifest(path: &Path) -> Result<String, String> {
    reject_symlinked_ancestors(path)
        .map_err(|error| format!("{}: {error}", diagnostic_path(path)))?;
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{}: {error}", diagnostic_path(path)))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "{}: render source must be a regular file (symlinks prohibited)",
            diagnostic_path(path)
        ));
    }
    read_bounded_manifest(path).map_err(|error| format!("{}: {error}", diagnostic_path(path)))
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
    let mut budget = ManifestBatchBudget::default();
    for entry in entries {
        let entry = entry.map_err(|error| format!("project directory entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            budget.note_file().map_err(|error| error.to_string())?;
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
    let mut byte_budget = ManifestBatchBudget::default();
    for path in paths {
        match read_render_manifest(&path) {
            Ok(text) => {
                if let Err(error) = byte_budget.note_bytes(text.len()) {
                    problems.push(format!("{}: {error}", diagnostic_path(&path)));
                    break;
                }
                match free_energy_catalog::validate_manifest(&text) {
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
                }
            }
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
        match read_regular_generated_page(&output, html.len()) {
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
    let mut path_budget = ManifestBatchBudget::default();
    let mut over_limit = false;
    for input in paths {
        let path = std::path::PathBuf::from(input);
        if let Err(error) = reject_symlinked_ancestors(&path) {
            errors.push(format!("{}: {error}", diagnostic_path(&path)));
            continue;
        }
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
                                    if let Err(error) = path_budget.note_file() {
                                        errors
                                            .push(format!("{}: {error}", diagnostic_path(&child)));
                                        over_limit = true;
                                        break;
                                    }
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
                    if count == 0 && !over_limit {
                        errors.push(format!(
                            "{}: no JSON manifests found",
                            diagnostic_path(&path)
                        ));
                    }
                }
                Err(error) => errors.push(format!("{}: {error}", diagnostic_path(&path))),
            }
        } else if let Err(error) = path_budget.note_file() {
            errors.push(format!("{}: {error}", diagnostic_path(&path)));
            over_limit = true;
        } else {
            manifest_paths.push(path);
        }
        if over_limit {
            break;
        }
    }
    if over_limit {
        errors.sort();
        for error in errors {
            eprintln!("{error}");
        }
        return ExitCode::FAILURE;
    }
    manifest_paths.sort();

    let mut loaded = Vec::new();
    let mut byte_budget = ManifestBatchBudget::default();
    for path in manifest_paths {
        if let Err(error) = reject_symlinked_ancestors(&path) {
            errors.push(format!("{}: {error}", diagnostic_path(&path)));
            continue;
        }
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
        match read_bounded_manifest(&path) {
            Ok(text) => {
                if let Err(error) = byte_budget.note_bytes(text.len()) {
                    errors.push(format!("{}: {error}", diagnostic_path(&path)));
                    break;
                }
                loaded.push((path.to_string_lossy().into_owned(), text));
            }
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
    fn bounded_manifest_reader_admits_regular_input_through_exact_byte_limit() {
        let file = env::temp_dir().join(format!(
            "free-energy-catalog-bounded-positive-{}.json",
            std::process::id()
        ));
        fs::write(&file, LUANTI).expect("create real pilot input");
        assert_eq!(
            read_bounded_manifest(&file).expect("read regular pilot"),
            LUANTI
        );
        let padded = format!(
            "{LUANTI}{}",
            " ".repeat(MAX_MANIFEST_BYTES as usize - LUANTI.len())
        );
        fs::write(&file, &padded).expect("create exact-limit UTF-8 manifest");
        assert_eq!(
            read_bounded_manifest(&file).expect("exact limit must be admitted"),
            padded
        );
        fs::remove_file(file).expect("remove test file");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn manifest_open_rejects_fifo_swapped_after_regular_file_preflight() {
        let path = env::temp_dir().join(format!(
            "free-energy-catalog-swapped-fifo-{}",
            std::process::id()
        ));
        fs::write(&path, LUANTI).expect("create preflighted regular manifest");
        let before = fs::symlink_metadata(&path).expect("observe regular file");
        fs::remove_file(&path).expect("replace regular file");
        let status = std::process::Command::new("mkfifo")
            .arg(&path)
            .status()
            .expect("invoke POSIX mkfifo on Linux");
        assert!(status.success(), "create swapped-in FIFO");

        // A blocking File::open would hang here before inode validation.
        // O_NONBLOCK lets the opened-descriptor type check reject the FIFO.
        let error = read_bounded_manifest_observed(&path, &before)
            .expect_err("special file substitution must not be admitted");
        assert!(error.to_string().contains("not a regular file"), "{error}");
        fs::remove_file(path).expect("remove test FIFO");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn manifest_open_rejects_symlink_swapped_after_regular_file_preflight() {
        use std::os::unix::fs::symlink;
        let path = env::temp_dir().join(format!(
            "free-energy-catalog-swapped-link-{}",
            std::process::id()
        ));
        let outside = env::temp_dir().join(format!(
            "free-energy-catalog-outside-link-{}",
            std::process::id()
        ));
        fs::write(&path, LUANTI).expect("create preflighted regular manifest");
        fs::write(&outside, LUANTI).expect("create unrelated regular manifest");
        let before = fs::symlink_metadata(&path).expect("observe regular file");
        fs::remove_file(&path).expect("replace regular file");
        symlink(&outside, &path).expect("replace with symlink");

        assert!(
            read_bounded_manifest_observed(&path, &before).is_err(),
            "a swapped-in symlink must not read the unrelated target"
        );
        fs::remove_file(path).expect("remove test symlink");
        fs::remove_file(outside).expect("remove unrelated test input");
    }

    #[test]
    fn oversized_sparse_manifest_is_rejected_before_parser_or_allocation() {
        let file = env::temp_dir().join(format!(
            "free-energy-catalog-bounded-negative-{}.json",
            std::process::id()
        ));
        let sparse = fs::File::create(&file).expect("create sparse input");
        sparse
            .set_len(MAX_MANIFEST_BYTES + 1)
            .expect("set oversized declared length");
        drop(sparse);
        let error = read_bounded_manifest(&file).expect_err("oversized input must fail");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("1 MiB input limit"));
        fs::remove_file(file).expect("remove test file");
    }

    #[test]
    fn batch_budget_admits_boundary_and_rejects_count_overflow() {
        let mut budget = ManifestBatchBudget::default();
        for _ in 0..MAX_BATCH_MANIFESTS {
            budget.note_file().expect("bounded manifest");
        }
        assert!(budget
            .note_file()
            .unwrap_err()
            .to_string()
            .contains("256 manifests"));

        budget.note_bytes(MAX_BATCH_BYTES).expect("exactly 16 MiB");
        assert!(budget
            .note_bytes(1)
            .unwrap_err()
            .to_string()
            .contains("16 MiB"));
        assert_eq!(budget.bytes, MAX_BATCH_BYTES);
        assert!(budget.note_bytes(0).is_ok());
    }

    #[test]
    fn render_collection_rejects_more_than_256_json_paths() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!(
            "free-energy-catalog-render-count-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&dir).expect("create disposable render project directory");
        for index in 0..=MAX_BATCH_MANIFESTS {
            fs::write(dir.join(format!("{index:03}.json")), "{}")
                .expect("create disposable manifest path");
        }
        let error = collect_render_manifests(&dir).expect_err("257 render paths must fail");
        assert!(error.contains("256 manifests"), "{error}");
        fs::remove_dir_all(&dir).expect("remove disposable render project directory");
    }

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

    #[cfg(unix)]
    #[test]
    fn nested_symlinked_output_ancestor_cannot_redirect_render_or_check() {
        use std::os::unix::fs::symlink;

        let dir = sandbox();
        let outside = dir.join("unrelated-directory");
        let real_site = outside.join("site");
        fs::create_dir_all(&real_site).expect("create unrelated nested output");
        fs::write(real_site.join("index.html"), "original").expect("seed external page");
        let alias = dir.join("redirect");
        symlink(&outside, &alias).expect("link an intermediate directory");
        let target = alias.join("site/index.html");

        let error = write_complete_page(&target, "replacement")
            .expect_err("symlinked ancestor must not redirect output");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("symlinked ancestor"));
        assert!(
            read_regular_generated_page(&target, 64).is_err(),
            "matching external HTML must not pass render --check"
        );
        assert_eq!(
            fs::read_to_string(real_site.join("index.html")).unwrap(),
            "original"
        );
        fs::remove_dir_all(&dir).expect("clean isolated sandbox");
    }

    #[test]
    fn check_requires_regular_existing_generated_page() {
        let dir = sandbox();
        let target = dir.join("index.html");
        assert_eq!(
            read_regular_generated_page(&target, 64).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        fs::create_dir(&target).expect("seed invalid directory output");
        assert_eq!(
            read_regular_generated_page(&target, 64).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        fs::remove_dir(&target).expect("remove invalid directory output");
        fs::write(&target, "<html>expected</html>").expect("seed real generated page");
        assert_eq!(
            read_regular_generated_page(&target, 64).unwrap(),
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
        let error = read_regular_generated_page(&target, 64)
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

    #[cfg(unix)]
    #[test]
    fn symlinked_project_ancestor_cannot_supply_real_manifest() {
        use std::os::unix::fs::symlink;

        let sandbox = Sandbox::new();
        let outside = sandbox.0.join("unrelated-root");
        let real_projects = outside.join("projects");
        fs::create_dir_all(&real_projects).expect("create unrelated project tree");
        fs::write(real_projects.join("luanti.json"), PILOT).expect("seed real manifest");
        let alias = sandbox.0.join("redirect");
        symlink(&outside, &alias).expect("link intermediate parent");
        let aliased_dir = alias.join("projects");
        let aliased_file = aliased_dir.join("luanti.json");

        let error = super::collect_render_manifests(&aliased_dir)
            .expect_err("symlinked ancestor must not supply project directory");
        assert!(error.contains("symlinked ancestor"), "{error}");
        let error = read_render_manifest(&aliased_file)
            .expect_err("symlinked ancestor must not supply a regular manifest");
        assert!(error.contains("symlinked ancestor"), "{error}");
        assert_eq!(
            fs::read_to_string(real_projects.join("luanti.json")).unwrap(),
            PILOT
        );
        assert!(super::reject_symlinked_ancestors(&real_projects).is_ok());
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

#[cfg(test)]
mod generated_page_read_bounds_tests {
    use super::read_regular_generated_page;
    use std::{
        fs,
        io::ErrorKind,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "free-energy-generated-check-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&dir).expect("create disposable page check");
            Self(dir)
        }

        fn page(&self) -> PathBuf {
            self.0.join("index.html")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn unchanged_page_admits_exact_bytes_and_shorter_page_is_only_a_mismatch() {
        let scratch = Scratch::new();
        let page = scratch.page();
        let expected = "<!doctype html>\n<html>draft</html>\n";
        fs::write(&page, expected).expect("write known page");
        assert_eq!(
            read_regular_generated_page(&page, expected.len()).expect("read exact page"),
            expected
        );
        fs::write(&page, "<html>stale</html>").expect("write shorter page");
        assert_eq!(
            read_regular_generated_page(&page, expected.len()).expect("read shorter page"),
            "<html>stale</html>"
        );
    }

    #[test]
    fn oversized_sparse_or_growing_page_cannot_allocate_to_declared_size() {
        let scratch = Scratch::new();
        let page = scratch.page();
        let expected_len = 32;
        fs::write(&page, vec![b'x'; expected_len + 1]).expect("write one-byte excess");
        let error = read_regular_generated_page(&page, expected_len).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("expected output length"));

        let file = fs::OpenOptions::new()
            .write(true)
            .open(&page)
            .expect("open disposable sparse file");
        file.set_len(64 * 1024 * 1024).expect("grow sparse page");
        let error = read_regular_generated_page(&page, expected_len).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("expected output length"));
    }

    #[test]
    fn malformed_utf8_and_nonregular_pages_fail_closed() {
        let scratch = Scratch::new();
        let page = scratch.page();
        fs::write(&page, [0xff, 0xfe]).expect("write invalid UTF-8");
        assert_eq!(
            read_regular_generated_page(&page, 2).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        fs::remove_file(&page).expect("remove sample");
        fs::create_dir(&page).expect("make nonregular page");
        assert_eq!(
            read_regular_generated_page(&page, 2).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_generated_page_is_not_followed() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let target = scratch.0.join("legitimate.html");
        let alias = scratch.page();
        fs::write(&target, "<html>draft</html>").expect("write real page");
        symlink(&target, &alias).expect("make symlink");
        assert_eq!(
            read_regular_generated_page(&alias, 128).unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }
}
