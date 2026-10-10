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
            if key.eq_ignore_ascii_case(name) {
                return Some(""); // A present boolean attribute has an empty value.
            }
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
/// Reject inert template and scripting-dependent noscript containers: their
/// descendants appear in this lexical source scanner but are not reliable
/// rendered page content. They cannot satisfy Contact/navigation/card checks.
fn disallowed_site_elements(elements: &[&str]) -> Vec<&'static str> {
    [
        "script", "iframe", "form", "object", "embed", "template", "noscript",
        "textarea", "xmp", "plaintext", "noembed", "noframes",
    ]
        .into_iter()
        .filter(|name| elements.iter().any(|tag| is_open_element(tag, name)))
        .collect()
}

/// Disallow markup that can fetch resources in addition to ordinary outbound
/// hyperlinks. The one accepted local stylesheet is separately checked for
/// exactly one instance. This is a fail-closed source guard, not an HTML parser
/// or proof of browser/network isolation.
fn has_unapproved_resource_markup(elements: &[&str]) -> bool {
    elements.iter().any(|tag| {
        let approved_css = is_open_element(tag, "link")
            && attribute(tag, "rel")
                .map(|rel| rel.eq_ignore_ascii_case("stylesheet"))
                .unwrap_or(false)
            && attribute(tag, "href") == Some("./styles.css");
        (is_open_element(tag, "link") && !approved_css)
            || [
                "img", "picture", "source", "audio", "video", "track", "svg",
                "style", "frame", "frameset",
            ]
            .iter()
            .any(|name| is_open_element(tag, name))
            // Links can send extra network requests through ping or the
            // Attribution Reporting API without any script or image tag.
            || [
                "src", "srcset", "poster", "background", "style",
                "xlink:href", "ping", "attributionsrc",
            ]
                .iter()
                .any(|name| attribute(tag, name).is_some())
    })
}

/// Reject executable inline HTML event handlers on any element. An inert
/// attribute value containing "onclick=" is not itself a handler. Scan the
/// actual attribute names while honoring quoted values and comments (which
/// the source tag tokenizer already strips). This intentionally fails closed
/// for unknown or future "on..." event names, not just onclick/onload.
fn has_inline_event_handler(elements: &[&str]) -> bool {
    elements.iter().any(|tag| {
        let bytes = tag.as_bytes();
        let mut i = 0;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1; // element name
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
                if i < bytes.len() {
                    i += 1;
                }
                continue;
            }
            let name = &bytes[start..i];
            if name.len() > 2
                && name[0].to_ascii_lowercase() == b'o'
                && name[1].to_ascii_lowercase() == b'n'
                && name[2..].iter().all(|b| b.is_ascii_alphabetic())
            {
                return true;
            }
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if bytes.get(i) != Some(&b'=') {
                continue;
            }
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if let Some(&quote) = bytes.get(i) {
                if quote == b'"' || quote == b'\'' {
                    i += 1;
                    while i < bytes.len() && bytes[i] != quote {
                        i += 1;
                    }
                    if i < bytes.len() {
                        i += 1;
                    }
                } else {
                    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                }
            }
        }
        false
    })
}

/// Catch ordinary CSS network-resource syntax and reject CSS escapes rather
/// than attempt to interpret escaped fetch-token names. Not a CSS engine:
/// browser QA is still required before publication.
fn has_css_resource_syntax(css: &str) -> bool {
    let compact = css
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    ["@import", "url(", "image-set(", "@font-face"]
        .iter()
        .any(|token| compact.contains(token))
        || compact.contains('\\')
}

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

/// The skip target must belong to the single real main landmark. A global
/// id=\"main\" on an unrelated element must not satisfy this check.
fn has_unique_main_landmark(elements: &[&str]) -> bool {
    let mut mains = elements.iter().filter(|tag| is_open_element(tag, "main"));
    match (mains.next(), mains.next()) {
        (Some(main), None) => {
            attribute(main, "id") == Some("main")
                && elements
                    .iter()
                    .filter(|tag| attribute(tag, "id") == Some("main"))
                    .count() == 1
        },
        _ => false,
    }
}

