//! Standalone offline Rust ad-hoc research fixture validator (issue #71).
//!
//! Implements the same 47-case semantic decisions as the frozen historical
//! fixture integration test, without runtime Python or network access.
//! NOTE: canonical SHA-256 identity parity is not established by this CLI.


use serde_json::{json, Map, Value};
use std::collections::HashSet;

const SEMANTIC_FIELDS: [&str; 16] = [
    "packet_schema",
    "topic",
    "authoritative_baseline",
    "unique_nondefault_sources",
    "external_sources",
    "observations",
    "derived_conclusions",
    "predictions",
    "unknowns",
    "contradictions",
    "stale_source_warnings",
    "discovery_vocabulary",
    "affected_packages",
    "proposed_deltas",
    "unresolved_questions",
    "useful_next_actions",
];

const GROUPS: [(&str, usize); 6] = [
    ("publication_cases", 10),
    ("identity_cases", 11),
    ("source_cases", 9),
    ("recovery_cases", 7),
    ("concurrency_cases", 5),
    ("packet_field_cases", 5),
];

fn flag(case: &Value, key: &str, default: bool) -> Result<bool, String> {
    match case.get(key) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(format!("{key} must be a Boolean")),
    }
}

fn number(case: &Value, key: &str, default: usize) -> Result<usize, String> {
    match case.get(key) {
        None => Ok(default),
        Some(Value::Number(value)) => value
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| format!("{key} must be a nonnegative integer")),
        Some(_) => Err(format!("{key} must be a nonnegative integer")),
    }
}

fn text<'a>(case: &'a Value, key: &str) -> Result<Option<&'a str>, String> {
    match case.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

fn packet<'a>(
    case: &'a Value,
    templates: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a Value>, String> {
    if let Some(value) = case.get(field) {
        return Ok((!value.is_null()).then_some(value));
    }
    let reference = format!("{field}_ref");
    match text(case, &reference)? {
        None => Ok(None),
        Some(name) => templates
            .get(name)
            .map(Some)
            .ok_or_else(|| format!("unknown packet template: {name}")),
    }
}

fn required_packet<'a>(
    case: &'a Value,
    templates: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Value, String> {
    packet(case, templates, field)?.ok_or_else(|| format!("missing {field}"))
}

fn projection(packet: &Value) -> Result<Value, String> {
    let object = packet
        .as_object()
        .ok_or_else(|| "packet must be an object".to_owned())?;
    let mut result = Map::new();
    for key in SEMANTIC_FIELDS {
        result.insert(
            key.to_owned(),
            object
                .get(key)
                .ok_or_else(|| format!("missing semantic packet field: {key}"))?
                .clone(),
        );
    }
    Ok(Value::Object(result))
}

// Comparing the complete projected JSON tree is sufficient for deciding
// equality in these fixtures. This does NOT produce or verify the Python
// canonical-JSON/SHA-256 identity bytes; that separate obligation remains #71.
fn same_packet(left: &Value, right: &Value) -> Result<bool, String> {
    Ok(projection(left)? == projection(right)?)
}

fn matching_packets<'a>(
    case: &'a Value,
    templates: &'a Map<String, Value>,
) -> Result<Vec<&'a Value>, String> {
    if let Some(references) = case.get("matching_packet_refs") {
        let references = references
            .as_array()
            .ok_or_else(|| "matching_packet_refs must be an array".to_owned())?;
        return references
            .iter()
            .map(|reference| {
                let key = reference
                    .as_str()
                    .ok_or_else(|| "matching packet reference must be a string".to_owned())?;
                templates
                    .get(key)
                    .ok_or_else(|| format!("unknown packet template: {key}"))
            })
            .collect();
    }
    let Some(values) = case.get("matching_packet_payloads") else {
        return Ok(Vec::new());
    };
    values
        .as_array()
        .ok_or_else(|| "matching_packet_payloads must be an array".to_owned())
        .map(|values| values.iter().collect())
}

