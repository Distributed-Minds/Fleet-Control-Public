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
    #[serde(rename = "UNAVAILABLE_EVIDENCED")]
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

/// Mirror the public schema's namespace/slug project ID pattern without
/// accepting uppercase, separators, or invalid first characters.
fn is_project_id(value: &str) -> bool {
    fn valid_part(part: &str) -> bool {
        let mut bytes = part.bytes();
        bytes
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
            && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }

    value
        .split_once('/')
        .is_some_and(|(namespace, slug)| valid_part(namespace) && valid_part(slug))
}

fn is_full_git_sha(value: &str) -> bool {
    // The pinned v0 schema requires lowercase SHA-1 literals, not uppercase aliases.
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// Unicode directional formatting can visually re-order repository paths while
// leaving different exact bytes. Never use such display-spoofable strings as
// identities for rights scopes or pinned evidence.
/// Conservative, offline RFC3339 profile for evidence and review timestamps.
/// Validates calendar dates and explicit time zones without network/currentness
/// inference. Leap-second (':60') claims are rejected unless a separately
/// authoritative leap-second schedule is introduced.
fn is_rfc3339_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20 || !value.is_ascii() {
        return false;
    }
    for (index, separator) in [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')] {
        if bytes.get(index) != Some(&separator) {
            return false;
        }
    }
    let number = |start: usize, end: usize| -> Option<u32> {
        let slice = bytes.get(start..end)?;
        if !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(slice).ok()?.parse().ok()
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0, 4),
        number(5, 7),
        number(8, 10),
        number(11, 13),
        number(14, 16),
        number(17, 19),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    if year == 0 || day == 0 || day > max_day || hour > 23 || minute > 59 || second > 59 {
        return false;
    }

    let mut zone = 19;
    if bytes.get(zone) == Some(&b'.') {
        zone += 1;
        let fraction_start = zone;
        while bytes.get(zone).is_some_and(u8::is_ascii_digit) {
            zone += 1;
        }
        if zone == fraction_start {
            return false;
        }
    }
    match bytes.get(zone).copied() {
        Some(b'Z') => zone + 1 == bytes.len(),
        Some(b'+') | Some(b'-') => {
            bytes.len() == zone + 6
                && bytes[zone + 3] == b':'
                && number(zone + 1, zone + 3).is_some_and(|hour| hour <= 23)
                && number(zone + 4, zone + 6).is_some_and(|minute| minute <= 59)
        }
        _ => false,
    }
}

fn is_bidi_format_character(ch: char) -> bool {
    matches!(
        ch,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

fn is_relative_path(value: &str) -> bool {
    // Repository paths are identity-bearing, not URL paths. Until an
    // authoritative percent-decoding/canonicalization contract exists, reject
    // escapes instead of admitting potentially aliased traversal or scope.
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('%')
        && !value.contains('\\')
        && !value
            .chars()
            .any(|ch| ch.is_control() || is_bidi_format_character(ch))
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".." && part.trim() == part)
        && value.as_bytes().get(1).is_none_or(|b| *b != b':')
}

// Bidirectional formatting controls change the perceived direction of link
// paths and evidence references without changing the underlying URL bytes.
// They must never be admitted as apparently trustworthy catalog links.
fn is_bidi_format(ch: char) -> bool {
    matches!(
        ch,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

/// Conservative, offline admission for externally displayed links. This does
/// not resolve DNS, follow redirects, authenticate a host or prove its rights.
/// IP literals, ports, userinfo and non-ASCII DNS names are intentionally out
/// of scope until their admission/normalization semantics are specified.
fn is_public_https_url(url: &str) -> bool {
    if url.chars().count() > 2048
        || !url.starts_with("https://")
        || url
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return false;
    }

    let rest = &url["https://".len()..];
    let authority = rest.split(&['/', '?', '#'][..]).next().unwrap_or_default();
    if authority.len() > 253 || authority.bytes().any(|c| matches!(c, b'@' | b':' | b'%')) {
        return false;
    }
    let labels: Vec<_> = authority.split('.').collect();
    if labels.len() < 2
        || labels.iter().any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
    {
        return false;
    }
    let tld = labels
        .last()
        .copied()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if tld.len() < 2
        || !tld.bytes().all(|b| b.is_ascii_alphabetic())
        || matches!(
            tld.as_str(),
            "localhost"
                | "local"
                | "internal"
                | "test"
                | "invalid"
                | "example"
                | "onion"
                | "lan"
                | "home"
                | "arpa"
        )
    {
        return false;
    }

    let bytes = url.as_bytes();
    let decode_hex = |b: u8| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    };
    // Decode exactly one URL-escape layer before checking display-control
    // characters. Checking only raw Unicode misses %E2%80%AE and similar
    // bidi spoofs that browsers decode when following a link.
    let mut decoded_url = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some((hi, lo)) = bytes.get(index + 1).zip(bytes.get(index + 2)) else {
                return false;
            };
            let (Some(hi), Some(lo)) = (decode_hex(*hi), decode_hex(*lo)) else {
                return false;
            };
            let decoded = hi * 16 + lo;
            // Reject encoded authority/path separators and a second encoding layer.
            // This is an offline admission policy, not a URL rewrite parser.
            if decoded < 0x20
                || decoded == 0x7f
                || matches!(decoded, b'/' | b'?' | b'#' | b'@' | b':' | b'\\' | b'%')
            {
                return false;
            }
            decoded_url.push(decoded);
            index += 3;
        } else {
            decoded_url.push(bytes[index]);
            index += 1;
        }
    }
    // Reject invalid UTF-8 after decoding; its browser presentation is not
    // reliably equivalent to the evidence URL being reviewed.
    if !std::str::from_utf8(&decoded_url).is_ok_and(|s| !s.chars().any(is_bidi_format)) {
        return false;
    }
    true
}

