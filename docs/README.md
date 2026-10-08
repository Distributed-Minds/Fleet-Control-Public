# FREE ENERGY static landing page

This directory is a zero-dependency landing-page prototype. It does **not** publish a website by itself, and intentionally does not select or hard-code a domain.

## Local preview

Open [`docs/index.html`](index.html) in a browser directly, or use an already available, trusted standalone static-file server for HTTP testing. The relative CSS path does not require a JavaScript runtime, Python, npm, remote fonts, tracking scripts, or remote images. Browser security policies may differ between local-file and HTTP previews.

## Offline structural smoke check (Rust)

Compile and run from the repository root with an installed Rust compiler and `mktemp`. The Unix commands use a per-run temporary directory, stop at the first failed stage, and remove both compiled binaries on success or failure, including when concurrent checks run:

```sh
(
  set -e
  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/free-energy-site.XXXXXXXX")"
  trap 'rm -rf -- "$tmp_dir"' EXIT
  rustc --edition=2021 -D warnings tests/check_free_energy_site.rs -o "$tmp_dir/site-check"
  "$tmp_dir/site-check"
  rustc --edition=2021 -D warnings --test tests/check_free_energy_site.rs -o "$tmp_dir/site-tests"
  "$tmp_dir/site-tests"
)
```


On **Windows PowerShell**, `/tmp/...` is not a native output path. From the repository root, compile and run the same Rust checker and unit tests as Windows executables:

```powershell
rustc --edition=2021 -D warnings tests/check_free_energy_site.rs -o .\free-energy-site-check.exe
if ($LASTEXITCODE -ne 0) { throw "Site checker compilation failed" }
.\free-energy-site-check.exe
if ($LASTEXITCODE -ne 0) { throw "Site checker failed" }
rustc --edition=2021 -D warnings --test tests/check_free_energy_site.rs -o .\free-energy-site-tests.exe
if ($LASTEXITCODE -ne 0) { throw "Unit-test compilation failed" }
.\free-energy-site-tests.exe
if ($LASTEXITCODE -ne 0) { throw "Unit tests failed" }
```

These Windows commands produce two local `.exe` files in the repository root; remove the generated executables when finished and do not commit them. A successful source smoke check remains narrower than actual browser, network, or installation validation.

No Cargo crates, dependency download, npm installation, scripting interpreter or external network requests are needed for this test. The checker validates local fragment links, the relative stylesheet, the direct starter ZIP and corrected-guide URLs, public Discussions/contact notices, PLAY/HELP/MAKE routes, implemented-versus-future disclosures, and basic keyboard-focus, responsive and reduced-motion CSS hooks. It is **not** a browser accessibility audit, live HTTP check, release installation test or deployment verification.

## Deployment and migration

1. Choose and verify the website domain/host first. Do not create a CNAME or assume a domain in this repository.
2. For GitHub Pages branch deployment, select the desired branch and /docs directory in repository settings. That is a separate, human-owned administrative decision.
3. Other static hosts can publish this directory directly; local CSS uses relative URLs.
4. Verify links to the actual preview release, starter pack, beginner guide, and license immediately before publishing.
   The v0.1.2 published ZIP predates the corrected online fork instructions merged via PR #59. A default-branch-only fork omits Phase0; use the current online guide, and do not present the archived guide as updated.
5. After the GitHub slug rename, replace the legacy Distributed-Minds/Fleet-Control-Public links. Preserve historical citations and commit provenance.
6. Maintain the distinction: FREE ENERGY is the public ecosystem, Fleet-Control is orchestration, Phase0 v0.1.2 is the available preview. Do not advertise envisioned platform functionality as shipped.
7. Review keyboard focus, skip link, mobile layouts, reduced-motion behavior, and color contrast before public launch.

Tracked by https://github.com/Distributed-Minds/Fleet-Control-Public/issues/57 .