fn publication(case: &Value, templates: &Map<String, Value>) -> Result<Value, String> {
    if !flag(case, "inventory_complete", false)? {
        return Ok(json!("BLOCK_INVENTORY_UNKNOWN"));
    }
    if !flag(case, "schema_compatible", false)? {
        return Ok(json!("BLOCK_SCHEMA_INCOMPATIBLE"));
    }
    if !flag(case, "authority_current", false)? {
        return Ok(json!("BLOCK_AUTHORITY_STALE"));
    }
    let Some(candidate) = packet(case, templates, "packet")? else {
        return Ok(json!("BLOCK_PACKET_IDENTITY_UNKNOWN"));
    };
    projection(candidate)?;
    let count = number(case, "matching_packets", 0)?;
    if count == 0 {
        return Ok(json!("CREATE"));
    }
    if count == 1 {
        let Some(existing) = packet(case, templates, "matching_packet")? else {
            return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
        };
        if !same_packet(candidate, existing)? {
            return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
        }
        return Ok(json!(if flag(case, "matching_complete", false)? {
            "ADOPT_EXISTING"
        } else {
            "RECONCILE_INCOMPLETE"
        }));
    }
    let matches = matching_packets(case, templates)?;
    if matches.len() != count {
        return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
    }
    for existing in matches {
        if !same_packet(candidate, existing)? {
            return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
        }
    }
    Ok(json!("CANONICALIZE_DUPLICATES"))
}

fn identity(case: &Value, templates: &Map<String, Value>) -> Result<Value, String> {
    let id = text(case, "id")?.ok_or_else(|| "missing identity case id".to_owned())?;
    let result = match id {
        "same-semantic-retry"
        | "presentation-only-variance-same-identity"
        | "changed-content-new-identity"
        | "changed-source-new-identity" => {
            let left = required_packet(case, templates, "packet")?;
            let right = required_packet(case, templates, "retry_packet")?;
            if same_packet(left, right)? {
                "SAME_LINEAGE"
            } else {
                "NEW_LINEAGE"
            }
        }
        "temporary-cannot-claim-persistent-state"
        | "shared-transport-principal-not-persistent-identity" => {
            if flag(case, "claims_persistent_identity", false)? {
                "REJECT_IDENTITY_ESCALATION"
            } else {
                "OK"
            }
        }
        "packet-storage-not-policy" => {
            if flag(case, "packet_complete", false)? && !flag(case, "canonicalized", false)? {
                "EVIDENCE_ONLY"
            } else {
                "CANONICAL"
            }
        }
        "recursive-derivative-not-independent" => {
            if flag(case, "same_upstream_lineage", false)? {
                "SAME_LINEAGE"
            } else {
                "INDEPENDENT"
            }
        }
        "pure-research-branch-creation" => {
            if flag(case, "pure_research", false)? && flag(case, "attempts_git_branch", false)? {
                "REJECT_BRANCH_CREATION"
            } else {
                "OK"
            }
        }
        "exceptional-existing-surface-without-authority" => {
            if flag(case, "explicit_existing_surface_authority", false)? {
                "MUTATION_ELIGIBLE"
            } else {
                "READ_ONLY"
            }
        }
        "exceptional-mutation-active-collision" => {
            if !flag(case, "explicit_existing_surface_authority", false)? {
                "READ_ONLY"
            } else if flag(case, "active_overlap", false)? && !flag(case, "safe_takeover", false)? {
                "YIELD"
            } else {
                "MUTATION_ELIGIBLE"
            }
        }
        other => return Err(format!("unknown identity case: {other}")),
    };
    Ok(json!(result))
}

fn source(case: &Value) -> Result<Value, String> {
    let result = if !flag(case, "default_head_resolved", false)? {
        "UNKNOWN"
    } else if flag(case, "default_head_moved", false)? {
        "REVALIDATE"
    } else if flag(case, "source_ref_deleted", false)? {
        if flag(case, "durable_content_identity", false)? {
            "PRESERVE_PROVENANCE"
        } else {
            "UNKNOWN"
        }
    } else if flag(case, "same_path_competes", false)? {
        "USE_DEFAULT_AUTHORITY"
    } else if flag(case, "nondefault_unique_proven", false)? {
        "CONSUME_UNIQUE_DELTA"
    } else {
        "SKIP_NO_UNIQUE_DELTA"
    };
    Ok(json!(result))
}

