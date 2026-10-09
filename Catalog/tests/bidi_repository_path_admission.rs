//! Black-box rights and pinned-evidence path admission regressions (#60 spec v5).
//! Reject display-spoofable bidirectional formatting characters without
//! forbidding ordinary non-ASCII repository filenames.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");

fn baseline() -> Value {
    serde_json::from_str(LUANTI).expect("authored manifest")
}

fn rights_path(value: &str) -> Value {
    let mut project = baseline();
    project["rights_claims"][0]["scope_kind"] = json!("PATH");
    project["rights_claims"][0]["scope"] = json!(value);
    // Preserve matching permissions so a rejection cannot be caused by an
    // unrelated broken rights/permission subject reference.
    for permission in project["permission_decisions"]
        .as_array_mut()
        .expect("permission decisions")
    {
        if permission["component"] == "CODE" {
            permission["scope"] = json!(value);
        }
    }
    project
}

fn evidence_path(value: &str) -> Value {
    let mut project = baseline();
    project["evidence"][0]["path"] = json!(value);
    // Keep the positive path test independent of permalink consistency:
    // a changed immutable path also requires a changed outward file link.
    let original = project["evidence"][0]["url"]
        .as_str()
        .expect("pilot evidence URL")
        .strip_suffix("LICENSE.txt")
        .expect("pilot URL points to LICENSE.txt")
        .to_owned();
    project["evidence"][0]["url"] = json!(format!("{original}{value}"));
    project
}

fn rejects_with(value: &Value, reason: &str) {
    let errors = validate_manifest(&value.to_string())
        .expect_err("display-spoofable repository path must not be admitted");
    assert!(
        errors.iter().any(|entry| entry.contains(reason)),
        "missing {reason:?} diagnostic: {errors:?}"
    );
}

#[test]
fn baseline_and_ordinary_unicode_repository_paths_are_admitted() {
    validate_manifest(LUANTI).expect("authored baseline");
    let path = "assets/café/音楽.ogg";
    validate_manifest(&rights_path(path).to_string()).expect("ordinary Unicode rights path");
    validate_manifest(&evidence_path(path).to_string()).expect("ordinary Unicode evidence path");
}

#[test]
fn directional_formatting_is_denied_in_rights_and_pinned_evidence_paths() {
    for cp in [
        0x061c, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066, 0x2067, 0x2068,
        0x2069,
    ] {
        let ch = char::from_u32(cp).expect("Unicode scalar");
        assert!(
            !ch.is_control(),
            "test needs non-control formatting codepoint"
        );
        let path = format!("assets/{ch}private/source.bin");
        rejects_with(&rights_path(&path), "unsafe rights path scope");
        rejects_with(&evidence_path(&path), "invalid pinned repository evidence");
    }
}
