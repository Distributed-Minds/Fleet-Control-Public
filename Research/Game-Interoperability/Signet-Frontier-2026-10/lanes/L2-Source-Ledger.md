# L2 — Source Ledger

**Lane:** L2 — Adapter Engineering & Adaptive Translation  
**Research cutoff:** 2026-10-05

Project/vendor performance claims remain **CLAIMED** unless independently reproduced.

## A. Signet implementation — primary project sources

Observed repository head:

`36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c`

Repository:
- https://github.com/kian-cx/signetprotocol

Key files:
- translator traits, integration modes and manifest:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/crates/signet-sdk/src/traductor.rs
- translator architecture and Doom/OpenArena/Minecraft examples:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/docs/content/concepts/translators.mdx
- Rust SDK:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/docs/content/sdk/rust.mdx
- C ABI:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/docs/content/sdk/c.mdx
- C header:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/crates/signet-ffi/include/signet.h
- C# / Unity preview binding:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/docs/content/sdk/csharp.mdx
- TypeScript SDK:  
  https://github.com/kian-cx/signetprotocol/blob/36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c/docs/content/sdk/typescript.mdx

**Supports:** current four-part translator model; integration modes; Rust→C→C# layering; current engine-facing C seam; TypeScript prediction limitation; C# preview status.

## B. Engine/API generation precedents

### Godot GDExtension — primary documentation

- overview:  
  https://docs.godotengine.org/en/latest/engine_details/engine_api/gdextension/what_is_gdextension.html
- C interface JSON:  
  https://docs.godotengine.org/en/latest/engine_details/engine_api/gdextension/gdextension_interface_json_file.html

Current documentation states that:
- GDExtension lets native shared libraries interact with Godot at runtime;
- the low-level surface is C-facing;
- `gdextension_interface.json` is the source of truth for the C interface;
- the JSON is intended for language bindings to generate code;
- interface functions carry version-introduction metadata.

**Limitation:** consulted pages are `latest`/unstable documentation. Pin stable engine versions during implementation.

### Unreal Engine — primary documentation

- C++ API: https://dev.epicgames.com/documentation/en-us/unreal-engine/API
- Modules: https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-engine-modules
- Plugins: https://dev.epicgames.com/documentation/en-us/unreal-engine/plugins-in-unreal-engine

**Supports:** modules/plugins as a target-specific lifecycle/hook shim surface.

### Unity native plug-ins — primary documentation

- https://docs.unity3d.com/Manual/NativePluginInterface.html

**Supports:** native-plugin lifecycle/interface precedent.

**Limitation:** implementation must pin the exact supported Unity version.

## C. IDL/binding-generation reference

### WebAssembly Component Model / WIT

- WIT design:  
  https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md
- Component Model documentation:  
  https://component-model.bytecodealliance.org/

**Supports:** versioned interface-description and generated-binding patterns.

**Boundary:** this pass does not recommend making WIT/Component Model a required Signet dependency. It is an architectural reference.

## D. Native-language and historical adapter research

### Chinese — Unreal ↔ HLA plugin adapter

杨妹, 王鹏.  
**一种基于插件的虚幻引擎HLA分布式仿真适配器**  
系统仿真学报, 2024, 36(10): 2231–2237.  
DOI: `10.16182/j.issn1004731x.joss.24-0872`

- https://www.china-simulation.com/CN/10.16182/j.issn1004731x.joss.24-0872

**Evidence class:** native-language journal source.

**Supports:** current plugin-style Unreal/HLA adapter architecture and rapid federate integration.

### Japanese — plug-in Manufacturing Adapter

日比野 浩典, 福田 好朗, 由良 佳之.  
**生産システム評価のための分散シミュレーションの研究 : 第二報 HLA を利用した Manufacturing Adapter の開発**  
2002.

- https://www.jstage.jst.go.jp/article/jsmemsd/2002/0/2002_99/_article/-char/ja/
- DOI: `10.1299/jsmemsd.2002.99`

**Evidence class:** native-language conference proceedings.

**Supports:** plug-in-style adapters connecting heterogeneous simulators to HLA/RTI.

