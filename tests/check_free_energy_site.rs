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
const CONTACT: &str = "https://github.com/Distributed-Minds/Fleet-Control-Public/discussions";

/// A public-contact anchor must be a live element inside the real primary
/// navigation, not text in an HTML comment or another attribute's value.
fn primary_contact_link(html: &str) -> bool {
    let mut in_primary_nav = false;
    let mut has_contact = false;
    for tag in tags(html) {
        if !in_primary_nav {
            if is_open_element(tag, "nav")
                && attribute(tag, "aria-label") == Some("Main navigation")
            {
                in_primary_nav = true;
            }
        } else if tag.trim().eq_ignore_ascii_case("/nav") {
            return has_contact;
        } else if is_open_element(tag, "a") && attribute(tag, "href") == Some(CONTACT) {
            has_contact = true;
        }
    }
    false // Unclosed primary navigation is not a valid contact route.
}

/// Collect complete source tags while excluding HTML comments and treating
/// angle brackets inside quoted attribute values as text, not new markup.
/// This is a bounded offline checker, not a browser-equivalent HTML parser.
fn tags(html: &str) -> Vec<&str> {
    let bytes = html.as_bytes();
    let mut found = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let Some(offset) = html[pos..].find('<') else {
            break;
        };
        let start = pos + offset;
        if html[start..].starts_with("<!--") {
            let Some(end) = html[start + 4..].find("-->") else {
                break; // An unclosed HTML comment consumes the remaining document.
            };
            pos = start + 4 + end + 3;
            continue;
        }

        let mut cursor = start + 1;
        let mut quote: Option<u8> = None;
        while cursor < bytes.len() {
            let current = bytes[cursor];
            match quote {
                Some(delimiter) if current == delimiter => quote = None,
                None if current == b'"' || current == b'\'' => quote = Some(current),
                None if current == b'>' => break,
                _ => {}
            }
            cursor += 1;
        }
        if cursor == bytes.len() {
            break; // An unterminated tag cannot supply valid attributes.
        }
        found.push(&html[start + 1..cursor]);
        pos = cursor + 1;
    }
    found
}

/// Match a real opening tag name at a tag boundary. HTML tag names are
/// ASCII case-insensitive and attributes may begin after tabs/newlines.
fn is_open_element(tag: &str, name: &str) -> bool {
    let end = tag
        .find(|c: char| c.is_ascii_whitespace() || c == '/')
        .unwrap_or(tag.len());
    tag[..end].eq_ignore_ascii_case(name)
}

/// Read HTML attributes as complete name/value tokens, not arbitrary substrings.
/// Otherwise data-href or href text inside another quoted attribute can forge
/// a passing link/id check. This remains a restricted source checker, not a
/// general HTML parser.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let bytes = tag.as_bytes();
    let mut i = 0;

    // Skip the element name before inspecting attributes.
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
        i += 1;
    }
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        let start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        if start == i {
            break;
        }
        let key = &tag[start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == bytes.len() || bytes[i] != b'=' {
            continue; // Boolean attribute; the next token may have a value.
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == bytes.len() {
            break;
        }

        let quote = bytes[i];
        let value = if quote == b'"' || quote == b'\'' {
            i += 1;
            let value_start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            if i == bytes.len() {
                break; // Unterminated quote: do not guess subsequent attributes.
            }
            let value = &tag[value_start..i];
            i += 1;
            value
        } else {
            let value_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            &tag[value_start..i]
        };
        if key.eq_ignore_ascii_case(name) {
            return Some(value);
        }
    }
    None
}

