# Proposed upstream Signet 2 patch series

**Target upstream:** `kian-cx/signetprotocol`  
**Exact base:** `2ddb136ee941705d3e1c020eaddad93be65026f2`  
**Generated:** 2026-10-05  
**Status:** independent proposal; not applied to upstream

This directory converts the Signet 2 response research into a reviewable three-part documentation patch series.

The patches deliberately separate low-risk wording/semantic corrections from the larger proposed contract and trust architecture.

## Apply order

~~~text
0001-harden-draft-semantics.patch
0002-semantic-contracts-and-profile-composition.patch
0003-translator-trust-and-validation.patch
~~~

### Patch 1 — harden the current draft

Touches current upstream Signet 2 / Forge docs.

Main changes:

- capability declarations become inputs to a negotiated session contract;
- generic "ignore or approximate" wording is narrowed for simulation semantics;
- resolver API can return `NO_MATCH`;
- pinned decisions are deterministic but can become compatibility-stale;
- server-authority wording no longer implies complete cheat impossibility;
- model suggestions are explicitly advisory;
- local mapping approval is separated from shared dataset contribution;
- the planned resolver experiment gains no-match and candidate-order tests;
- the roadmap becomes contract-first before model dependence.

This is the smallest patch and is intentionally useful on its own.

### Patch 2 — semantic contract proposal

Adds:

~~~text
docs/content/proposals/semantic-contracts.mdx
~~~

It proposes:

- stable semantic references;
- small core plus modular profiles;
- capability offers versus negotiated session contract;
- deterministic ordering / visibility selection;
- deterministic optional-profile activation;
- typed fallbacks;
- archetype versus authoritative ruleset separation;
- profile authority classes;
- publisher baseline plus participant overlay;
- compatibility freshness;
- effective-profile digest;
- canonical composition / hashing;
- executable conformance vectors.

This patch is additive. It does not force the proposal to become normative immediately.

### Patch 3 — translator trust and validation proposal

Adds:

~~~text
docs/content/proposals/translator-trust-and-validation.mdx
~~~

It proposes:

- integration mode as part of the trust subject;
- role contracts separate from sandbox enforcement;
- signature separate from current authorization;
- artifact provenance and reproducibility;
- effective-profile-aware compatibility evidence;
- lifecycle / revocation states;
- V0-V5 evidence scopes;
- pairwise interoperability distinct from local conformance;
- preservation of negative evidence;
- profile-only changes as behavioral supply-chain changes;
- local-first Forge and explicit export boundaries;
- local approval separate from dataset contribution;
- dataset invalidation lineage;
- separate governance authorities.

## Mechanical applicability check

The three saved patch files were replayed in order against the exact upstream content at:

~~~text
kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2
~~~

Result:

~~~text
APPLY_CHECK = PASS
~~~

The replay verified all context and deletion lines exactly against the base snapshot and reconstructed the expected resulting files.

This is a patch-application check, **not** an upstream build, docs-render, CI or maintainer-approval result.

No upstream repository was mutated.

## Files affected by the series

Current files modified by patch 1:

~~~text
docs/content/proposals/translation-profiles.mdx
docs/content/forge/architecture.mdx
docs/content/forge/workflow.mdx
docs/content/forge/understanding-a-game.mdx
docs/content/forge/how-the-model-decides.mdx
docs/content/forge/reliability-and-limits.mdx
docs/content/forge/fine-tuning.mdx
docs/content/forge/roadmap.mdx
~~~

Files added by patches 2 and 3:

~~~text
docs/content/proposals/semantic-contracts.mdx
docs/content/proposals/translator-trust-and-validation.mdx
~~~

## Local application

From a checkout of the upstream repository at the exact base revision:

~~~bash
git checkout 2ddb136ee941705d3e1c020eaddad93be65026f2

git apply --check 0001-harden-draft-semantics.patch
git apply 0001-harden-draft-semantics.patch

git apply --check 0002-semantic-contracts-and-profile-composition.patch
git apply 0002-semantic-contracts-and-profile-composition.patch

git apply --check 0003-translator-trust-and-validation.patch
git apply 0003-translator-trust-and-validation.patch
~~~

A maintainer can also review or accept only patch 1.

The series does not depend on upstream accepting the two new proposal documents.

## Review order

For the shortest path through the Fleet-Control response:

1. `../SIGNET-2-RESPONSE-HANDOFF.md`
2. `../SIGNET-2-PROPOSAL-DELTA-MATRIX.md`
3. this patch series
4. `../RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md`

## Evidence boundaries

The patch series preserves these distinctions:

~~~text
SHARED_LABELS != SHARED_SEMANTICS

PINNED_CHOICE != CORRECT_CHOICE

SERVER_AUTHORITY != CHEAT_IMPOSSIBILITY

DECLARED_PERMISSION != ENFORCED_PERMISSION

VALID_SIGNATURE != CURRENT_AUTHORIZATION

PACKAGE_BYTES != EFFECTIVE_PROFILE

MODEL_SUGGESTION != AUTHORITY

LOCAL_APPROVAL != DATASET_CONTRIBUTION

SELF_CONFORMANCE
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= ECOSYSTEM_ADOPTION
~~~

These are not rhetorical disclaimers. Each distinction corresponds to a different protocol rule, authority boundary or evidence requirement.

## Supporting research snapshot

Exact lane heads used by the current synthesis:

~~~text
L1  b8f47a332fcb0aa641cfe676e22b49a801731628
L2  a5a677f7bf3e80d2face8f2185b27fcb8e394ab3
L3  821d13fe33f293281c85dc9da3a32e2a3fb47148
L4  22cdaf37b08d940a518c4a6a9dcea51553cd7256
~~~

Upstream Signet remained at `2ddb136ee941705d3e1c020eaddad93be65026f2` when the applicability check was performed.
