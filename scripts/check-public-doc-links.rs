//! Offline checker for relative links in the nine FREE ENERGY onboarding documents.
//!
//! Build: rustc --edition=2021 -D warnings scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links
//! Run:   /tmp/free-energy-doc-links --root /absolute/path/to/checkout
//! Tests: rustc --edition=2021 --test -D warnings scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links-tests
//!        /tmp/free-energy-doc-links-tests
//!
//! Supported: ordinary inline Markdown links and image destinations, fragments,
//! optional quoted titles, and angle-delimited destinations. This is not a full
//! CommonMark or HTTP checker. Unsupported destination syntax is an error.

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

const DOCUMENTS: [&str; 9] = [
    "README.md",
    "CONTRIBUTING.md",
    "GETTING-STARTED.md",
    "HELP-A-PROJECT.md",
    "WORKFLOW-GUIDES.md",
    "TROUBLESHOOTING.md",
    "BRANDING.md",
    "release-kit/README-FIRST.md",
    "release-kit/INSTALL-CHECKLIST.md",
];

#[derive(Debug, Default)]
struct Report {
    local_links: usize,
    errors: Vec<String>,
}

fn fence_marker(line: &str) -> Option<(u8, usize, bool)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let first = *bytes.first()?;
    if first != b'~' && first != b'\x60' {
        return None;
    }
    let size = bytes.iter().take_while(|&&c| c == first).count();
    // A fence opener may have an info string, but a closing fence must have
    // only whitespace after the marker. Return the distinction to the caller.
    // CommonMark does not permit backticks in a backtick-fence info string.
    // Accepting one would hide ordinary links underneath an invalid opener.
    // Tilde-fenced info strings are allowed to contain backticks.
    if first == b'\x60' && bytes[size..].contains(&b'\x60') {
        return None;
    }
    let closing_suffix_is_whitespace = bytes[size..].iter().all(|&b| b == b' ' || b == b'\t');
    (size >= 3).then_some((first, size, closing_suffix_is_whitespace))
}

fn mask_inline_code(line: &str) -> String {
    let original = line.as_bytes();
    let mut masked = original.to_vec();
    let mut i = 0;
    while i < original.len() {
        if original[i] != b'\x60' {
            i += 1;
            continue;
        }
        // A backslash-escaped backtick is literal text, not a code delimiter.
        if preceded_by_escape(original, i) {
            i += 1;
            continue;
        }
        let n = original[i..].iter().take_while(|&&x| x == b'\x60').count();
        let mut j = i + n;
        let mut ending = None;
        while j < original.len() {
            if original[j] == b'\x60' {
                // An escaped backtick cannot terminate an active code span.
                if preceded_by_escape(original, j) {
                    j += 1;
                    continue;
                }
                let m = original[j..].iter().take_while(|&&x| x == b'\x60').count();
                if m == n {
                    ending = Some(j + m);
                    break;
                }
                j += m;
            } else {
                j += 1;
            }
        }
        if let Some(end) = ending {
            // Preserve line endings for stable per-line link diagnostics.
            for byte in &mut masked[i..end] {
                if *byte != b'\n' && *byte != b'\r' {
                    *byte = b' ';
                }
            }
            i = end;
        } else {
            i += n;
        }
    }
    String::from_utf8(masked).expect("replacing ASCII backticks preserves UTF-8")
}


// Mask code spans across contiguous Markdown paragraph lines, without allowing
// an unmatched opener to swallow links beyond a blank line or a fenced block.
// Existing single-line backtick escaping and fence rules remain authoritative.
fn mask_paragraph_code_spans(markdown: &str) -> String {
    let mut visible = String::with_capacity(markdown.len());
    let mut paragraph = String::new();
    let mut fenced: Option<(u8, usize)> = None;
    for raw_line in markdown.split_inclusive('\n') {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let fence_boundary = match fence_marker(line) {
            Some((marker, width, _)) if fenced.is_none() => {
                fenced = Some((marker, width));
                true
            }
            Some((marker, width, can_close)) => match fenced {
                Some((old, old_width))
                    if old == marker && width >= old_width && can_close =>
                {
                    fenced = None;
                    true
                }
                _ => false,
            },
            None => false,
        };
        if fence_boundary || fenced.is_some() || line.trim().is_empty() {
            if !paragraph.is_empty() {
                visible.push_str(&mask_inline_code(&paragraph));
                paragraph.clear();
            }
            visible.push_str(raw_line);
        } else {
            paragraph.push_str(raw_line);
        }
    }
    if !paragraph.is_empty() {
        visible.push_str(&mask_inline_code(&paragraph));
    }
    visible
}

