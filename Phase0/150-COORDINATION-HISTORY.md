# Coordination History Archival

This contract bounds an append-only coordination surface without weakening the evidence required to reconstruct authoritative current state.

## Safety invariant

A destructive compaction step is eligible only when all required predicates are independently established for the exact source record and exact effect boundary:

- exact archive bytes and manifest hash are remotely verified;
- reducer-relevant order is authoritative under a current `RecordOrderingBasis`;
- the archive manifest is selected by one rollback-resistant `ManifestLineageBasis`;
- archive plus live observations form one coherent `CoordinationHistorySnapshotBasis`;
- the live source identity/incarnation and content version still match the archived evidence;
- current bounded mutation authority covers the exact operation and target;
- the preservation objective is selected by a uniquely current `ReconstructionHorizonAuthorityBasis`;
- required archive material remains reconstructable through that objective under a current `ArchiveDurabilityBasis`; and
- latest persistent state, active ownership chains, configured live tail, and still-authoritative references remain protected.

Missing, ambiguous, inaccessible, forked, stale, or incompatible proof in any required dimension fails closed. A hash proves content identity, not order, authority, currentness, retention, or key availability.

## RecordOrderingBasis

Reducer-relevant records bind provider ordering position or causal/predecessor evidence, source incarnation, ordering-policy generation, pagination/frontier completeness, and compatibility semantics. Opaque IDs, lexical ordering, retrieval order, timestamps, and archive serialization order are not substitutes for authoritative order.

Movement of the ordering basis invalidates stale replay/deletion evidence unless explicit compatibility preserves the same required semantic order.

## ManifestLineageBasis

Each authoritative manifest binds archive namespace and destination incarnation, exact predecessor manifest, exact segment set and hashes, generation/schema identity, transition operation identity, and a current-selection fence or mechanically equivalent proof.

Competing successors, rollback, restore, clone, stale rejoin, or ambiguous current selection fail closed. Lost acknowledgement reconciles the exact intended successor before retry and never blindly forks manifest history.

## CoordinationHistorySnapshotBasis

Authoritative replay across archive plus mutable live history binds the exact current manifest, live-surface identity/incarnation, live frontier, cutover generation, ordering basis, archive/live continuity relation, stable record identity, and closing revalidation or an atomic snapshot identity.

The assembled history must represent one jointly valid cut. Archive-first overlap is deduplicated only by stable semantic identity under that exact snapshot basis. Old-manifest/post-delete omission and new-manifest/pre-delete duplication are rejected rather than normalized optimistically.

## ReconstructionHorizonAuthorityBasis

The reconstruction horizon is authority-bearing state, never caller/deleter input. It binds an authoritative policy/config source and incarnation, exact objective generation/horizon, predecessor/current-selection lineage, transition authority and allowed transition classes, current-selection fencing, and downgrade/retirement semantics.

A shorter successor objective requires an explicit authorized transition. It does not retroactively expose records protected by the predecessor objective unless exact retirement/migration eligibility proves that bounded range is safe. Rollback, fork, restore, clone, stale rejoin, conflicting current-looking sources, or unknown current selection fail closed.

## ArchiveDurabilityBasis

Durability binds the exact current reconstruction-horizon basis plus archive destination/incarnation, segment/manifest identity, retention/lifecycle policy generation and effective rules, immutability protections relied upon, storage class/provider generation where material, encryption/key-custody generation and recovery availability, movement transition identity, and effect-time currentness evidence.

A successful readback proves bytes exist at that observation only. If the authorized horizon exceeds provable retention, storage accessibility, or key availability, destructive compaction is ineligible.

## Effect-boundary protocol

Before each archive publication, manifest transition, source deletion, or recovery effect, bind one immutable operation/plan identity and prove the exact required pre-state plus current bounded mutation authority. Either revalidate every movement-sensitive basis immediately before the effect or execute under a trusted serialization primitive that prevents those predicates from moving between proof and effect.

Retries distinguish exact prior success, partial effects, changed state, and stale bases. Superseded actors are reconciliation-only unless an explicit bounded recovery transfer is current.

## Migration and rollback

Existing live-only installations remain valid. Migration is additive: establish authoritative ordering, manifest lineage/current selection, reconstruction-horizon authority, durability, and mutation-authority adapters; publish and verify immutable archive artifacts; then delete only exact records satisfying every current predicate.

Older archive artifacts remain historical evidence for the semantics and bytes they actually recorded. Missing authority/order/lineage/snapshot/horizon/durability identity is never backfilled from current credentials, timestamps, generation numbers, hashes, object presence, or past success.

Rollback disables further destructive compaction and consequential current-state reliance while preserving archive, manifest, and live evidence for reconciliation. It never rewrites historical records or silently reinstates a stale manifest or horizon generation as current.

## Deterministic verification

`fixtures/coordination-history-spec5.json` defines positive and negative controls for ordering, manifest forks/rollback, mixed archive/live cuts, source movement, stale authority, durability/key movement, horizon downgrade/rollback, partial effects, and lost acknowledgement. Implementations may use provider-native atomic primitives or explicit portable bases/fences, but must preserve the same fail-closed outcomes.
