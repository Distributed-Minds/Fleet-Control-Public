//! Offline, dependency-free structural checks for FREE ENERGY's static landing.
//!
//! Compile from the repository root:
//!   rustc --edition=2021 -D warnings tests/check_free_energy_site.rs -o /tmp/free-energy-site-check
//!   /tmp/free-energy-site-check
//!   rustc --edition=2021 --test tests/check_free_energy_site.rs -o /tmp/free-energy-site-tests
//!   /tmp/free-energy-site-tests
//!
//! Optional: pass --root PATH when not running from the repository root.
//! This checks source structure only; it cannot prove browser behavior, HTTP
//! availability, accessibility compliance, or deployment.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

const REPO: &str = "https://github.com/Distributed-Minds/Fleet-Control-Public";
const RELEASE: &str = "v0.1.2-phase0-preview";

fn tags(html: &str) -> Vec<&str> {
    html.split('<')
        .skip(1)
        .filter_map(|s| s.split_once('>').map(|(tag, _)| tag))
        .collect()
}

/// Match a real opening tag name at a tag boundary. HTML tag names are
/// ASCII case-insensitive and attributes may begin after tabs/newlines.
fn is_open_element(tag: &str, name: &str) -> bool {
    let end = tag
        .find(|c: char| c.is_ascii_whitespace() || c == '/')
        .unwrap_or(tag.len());
    tag[..end].eq_ignore_ascii_case(name)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let rest = tag.split_once(needle.as_str())?.1;
    Some(rest.split_once('"')?.0)
}

fn expect(errors: &mut Vec<String>, condition: bool, message: impl Into<String>) {
    if !condition {
        errors.push(message.into());
    }
}

