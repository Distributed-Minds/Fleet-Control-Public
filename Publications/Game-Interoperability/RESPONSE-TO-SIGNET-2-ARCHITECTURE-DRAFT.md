# From Shared Meaning to Verifiable Translation

## An engineering response to *Signet 2: intents, archetypes and translation profiles*

**Status:** unaffiliated public draft / technical response  
**Date:** 2026-10-05  
**Response target:** Signet Protocol draft proposal, document version 0.1  
**Observed Signet source state:** `kian-cx/signetprotocol@490dfa9423841a45f2917d8d013e010ca0eb5548`  
**Authors:** Distributed Minds / Fleet-Control research contribution  
**Scope:** adapter engineering, adaptive translation, calibration, profile evidence, drift, and evaluation

> This is an independent technical response. It is not official Signet Protocol documentation and does not imply endorsement by Signet's maintainers.

---

## Abstract

The Signet 2 proposal makes a strong architectural move: stop translating games pair-by-pair and instead translate each game into and out of a shared semantic layer. Player actions become **intents**; world entities become **archetypes**; each game declares **capabilities**; a **resolver** ranks closed candidates; and accepted decisions are pinned into **translation profiles** so adaptive machinery stays outside the real-time simulation loop.

We agree with that direction.

Our main argument is that the next step is not to add more intelligence to the resolver. It is to make the translation boundary **verifiable**.

A production adapter ecosystem needs to distinguish at least four kinds of state that the current proposal places near one profile concept:

1. player-local input bindings;
2. measured motion/integration evidence;
3. translator-level semantic mappings;
4. player-local appearance choices.

These have different owners, different invalidation rules, and different authority. A player's choice can legitimately override how a bow is rendered or which button means "fire"; it should not silently override a shared simulation-relevant semantic mapping.

We therefore propose an engineering architecture based on:

> **generated contract glue + thin game-specific hook shims + evidence-bearing frozen mappings + replay/conformance validation**

Adaptive models remain useful, especially as closed-set rankers for difficult mappings, failure triage, capability classification, and test prioritization. But model output should be treated as a proposal artifact. Accepted mappings should carry provenance, candidate-set identity, source-version fingerprints, conformance evidence, and explicit invalidation conditions.

The Signet 2 proposal's planned 30-case model experiment is a good pilot. We propose extending it into a staged benchmark with deterministic and lookup baselines, explicit `NO_MATCH`/abstention cases, held-out games and versions, multilingual/OOD cases, selective-risk metrics, and replay-based drift tests.

The resulting system preserves Signet 2's central virtue: during a match, shared-world truth remains deterministic and inspectable.

---

# 1. The Signet 2 proposal is pointed at the right scaling problem

The proposal's core observation is correct: pairwise translation grows poorly.

If every game needs special knowledge of every other game, interoperability cost grows approximately with the number of pairs. A shared semantic layer changes the engineering target from:

```text
game A <-> game B
game A <-> game C
game B <-> game C
...
```

to:

```text
game A -> shared meaning -> game A
game B -> shared meaning -> game B
game C -> shared meaning -> game C
```

This is the same anti-N-squared pattern seen in mature interoperability systems: each participant implements one contract rather than a growing set of pairwise bridges.

Signet 2 gives the shared layer two especially useful primitives:

- **intents**: what a player wants to do;
- **archetypes**: what exists in the shared world, described independently of local graphics.

That is a better foundation than making one game's object model canonical.

The proposal also makes an important authority distinction: the neutral server remains responsible for physics, rules, damage, and resulting state. Translators handle input and presentation. This is the right place to draw the hard line.

---

# 2. Five choices in Signet 2 that should be preserved

## 2.1 Closed candidates instead of free-form runtime generation

Signet 2 does not ask a model to invent a translation. It asks a resolver to rank a finite set of known choices.

That is a major safety and engineering improvement.

A closed candidate set makes several things testable:

