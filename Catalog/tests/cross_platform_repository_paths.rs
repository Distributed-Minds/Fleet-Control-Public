//! Fail-closed admission of repository path aliases in evidence and rights scopes.
//! This verifies only metadata path identity: no filesystem accesses or licenses.

use free_energy_catalog::validate_manifest;
use serde_json::{json, Value};

const LUANTI: &str = include_str!("../projects/luanti.json");

fn candidate() -> Value {
    serde_json::from_str(LUANTI).expect("valid catalog pilot fixture")
}

fn must_reject(value: Value, diagnostic: &str) {
    let errors = validate_manifest(&value.to_string()).expect_err("path alias must be denied");
    assert!(
        errors.iter().any(|error| error.contains(diagnostic)),
        "missing diagnostic {diagnostic:?}: {errors:?}"
    );
}

#[test]
fn rights_path_scopes_reject_platform_aliased_names() {
    for path in [
        "assets/CON",
        "assets/con.txt",
        "docs/PRN.md",
        "audio/AUX.ogg",
        "assets/nul.",
        "audio/COM1.wav",
        "audio/LPT9.dat",
        "audio/COM¹.wav",
        "audio/LPT².dat",
        "sound/theme.ogg:alternate",
        "sound/file?.ogg",
        "assets/foo<bar>.png",
        "assets/file|other.dat",
        "docs/LICENSE.",
        "assets/dir./image.png",
    ] {
        let mut value = candidate();
        value["rights_claims"][0]["scope_kind"] = json!("PATH");
        value["rights_claims"][0]["scope"] = json!(path);
        must_reject(value, "unsafe rights path scope");
    }
}

#[test]
fn pinned_repository_evidence_rejects_the_same_aliases() {
    for path in [
        "CON",
        "src/NUL.json",
        "src/lpt1.test",
        "assets/COM3.wav",
        "audio/COM³.ogg",
        "LICENSE.",
        "docs/file:stream",
        "docs/file*copy",
        "docs/file\"quote",
        "dir/name>other",
    ] {
        let mut value = candidate();
        value["evidence"][0]["path"] = json!(path);
        must_reject(value, "invalid pinned repository evidence");
    }
}

#[test]
fn ordinary_nested_paths_remain_usable() {
    for path in [
        "assets/.well-known/source.json",
        "src/CONSOLE.md",
        "docs/AUXILIARY.ron",
        "audio/lpt10.ogg",
        "assets/CONTRACT.txt",
        "docs/README.md",
    ] {
        let mut value = candidate();
        value["evidence"][0]["path"] = json!(path);
        validate_manifest(&value.to_string())
            .unwrap_or_else(|errors| panic!("safe path {path:?}: {errors:?}"));
    }
    validate_manifest(LUANTI).expect("unchanged pilot must remain valid");
}
