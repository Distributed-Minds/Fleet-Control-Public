//! Public catalog links must apply raw-space policy after URL percent decoding.
//! These are offline manifest-level checks, not DNS, license, or play verification.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const PILOT: &str = include_str!("../projects/luanti.json");

fn edited(pointer: &str, url: &str) -> String {
    let mut project: Value = serde_json::from_str(PILOT).expect("valid pilot JSON");
    *project.pointer_mut(pointer).expect("pilot external-link field") = json!(url);
    project.to_string()
}

#[test]
fn encoded_ascii_space_is_rejected_in_every_public_link_surface() {
    // Raw U+0020 is already prohibited; percent encoding must not bypass it.
    let surfaces = [
        ("/upstream/discovery_url", "upstream.discovery_url"),
        ("/upstream/canonical_source_url", "upstream.canonical_source_url"),
        ("/upstream/contribution_url", "upstream.contribution_url"),
        ("/upstream/issue_url", "upstream.issue_url"),
        ("/evidence/0/url", "evidence"),
        ("/play/upstream_download_url", "play.upstream_download_url"),
    ];
    for suffix in ["with%20space", "?q=with%20space", "#with%20space"] {
        let url = format!("https://github.com/luanti-org/luanti/{suffix}");
        for (pointer, field) in surfaces {
            let errors = validate_manifest(&edited(pointer, &url))
                .expect_err("encoded ASCII whitespace bypassed URL admission");
            assert!(
                errors.iter().any(|e| e.contains("inadmissible external URL") && e.contains(field)),
                "{pointer}: {url} returned {errors:?}"
            );
        }
    }
}

#[test]
fn encoded_visible_characters_and_plain_links_remain_admissible() {
    for url in [
        "https://github.com/luanti-org/luanti",
        "https://github.com/luanti-org/luanti/with%2Bplus",
        "https://github.com/luanti-org/luanti/%41",
        "https://github.com/luanti-org/luanti/caf%C3%A9",
        "https://github.com/luanti-org/luanti/?q=with%2Bplus",
    ] {
        let result = validate_manifest(&edited("/upstream/discovery_url", url));
        assert!(result.is_ok(), "{url}: {result:?}");
    }
}