fn recovery(case: &Value, templates: &Map<String, Value>) -> Result<Value, String> {
    if !flag(case, "authority_current", false)? {
        return Ok(json!(if flag(case, "created", false)? {
            "READ_ONLY_RECONCILE"
        } else {
            "BLOCK_AUTHORITY_STALE"
        }));
    }
    if !flag(case, "inventory_complete", true)? {
        return Ok(json!("BLOCK_INVENTORY_UNKNOWN"));
    }
    if !flag(case, "schema_compatible", false)? {
        return Ok(json!("BLOCK_SCHEMA_INCOMPATIBLE"));
    }
    let Some(candidate) = packet(case, templates, "packet")? else {
        return Ok(json!("BLOCK_PACKET_IDENTITY_UNKNOWN"));
    };
    projection(candidate)?;
    if flag(case, "created", false)? {
        let Some(existing) = packet(case, templates, "matching_packet")? else {
            return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
        };
        if !same_packet(candidate, existing)? {
            return Ok(json!("BLOCK_LOCATOR_CONFLICT"));
        }
        return Ok(json!(if flag(case, "matching_complete", false)? {
            "ADOPT_EXISTING"
        } else {
            "RECONCILE_INCOMPLETE"
        }));
    }
    Ok(json!("RECONCILE_THEN_CREATE_IF_ABSENT"))
}

fn concurrent(case: &Value) -> Result<Value, String> {
    let mut artifacts = number(case, "initial_packets", 0)?;
    if !flag(case, "authority_current", false)? {
        return Ok(json!({
            "disposition": "BLOCK_AUTHORITY_STALE",
            "provider_artifacts": artifacts,
            "canonical_lineages": artifacts
        }));
    }
    if !flag(case, "same_semantic_identity", false)? {
        return Ok(json!({
            "disposition": "DISTINCT_LINEAGES",
            "provider_artifacts": artifacts,
            "canonical_lineages": artifacts
        }));
    }
    let strategy = text(case, "strategy")?.ok_or_else(|| "missing strategy".to_owned())?;
    let publishers = number(case, "publishers", 2)?;
    let mut lineages = usize::from(artifacts > 0);
    match strategy {
        "atomic_unique" | "serialized" => {
            if artifacts == 0 && publishers > 0 {
                artifacts = 1;
                lineages = 1;
            }
        }
        "reconcile_after_create" => {
            artifacts = artifacts
                .checked_add(publishers)
                .ok_or_else(|| "artifact counter overflow".to_owned())?;
            if !flag(case, "inventory_complete", false)?
                || !flag(case, "semantic_identity_indexed", false)?
            {
                return Ok(json!({
                    "disposition": "BLOCK_INVENTORY_UNKNOWN",
                    "provider_artifacts": artifacts,
                    "canonical_lineages": lineages.max(artifacts)
                }));
            }
            lineages = usize::from(artifacts > 0);
        }
        "blind_presearch" => {
            artifacts = artifacts
                .checked_add(publishers)
                .ok_or_else(|| "artifact counter overflow".to_owned())?;
            lineages = artifacts;
        }
        other => return Err(format!("unknown concurrency strategy: {other}")),
    }
    if flag(case, "lost_ack_retry", false)? {
        if strategy == "atomic_unique" || strategy == "serialized" {
            artifacts = artifacts.max(1);
            lineages = 1;
        } else if strategy == "reconcile_after_create" && !flag(case, "inventory_complete", true)? {
            return Ok(json!({
                "disposition": "BLOCK_INVENTORY_UNKNOWN",
                "provider_artifacts": artifacts,
                "canonical_lineages": lineages
            }));
        }
    }
    Ok(json!({
        "disposition": if lineages <= 1 {
            "ONE_CANONICAL_LINEAGE"
        } else {
            "BLOCK_UNSAFE_ADAPTER"
        },
        "provider_artifacts": artifacts,
        "canonical_lineages": lineages
    }))
}

