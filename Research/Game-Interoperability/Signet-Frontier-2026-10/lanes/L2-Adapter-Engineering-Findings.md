# L2 — Adapter Engineering Findings

**Lane:** L2 — Adapter Engineering & Adaptive Translation  
**Research cutoff:** 2026-10-05  
**Status:** first implementation-oriented research pass  
**Baseline:** `b22daffcfed186be68cdd918b42c2732c4e3e5b9`

## Executive result

The strongest architecture found in this pass is:

> **generated contract glue + thin game-specific hook shim + replay/conformance evidence**

Adaptive models can assist with hook discovery, mapping proposals, shim completion, failure triage, and test prioritization. They should not silently become runtime authority over shared-world truth.

Separate three jobs:

1. **binding generation** — mechanical transformation of an explicit API/ABI description into Rust/C/C#/TypeScript/engine glue;
2. **semantic mapping** — deciding how a local event/capability maps into a shared semantic profile;
3. **runtime authority** — deciding actual shared-world truth.

The first is an excellent code-generation target. The second may use model assistance but must become explicit, versioned, testable data/code before release. The third stays deterministic.

## Evidence-driven claims

### C1 — Generated glue and handwritten hooks should be separate

**OBSERVED:** Signet already has Rust translator traits, a C FFI, a preview C# binding, and a TypeScript client/tooling surface. Its C docs explicitly position the C interface as an entry point for C/C++, Godot and Unreal.

**OBSERVED:** Godot's GDExtension tooling uses machine-readable API descriptions as generation inputs. Current Godot documentation calls `gdextension_interface.json` the source of truth for the low-level C API and says language bindings are intended to generate code from it.

**OBSERVED:** Japanese HLA work proposed plug-in adapters and support tooling that automatically generated additional source code from graphically defined exchange information. A 2024 HLA/MDA paper automates FOM and federate-code generation while still reporting that some federate behavior requires manual adjustment.

**DERIVED:** The reusable seam is not “generate the whole adapter”:

```text
shared semantic/profile contract
        |
machine-readable adapter ABI
        |
        +--> generated Rust glue
        +--> generated C ABI/header
        +--> generated C# P/Invoke
        +--> generated TypeScript/tool types
        +--> generated manifest/schema
        +--> generated fixture/conformance skeletons
        |
small handwritten game/engine hook shim
        |
explicit semantic mappings + tests
```

The shim owns engine callbacks, mod APIs, file formats, RCON/console calls, coordinate transforms, and game-version quirks.

### C2 — A C ABI is a useful lowest-common-denominator engine seam

**OBSERVED:** Signet currently ships a C ABI over its Rust core with non-blocking connection/state access, terrain, bodies/events, intent, prediction and reconciliation. Its C# binding is layered over that ABI.

**DERIVED:** Preserve the Rust implementation core while treating a versioned C ABI as the lowest-common-denominator runtime boundary for engines that can call native code. Generate higher-level bindings above it.

This does not make engine plugins identical. Godot GDExtension, Unreal modules/plugins, Unity native plugins, open-engine source integration, and server gateways still need thin target-specific shims.

### C3 — Syntax generation does not prove behavioral equivalence

**OBSERVED:** Signet's TypeScript client omits prediction because matching the Rust rules/floating-point behavior is a separate problem.

**OBSERVED:** Signet's C# binding is marked preview and untested in Unity/.NET.

**DERIVED:** Every generated binding needs compile/link fixtures plus behavioral replay fixtures for the capabilities it claims.

### C4 — Closed-set decision models fit L2 better than unconstrained runtime generation

**OBSERVED:** Laya exposes typed closed-set decisions and routing between checkpoints. Its own documentation records a failure mode where an English checkpoint can be highly confident on scripts it cannot read, motivating separate routing/domain gates.

**OBSERVED:** CLM exposes typed decisions and direct candidate ranking over a state/action scoring architecture.

**DERIVED:** Event mapping, fallback selection, failure triage, capability classification and test prioritization are good model tasks only after deterministic code bounds the domain and candidate set.

Preferred:

```text
domain/version gate
  -> permitted candidate set
  -> optional decision model
  -> deterministic schema/policy validation
  -> abstain or accept
  -> freeze accepted result
```

Not:

```text
raw game state -> model score -> authoritative shared-world action
```

### C5 — Compatibility claims need drift evidence, not only version strings

**OBSERVED:** Current translators depend on concrete integration surfaces: WAD/BSP/PK3 formats, RCON behavior, engine callbacks, C ABI behavior, and game-specific coordinate/event assumptions.

