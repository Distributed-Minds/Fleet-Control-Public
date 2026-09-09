# Phase0 Deterministic Merge-Base Topology

This contract extends `80-INTEGRATION-CANDIDATES.md` for repositories where two heads do not have one trivially provable merge base.

## Evidence basis

A merge-base observation is usable only when it binds an exact, versioned history-view basis:

- exact target and source commit IDs;
- repository/object format and selected Git implementation/version;
- history-completeness disposition;
- shallow-boundary identity when present;
- replace-ref / ancestry-substitution identity when supported;
- graft-equivalent local ancestry state when applicable;
- the complete unordered semantic set of best common ancestors.

If completeness or ancestry substitution cannot be proved under the selected implementation, the disposition is `UNPROVABLE`. A visible partial graph must never be promoted to globally complete evidence.

## Complete best-base discovery

Discovery must enumerate every best common ancestor. Enumeration order is presentation only. Canonicalize the set by sorting unique object IDs as bytewise UTF-8 strings before hashing or comparison.

Dispositions are:

- `UNIQUE` — exactly one best base;
- `MULTIPLE` — more than one best base;
- `NONE` — no common ancestor under the bound history view;
- `UNPROVABLE` — required history-view evidence is incomplete or unsupported;
- `ERROR` — malformed/unavailable input or deterministic discovery failure.

A caller must not replace `MULTIPLE` with an arbitrary first base.

## Virtual-base semantics

For `MULTIPLE`, an implementation may proceed only when it declares a versioned deterministic virtual-base algorithm. The algorithm identity binds:

- algorithm name/version;
- exact options and compatibility basis;
- canonical best-base set;
- every material intermediate base/tree/commit identity in deterministic order;
- final virtual-base identity.

Equivalent enumeration orders of the same semantic best-base set must produce the same computation identity and result. If the implementation cannot reproduce the declared computation, return `UNSUPPORTED` or `UNPROVABLE` before candidate construction.

## Staleness

Any change in target/source identity, effective history view, completeness, shallow boundaries, ancestry substitutions, best-base set, Git/object implementation compatibility, or virtual-base algorithm/options invalidates prior topology evidence unless an explicit compatibility rule proves semantic equivalence.

Deepening/unshallowing and replace-ref add/remove/repoint are stale-evidence events even when target/source object IDs do not move.

## Mutation boundary

Topology discovery and virtual-base computation are read-only planning evidence. They grant no mutation authority. Before authoritative candidate publication or ref movement, the executor must revalidate live heads, relevant policy, topology/history-view compatibility, candidate identity, and current mutation authority.

## Deterministic model

`Phase0/scripts/check-merge-base-topology-fixtures.py` evaluates the public fixture corpus in `Phase0/fixtures/merge-base-topology-spec2.json`.

The model deliberately does not invoke repository-local hooks, merge drivers, or other executable repository configuration. Such behavior is unsupported until separately authorized and modeled.

## Required invariants

A conforming implementation must prove at least:

1. one base under a complete supported view -> `UNIQUE`;
2. two or more best bases -> `MULTIPLE`, never arbitrary first-base success;
3. reversed discovery order -> same canonical set identity;
4. incomplete/shallow history without an exact supported bound view -> `UNPROVABLE`;
5. same heads with changed history-view identity -> stale evidence;
6. best-base movement -> stale evidence;
7. multi-base without declared virtual-base support -> `UNSUPPORTED` before candidate construction;
8. declared virtual-base algorithm/version/options drift -> stale/incompatible;
9. equivalent reordered multi-base inputs under the same declared algorithm -> identical computation identity;
10. differing material intermediate identities -> distinct computation identity even if a final tree happens to match.

## Portability and licensing

This contract uses no private repository identity or private control evidence. The root MIT `LICENSE` remains authoritative and must remain included in generated/downloadable starter packages.