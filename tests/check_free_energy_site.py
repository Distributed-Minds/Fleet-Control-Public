#!/usr/bin/env python3
"""Offline structural smoke checks for FREE ENERGY's static landing page.

Run from any directory: python3 tests/check_free_energy_site.py
No external HTTP requests, Python packages, browser, or GitHub credentials required.
"""

from html.parser import HTMLParser
from pathlib import Path
import sys
from urllib.parse import urlsplit


RELEASE = "v0.1.2-phase0-preview"
REPO = "https://github.com/Distributed-Minds/Fleet-Control-Public"
ZIP = f"{REPO}/releases/download/{RELEASE}/FREE-ENERGY-Phase0-Starter-v0.1.2-preview.zip"
GUIDE = f"{REPO}/blob/phase0/public-v0/GETTING-STARTED.md"


class Page(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids = []
        self.links = []
        self.stylesheets = []
        self.forbidden = []
        self.language = None
        self.viewport = False
        self.main_count = 0

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if "id" in a:
            self.ids.append(a["id"])
        if tag == "html":
            self.language = a.get("lang")
        if tag == "main":
            self.main_count += 1
        if tag == "meta" and a.get("name") == "viewport":
            self.viewport = True
        if tag == "a":
            self.links.append(a.get("href", ""))
        if tag == "link" and a.get("rel") == "stylesheet":
            self.stylesheets.append(a.get("href", ""))
        if tag in {"script", "iframe", "form", "object", "embed"}:
            self.forbidden.append(tag)

def validate(root):
    errors = []

    def require(ok, message):
        if not ok:
            errors.append(message)

    docs = root / "docs"
    for name in ("index.html", "styles.css", "README.md"):
        require((docs / name).is_file(), f"Missing docs/{name}")
    if errors:
        return errors

    html = (docs / "index.html").read_text(encoding="utf-8")
    css = (docs / "styles.css").read_text(encoding="utf-8")
    readme = (docs / "README.md").read_text(encoding="utf-8")
    page = Page()
    page.feed(html)
    page.close()

    require(page.language == "en", "HTML language must be English")
    require(page.viewport, "Missing viewport meta tag")
    require(page.main_count == 1 and "main" in page.ids, "Expected one main landmark with id=main")
    require(len(page.ids) == len(set(page.ids)), "Duplicate fragment ID")
    require(page.links and page.links[0] == "#main", "First link must skip to main")
    require(not page.forbidden, f"Unexpected active/embedded elements: {page.forbidden}")
    require(page.stylesheets == ["./styles.css"], "Expected one relative local stylesheet")

    for href in page.links:
        require(bool(href), "Anchor missing href")
        if href.startswith("#"):
            require(href[1:] in page.ids, f"Unresolved fragment: {href}")
        elif href:
            scheme = urlsplit(href).scheme
            require(scheme == "https", f"Unsafe or non-HTTPS outbound link: {href}")

    require(ZIP in page.links, "Starter download must link to exact released ZIP asset")
    require(f"{REPO}/releases/tag/{RELEASE}" in page.links, "Release overview must link to tagged release")
    require("Download the starter ZIP" in html, "ZIP CTA must describe a ZIP download")
    require(GUIDE in page.links, "Corrected online beginner guide not linked")
    require("older setup guide" in html and "before installing or forking" in html,
            "Archive's outdated setup instructions must be disclosed")
    require(f"{REPO}/discussions" in page.links, "Public Discussions contact link missing")
    require("Posting does not enroll a contributor" in html and "Do not post secrets" in html,
            "Contact must clarify enrollment and confidential-data boundaries")
    require("FUTURE VISION" in html and "FUTURE PLATFORM" in html and "Not yet available" in html,
            "Current-versus-future product boundary absent")
    require("Fleet-Control Phase0" in html, "Implemented orchestration preview not identified")
    require(":focus-visible" in css and ".skip:focus" in css,
            "Visible keyboard focus and skip-link styling required")
    require("@media(max-width:650px)" in css and "@media(max-width:980px)" in css,
            "Responsive breakpoint rules missing")
    require("@media(prefers-reduced-motion:reduce)" in css,
            "Reduced-motion override missing")
    require("python3 -m http.server 8000 --directory docs" in readme,
            "Local preview command missing from docs README")
    require("does **not** publish a website" in readme,
            "README must not imply deployment")
    require("predates the corrected online fork instructions" in readme,
            "README must warn about old setup guide packaged in released archive")
    return errors

if __name__ == "__main__":
    repo_root = Path(__file__).resolve().parents[1]
    problems = validate(repo_root)
    if problems:
        for problem in problems:
            print(f"FAIL: {problem}", file=sys.stderr)
        sys.exit(1)
    print("PASS: FREE ENERGY landing structural checks (offline; not browser, HTTP, or deployment QA)")
