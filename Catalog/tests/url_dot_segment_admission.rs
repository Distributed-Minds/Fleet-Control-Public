//! Compiled typed-admission regression for URL path canonicalization.
//! Browser URL consumers remove "." and ".." path segments, including
//! percent-encoded spellings. The admitted link must not change meaning later.
//! Syntax admission is not DNS, redirect, rights, or source authenticity proof.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const PILOT: &str = include_str!("../projects/veloren.json");
const URL_FIELDS: [&str; 8] = [
    "/upstream/discovery_url",
    "/upstream/canonical_source_url",
    "/upstream/contribution_url",
    "/upstream/issue_url",
    "/upstream/read_only_mirror_urls/0",
    "/play/upstream_download_url",
    "/evidence/0/url",
    "/evidence/0/repository",
];

fn substitute(field: &str, url: &str) -> String {
    let mut project: Value = serde_json::from_str(PILOT).expect("valid pilot JSON");
    if field.starts_with("/evidence/") {
        // A standalone URL policy test must not fail merely because it changed
        // one component of an immutable pinned repository/commit/path tuple.
        project["evidence"][0]["evidence_kind"] = json!("MUTABLE_UPSTREAM_PAGE");
        project["evidence"][0]["currentness"] = json!("OBSERVED_AT");
        project["evidence"][0]["commit"] = Value::Null;
        project["evidence"][0]["path"] = Value::Null;
    }
    *project
        .pointer_mut(field)
        .unwrap_or_else(|| panic!("missing URL field {field}")) = json!(url);
    project.to_string()
}

#[test]
fn raw_and_percent_encoded_dot_segments_fail_at_every_external_link_surface() {
    validate_manifest(PILOT).expect("baseline must pass");
    for url in [
        "https://github.com/owner/../different-repo",
        "https://github.com/owner/./repo",
        "https://github.com/owner/%2e%2e/different-repo",
        "https://github.com/owner/%2E%2E/different-repo",
        "https://github.com/owner/.%2e/different-repo",
        "https://github.com/owner/%2e./different-repo",
        "https://github.com/owner/%2e/repo",
        "https://github.com/owner/./repo?source=review",
    ] {
        for field in URL_FIELDS {
            let errors = validate_manifest(&substitute(field, url))
                .expect_err(&format!("{field}: admitted aliased URL {url}"));
            assert!(
                errors
                    .iter()
                    .any(|error| error.starts_with("inadmissible external URL:")),
                "{field}: rejection must be URL-policy based for {url}: {errors:?}"
            );
        }
    }
}

#[test]
fn ordinary_dotted_names_and_query_or_fragment_text_remain_allowed() {
    validate_manifest(PILOT).expect("baseline must pass");
    for url in [
        "https://github.com/owner/release.v1.2.3",
        "https://github.com/owner/release..notes",
        "https://github.com/owner/repo?ref=../v2",
        "https://github.com/owner/repo#./notes",
        "https://github.com/owner/%2Erelease",
        "https://github.com/owner/v1%2E2%2E3",
    ] {
        for field in URL_FIELDS {
            let result = validate_manifest(&substitute(field, url));
            assert!(
                result.is_ok(),
                "{field}: harmless URL {url} was rejected: {:?}",
                result.err()
            );
        }
    }
}

#[test]
fn optional_non_pinned_repository_metadata_does_not_bypass_url_admission() {
    validate_manifest(PILOT).expect("baseline must pass");
    for url in [
        "http://github.com/owner/repo",
        "https://127.0.0.1/private",
        "https://token@github.com/owner/repo",
        "https://github.com/owner/%2e%2e/private",
    ] {
        let errors = validate_manifest(&substitute("/evidence/0/repository", url))
            .expect_err("non-pinned repository URL bypassed typed admission");
        assert!(
            errors.iter().any(|error| {
                error.starts_with("inadmissible external URL: evidence cargo-snapshot repository")
            }),
            "{url}: expected non-pinned repository admission rejection, got {errors:?}"
        );
    }
}
