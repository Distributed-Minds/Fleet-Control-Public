# Phase0 Deterministic Integration Candidates

This file defines the reusable contract for predicting Git integration candidates without granting mutation authority.

## 1. Prediction is evidence, not authority

A planner MAY compute a candidate tree, candidate commit object, and deterministic candidate identity. Prediction MUST NOT move refs, create authoritative integration state, or imply that a later integration action is still valid.

Immediately before any later commit/ref mutation, the executing lane MUST re-read the exact current target and source heads, repository policy relevant to the declared operation kind, and constructor compatibility. Any unproved drift makes the prediction stale and unusable for mutation.

## 2. Candidate semantic envelope

A candidate identity binds one immutable semantic envelope containing at least:

- `schema_version`;
- `operation_kind`;
- exact `target_commit`;
- exact `source_commit`;
- exact ordered `parents` vector;
- `parent_count`;
- exact `tree` object identity;
- normalized commit-object inputs that affect object identity, including author/committer identities, timestamps/time zones, message bytes, encoding when material, and optional signature-header policy;
- `constructor_version`;
- `compatibility_basis` describing the explicit compatibility rule used for that constructor version.

The canonical candidate identity is a digest over a deterministic serialization of the complete semantic envelope. Implementations MUST NOT omit fields merely because two current constructors happen to produce the same commit object.

## 3. Ordered parent/cardinality semantics

Parent order and parent count are semantic.

For a normal explicit two-parent merge candidate:

`parents = [target_commit, source_commit]`

and `parent_count = 2`.

Reversing the vector produces a different candidate. Changing one-parent, two-parent, or three-plus-parent cardinality produces a different semantic operation even when the tree and message are identical.

A constructor that supports only a subset of cardinalities MUST return an explicit unsupported disposition for the rest. It MUST NOT truncate, reorder, or silently substitute another operation kind.

## 4. Operation-kind preservation

Repository policy may allow multiple ways to integrate the same tree, such as fast-forward or explicit merge commit. The declared `operation_kind` is part of candidate semantics.

A planner that predicts an explicit merge commit MUST NOT silently replace it with fast-forward merely because fast-forward is currently possible. A later executor may choose another operation only by discarding the old candidate and creating a new candidate under the new declared operation kind and current evidence.

## 5. Constructor compatibility and migration

`constructor_version` changes fail closed unless `compatibility_basis` names a deterministic compatibility rule proving that all candidate-identity semantics are preserved for the relevant operation/cardinality.

Compatibility MUST be explicit and testable. Sharing a helper name, producing the same tree, or accepting the same input shape is insufficient.

When a migration is declared compatible, fixtures MUST prove at least:

- the same semantic envelope remains the same candidate identity where intended;
- reordered parents do not alias;
- changed parent count does not alias;
- incompatible normalized metadata inputs do not alias;
- unsupported operation/cardinality combinations stay unsupported rather than being coerced.

## 6. Staleness and execution revalidation

Before mutation, execution MUST prove all of the following against live repository state:

1. `target_commit` still equals the intended target head;
2. `source_commit` still equals the intended source head;
3. ordered parent/cardinality semantics still match the declared operation;
4. repository policy still permits the declared operation kind;
5. the constructor version remains compatible under the recorded compatibility basis.

Failure of any item yields `STALE_OR_INCOMPATIBLE`. The executor MUST NOT silently reuse or reinterpret the candidate.

## 7. Deterministic serialization

A portable implementation SHOULD use a schema-versioned canonical JSON object with UTF-8 encoding, lexicographically sorted object keys, no insignificant whitespace, and arrays preserved in declared order. Hash the resulting bytes with SHA-256 unless a repository explicitly selects another versioned digest algorithm.

The serialization algorithm itself is part of `schema_version`. Changing canonicalization rules requires a new schema version and an explicit migration/compatibility decision.

## 8. Required fixture classes

A conforming implementation MUST deterministically cover:

- target/source parent reversal with identical tree/message -> distinct candidate identities;
- identical tree/message with different parent cardinality -> distinct or unsupported disposition as declared;
- target head movement after prediction -> stale candidate rejected;
- source head movement after prediction -> stale candidate rejected;
- fast-forward-capable history when `operation_kind=explicit-merge` -> operation kind preserved;
- three-plus-parent input reaching a two-parent-only constructor -> unsupported, never truncated;
- incompatible constructor-version change -> fail closed;
- explicitly compatible constructor migration -> stable identity semantics under the declared compatibility rule.

## 9. Safety and portability

The contract is repository-agnostic. It must not depend on private issue numbers, private branch names, inaccessible control data, or project-specific topology. The root MIT license and starter-package inclusion remain unchanged by adopting this contract.
