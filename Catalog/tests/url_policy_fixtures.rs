//! Black-box semantic conformance for the two authored v0 URL-policy fixture
//! families. The expected verdicts are checked against the real typed
//! manifest validator, not echoed or interpreted by a test-only URL parser.
//!
//! This is offline URL admission, not DNS resolution, redirect verification,
//! publisher authenticity, or redistribution permission (issue #60, spec 5).
use free_energy_catalog::validate_manifest;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;

const BASE: &str = include_str!("../projects/luanti.json");
const POLICY_CASES: &str = include_str!("../fixtures/url-policy-semantic-v0.json");
const ADMISSION_CASES: &str = include_str!("../fixtures/url-admission-v0.json");

#[derive(Debug, Deserialize)]
struct UrlCase {
    id: String,
    url: String,
    expected_allowed: Option<bool>,
    expected_admitted: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UrlSuite {
    cases: Vec<UrlCase>,
}

/// Exercise each entry point that could emit a clickable external link.
/// Testing the manifest boundary also catches an accidentally unwired policy.
const URL_FIELDS: [&str; 4] = [
    "/upstream/discovery_url",
    "/upstream/canonical_source_url",
    "/play/upstream_download_url",
    "/evidence/0/url",
];

fn check_suite(name: &str, fixture: &str, expected_count: usize) {
    let suite: UrlSuite = serde_json::from_str(fixture).expect("parse committed URL fixture");
    assert_eq!(
        suite.cases.len(),
        expected_count,
        "{name}: missing or unexpected fixture cases"
    );
    let mut seen = HashSet::new();
    let mut allow = 0;
    let mut deny = 0;

    // A valid manifest is necessary to isolate the URL admission decision:
    // malformed ambient data could otherwise make every negative case pass.
    assert!(
        validate_manifest(BASE).is_ok(),
        "pilot baseline must be valid"
    );

    for case in suite.cases {
        assert!(
            seen.insert(case.id.clone()),
            "{name}: duplicate URL case ID: {}",
            case.id
        );
        let admitted = match (case.expected_allowed, case.expected_admitted) {
            (Some(value), None) | (None, Some(value)) => value,
            _ => panic!("{name}/{}: exactly one verdict is required", case.id),
        };
        if admitted {
            allow += 1;
        } else {
            deny += 1;
        }

        for field in URL_FIELDS {
            let mut manifest: Value =
                serde_json::from_str(BASE).expect("parse unmodified Luanti pilot");
            *manifest
                .pointer_mut(field)
                .unwrap_or_else(|| panic!("missing tested URL field {field}")) = json!(case.url);
            let changed = serde_json::to_string(&manifest).expect("serialize test mutation");
            let result = validate_manifest(&changed);
            if admitted {
                assert!(
                    result.is_ok(),
                    "{name}/{}: allowed URL rejected at {field}: {:?}",
                    case.id,
                    result.err()
                );
            } else {
                let errors = result.expect_err(&format!(
                    "{name}/{}: unsafe URL accepted at {field}: {}",
                    case.id, case.url
                ));
                assert!(
                    errors
                        .iter()
                        .any(|error| error.starts_with("inadmissible external URL:")),
                    "{name}/{}: rejected for unrelated reasons at {field}: {errors:?}",
                    case.id
                );
            }
        }
    }

    assert!(allow > 0 && deny > 0, "{name}: both verdicts required");
}

#[test]
fn url_policy_semantic_fixture_cases_reach_real_validator() {
    check_suite("url-policy-semantic-v0", POLICY_CASES, 41);
}

#[test]
fn url_admission_fixture_cases_reach_real_validator() {
    check_suite("url-admission-v0", ADMISSION_CASES, 36);
}