- whether the correct answer was present;
- whether the resolver selected it;
- whether `NO_MATCH` should have won;
- whether candidate order changes the result;
- whether an apparently high score survives out-of-domain cases.

It also makes deterministic fallbacks possible.

## 2.2 Human confirmation before a mapping becomes durable

The proposed Forge cycle is:

```text
suggest -> inspect -> pin -> learn
```

This is much stronger than putting an adaptive model inside the simulation loop.

The model can change. The runtime mapping does not.

## 2.3 Calibration before intelligence

Signet 2 correctly removes obvious decisions from the AI problem.

A user can identify which control means forward. An adapter can measure or declare simple motion properties. There is no reason to ask a model to solve a deterministic configuration problem.

This is a valuable general rule:

> **Measure or ask when possible; infer only when necessary.**

## 2.4 Separate Signet Protocol from Signet Forge

The proposed split between a stable protocol/runtime and a companion translation-development tool is important.

The protocol should remain usable even if:

- Forge is unavailable;
- a model provider disappears;
- a model checkpoint changes;
- no GPU is present;
- the community replaces the ranking model.

Forge can evolve rapidly. The runtime contract should not need to.

## 2.5 Pin simulation-relevant mappings

The proposal explicitly notes that moving/firing mappings cannot remain provisional without risking prediction/server disagreement.

We agree and generalize this:

> Any translation decision that can affect authoritative state needs a frozen, versioned, testable mapping before it becomes part of an authoritative session path.

---

# 3. Main refinement: one translation profile contains several authority classes

The most important ambiguity in the current proposal is not the resolver model. It is profile authority.

The proposal describes a precedence order roughly equivalent to:

```text
player pin
  > translator profile
  > model suggestion
  > safe default
```

That is reasonable for some local choices. It is too broad if interpreted as a universal precedence rule.

Consider four different mappings:

1. **button X -> fire intent**
2. **local movement observations -> motion profile**
3. **game event OnPawnKilled -> shared death semantic**
4. **weapon.ranged -> local bow appearance**

These are not the same kind of decision.

## Proposed authority split

| Profile class | Primary owner | Player override? | Can affect shared authoritative state? | Typical invalidation |
|---|---|---:|---:|---|
| **Input binding profile** | player + adapter | yes | indirectly, through permitted intents | device/control/config change |
| **Motion observation profile** | adapter calibration evidence | not as an arbitrary value | should not redefine server physics | game/version/integration-surface change |
| **Semantic mapping lock** | translator release / accepted mapping | not arbitrarily in an authoritative session | yes | hook/schema/profile/transform change |
| **Appearance profile** | player/client translator | yes | no | local asset/palette/config change |
| **Session capability policy** | server/session | no client unilateral override | yes | session/ruleset/capability negotiation change |

One physical file may contain several sections, but the schema should preserve these authority classes.

## Why this matters

A user choosing:

```text
weapon.ranged -> minecraft:bow
```

is primarily choosing a local representation.

A user choosing:

```text
OnUseItem -> core.intent.fire
```

may be changing what the shared simulation receives.

Those decisions need different acceptance rules.

### Proposed precedence by class

**Input binding**

```text
player binding > adapter default > unbound
```

**Appearance**

```text
player pin > translator default > provisional local resolver > safe visual fallback
```

**Simulation-relevant semantic mapping**

```text
accepted translator mapping + server/session policy
    > explicit incompatibility / NO_MATCH
```

A model may propose the mapping, and a human may accept it, but an arbitrary per-player runtime override should not silently redefine shared semantics.

This preserves Signet 2's user configurability without weakening its server-authoritative design.

---

# 4. A lock file should lock evidence, not only a choice

The current example profile records a key, a choice, resolver identity, candidate scores, and whether the result is pinned.

That is a useful start.

For long-lived compatibility, the profile also needs to answer:

> **What exact environment made this decision valid?**

A frozen mapping can become wrong after:

