//! Regression coverage for invisible-format identity spoofing in catalog
//! display metadata. This does not authenticate upstream sources or rights.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const VELOREN: &str = include_str!("../projects/veloren.json");

const FIELDS: [(&str, &str); 4] = [
    ("/display_name", "display_name"),
    ("/rights_claims/0/statement", "rights_claims[0].statement"),
    ("/evidence/0/reviewer", "evidence[0].reviewer"),
    ("/review/reviewer", "review.reviewer"),
];

fn with_value(pointer: &str, text: &str) -> String {
    let mut manifest: Value = serde_json::from_str(VELOREN).expect("valid fixture JSON");
    *manifest
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("missing fixture field: {pointer}")) = json!(text);
    manifest.to_string()
}

#[test]
fn invisible_formats_fail_closed_in_each_visible_field() {
    validate_manifest(VELOREN).expect("baseline manifest must validate");
    for marker in [
        '\u{00ad}', '\u{034f}', '\u{180e}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}',
        '\u{feff}',
    ] {
        for (pointer, label) in FIELDS {
            let text = format!("visible{marker}metadata");
            let raw = with_value(pointer, &text);
            // Both literal UTF-8 and JSON Unicode escapes must be rejected
            // after semantic JSON decoding, not merely by raw-text scanning.
            let escaped = raw.replace(marker, &format!("\\u{:04x}", marker as u32));
            for payload in [&raw, &escaped] {
                let errors = validate_manifest(payload)
                    .expect_err("invisible formatting character must not be admitted");
                assert!(
                    errors.iter().any(|error| error.contains(&format!(
                        "{label}: default-ignorable formatting control in visible metadata"
                    ))),
                    "{pointer} with U+{:04X}: unexpected errors {errors:?}",
                    marker as u32
                );
            }
        }
    }
}

#[test]
fn ordinary_visible_unicode_remains_valid_and_bidi_diagnostics_are_preserved() {
    for good in ["Café", "東京", "مرحبا", "Veloren – collaborative"] {
        for (pointer, _) in FIELDS {
            let result = validate_manifest(&with_value(pointer, good));
            assert!(
                result.is_ok(),
                "{pointer} rejected {good:?}: {:?}",
                result.err()
            );
        }
    }

    for (pointer, label) in FIELDS {
        let errors = validate_manifest(&with_value(pointer, "left\u{202e}right"))
            .expect_err("bidi formatting must remain prohibited");
        assert!(
            errors.iter().any(|error| error.contains(&format!(
                "{label}: bidirectional formatting control in visible metadata"
            ))),
            "{pointer}: existing bidi diagnostic changed: {errors:?}"
        );
    }
}
