//! Real typed manifest admission for invisible external-link formatting.
//! A successful syntax check does not authenticate a host or clear rights.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const VELOREN: &str = include_str!("../projects/veloren.json");

const SURFACES: [(&str, &str); 7] = [
    (
        "/upstream/discovery_url",
        "inadmissible external URL: upstream.discovery_url",
    ),
    (
        "/upstream/canonical_source_url",
        "inadmissible external URL: upstream.canonical_source_url",
    ),
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
        "/evidence/0/url",
        "inadmissible external URL: evidence cargo-snapshot",
    ),
    (
        "/evidence/0/repository",
        "invalid pinned repository evidence: cargo-snapshot",
    ),
];

fn with_url(pointer: &str, url: &str) -> String {
    let mut manifest: Value = serde_json::from_str(VELOREN).expect("valid baseline JSON");
    *manifest
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("missing {pointer}")) = json!(url);
    manifest.to_string()
}

#[test]
fn raw_and_encoded_invisible_formats_are_rejected_at_each_link_surface() {
    validate_manifest(VELOREN).expect("baseline must validate");
    let encoded = [
        "%C2%AD",    // SOFT HYPHEN
        "%CD%8F",    // COMBINING GRAPHEME JOINER
        "%E1%A0%8E", // MONGOLIAN VOWEL SEPARATOR
        "%E2%80%8B", // ZERO WIDTH SPACE
        "%E2%80%8C", // ZERO WIDTH NON-JOINER
        "%E2%80%8D", // ZERO WIDTH JOINER
        "%E2%81%A0", // WORD JOINER
        "%EF%BB%BF", // ZERO WIDTH NO-BREAK SPACE
    ];
    let raw = [
        '\u{00ad}', '\u{034f}', '\u{180e}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}',
        '\u{feff}',
    ];
    for (code, marker) in encoded.into_iter().zip(raw) {
        for unsafe_url in [
            format!("https://github.com/example/{code}path"),
            format!("https://github.com/example/{marker}path"),
        ] {
            for (surface, diagnostic) in SURFACES {
                let errors = validate_manifest(&with_url(surface, &unsafe_url)).expect_err(
                    &format!("invisible link component admitted in {surface}: {unsafe_url:?}"),
                );
                assert!(
                    errors.iter().any(|error| error.contains(diagnostic)),
                    "{surface}: expected URL-admission rejection, got {errors:?}"
                );
            }
        }
    }
}

#[test]
fn visible_unicode_and_benign_percent_encodings_remain_accepted() {
    validate_manifest(VELOREN).expect("baseline must validate");
    for safe in [
        "https://github.com/example/caf%C3%A9",
        "https://github.com/example/café",
        "https://github.com/example/東京",
        "https://github.com/example/%E2%9C%93",
    ] {
        for (surface, _) in SURFACES {
            let result = validate_manifest(&with_url(surface, safe));
            assert!(
                result.is_ok(),
                "{surface}: visible Unicode link rejected: {safe}: {:?}",
                result.err()
            );
        }
    }
}
