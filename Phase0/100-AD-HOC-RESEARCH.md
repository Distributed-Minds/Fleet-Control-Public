# Ad-hoc research and additive evidence handoff

Status: public Phase0 protocol package  
Implements: public issue #24, Phase0 spec version 1

## Purpose

A temporary research worker may discover and contribute evidence without becoming a persistent fleet identity or inheriting standing mutation authority.

Durable invariants:

1. repository authority is resolved before non-default evidence is interpreted;
2. pure research is read-only with respect to existing shared surfaces;
3. temporary workers never claim persistent `AGENT_STATE` or ownership identity merely because they can write to the same transport;
4. one semantic completed research result has one stable packet identity and one reconciled publication lineage;
5. packet publication is a separately authorized additive side effect, not an implicit consequence of research completion; and
6. packet storage makes evidence durable but never turns that evidence into policy or implementation authority.

## Authority-first source discovery

Resolve the configured default branch and exact head first. Branch names, timestamps, PR state, or apparently newer prose are discovery hints rather than authority.

When non-default material may matter, bind exact ref/head and compare/ancestry evidence. Consume only the unique delta relative to current authority. If uniqueness, completeness, or supersession cannot be established, preserve `UNKNOWN` rather than guessing.

## Temporary-worker boundary

By default an ad-hoc worker may read and reason but may not mutate existing issues, PRs, branches, files, coordination state, persistent-agent state, authority policy, deployment state, or other shared canonical surfaces.

It does not impersonate a persistent identity and does not fabricate an ownership lease. Pure research creates no Git branch or PR merely to preserve prose.

A human may grant an exceptional narrow mutation separately. That grant must identify the exact target and expected old state, recheck overlapping ownership, stop on unresolved collision, perform only the authorized mutation, and verify the remote postcondition. Packet-publication authority never implies this existing-surface authority.

## Completed research packet

The normal durable output is an additive packet with a machine-detectable marker equivalent to:

`AD_HOC_RESEARCH_PACKET | status=COMPLETE | schema=1`

A packet binds at least:

- topic/question;
- exact authoritative baseline identity;
- exact unique non-default source identities actually consumed;
- material external-source identities;
- observations, derived conclusions, predictions, contradictions, stale-source warnings, and unknowns without flattening their evidence class;
- important discovery vocabulary where it opened new evidence paths;
- likely affected canonical packages and proposed deltas;
- unresolved questions and useful next actions;
- schema identity and immutable content identity; and
- stable semantic packet identity derived from canonical immutable packet inputs.

These handoff fields are durable evidence, not optional presentation metadata. If a source is known stale or superseded, the warning survives packet publication. Discovery terms that materially enabled retrieval survive when they are needed to reproduce or extend the evidence search. Useful next actions remain distinguishable from accepted policy or implementation instructions.

Material content, source-basis, topic, or semantic-conclusion change creates a new packet identity. An unchanged retry reuses the same identity.

## Publication authority

Publication is a bounded additive side effect. Current authority is established outside packet-authored content and binds at least:

- exact destination and packet namespace;
- temporary execution/principal identity available to the transport;
- semantic packet identity and immutable content identity;
- action class limited to additive packet creation or exact retry reconciliation;
- current destination permission/authority basis;
- schema/protocol compatibility;
- expiry/task boundary; and
- stable publication operation identity.

A packet marker, title, locator, body field, or self-declared worker name cannot mint authority.

## Deterministic create/reconcile rule

A supported publication adapter provides one of:

1. atomic create-if-absent keyed by semantic packet identity;
2. a unique-key/CAS/serialized packet namespace; or
3. authoritative reconciliation strong enough that concurrent and retried attempts converge on one canonical semantic lineage.

Provider-assigned locators are allowed, but retry safety still requires the stable semantic identity to be discoverable through trusted metadata or another authoritative index.

Before any retry create after cutoff or acknowledgement loss, reconcile the publication identity and distinguish at least:

- `ABSENT`;
- `ONE_COMPLETE`;
- `ONE_INCOMPLETE`;
- `MULTIPLE_EQUIVALENT`;
- `LOCATOR_CONFLICT`;
- `AUTHORITY_STALE`; and
- `INVENTORY_UNKNOWN`.

Only `ABSENT` under complete authoritative inventory plus current compatible publication authority may authorize a fresh create. `ONE_COMPLETE` adopts the existing canonical packet. Ambiguous, incomplete, stale, incompatible, or conflict state performs no blind create.

Concurrent equivalent researchers may compute the same packet identity. Separate execution identities remain provenance, but duplicate publication attempts do not become independent evidentiary corroboration.

## Canonicalization boundary

A completed packet is evidence input. Persistent workers independently evaluate and canonicalize supported findings through their ordinary planning, evidence, ownership, and acceptance rules.

Storage location does not grant authority. Derivative packet prose is not a new independent source lineage merely because another worker quotes or republishes it. Duplicate artifacts sharing one semantic packet identity count as one evidence lineage unless independent observations are separately proven.

## Cutoff, expiry, and recovery

After publication authority expires or is revoked, the temporary worker may perform bounded read-only reconciliation if permitted but cannot issue a fresh packet mutation under stale authority.

Acknowledgement loss never upgrades uncertainty into permission. Incomplete destination enumeration never proves absence. Schema/version movement requires explicit compatibility or a new publication basis.

## Deterministic verification

`fixtures/ad-hoc-research-spec1.json` and `scripts/check-ad-hoc-research-fixtures.py` model the public safety boundary without wall-clock or network dependence. Fixtures cover:

- authoritative baseline before non-default evidence;
- temporary identity unable to claim persistent state;
- concurrent equivalent publishers;
- acknowledgement loss and retry;
- incomplete inventory;
- provider-assigned locator conflicts;
- stale publication authority;
- schema incompatibility;
- duplicate-lineage canonicalization;
- packet-as-policy rejection;
- required stale-source warning preservation;
- required discovery-vocabulary preservation when it opened an evidence path;
- required useful-next-action preservation; and
- exceptional existing-surface mutation without fresh authority.

Run from repository root:

```bash
python3 Phase0/scripts/check-ad-hoc-research-fixtures.py
```

## Migration and rollback

Adopt additively. Historical ad-hoc research remains evidence for what it actually recorded. Legacy packet-like artifacts without stable semantic identity remain distinct unless exact content/source evidence mechanically proves equivalence; multiple locators do not prove independence.

Rollback disables fresh additive packet publication first while preserving existing packet evidence and read-only reconciliation. It never widens temporary workers into direct mutation authority.

## Package and license invariant

This contract, its fixtures, and checker live under `Phase0/`, which the starter release workflow copies recursively. The repository root MIT `LICENSE` remains authoritative and must remain included in generated starter packages.

## Done condition

An arbitrary installation can let a temporary researcher discover current and unique evidence, remain outside persistent-worker identity/state, and publish at most one canonical semantic result through a bounded, concurrency-safe, cutoff-safe, permission-safe, idempotent additive packet protocol whose output remains evidence rather than authority.
