# Signet 2 public-fork delivery plan

**Decision:** the maintainer-facing artifact will be a public fork of Signet, not a patch bundle.

## Target

Upstream:

`kian-cx/signetprotocol`

Public fork:

`geromet/signetprotocol`

Review branch:

`review/signet2-interoperability`

Base the branch on the current upstream Signet revision being reviewed. At the time of this plan the pinned draft snapshot is:

`2ddb136ee941705d3e1c020eaddad93be65026f2`

Before building the branch, verify whether upstream moved. If it did, rebase the review against current upstream where practical and record any draft changes that alter our conclusions.

## What the maintainer should see

The review branch should look like an ordinary Signet branch:

- existing Signet 2 / Forge docs edited directly;
- new proposal docs added directly where useful;
- upstream docs build passing;
- no Fleet-Control coordination machinery;
- no OpenCode agent files;
- no copied research-lane trees;
- no patch files as the primary delivery mechanism.

The branch should be understandable by browsing GitHub or checking it out locally.

## Relationship to Fleet-Control-Public

`Distributed-Minds/Fleet-Control-Public` remains the research/evidence notebook.

It contains:

- detailed response paper;
- experiments;
- executable vectors;
- resolver evaluation;
- adapter evidence;
- trust/validation research;
- historical proposed patch series;
- independent validation notes.

The Signet fork contains the **reviewable proposal**.

Fleet-Control contains the **supporting evidence**.

Do not make the Signet maintainer reconstruct the proposal from Fleet-Control first.

## Known corrections required before creating the final review branch

Use:

`Testing/Signet-2-DeepSeek/VALIDATION-SUMMARY.md`

as the current stop-ship list.

### L1

Repair on a dedicated correction branch rather than rewriting the historical L1 head.

Required work:

- define one normative cross-language semantic-reference ordering rule;
- make Python and JavaScript implement exactly that rule;
- add non-BMP adversarial vectors;
- make rejection/failure selection deterministic for declared-unordered collections;
- enforce HASH05;
- rerun the complete L1 corpus in both languages;
- record the corrected evidence head.

### L2

Repair on a dedicated correction branch rather than rewriting the historical L2 head.

Required work:

- fix malformed `role-contract.schema.json`;
- regenerate every generated JSON/header/Rust artifact from the generator;
- regenerate the digest/evidence output;
- run `check.py`;
- rerun deterministic-generation checks;
- preserve the original broken evidence as historical context rather than silently pretending it never existed;
- record the corrected evidence head.

### Patch bundle

The old patch files are internal research artifacts.

Do not send them to the maintainer.

Do not require them to apply cleanly for the public-fork delivery path.

Remove or correct any maintainer-facing claim that says the committed patch bundle itself was validated with `APPLY_CHECK=PASS` unless that exact committed bundle has been retested and repaired.

## Public-fork content

### P0: lead with these

1. stable/versioned semantic identity;
2. small semantic core plus modular profiles;
3. capability offers -> deterministic negotiated session contract;
4. explicit required/optional semantics;
5. typed fallback;
6. semantic acceptability separated from session preference;
7. deterministic optional-profile activation;
8. translation-profile authority classes;
9. publisher baseline separated from participant overlay;
10. `NO_MATCH / ABSTAIN` for resolver calls;
11. deterministic composition/canonicalization;
12. executable conformance vectors.

### P1: include but keep secondary

- profile dependency provenance;
- compatibility freshness/invalidation;
- effective-profile identity;
- integration mode as part of compatibility/trust evidence;
- declared permission versus actual enforcement;
- exact evidence-scope terminology.

### P2+: keep as future-facing notes

- richer package/update framework;
- TUF/Sigstore-style trust distribution;
- mature revocation/lifecycle infrastructure;
- detailed ecosystem governance;
- larger validation registries.

Do not make P2+ machinery look like a prerequisite for beginning Signet 2.

## Recommended fork structure

Prefer direct edits to current upstream docs plus at most a few focused proposal pages.

Example:

```text
docs/content/proposals/translation-profiles.mdx
docs/content/forge/architecture.mdx
docs/content/forge/workflow.mdx
docs/content/forge/understanding-a-game.mdx
docs/content/forge/how-the-model-decides.mdx
docs/content/forge/reliability-and-limits.mdx
docs/content/forge/fine-tuning.mdx
docs/content/forge/roadmap.mdx

docs/content/proposals/semantic-contracts.mdx
docs/content/proposals/translator-trust-and-validation.mdx
```

The trust/validation proposal should be shortened if needed so that P0/P1/P2 priority is obvious.

## Validation before maintainer contact

The fork branch is sendable only when:

1. upstream base is recorded;
2. corrected L1 evidence passes in Python and JavaScript, including new adversarial vectors;
3. corrected L2 role-contract evidence self-reproduces from source;
4. direct Signet docs modifications build using upstream's documented tooling;
5. no material upstream strawman is introduced;
6. all experimental claims link to an exact corrected evidence revision;
7. the fork contains no internal Fleet-Control operational clutter;
8. the first page/readme clearly distinguishes:
   - implemented upstream behavior;
   - upstream draft proposal;
   - our proposed changes;
   - experimental evidence;
9. a final targeted adversarial review returns no HIGH stop-ship finding.

## Fork creation

If the personal fork does not exist:

```bash
gh repo fork kian-cx/signetprotocol --clone=false
```

A fork of this public repository should remain public.

Verify:

```bash
gh repo view geromet/signetprotocol --json nameWithOwner,visibility,parent
```

Then create and push:

`review/signet2-interoperability`

Do not open an upstream PR automatically.

Do not create an upstream issue automatically.

Do not contact the maintainer automatically.

The stopping point is:

- public branch exists;
- branch build/tests are recorded;
- branch URL is ready;
- concise Discord message is drafted.

The human can then send the Discord message with one branch link.

## Maintainer-facing message strategy

Lead with:

- Signet 2's overall architecture is good;
- we tested several boundary cases;
- we made a public fork applying the feedback directly;
- semantic/determinism changes are the main proposal;
- deeper trust/governance notes are secondary;
- disagreement is welcome.

Do not lead with:
- a giant paper;
- patch application;
- Fleet-Control architecture;
- claims that Signet is broken;
- a demand to adopt the whole design.
