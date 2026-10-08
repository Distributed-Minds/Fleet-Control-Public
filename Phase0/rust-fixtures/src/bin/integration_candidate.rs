//! Typed structural/semantic oracle for the historical integration-candidate-v1 fixtures.
//!
//! This independently evaluates the six fixture dispositions, including order,
//! cardinality, constructor envelope, and stale-head rejection. It deliberately
//! does NOT yet verify Python-compatible canonical JSON SHA-256 candidate IDs,
//! actual Git merge construction, or multi-base topology. Those obligations
//! remain open in issue #71.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const HISTORICAL_FIXTURE: &str =
    include_str!("../../../fixtures/integration-candidate-v1.json");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Metadata {
    author: String,
    author_time: String,
    committer: String,
    committer_time: String,
    message: String,
    encoding: String,
    signature_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Candidate {
    schema_version: String,
    operation_kind: String,
    target_commit: String,
    source_commit: String,
    parents: Vec<String>,
    parent_count: usize,
    tree: String,
    metadata: Metadata,
    constructor_version: String,
    compatibility_basis: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    constructor_support: Vec<usize>,
    candidate: Candidate,
    expect: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StaleCase {
    name: String,
    predicted_target: String,
    live_target: String,
    predicted_source: String,
    live_source: String,
    expect: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: String,
    digest: String,
    cases: Vec<Case>,
    stale_head_cases: Vec<StaleCase>,
}

fn git_id(value: &str) -> bool {
    value.len() == 40
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_candidate(candidate: &Candidate, supported: &[usize]) -> Result<(), String> {
    if candidate.schema_version != "integration-candidate-v1"
        || candidate.operation_kind != "explicit-merge"
    {
        return Err("unsupported integration candidate schema/operation".to_owned());
    }
    if !git_id(&candidate.target_commit)
        || !git_id(&candidate.source_commit)
        || !git_id(&candidate.tree)
        || candidate.parents.iter().any(|parent| !git_id(parent))
    {
        return Err("invalid 40-character lowercase Git object identity".to_owned());
    }
    if candidate.parent_count != candidate.parents.len() || candidate.parent_count == 0 {
        return Err("parent_count disagrees with nonempty ordered parent array".to_owned());
    }
    if supported.is_empty() || supported.iter().any(|count| *count == 0) {
        return Err("empty or invalid constructor support set".to_owned());
    }
    let unique: HashSet<usize> = supported.iter().copied().collect();
    if unique.len() != supported.len() {
        return Err("duplicate constructor support cardinality".to_owned());
    }
    if candidate.constructor_version.trim().is_empty()
        || candidate.compatibility_basis.trim().is_empty()
    {
        return Err("missing constructor version/compatibility basis".to_owned());
    }
    Ok(())
}

fn normal_parents(candidate: &Candidate) -> bool {
    candidate.parents == [candidate.target_commit.as_str(), candidate.source_commit.as_str()]
}

fn disposition(case: &Case, normal: &Candidate) -> Result<String, String> {
    let c = &case.candidate;
    validate_candidate(c, &case.constructor_support)?;
    if !case.constructor_support.contains(&c.parent_count) {
        return Ok("UNSUPPORTED_PARENT_CARDINALITY".to_owned());
    }

    if c == normal && normal_parents(c) {
        return Ok("SUPPORTED".to_owned());
    }

    // Parent order is part of semantic candidate identity even when tree
    // and metadata are unchanged. Do not conflate this with a SHA-256 proof.
    let mut reversed = normal.clone();
    reversed.parents.reverse();
    if c == &reversed && c.parents != normal.parents {
        let original = serde_json::to_vec(normal).map_err(|e| e.to_string())?;
        let ordered = serde_json::to_vec(c).map_err(|e| e.to_string())?;
        if original == ordered {
            return Err("reversed parents did not change canonical payload".to_owned());
        }
        return Ok("SUPPORTED_DISTINCT_FROM:normal-two-parent".to_owned());
    }

    // A changed constructor can preserve the observed two-parent envelope,
    // but only when all other candidate fields (including parent order,
    // metadata and tree) are identical. This is a fixture-level compatibility
    // check, NOT external constructor approval.
    let mut original_envelope = c.clone();
    original_envelope.constructor_version = normal.constructor_version.clone();
    original_envelope.compatibility_basis = normal.compatibility_basis.clone();
    if original_envelope == *normal
        && c.constructor_version != normal.constructor_version
        && c.compatibility_basis != normal.compatibility_basis
    {
        return Ok("SUPPORTED_COMPATIBLE_WITH:normal-two-parent".to_owned());
    }

    Err("candidate has unsupported semantic difference from baseline".to_owned())
}

fn validate(fixture: &Fixture) -> Result<usize, String> {
    if fixture.schema_version != "integration-candidate-fixture-v1" || fixture.digest != "sha256" {
        return Err("unsupported fixture schema or declared digest".to_owned());
    }
    let required = [
        "normal-two-parent",
        "reversed-parents-same-tree",
        "unsupported-three-parent",
        "compatible-constructor-migration",
    ];
    let required_stale = ["target-moved", "source-moved"];
    if fixture.cases.len() != required.len()
        || fixture.stale_head_cases.len() != required_stale.len()
    {
        return Err("missing or extra integration fixture cases".to_owned());
    }

    let mut seen = HashSet::new();
    for case in &fixture.cases {
        if !required.contains(&case.name.as_str()) || !seen.insert(case.name.as_str()) {
            return Err(format!("unknown or duplicate candidate case: {}", case.name));
        }
    }
    let mut stale_seen = HashSet::new();
    for case in &fixture.stale_head_cases {
        if !required_stale.contains(&case.name.as_str())
            || !stale_seen.insert(case.name.as_str())
        {
            return Err(format!("unknown or duplicate stale-head case: {}", case.name));
        }
    }

    let normal = &fixture
        .cases
        .iter()
        .find(|case| case.name == "normal-two-parent")
        .ok_or("missing normal baseline")?
        .candidate;
    if !normal_parents(normal) || normal.parent_count != 2 {
        return Err("normal baseline has incorrect parent order/cardinality".to_owned());
    }
    for case in &fixture.cases {
        let actual =
            disposition(case, normal).map_err(|reason| format!("{}: {reason}", case.name))?;
        if case.expect != actual {
            return Err(format!(
                "{}: expected {:?}, computed {:?}",
                case.name, case.expect, actual
            ));
        }
    }
    for case in &fixture.stale_head_cases {
        for value in [
            &case.predicted_target,
            &case.live_target,
            &case.predicted_source,
            &case.live_source,
        ] {
            if !git_id(value) {
                return Err(format!("{}: malformed Git head identity", case.name));
            }
        }
        let actual = if case.predicted_target == case.live_target
            && case.predicted_source == case.live_source
        {
            "CURRENT"
        } else {
            "STALE_OR_INCOMPATIBLE"
        };
        if case.expect != actual {
            return Err(format!(
                "{}: expected {:?}, computed {:?}",
                case.name, case.expect, actual
            ));
        }
    }
    Ok(fixture.cases.len() + fixture.stale_head_cases.len())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 {
        eprintln!("usage: integration_candidate [fixture-path]");
        process::exit(2);
    }
    let path = args
        .first()
        .map(String::as_str)
        .unwrap_or("Phase0/fixtures/integration-candidate-v1.json");
    let result = fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|contents| serde_json::from_str::<Fixture>(&contents).map_err(|e| e.to_string()))
        .and_then(|fixture| validate(&fixture));
    match result {
        Ok(total) => println!(
            "integration candidate fixture envelope: {total} passed; SHA-256 identity parity NOT checked"
        ),
        Err(reason) => {
            eprintln!("FAIL: {reason}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Fixture {
        serde_json::from_str(HISTORICAL_FIXTURE).expect("historical fixture must parse")
    }

    #[test]
    fn original_six_cases_match_independent_envelopes() {
        assert_eq!(validate(&sample()).unwrap(), 6);
    }

    #[test]
    fn cannot_pass_by_dropping_required_case() {
        let mut fixture = sample();
        fixture.cases.pop();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn cannot_pass_by_duplicating_case_name() {
        let mut fixture = sample();
        fixture.cases[1].name = fixture.cases[0].name.clone();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn refuses_changed_expected_without_changed_semantics() {
        let mut fixture = sample();
        fixture.cases[0].expect = "UNSUPPORTED_PARENT_CARDINALITY".to_owned();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn rejects_baseline_parent_reordering() {
        let mut fixture = sample();
        fixture.cases[0].candidate.parents.reverse();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn refuses_silent_parent_count_mismatch() {
        let mut fixture = sample();
        fixture.cases[2].candidate.parent_count = 2;
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn detects_broadened_constructor_cardinality() {
        let mut fixture = sample();
        fixture.cases[2].constructor_support.push(3);
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn rejects_incompatible_constructor_migration_envelope() {
        let mut fixture = sample();
        fixture.cases[3].candidate.tree = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn detects_stale_head_becoming_current() {
        let mut fixture = sample();
        fixture.stale_head_cases[0].live_target =
            fixture.stale_head_cases[0].predicted_target.clone();
        assert!(validate(&fixture).is_err());
    }

    #[test]
    fn rejects_empty_stale_head_family() {
        let mut fixture = sample();
        fixture.stale_head_cases.clear();
        assert!(validate(&fixture).is_err());
    }
}
