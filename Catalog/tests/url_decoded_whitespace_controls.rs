//! Fail-closed UTF-8 percent-decoded URL admission for catalog evidence.
//! No external network, DNS, rights clearance, or redirect resolution is used.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const PILOT: &str = include_str!("../projects/luanti.json");

fn edited(pointer: &str, url: &str) -> String {
    let mut project: Value = serde_json::from_str(PILOT).expect("valid pilot JSON");
    *project.pointer_mut(pointer).expect("existing pilot URL field") = json!(url);
    project.to_string()
}

#[test]
fn encoded_unicode_spaces_and_c1_controls_are_rejected_at_every_link_boundary() {
    // These sequences were previously accepted because raw inputs were
    // whitespace-checked, but decoded UTF-8 was checked only for bidi marks.
    let unsafe_sequences = [
        "%C2%85",     // U+0085 NEXT LINE (C1)
        "%C2%9F",     // U+009F APPLICATION PROGRAM COMMAND (C1)
        "%C2%A0",     // U+00A0 NO-BREAK SPACE
        "%E2%80%80",  // U+2000 EN QUAD
        "%E2%80%A8",  // U+2028 LINE SEPARATOR
        "%E2%80%A9",  // U+2029 PARAGRAPH SEPARATOR
        "%E2%80%AF",  // U+202F NARROW NO-BREAK SPACE
        "%E3%80%80",  // U+3000 IDEOGRAPHIC SPACE
        "%e2%80%a8",  // case-insensitive hex decoding
    ];
    let surfaces = [
        ("/upstream/discovery_url", "upstream.discovery_url"),
        ("/upstream/canonical_source_url", "upstream.canonical_source_url"),
        ("/upstream/contribution_url", "upstream.contribution_url"),
        ("/upstream/issue_url", "upstream.issue_url"),
        ("/evidence/0/url", "evidence"),
        ("/play/upstream_download_url", "play.upstream_download_url"),
    ];
    for encoded in unsafe_sequences {
        let url = format!("https://github.com/luanti-org/luanti/{encoded}README.md");
        for (pointer, field) in surfaces {
            let errors = validate_manifest(&edited(pointer, &url))
                .expect_err("percent-encoded Unicode space/control was admitted");
            assert!(
                errors.iter().any(|e| e.contains("inadmissible external URL") && e.contains(field)),
                "{encoded} at {pointer}: {errors:?}"
            );
        }
    }
}

#[test]
fn encoded_noncontrol_unicode_and_plain_upstream_urls_still_pass() {
    for url in [
        "https://github.com/luanti-org/luanti",
        "https://github.com/luanti-org/caf%C3%A9",
        "https://github.com/luanti-org/%E2%9C%93",
        "https://github.com/luanti-org/%F0%9F%8E%AE",
        "https://github.com/luanti-org/%41",
    ] {
        let outcome = validate_manifest(&edited("/upstream/discovery_url", url));
        assert!(outcome.is_ok(), "{url}: {outcome:?}");
    }
}