- the game updates;
- a mod API changes;
- an event signature changes;
- a file format changes;
- an intent/archetype definition changes;
- the translator transform changes;
- the candidate palette changes.

## Proposed mapping-lock shape

```yaml
mapping:
  schema: signet-mapping-lock@1
  id: game-x:on-pawn-killed->core.event.death@1

source:
  game_id: game-x
  game_version: 1.4.2
  integration_mode: official_mod
  integration_surface:
    hook: OnPawnKilled
    signature_digest: sha256:...
    surface_digest: sha256:...

target:
  semantic_id: core.event.death@1
  semantic_profile: fps@1
  semantic_profile_digest: sha256:...

transform:
  implementation_ref: mappings/death.rs
  digest: sha256:...
  fallback: NO_MATCH

proposal:
  method: model_assisted
  resolver_id: ...
  resolver_version: ...
  weights_digest: sha256:...
  candidate_set_digest: sha256:...
  rubric_digest: sha256:...
  input_digest: sha256:...
  score: 0.91

acceptance:
  method: replay_plus_conformance
  replay_set_digest: sha256:...
  suite_digest: sha256:...
  accepted_by: ...
  accepted_at: ...

invalidates_on:
  - integration_surface_digest_change
  - semantic_profile_digest_change
  - transform_digest_change
  - incompatible_game_version_change
```

The runtime does not need the model.

It needs the accepted mapping and the evidence state that says whether the mapping is still valid.

---

# 5. Adapter engineering: generate the contract, not the whole game integration

Signet already provides useful implementation seams:

- Rust translator interfaces;
- a C ABI;
- a C# wrapper over the C ABI;
- a TypeScript client/tooling surface;
- separate integration modes for gateways, open engines, mods, and reimplementations.

The strongest next step is to make the repetitive part mechanically generatable.

## Proposed structure

```text
shared semantic/profile contract
        |
adapter ABI descriptor
        |
        +--> generated Rust glue
        +--> generated C header / ABI wrappers
        +--> generated C# P/Invoke
        +--> generated TypeScript/tool types
        +--> generated manifest schema
        +--> generated replay/conformance skeletons
        |
thin target-specific shim
        |
game/engine hooks
```

The handwritten shim should be small and explicit.

It owns things a generic generator cannot know without target-specific evidence:

- which engine callback exposes an event;
- how a mod API reports input;
- coordinate handedness;
- unit conversion;
- RCON/console commands;
- local data-file structure;
- engine lifecycle hooks;
- game-version quirks.

## Existing precedents

Godot's current GDExtension system is a useful modern pattern: it exposes a C-facing extension boundary and a machine-readable interface description intended to support generated bindings.

Historical HLA work reached a similar conclusion from another direction. Japanese distributed-simulation research built plug-in adapters and support tooling that generated integration source from exchange definitions. More recent model-driven HLA work automates large portions of federate/interface generation but still reports manual behavior-specific adjustments.

The recurring pattern is:

> **Generate the stable interface shape. Handwrite the irreducibly local behavior.**

That is a much more testable target than “AI writes a translator.”

---

# 6. Calibration should become an explicit adapter contract

The calibration wizard is one of Signet 2's strongest ideas, but it needs a technical boundary.

A generic wizard cannot measure a property unless the adapter exposes a legitimate observation surface.

## Proposed calibration-probe interface

An adapter can declare probes such as:

```yaml
probe:
  id: movement.forward.speed
  intent: move_forward
  instruction: "Hold forward for two seconds"
  preconditions:
    - grounded
    - no_status_effects
  observables:
    - local_position
    - elapsed_time
  measurement:
    unit: m/s
    method: median_steady_state
    sample_count: 5
  evidence:
    integration_surface_digest: sha256:...
```

A probe can return:

```yaml
result:
  value: 5.42
  unit: m/s
  dispersion: 0.08
  conditions_digest: sha256:...
  source: official_mod_api
  status: MEASURED
```

