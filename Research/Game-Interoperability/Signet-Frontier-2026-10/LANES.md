# Continuation Research Lanes

This package is split into four parallel research lanes. The split is by **research question and evidence surface**, not by file ownership in the underlying projects.

All four lanes use the same evidence labels:

- **OBSERVED**
- **DERIVED**
- **PROPOSED**
- **CLAIMED**
- **UNKNOWN**

All four lanes should continue native-language discovery rather than translating English queries only.

## Lane map

| Lane | Primary question | Owns | Does not own |
|---|---|---|---|
| **L1 — Protocol & Semantics** | What must all participants mean identically? | wire semantics, profiles, capability negotiation, federation/time/authority models, ontology structure, standards comparison | launcher security, AI model evaluation, package distribution |
| **L2 — Adapters & Adaptive Translation** | How do we connect real games efficiently without making AI authoritative? | translator SDKs, code generation, decision models, replay-assisted mapping, engine/plugin integration, adapter DX | protocol governance, legal policy, package trust |
| **L3 — Trust, Distribution & Governance** | How can strangers safely install, run, discover, and govern adapters? | launcher/package security, sandboxing, signatures, privacy, legal/EULA boundaries, registry governance, project governance | gameplay semantics, model architecture benchmarking |
| **L4 — Validation, Ecosystem & Adoption** | How do we prove interoperability works and grow it without hand-waving? | independent implementations, conformance methodology, compatibility evidence, benchmarks, adoption paths, ecosystem mapping, failed approaches | defining core semantics except through findings handed to L1 |

## Shared non-collision rule

A lane may **discover** something relevant to another lane but should not fully develop the other lane's design.

Use this handoff shape:

```yaml
handoff:
  from_lane: Lx
  to_lane: Ly
  finding: ...
  evidence:
    - ...
  why_material: ...
  requested_followup: ...
```

## Shared output contract

Each continuation pass should end with:

1. **new sources** — especially primary/native-language sources;
2. **new claims** — what changed from the current package;
3. **contradictions** — evidence against current assumptions;
4. **design delta** — concrete consequence, if any;
5. **no-delta evidence** — when a search branch repeats known material;
6. **next recursion** — exact vocabulary/source/citation frontier;
7. **handoffs** — only when another lane owns the follow-up.

## Shared stop rule

Do not expand a lane just because adjacent material is interesting. Stop or hand off when the next useful question belongs to another lane.

## Current synthesis target

The four lanes should converge on one testable statement:

> A cross-game interoperability stack is credible when independently built adapters can negotiate explicit capabilities, agree on shared semantics, pass reproducible conformance/replay tests, and run through a distribution surface whose trust claims are specific and auditable.

## Recommended parallel order

All four lanes can start immediately.

- L1 should stabilize vocabulary and semantic-profile options.
- L2 should explore implementation/tooling options against those candidate contracts.
- L3 should define what a safe public adapter/package ecosystem must prove.
- L4 should turn all three into independent experiments and measurable adoption evidence.

No lane needs to wait for another to finish; unresolved dependencies should be recorded as assumptions and handed off.
