//! Typed-admission regression for source-pinned evidence URI/identity binding.
//! This does not fetch upstream content or grant redistribution permission.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");
const OPENRA: &str = include_str!("../projects/openra.json");
const VELOREN: &str = include_str!("../projects/veloren.json");

fn mutate_evidence(base: &str, evidence_id: &str, mutate: impl FnOnce(&mut Value)) -> String {
    let mut record: Value = serde_json::from_str(base).expect("checked-in pilot JSON");
    let evidence = record["evidence"]
        .as_array_mut()
        .expect("pilot has evidence")
        .iter_mut()
        .find(|item| item["evidence_id"].as_str() == Some(evidence_id))
        .expect("target evidence exists");
    mutate(evidence);
    serde_json::to_string(&record).expect("serialize mutated fixture")
}

fn assert_uri_mismatch(base: &str, evidence_id: &str, mutate: impl FnOnce(&mut Value)) {
    let input = mutate_evidence(base, evidence_id, mutate);
    let errors = validate_manifest(&input).expect_err("inconsistent pinned evidence must fail");
    assert!(
        errors.iter().any(|error| {
            error.contains("pinned repository evidence URL does not match repository/commit/path")
        }),
        "missing cross-field rejection: {errors:?}"
    );
}

#[test]
fn all_original_pinned_pilot_evidence_remains_admissible() {
    for manifest in [LUANTI, OPENRA, VELOREN] {
        validate_manifest(manifest).expect("unchanged source-pinned pilot");
    }
}

#[test]
fn mismatched_commits_paths_repositories_and_urls_are_rejected() {
    assert_uri_mismatch(LUANTI, "license-snapshot", |item| {
        item["commit"] = json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    });
    assert_uri_mismatch(OPENRA, "copying-snapshot", |item| {
        item["path"] = json!("README.md");
    });
    assert_uri_mismatch(VELOREN, "cargo-snapshot", |item| {
        item["repository"] = json!("https://github.com/other-owner/other-repository");
    });
    assert_uri_mismatch(OPENRA, "readme-snapshot", |item| {
        item["url"] = json!(
            "https://github.com/OpenRA/OpenRA/blob/b6fc03fcfaef1277592bbd4cbc7d44dd85219902/COPYING"
        );
    });
}

#[test]
fn ambiguous_or_deceptive_repository_identity_fails_closed() {
    for repository in [
        "https://github.com/luanti-org/luanti/",
        "https://github.com/luanti-org/luanti?branch=main",
        "https://github.com/luanti-org/luanti#README",
        "https://github.com/luanti-org/luanti/blob/malicious",
        "https://example.org/luanti-org/luanti",
    ] {
        assert_uri_mismatch(LUANTI, "license-snapshot", |item| {
            item["repository"] = json!(repository);
        });
    }
    assert_uri_mismatch(LUANTI, "license-snapshot", |item| {
        item["url"] = json!(
            "https://github.com/luanti-org/luanti/blob/9a1b92d0d4d2c47fced18e6077722c6301eb04f5/LICENSE.txt#L10"
        );
    });
}

#[test]
fn supported_gitlab_permalink_layout_is_checked_without_remote_trust() {
    let manifest = mutate_evidence(VELOREN, "cargo-snapshot", |item| {
        item["repository"] = json!("https://gitlab.com/veloren/veloren");
        item["url"] = json!(
            "https://gitlab.com/veloren/veloren/-/blob/81283b7babbadd2b9beb17f6763de2e545fff853/Cargo.toml"
        );
    });
    // Structural equality, not a claim that this GitLab revision exists.
    validate_manifest(&manifest).expect("canonical GitLab structural permalink admitted");
}

#[test]
fn pinned_file_paths_with_ordinary_interior_spaces_use_encoded_permalinks() {
    let input = mutate_evidence(LUANTI, "license-snapshot", |item| {
        item["path"] = json!("docs/license documents/LICENSE notice.txt");
        item["url"] = json!(
            "https://github.com/luanti-org/luanti/blob/9a1b92d0d4d2c47fced18e6077722c6301eb04f5/docs/license%20documents/LICENSE%20notice.txt"
        );
    });
    validate_manifest(&input).expect("safe spaces are canonicalized as %20 in pinned URL");
}