If the game does not expose the required observable through a legitimate integration surface, the adapter should say so.

Do not manufacture false precision.

## Minimum useful motion profile

If server physics remains canonical, the motion profile should not become a second physics ruleset.

Its purpose is to support:

- input normalization;
- local prediction/presentation;
- fidelity checks;
- capability compatibility;
- detecting large local/server behavior mismatches.

A reasonable first profile can include:

```text
coordinate frame / handedness
units-per-metre
input ranges / deadzones where relevant
walk-speed observation
run-speed observation
strafe-speed observation
turn/yaw mapping
jump capability + observed apex/airtime where observable
eye/camera height
step/crouch envelope where relevant
measurement conditions + source + fingerprint
```

Not every field must exist for every game.

## What if the game does not report position?

Use the strongest legitimate source available, in this order:

1. open-engine state;
2. official mod/plugin API;
3. controlled dedicated-server telemetry;
4. documented external API;
5. reproducible manual declaration;
6. otherwise mark the measurement unavailable.

External visual inference can be explored experimentally, but it should not be presented as an exact calibration mechanism unless its error is measured and acceptable.

An unsupported measurement is better than a fabricated one.

---

# 7. Closed-list ranking is necessary, but not sufficient

The resolver design is directionally strong.

However, “the model cannot invent outside the list” only protects against one failure mode.

The candidate list itself can be wrong.

For every resolver call, include:

```text
NO_MATCH / ABSTAIN
```

where a valid no-match state is possible.

Otherwise a model is forced to choose a wrong candidate.

## Domain gating before ranking

A robust adaptive path is:

```text
integration/game/version gate
        |
candidate generator
        |
is this input inside the supported domain?
        |
closed-set ranker
        |
deterministic validation
        |
accept / abstain / escalate
```

This matters because model confidence is not proof that the model understands the domain.

Laya's own current documentation provides a useful caution: a checkpoint can be confidently wrong on language/script input it cannot properly interpret. The project's solution is routing/domain selection, not confidence alone.

The same principle applies to game translation.

## Scores are inspectable, not automatically explanatory

A ranked candidate list is useful evidence:

```text
bow       0.81
crossbow  0.74
trident   0.55
```

But the scores do not, by themselves, explain why the model chose the winner.

For review tooling, record:

- candidate descriptions;
- candidate-set version/digest;
- input facts used;
- source game/version;
- semantic target definition/version;
- resolver version;
- ranking;
- prior accepted examples retrieved, if any;
- reviewer decision.

That makes the decision reproducible even if the model is later replaced.

---

# 8. Approved decisions are not all the same kind of training label

Signet Forge proposes learning from every approved decision.

That can work, but only if label scope is preserved.

Consider:

```text
weapon.ranged -> bow
```

One player may prefer a bow. Another may prefer a crossbow.

That is not necessarily a universal semantic truth.

By contrast:

```text
OnPawnKilled -> death event
```

may be a translator-level semantic mapping.

Do not put both into one undifferentiated training target.

## Proposed training-example scopes

```yaml
label_scope:
  - LOCAL_PREFERENCE
  - TRANSLATOR_DEFAULT
  - SEMANTIC_MAPPING
  - CAPABILITY_CLASSIFICATION
  - FAILURE_TRIAGE
```

A local appearance preference can improve recommendation personalization without becoming global ground truth.

A semantic mapping can become general training data only with the source/version/context that made it correct.

## Preserve negative candidates

For a ranker, training data should include:

- state/context;
- full candidate set or reproducible candidate-set identity;
- accepted choice;
- rejected hard negatives where useful;
- `NO_MATCH` cases;
- reviewer agreement/disagreement;
- source/version fingerprints.

Without candidate context, an approved choice is incomplete supervision.

---

# 9. The proposed 30-case experiment is a good pilot, not a final architecture test

Signet 2 proposes approximately 30 cases comparing:

