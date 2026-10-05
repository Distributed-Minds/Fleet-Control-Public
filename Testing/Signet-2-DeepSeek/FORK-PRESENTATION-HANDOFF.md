# Signet 2 fork presentation fix

## Problem

The public fork is technically ready, but the maintainer-facing presentation is still too diffuse.

Current branch:

`geromet/signetprotocol@review/signet2-interoperability`

Current head when this handoff was written:

`ab313ff9e3005ac52451d985b6574e7f74b41fce`

The branch is docs-only and the important changes are spread across the docs tree. A maintainer opening the branch should not have to browse around to discover what the proposal actually is.

## Goal

Make the branch self-explanatory from the repository root.

The maintainer should be able to:

1. open the branch;
2. see one obvious "start here" link immediately;
3. read one concise page in roughly 2–4 minutes;
4. understand:
   - what we agree with;
   - the 5 core changes;
   - what is P0 vs P1/P2;
   - what evidence actually ran;
   - what is explicitly *not* being proposed;
   - which 2–3 files to inspect next.

Do not make them read Fleet-Control first.

## Required changes in the public fork

### 1. Add root-level `SIGNET2-REVIEW.md`

This is the primary human handoff.

Keep it concise. Prefer roughly 100–180 lines, not another architecture paper.

Use this structure:

~~~markdown
# Signet 2 review — start here

This branch is a small, concrete review of the Signet 2 / Forge draft.

I think the core direction is right:

game
→ shared meaning
→ authoritative server
→ shared meaning
→ game

The main thing I think is still missing is that "shared meaning" needs to become an explicit, deterministic contract before independent translators start depending on it.

## The short version

I would keep Signet 2's architecture, but tighten five boundaries:

1. Semantic identity
   - `fire` is a label, not a complete protocol identity.
   - Give semantic items stable versioned identity + normative definition.

2. Capability negotiation
   - Capabilities are an offer, not final behavior.
   - Derive one deterministic session contract from offers + session requirements.

3. Optional behavior
   - "Optional" should not mean "enable it if available."
   - Activate in declared preference order and record exact skip reasons.

4. Resolver failure
   - Closed candidate lists are good, but the right answer can still be absent.
   - Support `NO_MATCH / ABSTAIN`.

5. Pinned does not mean current
   - Pinning makes a decision repeatable.
   - It does not prove compatibility after dependencies change.

That is the core proposal.

Everything else is secondary.

## What I think is P0 vs later

### P0 — settle before independent Signet 2 translators

- versioned semantic identity;
- small core + modular profiles;
- deterministic negotiated session contract;
- required vs optional semantics;
- typed fallback;
- preference separated from semantic acceptability;
- deterministic optional-profile activation;
- profile authority classes;
- publisher baseline vs participant overlay;
- `NO_MATCH / ABSTAIN`;
- canonical composition/hashing;
- executable cross-language conformance vectors.

### P1 — useful boundaries, secondary

- mapping provenance;
- compatibility freshness;
- effective-profile identity;
- integration mode;
- declared vs enforced permission;
- precise evidence labels.

### P2+ — not blockers

- mature package/update trust infrastructure;
- richer revocation/lifecycle machinery;
- ecosystem governance;
- large validation registries.

## What I am not proposing

- replacing game → meaning → game;
- AI in the real-time protocol path;
- one giant ontology;
- every game implementing every profile;
- changing Signet/1 compatibility rules;
- building a heavyweight security stack before Signet 2 can ship;
- treating our research prototype as production behavior.

## Smallest useful test

Freeze approximately:

5–8 intents
4–6 archetypes
1 movement/body profile
1 damage/death lifecycle
1 optional profile
1 typed fallback

Build two materially different translators.

Success:

same semantic definitions
+ same capability offers
+ same session requirements
→ same negotiated contract

If independent implementations produce different contracts, the specification is missing a rule.

## What we actually tested

### L1 deterministic semantics

Corrected evidence:

