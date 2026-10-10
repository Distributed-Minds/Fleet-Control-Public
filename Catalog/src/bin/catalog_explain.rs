//! Offline, machine-readable inspection of draft FREE ENERGY project records.
//!
//! This command is a READ-ONLY view of the typed v0 admission boundary, not a
//! complete JSON Schema validator, human rights approval, a playtest, or a
//! remotely verified upstream-currentness check. It does not contact providers.
//! Usage: catalog_explain <manifest.json> [manifest.json ...]
//!
//! On any invalid file, duplicate project ID, or unsafe input path, stdout
//! remains empty and the command fails: callers never receive partial success.

use free_energy_catalog::{AdapterStatus, PermissionStatus, PlayStatus, Project};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn play_status(status: &PlayStatus) -> &'static str {
    match status {
        PlayStatus::EngineOnly => "ENGINE_ONLY",
        PlayStatus::UpstreamLinkOnly => "UPSTREAM_LINK_ONLY",
        PlayStatus::FreeEnergyVerified => "FREE_ENERGY_VERIFIED",
        PlayStatus::Unavailable => "UNAVAILABLE_EVIDENCED",
        PlayStatus::Unknown => "UNKNOWN",
    }
}

fn adapter_status(status: &AdapterStatus) -> &'static str {
    match status {
        AdapterStatus::None => "NONE",
        AdapterStatus::Proposed => "PROPOSED",
        AdapterStatus::Tested => "TESTED",
    }
}

fn permission_status(status: &PermissionStatus) -> &'static str {
    match status {
        PermissionStatus::ReviewRequired => "REVIEW_REQUIRED",
        PermissionStatus::NotAuthorized => "NOT_AUTHORIZED",
    }
}

/// A JSON object is used as a transport envelope; only admitted typed values
/// are emitted. An ordinary claim, reviewer string, or license expression
/// cannot be promoted into FREE ENERGY redistribution authorization.
fn explain(project: &Project, source_file: &str) -> Value {
    let mut permissions: Vec<_> = project.permission_decisions.iter().collect();
    permissions.sort_by(|a, b| {
        (&a.component, &a.scope, &a.use_kind).cmp(&(&b.component, &b.scope, &b.use_kind))
    });
    let permissions: Vec<_> = permissions
        .into_iter()
        .map(|decision| {
            json!({
                "component": &decision.component,
                "scope": &decision.scope,
                "use": &decision.use_kind,
                "decision": permission_status(&decision.decision),
                "reason": &decision.decision_reason,
                "evidence_ids": &decision.review_evidence_ids
            })
        })
        .collect();

    let mut claims: Vec<_> = project.rights_claims.iter().collect();
    claims.sort_by(|a, b| a.claim_id.cmp(&b.claim_id));
    let claims: Vec<_> = claims
        .into_iter()
        .map(|claim| {
            json!({
                "id": &claim.claim_id,
                "component": &claim.component,
                "scope": &claim.scope,
                "scope_kind": &claim.scope_kind,
                "license_id": &claim.license_id,
                "statement": &claim.statement,
                "status": &claim.status,
                "evidence_ids": &claim.evidence_ids,
                "restrictions": &claim.exceptions_or_restrictions
            })
        })
        .collect();

    let mut evidence: Vec<_> = project.evidence.iter().collect();
    evidence.sort_by(|a, b| a.evidence_id.cmp(&b.evidence_id));
    let evidence: Vec<_> = evidence
        .into_iter()
        .map(|item| {
            json!({
                "id": &item.evidence_id,
                "kind": &item.evidence_kind,
                "subject_scope": &item.subject_scope,
                "url": &item.url,
                "observed_at": &item.observed_at,
                "currentness": &item.currentness,
                "repository": &item.repository,
                "commit": &item.commit,
                "path": &item.path
            })
        })
        .collect();

    json!({
        "explanation_schema": "free-energy.catalog-explanation/v1",
        "project_id": &project.id,
        "display_name": &project.display_name,
        "kind": &project.kind,
        "input_manifest": source_file,
        "source": {
            "canonical_url": &project.upstream.canonical_source_url,
            "revision": &project.upstream.source_revision,
            "revision_unpinned_reason": &project.upstream.source_revision_reason,
            "contribution_url": &project.upstream.contribution_url,
            "issues_url": &project.upstream.issue_url,
            "mirror_urls_read_only": &project.upstream.read_only_mirror_urls
        },
        "play": {
            "status": play_status(&project.play.status),
            "download_url_upstream_only": &project.play.upstream_download_url,
            "content_requirements": &project.play.content_requirements,
            "local_test_evidence_id": &project.play.local_test_evidence_id,
            "free_energy_hosted_download": false
        },
        "adapter": {
            "status": adapter_status(&project.adapter.status),
            "target_id": &project.adapter.target_id,
            "test_evidence_id": &project.adapter.test_evidence_id
        },
        "rights": {
            "free_energy_redistribution_authorized": false,
            "permission_decisions": permissions,
            "upstream_claims_not_clearance": claims
        },
        "evidence": evidence,
        "record_review": {
            "status": &project.review.record_status,
            "observed_at": &project.review.reviewed_at,
            "reviewer_string_not_authenticated": &project.review.reviewer
        },
        "limitations": [
            "Typed admission is not complete Draft 2020-12 validation.",
            "Publisher claims and contributor review metadata do not grant redistribution rights.",
            "Upstream links, build, local play and adapter conformance are not verified by this command."
        ]
    })
}

