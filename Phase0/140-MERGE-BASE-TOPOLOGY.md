# Deterministic Merge-Base Topology

This contract defines the read-only evidence needed before a Git integration planner may justify a two-head candidate when one or several best common ancestors can exist.

## Core invariant

Target and source commit IDs are not a complete merge-base basis. Planning identity also binds the effective history view actually traversed, the complete unordered best-base set, and any deterministic virtual-base computation used to consolidate several best bases.

If those facts cannot be proven completely enough for the claimed semantics, planning fails closed before candidate construction. This contract never grants ref-mutation authority.

## Effective history view

Bind a versioned canonical history-view record that includes at least:

- history completeness disposition;
- exact shallow-boundary identities when present;
- active replacement ancestry mappings;
- graft-equivalent ancestry state when the selected Git implementation supports it;
- Git implementation/version and repository object format;
- material repository attributes that affect the selected merge computation.

An installation may instead forbid ancestry-altering local state, but the prohibition must be mechanically checked. Unknown or incomplete history is `UNPROVABLE`; visible ancestry must not be promoted to global completeness merely because a command returned successfully.

Deepening or unshallowing, adding/removing/repointing replacement ancestry, changing graft-equivalent state, or changing an incompatible Git/object-graph basis invalidates prior history-view evidence even when target and source object IDs do not move.

## Complete best-base discovery

Discover the complete best common ancestor set under the bound history view. For Git CLI implementations this means using semantics equivalent to `git merge-base --all`, not treating the first result of a single-result command as canonical.

Canonicalize the semantic set independently of discovery order and bind:

- exact target and source identities;
- canonical history-view identity;
- complete canonical best-base set and its identity;
- discovery algorithm and version;
- repository object format;
- `NONE`, `UNIQUE`, `MULTIPLE_VIRTUAL`, `UNSUPPORTED`, `UNPROVABLE`, or equivalent disposition.

Missing, malformed, duplicated, or partially enumerated base identities are non-success rather than guessed topology.

## Unique and multiple best bases

Exactly one best base may be carried forward as a unique-base planning result while mutation authority remains false.

Several best bases must never silently collapse to an arbitrary member. The planner must either:

1. return `UNSUPPORTED`/`UNPROVABLE` before candidate construction; or
2. execute a separately defined deterministic virtual-base/consolidation algorithm and bind its algorithm name, version, options, and all material intermediate identities.

Equivalent reordered discovery of the same best-base set under the same history view and virtual-base basis must produce the same semantic plan identity. A material change to target/source, history view, base set, algorithm/version/options, or intermediate identities must change that identity unless an explicit compatibility rule proves equivalence.

The identity used here is evidence identity, not permission. A caller cannot acquire write authority by supplying a desired base, desired final tree, or desired identity.

## Executable and repository-controlled behavior

Repository-controlled executable merge drivers, hooks, or comparable code are outside this bounded topology model unless separately authorized and isolated. Their presence on a material path yields `UNPROVABLE`/`UNSUPPORTED` rather than executing untrusted repository code as part of a correctness proof.

A provider-valid command result is still only evidence from the exact bound execution basis. Historical results are not silently reinterpreted after Git/version/history-view drift.

## Planning versus mutation

Merge-base discovery and virtual-base computation are read-only. Every modeled result carries `mutation_authority=false`.

A later integration step must independently revalidate current target/source heads, repository policy, candidate identity, applicable checks, exact history-view compatibility when material, and current mutation authority at the authoritative write boundary.

## Deterministic acceptance

The executable harness must prove at least:

- a real Git DAG with one best common ancestor returns that exact unique base;
- a real criss-cross Git DAG exposes exactly two best bases under complete discovery;
- reversing equivalent best-base enumeration preserves semantic plan identity;
- an unsupported multi-base topology fails before candidate construction;
- a supported virtual-base path binds algorithm/version/options and material intermediate identities;
- reversing equivalent intermediate/base enumeration preserves virtual plan identity;
- shallow/completeness uncertainty yields `UNPROVABLE`;
- a bound shallow boundary, replacement map, or graft-equivalent state changes history-view/plan identity;
- target/source movement invalidates plan identity;
- incompatible Git/version or virtual-base version movement invalidates plan identity;
- no-common-base is explicit non-success;
- repository-controlled executable merge behavior fails closed; and
- no read-only topology result grants mutation authority.

No correctness case depends on sleeps, uncontrolled live races, or fixture expected labels being supplied to the reducer.

## Migration

Evidence produced before this contract that assumed a single merge base without proving uniqueness and a compatible effective history view remains historical only. Recompute before using it to justify a current candidate where shallow history, ancestry replacement, multiple best bases, or incompatible algorithm/version movement may matter.

Future changes to canonicalization or virtual-base semantics require a new identity version or explicit compatibility transition; old evidence is never silently upgraded.