`Distributed-Minds/Fleet-Control-Public@f109e15ed1c73b0c76876524ec3c126e34bd63ae`

The cross-language exercise found and fixed:
- non-BMP string-order divergence;
- input-order-dependent rejection reasons;
- missing HASH05 enforcement.

Python and JavaScript now reproduce the same corpus plus adversarial vectors.

### L2 adapter evidence

Corrected evidence:

`Distributed-Minds/Fleet-Control-Public@ba65b131559a28cf0cb7d3416d492dbd1d91b316`

The first generic adapter abstraction was falsified by the Minecraft RCON fixture.

Original coverage:
- NO = 5
- PARTIAL = 2

The revised role fixture reproduces 7/7 coverage using:
HOST / IMPORTER / WORLD / INPUT / PRESENTATION / AUTHORITY

This is prototype evidence, not a claim that these six roles are the final API.

### Resolver warning

Top-1 alone was misleading:

oracle/lookup:
  top-1 1.000
  no-match 1.000
  order-stability 1.000

lexical:
  top-1 1.000
  no-match 0.000
  order-stability ~0.167

first-candidate:
  top-1 1.000
  no-match 0.000
  order-stability 0.000

So test `NO_MATCH`, candidate-order perturbation and overrides—not just top-1.

## Read next

If you want more detail:

1. `docs/content/proposals/semantic-contracts.mdx`
2. `docs/content/proposals/translation-profiles.mdx`
3. `docs/content/forge/roadmap.mdx`

Only then, if useful:

4. `docs/content/proposals/translator-trust-and-validation.mdx`

You should not need to read the Fleet-Control repository unless you want to inspect the experiments.

## Validation status

Base upstream:
`2ddb136ee941705d3e1c020eaddad93be65026f2`

Validated locally:
- docs build PASS;
- 43/43 routes;
- changed-page internal links PASS;
- corrected L1 cross-language vectors PASS;
- corrected L2 reproduction PASS;
- final adversarial audit: no HIGH finding.

Deliberate limitation:

`ABI_HOST_HARNESS_PASS != GODOT_RUNTIME_PASS`

Godot runtime reproduction was blocked, so do not claim it.

## Main question

Should Signet 2 define a deterministic, versioned semantic/session contract before independent translators are built against the shared vocabulary?

If yes, the exact P0 mechanics can be simplified together.
~~~

Improve wording where useful, but preserve the hierarchy and keep it compact.

### 2. Add a prominent README banner

Directly after the badges / initial header, add something like:

~~~markdown
> [!IMPORTANT]
> **This is the `review/signet2-interoperability` review branch.**
> Start with **[SIGNET2-REVIEW.md](SIGNET2-REVIEW.md)** for the 2–4 minute summary.
>
> The branch reviews the Signet 2 / Forge draft. It keeps the existing architecture and proposes a deterministic semantic/session contract around it. The detailed docs are secondary.
~~~

Do not rewrite the rest of upstream README.

### 3. Keep the docs tree

Do not delete the existing detailed proposal pages. They are the drill-down.

The new hierarchy should be:

~~~text
README banner
    ↓
SIGNET2-REVIEW.md
    ↓
semantic-contracts.mdx / translation-profiles.mdx / roadmap.mdx
    ↓
trust-validation.mdx
    ↓
external evidence only when wanted
~~~

### 4. No new giant report

Do not add another long paper, generated PDF, issue template, patch bundle, or Fleet-Control metadata to the fork.

### 5. Validate

After the presentation edit:

- run the docs build again;
- verify the README link to `SIGNET2-REVIEW.md`;
- verify every evidence SHA;
- inspect the root branch view as if you were the maintainer;
- use a fresh subagent for a final "can I understand the point without digging?" test.

Pass condition:

A fresh reviewer can summarize the branch's main proposal, P0 scope, experimental evidence and limitations after reading only the README banner + `SIGNET2-REVIEW.md`.

Do not contact upstream.