/// Validate the complete batch before returning even one explanation. This
/// protects consumers from accepting a prefix before a later input fails.
fn explain_sources<'a, I>(sources: I) -> Result<Vec<Value>, Vec<String>>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut seen = HashSet::new();
    let mut accepted = Vec::new();
    let mut errors = Vec::new();

    for (filename, source) in sources {
        match free_energy_catalog::validate_manifest(source) {
            Ok(project) => {
                if !seen.insert(project.id.clone()) {
                    errors.push(format!(
                        "{filename:?}: duplicate project ID {:?}",
                        project.id
                    ));
                } else {
                    accepted.push((project.id.clone(), explain(&project, filename)));
                }
            }
            Err(problems) => errors.extend(
                problems
                    .into_iter()
                    .map(|problem| format!("{filename:?}: {problem}")),
            ),
        }
    }
    if !errors.is_empty() {
        errors.sort();
        return Err(errors);
    }
    accepted.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(accepted
        .into_iter()
        .map(|(_, explanation)| explanation)
        .collect())
}

// Match the 1 MiB per-manifest input budget of the catalog validate/render CLI.
// This limits an individual explain input, not aggregate memory across a batch.
const MAX_EXPLAIN_MANIFEST_BYTES: u64 = 1024 * 1024;
// Bound retained input memory and argument fan-out independently of per-file size.
const MAX_EXPLAIN_BATCH_BYTES: usize = 16 * 1024 * 1024;
const MAX_EXPLAIN_BATCH_FILES: usize = 256;

