//! Reject invisible identity spoofing in scoped Git repository paths.
//! Pure manifest admission; this does not authenticate third-party rights.
use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");

fn pilot() -> Value {
    serde_json::from_str(LUANTI).expect("valid pilot manifest")
}

#[test]
fn rights_paths_fail_closed_for_invisible_format_codepoints() {
    assert!(validate_manifest(LUANTI).is_ok());
    for invisible in [
        '\u{00ad}', '\u{034f}', '\u{180e}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}',
        '\u{feff}',
    ] {
        let mut input = pilot();
        input["rights_claims"][0]["scope_kind"] = json!("PATH");
        input["rights_claims"][0]["scope"] = json!(format!("assets/LI{invisible}CENSE.txt"));
        let errors =
            validate_manifest(&input.to_string()).expect_err("invisible rights path accepted");
        assert!(
            errors
                .iter()
                .any(|message| message.contains("unsafe rights path scope")),
            "U+{:04X} not rejected as a rights path: {errors:?}",
            invisible as u32
        );
    }
}

#[test]
fn pinned_evidence_paths_fail_closed_even_when_commit_is_valid() {
    for invisible in [
        '\u{00ad}', '\u{034f}', '\u{180e}', '\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}',
        '\u{feff}',
    ] {
        let mut input = pilot();
        input["evidence"][0]["path"] = json!(format!("docs/LICE{invisible}NSE.txt"));
        let errors =
            validate_manifest(&input.to_string()).expect_err("invisible pinned path accepted");
        assert!(
            errors
                .iter()
                .any(|message| message.contains("invalid pinned repository evidence")),
            "U+{:04X} not rejected as pinned evidence: {errors:?}",
            invisible as u32
        );
    }
}

#[test]
fn ordinary_scoped_and_pinned_paths_still_pass() {
    let mut input = pilot();
    input["rights_claims"][0]["scope_kind"] = json!("PATH");
    input["rights_claims"][0]["scope"] = json!("assets/LICENSE.txt");
    input["evidence"][0]["path"] = json!("docs/LICENSE.txt");
    // A changed rights scope also changes which scope each related decision
    // refers to; preserve semantic consistency in this positive control.
    for decision in input["permission_decisions"].as_array_mut().unwrap() {
        if decision["component"] == "CODE" {
            decision["scope"] = json!("assets/LICENSE.txt");
        }
    }
    validate_manifest(&input.to_string())
        .unwrap_or_else(|errors| panic!("ordinary paths rejected: {errors:?}"));
}