### Japanese — support tool + code generation

清水 崇文, 金井 理, 岸浪 建史.  
**HLAを用いた製品設計用多分野分散協調シミュレーションの研究 — 協調シミュレーション支援ツールの開発**  
2004.

- https://www.jstage.jst.go.jp/article/pscjspe/2004S/0/2004S_0_361/_article/-char/ja/
- DOI: `10.11522/pscjspe.2004S.0.361.0`

**Evidence class:** native-language conference proceedings.

**Supports:** reading simulator models, graphically defining exchanged information, and automatically generating additional HLA integration source code.

### Japanese — HLA environment support system

日比野 浩典, 福田 好朗.  
**生産システム評価のための分散シミュレーションの研究 : 第三報 HLA 環境サポートシステム**  
2002.

- https://www.jstage.jst.go.jp/article/jsmemecjo/2002.5/0/2002.5_373/_article/-char/ja/
- DOI: `10.1299/jsmemecjo.2002.5.0_373`

**Evidence class:** native-language conference proceedings.

**Supports:** support tooling as part of practical interoperability engineering.

## E. Model-driven HLA generation

M. El Kassis et al.  
**An HLA-based automated approach for the interoperable simulation of collaborative business processes**  
Simulation Modelling Practice and Theory 135 (2024), 102977.

- https://www.sciencedirect.com/science/article/pii/S1569190X24000911
- DOI: `10.1016/j.simpat.2024.102977`
- institutional record: https://art.torvergata.it/handle/2108/388248

**Evidence class:** peer-reviewed open-access research.

**Supports:** automated FOM and federate-code generation from model-driven inputs.

**Negative finding:** the paper still reports manual adjustment for some federate behavior. This directly argues against treating schema/model generation as full adapter generation.

## F. HLA wrapper generator

EODiSP — **HLA Wrapper Generator**

- https://www.pnp-software.com/eodisp/documentation/developerManuals/HLAWrapperGenerator.html

**Evidence class:** project/developer documentation.

**Supports:** generating a typed simulation-independent wrapper/skeleton around a difficult HLA interface while leaving package-specific behavior for hooks.

## G. Decision models — primary project sources

### Laya

- https://github.com/Dancing-coin/l-aya

**Evidence class:** primary project self-description.

Relevant current properties:
- typed closed-set decisions;
- English and multilingual checkpoints;
- routing/domain-selection logic;
- local/self-hosted deployment;
- Jev-compatible HTTP surface.

**Negative finding from project documentation:** a checkpoint can remain highly confident outside a readable language/script domain. Domain/OOD routing is therefore a distinct requirement from confidence/calibration.

Project benchmark and latency numbers remain **CLAIMED**.

### CLM

- https://github.com/AmesianX/clm

**Evidence class:** primary project self-description.

Relevant current properties:
- typed decision API;
- direct candidate ranking;
- contrastive state/action scoring;
- cached candidate/state representations;
- local/self-hosted implementation and fine-tuning tools.

Project benchmark and latency numbers remain **CLAIMED**.

### Jev / System One

Used only as a commercial/API comparison point in this pass.

**Boundary:** vendor benchmark claims are not used here as independent evidence.

## H. No-delta / unfinished native lane

### Korean

A bounded Korean-language discovery pass did not produce a sufficiently strong primary/native source adding a distinct adapter-engineering mechanism beyond the plugin/codegen/support-tool patterns already established above.

**Status:** `NO_MATERIAL_DELTA_YET`, not “no Korean work exists.”

Next vocabulary:
- HLA 연동
- 분산 시뮬레이션
- 어댑터
- 플러그인
- 코드 자동 생성
- 연동 프레임워크

## Source discipline

1. Current project behavior comes from primary project code/docs where possible.
2. Engine architecture claims come from engine documentation.
3. Native research preserves original titles and DOIs.
4. Model performance remains **CLAIMED** until independently reproduced.
5. A failed bounded search is no-material-delta evidence, never proof of nonexistence.
6. Legal/privacy/package-trust interpretation discovered during L2 is handed to L3.