fn packet_fields(case: &Value) -> Value {
    if [
        "stale_source_warnings",
        "discovery_vocabulary",
        "useful_next_actions",
    ]
    .iter()
    .all(|key| case.get(*key).is_some())
    {
        json!("PRESERVE_REQUIRED_FIELDS")
    } else {
        json!("REJECT_INCOMPLETE_PACKET")
    }
}

fn validate(data: &Value) -> Result<usize, Vec<String>> {
    let mut failures = Vec::new();
    if data.get("schema_version").and_then(Value::as_u64) != Some(3)
        || data.get("spec_version").and_then(Value::as_u64) != Some(1)
    {
        failures.push("unsupported ad-hoc research fixture schema/spec".to_owned());
    }
    let Some(templates) = data.get("packet_templates").and_then(Value::as_object) else {
        return Err(vec!["missing packet_templates object".to_owned()]);
    };
    let mut ids = HashSet::new();
    let mut total = 0;
    for (group, minimum) in GROUPS {
        let Some(cases) = data.get(group).and_then(Value::as_array) else {
            failures.push(format!("missing fixture group: {group}"));
            continue;
        };
        if cases.len() < minimum {
            failures.push(format!(
                "incomplete {group}: at least {minimum} cases, found {}",
                cases.len()
            ));
        }
        for case in cases {
            total += 1;
            let id = match text(case, "id") {
                Ok(Some(id)) if !id.trim().is_empty() => id.to_owned(),
                _ => {
                    failures.push(format!("{group}: missing or invalid case id"));
                    continue;
                }
            };
            if !ids.insert(id.clone()) {
                failures.push(format!("duplicate case id: {id}"));
            }
            let actual = match group {
                "publication_cases" => publication(case, templates),
                "identity_cases" => identity(case, templates),
                "source_cases" => source(case),
                "recovery_cases" => recovery(case, templates),
                "concurrency_cases" => concurrent(case),
                "packet_field_cases" => Ok(packet_fields(case)),
                _ => unreachable!("fixed fixture group"),
            };
            match (actual, case.get("expected")) {
                (Ok(actual), Some(expected)) if &actual == expected => {}
                (Ok(actual), expected) => {
                    failures.push(format!("{id}: expected {expected:?}, computed {actual}"));
                }
                (Err(error), _) => failures.push(format!("{id}: {error}")),
            }
        }
    }
    failures.sort();
    if failures.is_empty() {
        Ok(total)
    } else {
        Err(failures)
    }
}

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn run() -> Result<usize, String> {
    let mut args = env::args_os().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("Phase0/fixtures/ad-hoc-research-spec1.json"));
    if args.next().is_some() {
        return Err("usage: ad_hoc_research [fixture.json]".to_owned());
    }
    let data = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let fixture: Value = serde_json::from_str(&data)
        .map_err(|error| format!("invalid fixture {}: {error}", path.display()))?;
    validate(&fixture).map_err(|errors| errors.join("\n"))
}

fn main() {
    match run() {
        Ok(count) => println!("ad-hoc research fixtures (Rust): {count} passed"),
        Err(error) => {
            eprintln!("FAIL: {error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod cli_semantic_tests {
    use super::*;

    const BASELINE: &str = include_str!("../../../fixtures/ad-hoc-research-spec1.json");

    #[test]
    fn baseline_47_cases_execute_without_reading_expected_as_oracle() {
        let fixture: Value = serde_json::from_str(BASELINE).expect("valid baseline");
        assert_eq!(validate(&fixture).unwrap(), 47);
    }

    #[test]
    fn changed_authority_and_forged_expected_both_fail() {
        let mut fixture: Value = serde_json::from_str(BASELINE).expect("valid baseline");
        fixture["publication_cases"][0]["authority_current"] = json!(false);
        assert!(validate(&fixture).is_err());

        let mut fixture: Value = serde_json::from_str(BASELINE).expect("valid baseline");
        fixture["identity_cases"][0]["expected"] = json!("NEW_LINEAGE");
        assert!(validate(&fixture).is_err());
    }
}
