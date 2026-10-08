//! Offline checker for relative links in the eight FREE ENERGY onboarding documents.
//!
//! Build: rustc --edition=2021 -D warnings scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links
//! Run:   /tmp/free-energy-doc-links --root /absolute/path/to/checkout
//! Tests: rustc --edition=2021 --test -D warnings scripts/check-public-doc-links.rs -o /tmp/free-energy-doc-links-tests
//!        /tmp/free-energy-doc-links-tests
//!
//! Supported: ordinary inline Markdown links with file destinations, fragments,
//! optional quoted titles, and angle-delimited destinations. This is not a full
//! CommonMark or HTTP checker. Unsupported destination syntax is an error.

use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

const DOCUMENTS: [&str; 8] = [
    "README.md",
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
        let n = original[i..].iter().take_while(|&&x| x == b'\x60').count();
        let mut j = i + n;
        let mut ending = None;
        while j < original.len() {
            if original[j] == b'\x60' {
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
            masked[i..end].fill(b' ');
            i = end;
        } else {
            i += n;
        }
    }
    String::from_utf8(masked).expect("replacing ASCII backticks preserves UTF-8")
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
        if path.contains('(') || path.contains(')') || path.contains('<') || path.contains('>') {
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
    let decoded = decode_once(path)?;
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
    let mut fenced: Option<(u8, usize)> = None;
    for (line_idx, line) in markdown.lines().enumerate() {
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
            if bytes[i] != b'['
                || preceded_by_escape(bytes, i)
                || (i > 0 && bytes[i - 1] == b'!' && !preceded_by_escape(bytes, i - 1))
            {
                i += 1;
                continue;
            }
            let after = match bytes[i + 1..].iter().position(|&c| c == b']') {
                Some(k) => i + 1 + k,
                None => {
                    i += 1;
                    continue;
                }
            };
            if bytes.get(after + 1) != Some(&b'(') {
                i = after + 1;
                continue;
            }
            let mut p = after + 2;
            let mut depth = 1usize;
            let mut quote: Option<u8> = None;
            while p < bytes.len() {
                let c = bytes[p];
                if let Some(q) = quote {
                    if c == q {
                        quote = None;
                    }
                } else {
                    match c {
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
    let mut lexical = root.join(document).parent().expect("rooted document").to_path_buf();
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
    match lexical.canonicalize() {
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
        let source = match fs::read_to_string(&page) {
            Ok(text) => text,
            Err(e) => {
                report.errors.push(format!("{document}: document unreadable or missing: {e}"));
                continue;
            }
        };
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
        assert_eq!(paths, vec!["exists.md", "other.md"]);
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
        assert_eq!(paths, vec!["a.md", "b.md"]);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    #[test]
    fn valid_relative_title_fragment_percent_and_ignored_links() {
        let sandbox = Sandbox::new();
        sandbox.write("NEXT.md", "# next");
        sandbox.write("a b.md", "# space");
        sandbox.write("café.md", "# unicode");
        sandbox.write("README.md", concat!(
            "[one](NEXT.md#section) [two](<a%20b.md> \"title\")\n",
            "[three](caf%C3%A9.md 'title') [frag](#heading)\n",
            "[external](https://example.com/no.md) [mail](mailto:me@example.com)\n",
            "![image](missing.png) \x60[inline](missing.md)\x60\n",
            "\x60\x60\x60md\n[code](missing.md)\n\x60\x60\x60\n"
        ));
        let result = sandbox.scan();
        assert_eq!(result.local_links, 3);
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
}