## I. Signet 2 proposal — new primary response target

Observed current Signet main:

`490dfa9423841a45f2917d8d013e010ca0eb5548`

Commit message:

`Docs: Signet Forge section, Signet 2 proposal and architecture PDF`

This commit directly follows the L2 package's earlier observed Signet baseline `36cf99c9a24d0ea1ef984c74a2f4d72fcb7fa85c`.

Primary new sources:
- `docs/public/signet-2-architecture.pdf`
- `docs/content/proposals/translation-profiles.mdx`
- `docs/content/forge/index.mdx`
- `docs/content/forge/understanding-a-game.mdx`
- `docs/content/forge/workflow.mdx`
- `docs/content/forge/fine-tuning.mdx`

Material new observations:
- the proposal is explicitly a draft and not implemented;
- Signet 2 defines intents, archetypes, capabilities, appearance palettes, resolvers and translation profiles;
- calibration handles obvious and measurable mappings before model use;
- the resolver ranks closed candidates and accepted choices are pinned;
- simulation-affecting intents must be pinned;
- the model is explicitly unvalidated for this use case;
- the proposed first experiment is approximately 30 cases comparing rules, text similarity and a contrastive ranker;
- Forge is separated from the runtime standard.

L2 uses these as primary project claims and responds in:

`Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md`

## J. Expanded Forge documentation and executable profile fixture

Signet main advanced again to:

`2ddb136ee941705d3e1c020eaddad93be65026f2`

Commit:

`Docs: full Signet 2 / Signet Forge design document under /forge`

Additional primary pages checked:

- `docs/content/forge/architecture.mdx`
- `docs/content/forge/how-the-model-decides.mdx`
- `docs/content/forge/reliability-and-limits.mdx`
- `docs/content/forge/roadmap.mdx`

**NO MATERIAL REVERSAL:** the expanded docs preserve the proposal's core claims used by L2: one ranked translation API, player-pin/profile/model/default precedence, model outside the game loop, pinned in-match decisions, server-authoritative physics, and an unvalidated 30-case resolver pilot.

The clearer global precedence statement strengthens L2's typed-authority question rather than resolving it: local appearance/input choices and simulation-relevant semantic mappings are still represented near one translation-profile abstraction.

L2 therefore added an executable research fixture:

`Research/Game-Interoperability/Signet-Frontier-2026-10/experiments/L2-Verifiable-Translation-Prototype/`

Local validation result:

```text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
```

The fixture is **PROPOSED research design**, not a claim that Signet has adopted these schemas.

## K. Godot 4.7.2 concrete target

Current stable target selected for the empirical native-engine pass:

- Godot `4.7.2-stable`;
- release commit `ed1daf0bf001b61586d9930840f2f1394092c079`;
- `core/extension/gdextension_interface.json` blob `9b55cc9810d458940acf7659dfac879abbcb8949`;
- interface JSON format version `1`.

Primary Godot documentation states that `gdextension_interface.json` is the low-level C-interface source of truth and is intended for language-binding code generation.

L2 extracted only the ABI subset required by the harness. No performance or runtime-compatibility claim is inferred from documentation alone.

## L. Minecraft gateway and RCON concrete target

Signet primary sources at `2ddb136ee941705d3e1c020eaddad93be65026f2`:

- `docs/content/guides/gateway.mdx` blob `00dd8db68b6bb123a6c3295ce822ea505e388bcf`;
- `docs/content/concepts/translators.mdx` blob `6f537cffac65da1287106e2fdcf222aa0e3348a5`;
- `CHANGELOG.md` blob `9aebc95baf742890fc2decf71a07adbe9f748044`.

These support the documented Minecraft mechanisms and the known unnumbered-intent gap.

Independent RCON implementation evidence:

- `gorcon/rcon`;
- `packet.go` blob `168527b1866a0e0ca19558c1df46edcba18e877a`;
- MIT license.

Its packet implementation corroborates little-endian size/id/type fields, ASCII body and two NUL terminators.

L2's fake-server roundtrip is an executed transport fixture, not evidence of a live Minecraft-server test.
