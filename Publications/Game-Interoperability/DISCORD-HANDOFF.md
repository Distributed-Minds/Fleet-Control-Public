# Compact Community Handoff

Unaffiliated research summary for the cross-game interoperability discussion.

## Short version

Signet and Melty look complementary:

- **Signet:** neutral authoritative runtime + translators + conformance.
- **Melty:** one-click packaging/install/launch + mashup distribution + AI-assisted creation.

The strongest architecture found across current projects, HLA/DIS, and multilingual simulation research is:

> **deterministic core, adaptive edge**

Keep world/rules/events/capabilities deterministic and testable. Use local decision models/LLMs for bounded mapping, routing, adapter generation, diagnostics, and fallback selection.

## Three things I would build next

1. **Machine-readable capability/profile negotiation**
   - not just “translator works”
   - exact actions/events/world features/fallbacks

2. **Semantic profiles + stable IDs**
   - `core`, `fps`, `voxel`, `vehicle`, etc.
   - avoid one giant “universal language”
   - human labels can be translated without changing wire meaning

3. **Compatibility evidence registry**
   - game version
   - translator version
   - conformance suite/hash
   - exact PASS/fail/fallbacks
   - reproducible by others

## Useful prior art

- IEEE HLA: independent simulators as federates in a federation.
- DIS: explicit realtime simulation protocol families.
- Chinese 2024 paper: plugin Unreal Engine ↔ HLA adapter.
- Japanese 2002–2004 work: plug-in simulation adapters + automatic glue-code generation.
- Spanish HLA research: syntax vs semantics vs pragmatics + ontology networks.
- German semantic-interop work: mediator ontology instead of N² pairwise mappings.

## AI decision models

Laya / CLM-style models are promising for a “very smart if”.

Good:
- choose among explicit valid fallbacks;
- map local event → candidate neutral event;
- prioritize tests;
- triage compatibility failures.

Bad:
- authoritative movement/hits/damage/inventory/auth.

Rule:

`model suggests -> deterministic contract validates -> server decides`

## Security note

Treat translators/mashups as supply-chain software. They may touch game files, launch processes, and expose networking.

Use specific labels instead of one “verified” badge:

- signed;
- reproducible source/build;
- static scan;
- conformance pass;
- install smoke pass;
- publisher identity.

## Full package

See:

`Research/Game-Interoperability/Signet-Frontier-2026-10/`

and:

`Publications/Game-Interoperability/OPEN-GAME-INTEROPERABILITY-FRONTIER-DRAFT.md`