- handwritten rules;
- text similarity;
- a contrastive ranker.

That is exactly the right first experiment.

We recommend keeping it as **Stage A**, then expanding only if the result is promising.

## Stage A — 30-case smoke experiment

Goal:

> Is there any evidence that a learned ranker adds value over trivial baselines?

Use hand-audited cases.

Include `NO_MATCH`.

If rules or similarity already solve the problem, stop.

## Stage B — adapter benchmark

Scale to enough cases to separate task families:

```text
EVENT_MAP
APPEARANCE_MAP
FALLBACK_SELECT
FAILURE_TRIAGE
CAPABILITY_CLASSIFY
TEST_PRIORITIZE
```

Use the same candidate sets for every resolver.

Baselines:

1. exact lookup;
2. deterministic rule engine;
3. text/embedding similarity;
4. Laya;
5. CLM;
6. general LLM constrained to the same candidates.

Model/vendor benchmark claims should not substitute for this task-specific evaluation.

## Stage C — generalization and failure testing

Hold out:

- whole games;
- game versions;
- event families;
- languages;
- adapter/integration modes.

Add:

- ambiguous cases;
- missing-correct-candidate cases;
- malformed observations;
- decoy candidates;
- candidate-order permutations;
- semantically close hard negatives.

## Metrics

Top-1 accuracy is useful but insufficient.

Measure:

- exact-choice accuracy;
- top-k accuracy where appropriate;
- abstention rate;
- **selective risk vs coverage**;
- critical-error-weighted loss;
- calibration metrics;
- OOD/no-match detection;
- candidate-order stability;
- decoy robustness;
- latency p50/p95;
- cold/warm latency;
- memory/VRAM;
- human override rate;
- reviewer agreement.

A resolver that achieves slightly lower raw accuracy but reliably abstains on dangerous cases may be the better engineering choice.

Independent benchmark execution belongs naturally in Signet Forge or an external conformance/evaluation package, not in the real-time protocol.

---

# 10. Translation profiles need drift and invalidation semantics

A pinned answer is deterministic.

It is not automatically still correct six months later.

## Fingerprint the surfaces a mapping depends on

Do not hash every game asset.

Bind a compatibility claim to the narrow surfaces the adapter actually uses:

```yaml
fingerprint:
  game:
    version: ...
  integration_surface:
    api_schema_digest: ...
    hook_signature_digest: ...
  data_format:
    structural_sentinel_digest: ...
  translator:
    version: ...
    transform_digest: ...
  semantics:
    profile_digest: ...
  evidence:
    replay_corpus_digest: ...
```

## Evidence state

```text
VALID
  |
dependency/fingerprint changed
  v
SUSPECT
  |
targeted replay/conformance
  v
REVALIDATING
  |                 |
 pass               fail / unknown
  v                 v
VALID(new evidence) INVALID
```

A launcher, registry, or server should not continue presenting an old PASS as current while its dependencies are `SUSPECT`.

## Impact-directed revalidation

Track dependencies:

```text
game hook / file format
        |
mapping / transform
        |
generated binding
        |
capability claim
        |
replay fixtures
        |
registry compatibility evidence
```

When one hook changes, rerun the affected mappings.

When dependency information is incomplete, fail toward broader revalidation rather than silently preserving compatibility.

---

# 11. Replay should be the common evidence format

Replay is useful for more than conformance.

It can connect:

- adapter development;
- profile acceptance;
- drift detection;
- regression testing;
- debugging;
- differential comparison.

## Proposed trace

```yaml
trace:
  schema: signet-adapter-trace@1

  adapter:
    id: ...
    version: ...
    digest: ...

  game:
    id: ...
    version: ...
    integration_fingerprint: ...

  local:
    sequence: ...
    hook: ...
    observation_ref: local://trace/...
    observation_digest: sha256:...

  mapping:
    lock_id: ...
    lock_digest: ...
    semantic_output:
      id: ...
      payload: ...

  capability:
    result: ...
    fallback_reason: ...

  adaptive_hint:
    present: false
    artifact_ref: null

  validation:
    schema: PASS
    policy: PASS

  network:
    tick: ...
    command_or_event_id: ...
    acknowledgement: ...

  authority:
    resulting_state_or_event_ref: ...

  presentation:
    callback: ...
    result: ...

  errors: []
```