/// Count actual participation-card articles rather than matching text that
/// might appear in comments, quoted attributes, or unrelated elements.
fn route_card_count(html: &str) -> usize {
    tags(html)
        .iter()
        .filter(|tag| is_open_element(tag, "article"))
        .filter(|tag| {
            attribute(tag, "class")
                .map(|classes| classes.split_ascii_whitespace().any(|class| class == "route-card"))
                .unwrap_or(false)
        })
        .count()
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
    expect(
        &mut errors,
        primary_contact_link(html),
        "Primary navigation is missing a direct public Contact link",
    );
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
        (CONTACT, "Public Discussions link missing"),
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
        route_card_count(html) == 3,
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
    fn route_card_count_ignores_comments_attribute_spoofing_and_non_articles() {
        let html = r#"
<!-- <article class="route-card"></article> -->
<section title='<article class="route-card">not a real card</article>'></section>
<article data-class="route-card"></article>
<article class="route-card-disabled"></article>
<div class="route-card"></div>
<article class="route-card"></article>
"#;
        assert_eq!(route_card_count(html), 1);
    }

    #[test]
    fn route_card_count_accepts_real_articles_with_multiple_classes() {
        let html = r#"
<ARTICLE class="card route-card featured"></ARTICLE>
<article class="route-card"></article>
<article class="card
    route-card"></article>
"#;
        assert_eq!(route_card_count(html), 3);
    }

    #[test]
    fn commented_markup_cannot_forge_a_required_link_or_active_element() {
        let doc = r##"<!-- <a href="https://example.com/forged">Pretend</a>
<script src="not-executed.js"></script><main id="decoy"></main> -->
<main id="real"><a href="#real">Actual</a></main>"##;
        let elements = tags(doc);
        let links: Vec<&str> = elements
            .iter()
            .filter(|tag| is_open_element(tag, "a"))
            .filter_map(|tag| attribute(tag, "href"))
            .collect();
        assert_eq!(links, ["#real"]);
        assert!(!elements.iter().any(|tag| is_open_element(tag, "script")));
        assert!(!elements.iter().any(|tag| attribute(tag, "id") == Some("decoy")));
    }

    #[test]
    fn quoted_angle_brackets_and_greater_than_do_not_forge_links() {
        let doc = r##"<a title="<a href='https://example.com/forged'> > still text" href="#real">Actual</a>
<main id="real"></main>"##;
        let elements = tags(doc);
        let links: Vec<&str> = elements
            .iter()
            .filter(|tag| is_open_element(tag, "a"))
            .filter_map(|tag| attribute(tag, "href"))
            .collect();
        assert_eq!(links, ["#real"]);
        assert_eq!(elements.iter().filter(|tag| is_open_element(tag, "a")).count(), 1);
    }

    #[test]
    fn unclosed_comments_and_tags_do_not_create_pseudo_links() {
        assert!(tags(r#"<!-- <a href="https://example.com/forged">"#).is_empty());
        assert!(tags(r##"<a href="#incomplete""##).is_empty());
    }

    #[test]
    fn extracts_attributes_without_external_parser_dependencies() {
        let doc = r##"<main id="main"><a href="#main">Skip</a></main>"##;
        let elements = tags(doc);
        assert_eq!(attribute(elements[0], "id"), Some("main"));
        assert_eq!(attribute(elements[1], "href"), Some("#main"));
    }

    #[test]
    fn public_contact_must_be_in_primary_navigation() {
        let link = format!(r#"<a href="{CONTACT}">Contact</a>"#);
        assert!(primary_contact_link(&format!(
            r#"<nav aria-label="Main navigation">{link}</nav>"#
        )));
        assert!(!primary_contact_link(&format!(
            r#"<nav aria-label="Main navigation"></nav><footer>{link}</footer>"#
        )));
    }

    #[test]
    fn commented_or_attribute_embedded_contact_links_do_not_satisfy_navigation() {
        let link = format!(r#"<a href="{CONTACT}">Contact</a>"#);
        let commented_nav = format!(
            r#"<!-- <nav aria-label="Main navigation">{link}</nav> -->"#
        );
        assert!(!primary_contact_link(&commented_nav));
        let commented_link = format!(
            r#"<nav aria-label="Main navigation"><!-- {link} --></nav>"#
        );
        assert!(!primary_contact_link(&commented_link));
        let attribute_link = format!(
            r#"<nav aria-label="Main navigation"><span title='{link}'></span></nav>"#
        );
        assert!(!primary_contact_link(&attribute_link));
        let real_link = format!(
            r#"<nav aria-label="Main navigation">{link}</nav>"#
        );
        assert!(primary_contact_link(&real_link));
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
method="post"></FORM><OBject
data="x"></OBject><EMBED/>"#;
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

    #[test]
    fn attribute_names_cannot_be_spoofed_by_prefixes_or_quoted_text() {
        assert_eq!(attribute(r#"a data-href="#main""#, "href"), None);
        assert_eq!(attribute(r#"main data-id="main""#, "id"), None);
        assert_eq!(attribute(r#"a title='fake href="#main"' data-href="#main""#, "href"), None);
        assert_eq!(attribute(r#"a aria-label="fake id=main" data-id="main""#, "id"), None);
        assert_eq!(
            attribute(r#"a data-href="#wrong" HREF = "#main""#, "href"),
            Some("#main")
        );
        assert_eq!(
            attribute(r#"a title='quoted attribute text' href="#main""#, "href"),
            Some("#main")
        );
    }
}