// These enum vocabularies are part of the checked-in v5 JSON Schema, but
// serde String fields otherwise accept arbitrary values. Enforce the exact
// closed sets at the typed admission boundary until full schema execution
// is implemented; a structurally valid JSON string is not a valid status.
fn check_vocabulary(problems: &mut Vec<String>, path: &str, value: &str, allowed: &[&str]) {
    if !allowed.contains(&value) {
        problems.push(format!(
            "{path}: unsupported schema vocabulary value {value:?}"
        ));
    }
}

fn validate_vocabulary(record: &Project, problems: &mut Vec<String>) {
    check_vocabulary(
        problems,
        "project.kind",
        &record.kind,
        &["ENGINE", "GAME", "MOD", "REMASTER", "TOOL"],
    );
    for (index, claim) in record.rights_claims.iter().enumerate() {
        check_vocabulary(
            problems,
            &format!("rights_claims[{index}].claim_kind"),
            &claim.claim_kind,
            &["PUBLISHER_LICENSE", "UPSTREAM_POLICY", "INTERPRETATION"],
        );
        check_vocabulary(
            problems,
            &format!("rights_claims[{index}].component"),
            &claim.component,
            &[
                "CODE",
                "MEDIA",
                "THIRD_PARTY_DATA",
                "TRADEMARK",
                "BUILD",
                "OTHER",
            ],
        );
        check_vocabulary(
            problems,
            &format!("rights_claims[{index}].scope_kind"),
            &claim.scope_kind,
            &["PROJECT_DEFAULT", "PATH", "PREFIX", "CONTENT_PACK"],
        );
        check_vocabulary(
            problems,
            &format!("rights_claims[{index}].status"),
            &claim.status,
            &["OBSERVED_AT", "SUPERSEDED", "RETRACTED", "UNKNOWN"],
        );
    }
    for (index, permission) in record.permission_decisions.iter().enumerate() {
        check_vocabulary(
            problems,
            &format!("permission_decisions[{index}].component"),
            &permission.component,
            &[
                "CODE",
                "MEDIA",
                "THIRD_PARTY_DATA",
                "TRADEMARK",
                "BUILD",
                "OTHER",
            ],
        );
        check_vocabulary(
            problems,
            &format!("permission_decisions[{index}].use"),
            &permission.use_kind,
            &["BUNDLE", "HOST", "TRANSFORM", "CROSS_GAME_REUSE"],
        );
    }
    for (index, evidence) in record.evidence.iter().enumerate() {
        check_vocabulary(
            problems,
            &format!("evidence[{index}].evidence_kind"),
            &evidence.evidence_kind,
            &[
                "PINNED_REPOSITORY_FILE",
                "MUTABLE_UPSTREAM_PAGE",
                "FREE_ENERGY_LOCAL_TEST",
                "HUMAN_RIGHTS_REVIEW",
                "EXTERNAL_RESEARCH_CLAIM",
            ],
        );
        check_vocabulary(
            problems,
            &format!("evidence[{index}].currentness"),
            &evidence.currentness,
            &["PINNED_SNAPSHOT", "OBSERVED_AT", "INVALIDATED", "UNKNOWN"],
        );
    }
    check_vocabulary(
        problems,
        "review.record_status",
        &record.review.record_status,
        &["DRAFT", "REVIEWED", "WITHDRAWN"],
    );
}

