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


/// A complete global HTML attribute name is present even when '=' has no
/// value. Do not use attribute(): its value parser deliberately gives up on
/// trailing '=' and cannot enforce a presence-only prohibition.
fn has_attribute_name(tag: &str, name: &str) -> bool {
    let bytes = tag.as_bytes();
    let mut i = 0;
    let start = tag.trim_start();
    if start.starts_with('/') || start.starts_with('!') || start.starts_with('?') {
        return false;
    }
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
        i += 1; // Opening element name.
    }
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        let begin = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        if begin == i {
            if i < bytes.len() {
                i += 1; // Malformed separator, not a valid name.
            }
            continue;
        }
        if tag[begin..i].eq_ignore_ascii_case(name) {
            return true; // Check the name before attempting to parse its value.
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
                if i == bytes.len() {
                    break; // Unterminated value never creates a later name.
                }
                i += 1;
            } else {
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
        }
    }
    false
}

/// This intentionally accepts only the known single-page head/title/body
/// structure, not arbitrary HTML. In particular, the source tag scanner sees
/// markup-looking tokens in title RCDATA that browsers do not render.
fn has_valid_head_title(elements: &[&str]) -> bool {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Zone {
        BeforeHead,
        Head,
        BetweenHeadAndBody,
        Body,
        AfterBody,
    }
    let mut zone = Zone::BeforeHead;
    let mut seen_head = false;
    let mut seen_body = false;
    let mut seen_title = false;
    let mut in_title = false;

    for tag in elements {
        let tag = tag.trim();
        if in_title {
            if tag.eq_ignore_ascii_case("/title") {
                in_title = false;
            } else {
                // RCDATA does not expose parsed source tags. Rather than let a
                // forged head/body exit or navigation count, reject ambiguity.
                return false;
            }
            continue;
        }
        if tag.eq_ignore_ascii_case("/title") {
            return false; // Orphan closing title.
        }
        if is_open_element(tag, "title") {
            if zone != Zone::Head || seen_title || tag.ends_with('/') {
                return false; // Outside head, duplicate, or self-closing-looking.
            }
            seen_title = true;
            in_title = true;
            continue;
        }
        if is_open_element(tag, "head") {
            if zone != Zone::BeforeHead || seen_head || tag.ends_with('/') {
                return false;
            }
            seen_head = true;
            zone = Zone::Head;
        } else if tag.eq_ignore_ascii_case("/head") {
            if zone != Zone::Head || !seen_title {
                return false;
            }
            zone = Zone::BetweenHeadAndBody;
        } else if is_open_element(tag, "body") {
            if zone != Zone::BetweenHeadAndBody || seen_body || tag.ends_with('/') {
                return false;
            }
            seen_body = true;
            zone = Zone::Body;
        } else if tag.eq_ignore_ascii_case("/body") {
            if zone != Zone::Body {
                return false;
            }
            zone = Zone::AfterBody;
        }
    }

    seen_head && seen_title && seen_body && !in_title && zone == Zone::AfterBody
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

/// Bound required user-facing disclosure checks to source text between exactly
/// one real body open/close pair. An ordinary valid head title is metadata, not
/// body copy. Preserve existing comment/quoted-attribute masking and source
/// word-separation by delegating text extraction to the existing helper.
fn body_source_text_outside_markup(html: &str) -> Option<String> {
    let bytes = html.as_bytes();
    let mut pos = 0;
    let mut body_start = None;
    let mut body_end = None;

    while pos < bytes.len() {
        let Some(offset) = html[pos..].find('<') else {
            break;
        };
        let start = pos + offset;
        if html[start..].starts_with("<!--") {
            let closing = html[start + 4..].find("-->")?;
            pos = start + 4 + closing + 3;
            continue;
        }

        let mut cursor = start + 1;
        let mut quote: Option<u8> = None;
        while cursor < bytes.len() {
            match (quote, bytes[cursor]) {
                (Some(delimiter), current) if current == delimiter => quote = None,
                (None, b'"' | b'\'') => quote = Some(bytes[cursor]),
                (None, b'>') => break,
                _ => {}
            }
            cursor += 1;
        }
        if cursor == bytes.len() {
            return None;
        }

        let tag = &html[start + 1..cursor];
        if is_open_element(tag, "body") {
            if body_start.is_some() || body_end.is_some() {
                return None;
            }
            body_start = Some(cursor + 1);
        } else if tag.trim().eq_ignore_ascii_case("/body") {
            if body_start.is_none() || body_end.is_some() {
                return None;
            }
            body_end = Some(start);
        }
        pos = cursor + 1;
    }

    let start = body_start?;
    let end = body_end?;
    if end < start {
        return None;
    }
    Some(source_text_outside_markup(&html[start..end]))
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
    let source_copy = body_source_text_outside_markup(html);
    let css = &content[1];
    let readme = &content[2];
    let elements = tags(html);
    let mut errors = Vec::new();

    expect(&mut errors, source_copy.is_some(), "Missing or ambiguous source body boundaries");
    expect(
        &mut errors,
        has_valid_head_title(&elements),
        "Unexpected title placement outside head or malformed head title",
    );

    expect(
        &mut errors,
        !elements.iter().any(|tag| has_attribute_name(tag, "hidden")),
        "HTML hidden global attribute is forbidden",
    );

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
        expect(&mut errors, source_copy.as_deref().unwrap_or("").contains(required), explanation);
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
    fn real_page_hidden_attribute_presence_and_decoys() {
        let root = env::temp_dir().join(format!(
            "free-energy-hidden-global-n8c5-{}", std::process::id()
        ));
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create hidden fixtures");
        let original = include_str!("../docs/index.html");
        let needle = "Posting does not enroll a contributor";
        assert_eq!(original.matches(needle).count(), 1, "fixture must be unambiguous");
        fs::write(docs.join("styles.css"), include_str!("../docs/styles.css"))
            .expect("fixture css");
        fs::write(docs.join("README.md"), include_str!("../docs/README.md"))
            .expect("fixture readme");
        fs::write(docs.join("index.html"), original).expect("baseline landing");
        assert!(validate(&root).is_empty(), "baseline landing must pass");

        for (case, attrs) in [
            ("boolean", "hidden"),
            ("missing-value", "hidden="),
            ("space-value", "hidden= "),
            ("tab-value", "hidden=\t"),
            ("linefeed-value", "hidden=\n"),
            ("carriage-return-value", "hidden=\r"),
            ("formfeed-value", "hidden=\x0c"),
            ("mixed-case", "HiDdEn =\t"),
            ("reordered", "class=notice hidden=  "),
            ("quoted-empty", "hidden=\"\""),
            ("single-quoted-empty", "hidden=''"),
            ("until-found", "hidden=\"until-found\""),
            ("invalid-false", "hidden=\"false\""),
            ("unquoted-value", "hidden=until-found"),
            ("trailing-attribute", "class='notice' hidden"),
        ] {
            let marked = format!("<span {attrs}>{needle}</span>");
            let html = original.replacen(needle, &marked, 1);
            fs::write(docs.join("index.html"), html).expect("negative hidden fixture");
            let errors = validate(&root);
            assert!(
                errors.iter().any(|error| error == "HTML hidden global attribute is forbidden"),
                "hidden case {case} must have exact diagnostic, got {errors:?}"
            );
        }
        for (case, attrs) in [
            ("data-name", "data-hidden"),
            ("aria-name", "aria-hidden=\"true\""),
            ("suffix", "hiddenx"),
            ("prefix", "xhidden"),
            ("inert-quoted-name", "title='hidden= > literal'"),
            ("quoted-tag", "title='<span hidden= >'"),
            ("attribute-value", "data-probe=\"hidden\""),
        ] {
            let marked = format!("<span {attrs}>{needle}</span>");
            let html = original.replacen(needle, &marked, 1);
            fs::write(docs.join("index.html"), html).expect("positive hidden decoy");
            assert!(validate(&root).is_empty(), "benign case {case} rejected");
        }
        let with_comment = original.replacen(
            "<body>",
            "<body><!-- <span hidden= >not markup</span> -->",
            1,
        );
        fs::write(docs.join("index.html"), with_comment).expect("comment decoy");
        assert!(validate(&root).is_empty(), "comment-only hidden must not count");
        fs::remove_dir_all(&root).expect("remove hidden fixtures");
    }

    #[test]
    fn hidden_name_helper_respects_tag_and_attribute_boundaries() {
        for tag in [
            "span hidden",
            "span hidden=",
            "span hidden=  ",
            "SPAN class='other hidden=' HiDdEn =\t",
            "span title='<span hidden>' hidden=\"false\"",
        ] {
            assert!(has_attribute_name(tag, "hidden"), "{tag}");
        }
        for tag in [
            "span data-hidden=true",
            "span aria-hidden=true",
            "span hiddenx=1",
            "span title='hidden='",
            "/span hidden",
            "!doctype hidden",
            "span title='<span hidden= >' data-hidden='x'",
        ] {
            assert!(!has_attribute_name(tag, "hidden"), "{tag}");
        }
        assert!(!has_attribute_name("span μeta='hidden='", "hidden"));
    }

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
        // Relocate only the actual primary navigation: source link tokens
        // remain present, but they are title RCDATA rather than visible nav.
        let nav_start = original.find("<nav aria-label=\"Main navigation\">")
            .expect("real primary nav");
        let nav_end = nav_start
            + original[nav_start..].find("</nav>").expect("real nav closer")
            + "</nav>".len();
        let mut only_nav = original.to_string();
        only_nav.insert_str(nav_end, "</title>");
        only_nav.insert_str(nav_start, "<title>");
        negatives.push(("real-nav-in-title", only_nav));

        // Independently hide the exact three route-card articles while
        // retaining all source markup for the legacy token counter.
        let grid_start = original.find("<div class=\"route-grid\"")
            .expect("real route grid");
        let card_start = grid_start
            + original[grid_start..].find("<article class=\"route-card\">")
                .expect("first real route card");
        let mut card_end = card_start;
        for _ in 0..3 {
            card_end += original[card_end..].find("</article>")
                .expect("real card closing tag") + "</article>".len();
        }
        let mut only_cards = original.to_string();
        only_cards.insert_str(card_end, "</title>");
        only_cards.insert_str(card_start, "<title>");
        negatives.push(("real-three-cards-in-title", only_cards));
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
    fn all_required_body_disclosures_reject_valid_title_only_spoofs() {
        let root = env::temp_dir().join(format!(
            "free-energy-body-disclosures-a6r4-{}", std::process::id()
        ));
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("create required-copy fixture directory");
        let original = include_str!("../docs/index.html");
        fs::write(docs.join("styles.css"), include_str!("../docs/styles.css"))
            .expect("fixture stylesheet");
        fs::write(docs.join("README.md"), include_str!("../docs/README.md"))
            .expect("fixture README");
        fs::write(docs.join("index.html"), original).expect("fixture original");
        assert!(validate(&root).is_empty(), "unmodified landing must pass");

        // Real-page negative controls: every body occurrence of one disclosure
        // is removed and one exact copy is placed inside the existing valid
        // HTML head title. Metadata is not rendered participation copy.
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
            let start = original.find("<body>").expect("opening body") + "<body>".len();
            let end = original.find("</body>").expect("closing body");
            assert!(original[start..end].contains(required), "body fixture missing {required}");
            let hidden_body = original[start..end].replace(required, "");
            let mutated = format!(
                "{}{}{}", &original[..start], hidden_body, &original[end..]
            ).replacen("</title>", &format!(" {required}</title>"), 1);
            let body_start = mutated.find("<body>").expect("mutated body");
            let body_end = mutated.find("</body>").expect("mutated body end");
            assert!(!mutated[body_start..body_end].contains(required), "body still has {required}");
            assert!(mutated[..body_start].contains(required), "head must contain {required}");
            fs::write(docs.join("index.html"), mutated).expect("write title-only fixture");
            let errors = validate(&root);
            assert!(
                errors.iter().any(|error| error.as_str() == explanation),
                "title-only disclosure {required} must fail with {explanation}, got {errors:?}"
            );
        }
        fs::remove_dir_all(&root).expect("remove body-copy fixtures");
    }

    #[test]
    fn body_copy_source_boundary_respects_comments_attributes_and_tag_case() {
        for (name, source, wanted) in [
            ("ordinary", "<head><title>fake disclosure</title></head><body><p>real body copy</p></body>", "real body copy"),
            ("comment decoy", "<!-- <body>fake</body> --><body>real text</body>", "real text"),
            ("attribute decoy", "<head><meta content='<body>fake</body>'></head><body>real text</body>", "real text"),
            ("mixed-case", "<BODY data-probe='<body>fake</body>'>real text</BODY>", "real text"),
            ("body comment", "<body>before<!-- hidden phrase -->after</body>", "beforeafter"),
            ("tail after close", "<body>real text</body>fake disclosure", "real text"),
            ("head/body split", "<head>in head</head><body>in body</body>", "in body"),
        ] {
            let copy = body_source_text_outside_markup(source)
                .unwrap_or_else(|| panic!("{name} should have one valid source body"));
            assert!(copy.contains(wanted), "{name}: missing real body text {copy:?}");
            assert!(!copy.contains("fake disclosure"), "{name}: head/tail text leaked");
            assert!(!copy.contains("hidden phrase"), "{name}: comment text leaked");
            assert!(!copy.contains("in head"), "{name}: head text leaked");
        }
        for (name, source) in [
            ("no body", "<head><title>head only</title></head>"),
            ("unclosed body", "<body>no closing tag"),
            ("stray closer", "</body><body>late body</body>"),
            ("duplicate opener", "<body>first<body>second</body></body>"),
            ("duplicate closer", "<body>first</body></body>"),
            ("second body", "<body>first</body><body>second</body>"),
            ("unterminated comment", "<body>good<!-- unclosed"),
            ("unterminated attribute", "<body><p title='unfinished>text</body>"),
        ] {
            assert!(
                body_source_text_outside_markup(source).is_none(),
                "{name}: malformed source boundary must fail closed"
            );
        }
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
