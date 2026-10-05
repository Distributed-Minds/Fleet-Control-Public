# L1 Continuation 03 — Standards Delta, Contradictions, and Lane Handoffs

**Lane:** L1 — Protocol & Semantic Interoperability  
**Pass:** continuation 1  
**Research cutoff:** 2026-10-05

This file records which external patterns materially changed L1, which searches produced no new architectural delta, and which findings belong to other lanes.

# 1. Standards comparison

## IEEE HLA 1516 family

### Evidence

IEEE 1516-2025 is the active HLA framework/rules standard. IEEE 1516.1-2025 defines services/interfaces for coordinated exchange through the RTI. IEEE 1516.2-2025 defines the format and syntax of HLA object models, explicitly **not their content**.

SISO publishes public HLA-related data files, including modular FOM examples and reference enumerations.

### Material L1 lesson

HLA demonstrates that:

- distributed participants need an agreed federation contract;
- publication/subscription and ownership/time are first-class concerns;
- object models can be modular;
- a standard object-model syntax does not automatically supply common domain meaning.

### Design delta

Use HLA as a source of **coordination patterns**, not as a requirement to adopt RTI complexity.

The closest Signet analogue to a modular FOM is a set of independently versioned semantic profiles composed into a session contract.

## OpenXR 1.1

### Evidence

OpenXR exposes optional extensions. Applications query which extensions are available, then explicitly enable a subset when creating the instance. Extension names use controlled author/vendor namespaces.

### Material L1 lesson

`supported` and `active` are separate protocol states.

### Design delta

A Signet adapter should not be considered to be “using” every profile it knows. Session negotiation activates a selected subset.

This also argues for namespaced semantic identifiers rather than short words such as `damage` or `move`.

## glTF 2.0

### Evidence

glTF declares both `extensionsUsed` and `extensionsRequired`, with required extensions a subset of used extensions. The specification also requires minor-version evolution not to change existing behavior.

### Material L1 lesson

There are at least three distinct compatibility states:

1. understood/supported;
2. used/activated;
3. required for correct interpretation.

### Design delta

The capability model should preserve those distinctions rather than collapsing them into one boolean.

“Unknown fields are ignored” is safe only when unknown additions are genuinely optional and do not redefine existing behavior.

## LCIM / conceptual interoperability literature

### Evidence

Tolk and Muguira's Levels of Conceptual Interoperability Model separates technical, syntactic, semantic and pragmatic levels. Later work revisits conceptual alignment and stresses that meaningful interoperability requires more than transport/schema agreement.

### Material L1 lesson

A participant can share meaning but still be unable to perform the required behavior.

### Design delta

`OBSERVE_ONLY` is not merely an error state. It can represent semantic interoperability without pragmatic ability to participate as an actor.

## RFC 8141 / IANA URN namespace registry

### Evidence

RFC 8141 states that a syntactically plausible `urn:<nid>:...` string is not a valid URN unless the namespace identifier is registered and names are assigned according to that namespace. The current IANA registry has no `signet` NID.

### Material L1 lesson

The earlier `urn:signet:...` examples are architecture placeholders, not deployable global identifiers.

### Design delta

For the draft, use URI identifiers under a domain controlled by the protocol project, e.g. `https://signetprotocol.io/sem/...`. If the project later wants a formal URN namespace, register it deliberately rather than assuming it exists.

# 2. Current Signet/1 semantic gaps

These are not criticisms of the beta transport. They identify exactly where a larger independent ecosystem needs additional contract surface.

## G1 — Scalar version is not capability negotiation

`Hola.version` and `Bienvenida.version` can detect a number difference but cannot establish which semantic features are jointly usable.

## G2 — Game ID is overloaded

`juego` tells the server which game a translator represents. It does not identify:

- adapter implementation;
- adapter version;
- profile definitions;
- supported semantic roles;
- local limitations;
- safe fallbacks.

## G3 — FPS vocabulary is embedded in the common language

`Intencion` contains move/strafe/yaw/run/fire/use/weapon.

That is a useful beta vocabulary but not a universal cross-game core.

## G4 — Optional defaults can hide semantic absence

A missing field and a field explicitly present with a value can be operationally different even if the parser supplies a default.

Future profiles must define absence semantics rather than assume parser defaults prove compatibility.

## G5 — Legacy causality is implicit

`seq=0` and numbered commands represent different command-application semantics.

The next contract should advertise which model is active rather than infer it after play begins.

## G6 — `Posicion` is a different semantic path

Sending a desired position bypasses ordinary intent/rules semantics.

It should be represented as an explicit fallback/legacy capability rather than as equivalent movement intent.

## G7 — Mode strings identify names, not definitions

`doom:deathmatch` is useful as a label, but independent implementations need a stable ruleset definition identity and equivalence check.

# 3. Source delta

## Newly accepted sources for L1 continuation

### OpenXR extension model

- Khronos OpenXR 1.1.63 specification:
  - https://registry.khronos.org/OpenXR/specs/1.1/html/xrspec.html