fn decode_once(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err("malformed percent escape".into());
            }
            let hex = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            let hi = hex(bytes[i + 1]).ok_or("malformed percent escape")?;
            let lo = hex(bytes[i + 2]).ok_or("malformed percent escape")?;
            out.push(hi * 16 + lo);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "invalid UTF-8 after percent decoding".into())
}

// Only Markdown-escaped parentheses are supported inside bare paths.
// Other backslashes remain unsafe/ambiguous, including doubled escapes and
// percent-decoded filesystem separators. Decode percent escapes just once.
fn decode_markdown_path(path: &str) -> Result<String, String> {
    let mut unescaped = String::with_capacity(path.len());
    let mut chars = path.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('(') => unescaped.push('('),
                Some(')') => unescaped.push(')'),
                _ => return Err("ambiguous backslash path".into()),
            }
        } else {
            unescaped.push(ch);
        }
    }
    decode_once(&unescaped)
}

/// None means a deliberately ignored fragment or external URI.
fn parse_destination(raw: &str) -> Result<Option<String>, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("empty link destination".into());
    }

    let (destination, tail) = if let Some(remainder) = value.strip_prefix('<') {
        let end = remainder.find('>').ok_or("unclosed angle destination")?;
        (&remainder[..end], remainder[end + 1..].trim())
    } else {
        let split = value
            .char_indices()
            .find(|(_, c)| c.is_whitespace())
            .map(|(i, _)| i)
            .unwrap_or(value.len());
        let (path, rest) = value.split_at(split);
        let has_unescaped_paren = path
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(index, byte)| {
                matches!(byte, b'(' | b')') && !preceded_by_escape(path.as_bytes(), index)
            });
        if has_unescaped_paren || path.contains('<') || path.contains('>') {
            return Err("unsupported nested/angle destination syntax".into());
        }
        (path, rest.trim())
    };
    if !tail.is_empty() {
        let title = tail.as_bytes();
        if title.len() < 2
            || !matches!((title[0], title[title.len() - 1]), (b'"', b'"') | (b'\'', b'\''))
        {
            return Err("unsupported link title syntax".into());
        }
        if tail[1..tail.len() - 1].contains(title[0] as char) {
            return Err("ambiguous link title syntax".into());
        }
    }

    if destination.is_empty() {
        return Err("empty link destination".into());
    }
    if destination.starts_with('#') || destination.starts_with("//") {
        return Ok(None);
    }
    let prefix = destination
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(destination);
    if let Some((scheme, _)) = prefix.split_once(':') {
        match scheme.to_ascii_lowercase().as_str() {
            "http" | "https" | "mailto" => return Ok(None),
            _ => return Err("unsupported or unsafe URI scheme".into()),
        }
    }
    let path = destination.split(['#', '?']).next().unwrap_or("");
    if path.is_empty() {
        return Ok(None);
    }
    let decoded = decode_markdown_path(path)?;
    if decoded.is_empty() || decoded.contains('\0') {
        return Err("empty or NUL destination".into());
    }
    if decoded.contains('\\') {
        return Err("ambiguous backslash path".into());
    }
    if Path::new(&decoded).is_absolute() {
        return Err("absolute path not permitted".into());
    }
    Ok(Some(decoded))
}

/// Odd-length runs of backslashes escape the next Markdown punctuation token.
/// Even-length runs leave it active (with a literal backslash in the text).
fn preceded_by_escape(bytes: &[u8], index: usize) -> bool {
    let mut start = index;
    while start > 0 && bytes[start - 1] == b'\\' {
        start -= 1;
    }
    (index - start) % 2 == 1
}