Raw observations may remain local and be referred to by digest. Privacy/retention/upload policy belongs to the trust/privacy layer, not to the adapter schema itself.

## Useful replay modes

1. **capture replay** — local observation -> expected intent/event;
2. **presentation replay** — neutral event -> expected local callback;
3. **network replay** — intent/event sequence against deterministic server;
4. **differential replay** — old adapter vs candidate adapter;
5. **drift replay** — previous validated surface vs updated surface.

A developer tool can then explain the first divergent stage rather than returning “translator incompatible.”

---

# 12. Two semantic questions Signet 2 should keep separate from L2 engineering

These belong primarily to the shared-semantics layer, but they affect adapter correctness.

## 12.1 Archetype identity versus rule parameters

`weapon.ranged` is a useful coarse semantic class.

It should not need to encode every mechanical parameter into the archetype name.

Prefer a composition such as:

```text
archetype identity
+ authoritative rule/state parameters
+ local appearance mapping
```

For example:

```yaml
entity:
  archetype: weapon.ranged@1
  authoritative:
    damage_model: ...
    range_m: ...
    cadence: ...
  presentation:
    local_choice: minecraft:bow
```

The server owns the authoritative behavior. The client chooses the local presentation.

That avoids turning the archetype catalog into an enumeration of every possible weapon balance configuration.

## 12.2 “Common subset” can become a lowest-common-denominator trap

A capability intersection is a useful baseline, but a future system may need richer outcomes than yes/no common support:

```text
SUPPORTED
SUPPORTED_WITH_FALLBACK
OBSERVE_ONLY
LOCALLY_APPROXIMATED
INCOMPATIBLE
```

A game that cannot jump should not necessarily remove jump from everyone else's universe.

It may instead need an explicit degradation policy.

The exact capability semantics belong to the protocol/semantic layer, but adapters should be able to report the evidence needed for that negotiation.

---

# 13. Direct answers to the Signet 2 open questions

## Q1. What is the minimum a motion profile must measure?

Only values required for input normalization, local fidelity/prediction, and compatibility evidence.

Start with:

- coordinate/unit transform;
- move/strafe input ranges;
- walk/run observations;
- turn/yaw mapping;
- jump capability and observed jump envelope where available;
- eye/camera height;
- step/crouch envelope where relevant;
- measurement conditions;
- integration-surface fingerprint.

Do not let the motion profile redefine canonical server physics.

## Q2. How do we measure it in a game that does not report its position?

Use legitimate observability surfaces in priority order:

1. open engine;
2. permitted plugin/mod API;
3. controlled server telemetry;
4. official external API;
5. reproducible manual declaration;
6. otherwise mark the field unavailable.

Do not make “must measure” an architectural requirement when the integration cannot actually observe the quantity.

## Q3. Who maintains the vocabulary and catalog?

Use a small versioned core plus extension namespaces/profiles rather than one permanently expanding flat list.

This response does not prescribe Signet governance, but the technical artifacts should allow:

- core semantic IDs;
- profile/package versions;
- extension namespaces;
- deprecation without silent reinterpretation;
- deterministic conformance vectors.

## Q4. Where should the resolver run?

Default:

> **Forge/build-time or translation-development time.**

Optional:

- locally on the translator author's machine;
- in an opt-in hosted service;
- on a profile miss only for non-authoritative local choices.

The runtime protocol should not require the resolver to be available.

## Q5. How should conflicting community profiles be resolved?

First distinguish profile class.

For player-local appearance/input preferences, conflict is normal.

