# L2 Verifiable Translation Prototype v0

This is an **unaffiliated research fixture**, not a Signet standard or implementation.

It turns one claim from the response paper into executable data contracts:

> a translation profile should preserve different authority classes, and a pinned mapping should become `SUSPECT` when the evidence surface that justified it changes.

The fixture intentionally does **not** define Signet's shared semantic vocabulary. Semantic identifiers remain opaque references owned by the protocol/semantics layer.

## Included proof

- `schemas/typed-translation-profile.v0.schema.json` — separates input bindings, motion observations, semantic mappings, appearance mappings, and session policy.
- `examples/typed-profile.example.json` — one profile carrying all five without conflating their authority.
- `examples/drift.surface-change.example.json` — changes the integration-surface fingerprint and game version.
- `tools/validate_profile.py` — validates the example profile against JSON Schema Draft 2020-12.
- `tools/check_drift.py` — deterministically classifies a semantic mapping as `SUSPECT` when a declared dependency changes.

## Authority classes

| Class | Who may choose it? | Shared-state authority | Typical evidence |
|---|---|---|---|
| `INPUT_BINDING` | player/local adapter | indirect only | device/control identity |
| `MOTION_OBSERVATION` | calibration process | does not redefine server physics | measured value + conditions + integration fingerprint |
| `SEMANTIC_MAPPING` | accepted translator release | may feed authoritative simulation | source surface + semantic profile + transform + replay/conformance |
| `APPEARANCE_MAPPING` | player/client translator | local only | local palette/version |
| `SESSION_POLICY` | server/session | authoritative policy | ruleset/capability negotiation |

## Drift rule

A `SEMANTIC_MAPPING` entry records dependency digests. If the current adapter reports a different digest for any declared dependency, the mapping state becomes `SUSPECT` until replay/conformance revalidates it.

`pinned: true` therefore means **deterministic choice**, not **permanent correctness**.

## Run

```bash
python3 tools/validate_profile.py
python3 tools/check_drift.py examples/typed-profile.example.json examples/drift.surface-change.example.json
```

Validated locally on 2026-10-05:

```text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
```

The drift checker returns exit code `10` when a mapping becomes `SUSPECT`, so CI can distinguish drift from parser/tool failure.

## Status

This is deliberately a small falsifiable prototype. The next prototype should add generated adapter ABI/binding glue and replay fixtures, not expand this schema into Signet's semantic vocabulary.