fn collect_links(markdown: &str, document: &str, report: &mut Report) -> Vec<String> {
    let mut paths = Vec::new();
    let visible_markdown = mask_paragraph_code_spans(markdown);
    let mut fenced: Option<(u8, usize)> = None;
    for (line_idx, line) in visible_markdown.lines().enumerate() {
        if let Some((marker, width, can_close)) = fence_marker(line) {
            match fenced {
                None => {
                    fenced = Some((marker, width));
                    continue;
                }
                Some((old, old_width)) if old == marker && width >= old_width && can_close => {
                    fenced = None;
                    continue;
                }
                _ => {}
            }
        }
        if fenced.is_some() {
            continue;
        }
        let visible = mask_inline_code(line);
        let bytes = visible.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'[' || preceded_by_escape(bytes, i) {
                i += 1;
                continue;
            }
            // Markdown labels may contain balanced nested brackets. Stopping
            // at the first ] silently misses a valid [outer [inner]](file.md).
            // Escaped brackets do not affect the nesting depth.
            let mut after = i + 1;
            let mut label_depth = 1usize;
            while after < bytes.len() {
                if !preceded_by_escape(bytes, after) {
                    match bytes[after] {
                        b'[' => label_depth += 1,
                        b']' => {
                            label_depth -= 1;
                            if label_depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                after += 1;
            }
            if after == bytes.len() {
                i += 1;
                continue;
            }
            if bytes.get(after + 1) != Some(&b'(') {
                i = after + 1;
                continue;
            }
            let mut p = after + 2;
            let mut depth = 1usize;
            let mut quote: Option<u8> = None;
            // A ')' inside <...> is part of an angle-delimited destination,
            // not the closing ')' of the Markdown link.
            let mut in_angle_destination = false;
            while p < bytes.len() {
                let c = bytes[p];
                if let Some(q) = quote {
                    if c == q {
                        quote = None;
                    }
                } else if in_angle_destination {
                    if c == b'>' {
                        in_angle_destination = false;
                    }
                } else {
                    // An escaped parenthesis is path content, not a Markdown
                    // delimiter. Even-numbered backslash runs still close it.
                    if matches!(c, b'(' | b')') && preceded_by_escape(bytes, p) {
                        p += 1;
                        continue;
                    }
                    match c {
                        b'<' if p == after + 2 => in_angle_destination = true,
                        b'"' | b'\'' => quote = Some(c),
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                p += 1;
            }
            if depth != 0 {
                report.errors.push(format!(
                    "{document}:{}: unsupported/unclosed link syntax",
                    line_idx + 1
                ));
                i = after + 2;
                continue;
            }
            match parse_destination(&visible[after + 2..p]) {
                Ok(Some(path)) => paths.push(path),
                Ok(None) => {}
                Err(problem) => report.errors.push(format!(
                    "{document}:{}: {problem}",
                    line_idx + 1
                )),
            }
            i = p + 1;
        }
    }
    paths
}

fn check_target(root: &Path, document: &str, target: &str) -> Result<(), String> {
    let document_dir = root.join(document).parent().expect("rooted document").to_path_buf();
    // Lexical analysis is an early escape check, not the path to resolve:
    // collapsing symlink/.. first can validate a different file than the
    // filesystem follows, including one outside the repository.
    let mut lexical = document_dir.clone();
    for component in Path::new(target).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => lexical.push(part),
            Component::ParentDir => {
                lexical.pop();
                if !lexical.starts_with(root) {
                    return Err("link escapes repository".into());
                }
            }
            _ => return Err("absolute or unsupported path component".into()),
        }
    }
    if !lexical.starts_with(root) {
        return Err("link escapes repository".into());
    }
    // Resolve the original path with its symlink/.. order intact. The lexical
    // path above is deliberately not used for the filesystem lookup.
    match document_dir.join(target).canonicalize() {
        Ok(actual) if !actual.starts_with(root) => Err("link escapes repository (symlink)".into()),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err("target missing".into()),
        Err(e) => Err(format!("cannot inspect target: {e}")),
    }
}

fn check(root: &Path, docs: &[&str]) -> Report {
    let mut report = Report::default();
    let root = match root.canonicalize() {
        Ok(path) if path.is_dir() => path,
        Ok(_) => {
            report.errors.push("root: not a directory".into());
            return report;
        }
        Err(e) => {
            report.errors.push(format!("root: cannot resolve directory: {e}"));
            return report;
        }
    };
    for document in docs {
        let page = root.join(document);
        // Target links are checked against the checkout; mandatory source inputs
        // need the same boundary before their content is read.
        let canonical_page = match page.canonicalize() {
            Ok(path) if !path.starts_with(&root) => {
                report.errors.push(format!(
                    "{document}: source document escapes repository (symlink)"
                ));
                continue;
            }
            Ok(path) if !path.is_file() => {
                report.errors.push(format!("{document}: source document is not a file"));
                continue;
            }
            Ok(path) => path,
            Err(e) => {
                report.errors.push(format!("{document}: document unreadable or missing: {e}"));
                continue;
            }
        };
        let source = match fs::read_to_string(&canonical_page) {
            Ok(text) => text,
            Err(e) => {
                report.errors.push(format!("{document}: document unreadable or missing: {e}"));
                continue;
            }
        };
        if *document == "HELP-A-PROJECT.md" {
            if let Err(problem) = inspect_help_guide_contract(&source) {
                report.errors.push(format!("{document}: {problem}"));
            }
        }
        for path in collect_links(&source, document, &mut report) {
            report.local_links += 1;
            if let Err(reason) = check_target(&root, document, &path) {
                report.errors.push(format!("{document}: {reason}: {path}"));
            }
        }
    }
    report.errors.sort();
    report
}

fn inspect_help_guide_contract(guide: &str) -> Result<(), &'static str> {
    // Reuse the production Markdown fence recognizer: a short inner
    // delimiter must not close a longer fence or create a phantom prompt.
    let mut opening: Option<(u8, usize, bool)> = None;
    let mut contents = String::new();
    let mut prompts = Vec::new();
    for line in guide.lines() {
        if let Some((marker, width, closing_suffix)) = fence_marker(line) {
            if let Some((open_marker, open_width, is_prompt)) = opening {
                if marker == open_marker && width >= open_width && closing_suffix {
                    if is_prompt && contents.starts_with("I want to help [") {
                        prompts.push(std::mem::take(&mut contents));
                    }
                    opening = None;
                    contents.clear();
                    continue;
                }
            } else {
                let trimmed = line.trim_start_matches(' ');
                let language = trimmed[width..].trim();
                opening = Some((marker, width, marker == b'\x60' && language == "text"));
                contents.clear();
                continue;
            }
        }
        if opening.is_some() {
            contents.push_str(line);
            contents.push('\n');
        }
    }
    if opening.is_some() {
        return Err("unclosed code fence");
    }
    if prompts.len() != 1 {
        return Err("expected one copyable HELP prompt");
    }
    let prompt = &prompts[0];
    for (needle, diagnostic) in [
        ("Join the EXISTING project", "existing-project workflow missing"),
        ("If the project prohibits AI-assisted contributions", "AI policy denial missing"),
        (
            "remain read-only until the proposed public submission is permitted",
            "unknown AI policy fail-closed guard missing",
        ),
        ("Do not conceal AI involvement", "AI authorship disclosure guard missing"),
        ("An unassigned issue may still be claimed", "collision guard missing"),
        ("Never commit proprietary game assets", "rights guard missing"),
        ("Ask for authorization before a push", "external-write permission missing"),
        ("Never merge into default yourself", "default merge guard missing"),
    ] {
        if !prompt.contains(needle) {
            return Err(diagnostic);
        }
    }
    if !guide.contains("**Not available yet:**") {
        return Err("manual versus future capability disclosure missing");
    }
    if !guide.contains("](GETTING-STARTED.md)") {
        return Err("distinct Phase0 installation link missing");
    }
    Ok(())
}

fn main() {
    let mut arguments = env::args().skip(1);
    let mut root: Option<PathBuf> = None;
    while let Some(arg) = arguments.next() {
        match arg.as_str() {
            "--root" if root.is_none() => root = arguments.next().map(PathBuf::from),
            "--help" | "-h" => {
                println!("Usage: check-public-doc-links --root CHECKOUT_DIRECTORY");
                return;
            }
            _ => {
                eprintln!("FAIL: unsupported argument: {arg}");
                std::process::exit(2);
            }
        }
    }
    let root = root.unwrap_or_else(|| {
        eprintln!("FAIL: --root CHECKOUT_DIRECTORY is required");
        std::process::exit(2);
    });
    let result = check(&root, &DOCUMENTS);
    for problem in &result.errors {
        eprintln!("FAIL: {problem}");
    }
    println!(
        "Checked {} local links across {} documents; {} errors",
        result.local_links,
        DOCUMENTS.len(),
        result.errors.len()
    );
    if !result.errors.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!("free-energy-doclinks-{}-{id}", std::process::id()));
            fs::create_dir_all(&root).expect("fixture root");
            Self(root)
        }
        fn write(&self, path: &str, value: &str) {
            let full = self.0.join(path);
            fs::create_dir_all(full.parent().expect("parent")).expect("parent directory");
            fs::write(full, value).expect("fixture content");
        }
        fn scan(&self) -> Report {
            check(&self.0, &["README.md"])
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }


    #[test]
    fn invalid_backtick_fence_info_does_not_hide_broken_link() {
        assert!(fence_marker("\x60\x60\x60rust\x60invalid").is_none());
        let mut report = Report::default();
        let paths = collect_links(
            "\x60\x60\x60rust\x60invalid\n[broken](missing.md)\n\x60\x60\x60\n",
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["missing.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn tilde_fence_info_may_contain_backticks() {
        assert!(fence_marker("~~~rust\x60allowed").is_some());
        let mut report = Report::default();
        let paths = collect_links(
            "~~~rust\x60allowed\n[hidden](missing.md)\n~~~\n[real](present.md)\n",
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["present.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_parentheses_in_bare_link_destinations_resolve_real_files() {
        let sandbox = Sandbox::new();
        sandbox.write("docs/a)b.md", "existing close parenthesis path");
        sandbox.write("docs/(draft).md", "existing balanced parentheses path");
        sandbox.write(
            "README.md",
            r#"[one](docs/a\)b.md) [two](docs/\(draft\).md) [encoded](docs/a%29b.md)"#,
        );
        let report = sandbox.scan();
        assert_eq!(report.local_links, 3);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_parenthesis_does_not_hide_missing_file_or_unclosed_syntax() {
        let sandbox = Sandbox::new();
        sandbox.write("README.md", r#"[missing](docs/a\)b.md)"#);
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("target missing"), "{:?}", report.errors);

        sandbox.write("README.md", r#"[unterminated](docs/a\)b.md"#);
        let report = sandbox.scan();
        assert_eq!(report.local_links, 0);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(
            report.errors[0].contains("unsupported/unclosed link syntax"),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn unescaped_nested_parens_and_doubled_backslashes_remain_unsafe() {
        let sandbox = Sandbox::new();
        sandbox.write("docs/a)b.md", "real target must not excuse bad syntax");
        sandbox.write(
            "README.md",
            r#"[nested](docs/a(b).md) [ambiguous](docs/a\\)b.md)"#,
        );
        let report = sandbox.scan();
        assert!(
            report.errors.iter().any(|error| error.contains("unsupported nested/angle")),
            "{:?}",
            report.errors
        );
        assert!(
            report.errors.iter().any(|error| error.contains("ambiguous backslash")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn parentheses_inside_angle_destinations_do_not_close_links() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"[one](<docs/a)b.md>) [two](<docs/(draft).md> "title") [three](plain.md)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["docs/a)b.md", "docs/(draft).md", "plain.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn unclosed_angle_destination_still_fails_without_hiding_next_line() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"[bad](<unclosed)
[real](present.md)
"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["present.md"]);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("unsupported/unclosed link syntax"));
    }


    #[test]
    fn multiline_inline_code_span_does_not_create_false_missing_link() {
        let mut report = Report::default();
        let paths = collect_links(
            "\x60first code line\n[hidden](missing.md)\nlast code line\x60 [real](present.md)\n",
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["present.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn unmatched_multiline_code_opener_cannot_hide_real_links() {
        let mut report = Report::default();
        let paths = collect_links(
            "\x60unclosed opener\n[real](missing.md)\n",
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["missing.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn multiline_code_does_not_cross_blank_or_fenced_boundaries() {
        let mut report = Report::default();
        let paths = collect_links(
            "\x60unclosed opener\n\n[before](one.md)\n\x60\x60\x60text\n[hidden](skip.md)\n\x60\x60\x60\n[after](two.md)\n",
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["one.md", "two.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn code_formatted_link_label_still_checks_local_target() {
        let sandbox = Sandbox::new();
        sandbox.write("LICENSE", "MIT license fixture");
        sandbox.write("README.md", "See [`LICENSE`](LICENSE) and `[hidden](missing.md)` inside code.\n");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }
    #[test]
    fn escaped_backticks_leave_real_links_visible() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"\`[visible](missing.md)\` `[hidden](skip.md)` [also-visible](real.md)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["missing.md", "real.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_backtick_cannot_close_a_real_code_span() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"`literal \`[not-real](ignore.md) remains code` [real](exists.md)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["exists.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn info_string_does_not_close_fenced_code() {
        let mut report = Report::default();
        let paths = collect_links(
            concat!(
                "```rust\n",
                "[hidden](missing.md)\n",
                "```rust\n",
                "[still-hidden](missing.md)\n",
                "~~~/other-fence\n",
                "``\n", // shorter run does not close
                "[hidden-again](missing.md)\n",
                "``` \t \n", // only whitespace after marker may close
                "[first](one.md)\n",
                "~~~python\n",
                "[hidden-tilde](missing.md)\n",
                "~~~~not-a-closer\n",
                "[still-hidden-tilde](missing.md)\n",
                "~~~\n",
                "[second](two.md)\n"
            ),
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["one.md", "two.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_bracket_is_not_a_link_and_escaped_bang_does_not_hide_one() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"\[literal](missing.md) [normal](exists.md) \![also-real](other.md) ![image](missing.png)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["exists.md", "other.md", "missing.png"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn even_and_odd_backslash_runs_preserve_markdown_escape_parity() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"\\[normal](a.md) \\\[literal](missing.md) \\![image](missing.png) \![normal](b.md)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["a.md", "missing.png", "b.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_closing_brackets_in_labels_still_check_real_targets() {
        let mut report = Report::default();
        let paths = collect_links(
            r#"[odd \] text](a.md) [triple \\\] text](b.md) [even \\](c.md) \[ignored](none.md)"#,
            "README.md",
            &mut report,
        );
        assert_eq!(paths, vec!["a.md", "b.md", "c.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn escaped_closing_label_bracket_cannot_hide_missing_target() {
        let sandbox = Sandbox::new();
        sandbox.write("README.md", r#"[label \] more](missing.md)"#);
        let result = sandbox.scan();
        assert_eq!(result.local_links, 1);
        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(result.errors[0].contains("target missing"));
    }

    #[test]
    fn balanced_nested_labels_are_real_links() {
        let sandbox = Sandbox::new();
        sandbox.write("present.md", "present");
        sandbox.write(
            "README.md",
            "[outer [inner]](missing.md) [plain](present.md)\n",
        );
        let report = sandbox.scan();
        assert_eq!(report.local_links, 2);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("target missing: missing.md"));
    }

    #[test]
    fn nested_image_alt_labels_do_not_make_missing_links() {
        let mut report = Report::default();
        let links = collect_links(
            "![picture [alt](not-present.md)](missing.png) [real [link]](present.md)",
            "README.md",
            &mut report,
        );
        assert_eq!(links, vec!["missing.png", "present.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn local_image_destination_is_checked_but_nested_alt_link_is_not() {
        let sandbox = Sandbox::new();
        sandbox.write(
            "README.md",
            "![picture [alt](unrelated.md)](missing.png) [real](present.md)",
        );
        sandbox.write("present.md", "present");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 2);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(
            report.errors[0].contains("target missing: missing.png"),
            "{:?}",
            report.errors
        );
        sandbox.write("missing.png", "local image");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 2);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn unsafe_image_scheme_is_rejected_even_when_no_text_link_exists() {
        let sandbox = Sandbox::new();
        sandbox.write("README.md", "![unsafe](javascript:payload)");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 0);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("unsafe URI scheme"), "{:?}", report.errors);
    }

    #[test]
    fn valid_relative_title_fragment_percent_and_ignored_links() {
        let sandbox = Sandbox::new();
        sandbox.write("NEXT.md", "# next");
        sandbox.write("a b.md", "# space");
        sandbox.write("café.md", "# unicode");
        sandbox.write("image.png", "local image fixture");
        sandbox.write("README.md", concat!(
            "[one](NEXT.md#section) [two](<a%20b.md> \"title\")\n",
            "[three](caf%C3%A9.md 'title') [frag](#heading)\n",
            "[external](https://example.com/no.md) [mail](mailto:me@example.com)\n",
            "![image](image.png) \x60[inline](missing.md)\x60\n",
            "\x60\x60\x60md\n[code](missing.md)\n\x60\x60\x60\n"
        ));
        let result = sandbox.scan();
        assert_eq!(result.local_links, 4);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
    }

    #[test]
    fn broken_missing_document_and_relative_escape_fail() {
        let sandbox = Sandbox::new();
        assert_eq!(check(&sandbox.0, &["README.md"]).errors.len(), 1);
        sandbox.write("README.md", "[a](not-present.md) [b](../outside.md)\n");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 2);
        assert_eq!(report.errors.len(), 2);
        assert!(report.errors.iter().any(|e| e.contains("escapes repository")));
        assert!(report.errors.iter().any(|e| e.contains("target missing")));
    }

    #[test]
    fn unsafe_or_malformed_destinations_are_errors() {
        let sandbox = Sandbox::new();
        sandbox.write("README.md", concat!(
            "[absolute](/etc/passwd)\n",
            "[file](file:///etc/passwd)\n",
            "[drive](C:/secret)\n",
            "[slash](..\\outside.md)\n",
            "[traverse](%2e%2e/outside.md)\n",
            "[malformed](%ZZ)\n",
            "[short](%2)\n",
            "[utf8](%FF)\n",
            "[nested](nested(file).md)\n"
        ));
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1); // decoded traversal is a local candidate
        assert_eq!(report.errors.len(), 9, "{:?}", report.errors);
    }

    #[test]
    fn percent_decodes_only_once() {
        let sandbox = Sandbox::new();
        sandbox.write("%2e%2e", "literal directory or file");
        sandbox.write("README.md", "[literal](%252e%252e)\n");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn diagnostics_are_sorted_and_stable() {
        let sandbox = Sandbox::new();
        sandbox.write("README.md", "[z](z-missing.md)\n[a](a-missing.md)\n");
        let first = sandbox.scan();
        let second = sandbox.scan();
        assert_eq!(first.errors, second.errors);
        let mut sorted = first.errors.clone();
        sorted.sort();
        assert_eq!(first.errors, sorted);
    }

    #[cfg(unix)]
    #[test]
    fn mandatory_input_symlink_cannot_escape_checkout() {
        use std::os::unix::fs::symlink;
        let sandbox = Sandbox::new();
        let outside = Sandbox::new();
        outside.write("outside.md", "[secret](missing.md)\\n");
        symlink(outside.0.join("outside.md"), sandbox.0.join("README.md"))
            .expect("source symlink");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 0);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("source document escapes repository (symlink)"));
    }

    #[cfg(unix)]
    #[test]
    fn mandatory_input_symlink_inside_checkout_is_readable() {
        use std::os::unix::fs::symlink;
        let sandbox = Sandbox::new();
        sandbox.write("source.md", "[works](target.md)\\n");
        sandbox.write("target.md", "present");
        symlink(sandbox.0.join("source.md"), sandbox.0.join("README.md"))
            .expect("source symlink");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_before_parent_component_cannot_hide_external_target() {
        use std::os::unix::fs::symlink;
        let sandbox = Sandbox::new();
        let outside = Sandbox::new();
        outside.write("inner/marker.md", "directory exists");
        outside.write("outer.md", "external");
        // The old lexical-only lookup selected this in-checkout shadow.
        sandbox.write("outer.md", "inside shadow");
        symlink(outside.0.join("inner"), sandbox.0.join("jump")).expect("directory symlink");
        sandbox.write("README.md", "[bad](jump/../outer.md)\n");

        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("link escapes repository (symlink)"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_before_parent_component_resolves_inside_true_target() {
        use std::os::unix::fs::symlink;
        let sandbox = Sandbox::new();
        sandbox.write("nested/inner/marker.md", "directory exists");
        sandbox.write("nested/outer.md", "actual destination");
        // There is no root/outer.md: lexical collapse would report missing.
        symlink(sandbox.0.join("nested/inner"), sandbox.0.join("jump"))
            .expect("directory symlink");
        sandbox.write("README.md", "[good](jump/../outer.md)\n");

        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_must_not_escape_canonical_root() {
        use std::os::unix::fs::symlink;
        let sandbox = Sandbox::new();
        let outside = Sandbox::new();
        outside.write("secret.md", "private");
        symlink(outside.0.join("secret.md"), sandbox.0.join("in-tree.md")).expect("symlink");
        sandbox.write("README.md", "[bad](in-tree.md)\n");
        let report = sandbox.scan();
        assert_eq!(report.local_links, 1);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("symlink"));
    }

    // The manual contributor path is a source-level contract, not a hosted agent.
    // PR CI already runs this Rust test module, so guard the copyable prompt here.
    #[test]
    fn copyable_help_guide_policy_and_rights_contract() {
        let guide = include_str!("../HELP-A-PROJECT.md");
        assert_eq!(inspect_help_guide_contract(guide), Ok(()));
    }

    #[test]
    fn copyable_help_guide_rejects_removed_guards_and_broken_fences() {
        let guide = include_str!("../HELP-A-PROJECT.md");
        for (needle, diagnostic) in [
            (
                "If the project prohibits AI-assisted contributions",
                "AI policy denial missing",
            ),
            (
                "remain read-only until the proposed public submission is permitted",
                "unknown AI policy fail-closed guard missing",
            ),
            ("Never commit proprietary game assets", "rights guard missing"),
            ("Ask for authorization before a push", "external-write permission missing"),
        ] {
            let damaged = guide.replacen(needle, "", 1);
            assert_ne!(damaged, guide, "fixture anchor missing: {needle}");
            assert_eq!(inspect_help_guide_contract(&damaged), Err(diagnostic));
        }
        let duplicate = format!("{guide}\n```text\nI want to help [DUPLICATE]\n```\n");
        assert_eq!(
            inspect_help_guide_contract(&duplicate),
            Err("expected one copyable HELP prompt")
        );
        let unterminated = guide.replacen("\n```\n\n## 4.", "\n\n## 4.", 1);
        assert_ne!(unterminated, guide, "fixture must remove the closing prompt fence");
        assert_eq!(
            inspect_help_guide_contract(&unterminated),
            Err("unclosed code fence")
        );
    }

    #[test]
    fn help_prompt_extraction_respects_markdown_fence_length() {
        let guide = include_str!("../HELP-A-PROJECT.md");
        // A triple-backtick example inside a four-backtick Markdown fence
        // is literal text, not a second copyable agent prompt.
        let nested = format!("{guide}\n````markdown\n```text\nI want to help [FAKE].\n```\n````\n");
        assert_eq!(inspect_help_guide_contract(&nested), Ok(()));

        // A real four-backtick text fence containing a second prompt counts.
        let duplicate = format!("{guide}\n````text\nI want to help [SECOND].\n````\n");
        assert_eq!(
            inspect_help_guide_contract(&duplicate),
            Err("expected one copyable HELP prompt")
        );

        // A three-backtick close cannot terminate a four-backtick opener.
        let broken = guide.replacen(
            "\n```text\nI want to help [",
            "\n````text\nI want to help [",
            1,
        );
        assert_ne!(broken, guide, "fixture must lengthen the real prompt opener");
        assert_eq!(inspect_help_guide_contract(&broken), Err("unclosed code fence"));
    }

}