For translator-level semantic mappings, key the claim by:

```text
game/version
+ integration surface
+ translator version
+ semantic profile version
+ evidence
```

Prefer the candidate with current reproducible conformance/replay evidence.

Popularity can help discovery. It should not replace compatibility evidence.

---

# 14. A concrete joint experiment

The fastest useful next step is not a universal translator.

It is a small end-to-end proof that exercises the complete evidence chain.

## Experiment 1 — no AI

### Step 1: freeze a tiny semantic target

For example:

```text
intents:
  move
  turn
  fire
  use

archetypes:
  body
  weapon.ranged
  health_pickup
```

### Step 2: define one adapter ABI descriptor

Generate:

- Rust glue;
- C header;
- one engine binding;
- manifest skeleton;
- replay fixture skeleton.

### Step 3: implement two different integration modes

For example:

- a Godot/Open-engine plugin-style adapter;
- a Minecraft controlled-server gateway or Doom/OpenArena importer/reimplementation path.

### Step 4: calibrate without AI

Capture:

- input binding;
- a minimal motion profile;
- capabilities;
- an appearance mapping.

### Step 5: produce evidence-bearing lock records

Pin mappings with:

- source fingerprints;
- semantic-profile digest;
- transform digest;
- replay/conformance evidence.

### Step 6: deliberately break one dependency

Change:

- a hook signature;
- a palette entry;
- a game version;
- or a profile definition.

The old compatibility claim should become `SUSPECT`, not silently remain PASS.

If this works, the adapter/profile architecture is real.

## Experiment 2 — resolver pilot

Then run the Signet 2 30-case experiment.

Include:

- deterministic rules;
- exact/normalized lookup;
- text similarity;
- one contrastive ranker;
- `NO_MATCH`.

Only add a larger model if the simpler methods leave a meaningful unresolved error class.

If the model adds value, expand to the Stage B/C benchmark above.

---

# 15. Proposed changes to the Signet 2 draft

The following changes would make the current proposal substantially easier to implement and test without changing its core direction.

1. **Split translation-profile entries by authority class.**
   - input binding;
   - motion observation;
   - semantic mapping;
   - appearance mapping;
   - session policy.

2. **Add source/version/evidence identity to pinned mappings.**
   - game version;
   - integration-surface digest;
   - semantic-profile digest;
   - transform digest;
   - replay/conformance evidence.

3. **Require `NO_MATCH` / abstention where a candidate set may be incomplete.**

4. **Define a calibration-probe interface.**
   The wizard can only measure values the adapter exposes through legitimate observability.

5. **Treat resolver scores as ranking evidence, not as explanation by themselves.**

6. **Scope training labels.**
   A player appearance preference should not automatically become universal semantic ground truth.

7. **Keep the 30-case comparison as a pilot, then use held-out games/versions/OOD cases before architectural dependence on a model.**

8. **Make compatibility claims stateful.**
   `VALID -> SUSPECT -> REVALIDATING -> VALID/INVALID`.

9. **Use replay as the common adapter evidence format.**

10. **Generate repetitive ABI/binding/conformance glue mechanically; keep target-specific hooks explicit and small.**

These changes are compatible with the strongest part of the proposal:

> shared meaning is deterministic; adaptive machinery helps build the translation, then gets out of the runtime path.

---

# 16. Conclusion

Signet 2 has moved the discussion from “how do two games talk?” to the more scalable question:

> **What shared meaning can many games implement independently?**

That is the right question.

The next engineering risk is not primarily whether CLM, Laya, a general LLM, or handwritten rules produce the best ranking.

The larger risk is allowing a convenient translation file to become an ambiguous authority surface.

If input preferences, measured motion, semantic mappings, appearance choices, and session policy are separated by type and authority, the rest becomes much easier to reason about.

If pinned mappings also carry the evidence that made them valid, Signet can detect when compatibility has drifted instead of treating determinism as correctness.