- OpenXR extension process:
  - https://registry.khronos.org/OpenXR/specs/1.1/extprocess.html

Accepted because they provide primary current evidence for query-then-enable capability semantics and controlled extension namespaces.

### glTF extension/version model

- glTF 2.0 specification:
  - https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html

Accepted because it provides a simple mature distinction between extensions used and extensions required, plus behavioral compatibility rules for minor evolution.

### IEEE HLA current standards

- IEEE 1516-2025:
  - https://standards.ieee.org/ieee/1516/6687/
- IEEE 1516.1-2025:
  - https://standards.ieee.org/ieee/1516.1/6688/
- IEEE 1516.2-2025:
  - https://standards.ieee.org/ieee/1516.2/6689/

Accepted as current primary standards metadata. The public summaries are sufficient for claims made here; this pass does not claim access to paywalled normative text beyond the surfaced metadata/abstract.

### SISO public HLA data files

- https://www.sisostandards.org/page/DataFiles

Accepted because the public SISO page exposes concrete modular FOM examples/reference data and confirms the modular object-model ecosystem in practice.

### LCIM / conceptual alignment

- Tolk & Muguira, “The Levels of Conceptual Interoperability Model”:
  - https://www.researchgate.net/publication/240319008_The_Levels_of_Conceptual_Interoperability_Model
- Tolk, “Conceptual alignment for simulation interoperability: lessons learned from 30 years of interoperability research”:
  - https://journals.sagepub.com/doi/full/10.1177/00375497231216471

Accepted for the semantic/pragmatic distinction and current retrospective framing.

### URI/URN identity

- IANA URN namespace registry:
  - https://www.iana.org/assignments/urn-namespaces
- RFC 8141:
  - https://www.rfc-editor.org/rfc/rfc8141.html
- RFC 3986:
  - https://www.rfc-editor.org/rfc/rfc3986.html

Accepted because the identifier recommendation depends on exact namespace mechanics.

# 4. Native-language recursion record

## Spanish — material reinforcement, no new architecture branch

The already-ledgered SCFHLA paper remains unusually relevant. It states that HLA/FOM-level agreement supplies a structural/syntactic contract while semantic agreement still has to be established among participants, and it uses a network of ontologies to keep domain knowledge modular.

Material effect:

- reinforces modular profiles rather than one monolithic global ontology;
- reinforces explicit mapping relations;
- supports treating a common schema as necessary but insufficient.

No duplicate source entry was added as “new” merely because this pass reread it.

## Chinese/Japanese — no accepted new delta this pass

Searches around semantic interoperability + HLA/ontology mostly rediscovered the already-ledgered adapter/distributed-simulation literature or secondary mirrors.

Disposition: **NO MATERIAL DELTA**.

Next recursion terms should target:

- capability negotiation in distributed simulation;
- modular object-model conflict/merge rules;
- semantic version/equivalence of federation models;
- pragmatic interoperability in interactive simulation.

# 5. Cross-lane handoffs

## L1 -> L2

~~~yaml
handoff:
  from_lane: L1
  to_lane: L2
  finding: The contract now separates semantic IDs/profiles from adapter implementation and defines a durable mapping-provenance record.
  evidence:
    - lanes/L1/01-Semantic-Contract-Draft.md
    - lanes/L1/02-Negotiation-Mapping-and-Conformance.md
  why_material: Adapter/code-generation tooling can target a stable contract rather than invent vocabulary.
  requested_followup: Prototype profile-to-SDK/mapping scaffolding and report any contract field that cannot be generated or preserved cleanly.
~~~

## L1 -> L3

~~~yaml
handoff:
  from_lane: L1
  to_lane: L3
  finding: Authority semantics require participant/adapter identifiers and authority epochs, but L1 deliberately leaves authentication/signatures outside scope.
  evidence:
    - L1 authority table and transfer model
  why_material: L3 must make the claimed issuer trustworthy without redefining L1 authority meaning.
  requested_followup: Bind package/session identity, revocation and signature policy to L1 identifiers while keeping semantic authority separate from cryptographic identity.
~~~

## L1 -> L4

~~~yaml
handoff:
  from_lane: L1
  to_lane: L4
  finding: Exact conformance properties now exist for semantic identity, negotiation, mappings, authority, causality, profile modularity and the Signet/1 bridge.
  evidence:
    - lanes/L1/02-Negotiation-Mapping-and-Conformance.md
  why_material: L4 can turn interoperability claims into independent reproducible fixtures.
  requested_followup: Implement the properties as cross-implementation fixtures and return any contradiction or untestable property to L1.
~~~

# 6. L1 stop rule for this pass

Do not expand into:

- launcher/package signing;
- sandboxing;
- legal/EULA analysis;
- model benchmarking;
- SDK ergonomics;
- compatibility-registry product design.

Those belong to L2-L4.

L1 should continue only where new evidence changes:

- semantic identity;
- profile composition;
- capability negotiation;
- authority/time/causality semantics;
- mapping provenance;
- deterministic incompatibility behavior.