// Match the published schema's ASCII slug vocabulary. Identifiers have a
// semantic role in rights/evidence references, not just a display role.
fn check_slug(problems: &mut Vec<String>, path: &str, value: &str) {
    let mut bytes = value.bytes();
    let valid = bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'));
    if !valid {
        problems.push(format!("{path}: invalid slug identifier"));
    }
}

/// Parse the closed object layout and reject a bounded set of dangerous
/// cross-record claims. Success is NOT full JSON Schema or rights approval.
pub fn validate_manifest(json: &str) -> Result<Project, Vec<String>> {
    let record: Project =
        serde_json::from_str(json).map_err(|e| vec![format!("JSON/typed manifest: {e}")])?;
    let mut problems = Vec::new();
    validate_vocabulary(&record, &mut problems);
    // These cardinality rules are required by the checked-in v5 JSON Schema;
    // serde accepts an empty Vec, so typed decoding alone cannot enforce them.
    for (path, count) in [
        ("rights_claims", record.rights_claims.len()),
        ("permission_decisions", record.permission_decisions.len()),
        ("evidence", record.evidence.len()),
    ] {
        if count == 0 {
            problems.push(format!("{path} requires at least one item"));
        }
    }
    // Subset of checked-in Draft 2020-12 string/array shape constraints.
    // This is not a replacement for the full pinned offline schema engine.
    if record.display_name.trim().is_empty() || record.display_name.chars().count() > 140 {
        problems.push("display_name must contain 1..=140 nonblank characters".to_string());
    }
    for (index, requirement) in record.play.content_requirements.iter().enumerate() {
        if requirement.trim().is_empty() {
            problems.push(format!(
                "play.content_requirements[{index}] must be nonblank"
            ));
        }
    }
    for (index, claim) in record.rights_claims.iter().enumerate() {
        for (field, value) in [
            ("scope", claim.scope.as_str()),
            ("statement", claim.statement.as_str()),
        ] {
            if value.trim().is_empty() {
                problems.push(format!("rights_claims[{index}].{field} must be nonblank"));
            }
        }
        for (part, exception) in claim.exceptions_or_restrictions.iter().enumerate() {
            if exception.trim().is_empty() {
                problems.push(format!(
                    "rights_claims[{index}].exceptions_or_restrictions[{part}] must be nonblank"
                ));
            }
        }
    }
    for (index, item) in record.evidence.iter().enumerate() {
        for (field, value) in [
            ("subject_scope", item.subject_scope.as_str()),
            ("reviewer", item.reviewer.as_str()),
            ("observed_at", item.observed_at.as_str()),
        ] {
            if value.trim().is_empty() {
                problems.push(format!("evidence[{index}].{field} must be nonblank"));
            }
        }
        if !is_rfc3339_timestamp(&item.observed_at) {
            problems.push(format!(
                "evidence[{index}].observed_at must be an RFC3339 timestamp"
            ));
        }
    }
    for (field, value) in [
        ("reviewer", record.review.reviewer.as_str()),
        ("reviewed_at", record.review.reviewed_at.as_str()),
    ] {
        if value.trim().is_empty() {
            problems.push(format!("review.{field} must be nonblank"));
        }
    }
    if !is_rfc3339_timestamp(&record.review.reviewed_at) {
        problems.push("review.reviewed_at must be an RFC3339 timestamp".to_string());
    }
    for (index, event) in record.review.claim_history.iter().enumerate() {
        for (field, value) in [("reason", event.reason.as_str()), ("at", event.at.as_str())] {
            if value.trim().is_empty() {
                problems.push(format!(
                    "review.claim_history[{index}].{field} must be nonblank"
                ));
            }
        }
        if !is_rfc3339_timestamp(&event.at) {
            problems.push(format!(
                "review.claim_history[{index}].at must be an RFC3339 timestamp"
            ));
        }
    }
    for (index, permission) in record.permission_decisions.iter().enumerate() {
        if let Some(date) = &permission.decided_at {
            if !is_rfc3339_timestamp(date) {
                problems.push(format!(
                    "permission_decisions[{index}].decided_at must be an RFC3339 timestamp"
                ));
            }
        }
    }

    if record.schema != "free-energy.project/v0" {
        problems.push("unsupported catalog schema version".to_string());
    }
    if !is_project_id(&record.id) {
        problems.push("invalid project ID: expected lowercase namespace/slug".to_string());
    }
    match record.upstream.source_revision.as_deref() {
        Some(rev) if !is_full_git_sha(rev) => {
            problems.push("upstream source_revision must be a full 40-hex commit".to_string());
        }
        None if record
            .upstream
            .source_revision_reason
            .as_deref()
            .is_none_or(|r| r.trim().is_empty()) =>
        {
            problems.push("unpinned upstream source revision requires a reason".to_string());
        }
        _ => {}
    }

    for (field, url) in [
        ("upstream.discovery_url", &record.upstream.discovery_url),
        (
            "upstream.canonical_source_url",
            &record.upstream.canonical_source_url,
        ),
        (
            "upstream.contribution_url",
            &record.upstream.contribution_url,
        ),
        ("upstream.issue_url", &record.upstream.issue_url),
    ] {
        if !is_public_https_url(url) {
            problems.push(format!("inadmissible external URL: {field}"));
        }
    }
    let mut mirror_urls = HashSet::new();
    for (index, url) in record.upstream.read_only_mirror_urls.iter().enumerate() {
        if !mirror_urls.insert(url.as_str()) {
            problems.push(format!(
                "duplicate upstream read-only mirror URL at index {index}"
            ));
        }
        if !is_public_https_url(url) {
            problems.push(format!(
                "inadmissible external URL: upstream.read_only_mirror_urls[{index}]"
            ));
        }
    }
    if let Some(url) = &record.play.upstream_download_url {
        if !is_public_https_url(url) {
            problems.push("inadmissible external URL: play.upstream_download_url".to_string());
        }
    }

    let mut evidence = HashMap::new();
    for item in &record.evidence {
        check_slug(&mut problems, "evidence.evidence_id", &item.evidence_id);
        if !is_public_https_url(&item.url) {
            problems.push(format!(
                "inadmissible external URL: evidence {}",
                item.evidence_id
            ));
        }
        if evidence.insert(item.evidence_id.as_str(), item).is_some() {
            problems.push(format!("duplicate evidence ID: {}", item.evidence_id));
        }
        if item.evidence_kind == "PINNED_REPOSITORY_FILE"
            && (item.commit.as_deref().is_none_or(|s| !is_full_git_sha(s))
                || item.path.as_deref().is_none_or(|s| !is_relative_path(s))
                || item
                    .repository
                    .as_deref()
                    .is_none_or(|url| !is_public_https_url(url)))
        {
            problems.push(format!(
                "invalid pinned repository evidence: {}",
                item.evidence_id
            ));
        }
    }

    let mut claims = HashSet::new();
    let mut scopes = HashSet::new();
    for item in &record.rights_claims {
        check_slug(&mut problems, "rights_claims.claim_id", &item.claim_id);
        let mut referenced = HashSet::new();
        for id in &item.evidence_ids {
            check_slug(&mut problems, "rights_claims.evidence_ids", id);
            if !referenced.insert(id.as_str()) {
                problems.push(format!(
                    "claim {} duplicates evidence reference {id}",
                    item.claim_id
                ));
            }
        }
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
                    problems.push(format!(
                        "claim {} references invalidated evidence {id}",
                        item.claim_id
                    ));
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
        let mut reviewed = HashSet::new();
        for id in &item.review_evidence_ids {
            check_slug(
                &mut problems,
                "permission_decisions.review_evidence_ids",
                id,
            );
            if !reviewed.insert(id.as_str()) {
                problems.push(format!(
                    "duplicate permission review evidence reference {id}"
                ));
            }
        }
        if !scopes.contains(&(item.component.as_str(), item.scope.as_str())) {
            problems.push(format!(
                "permission scope has no rights claim: {} / {}",
                item.component, item.scope
            ));
        }
        if !permission_keys.insert((
            item.component.as_str(),
            item.scope.as_str(),
            item.use_kind.as_str(),
        )) {
            problems.push(format!(
                "duplicate permission action: {} / {} / {}",
                item.component, item.scope, item.use_kind
            ));
        }
        if item.decision_reason.trim().is_empty() {
            problems.push("permission decision requires a nonblank reason".to_string());
        }
        if item.decision == PermissionStatus::NotAuthorized
            && (item.decided_at.is_none()
                || item.reviewer.as_deref().is_none_or(|s| s.trim().is_empty()))
        {
            problems.push("NOT_AUTHORIZED requires review date and reviewer".to_string());
        }
        for id in &item.review_evidence_ids {
            match evidence.get(id.as_str()) {
                None => problems.push(format!("permission references missing evidence {id}")),
                Some(source) if source.currentness == "INVALIDATED" => {
                    problems.push(format!("permission references invalidated evidence {id}"));
                }
                _ => {}
            }
        }
    }

    if let Some(id) = record.adapter.test_evidence_id.as_deref() {
        check_slug(&mut problems, "adapter.test_evidence_id", id);
    }
    if let Some(id) = record.play.local_test_evidence_id.as_deref() {
        check_slug(&mut problems, "play.local_test_evidence_id", id);
    }

    // Local-play evidence must not be borrowed from an unrelated rights or
    // upstream snapshot, or attached to a status that makes no play-test claim.
    if let Some(id) = record.play.local_test_evidence_id.as_deref() {
        if evidence.get(id).is_none_or(|item| {
            item.evidence_kind != "FREE_ENERGY_LOCAL_TEST" || item.currentness == "INVALIDATED"
        }) {
            problems.push(format!(
                "local play references missing or ineligible test evidence {id}"
            ));
        }
        if record.play.status != PlayStatus::FreeEnergyVerified {
            problems.push("unverified play status must not assert local test evidence".to_string());
        }
    }

    // The manifest is a current snapshot, not an append-only ledger.
    // Each supersession source has at most one disposition; known IDs alone
    // cannot justify cyclic, self-referential or contradictory history.
    let mut history_sources = HashSet::new();
    let mut history_links = HashMap::new();
    for event in &record.review.claim_history {
        check_slug(
            &mut problems,
            "review.claim_history.old_claim_id",
            &event.old_claim_id,
        );
        if let Some(new_id) = event.new_claim_id.as_deref() {
            check_slug(&mut problems, "review.claim_history.new_claim_id", new_id);
        }
        if !claims.contains(event.old_claim_id.as_str()) {
            problems.push(format!(
                "supersession references unknown old claim {}",
                event.old_claim_id
            ));
        }
        if !history_sources.insert(event.old_claim_id.as_str()) {
            problems.push(format!(
                "duplicate supersession source {}",
                event.old_claim_id
            ));
        }
        if let Some(new_id) = event.new_claim_id.as_deref() {
            if !claims.contains(new_id) {
                problems.push(format!(
                    "supersession references unknown new claim {new_id}"
                ));
            }
            if new_id == event.old_claim_id {
                problems.push(format!("claim self-supersession: {new_id}"));
            }
            history_links
                .entry(event.old_claim_id.as_str())
                .or_insert(new_id);
        }
    }
    let mut examined = HashSet::new();
    for origin in history_links.keys().copied() {
        if examined.contains(origin) {
            continue;
        }
        let mut chain = HashSet::new();
        let mut node = origin;
        while let Some(&next) = history_links.get(node) {
            if !chain.insert(node) {
                problems.push(format!("cyclic claim supersession at {node}"));
                break;
            }
            if examined.contains(node) {
                break;
            }
            node = next;
        }
        examined.extend(chain);
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
        problems.push(
            "TESTED adapter requires a separately implemented conformance contract".to_string(),
        );
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
    fn unavailable_status_uses_schema_spelling() {
        let admitted = changed(LUANTI, |record| {
            record["play"]["status"] = json!("UNAVAILABLE_EVIDENCED");
        });
        assert!(
            validate_manifest(&admitted).is_ok(),
            "the published v5 schema status must deserialize"
        );

        let forged = changed(LUANTI, |record| {
            record["play"]["status"] = json!("UNAVAILABLE");
        });
        assert!(
            validate_manifest(&forged).is_err(),
            "the unpublished short alias must not be admitted"
        );
    }

    #[test]
    fn schema_vocabulary_rejects_unsupported_statuses_and_components() {
        // Each mutation is a syntactically valid JSON string and must be
        // rejected on its own, not just because another field becomes invalid.
        for (pointer, field) in [
            ("/kind", "project.kind"),
            ("/rights_claims/0/claim_kind", "rights_claims[0].claim_kind"),
            ("/rights_claims/0/component", "rights_claims[0].component"),
            ("/rights_claims/0/scope_kind", "rights_claims[0].scope_kind"),
            ("/rights_claims/0/status", "rights_claims[0].status"),
            (
                "/permission_decisions/0/component",
                "permission_decisions[0].component",
            ),
            ("/permission_decisions/0/use", "permission_decisions[0].use"),
            ("/evidence/0/evidence_kind", "evidence[0].evidence_kind"),
            ("/evidence/0/currentness", "evidence[0].currentness"),
            ("/review/record_status", "review.record_status"),
        ] {
            let forged = changed(LUANTI, |v| {
                *v.pointer_mut(pointer).expect("existing catalog field") = json!("FORGED_STATUS");
            });
            let errors = validate_manifest(&forged).unwrap_err();
            assert!(
                errors.iter().any(|error| error.contains(field)),
                "{pointer} escaped schema vocabulary admission: {errors:?}"
            );
        }
        for manifest in [LUANTI, OPENRA, VELOREN] {
            assert!(validate_manifest(manifest).is_ok());
        }
    }

    #[test]
    fn project_id_admission_matches_published_schema_pattern() {
        for id in ["free-energy/luanti", "a/b", "a0-b9/z-"] {
            let mutated = changed(LUANTI, |value| value["id"] = json!(id));
            assert!(
                validate_manifest(&mutated).is_ok(),
                "expected admissible project ID: {id}"
            );
        }

        for id in [
            "", "luanti", "/game", "a/", "-a/b", "a/-b", "a/B", "A/b", "a/b/c", "a/b_c", "a/é",
            "a/b ",
        ] {
            let mutated = changed(LUANTI, |value| value["id"] = json!(id));
            let errors = validate_manifest(&mutated).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("invalid project ID")),
                "unexpected outcome for invalid project ID {id:?}: {errors:?}"
            );
        }
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
    fn invalidated_review_evidence_cannot_support_permission_decisions() {
        let revoked = changed(OPENRA, |manifest| {
            let mut review = manifest["evidence"][0].clone();
            review["evidence_id"] = json!("revoked-review-evidence");
            review["evidence_kind"] = json!("HUMAN_RIGHTS_REVIEW");
            review["currentness"] = json!("INVALIDATED");
            manifest["evidence"].as_array_mut().unwrap().push(review);
            manifest["permission_decisions"][0]["review_evidence_ids"] =
                json!(["revoked-review-evidence"]);
        });
        let errors = validate_manifest(&revoked).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|message| message.contains("permission references invalidated evidence")),
            "{errors:?}"
        );

        // Referencing a still-observed review record does not grant approval:
        // all existing project permission decisions remain REVIEW_REQUIRED.
        let observed = changed(OPENRA, |manifest| {
            let mut review = manifest["evidence"][0].clone();
            review["evidence_id"] = json!("observed-review-evidence");
            review["evidence_kind"] = json!("HUMAN_RIGHTS_REVIEW");
            review["currentness"] = json!("OBSERVED_AT");
            manifest["evidence"].as_array_mut().unwrap().push(review);
            manifest["permission_decisions"][0]["review_evidence_ids"] =
                json!(["observed-review-evidence"]);
        });
        assert!(validate_manifest(&observed).is_ok());
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
    fn percent_escaped_repository_paths_fail_closed_for_rights_and_evidence() {
        // This is admission policy, not a claim that a subsequent URL fetch
        // or filesystem consumer safely normalizes arbitrary repository paths.
        assert!(is_relative_path("assets/audio/music theme.ogg"));
        assert!(validate_manifest(LUANTI).is_ok());
        for path in [
            "assets/%2e%2e/private",
            "assets/%2Fadmin",
            "assets/%5cprivate",
            "assets/%252e%252e/private",
        ] {
            assert!(!is_relative_path(path), "ambiguous repository path: {path}");
            let rights = changed(LUANTI, |record| {
                record["rights_claims"][0]["scope_kind"] = json!("PATH");
                record["rights_claims"][0]["scope"] = json!(path);
            });
            let errors = validate_manifest(&rights).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("unsafe rights path scope")),
                "rights scope {path}: {errors:?}"
            );
            let evidence = changed(LUANTI, |record| {
                record["evidence"][0]["path"] = json!(path);
            });
            let errors = validate_manifest(&evidence).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.contains("invalid pinned repository evidence")),
                "evidence path {path}: {errors:?}"
            );
        }
    }

    #[test]
    fn unknown_top_level_keys_are_rejected() {
        let attack = changed(OPENRA, |v| {
            v["unknown_authority"] = json!(true);
        });
        assert!(validate_manifest(&attack).is_err());
    }
    #[test]
    fn claim_history_rejects_self_cycles_and_duplicate_predecessors() {
        let self_link = changed(LUANTI, |v| {
            v["review"]["claim_history"] = json!([{
                "old_claim_id":"code-lgpl", "new_claim_id":"code-lgpl",
                "reason":"invalid self", "at":"2026-10-08T00:00:00Z"
            }]);
        });
        let errors = validate_manifest(&self_link).unwrap_err();
        assert!(
            errors.iter().any(|e| e.contains("self-supersession")),
            "{errors:?}"
        );

        let cycle = changed(LUANTI, |v| {
            v["review"]["claim_history"] = json!([
                {"old_claim_id":"code-lgpl", "new_claim_id":"media-default",
                 "reason":"forward", "at":"2026-10-08T00:00:00Z"},
                {"old_claim_id":"media-default", "new_claim_id":"code-lgpl",
                 "reason":"backward", "at":"2026-10-08T00:00:01Z"}
            ]);
        });
        let errors = validate_manifest(&cycle).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("cyclic claim supersession")),
            "{errors:?}"
        );

        let duplicate = changed(LUANTI, |v| {
            v["review"]["claim_history"] = json!([
                {"old_claim_id":"code-lgpl", "new_claim_id":"media-default",
                 "reason":"first", "at":"2026-10-08T00:00:00Z"},
                {"old_claim_id":"code-lgpl", "new_claim_id":null,
                 "reason":"conflicting", "at":"2026-10-08T00:00:01Z"}
            ]);
        });
        let errors = validate_manifest(&duplicate).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("duplicate supersession source")),
            "{errors:?}"
        );

        let acyclic = changed(LUANTI, |v| {
            v["review"]["claim_history"] = json!([{
                "old_claim_id":"code-lgpl", "new_claim_id":"media-default",
                "reason":"valid direct link", "at":"2026-10-08T00:00:00Z"
            }]);
        });
        assert!(validate_manifest(&acyclic).is_ok());
    }

    fn assert_url_fixture(source: &str, expected_key: &str) {
        let fixture: Value = serde_json::from_str(source).expect("valid fixture JSON");
        for case in fixture["cases"].as_array().expect("cases array") {
            let url = case["url"].as_str().expect("url string");
            let expected = case[expected_key].as_bool().expect("expected boolean");
            assert_eq!(
                is_public_https_url(url),
                expected,
                "URL admission mismatch: {} ({url})",
                case["id"].as_str().expect("case id")
            );
        }
    }

    #[test]
    fn existing_url_admission_vectors_match_semantic_policy() {
        assert_url_fixture(
            include_str!("../fixtures/url-admission-v0.json"),
            "expected_admitted",
        );
        assert_url_fixture(
            include_str!("../fixtures/url-policy-semantic-v0.json"),
            "expected_allowed",
        );
    }

    #[test]
    fn authored_semantic_identity_fixture_covers_all_cases() {
        let base: Value = serde_json::from_str(LUANTI).expect("Luanti baseline");
        let suite: Value =
            serde_json::from_str(include_str!("../fixtures/semantic-id-resolution-v0.json"))
                .expect("semantic ID fixture");
        let cases = suite["cases"].as_array().expect("fixture cases");
        assert_eq!(cases.len(), 13, "all authored semantic ID cases must run");
        for case in cases {
            let mut record = base.clone();
            for change in case["changes"].as_array().expect("case changes") {
                assert_eq!(change["op"].as_str(), Some("replace"));
                let pointer = change["pointer"].as_str().expect("JSON pointer");
                *record.pointer_mut(pointer).expect("existing target") = change["value"].clone();
            }
            let expected = case["expected_semantic_valid"]
                .as_bool()
                .expect("expected verdict");
            let actual = validate_manifest(&record.to_string()).is_ok();
            assert_eq!(actual, expected, "semantic fixture {}", case["id"]);
        }
    }

    #[test]
    fn pins_and_repository_coordinates_must_be_canonical() {
        let uppercase = changed(LUANTI, |v| {
            v["evidence"][0]["commit"] = json!("9A1B92D0D4D2C47FCED18E6077722C6301EB04F5");
        });
        assert!(validate_manifest(&uppercase).is_err());
        let empty_repo = changed(LUANTI, |v| {
            v["evidence"][0]["repository"] = json!("");
        });
        assert!(validate_manifest(&empty_repo).is_err());
    }

    #[test]
    fn every_external_link_surface_is_checked() {
        let upstream = changed(LUANTI, |v| {
            v["upstream"]["contribution_url"] = json!("javascript:alert(1)");
        });
        assert!(validate_manifest(&upstream).is_err());

        let download = changed(OPENRA, |v| {
            v["play"]["upstream_download_url"] = json!("https://127.0.0.1/secret");
        });
        assert!(validate_manifest(&download).is_err());

        let evidence = changed(VELOREN, |v| {
            v["evidence"][0]["url"] = json!("https://user@example.org/");
        });
        assert!(validate_manifest(&evidence).is_err());

        let mirror = changed(VELOREN, |v| {
            v["upstream"]["read_only_mirror_urls"][0] = json!("http://github.com/veloren");
        });
        assert!(validate_manifest(&mirror).is_err());
    }

    #[test]
    fn percent_encoded_delimiters_and_double_encoding_fail_closed() {
        // An external link can cross parse/redirect boundaries after decoding
        // even if its original HTTPS authority looks syntactically benign.
        for encoded in [
            "%2f", "%2F", "%3f", "%3F", "%23", "%40", "%3a", "%3A", "%5c", "%5C", "%25", "%252f",
        ] {
            let url = format!("https://example.org/a{encoded}b");
            assert!(
                !is_public_https_url(&url),
                "accepted encoded delimiter: {url}"
            );
        }

        // Encoded ordinary text still works in existing public links.
        for url in [
            "https://github.com/search?q=game%20engine",
            "https://example.org/Project%20Files/readme",
            "https://example.org/readme%7Ecurrent",
        ] {
            assert!(is_public_https_url(url), "rejected safe encoding: {url}");
        }

        let forged_upstream = changed(OPENRA, |record| {
            record["upstream"]["contribution_url"] =
                json!("https://github.com/OpenRA%2fadmin/commands");
        });
        assert!(validate_manifest(&forged_upstream).is_err());

        let forged_evidence = changed(LUANTI, |record| {
            record["evidence"][0]["url"] =
                json!("https://github.com/luanti-org%252fprivate/README");
        });
        assert!(validate_manifest(&forged_evidence).is_err());
    }
}
