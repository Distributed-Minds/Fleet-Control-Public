# 04 — Decision Models and Adaptive Translation

## The “very smart if” idea is useful—with a boundary

Recent decision-model projects make the community's “very smart if” intuition concrete.

### Laya

Current Laya documentation describes:

- non-autoregressive typed decisions;
- `choice`, `score`, and yes/no-style decisions;
- a 421M English checkpoint;
- a 322M multilingual checkpoint;
- 100+ language support claims;
- local/self-hosted operation;
- Apache-2.0 licensing.

Its benchmark and latency figures are project-reported and were not independently reproduced here.

### CLM-8B

Current CLM documentation describes:

- contrastive state/action scoring;
- candidate ranking;
- typed decisions;
- cached action/state embeddings;
- a Qwen3-8B encoder backbone;
- Apache-2.0 code and released weights;
- game/tool/computer-use benchmarks.

Again, project benchmark figures are `CLAIMED` unless independently reproduced.

### Jev

Jev is a useful commercial reference point for current “System One” decision models, but vendor/media benchmark claims should not be treated as neutral evidence.

---

## Good uses inside game interoperability

Decision models are attractive when the candidate set is already bounded.

### Capability fallback selection

Given required and available features, rank permitted outcomes such as:

- use ordinary audio fallback;
- observer mode;
- reject session.

Deterministic policy still defines which candidates are allowed.

### Semantic mapping suggestion

Map a local event into explicit candidates:

```text
OnPawnKilled
->
core.event.death
core.event.damage
core.event.despawn
no_match
```

The model may propose a mapping. An accepted mapping becomes explicit, versioned, testable, and provenance-bearing.

### Error triage

Rank likely causes:

- wrong game version;
- missing local file;
- unsupported capability;
- translator bug;
- network/auth problem.

### Test prioritization

Rank conformance fixtures most likely to fail after a game update.

### Offline compatibility mining

Use opt-in traces to propose mappings, missing event classes, fallback rules, and regression tests. Ship improvements as normal versioned code/data.

---

## Bad uses

Do not make an adaptive model authoritative for:

- position;
- collision;
- hit detection;
- damage;
- inventory ownership;
- respawn;
- score;
- permission/authentication;
- protocol version acceptance;
- whether unsigned code may execute;
- legal/EULA compliance.

A hidden model update must not become an undeclared ruleset update.

---

## Typed hint interface

Treat model output as a non-authoritative artifact:

```yaml
decision_hint:
  task: semantic_mapping
  model:
    id: laya-multilingual
    version: ...
    weights_hash: ...
  input_digest: ...
  candidates:
    - core.event.death@1
    - core.event.damage@1
    - no_match
  selected: core.event.death@1
  score: 0.91
  rubric_version: semantic-map-v3
  abstained: false
  expires_after:
    translator_version: game-x@1.4.2
```

The deterministic layer validates that the candidate is allowed, schema-compatible, and meets any threshold/human-acceptance rule.

The model never directly creates a new wire semantic class.

---

## Confidence is not truth

Current Laya documentation gives a useful cautionary example: an English checkpoint can be confidently wrong on scripts it cannot read, which motivated explicit language routing.

Analogy:

> A high score does not prove the model is operating inside the domain it understands.

Prefer:

```text
domain gate
-> candidate set
-> decision model
-> deterministic validation
-> optional abstention/human escalation
```

not:

```text
raw game state
-> confidence
-> authoritative action
```

---

## Local-first deployment

A privacy-friendly default:

- no raw game files leave the machine;
- no continuous screen capture is uploaded;
- no personal directory inventory is uploaded;
- embeddings/traces stay local by default;
- telemetry is opt-in and minimized;
- model weights are content-hashed.

Small decision models fit repetitive closed-set compatibility decisions better than an always-online general LLM.

---

## Strongest AI workflow: generate → test → freeze

```text
schema / manifest / source API
        |
AI generates candidate adapter
        |
static validation
        |
conformance suite
        |
replay corpus
        |
cross-implementation differential tests
        |
human inspection where required
        |
versioned translator release
```

This is safer and easier to debug than a runtime that continuously improvises mappings.

## Train the compatibility layer, not the physics

A future compatibility dataset can include:

- game/version;
- translator/version;
- observed APIs/events;
- conformance results;
- fallbacks;
- failures;
- installation errors;
- accepted semantic mappings.

That is excellent training data for local compatibility assistants.

The rules/physics engine should remain ordinary deterministic code.
