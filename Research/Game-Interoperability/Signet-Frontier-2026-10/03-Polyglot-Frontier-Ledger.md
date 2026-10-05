# 03 — Polyglot Frontier Ledger

## Method

This pass used native discovery terms and local publication ecosystems where reachable, then measured **marginal information gain** over the English/current baseline.

Status values:

- `MATERIAL_DELTA`
- `NO_MATERIAL_DELTA`
- `PARTIAL`
- `BLOCKED`

## English / standards baseline — MATERIAL_DELTA

Discovery terms:

- High Level Architecture / HLA
- federate / federation
- Object Model Template / OMT
- Distributed Interactive Simulation / DIS
- glTF
- OpenUSD
- OpenXR
- Web of Worlds

The mature simulation field frames interoperability as a federation of independently developed simulators with explicit interface/object models, time coordination, and responsibilities.

**Delta:** prefer a small common runtime plus explicit semantic object/event models, capability profiles, and conformance.

---

## Chinese — MATERIAL_DELTA

Native discovery terms:

- `互操作` — interoperability
- `分布式仿真` — distributed simulation
- `高层体系结构` — High Level Architecture
- `虚幻引擎` — Unreal Engine
- `适配器` — adapter
- `联邦成员` — federate

Key source:

杨妹, 王鹏, **《一种基于插件的虚幻引擎HLA分布式仿真适配器》**, 系统仿真学报, 2024, 36(10): 2231–2237. DOI `10.16182/j.issn1004731x.joss.24-0872`.

The paper describes a plugin-based Unreal Engine adapter that lets Unreal applications become HLA federates and integrate with traditional distributed simulations.

**New implication:** a game engine can be integrated as a reusable federation participant through a plugin adapter instead of building a custom pairwise bridge.

**Downstream delta:** adapter SDK scaffolding and reusable engine-side plugin infrastructure become high-value priorities.

---

## Japanese — MATERIAL_DELTA

Native discovery terms:

- `分散シミュレーション` — distributed simulation
- `協調シミュレーション` — cooperative simulation
- `HLA/RTI`
- `Manufacturing Adapter`
- `支援ツール` — support tool
- `自動生成` — automatic generation

Key sources include:

1. **「生産システム評価のための分散シミュレーションの研究 : 第二報 HLA を利用した Manufacturing Adapter の開発」** (2002)
2. **「HLAを用いた製品設計用多分野分散協調シミュレーションの研究 — 協調シミュレーション支援ツールの開発」** (2004)
3. **「分散通信ミドルウェアHLA/RTIの宇宙機シミュレーションへの適用性評価」** (2004)

The work identifies the developer-experience problem: simulators need exchange definitions and HLA glue code; support tools can graphically define exchanged information and generate source code.

**New implication:** modern AI-assisted translator creation should target a strict adapter schema/test suite rather than inventing the interoperability contract.

---

## Spanish — MATERIAL_DELTA

Native discovery terms:

- `interoperabilidad sintáctica`
- `interoperabilidad semántica`
- `interoperabilidad pragmática`
- `red de ontologías`
- `simulación distribuida`
- `federados`

Key source:

Gutiérrez, Sarli, Leone, **“SCFHLA: Un Modelo de Interoperabilidad Semántica para Simulación Distribuida de Cadenas de Suministro”**, RISTI 30 (2018).

The work uses HLA for distributed simulation but argues that syntactic interoperability is insufficient: participants must agree on the meaning of objects, events, interactions, and metrics. It uses a network of ontologies and mappings.

**Design correction:**

```text
PARSES SAME MESSAGE
!=
MEANS SAME THING
!=
CAN USE IT CORRECTLY IN CONTEXT
```

**Downstream delta:** explicitly distinguish syntactic, semantic, and pragmatic compatibility.

---

## Russian — MATERIAL_DELTA

Native discovery terms:

- `интероперабельность`
- `модель предметной области`
- `онтология`
- `графы знаний`
- `декларативное проектирование`
- `жизненный цикл`

Key source:

Охтилев et al., **«Концепция инженерии знаний в задачах обеспечения интероперабельности АСУ и информационных систем на основе интеллектуальных технологий»** (2023).

The paper frames interoperability around declarative design, conceptual domain models, knowledge graphs, and lifecycle-specific ontologies.

**New implication:** a translator manifest can become part of a declarative knowledge layer spanning supported concepts, mappings, conformance, versions, and lifecycle/deprecation.

---

## German — MATERIAL_DELTA

Native discovery terms:

- `semantische Interoperabilität`
- `Ontologie`
- `Kommunikationsstandards`
- `Mediator`
- `Mapping`

Key source:

University of Regensburg dissertation, **“Entwicklung einer ontologiebasierten Architektur zur Sicherung semantischer Interoperabilität zwischen Kommunikationsstandards im Gesundheitswesen”** (2011).

A particularly relevant result is that arbitrary communication standards should not all be mapped directly to each other; a mediating application ontology can serve as the middle layer.

**New implication:** this is a clean anti-N² precedent.

```text
game A -> semantic mediator <- game B
game C -> semantic mediator <- game D
```

---

## French — MATERIAL_DELTA

Native discovery terms:

- `interopérabilité`
- `simulation distribuée`
- `ontologie`
- `fédération`
- `ontologies éphémères`

French enterprise-interoperability research around HLA and ontologies includes work on contextual or “ephemeral” ontologies.

**New implication:** not every mapping needs to become a permanent global protocol concept. Some mappings can be scenario-specific, session-specific, version-bound, and discarded after use while retaining provenance/test evidence.

---

## Portuguese / Brazil — MATERIAL_DELTA

Native discovery terms:

- `interoperabilidade semântica`
- `organização do conhecimento`
- `ontologia`
- `domínios de conhecimento`

Key source:

**“Interoperabilidade Semântica: uma análise sob a perspectiva da abordagem ontológica de Quine”**, Perspectivas em Ciência da Informação 28 (2023).

**New implication:** a semantic model is not automatically neutral merely because it calls itself an ontology. Preserve source-game concepts and mapping provenance rather than forcing all native semantics to disappear into one supposedly objective model.

---

## Korean — PARTIAL / NO MATERIAL DELTA IN THIS BOUNDED PASS

Native queries around `상호운용성`, `분산 시뮬레이션`, `HLA`, and game-engine integration did not yield an accessible source that materially changed the architecture in this pass.

This is a search/access limitation, **not** evidence that Korean literature lacks relevant work.

---

## Cross-language synthesis

The multilingual sources converge on a more precise architecture than “make a universal protocol”:

```text
stable common transport/session contract
        +
explicit object/event/profile schemas
        +
modular domain semantics
        +
adapter/federate boundaries
        +
mapping provenance
        +
developer tooling / code generation
        +
conformance tests
```

The strongest new lesson is that **adapter generation and semantic mediation are both old problems**. AI can accelerate them, but should not erase the explicit contracts that make generated adapters independently testable.