fn validate(root: &Path) -> Vec<String> {
    let files = [
        ("index.html", root.join("docs/index.html")),
        ("styles.css", root.join("docs/styles.css")),
        ("README.md", root.join("docs/README.md")),
    ];
    let mut content = Vec::new();
    for (name, path) in &files {
        match fs::read_to_string(path) {
            Ok(value) => content.push(value),
            Err(error) => return vec![format!("Cannot read docs/{name}: {error}")],
        }
    }
    let html = &content[0];
    let css = &content[1];
    let readme = &content[2];
    let elements = tags(html);
    let mut errors = Vec::new();

    let ids: Vec<&str> = elements
        .iter()
        .filter_map(|tag| attribute(tag, "id"))
        .collect();
    let links: Vec<&str> = elements
        .iter()
        .filter(|tag| is_open_element(tag, "a"))
        .map(|tag| attribute(tag, "href").unwrap_or(""))
        .collect();
    let stylesheets: Vec<&str> = elements
        .iter()
        .filter(|tag| is_open_element(tag, "link") && attribute(tag, "rel") == Some("stylesheet"))
        .filter_map(|tag| attribute(tag, "href"))
        .collect();

    expect(&mut errors, html.contains("<html lang=\"en\">"), "Missing English HTML language");
    expect(
        &mut errors,
        html.contains("<meta name=\"viewport\""),
        "Missing viewport meta tag",
    );
    expect(
        &mut errors,
        elements.iter().filter(|tag| is_open_element(tag, "main")).count() == 1
            && ids.contains(&"main"),
        "Expected one main landmark with id=main",
    );
    expect(
        &mut errors,
        ids.len() == ids.iter().collect::<HashSet<_>>().len(),
        "Duplicate fragment ID",
    );
    expect(
        &mut errors,
        links.first().copied() == Some("#main"),
        "First link must skip to main",
    );
    expect(
        &mut errors,
        stylesheets == ["./styles.css"],
        "Expected one relative local stylesheet",
    );
    for forbidden in ["script", "iframe", "form", "object", "embed"] {
        expect(
            &mut errors,
            !elements.iter().any(|tag| is_open_element(tag, forbidden)),
            format!("Unexpected active/embedded element: {forbidden}"),
        );
    }
    for href in &links {
        if let Some(fragment) = href.strip_prefix('#') {
            expect(
                &mut errors,
                ids.contains(&fragment),
                format!("Unresolved internal fragment: {href}"),
            );
        } else {
            expect(
                &mut errors,
                href.starts_with("https://"),
                format!("Missing or non-HTTPS outbound link: {href}"),
            );
        }
    }

    let zip = format!(
        "{REPO}/releases/download/{RELEASE}/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip"
    );
    let release_page = format!("{REPO}/releases/tag/{RELEASE}");
    let guide = format!("{REPO}/blob/phase0/public-v0/GETTING-STARTED.md");
    let help = format!("{REPO}/blob/phase0/public-v0/HELP-A-PROJECT.md");
    let workflows = format!("{REPO}/blob/phase0/public-v0/WORKFLOW-GUIDES.md");

    for (url, explanation) in [
        (zip.as_str(), "Exact starter ZIP download link missing"),
        (release_page.as_str(), "Release overview link missing"),
        (guide.as_str(), "Corrected beginner setup guide missing"),
        (help.as_str(), "Existing-project contributor guide missing"),
        (workflows.as_str(), "Project workflow guide map missing"),
        ("https://github.com/Distributed-Minds/Fleet-Control-Public/discussions", "Public Discussions link missing"),
    ] {
        expect(&mut errors, links.contains(&url), explanation);
    }
    for (required, explanation) in [
        ("Download the starter ZIP", "Download CTA does not identify ZIP"),
        ("older setup guide", "Release archive age warning missing"),
        ("before installing or forking", "Corrected online guide warning missing"),
        ("Posting does not enroll a contributor", "Public contact enrollment boundary missing"),
        ("Do not post secrets", "Public contact confidentiality boundary missing"),
        ("PLAY / DISCOVER", "PLAY route missing"),
        ("HELP AN EXISTING PROJECT", "HELP route missing"),
        ("MAKE / REMIX", "MAKE route missing"),
        ("A verified playable catalog is still planned", "Missing catalog-not-shipped disclosure"),
        ("Want to run your own agent fleet?", "Separate fleet installation route missing"),
        ("You do not need it to help", "Fleet installation requirement is misleading"),
        ("FUTURE VISION", "Future vision label missing"),
        ("FUTURE PLATFORM", "Future platform label missing"),
        ("Not yet available", "Missing future-feature disclaimer"),
        ("Fleet-Control Phase0", "Current orchestration preview not identified"),
    ] {
        expect(&mut errors, html.contains(required), explanation);
    }
    expect(
        &mut errors,
        html.matches("class=\"route-card\"").count() == 3,
        "Expected exactly three participation routes",
    );
    for (required, explanation) in [
        (".route-grid{", "Route layout missing"),
        (".route-card{", "Route styling missing"),
        (
            "@media(max-width:980px){.route-grid{grid-template-columns:1fr}}",
            "Responsive single-column route fallback missing",
        ),
        (":focus-visible", "Visible keyboard focus styling missing"),
        (".skip:focus", "Skip-link focus styling missing"),
        ("@media(max-width:650px)", "Small-screen breakpoint missing"),
        ("@media(max-width:980px)", "Tablet breakpoint missing"),
        ("@media(prefers-reduced-motion:reduce)", "Reduced-motion override missing"),
    ] {
        expect(&mut errors, css.contains(required), explanation);
    }
    for (required, explanation) in [
        ("docs/index.html", "Local preview path missing"),
        ("rustc --edition=2021", "Documented Rust build command missing"),
        ("tests/check_free_energy_site.rs", "Documented Rust checker missing"),
        ("does **not** publish a website", "Deployment boundary missing"),
        ("predates the corrected online fork instructions", "Packaged guide warning missing"),
    ] {
        expect(&mut errors, readme.contains(required), explanation);
    }
    errors
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let root = match args.as_slice() {
        [] => env::current_dir().expect("Cannot determine current directory"),
        [flag, path] if flag == "--root" => Path::new(path).to_path_buf(),
        _ => {
            eprintln!("Usage: check_free_energy_site [--root REPO_ROOT]");
            std::process::exit(2);
        }
    };
    let errors = validate(&root);
    if errors.is_empty() {
        println!("PASS: FREE ENERGY landing structural checks (offline Rust; no browser/HTTP/deployment QA)");
    } else {
        for error in errors {
            eprintln!("FAIL: {error}");
        }
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_attributes_without_external_parser_dependencies() {
        let doc = r##"<main id="main"><a href="#main">Skip</a></main>"##;
        let elements = tags(doc);
        assert_eq!(attribute(elements[0], "id"), Some("main"));
        assert_eq!(attribute(elements[1], "href"), Some("#main"));
    }

    #[test]
    fn detects_a_missing_local_fixture() {
        let missing = env::temp_dir().join("free-energy-nonexistent-site-fixture");
        assert!(!validate(&missing).is_empty());
    }

    #[test]
    fn rejects_mixed_case_and_whitespace_obfuscated_active_elements() {
        let doc = r#"<ScRiPt
src="x"></ScRiPt><IFRAME	src="x"></IFRAME><FORM
method="post"></FORM><OBjectdata="x"></OBject><EMBED/>"#;
        let elements = tags(doc);
        for forbidden in ["script", "iframe", "form", "object", "embed"] {
            assert!(
                elements.iter().any(|tag| is_open_element(tag, forbidden)),
                "Failed to identify active element: {forbidden}"
            );
        }
        assert!(!is_open_element("scripture src=\"x\"", "script"));
        assert!(!is_open_element("/script", "script"));
        assert!(is_open_element("a\nhref=\"#main\"", "a"));
        assert!(is_open_element("MAIN\tid=\"main\"", "main"));
    }

    #[test]
    fn empty_attribute_is_not_a_working_link() {
        assert_eq!(attribute(r#"a href="""#, "href"), Some(""));
    }
}
