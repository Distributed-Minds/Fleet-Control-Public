//! Typed, offline manifest admission boundary (issue #60, specification v5).
//!
//! This is deliberately NOT the complete v0 validator: a pinned, offline
//! Draft 2020-12 engine, strict URL admission, reviewed Cargo.lock,
//! source/rights authentication, rendering and artifact verification remain.
//! In particular, user-supplied reviewer names cannot confer legal rights.
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema: String,
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub upstream: Upstream,
    pub play: Play,
    pub rights_claims: Vec<RightsClaim>,
    pub permission_decisions: Vec<PermissionDecision>,
    pub adapter: Adapter,
    pub evidence: Vec<Evidence>,
    pub review: Review,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upstream {
    pub discovery_url: String,
    pub canonical_source_url: String,
    pub source_revision: Option<String>,
    pub source_revision_reason: Option<String>,
    pub contribution_url: String,
    pub issue_url: String,
    pub read_only_mirror_urls: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlayStatus {
    EngineOnly,
    UpstreamLinkOnly,
    FreeEnergyVerified,
    Unavailable,
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Play {
    pub status: PlayStatus,
    pub upstream_download_url: Option<String>,
    pub content_requirements: Vec<String>,
    pub local_test_evidence_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightsClaim {
    pub claim_id: String,
    pub claim_kind: String,
    pub component: String,
    pub scope_kind: String,
    pub scope: String,
    pub statement: String,
    pub license_id: Option<String>,
    pub exceptions_or_restrictions: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionStatus {
    ReviewRequired,
    NotAuthorized,
    // APPROVED_FOR_SCOPE is deliberately absent. A manifest cannot
    // authenticate its own purported HUMAN_RIGHTS_REVIEW.
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionDecision {
    pub component: String,
    pub scope: String,
    #[serde(rename = "use")]
    pub use_kind: String,
    pub decision: PermissionStatus,
    pub review_evidence_ids: Vec<String>,
    pub decision_reason: String,
    pub decided_at: Option<String>,
    pub reviewer: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdapterStatus {
    None,
    Proposed,
    Tested,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    pub status: AdapterStatus,
    pub target_id: Option<String>,
    pub test_evidence_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub evidence_id: String,
    pub evidence_kind: String,
    pub subject_scope: String,
    pub url: String,
    pub observed_at: String,
    pub reviewer: String,
    pub currentness: String,
    pub repository: Option<String>,
    pub commit: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimHistory {
    pub old_claim_id: String,
    pub new_claim_id: Option<String>,
    pub reason: String,
    pub at: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub reviewed_at: String,
    pub reviewer: String,
    pub record_status: String,
    pub supersedes_record_revision: Option<String>,
    pub claim_history: Vec<ClaimHistory>,
}

fn is_full_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && !value.chars().any(char::is_control)
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.trim() == part
        })
        && !value.as_bytes().get(1).is_some_and(|b| *b == b':')
}

/// Parse the closed object layout and reject a bounded set of dangerous
/// cross-record claims. Success is NOT full JSON Schema or rights approval.
pub fn validate_manifest(json: &str) -> Result<Project, Vec<String>> {
    let record: Project = serde_json::from_str(json)
        .map_err(|e| vec![format!("JSON/typed manifest: {e}")])?;
    let mut problems = Vec::new();

    if record.schema != "free-energy.project/v0" {
        problems.push("unsupported catalog schema version".to_string());
    }
    match record.upstream.source_revision.as_deref() {
        Some(rev) if !is_full_git_sha(rev) => {
            problems.push("upstream source_revision must be a full 40-hex commit".to_string());
        }
        None if record.upstream.source_revision_reason.as_deref().is_none_or(|r| r.trim().is_empty()) => {
            problems.push("unpinned upstream source revision requires a reason".to_string());
        }
        _ => {}
    }

    let mut evidence = HashMap::new();
    for item in &record.evidence {
        if evidence.insert(item.evidence_id.as_str(), item).is_some() {
            problems.push(format!("duplicate evidence ID: {}", item.evidence_id));
        }
        if item.evidence_kind == "PINNED_REPOSITORY_FILE" {
            if item.commit.as_deref().is_none_or(|s| !is_full_git_sha(s))
                || item.path.as_deref().is_none_or(|s| !is_relative_path(s))
                || item.repository.is_none()
            {
                problems.push(format!("invalid pinned repository evidence: {}", item.evidence_id));
            }
        }
    }

    let mut claims = HashSet::new();
    let mut scopes = HashSet::new();
    for item in &record.rights_claims {
        if !claims.insert(item.claim_id.as_str()) {
            problems.push(format!("duplicate rights claim ID: {}", item.claim_id));
        }
        scopes.insert((item.component.as_str(), item.scope.as_str()));
        if item.evidence_ids.is_empty() {
            problems.push(format!("rights claim {} has no evidence", item.claim_id));
        }
        for id in &item.evidence_ids {
            match evidence.get(id.as_str()) {
                None => problems.push(format!("claim {} has unknown evidence {id}", item.claim_id)),
                Some(ref_item) if ref_item.currentness == "INVALIDATED" => {
                    problems.push(format!("claim {} references invalidated evidence {id}", item.claim_id));
                }
                _ => {}
            }
        }
        if (item.scope_kind == "PATH" || item.scope_kind == "PREFIX")
            && !is_relative_path(&item.scope)
        {
            problems.push(format!("unsafe rights path scope: {}", item.claim_id));
        }
    }

    let mut permission_keys = HashSet::new();
    for item in &record.permission_decisions {
        if !scopes.contains(&(item.component.as_str(), item.scope.as_str())) {
            problems.push(format!("permission scope has no rights claim: {} / {}", item.component, item.scope));
        }
        if !permission_keys.insert((
            item.component.as_str(),
            item.scope.as_str(),
            item.use_kind.as_str(),
        )) {
            problems.push(format!("duplicate permission action: {} / {} / {}", item.component, item.scope, item.use_kind));
        }
        if item.decision_reason.trim().is_empty() {
            problems.push("permission decision requires a nonblank reason".to_string());
        }
        if item.decision == PermissionStatus::NotAuthorized
            && (item.decided_at.is_none() || item.reviewer.as_deref().is_none_or(|s| s.trim().is_empty()))
        {
            problems.push("NOT_AUTHORIZED requires review date and reviewer".to_string());
        }
        for id in &item.review_evidence_ids {
            if !evidence.contains_key(id.as_str()) {
                problems.push(format!("permission references missing evidence {id}"));
            }
        }
    }

    match &record.play.status {
        PlayStatus::UpstreamLinkOnly if record.play.upstream_download_url.is_none() => {
            problems.push("UPSTREAM_LINK_ONLY requires an upstream download link".to_string());
        }
        // The current v0 schema has no complete artifact hash, platform and
        // execution-result contract. Fail closed rather than accept a string.
        PlayStatus::FreeEnergyVerified => {
            problems.push("FREE_ENERGY_VERIFIED requires a separately implemented artifact-verification contract".to_string());
        }
        _ => {}
    }
    if record.adapter.status == AdapterStatus::Tested {
        problems.push("TESTED adapter requires a separately implemented conformance contract".to_string());
    }

    if problems.is_empty() {
        Ok(record)
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const LUANTI: &str = include_str!("../projects/luanti.json");
    const OPENRA: &str = include_str!("../projects/openra.json");
    const VELOREN: &str = include_str!("../projects/veloren.json");
    const FORGED_APPROVAL: &str = include_str!("../fixtures/forged-approval-v0.json");

    fn changed(base: &str, f: impl FnOnce(&mut Value)) -> String {
        let mut value: Value = serde_json::from_str(base).expect("valid test source");
        f(&mut value);
        serde_json::to_string(&value).expect("serializable mutated fixture")
    }

    #[test]
    fn pilot_records_are_admitted_at_the_typed_boundary() {
        for record in [LUANTI, OPENRA, VELOREN] {
            let result = validate_manifest(record);
            assert!(result.is_ok(), "{result:?}");
        }
    }

    #[test]
    fn forged_rights_approval_is_rejected() {
        assert!(validate_manifest(FORGED_APPROVAL).is_err());
    }

    #[test]
    fn missing_and_duplicate_evidence_are_rejected() {
        let missing = changed(LUANTI, |v| {
            v["rights_claims"][0]["evidence_ids"] = json!(["not-a-real-source"]);
        });
        assert!(validate_manifest(&missing).is_err());
        let duplicate = changed(LUANTI, |v| {
            let first_id = v["evidence"][0]["evidence_id"].clone();
            v["evidence"][1]["evidence_id"] = first_id;
        });
        assert!(validate_manifest(&duplicate).is_err());
    }

    #[test]
    fn permission_scope_must_match_rights_subject() {
        let attack = changed(VELOREN, |v| {
            v["permission_decisions"][0]["scope"] = json!("entire-game");
        });
        assert!(validate_manifest(&attack).is_err());
    }

    #[test]
    fn local_play_and_adapter_test_labels_fail_closed() {
        let forged_play = changed(LUANTI, |v| {
            v["play"]["status"] = json!("FREE_ENERGY_VERIFIED");
            v["play"]["local_test_evidence_id"] = json!("license-snapshot");
        });
        assert!(validate_manifest(&forged_play).is_err());
        let forged_adapter = changed(OPENRA, |v| {
            v["adapter"]["status"] = json!("TESTED");
            v["adapter"]["target_id"] = json!("engine/luanti");
            v["adapter"]["test_evidence_id"] = json!("readme-snapshot");
        });
        assert!(validate_manifest(&forged_adapter).is_err());
    }

    #[test]
    fn source_pins_must_be_immutable_full_commit_ids() {
        let attack = changed(LUANTI, |v| {
            v["upstream"]["source_revision"] = json!("master");
        });
        assert!(validate_manifest(&attack).is_err());
    }

    #[test]
    fn unknown_top_level_keys_are_rejected() {
        let attack = changed(OPENRA, |v| {
            v["unknown_authority"] = json!(true);
        });
        assert!(validate_manifest(&attack).is_err());
    }
}