If adapter interfaces are machine-readable, Signet can generate most repetitive glue and leave human effort concentrated on the small amount of irreducibly game-specific work.

If model suggestions are evaluated against deterministic baselines, allowed to abstain, frozen after acceptance, and replay-tested, adaptive translation can improve the ecosystem without becoming hidden shared-world authority.

That gives a concrete interpretation of the architecture:

```text
                       SIGNET SHARED SEMANTICS
                   intents / archetypes / profiles
                              |
                              v
                     versioned adapter contract
                              |
                mechanically generated glue
                              |
                  thin game-specific hook shim
                              |
             +----------------+----------------+
             |                                 |
       deterministic                     adaptive helper
       runtime path                     build/review path
             |                                 |
      frozen mappings <---- inspect/test <--- suggestions
             |
       replay + conformance
             |
      evidence-bearing compatibility
```

The useful target is therefore not “an AI that can translate any game.”

It is:

> **a translator ecosystem where most adapters are straightforward to build, every adaptive choice becomes an inspectable artifact, and compatibility can be reproduced after the game changes.**

That would make Signet 2 not only scalable in number of games, but scalable in maintenance and trust.

---

# References

## Signet Protocol

1. Signet Protocol. *Signet 2: intents, archetypes and translation profiles*. Draft proposal, document version 0.1, 2026-10-05. Added to `kian-cx/signetprotocol` at commit `490dfa9423841a45f2917d8d013e010ca0eb5548`.
2. Signet Protocol. `docs/content/proposals/translation-profiles.mdx`, same commit.
3. Signet Protocol. Signet Forge documentation: `docs/content/forge/`, same commit.
4. Signet Protocol. Translator traits: `crates/signet-sdk/src/traductor.rs`.
5. Signet Protocol. C SDK interface: `docs/content/sdk/c.mdx` and `crates/signet-ffi/include/signet.h`.
6. Signet Protocol. C# and TypeScript SDK documentation.

## Adapter and generated-integration precedents

7. Godot Engine documentation. *What is GDExtension?* and *GDExtension interface JSON file*.
8. 日比野 浩典, 福田 好朗, 由良 佳之. *生産システム評価のための分散シミュレーションの研究 : 第二報 HLA を利用した Manufacturing Adapter の開発*. 2002. DOI: `10.1299/jsmemsd.2002.99`.
9. 清水 崇文, 金井 理, 岸浪 建史. *HLAを用いた製品設計用多分野分散協調シミュレーションの研究 — 協調シミュレーション支援ツールの開発*. 2004. DOI: `10.11522/pscjspe.2004S.0.361.0`.
10. 杨妹, 王鹏. *一种基于插件的虚幻引擎HLA分布式仿真适配器*. 系统仿真学报 36(10), 2024. DOI: `10.16182/j.issn1004731x.joss.24-0872`.
11. El Kassis et al. *An HLA-based automated approach for the interoperable simulation of collaborative business processes*. Simulation Modelling Practice and Theory 135 (2024), 102977. DOI: `10.1016/j.simpat.2024.102977`.

## Adaptive decision systems

12. Laya project documentation and repository, current 2026-10 research snapshot. Project performance claims treated as author-reported until independently reproduced.
13. Contrastive Language Models (CLM) project documentation and repository, current 2026-10 research snapshot. Project performance claims treated as author-reported until independently reproduced.

---

## Reproducibility note

The companion Fleet-Control research package for this draft lives on:

`Distributed-Minds/Fleet-Control-Public:research/signet-l2-adapters-ai`

Relevant supporting files:

- `Research/Game-Interoperability/Signet-Frontier-2026-10/lanes/L2-Adapter-Engineering-Findings.md`
- `Research/Game-Interoperability/Signet-Frontier-2026-10/lanes/L2-Source-Ledger.md`
- `Research/Game-Interoperability/Signet-Frontier-2026-10/lanes/L2-Adapters-Adaptive-Translation.md`

