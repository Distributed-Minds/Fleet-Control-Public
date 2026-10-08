#!/usr/bin/env python3
"""Check local Markdown links in public FREE ENERGY onboarding documents.

This checks repository paths, not remote HTTP links or Markdown fragments.
Run from any directory: python3 scripts/check-public-doc-links.py
"""
import argparse
from pathlib import Path
import re
import tempfile
from urllib.parse import unquote, urlsplit

ENTRYPOINTS = (
    "README.md",
    "GETTING-STARTED.md",
    "HELP-A-PROJECT.md",
    "WORKFLOW-GUIDES.md",
    "TROUBLESHOOTING.md",
    "BRANDING.md",
    "release-kit/README-FIRST.md",
    "release-kit/INSTALL-CHECKLIST.md",
)
LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})")
INLINE_CODE = re.compile(r"`[^`\n]*`")


def links(text):
    """Yield local Markdown link destinations outside fenced and inline code."""
    marker = None
    for line in text.splitlines():
        fence = FENCE.match(line)
        if fence:
            seq = fence.group(1)
            if marker is None:
                marker = seq
            elif seq[0] == marker[0] and len(seq) >= len(marker):
                marker = None
            continue
        if marker is not None:
            continue
        line = INLINE_CODE.sub("", line)
        for match in LINK.finditer(line):
            target = match.group(1).strip()
            if target.startswith("<") and ">" in target:
                target = target[1:target.index(">")]
            else:
                target = re.split(r"\s+['\"]", target, maxsplit=1)[0]
            if target and not target.startswith("#"):
                parsed = urlsplit(target)
                if not parsed.scheme and not parsed.netloc and parsed.path:
                    yield unquote(parsed.path)


def check(root, entrypoints=ENTRYPOINTS):
    root = root.resolve()
    errors = []
    checked = 0
    for name in entrypoints:
        page = root / name
        if not page.is_file():
            errors.append(f"{name}: document missing")
            continue
        for target in links(page.read_text(encoding="utf-8")):
            checked += 1
            dest = (page.parent / target).resolve()
            if not dest.is_relative_to(root):
                errors.append(f"{name}: link escapes repository: {target}")
            elif not dest.exists():
                errors.append(f"{name}: target missing: {target}")
    return checked, errors


def self_test():
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "README.md").write_text(
            "[works](NEXT.md#section) ` [ignore](NO.md) `\n"
            "```md\n[ignore](NO.md)\n```\n[external](https://example.com/no.md)\n",
            encoding="utf-8",
        )
        (root / "NEXT.md").write_text("# Section\n", encoding="utf-8")
        count, errors = check(root, ("README.md",))
        assert count == 1 and not errors, (count, errors)
        (root / "README.md").write_text("[broken](NO.md) [escape](../NO.md)\n", encoding="utf-8")
        count, errors = check(root, ("README.md",))
        assert count == 2 and len(errors) == 2, (count, errors)
    print("PASS: link checker positive and negative fixtures")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    count, errors = check(args.root)
    for message in errors:
        print("FAIL:", message)
    print(f"Checked {count} local links across {len(ENTRYPOINTS)} documents; {len(errors)} errors")
    if errors:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