**DERIVED:** A game/adapter version string is necessary metadata but insufficient evidence that a prior mapping still applies. Compatibility should bind to fingerprints of the surfaces that matter plus replay/conformance evidence.

---

# L2.1 — Integration pattern matrix

The final column identifies a dependency only. Prescriptive EULA/licensing analysis belongs to L3.

| Mode | Typical hook | Shared-state authority | Latency | Update fragility | Asset access | Multiplayer limits | Maintainability | Policy dependency |
|---|---|---|---|---|---|---|---|---|
| **OPEN_ENGINE** | source hooks / linked SDK | neutral server/rules | lowest; in-process | medium; source/API movement is diffable | strong/local | engine netcode replacement may be invasive | high with narrow upstream seam | licence/build/distribution |
| **OFFICIAL_MOD** | supported plugin/mod API | neutral server; mod captures input/presents state | low–medium | medium | usually strong but API-bounded | anti-cheat/server policy can constrain | high when API stable | official mod/API terms |
| **OFFICIAL_API** | documented SDK/IPC/control API | API-specific; access != authority | medium | low–medium when versioned | bounded | intentionally limited in many games | highest if API covers semantics | API/auth/rate terms |
| **CONTROLLED_SERVER_GATEWAY** | RCON/console/server plugin/proxy | explicit bridge between authority domains | medium; process/network hop | medium–high | server-side/local | client presentation/input may remain coarse | good for coarse integration | server/mod/hosting |
| **LOCAL_REIMPLEMENTATION** | parse owned files + independent viewer/input | neutral server | low–medium after load | high | potentially broad local access | may not reproduce proprietary online behavior | expensive unless formats mature | RE/content-use boundary |

## Selection rule

Choose the **least invasive mode that exposes the hooks required by the claimed capability profile**.

Do not choose reimplementation merely because it is flexible when an official API or supported mod exposes sufficient hooks. Do not claim high-fidelity capability through a gateway that only exposes coarse commands.

---

# L2.2 — Schema → adapter scaffolding

## Source split

L1 owns shared semantic/profile definitions. L2 consumes them and owns adapter implementation tooling.

Generation inputs:

```text
semantic-profile package        # L1-owned meaning
adapter ABI descriptor          # L2-owned callable/runtime shape
engine target descriptor        # L2-owned binding/plugin target
translator manifest values      # adapter-specific declarations
```

The adapter ABI descriptor should be machine-readable and versioned. JSON Schema, a purpose-built IDL, or an existing IDL can implement it.

**Do not make WebAssembly WIT a required dependency yet.** WIT is a useful precedent for versioned imports/exports and generated bindings, but L2 only needs the source-of-truth/code-generation pattern at this stage.

## Generated artifacts

```text
generated/
  rust/adapter_api.rs
  c/signet_adapter.h
  csharp/SignetAdapter.g.cs
  typescript/adapter-api.ts
  schema/signet-adapter-manifest.schema.json
  fixtures/binding-smoke/
  fixtures/replay-skeleton/
  conformance/generated-contract-tests.*
```

Target packages:

```text
engines/
  godot/{generated,shim}/
  unreal/{generated,shim}/
  unity/{generated,shim}/
  open-engine/{generated,shim}/
  gateway/{generated,shim}/
```

## Generation provenance

```yaml
generation:
  adapter_api_version: adapter-api@1
  adapter_api_digest: sha256:...
  semantic_profile_refs:
    - urn:signet:profile:fps@1
  generator:
    id: signet-adapter-gen
    version: 0.1.0
    digest: sha256:...
  target:
    language: csharp
    engine: unity
    target_version_range: ...
```

Generated output should be reproducible from these inputs. Regeneration must never overwrite handwritten shim files.

## Mechanical generator owns

- ABI types and signatures;
- enums/constants;
- serialization wrappers;
- manifest/schema fields;
- required callback skeletons;
- compile-time version guards;
- replay/conformance harness skeletons;
- generated documentation tables.

## Model/LLM may propose

- likely local hook for a required callback;
- coordinate/unit transforms;
- event-mapping candidates;
- error-handling glue;
- tests for observed local behavior;
- migration patches after an update.

Model output enters the normal code/test path. Generated prose is not evidence that the hook works.

## Generator acceptance fixtures

