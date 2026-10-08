//! Regression for display-direction spoofing in evidence-bearing catalog URLs.
//! No DNS, redirects, external network, or rights clearance is involved.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const PILOT: &str = include_str!("../projects/luanti.json");

fn changed(pointer: &str, url: &str) -> String {
    let mut record: Value = serde_json::from_str(PILOT).expect("valid pilot JSON");
    *record
        .pointer_mut(pointer)
        .expect("existing URL field in pilot") = json!(url);
    record.to_string()
}

fn rejected(url: &str, path: &str) {
    let record = changed(path, url);
    let errors = validate_manifest(&record)
        .expect_err("untrusted bidirectional-control URL was admitted");
    assert!(
        errors.iter().any(|error| error.contains("inadmissible external URL")),
        "{path}: {url:?}: {errors:?}"
    );
}

#[test]
fn raw_bidi_formatting_controls_cannot_spoof_upstream_links() {
    for character in [
        '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}',
        '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}',
        '\u{2068}', '\u{2069}',
    ] {
        let url = format!("https://github.com/upstream/{character}README.md");
        rejected(&url, "/upstream/discovery_url");
    }
}

#[test]
fn percent_encoded_bidi_controls_cannot_bypass_the_url_guard() {
    for encoded in [
        "%D8%9C", "%E2%80%8E", "%E2%80%8F", "%E2%80%AA",
        "%E2%80%AB", "%E2%80%AC", "%E2%80%AD", "%E2%80%AE",
        "%E2%81%A6", "%E2%81%A7", "%E2%81%A8", "%E2%81%A9",
    ] {
        let url = format!("https://github.com/upstream/{encoded}README.md");
        rejected(&url, "/upstream/discovery_url");
    }
}

#[test]
fn evidence_urls_and_mirror_links_use_the_same_fail_closed_guard() {
    let spoof = "https://github.com/upstream/%E2%80%AEreadme";
    rejected(spoof, "/evidence/0/url");

    let mut record: Value = serde_json::from_str(PILOT).unwrap();
    record["upstream"]["read_only_mirror_urls"] = json!([spoof]);
    let errors = validate_manifest(&record.to_string()).unwrap_err();
    assert!(
        errors.iter().any(|error| error.contains("read_only_mirror_urls")),
        "{errors:?}"
    );
}

#[test]
fn harmless_unicode_and_valid_utf8_escapes_remain_admissible() {
    for url in [
        "https://github.com/luanti-org/luanti",
        "https://github.com/luanti-org/caf%C3%A9",
        "https://github.com/luanti-org/café",
        "https://github.com/luanti-org/luanti?source=public#readme",
    ] {
        let record = changed("/upstream/discovery_url", url);
        let errors = validate_manifest(&record);
        assert!(errors.is_ok(), "{url:?}: {errors:?}");
    }
}

#[test]
fn invalid_percent_decoded_utf8_must_not_be_displayed_as_valid_evidence() {
    rejected("https://github.com/upstream/%E2%80", "/upstream/discovery_url");
    rejected("https://github.com/upstream/%FF", "/evidence/0/url");
}
