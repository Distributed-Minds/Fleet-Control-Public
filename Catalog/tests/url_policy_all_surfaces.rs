//! Negative and positive URL-admission checks for the catalog's less common
//! external-link fields. Uses the production typed boundary, not a test parser.
//! This establishes syntax policy only, not link authenticity or rights clearance.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const VELOREN: &str = include_str!("../projects/veloren.json");

const URL_FIELDS: [(&str, &str); 4] = [
    (
        "/upstream/contribution_url",
        "inadmissible external URL: upstream.contribution_url",
    ),
    (
        "/upstream/issue_url",
        "inadmissible external URL: upstream.issue_url",
    ),
    (
        "/upstream/read_only_mirror_urls/0",
        "inadmissible external URL: upstream.read_only_mirror_urls[0]",
    ),
    (
        "/evidence/0/repository",
        "invalid pinned repository evidence: cargo-snapshot",
    ),
];

fn changed_url(pointer: &str, url: &str) -> String {
    let mut record: Value = serde_json::from_str(VELOREN).expect("valid pilot JSON");
    *record
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("missing URL field {pointer}")) = json!(url);
    record.to_string()
}

#[test]
fn untouched_pilot_is_admitted_before_testing_url_mutations() {
    validate_manifest(VELOREN).expect("test baseline must be semantically valid");
}

#[test]
fn all_four_additional_external_url_fields_reject_hostile_inputs() {
    validate_manifest(VELOREN).expect("invalid baseline would mask URL regressions");

    for unsafe_url in [
        "http://github.com/example/repository",
        "https://user:token@github.com/example/repository",
        "https://127.0.0.1/private",
        "https://localhost/internal",
        "https://host.internal/path",
        "https://github.com:443/example",
        "https://github.com/%2Fprivate",
        "https://github.com/%2540",
        "https://github.com\\@evil.example/repository",
        "https://github.com/path\nheader",
    ] {
        for (field, diagnostic) in URL_FIELDS {
            let modified = changed_url(field, unsafe_url);
            let errors = validate_manifest(&modified)
                .expect_err(&format!("unsafe URL admitted in {field}: {unsafe_url:?}"));
            assert!(
                errors.iter().any(|error| error.contains(diagnostic)),
                "{field}: rejection must come from URL admission, got {errors:?}"
            );
        }
    }
}

#[test]
fn benign_https_urls_remain_admitted_at_every_additional_surface() {
    validate_manifest(VELOREN).expect("invalid baseline would mask positive controls");

    for (field, _) in URL_FIELDS {
        for safe_url in [
            "https://github.com/example/project",
            "https://gitlab.com/example/project/issues",
            "https://codeberg.org/example/project",
        ] {
            let result = validate_manifest(&changed_url(field, safe_url));
            assert!(
                result.is_ok(),
                "{field}: valid HTTPS URL rejected: {safe_url}: {:?}",
                result.err()
            );
        }
    }
}