1. same inputs produce deterministic generated output;
2. generated code compiles/links for supported targets;
3. unknown/new API members fail or degrade explicitly;
4. a minimal observer adapter connects and consumes world/state/events;
5. prediction/input claims pass behavioral replay, not only type checking;
6. regeneration does not modify handwritten shim paths;
7. old generated clients remain compatible or hit an explicit version boundary.

---

# L2.3 — “Very smart if” benchmark

## Tasks

```text
EVENT_MAP
FALLBACK_SELECT
FAILURE_TRIAGE
CAPABILITY_CLASSIFY
TEST_PRIORITIZE
```

Every case supplies a closed candidate set plus `NO_MATCH` / `ABSTAIN` where meaningful.

Example:

```yaml
case_id: event-map-openarena-kill-001
task: EVENT_MAP
input:
  game: openarena
  game_version: 0.8.8
  adapter_surface: event-hook@sha256:...
  observation:
    hook: G_PlayerDie
    fields: {...}
candidates:
  - urn:signet:core:event:death@1
  - urn:signet:core:event:damage@1
  - urn:signet:core:event:despawn@1
  - NO_MATCH
gold:
  choice: urn:signet:core:event:death@1
severity_if_wrong: HIGH
language: en
provenance:
  source_refs: [...]
```

## Baselines

Run frozen identical cases through:

1. deterministic rules;
2. exact/normalized lookup;
3. Laya;
4. CLM;
5. general LLM constrained to the same candidates.

Do not give a model extra hidden context unless that is explicitly a separate experiment.

## Metrics

Basic:
- exact-choice accuracy;
- top-k accuracy where valid;
- abstention rate;
- p50/p95 latency;
- cold/warm latency;
- resident memory/VRAM;
- hosted request cost where applicable.

Robustness:
- **selective risk vs coverage**;
- OOD detection;
- candidate permutation stability;
- irrelevant-decoy robustness;
- matched multilingual performance;
- game-version drift sensitivity;
- repeated-run determinism under fixed settings;
- critical-error-weighted loss.

A single confidence score is not sufficient.

## Dataset split

Hold out:
- entire games;
- game versions;
- event families;
- languages;
- near-miss candidate sets;
- malformed/partial observations.

Also rerun a strengthened deterministic baseline so extra research effort is not mistaken for model advantage.

L4 owns independent execution/public benchmark methodology. L2 owns the adapter-oriented task/data contract.

---

# L2.4 — Generate → test → freeze

Adaptive suggestions become release artifacts only after validation.

## Accepted mapping record

```yaml
mapping:
  id: game-x:death-hook->urn:signet:core:event:death@1
  schema: signet-mapping-lock@1

source:
  game_id: game-x
  game_version: 1.4.2
  integration_mode: official_mod
  integration_surface:
    hook: OnPawnKilled
    signature_digest: sha256:...
    package_fingerprint: sha256:...

target:
  semantic_id: urn:signet:core:event:death@1
  semantic_profile: urn:signet:profile:fps@1
  profile_digest: sha256:...

transform:
  implementation_ref: mappings/death.rs
  digest: sha256:...
  fallback: NO_MATCH

proposal:
  method: model_assisted
  model_id: ...
  model_version: ...
  weights_digest: sha256:...
  rubric_digest: sha256:...
  candidate_set_digest: sha256:...
  input_digest: sha256:...
  score: 0.91

acceptance:
  method: replay_plus_conformance
  suite_id: signet-adapter-conformance@...
  suite_digest: sha256:...
  replay_set_digest: sha256:...
  accepted_by: human_or_ci_policy_id
  accepted_at: ...

invalidates_on:
  - game_version_change
  - integration_surface_digest_change
  - semantic_profile_digest_change
  - transform_digest_change
```

Runtime consumes the frozen mapping/transform. The proposal model is not required at runtime.

## Revalidation classes

| Change | Default action |
|---|---|
| docs-only adapter change | no semantic revalidation |
| generated binding changes, same ABI digest | compile + binding smoke |
| hook signature changes | affected mapping replays + conformance |
| file-format sentinel changes | importer fixtures + affected world replays |
| semantic profile digest changes | full affected mapping/conformance |
| proposal model/rubric changes, frozen output unchanged | proposal benchmark only |
| accepted mapping/transform changes | full affected replay/conformance |

---

# L2.5 — Drift detection

## Fingerprint layers

```yaml
fingerprint:
  package:
    version: ...
    executable_or_package_digest: ...
  integration_surface:
    api_schema_digest: ...
    hook_signature_digest: ...
  data_format:
    sentinel_files:
      - path: ...
        format_version: ...
        structural_digest: ...
  behavior:
    replay_corpus_digest: ...
    expected_neutral_output_digest: ...
```