/// A base URL changes where relative CSS and fragment links resolve; a
/// meta refresh can redirect the visitor without JavaScript. Neither belongs
/// in this no-navigation-side-effects static landing.
fn has_browser_navigation_override(elements: &[&str]) -> bool {
    elements.iter().any(|tag| {
        is_open_element(tag, "base")
            || (is_open_element(tag, "meta")
                && attribute(tag, "http-equiv")
                    .map(|directive| directive.trim().eq_ignore_ascii_case("refresh"))
                    .unwrap_or(false))
    })
}

/// Ignore required copy that exists only in source comments. This is a
/// conservative source check, not a complete HTML text-content parser.
/// An unclosed comment consumes the rest of the document.
fn without_html_comments(html: &str) -> String {
    let mut uncommented = String::with_capacity(html.len());
    let mut remaining = html;
    loop {
        let Some(start) = remaining.find("<!--") else {
            uncommented.push_str(remaining);
            return uncommented;
        };
        uncommented.push_str(&remaining[..start]);
        let Some(end) = remaining[start + 4..].find("-->") else {
            return uncommented;
        };
        remaining = &remaining[start + 4 + end + 3..];
    }
}

/// Extract text-node source content, excluding tags, quoted attributes,
/// and HTML comments. This does not prove browser visibility or accessibility:
/// CSS-hidden text and inert template content need browser-level checks.
fn source_text_outside_markup(html: &str) -> String {
    let clean = without_html_comments(html);
    let bytes = clean.as_bytes();
    let mut text = String::with_capacity(clean.len());
    let mut pos = 0;

    while let Some(offset) = clean[pos..].find('<') {
        let start = pos + offset;
        text.push_str(&clean[pos..start]);
        let mut cursor = start + 1;
        let mut quote: Option<u8> = None;
        while cursor < bytes.len() {
            let current = bytes[cursor];
            match quote {
                Some(delimiter) if current == delimiter => quote = None,
                None if current == b'"' || current == 0x27 => quote = Some(current),
                None if current == b'>' => break,
                _ => {}
            }
            cursor += 1;
        }
        if cursor == bytes.len() {
            return text; // Unterminated markup cannot supply user-facing copy.
        }
        text.push(' '); // Prevent words on different sides of a tag from joining.
        pos = cursor + 1;
    }
    text.push_str(&clean[pos..]);
    text
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
    let uncommented_html = without_html_comments(html);
    let source_copy = source_text_outside_markup(html);
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

    expect(&mut errors, uncommented_html.contains("<html lang=\"en\">"), "Missing English HTML language");
    expect(
        &mut errors,
        uncommented_html.contains("<meta name=\"viewport\""),
        "Missing viewport meta tag",
    );
    expect(
        &mut errors,
        has_unique_main_landmark(&elements),
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
    expect(
        &mut errors,
        !has_browser_navigation_override(&elements),
        "Unexpected base URL override or automatic meta refresh",
    );
    for forbidden in disallowed_site_elements(&elements) {
        errors.push(format!("Unexpected active, embedded, or inert element: {forbidden}"));
    }
    expect(
        &mut errors,
        !has_unapproved_resource_markup(&elements),
        "Unexpected browser resource-loading markup",
    );
    expect(
        &mut errors,
        !has_inline_event_handler(&elements),
        "Inline JavaScript event-handler attribute is forbidden",
    );
    expect(
        &mut errors,
        !has_css_resource_syntax(css),
        "Unexpected CSS resource import, image URL, or escape syntax",
    );
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
        expect(&mut errors, source_copy.contains(required), explanation);
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
    fn title_placement_rejects_hidden_real_page_and_malformed_head() {
        // All cases mutate the actual static landing, not a synthetic substitute.
        let root = env::temp_dir().join(format!(
            "free-energy-title-placement-{}", std::process::id()
        ));
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("title fixture docs");
        let original = include_str!("../docs/index.html");
        fs::write(docs.join("styles.css"), include_str!("../docs/styles.css"))
            .expect("title fixture css");
        fs::write(docs.join("README.md"), include_str!("../docs/README.md"))
            .expect("title fixture readme");
        fs::write(docs.join("index.html"), original).expect("title fixture original");
        assert!(validate(&root).is_empty(), "unmodified landing must pass");

        let mut negatives: Vec<(&str, String)> = Vec::new();
        for (name, opening, closing) in [
            ("ordinary", "<title>", "</title>"),
            ("uppercase", "<TITLE>", "</TITLE>"),
            ("mixed", "<TiTlE>", "</TiTlE>"),
            ("unclosed", "<title>", ""),
            ("solidus", "<title/>", "</title>"),
            ("spaced-solidus", "<TiTlE />", ""),
            ("attribute-solidus", "<title data-probe='x' />", "</title>"),
        ] {
            // Move the entire original body inside title RCDATA, retaining
            // all real source navigation/cards to challenge lexical false PASS.
            let html = original
                .replacen("<body>", &format!("<body>{opening}"), 1)
                .replacen("</body>", &format!("{closing}</body>"), 1);
            negatives.push((name, html));
        }
        negatives.push((
            "missing-head-title",
            original.replace("<title>FREE ENERGY — Remasters Everything</title>", ""),
        ));
        negatives.push((
            "duplicate-head-title",
            original.replacen(
                "<title>FREE ENERGY",
                "<title>Duplicate</title><title>FREE ENERGY",
                1,
            ),
        ));
        negatives.push((
            "unclosed-head-title",
            original.replacen("</title>", "", 1),
        ));
        negatives.push((
            "selfclosing-head-title",
            original.replacen("<title>FREE ENERGY", "<title/>FREE ENERGY", 1),
        ));
        negatives.push((
            "before-head",
            original.replacen("<head>", "<title>Outside</title><head>", 1),
        ));
        negatives.push((
            "between-head-and-body",
            original.replacen("</head>", "</head><title>Outside</title>", 1),
        ));
        negatives.push((
            "after-body",
            original.replacen("</body>", "</body><title>Outside</title>", 1),
        ));
        negatives.push((
            "unmatched-closing-title",
            original.replacen("<body>", "<body></title>", 1),
        ));
        negatives.push((
            "false-head-exit-inside-rcdata",
            original.replacen(
                "Remasters Everything</title>",
                "Remasters </head> Everything</title>",
                1,
            ),
        ));

        for (name, html) in negatives {
            fs::write(docs.join("index.html"), html).expect("write title negative");
            let errors = validate(&root);
            assert!(
                errors.iter().any(|error| error.contains(
                    "Unexpected title placement outside head or malformed head title"
                )),
                "title negative {name} must have placement diagnostic, got {errors:?}"
            );
        }

        for (name, injection) in [
            ("comment", "<!-- <title>fake</title> -->"),
            ("quoted-attribute", "<div data-fake='<title>fake</title>'></div>"),
            ("different-tag", "<title-other>ordinary text</title-other>"),
        ] {
            let html = original.replacen("<body>", &format!("<body>{injection}"), 1);
            fs::write(docs.join("index.html"), html).expect("write title positive");
            assert!(
                validate(&root).is_empty(),
                "title-like positive control {name} must remain valid"
            );
        }
        let _ = fs::remove_dir_all(root);
    }


    #[test]
    fn required_copy_in_attributes_is_not_treated_as_page_copy() {
        let spoof = r#"<main>
<meta content="Do not post secrets">
<section title='Posting does not enroll a contributor' aria-label="A verified playable catalog is still planned">No disclosure here</section>
</main>"#;
        let source = source_text_outside_markup(spoof);
        for required in [
            "Do not post secrets",
            "Posting does not enroll a contributor",
            "A verified playable catalog is still planned",
        ] {
            assert!(!source.contains(required), "Attribute spoof passed: {required}");
        }
    }

    #[test]
    fn required_copy_in_real_text_nodes_is_found() {
        let live = r#"<main><p>Do not post secrets</p>
<p>Posting does not enroll a contributor</p>
<p>A verified playable catalog is still planned</p></main>"#;
        let source = source_text_outside_markup(live);
        assert!(source.contains("Do not post secrets"));
        assert!(source.contains("Posting does not enroll a contributor"));
        assert!(source.contains("A verified playable catalog is still planned"));
    }

    #[test]
    fn comments_and_quoted_angle_brackets_cannot_supply_page_copy() {
        let spoof = r#"<!-- Do not post secrets -->
<aside title='<p>Posting does not enroll a contributor</p>'>Ordinary text</aside>"#;
        let source = source_text_outside_markup(spoof);
        assert!(!source.contains("Do not post secrets"));
        assert!(!source.contains("Posting does not enroll a contributor"));
        assert!(source.contains("Ordinary text"));
    }

    #[test]
    fn comments_cannot_forge_required_public_safety_copy() {
        let html = r#"<main><!-- Do not post secrets -->
<p>Posting does not enroll a contributor.</p></main>"#;
        let live = without_html_comments(html);
        assert!(!live.contains("Do not post secrets"));
        assert!(live.contains("Posting does not enroll a contributor."));
    }

    #[test]
    fn unclosed_html_comment_cannot_forge_required_copy() {
        let live = without_html_comments(
            "<main><p>Public questions.</p><!-- Do not post secrets",
        );
        assert_eq!(live, "<main><p>Public questions.</p>");
        assert!(!live.contains("Do not post secrets"));
    }

    #[test]
    fn required_copy_survives_adjacent_complete_comments() {
        let live = without_html_comments(
            "<!-- Do not post secrets --> <p>Do not post secrets</p> <!-- hidden -->",
        );
        assert_eq!(live, " <p>Do not post secrets</p> ");
    }

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
    fn main_landmark_must_own_skip_target_id() {
        let valid = tags(r##"<a href="#main">Skip</a><main id="main">Content</main>"##);
        assert!(has_unique_main_landmark(&valid));

        // Previously this passed: the ID existed, but only on another element.
        let unrelated = tags(r##"<main>Content</main><div id="main"></div>"##);
        assert!(!has_unique_main_landmark(&unrelated));

        let spoofed = tags(r##"<main data-id="main"></main><section id="main"></section>"##);
        assert!(!has_unique_main_landmark(&spoofed));
    }

    #[test]
    fn main_landmark_rejects_missing_or_duplicate_main_elements() {
        assert!(!has_unique_main_landmark(&tags(r##"<div id="main"></div>"##)));
        assert!(!has_unique_main_landmark(&tags(r##"<main id="main"></main><main></main>"##)));
    }

    #[test]
    fn duplicate_main_id_on_non_main_element_must_fail_skip_target_check() {
        // HTML IDs must be unique: a preceding duplicate intercepts #main,
        // while a later duplicate leaves fragment navigation ambiguous.
        let preceding = tags(r##"<div id="main"></div><main id="main"></main>"##);
        assert!(!has_unique_main_landmark(&preceding));

        let following = tags(r##"<main id="main"></main><section id="main"></section>"##);
        assert!(!has_unique_main_landmark(&following));

        let clean = tags(r##"<div id="other"></div><main id="main"></main>"##);
        assert!(has_unique_main_landmark(&clean));
    }

    #[test]
    fn main_landmark_accepts_html_case_and_attribute_spacing() {
        let elements = tags("<MAIN\nID = 'main'></MAIN>");
        assert!(has_unique_main_landmark(&elements));
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
    fn inert_template_cannot_forge_visible_contact_and_participation() {
        let html = format!(
            r#"<template><nav aria-label="Main navigation"><a href="{CONTACT}">Contact</a></nav><article class="route-card"></article></template>"#
        );
        // The lightweight scanner sees template descendants: they must not be
        // trusted as visible site content without rejecting the container.
        assert!(primary_contact_link(&html));
        assert_eq!(route_card_count(&html), 1);
        assert_eq!(disallowed_site_elements(&tags(&html)), ["template"]);

        let ordinary = format!(
            r#"<nav aria-label="Main navigation"><a href="{CONTACT}">Contact</a></nav><article class="route-card"></article>"#
        );
        assert!(primary_contact_link(&ordinary));
        assert_eq!(route_card_count(&ordinary), 1);
        assert!(disallowed_site_elements(&tags(&ordinary)).is_empty());

        let mixed_case = tags("<TeMpLaTe data-purpose='hidden'>Text</TeMpLaTe>");
        assert_eq!(disallowed_site_elements(&mixed_case), ["template"]);
    }


    #[test]
    fn noscript_cannot_forge_visible_site_cards_or_navigation() {
        // Use the actual shipping landing and support files, not an isolated
        // synthetic fragment that could fail unrelated structural assertions.
        let root = env::temp_dir().join(format!(
            "free-energy-noscript-site-{}",
            std::process::id()
        ));
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create noscript fixture directory");
        let original = include_str!("../docs/index.html");
        fs::write(docs.join("index.html"), original).expect("write index fixture");
        fs::write(docs.join("styles.css"), include_str!("../docs/styles.css"))
            .expect("write stylesheet fixture");
        fs::write(docs.join("README.md"), include_str!("../docs/README.md"))
            .expect("write readme fixture");
        assert!(
            validate(&root).is_empty(),
            "unmodified landing fixture must pass"
        );

        let route_open = r#"<div class="route-grid" aria-label="Choose how to participate">"#;
        assert_eq!(original.matches(route_open).count(), 1);
        let start = original.find(route_open).expect("route grid") + route_open.len();
        let end = start + original[start..].find("</div>").expect("route grid closing tag");
        let wrap_routes = |open: &str, close: &str| {
            format!(
                "{}{open}{}{close}{}",
                &original[..start],
                &original[start..end],
                &original[end..]
            )
        };
        let nav_open = r#"<nav aria-label="Main navigation">"#;
        let nav_hidden = original
            .replacen(nav_open, &format!("<noscript>{nav_open}"), 1)
            .replacen("</nav>", "</nav></noscript>", 1);

        for (case, mutated) in [
            ("three cards hidden", wrap_routes("<noscript>", "</noscript>")),
            ("navigation hidden", nav_hidden),
            ("mixed-case hidden cards", wrap_routes("<NoScRiPt>", "</NoScRiPt>")),
            (
                "nested hidden cards",
                wrap_routes("<noscript><noscript>", "</noscript></noscript>"),
            ),
            (
                "self-closing noscript",
                original.replacen("</head>", "<NoScRiPt/></head>", 1),
            ),
        ] {
            assert_ne!(mutated, original, "{case}: fixture must mutate source");
            fs::write(docs.join("index.html"), mutated).expect("write mutated index fixture");
            let errors = validate(&root);
            assert!(
                errors.iter().any(|error| {
                    error == "Unexpected active, embedded, or inert element: noscript"
                }),
                "{case}: expected noscript denial, got {errors:?}"
            );
        }

        for (case, mutated) in [
            (
                "HTML comment literal",
                original.replacen("</head>", "<!-- <noscript> --></head>", 1),
            ),
            (
                "multiline HTML comment literal",
                original.replacen("</head>", "<!--\n<noscript>\n--></head>", 1),
            ),
            (
                "quoted attribute literal",
                original.replacen("<body", "<body data-example='<noscript>'", 1),
            ),
        ] {
            fs::write(docs.join("index.html"), mutated).expect("write allowed index fixture");
            let errors = validate(&root);
            assert!(errors.is_empty(), "{case}: should remain valid: {errors:?}");
        }
        fs::remove_dir_all(&root).expect("remove noscript test fixture");
    }

    #[test]
    fn raw_text_containers_cannot_forge_visible_site_routes() {
        // Full shipping-page fixtures, not synthetic isolated fragments.
        let root = env::temp_dir().join(format!(
            "free-energy-raw-text-site-{}",
            std::process::id()
        ));
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create raw-text fixture directory");
        let original = include_str!("../docs/index.html");
        fs::write(docs.join("index.html"), original).expect("write original landing");
        fs::write(docs.join("styles.css"), include_str!("../docs/styles.css"))
            .expect("write fixture stylesheet");
        fs::write(docs.join("README.md"), include_str!("../docs/README.md"))
            .expect("write fixture readme");
        assert!(
            validate(&root).is_empty(),
            "unmodified landing must remain valid"
        );
        assert!(original.contains("<title>FREE ENERGY"));

        let route_open = r#"<div class="route-grid" aria-label="Choose how to participate">"#;
        assert_eq!(original.matches(route_open).count(), 1);
        let start = original.find(route_open).expect("unique route grid") + route_open.len();
        let end = start + original[start..].find("</div>").expect("route grid closing");
        assert_eq!(original[start..end].matches(r#"class="route-card""#).count(), 3);
        let wrap_routes = |open: &str, close: &str| {
            format!(
                "{}{open}{}{close}{}",
                &original[..start],
                &original[start..end],
                &original[end..]
            )
        };
        let nav_open = r#"<nav aria-label="Main navigation">"#;
        assert_eq!(original.matches(nav_open).count(), 1);

        for element in ["textarea", "xmp", "plaintext", "noembed", "noframes"] {
            let expected = format!(
                "Unexpected active, embedded, or inert element: {element}"
            );

            for tag in [element.to_string(), element.to_ascii_uppercase()] {
                let navigation_hidden = original
                    .replacen(nav_open, &format!("<{tag}>{nav_open}"), 1)
                    .replacen("</nav>", &format!("</nav></{tag}>"), 1);
                for (case, candidate) in [
                    ("three cards", wrap_routes(&format!("<{tag}>"), &format!("</{tag}>"))),
                    ("primary navigation", navigation_hidden),
                    ("unclosed cards", wrap_routes(&format!("<{tag}>"), "")),
                    ("self-closing-like opener", original.replacen(
                        "</head>",
                        &format!("<{tag}/></head>"),
                        1,
                    )),
                ] {
                    assert_ne!(candidate, original, "{element}/{case}: fixture unchanged");
                    assert_eq!(
                        candidate.matches(&format!("<{tag}")).count(),
                        1,
                        "{element}/{case}: malformed fixture"
                    );
                    fs::write(docs.join("index.html"), candidate)
                        .expect("write forbidden raw-text fixture");
                    let errors = validate(&root);
                    assert!(
                        errors.iter().any(|error| error == &expected),
                        "{element}/{case}: expected {expected:?}, got {errors:?}"
                    );
                }
            }

            for (case, candidate) in [
                ("comment-only", original.replacen(
                    "</head>",
                    &format!("<!-- <{element}> --></head>"),
                    1,
                )),
                ("attribute-only", original.replacen(
                    "<body",
                    &format!("<body data-example='<{element}>'"),
                    1,
                )),
                ("extended tag name", original.replacen(
                    "</head>",
                    &format!("<{element}-extra></{element}-extra></head>"),
                    1,
                )),
            ] {
                assert_ne!(candidate, original, "{element}/{case}: fixture unchanged");
                fs::write(docs.join("index.html"), candidate)
                    .expect("write permitted lookalike fixture");
                let errors = validate(&root);
                assert!(
                    errors.is_empty(),
                    "{element}/{case}: false positive for harmless lookalike: {errors:?}"
                );
            }
        }

        fs::write(docs.join("index.html"), original).expect("restore original fixture");
        assert!(validate(&root).is_empty(), "real head title and landing must pass");
        fs::remove_dir_all(&root).expect("remove raw-text fixtures");
    }

    #[test]
    fn base_url_and_meta_refresh_cannot_override_visitor_navigation() {
        let base = tags(r##"<head><BASE href="https://example.invalid/"></head>"##);
        assert!(has_browser_navigation_override(&base));

        let redirect = tags(
            r##"<meta HTTP-EQUIV = 'ReFrEsH' content="0; url=https://example.invalid/">"##,
        );
        assert!(has_browser_navigation_override(&redirect));

        let timed_reload = tags(r##"<meta http-equiv=" refresh " content="30">"##);
        assert!(has_browser_navigation_override(&timed_reload));
    }

    #[test]
    fn ordinary_meta_tags_and_spoofed_navigation_markup_are_allowed() {
        let normal = tags(
            r##"<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="theme-color" content="#0b1015">"##,
        );
        assert!(!has_browser_navigation_override(&normal));

        let decoys = tags(
            r##"<!-- <base href="https://example.invalid/">
<meta http-equiv="refresh" content="0"> -->
<a title='<base href="https://example.invalid/">' href="#main">Skip</a>
<meta data-http-equiv="refresh" content="0">"##,
        );
        assert!(!has_browser_navigation_override(&decoys));
    }

    #[test]
    fn inline_event_handlers_are_forbidden_even_without_script_tags() {
        for bad in [
            r#"<a href="https://example.org/" onclick="alert(1)">Link</a>"#,
            r#"<BODY ONLOAD='alert(1)'>Safe-looking text</BODY>"#,
            r#"<button ONPOINTERDOWN="steal()">Go</button>"#,
            r#"<div onmouseover>Text</div>"#,
            r#"<a href='https://example.org/' title="quoted > is text" onfocusin = 'go()'>Link</a>"#,
            r#"<a href="https://example.org/" oNcLiCk>Boolean-style handler</a>"#,
        ] {
            assert!(
                has_inline_event_handler(&tags(bad)),
                "Executable inline handler escaped source guard: {bad}"
            );
        }
    }

    #[test]
    fn event_handler_scanner_ignores_attribute_values_comments_and_data_names() {
        let allowed = r#"
<!-- <body onload="evil()"> -->
<a title='literal onclick="evil()" > not markup' data-onclick="inert" href="https://example.org/">Link</a>
<p aria-label='onload=alert(1)' data-onload="inert">Real text</p>
"#;
        assert!(!has_inline_event_handler(&tags(allowed)));
        assert!(!has_inline_event_handler(&tags(
            r#"<main id="main"><p>Static landing</p></main>"#
        )));
        // Non-ASCII attribute names are legal source bytes; scanning must
        // never panic by slicing through a UTF-8 code point.
        assert!(!has_inline_event_handler(&tags(
            r#"<p μeta="ordinary" title="onclick is just text">Text</p>"#
        )));
    }

    #[test]
    fn browser_resource_markup_is_rejected_but_normal_links_remain_usable() {
        for bad in [
            r#"<img src="https://example.invalid/tracker.png">"#,
            r#"<LINK rel="preload" as="image" href="https://example.invalid/pixel">"#,
            r#"<video poster="https://example.invalid/poster.png"></video>"#,
            r#"<main style="background: url(https://example.invalid/bg.png)"></main>"#,
            r#"<svg><image href="https://example.invalid/pixel"></image></svg>"#,
            r#"<audio><source src="https://example.invalid/sound"></audio>"#,
        ] {
            assert!(
                has_unapproved_resource_markup(&tags(bad)),
                "Resource markup was not rejected: {bad}"
            );
        }
        let allowed = r#"<link rel="stylesheet" href="./styles.css">
<a href="https://github.com/Distributed-Minds/Fleet-Control-Public">Project</a>"#;
        assert!(!has_unapproved_resource_markup(&tags(allowed)));
        let quoted_and_commented = r#"<!-- <img src="https://example.invalid/"> -->
<p title='<img src="https://example.invalid/">' >Ordinary text</p>"#;
        assert!(!has_unapproved_resource_markup(&tags(quoted_and_commented)));
    }

    #[test]
    fn outbound_anchor_beacon_attributes_are_not_passive_links() {
        for active_link in [
            r#"<a href="https://example.org/" ping="https://example.invalid/track">Visit</a>"#,
            r#"<a href="https://example.org/" PING="">Visit</a>"#,
            r#"<A HREF="https://example.org/" ATTRIBUTIONSRC="https://example.invalid/report">Visit</A>"#,
            r#"<a attributionsrc href="https://example.org/">Visit</a>"#,
            r#"<a rel="noopener" AtTrIbUtIoNsRc href="https://example.org/">Visit</a>"#,
            r#"<a href="https://example.org/" AtTrIbUtIoNsRc>Visit</a>"#,
        ] {
            assert!(
                has_unapproved_resource_markup(&tags(active_link)),
                "Link-level network beacon escaped the guard: {active_link}"
            );
        }
        assert!(!has_unapproved_resource_markup(&tags(
            r#"<a href="https://example.org/" rel="noopener noreferrer">Visit</a>"#
        )));
    }

    #[test]
    fn css_resource_syntax_rejects_imports_urls_images_fonts_and_escapes() {
        for bad in [
            r#"@IMPORT url("https://example.invalid/ext.css");"#,
            r#"background: URL (https://example.invalid/pixel.png);"#,
            r#"background: image-set ("https://example.invalid/pixel.png" 1x);"#,
            r#"@FONT-FACE {font-family: x; src: local(x);}"#,
            r"@\69mport 'https://example.invalid/hidden.css';",
        ] {
            assert!(has_css_resource_syntax(bad), "Missed CSS resource syntax: {bad}");
        }
        assert!(!has_css_resource_syntax(
            "body{font-family:system-ui,sans-serif;background:#0b1015}"
        ));
    }

    #[test]
    fn empty_attribute_is_not_a_working_link() {
        assert_eq!(attribute(r#"a href="""#, "href"), Some(""));
    }

    #[test]
    fn attribute_names_cannot_be_spoofed_by_prefixes_or_quoted_text() {
        assert_eq!(attribute(r##"a data-href="#main""##, "href"), None);
        assert_eq!(attribute(r##"main data-id="main""##, "id"), None);
        assert_eq!(attribute(r##"a title='fake href="#main"' data-href="#main""##, "href"), None);
        assert_eq!(attribute(r##"a aria-label="fake id=main" data-id="main""##, "id"), None);
        assert_eq!(
            attribute(r##"a data-href="#wrong" HREF = "#main""##, "href"),
            Some("#main")
        );
        assert_eq!(
            attribute(r##"a title='quoted attribute text' href="#main""##, "href"),
            Some("#main")
        );
    }
}
