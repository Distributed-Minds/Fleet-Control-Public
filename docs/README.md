# FREE ENERGY static landing page

This directory is a zero-dependency landing-page prototype. It does **not** publish a website by itself, and intentionally does not select or hard-code a domain.

## Local preview

From the repository root, run: python3 -m http.server 8000 --directory docs

Then open http://localhost:8000/. No npm dependencies, JavaScript runtime, remote fonts, tracking scripts, or remote images are required.

## Offline structural smoke check

From the repository root:

```sh
python3 tests/check_free_energy_site.py
```

This standard-library-only check validates local fragment links, the relative stylesheet, the direct starter ZIP and corrected-guide links, public Discussions/contact notices, implemented-versus-future disclosures, and basic keyboard-focus, responsive, and reduced-motion CSS hooks. It makes **no network requests** and is not a substitute for a browser accessibility review, external-link checks, release installation, or a deployed-site test.

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