Do not hash every proprietary byte. Bind compatibility claims to the exact surfaces the adapter depends on.

## Claim state

```text
VALID
  | dependency/fingerprint mismatch
  v
SUSPECT
  | targeted revalidation
  v
REVALIDATING
  | pass -----------------> VALID(new evidence)
  |
  +-- fail/unknown -------> INVALID
```

A launcher/registry should not display an old PASS as current while evidence is `SUSPECT` or `INVALID`.

## Impact-directed revalidation

```text
local hook/file format
   -> mapping/transform
   -> generated binding
   -> capability claim
   -> replay fixtures
   -> registry evidence
```

When dependency information is complete, only affected claims need invalidation. Unknown dependency edges fail toward broader revalidation rather than silently preserving PASS.

---

# L2.6 — Debugging and replay

A translator failure should be explainable stage-by-stage.

## Trace

```yaml
trace:
  schema: signet-adapter-trace@1
  trace_id: ...
  adapter: {id: ..., version: ..., digest: ...}
  game: {id: ..., version: ..., integration_fingerprint: ...}
  local:
    frame_or_sequence: ...
    hook: ...
    observation_ref: local://trace/...
    observation_digest: sha256:...
  mapping:
    lock_id: ...
    lock_digest: sha256:...
    neutral_output:
      semantic_id: ...
      payload: ...
  decision:
    capability_result: ...
    fallback_reason: ...
    model_hint:
      present: false
      artifact_ref: null
  validation: {schema: PASS, policy: PASS}
  network:
    command_or_event_id: ...
    tick: ...
    acknowledgement: ...
  authority:
    resulting_event_or_state_ref: ...
  presentation:
    callback: ...
    result: ...
  errors: []
```

Raw observations may contain sensitive/private information; retention/redaction/telemetry policy is an L3 handoff. The trace supports local content-addressed references so debugging does not require uploading raw game files.

## Replay modes

1. capture replay: local observation → expected neutral output;
2. presentation replay: neutral state/event → expected presentation callback;
3. network replay against a deterministic test server;
4. differential replay: old adapter vs candidate over the same corpus;
5. drift replay after an integration-surface fingerprint changes.

## Explain surface

```text
signet-adapter explain trace.json --against expected.trace.json

PASS  local capture
PASS  mapping lock
FAIL  transform output
      expected: urn:signet:core:event:death@1
      actual:   NO_MATCH
CAUSE hook signature changed:
      old sha256:...
      new sha256:...
NEXT  run affected EVENT_MAP fixtures
```

This is more actionable than “translator incompatible.”

---

# Design delta

1. **Generated ABI glue and adaptive semantic mapping are different risk classes.**
2. **Adapter tooling should have a machine-readable generation source.**
3. **A stable C ABI is a pragmatic bridge over Signet's Rust core.**
4. **Every capability claim should bind to integration-surface fingerprints plus replay/conformance evidence.**
5. **Selective-risk/abstention metrics matter more than raw confidence for adaptive mapping.**
6. **Replay is also the adapter migration mechanism**, tying codegen, mapping, drift and explainability together.

# Contradictions / rejected overclaims

### “Generate the whole adapter”
**REJECTED as a general rule.** Evidence supports substantial generated scaffolding, but engine/game-specific behavior still requires hook knowledge and sometimes manual adjustment.

### “High confidence means safe mapping”
**REJECTED.** Domain routing/OOD handling must be evaluated separately from score magnitude.

### “If the binding compiles, the adapter works”
**REJECTED.** API-shape compatibility and behavioral equivalence are separate properties.

# No-delta evidence

- Repeated HLA sources reinforce adapters, generated glue and support tooling; they do not justify moving authoritative simulation semantics into models.
- Engine plugin systems still converge on explicit interfaces rather than self-modifying runtime translation.
- A bounded Korean search did not yield a strong primary/native source adding a distinct mechanism in this pass. Record `NO_MATERIAL_DELTA_YET`, not “no Korean work exists.”

# Next recursion

1. Godot GDExtension proof adapter generated from an L2 descriptor;
2. Unreal plugin proof adapter using the same generated C ABI;
3. Unity native-plugin smoke target for the preview C# path;
4. Minecraft/RCON gateway target;
5. Doom/OpenArena reimplementation/importer target;
6. matched rules/lookup/Laya/CLM/LLM event-mapping benchmark;
7. game-update fixtures that independently change hook signatures, data formats and semantics.

