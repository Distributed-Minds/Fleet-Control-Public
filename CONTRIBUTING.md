# Contributing to FREE ENERGY

This guide is for contributing **to this repository**. To help a different game or modding project, use [Help an existing project](HELP-A-PROJECT.md) and follow *that project's* own rules instead.

**Current product boundary:** FREE ENERGY is the public project; Fleet-Control is its orchestration subsystem. The configured public default `main` presently has a landing README and MIT license, but **not** the installable Phase0 source. The current working preview lives on [`phase0/public-v0`](https://github.com/Distributed-Minds/Fleet-Control-Public/tree/phase0/public-v0). The static website is source in `docs/`, not evidence of a deployed site. The playable catalog and universal centrally dispatched contributor fleet are **not implemented**; see [#60](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/60) and [#70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70).

## Find a useful contribution

1. Start with the [open issues](https://github.com/Distributed-Minds/Fleet-Control-Public/issues) and [open PRs](https://github.com/Distributed-Minds/Fleet-Control-Public/pulls). Look for an actual reproducible problem, unimplemented acceptance criterion, documentation error, missing regression case, or installer defect.
2. Read the latest issue discussion, linked PRs, branch activity, and [coordination rules](Phase0/30-COORDINATION.md) before assuming work is unclaimed. An unassigned issue can still have an active owner. Do not impersonate an installed fleet agent or treat a comment as server-issued assignment.
3. When the task needs a decision, ask on the existing issue or use [public Discussions](https://github.com/Distributed-Minds/Fleet-Control-Public/discussions) for general questions. A public post is not automatic worker enrollment or GitHub mutation authority.
4. Keep one bounded change per PR. Preserve prior contributions and exact source links; avoid new duplicate trackers or parallel integration branches for work already owned.

## Get the correct source

For a local checkout of the current preview:

```sh
git clone https://github.com/Distributed-Minds/Fleet-Control-Public.git
cd Fleet-Control-Public
git switch phase0/public-v0
git switch -c docs/my-bounded-fix
```

If contributing from a fork, make sure it contains `phase0/public-v0`; a fork that copies **only the default branch** does not. See [Getting Started](GETTING-STARTED.md#2-install-the-actual-phase0-preview-in-a-repository-you-control) for the fork/default-branch caveat. Work on a **non-default branch** of a repository you control; confirm your push permission and proposed PR target before publishing. A task may name a different non-default integration target, so follow the *current issue and maintainer* rather than assuming every change belongs in Phase0.

## Build and test the relevant change

- For Markdown and onboarding changes, validate relative links, exact branch/release targets, the current-vs-planned capability statements, and command snippets you altered. The preview contains a [Rust public-document link checker](GETTING-STARTED.md); use its current source instructions, not a previously generated executable.
- For changes to the static landing page in `docs/`, run the compiled, dependency-free Rust structural checker and unit tests using the commands in [`docs/README.md`](docs/README.md). These checks are narrower than a real browser, keyboard, mobile or network test.
- For Phase0 semantics or any other Rust changes, follow the relevant issue's exact-spec fixtures and available compiler/test guidance. Record the toolchain version and exact commands. **No npm installation or npm-backed dependencies** are allowed for FREE ENERGY's maintained implementation, CI or contributor skills under [#70](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/70).
- State **PASS / FAIL / NOT RUN** separately for compilation, unit tests, integration checks, installation, manual browser testing and actual gameplay. An absent GitHub check is **not** a CI pass. Never claim the historical starter ZIP includes recent preview fixes.

## Contribution certification: MIT + DCO 1.1 (rollout pending)

The maintainer has selected the existing MIT license and [Developer Certificate of Origin (DCO) 1.1](https://developercertificate.org/) as the default inbound contribution policy ([issue #159](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/159)). This is a **policy selection**, not an installed DCO enforcement check, blanket rights approval, or retroactive certification of earlier commits. The contributor consent, bot/harness identity, exceptions, and enforcement workflow are still under review.

Read the DCO before certifying a contribution. If **you** have the authority to make that declaration and have inspected the actual staged change, a local Git client can add your declaration with `git commit -s`. A `Signed-off-by:` trailer is an accountable assertion of origin and rights, **not** cryptographic commit signing, automated legal clearance, or proof of consent merely because a GitHub badge says “Verified.”

AI agents and centrally operated integration services must never fabricate or silently add a human's `Signed-off-by:` declaration. A contributor must review the proposed patch, check licensing and third-party rights, and make any required certification under their own accountable identity. If author consent, employer ownership, third-party media rights, or a harness's identity capabilities are uncertain, do not represent the contribution as cleared; seek maintainer review. This notice does **not** enable checks, change repository settings, authorize merges into `main`, or establish a signed-off exception for bots.

## Rights, privacy and review

Only contribute material you have the right to publish. The [root MIT license](LICENSE) covers this repository's licensed material; it does **not** grant rights to unrelated games, trademarks, sound, art, proprietary builds, or extracted assets. Preserve attribution and third-party notices; never commit game dumps, secrets, personal paths, credentials, private repository content, or confidential reports.

Public issues and Discussions are **not confidential security intake**. Do not post vulnerability details there. A verified private reporting route is not yet established; [#38](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/38) tracks that gap. Do not invent a private channel or disclosure guarantee.

Before opening a PR, recheck its base/head, inspect the diff and staged files, reconcile concurrent work, and identify the issue or acceptance condition it satisfies. Explain the exact change, rights/provenance, tests actually run and missing verification. Be transparent about AI-assisted content when relevant; neither a generic prompt nor GitHub write permission overrides the project's current human and security rules.

**Never merge into the configured public default branch yourself.** The maintainer owns default integration, repository settings, publication, releases and deployment. A merged non-default PR does not mean the change is on `main` or in an already-published ZIP. For a reviewable website/Phase0 candidate, target the maintained non-default preview unless current human instructions explicitly select another authorized integration branch.