/// Reject symlinks and directories before reading. This does not pretend to
/// defend against a privileged concurrent replacement of the same pathname.
fn read_file(path: &Path) -> Result<String, String> {
    // Inspect every component before opening the leaf. symlink_metadata on
    // only the final file silently follows a symlinked parent directory.
    // This is a best-effort input boundary, not a defence against a
    // privileged concurrent pathname replacement.
    let mut component_path = PathBuf::new();
    for component in path.components() {
        component_path.push(component.as_os_str());
        let kind =
            fs::symlink_metadata(&component_path).map_err(|error| format!("{path:?}: {error}"))?;
        if kind.file_type().is_symlink() {
            return Err(format!(
                "{path:?}: symlink path component {component_path:?} prohibited"
            ));
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("{path:?}: {error}"))?;
    read_file_after_preflight(path, &metadata)
}

// Separate the preflight identity from the open so regressions can inject a
// deterministic pathname replacement at exactly the TOCTOU boundary.
fn read_file_after_preflight(path: &Path, metadata: &fs::Metadata) -> Result<String, String> {
    if !metadata.file_type().is_file() {
        return Err(format!(
            "{path:?}: expected a regular non-symlink JSON file"
        ));
    }
    if metadata.len() > MAX_EXPLAIN_MANIFEST_BYTES {
        return Err(format!("{path:?}: manifest exceeds 1 MiB input limit"));
    }
    // A pathname can be replaced after lstat. On Linux, opening a swapped-in
    // FIFO without O_NONBLOCK can hang before metadata checks; following a
    // swapped-in symlink can access a different path before the inode check.
    // The inode check remains necessary for a swapped-in *regular* file.
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NONBLOCK: i32 = 0o4000;
        const O_NOFOLLOW: i32 = 0o400000;
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("{path:?}: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("{path:?}: {error}"))?;
    if !opened.file_type().is_file() {
        return Err(format!("{path:?}: opened manifest is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.dev() != opened.dev() || metadata.ino() != opened.ino() {
            return Err(format!(
                "{path:?}: manifest changed between preflight and open"
            ));
        }
    }
    if opened.len() > MAX_EXPLAIN_MANIFEST_BYTES {
        return Err(format!("{path:?}: manifest exceeds 1 MiB input limit"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_EXPLAIN_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{path:?}: {error}"))?;
    if bytes.len() as u64 > MAX_EXPLAIN_MANIFEST_BYTES {
        return Err(format!("{path:?}: manifest exceeds 1 MiB input limit"));
    }
    String::from_utf8(bytes).map_err(|error| format!("{path:?}: {error}"))
}

fn execute(args: Vec<PathBuf>) -> Result<String, Vec<String>> {
    if args.is_empty() {
        return Err(vec![
            "Usage: catalog_explain <manifest.json> [manifest.json ...]".to_owned(),
        ]);
    }

    if args.len() > MAX_EXPLAIN_BATCH_FILES {
        return Err(vec![format!(
            "manifest batch has {} inputs (maximum {MAX_EXPLAIN_BATCH_FILES})",
            args.len()
        )]);
    }

    let mut files = Vec::new();
    let mut errors = Vec::new();
    let mut batch_bytes = 0usize;
    for path in args {
        let filename = path.to_string_lossy().into_owned();
        match read_file(&path) {
            Ok(text) => {
                if text.len() > MAX_EXPLAIN_BATCH_BYTES.saturating_sub(batch_bytes) {
                    errors.push(format!(
                        "{path:?}: manifest batch exceeds 16 MiB cumulative input limit"
                    ));
                    break;
                }
                batch_bytes += text.len();
                files.push((filename, text));
            }
            Err(error) => errors.push(error),
        }
    }
    let inspected = explain_sources(
        files
            .iter()
            .map(|(filename, contents)| (filename.as_str(), contents.as_str())),
    );
    match inspected {
        Ok(items) if errors.is_empty() => serde_json::to_string_pretty(&items)
            .map_err(|error| vec![format!("JSON encoding failed: {error}")]),
        Ok(_) => {
            errors.sort();
            Err(errors)
        }
        Err(mut problems) => {
            errors.append(&mut problems);
            errors.sort();
            Err(errors)
        }
    }
}

fn main() -> ExitCode {
    let args = env::args_os().skip(1).map(PathBuf::from).collect();
    match execute(args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
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

    const LUANTI: &str = include_str!("../../projects/luanti.json");
    const OPENRA: &str = include_str!("../../projects/openra.json");
    const VELOREN: &str = include_str!("../../projects/veloren.json");

    #[test]
    fn all_three_pilots_produce_deterministic_separate_explanations() {
        let first = explain_sources([
            ("veloren.json", VELOREN),
            ("openra.json", OPENRA),
            ("luanti.json", LUANTI),
        ])
        .expect("all original typed pilot manifests admitted");
        let second = explain_sources([
            ("luanti.json", LUANTI),
            ("veloren.json", VELOREN),
            ("openra.json", OPENRA),
        ])
        .expect("input ordering cannot change transport output");
        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
        assert_eq!(first[0]["project_id"], "engine/luanti");
        assert_eq!(first[1]["project_id"], "engine/openra");
        assert_eq!(first[2]["project_id"], "game/veloren");
        for project in first {
            assert_eq!(
                project["rights"]["free_energy_redistribution_authorized"],
                false
            );
            assert_eq!(project["play"]["free_energy_hosted_download"], false);
            assert_eq!(project["record_review"]["status"], "DRAFT");
            assert!(project["rights"]["permission_decisions"]
                .as_array()
                .is_some_and(|decisions| !decisions.is_empty()));
        }
    }

    #[test]
    fn known_engine_does_not_become_claimed_playable_or_reusable() {
        let result = explain_sources([("luanti.json", LUANTI)]).unwrap();
        assert_eq!(result[0]["play"]["status"], "ENGINE_ONLY");
        assert_eq!(
            result[0]["rights"]["permission_decisions"][0]["decision"],
            "REVIEW_REQUIRED"
        );
        assert_eq!(
            result[0]["source"]["contribution_url"],
            "https://github.com/luanti-org/luanti/pulls"
        );
    }

    #[test]
    fn invalid_tail_and_duplicate_identity_reject_entire_batch() {
        let invalid = explain_sources([
            ("valid.json", LUANTI),
            ("malformed.json", r#"{"schema":"free-energy.project/v0"}"#),
        ]);
        assert!(invalid.is_err(), "no successful prefix can be returned");
        let duplicate = explain_sources([("one.json", OPENRA), ("two.json", OPENRA)]).unwrap_err();
        assert!(duplicate
            .iter()
            .any(|error| error.contains("duplicate project ID")));
        assert!(duplicate.iter().any(|error| error.contains("\"two.json\"")));
    }

    #[test]
    fn untrusted_metadata_is_json_escaped_not_rendered_as_markup() {
        let mut project = free_energy_catalog::validate_manifest(LUANTI).unwrap();
        project.display_name = "<img src=x onerror=alert(1)>".to_owned();
        let encoded = serde_json::to_string(&explain(&project, "bad\nsource.json")).unwrap();
        assert!(encoded.contains(r#"\nsource.json"#));
        assert!(encoded.contains("<img src=x onerror=alert(1)>"));
        assert!(!encoded.contains("<h2>"));
        let decoded: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded["display_name"], "<img src=x onerror=alert(1)>");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_ancestor_of_regular_manifest_rejects_entire_batch() {
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic relative to epoch")
            .as_nanos();
        let scratch = env::temp_dir().join(format!(
            "free-energy-catalog-explain-ancestor-{}-{nonce}",
            std::process::id()
        ));
        let real_dir = scratch.join("real");
        fs::create_dir_all(&real_dir).expect("create isolated manifest directory");
        let good = real_dir.join("luanti.json");
        fs::write(&good, LUANTI).expect("write valid manifest");
        symlink(&real_dir, scratch.join("alias")).expect("create directory symlink");
        let aliased = scratch.join("alias/luanti.json");

        assert!(
            read_file(&good).is_ok(),
            "real regular file must still work"
        );
        let failure = read_file(&aliased).expect_err("symlinked ancestor must fail closed");
        assert!(failure.contains("symlink"), "{failure}");
        assert!(
            execute(vec![good, aliased]).is_err(),
            "valid prefix + aliased tail must never produce partial JSON success"
        );
        fs::remove_dir_all(&scratch).expect("clean isolated fixture");
    }

    #[cfg(unix)]
    #[test]
    fn preflighted_manifest_rejects_swapped_regular_inode_and_symlink() {
        use std::os::unix::fs::symlink;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = env::temp_dir().join(format!(
            "free-energy-catalog-explain-open-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&scratch).unwrap();
        let input = scratch.join("manifest.json");
        fs::write(&input, LUANTI).unwrap();
        assert_eq!(read_file(&input).unwrap(), LUANTI);

        let observed = fs::symlink_metadata(&input).unwrap();
        let replacement = scratch.join("replacement.json");
        fs::write(&replacement, OPENRA).unwrap();
        fs::rename(&replacement, &input).unwrap();
        let error = read_file_after_preflight(&input, &observed).unwrap_err();
        assert!(error.contains("changed between preflight and open"), "{error}");

        let observed = fs::symlink_metadata(&input).unwrap();
        let linked_target = scratch.join("linked-target.json");
        fs::write(&linked_target, LUANTI).unwrap();
        fs::remove_file(&input).unwrap();
        symlink(&linked_target, &input).unwrap();
        assert!(
            read_file_after_preflight(&input, &observed).is_err(),
            "a swapped-in symlink must never be admitted"
        );
        assert!(read_file(&input).is_err(), "the ordinary CLI path must fail");
        fs::remove_dir_all(&scratch).unwrap();
    }

    #[test]
    fn empty_invocation_fails_without_false_success() {
        assert!(execute(Vec::new()).is_err());
    }
}