# Handoffs

```yaml
handoff:
  from_lane: L2
  to_lane: L1
  finding: L2 generation needs a stable machine-readable semantic/profile input distinct from the adapter ABI.
  why_material: generated syntax cannot validate meaning without semantic IDs, units, authority and profile identity.
  requested_followup: define the minimum profile artifact L2 generators consume without L2 inventing shared semantics.
```

```yaml
handoff:
  from_lane: L2
  to_lane: L3
  finding: replay/debug artifacts may contain local game state, paths, identifiers or proprietary data; generated/model-assisted packages need provenance/trust policy.
  why_material: storage, redaction, permissions, signing and legal policy are outside L2.
  requested_followup: define trace-sharing and generated-package privacy/trust constraints.
```

```yaml
handoff:
  from_lane: L2
  to_lane: L4
  finding: L2 now defines falsifiable adapter/codegen/model/drift experiments.
  why_material: independent execution and public benchmark methodology belong to L4.
  requested_followup: execute template-vs-LLM and rules-vs-Laya-vs-CLM-vs-LLM experiments with published fixtures.
```

# First-pass completion

- adapter integration matrix — **DONE**
- generated scaffold specification — **DONE**
- model-evaluation dataset design — **DONE**
- decision-model benchmark plan — **DONE**
- accepted-mapping/provenance format — **DONE**
- game-update drift strategy — **DONE**
- debugging/replay tooling proposal — **DONE**

Empirical implementation and benchmark execution remain open.

---

# Signet 2 response delta — 2026-10-05

After the first L2 pass, Signet published its Signet 2 draft at `kian-cx/signetprotocol@490dfa9423841a45f2917d8d013e010ca0eb5548`.

## Material convergence

The new proposal independently adopts several L2 directions:

- game -> shared meaning -> game rather than pairwise game bridges;
- explicit intents and archetypes;
- capabilities;
- closed-set resolver ranking;
- human confirmation;
- pinned translation profiles;
- calibration for obvious/measurable cases before model use;
- adaptive models outside the per-tick simulation loop;
- a separate Forge/development tool.

This materially narrows the disagreement surface.

## New L2 findings caused by the proposal

1. **Profile authority must be typed.** A universal `player pin > translator > model > default` precedence is safe for local appearance/input preferences but ambiguous for simulation-relevant semantic mappings.
2. **Pinned is not equivalent to valid.** A lock needs source/version/integration/profile/transform identity plus evidence and invalidation conditions.
3. **Candidate closure needs `NO_MATCH`.** A closed list prevents invention but can still force a wrong answer if the correct semantic is absent.
4. **Calibration needs an adapter observation contract.** A wizard cannot measure motion properties the integration surface cannot legitimately observe.
5. **Approved decisions need label scope.** Local appearance preference, translator default, semantic mapping and capability classification should not become one undifferentiated training label.
6. **The proposed 30-case test is a good Stage-A pilot.** Architecture-level dependence on a model should wait for held-out games/versions/OOD/abstention tests.
7. **Drift must be first-class.** Game/API/profile changes should move evidence from VALID to SUSPECT until targeted replay/conformance revalidates it.

The public response manuscript develops these points:

`Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md`

---

# Executable proof: pinned choice versus stale evidence

L2 now has a checked research fixture at:

`Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/`

The profile schema gives local input, calibration evidence, semantic mappings, local appearance, and server/session policy different authority classes.

A semantic mapping begins `VALID` and `pinned: true`. The drift fixture changes the integration-surface digest and game version while leaving the pinned choice unchanged.

Observed execution:

```text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
```

**DERIVED:** determinism and currency are orthogonal properties. A lock/profile format should encode both the frozen decision and enough dependency identity to know when that decision requires revalidation.

# Executable proof: deterministic adapter-contract generation

L2 now also has:

`Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/adapter-shim-generation/`

A small machine-readable adapter-shim descriptor generates C, Rust, C# and TypeScript surfaces.

Two independent clean runs produced identical output hashes. C and TypeScript syntax checks passed in the available environment.

**OBSERVED:** deterministic code generation of the repetitive interface layer is feasible for this bounded fixture.

**UNKNOWN:** whether this exact logical shim is sufficient or desirable for Signet 2. Real Godot/Unreal/Unity/gateway implementations remain required before recommending the interface itself.

**DERIVED:** the paper's proposed split can now be stated more narrowly: mechanical generation is justified for contract shape; local engine/game behavior still requires explicit implementation and behavioral evidence.