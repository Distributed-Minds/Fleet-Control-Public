# From Shared Meaning to Verifiable Interoperability

## A technical response to Signet 2: intents, archetypes and translation profiles

**Status:** unaffiliated public draft / technical response  
**Date:** 2026-10-05  
**Response target:** Signet Protocol, Signet 2 draft proposal v0.1  
**Observed upstream state:** proposal introduced at kian-cx/signetprotocol@490dfa9423841a45f2917d8d013e010ca0eb5548; expanded Signet Forge documentation checked through kian-cx/signetprotocol@2ddb136ee941705d3e1c020eaddad93be65026f2  
**Authors:** Distributed Minds / Fleet-Control research contribution

> This is an independent technical response. It is not official Signet Protocol documentation and does not imply endorsement by Signet's maintainers.

---

## Abstract

Signet 2 makes the right architectural move. Instead of translating every game directly into every other game, each game translates local behavior into a shared semantic layer and translates shared state back into local presentation. Player actions become **intents**; neutral world concepts become **archetypes**; clients declare **capabilities**; obvious mappings are calibrated; harder mappings are selected from closed candidate sets; accepted choices are pinned into **translation profiles**; and model inference remains outside the authoritative per-tick simulation path.

That architecture can reduce translator-authoring topology from approximately O(n²) pairwise integrations toward O(n) game-to-hub integrations. It does not, by itself, make interoperability correct, secure, current, or independently reproducible. The shared hub moves the hardest assumptions into a common surface: what an intent means, what an archetype guarantees, which capabilities are required, who may change a mapping, what evidence made a mapping acceptable, what authority an adapter receives locally, and when old evidence must stop being presented as current.

Our response is therefore constructive: keep the Signet 2 scaling architecture, but make its boundaries explicit. We propose a **small versioned semantic core plus profiles**, an explicit negotiated session contract, typed translation-profile authority, publisher baselines plus participant-owned overlays, evidence-bearing mapping locks with drift semantics, mechanically generated adapter contract glue, explicit target-specific hooks, abstention for resolver misses, integration-specific confinement, local-first Forge behavior, explicit training-data contribution, independent interoperability evidence, and separate governance and revocation authorities.

Two implementation experiments sharpen this argument. A Godot 4.7.2 GDExtension host harness validated a generated native boundary without claiming full Godot runtime validation. A second experiment against Signet's documented Minecraft RCON integration falsified an overly generic adapter interface: the code-generation mechanism worked, but the logical interface could not express the target's actual responsibilities. Revising the descriptor around explicit HOST, IMPORTER, WORLD, INPUT, PRESENTATION, and AUTHORITY roles produced a better target contract. This is the kind of falsification an interoperability specification should invite.

The compact thesis is:

~~~text
deterministic core
+ adaptive edge
+ explicit trust boundary
+ reproducible evidence
~~~

Signet 2 is already pointed in that direction. The next step is to turn shared meaning from an architectural idea into an independently implementable, testable, governable contract.

---

## 1. Why Signet 2 is the right scaling direction

The proposal starts from a real problem. If every game translator understands every peer directly, the number of bilateral integration edges grows rapidly.

For n games, a pairwise topology has approximately:

~~~text
M_pair(n) = n(n - 1) / 2
~~~

A semantic hub instead asks each game to implement one path into and out of the shared model:

~~~text
M_hub(n) = n
~~~

This is the same broad anti-N² pattern used in other interoperability domains: local systems keep their native representation while a canonical layer carries shared meaning.

The important part of Signet 2 is not merely the graph reduction. It also places several responsibilities on the correct side of the boundary.

### 1.1 Preserve intent versus effect

A translator should be able to say what a player wants to do without claiming that the requested effect already happened.

These are different facts:

~~~text
fire intent       != shot emitted
jump intent       != authoritative trajectory
use intent        != interaction completed
damage proposal   != damage applied
teleport request  != authoritative pose update
~~~

The authoritative server should remain responsible for applying the shared rules and producing authoritative state and events.

### 1.2 Preserve local presentation

An archetype can carry shared functional meaning while each game chooses a local representation.

That is the correct layer boundary for:

- meshes and sprites;
- sounds;
- animations;
- HUD and camera behavior;
- local item presentation;
- other game-specific affordances that do not alter shared authority.

### 1.3 Preserve calibration before inference

Signet 2's strongest adaptive-design decision is that obvious configuration should not become an AI problem.

A player can identify which local control means forward. An adapter can measure or declare some local behavior. A person can choose among presentation candidates.

The correct default is:

> **Measure or ask when possible; infer only when necessary.**

### 1.4 Preserve closed candidate ranking

A resolver that ranks a finite set of known candidates is substantially easier to constrain and evaluate than free-form generation.

Closed candidates make it possible to test:

- whether the correct candidate was present;
- whether the resolver selected it;
- whether no candidate was acceptable;
- whether candidate ordering biased the result;
- whether a model remained reliable outside its intended domain.

### 1.5 Preserve pinned runtime decisions

Simulation-affecting choices should become deterministic before they enter an authoritative session path.

The model may change. The human reviewer may change. Forge may disappear. The runtime decision should remain an inspectable artifact.

### 1.6 Preserve Forge/runtime separation

Signet Protocol should remain usable if:

- Forge is unavailable;
- no model is installed;
- a model checkpoint changes;
- a hosted resolver is offline;
- the community replaces the ranking backend.

Forge can evolve rapidly. The runtime contract should remain stable.

### 1.7 Preserve the official integration boundary

The proposal explicitly avoids online-game code injection and anti-cheat bypass as an official integration path.

That is a useful project policy. It keeps early engineering focused on open engines, permitted mod/plugin APIs, controlled servers, documented external APIs, and reproducible manual declarations.

---

## 2. Shared meaning must become an explicit semantic contract

The central Signet 2 idea is "game to meaning to game."

The next question is: what makes that meaning identical across independent implementations?

### 2.1 Shared labels are not automatically shared semantics

A string such as **fire** is not a complete contract.

Independent teams can reasonably interpret it as:

- trigger pressed;
- trigger held;
- request to attack;
- accepted attack;
- projectile spawned;
- hitscan resolved;
- authoritative shot event.

Likewise, **weapon.ranged** can describe a useful functional class while leaving authoritative damage, range, cadence, ammunition, equip timing, and hit resolution unspecified.

Therefore:

~~~text
SHARED_LABELS != SHARED_SEMANTICS
~~~

A simulation-grade semantic item should bind at least:

~~~text
stable semantic identity
+ semantic version
+ normative definition
+ profile membership
+ conformance assertions
~~~

A human-readable label can remain short. The identity should not depend on the label remaining unchanged.

### 2.2 Keep the universal core small

A flat global vocabulary will eventually become a monolith.

Movement, jumping, weapons, pickups, inventories, cards, cities, turn phases, colonies, vehicles, rhythm timing, and strategic diplomacy do not belong in one mandatory universal schema.

A stronger architecture is:

~~~text
core@1
  identity
  session/participant identity
  capability negotiation
  ordering/causality
  authority references
  typed incompatibility

profile/spatial-3d@1
profile/entity-lifecycle@1
profile/fps-combat@1
profile/inventory@1
profile/voxel-world@1
profile/turn-based@1
profile/card-game@1
...
~~~

A session activates only what it needs.

This also gives Signet a practical falsification test: a non-spatial or non-combat game should be able to implement the core without inventing dummy movement, weapon, or health fields.

### 2.3 Intent is not authoritative effect

The semantic contract should explicitly distinguish requests, accepted actions, applied effects, and resulting events.

A useful boundary is:

~~~text
local input
-> semantic intent/request
-> authoritative acceptance
-> authoritative transition
-> authoritative event/state
-> local presentation
~~~

That distinction improves replay, debugging, security analysis, and interoperability testing at the same time.

### 2.4 Archetype identity is not the ruleset

The server can tell clients that an entity has the role **weapon.ranged** while separately owning the authoritative parameters and behavior.

For example:

~~~yaml
entity:
  archetype: weapon.ranged@1

  authoritative:
    damage_model: ...
    range_m: ...
    cadence: ...
    ammunition: ...
    hit_resolution: ...

  local_presentation:
    chosen_item: minecraft:bow
~~~

A Minecraft bow can be the local presentation without importing native Minecraft bow mechanics into shared authority.

### 2.5 Capability declaration is not session negotiation

Signet 2 currently says the server uses the common subset.

A set intersection is not enough once capabilities can be required, optional, parameterized, approximated, or unavailable.

Suppose a ruleset requires jumping:

- Game A supports jump natively.
- Game B cannot emit jump.
- Game C can represent it only through an approved deterministic fallback.

"Remove jump from the common subset" is only one possible policy. The correct result might instead be:

~~~text
SUPPORTED
SUPPORTED_WITH_FALLBACK
OBSERVE_ONLY
INCOMPATIBLE
~~~

A session therefore needs an explicit negotiated semantic contract.

It should record:

- semantic core/profile versions;
- exact semantic definition identities;
- supported capabilities;
- required capabilities;
- activated capabilities;
- accepted fallbacks;
- rejected capabilities;
- authority model;
- ruleset identity;
- deterministic admission result.

A capability offer says "I can do X."

The negotiated contract says "this session selected X under these exact conditions."

### 2.6 Session policy should choose among semantically acceptable models

The L1 continuation now makes one part of that negotiation executable.

If several ordering models or visibility policies preserve every activated profile's semantics, implementations must not select among them through enumeration order or an implicit local default.

The useful split is:

~~~text
profile:
  which semantic models are acceptable?

session:
  among acceptable models, what do we prefer?

negotiation:
  select the first session preference accepted by every activated profile
~~~

For example:

~~~text
session ordering preferences:
  fixed-tick@1
  event-sequence@1
  turn-sequence@1

profile A accepts:
  fixed-tick@1
  event-sequence@1

profile B accepts:
  event-sequence@1
  turn-sequence@1

selected:
  event-sequence@1
~~~

The important design point is that reusable semantic profiles define **validity**, while the session/ruleset/operator defines **deployment preference**.

This prevents a semantically arbitrary lexicographic rule or implementation-specific enumeration order from becoming hidden protocol policy.

The current L1 executable negotiation vector reproduces this rule in both Python and JavaScript and also applies it to visibility-policy selection.

### 2.7 Optional profiles also need deterministic activation

"Optional" cannot mean "enable it if this implementation happens to have it."

Activating an optional semantic profile can:

- add required dependencies;
- collide with an existing concept;
- narrow the valid ordering-model set;
- narrow the valid visibility-policy set;
- conflict with an earlier optional feature.

The current L1 candidate rule is deliberately simple:

1. compose the required baseline;
2. iterate optional profiles in an explicit session preference order;
3. tentatively add one candidate;
4. resolve its dependencies and rerun full composition and policy selection;
5. keep it if the complete contract remains valid;
6. otherwise skip it with an exact reason and continue.

Earlier accepted optionals remain preferred.

This is greedy by declared session policy, not an attempt to maximize the number of enabled features.

A published executable vector currently demonstrates:

~~~text
activate optional-event@1

skip optional-fixed@1
  reason = ORDERING_MODEL_CONFLICT

skip missing@1
  reason = REQUIRED_PROFILE_UNSUPPORTED

selected ordering:
  event-sequence@1
~~~

This makes graceful degradation inspectable and deterministic.

Two peers can compare the exact activated and skipped semantic modules instead of independently deriving what "optional support" means.

### 2.8 Fallback must be typed

Ignore and approximate are acceptable for some presentation failures. They can be dangerous for authoritative gameplay.

Examples:

~~~text
missing cosmetic effect
-> IGNORE may be safe

missing jump in a no-jump ruleset
-> irrelevant

missing jump in a jump-required ruleset
-> likely INCOMPATIBLE unless an approved fallback exists

weapon.ranged -> different local item
-> presentation substitution

fire intent -> use intent
-> not a generic safe approximation
~~~

A simulation-affecting fallback should identify:

- source concept;
- target concept;
- fallback class;
- information loss;
- policy permission;
- conformance evidence.

Unknown or unsupported is often better than invented precision.

---

## 3. Translation profiles need typed authority, ownership, evidence, and invalidation

Signet 2's lock-file analogy is useful. The lock file should lock more than a choice.

### 3.1 A profile contains different authority classes

At minimum, distinguish:

| Profile class | Primary authority | Player override? | Shared-state impact |
|---|---|---:|---:|
| Input binding | participant + adapter | yes | indirect |
| Motion observation | measured adapter evidence | not arbitrary | should not redefine server physics |
| Semantic mapping | translator/profile authority | not arbitrary in authoritative play | yes |
| Appearance mapping | participant/client | yes | no |
| Session policy | server/session | no unilateral client override | yes |

One physical file may contain these sections. The schema should not imply they have the same owner or precedence.

A player's decision that **weapon.ranged** should look like a bow is not equivalent to a player's decision that a local event means the authoritative **fire** intent.

### 3.2 Split publisher baseline from participant overlay

Contents and ownership are separate questions.

A safer model is:

~~~text
publisher_profile
  immutable/versioned baseline shipped by translator publisher

participant_overlay
  local participant-owned bindings, pins, and explicit overrides

session_resolution
  derived effective profile for one exact dependency cut
~~~

A package update may change publisher defaults.

It should not silently rewrite participant-owned decisions.

If a participant pinned:

~~~text
weapon.ranged -> minecraft:bow
~~~

and the next publisher profile prefers:

~~~text
weapon.ranged -> minecraft:crossbow
~~~

the publisher baseline may change while the participant overlay remains intact.

If the new semantic revision makes the old overlay invalid, the correct outcome is:

~~~text
RECONFIRM_REQUIRED
~~~

not silent migration.

### 3.3 Pinned choice is not correct choice

A pinned decision guarantees repeatability.

It does not establish:

- the exact source-game version;
- the semantic definition intended;
- whether the relation is exact or approximate;
- what information is lost;
- what transform was applied;
- what candidate set was used;
- what tests passed;
- whether any dependency changed later.

Therefore:

~~~text
PINNED_CHOICE != CORRECT_CHOICE
~~~

A simulation-grade mapping should carry provenance and evidence.

A minimal logical shape is:

~~~yaml
source:
  game_id: ...
  game_version_or_fingerprint: ...
  integration_mode: ...
  integration_surface_digest: ...

target:
  semantic_id: ...
  semantic_profile_digest: ...

relation:
  type: exact | close | narrowing | broadening | transform | approximate
  lossy: true | false

transform:
  digest: ...

proposal:
  method: lookup | rules | human | model_assisted
  resolver_identity: ...
  candidate_set_digest: ...

acceptance:
  method: ...
  replay_set_digest: ...
  conformance_suite_digest: ...

invalidates_on:
  - game_or_surface_change
  - semantic_profile_change
  - transform_change
  - candidate_basis_change
~~~

### 3.4 Package bytes are not the effective profile

A profile-only change can alter behavior without changing executable bytes.

A useful compatibility subject includes:

~~~text
translator package digest
+ publisher profile digest
+ participant overlay digest
+ effective profile digest
+ game version/fingerprint
+ protocol/profile revisions
+ calibration revision
~~~

Therefore:

~~~text
PACKAGE_BYTES != EFFECTIVE_PROFILE
~~~

### 3.5 Compatibility evidence needs drift semantics

A deterministic mapping can become stale.

Use explicit evidence state:

~~~text
VALID
  |
dependency changed
  v
SUSPECT
  |
targeted replay / conformance
  v
REVALIDATING
  |             |
 pass           fail / unknown
  v             v
VALID(new)     INVALID
~~~

A launcher or directory should not continue showing an old PASS as current while its dependency identity changed.

---

## 4. Adapter engineering: generate contract glue, keep target hooks explicit

A scalable adapter ecosystem should automate repetitive interface work without pretending that game-specific behavior is generic.

### 4.1 The engineering target

A useful pipeline is:

~~~text
shared semantic/profile contract
        |
target role descriptor
        |
generated contract glue
        |
thin explicit target shim
        |
engine / mod API / gateway / reimplementation
~~~

Generated artifacts may include:

- Rust traits;
- C headers and ABI wrappers;
- C# bindings;
- TypeScript/tool types;
- operation manifests;
- profile schemas;
- conformance/replay skeletons.

Handwritten code should remain responsible for irreducibly local facts such as:

- engine callbacks;
- mod/plugin APIs;
- RCON commands;
- coordinate systems;
- unit conversions;
- local data formats;
- lifecycle hooks;
- version-specific quirks.

The goal is not "AI writes the translator."

The goal is:

> **Generate stable contract shape; handwrite visible local behavior.**

### 4.2 Observed experiment: typed profile drift

The L2 prototype defined five entry classes:

- INPUT_BINDING;
- MOTION_OBSERVATION;
- SEMANTIC_MAPPING;
- APPEARANCE_MAPPING;
- SESSION_POLICY.

The example semantic mapping carried source and semantic fingerprints plus replay/conformance evidence.

After changing the integration-surface fingerprint and game version, the observed checker result was:

~~~text
typed profile: PASS
semantic mapping fire-primary: SUSPECT
changed dependency: integration_surface_digest
changed dependency: game_version
~~~

This demonstrates one narrow property:

> A mapping can remain pinned while its compatibility evidence becomes stale.

### 4.3 Observed experiment: deterministic glue generation

A small adapter-shim descriptor generated C, Rust, C#, and TypeScript surfaces.

Two clean runs produced byte-identical generated outputs. The C surface was syntax-checked with cc and the TypeScript surface with tsc. Rust and C# compiler validation were not available in that pass and are not claimed.

The result supports mechanical generation of the stable interface boundary, not automatic generation of game behavior.

### 4.4 Observed experiment: Godot 4.7.2 GDExtension

A native shared library implemented the generated C shim and exposed a GDExtension initialization symbol using the Godot 4.7.2 interface shape.

Observed:

~~~text
godot-entry-init: PASS
godot-version-discovery: 0x040702
generated-shim-surface: PASS
~~~

This is deliberately classified as:

~~~text
ABI_HOST_HARNESS_PASS
~~~

not:

~~~text
GODOT_RUNTIME_PASS
~~~

The Godot executable itself was not launched in that experiment.

That distinction matters. Evidence should be named at the level actually observed.

### 4.5 Observed experiment: Minecraft RCON gateway falsifies the first interface

Signet's documented Minecraft path uses an unmodified client connected to a controlled server, with a gateway using RCON-like command authority.

The documented responsibilities include:

- world construction;
- player/actor observation;
- selected input observation;
- teleport/reconciliation;
- damage;
- death/kill;
- command execution.

A representative Source-RCON framing fixture passed for five commands.

But the first generic adapter shim could not express the target:

~~~text
v0-shim-coverage: NO=5, PARTIAL=2
v0-shim-sufficiency: FAIL_EXPECTED
~~~

That is a valuable result.

The correct response was not to stretch one generic function until it meant everything. The descriptor was revised around explicit roles:

~~~text
HOST
IMPORTER
WORLD
INPUT
PRESENTATION
AUTHORITY
~~~

The revised Minecraft descriptor mapped all seven documented responsibility classes:

~~~text
minecraft-responsibility-coverage: 7/7
minecraft-role-contract: PASS
~~~

The generic Godot descriptor, by contrast, made only a host/version claim and no fabricated game-level gameplay claims.

The lesson is:

> **Real targets should be allowed to falsify the descriptor model before the descriptor becomes a standard.**

---

## 5. Calibration and resolver behavior: measure or ask first, infer only when necessary

### 5.1 Calibration should be an explicit adapter contract

A wizard cannot measure something the integration cannot legitimately observe.

A calibration probe should therefore declare:

- which intent or property it is measuring;
- required preconditions;
- observable inputs;
- measurement method;
- units;
- repetition/sample policy;
- integration surface;
- uncertainty or unavailable state.

Example:

~~~yaml
probe:
  id: movement.forward.speed
  intent: move_forward
  preconditions:
    - grounded

  observables:
    - local_position
    - elapsed_time

  measurement:
    unit: m/s
    method: median_steady_state

  evidence:
    integration_surface_digest: ...
~~~

If the game does not expose the necessary quantity, the adapter should record:

~~~text
UNKNOWN
UNSUPPORTED
OBSERVE_ONLY
~~~

rather than fabricate an exact number.

### 5.2 Motion calibration must not redefine canonical physics

If the server owns authoritative physics, the motion profile is evidence and local transduction state, not a second ruleset.

It can support:

- input normalization;
- prediction/presentation tuning;
- compatibility checks;
- local movement envelopes;
- fidelity measurements.

It should not silently override shared walk speed, jump physics, damage, or other authoritative rules.

### 5.3 Closed candidates still need NO_MATCH

A finite candidate list prevents free invention.

It does not guarantee that the list contains a correct answer.

Resolver calls should permit:

~~~text
NO_MATCH
ABSTAIN
~~~

where appropriate.

Otherwise the system forces a wrong choice when the correct action is refusal.

### 5.4 Scores are not semantic proof

A ranked list is inspectable evidence about the resolver's preference.

It is not proof that the top candidate is semantically correct.

For simulation-affecting mappings:

~~~text
proposal
-> human/maintainer acceptance
-> deterministic profile record
-> replay/conformance evidence
-> session admission
~~~

A resolver remains advisory.

~~~text
MODEL_SUGGESTION != AUTHORITY
~~~

### 5.5 Observed resolver pilot: candidate ordering matters

A six-case seed harness used mappings explicitly present in the Signet 2 examples and compared three simple mechanisms under base order, candidate rotations, and correct-candidate removal.

Observed:

~~~text
lookup:
  base_accuracy=1.000
  no_match=1.000
  order_stability=1.000

lexical:
  base_accuracy=1.000
  no_match=0.000
  order_stability=0.167

first-candidate:
  base_accuracy=1.000
  no_match=0.000
  order_stability=0.000
~~~

The result is not a model benchmark.

It shows that original-order top-1 accuracy can be misleading when examples place the intended answer first.

The planned 30-case resolver experiment should therefore report at least:

- top-1 accuracy;
- candidate-order stability;
- NO_MATCH/abstention behavior;
- candidate-set provenance;
- human override rate;
- severe-error rate.

Only after deterministic/simple baselines are understood should larger model dependence become architectural.

---

## 6. Public translators are supply-chain software

A translator ecosystem adds a second question beyond semantic correctness:

> **What authority does this package receive on a stranger's machine or controlled server?**

A translator, launcher, gateway, or setup tool may need to:

- inspect a game installation;
- read official registries or configuration;
- create generated/staged files;
- start a game or dedicated server;
- communicate with a local service;
- connect to a Signet session;
- update itself or dependencies.

Those are real privileges.

### 6.1 Keep trust claims separate

The ecosystem should avoid one overloaded "trusted" or "verified" badge.

These are different claims:

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
CURRENT_AUTHORIZATION != SAFE_ARTIFACT
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
COMPATIBILITY         != LEGAL_OR_POLICY_PERMISSION
~~~

A signed release can later be revoked.

A reproducible build can reproducibly contain a vulnerability.

A conformant translator can ask for excessive filesystem authority.

A well-sandboxed translator can still be semantically incompatible.

### 6.2 Broker symbolic permissions

Portable manifests should prefer symbolic resources instead of arbitrary package-selected absolute paths.

Examples:

~~~text
GAME_INSTALL_READ
GAME_INSTALL_WRITE_STAGED
TRANSLATOR_WORKDIR_RW
SIGNET_SESSION_NETWORK
CONTROLLED_SERVER_PROCESS
USER_SELECTED_FILE_READ
~~~

A launcher or broker resolves those resources into platform-specific access after user or policy approval.

A useful enforcement pipeline is:

~~~text
target role contract
-> declared authority/permissions
-> launcher/broker resolution
-> platform enforcement
-> measured enforcement evidence
~~~

The descriptor can inform permission generation.

It cannot itself enforce permissions.

Therefore:

~~~text
ROLE_CONTRACT != SANDBOX_POLICY
~~~

### 6.3 Integration mode is part of the trust subject

The Godot and Minecraft experiments show why.

#### Godot GDExtension

A normal native GDExtension executes in the host application's process.

Its approximate confinement class is:

~~~text
HOST_PROCESS_CODE
~~~

It may be reviewed, signed, reproducible, narrow, and constrained by API design.

It is not a separately kernel-sandboxed worker.

#### Minecraft controlled-server gateway

The documented RCON gateway can exercise controlled-server authority including world mutation, actor reposition, damage, death, observation, and commands.

That is materially different from a read-only helper.

Its trust record should expose the actual authority rather than hide it behind the label "gateway."

### 6.4 RCON credentials are a separate secret boundary

A gateway that requires an RCON credential should not inherit every other launcher secret.

Prefer:

- dedicated controlled-server credentials;
- minimum necessary scope;
- local secret storage;
- redaction from logs and crash reports;
- rotation where practical;
- process-bound delivery;
- broker mediation where feasible.

### 6.5 Signing, provenance, reproducibility, and update currentness solve different problems

A public ecosystem can compose mechanisms inspired by:

- TUF-like update metadata for currentness, rollback/freeze resistance, delegation, and revocation;
- Sigstore-style identity and transparency;
- SLSA-style build provenance;
- independent reproducibility;
- platform confinement;
- conformance testing;
- compatibility evidence.

Using one does not imply the guarantees of the others.

---

## 7. Forge, privacy, and training-data contribution

### 7.1 Forge should be local-first

A good default separation is:

~~~text
runtime play:
  profile lookup only

calibration/review:
  local where practical

Forge development:
  local by default

hosted resolver:
  optional, explicit export contract

training contribution:
  separate explicit action
~~~

A remote resolver should not inherit everything the local tool can observe.

### 7.2 Remote execution creates an export boundary

A hosted resolver should declare what leaves the machine.

For example:

~~~yaml
remote_resolver_export:
  game_identifier: true
  game_version: optional
  semantic_identifier: true
  candidate_identifiers: true

  local_paths: false
  installed_game_inventory: false
  raw_input_events: false
  save_files: false
  game_assets: false
  usernames: false
~~~

The exact schema can evolve.

The boundary should be visible and testable.

### 7.3 Local approval is not training consent

When a person accepts a mapping locally, they answer:

> Should my translator use this choice?

That is not automatically the same as:

> May this record be exported, licensed, retained, and used as shared model-training data?

Therefore:

~~~text
LOCAL_APPROVAL != DATASET_CONTRIBUTION
~~~

Training contribution should be explicit.

A contributed record should retain:

- task scope;
- game/adapter basis;
- semantic subject;
- candidate-set revision;
- selected candidate;
- provenance;
- contribution license;
- invalidation/supersession state.

### 7.4 Invalidated mappings need dataset lineage

If a mapping is later disproved after a game or semantic change, the training system should not keep treating it as current positive evidence.

Useful states include:

~~~text
accepted_for_local_profile
accepted_for_publisher_profile
eligible_for_dataset
licensed_for_training
validated_training_example
invalidated_training_example
~~~

These states are related but not interchangeable.

---

## 8. Proving interoperability: conformance, pairwise execution, reproduction, and freshness

A shared hub reduces implementation duplication. It does not eliminate validation.

### 8.1 Mapping complexity and validation complexity are different

For n games, the shared hub can reduce authoring to roughly:

~~~text
M_hub(n) = O(n)
~~~

If an ecosystem wants direct empirical evidence for every unordered pair, that evidence set can still be:

~~~text
V_all_pairs(n) = n(n - 1) / 2 = O(n²)
~~~

Therefore:

~~~text
LINEAR_MAPPING_GROWTH != LINEAR_VALIDATION_DEMAND
~~~

This is not a defect in Signet 2.

It means the architecture removes quadratic translator duplication without automatically removing quadratic evidence demand.

Strong conformance may eventually justify sampling for some bounded claims. That reduction has to be earned by precise semantics and observed independent agreement.

### 8.2 Self-conformance is not pairwise interoperability

The following implication is not automatically valid:

~~~text
A passes self-conformance
+
B passes self-conformance
=
A and B have demonstrated interoperability
~~~

A useful evidence ladder is:

**V0 — structural validity**  
Profiles/messages parse and required identities resolve.

**V1 — behavioral conformance**  
One implementation satisfies named normative assertions.

**V2 — independent implementation**  
Another author/team independently implements the contract.

**V3 — pairwise interoperability**  
Two implementations execute the same negotiated profile and satisfy authoritative fixtures.

**V4 — independent reproduction**  
An outside party reproduces the claim from published artifacts.

**V5 — operational freshness**  
The evidence remains current after relevant dependency changes and actual use.

Thus:

~~~text
SELF_CONFORMANCE
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= ECOSYSTEM_ADOPTION
~~~

### 8.3 Compatibility must be scoped

Avoid one global "supports Signet" Boolean.

A compatibility record should identify:

- protocol/core revision;
- semantic profile revision;
- game and translator identity;
- effective profile digest;
- integration mode;
- selected capability contract;
- test suite revision;
- environment;
- pass/fail/skip/fallback counts;
- known failures;
- freshness state;
- reproduction state.

### 8.4 Preserve negative evidence

Pairwise matrices should publish denominators and failures, not demonstrations only.

A failed reproduction should not disappear when a later version passes.

A correct session rejection is also evidence: graceful refusal is part of interoperability behavior.

### 8.5 Profile composition itself needs tests

If publisher baseline, participant overlay, session policy, semantic profiles, and fallbacks compose into an effective profile, the composition algorithm becomes protocol-relevant.

Test at least:

- deterministic composition;
- conflicting entries;
- stale overlay entries;
- profile version mismatch;
- participant pin preservation;
- required-capability rejection;
- profile-only behavior changes;
- deterministic ordering/visibility pre-selection;
- deterministic optional-profile activation and exact skip reasons.

The L1 lane now contains executable evidence for the last two points.

Its current Python and JavaScript reference implementations reproduce:

~~~text
PASS: HASH-VECTOR-001
PASS: COMP-VECTOR-001
PASS: NEGOTIATION-VECTOR-001
PASS: OPTIONAL-NEGOTIATION-VECTOR-001
PASS: CONTRACT-VECTOR-001
PASS: 5 additional composition vectors
PASS: 9 negative composition vectors
~~~

The optional-profile fixture also caught an error in its own originally published expected canonical order before the result was recorded. That is a useful specification-engineering result: canonical executable vectors can falsify the research artifact itself, not merely downstream implementations.

This evidence remains candidate L1 research, not implemented Signet runtime behavior.

---

## 9. Governance, revocation, and legal/policy representation

Signet's current governance can remain lightweight while the project is small. The system should still name different powers separately.

### 9.1 Separate governance authorities

A mature ecosystem contains at least:

**Protocol governance**
- wire contract;
- intent vocabulary;
- archetype catalog;
- profile/version rules.

**Translator/profile governance**
- translator implementation;
- integration surface;
- publisher baseline;
- palette;
- calibration logic.

**Participant authority**
- local bindings;
- participant pins;
- local privacy choices;
- hosted-Forge opt-in.

**Package-directory governance**
- listings;
- namespace authorization;
- evidence display;
- compatibility yanks;
- directory policy.

**Update-root authority**
- trust roots;
- publisher delegation;
- key-compromise recovery.

**Security incident authority**
- emergency quarantine;
- security revocation;
- advisory publication;
- restoration criteria.

**Forge/dataset/model governance**
- dataset admission;
- contribution licensing;
- example invalidation;
- model release identity.

**Conformance authority**
- test-suite revisions;
- PASS semantics;
- reproducibility requirements.

Initially the same people may occupy several roles.

The artifacts should not collapse them into the phrase "maintainer approved."

### 9.2 Community profile PRs are behavioral supply-chain changes

A profile pull request that changes a simulation-affecting mapping or calibration assumption can change behavior even when code bytes remain identical.

Release tooling should identify:

- exact profile subject;
- semantic diff;
- authority class;
- whether shared behavior can change;
- evidence invalidated;
- replay/conformance required;
- resulting immutable profile digest.

### 9.3 Lifecycle states need more than latest version

Useful states include:

~~~text
ACTIVE
SUPERSEDED
DEPRECATED
YANKED_COMPATIBILITY
COMPATIBILITY_INVALIDATED
SECURITY_REVOKED
PUBLISHER_REVOKED
LEGAL_OR_POLICY_BLOCKED
~~~

A compatibility failure should not be mislabeled as malware.

A security revocation should not be treated as merely "old."

A historically valid signature can remain valid while current authorization becomes revoked.

~~~text
VALID_SIGNATURE != CURRENT_AUTHORIZATION
~~~

### 9.4 Official integration policy is not a universal legal conclusion

Signet 2's conservative official scope is useful:

- open-source engines;
- permitted mods/plugin APIs;
- controlled servers;
- no online-game injection;
- no anti-cheat bypass.

Keep that policy.

Do not convert it into:

~~~text
outside_supported_modes == illegal
~~~

Legal analysis depends on jurisdiction, technical action, contractual context, circumvention, redistribution, privacy, and other facts.

The package/directory should instead represent scoped technical facts and an official project disposition such as:

~~~text
ALLOW
ALLOW_WITH_USER_WARNING
REVIEW_REQUIRED
EXCLUDE_OFFICIAL_DIRECTORY
UNKNOWN
~~~

That is a governance decision for a declared scope, not a universal court judgment.

---

## 10. Direct answers to Signet 2's open questions

### 10.1 What is the minimum a motion profile must capture?

There is no one universal list.

The minimum is profile/ruleset dependent and should include only what is required for:

- input normalization;
- local fidelity/prediction;
- compatibility evidence;
- declared movement limits.

For a spatial/FPS adapter, useful fields can include:

- coordinate frame and handedness;
- units;
- input ranges/deadzones where relevant;
- yaw/pitch mapping;
- local camera/eye offset;
- walk/run/strafe observations;
- jump envelope where observable;
- step/crouch envelope where relevant;
- measurement conditions and source.

Do not let local calibration redefine authoritative server physics.

### 10.2 What if the game does not expose position?

Use the strongest legitimate observable source available:

1. open-engine state;
2. permitted mod/plugin API;
3. controlled-server telemetry;
4. documented external API;
5. reproducible manual declaration where appropriate.

If the quantity cannot be observed with sufficient confidence:

~~~text
UNKNOWN
UNSUPPORTED
OBSERVE_ONLY
~~~

is better than fabricated exactness.

### 10.3 Who governs the vocabulary and catalog?

Use protocol governance with:

- stable semantic IDs;
- a small conservative core;
- separately versioned profiles/namespaces;
- proposal/review;
- conformance vectors;
- deprecation/supersession;
- migration rules.

Avoid one giant universal flat vocabulary.

### 10.4 Where should the resolver run?

Prefer development/calibration time, local by default.

A hosted resolver may be optional if it has an explicit privacy/export contract.

The resolver should never be required for deterministic per-tick play.

### 10.5 How should conflicting community profiles resolve?

First determine whether they describe the same subject.

Use:

~~~text
effective profile
=
versioned publisher baseline
+
participant-owned overlay
+
session-compatible derivation
~~~

Allow competing publisher/community alternatives.

Compare them with evidence and semantic diff, not silent popularity merging.

A simulation-affecting profile change should receive a new identity and stale the old compatibility evidence.

---

## 11. One falsifiable joint experiment

The next milestone should not be "support every game."

It should be a small end-to-end experiment that can fail clearly.

### Step 1 — Freeze a tiny semantic profile

Use approximately:

- 5–8 intents;
- 4–6 archetypes;
- one movement/body model;
- one damage/death lifecycle;
- explicit capability negotiation.

### Step 2 — Publish normative assertions

Each semantic item should have:

- stable identity;
- definition;
- examples/non-examples;
- conformance assertions;
- version.

### Step 3 — Implement two materially different integration modes

For example:

- one open-engine/native integration;
- one controlled-server gateway.

The point is to stress different authority and observability boundaries.

### Step 4 — Generate repetitive contract glue

Use a versioned role/operation descriptor to generate bindings, manifests, and test skeletons.

Keep target-specific hooks explicit.

### Step 5 — Calibrate without AI

Measure or ask for:

- input bindings;
- minimal motion observations;
- declared capabilities;
- appearance selections.

### Step 6 — Produce evidence-bearing package/profile records

Bind:

- exact game/integration identity;
- semantic profile digest;
- transform digest;
- effective profile digest;
- permissions/confinement class;
- replay/conformance evidence.

### Step 7 — Run replay and pairwise interoperability tests

Exercise:

- request/accept/apply distinctions;
- duplicate/reordered input;
- attack/damage/death lifecycle;
- capability mismatch;
- accepted fallback;
- correct session rejection;
- archetype presentation.

### Step 8 — Record integration-specific authority

Expected qualitative distinction:

~~~text
native engine extension:
  HOST_PROCESS_CODE

controlled-server gateway:
  separate process
  + controlled-server command authority
  + protected server credential boundary
~~~

### Step 9 — Change one dependency

Change a hook, game version, profile definition, or transform.

Expected:

~~~text
pinned decision: unchanged
old evidence: SUSPECT / STALE
~~~

### Step 10 — Change only the profile

Alter one simulation-affecting mapping without changing executable bytes.

Expected:

~~~text
artifact_digest: unchanged
effective_profile_digest: changed
compatibility evidence: stale
~~~

### Step 11 — Update publisher baseline

Start with publisher profile v1 plus a participant pin.

Install publisher profile v2.

Expected:

~~~text
publisher baseline: updated
participant overlay: preserved
effective profile: rederived
~~~

If they no longer compose:

~~~text
RECONFIRM_REQUIRED
~~~

### Step 12 — Test local approval versus training contribution

Accept one resolver suggestion locally.

Expected default:

~~~text
local profile: updated
remote dataset: unchanged
~~~

Then perform an explicit contribution operation and inspect the export.

### Step 13 — Test revocation

Publish a signed test release, then revoke its current authorization.

Expected:

~~~text
signature validity: PASS
current authorization: REVOKED
automatic launch: BLOCK
~~~

### Step 14 — Only then run the resolver experiment

Run approximately 30 semantic cases with:

- lookup/rules baseline;
- text-similarity baseline;
- one contrastive ranker;
- NO_MATCH;
- candidate-order permutations;
- held-out cases where practical.

Publish raw evidence so an outsider can reproduce the result.

---

## 12. Proposed changes to the Signet 2 draft

The following changes strengthen the proposal without replacing its core architecture.

1. **Define stable semantic identity and versioning.**  
   Human labels should not be the only identity of an intent or archetype.

2. **Use a small core plus versioned profiles/namespaces.**  
   Avoid a flat universal vocabulary that becomes FPS-shaped by default.

3. **Define explicit capability negotiation.**  
   Distinguish supported, required, activated, fallback, observe-only, and incompatible states.

4. **Make fallback typed.**  
   Simulation-affecting approximations need explicit policy and evidence.

5. **Separate archetype identity from authoritative ruleset parameters.**

6. **Split translation-profile entries by authority class.**  
   Input binding, motion observation, semantic mapping, appearance mapping, and session policy should not share one precedence rule.

7. **Separate publisher profile from participant overlay.**  
   Package updates must not silently overwrite participant-owned pins.

8. **Bind mappings to provenance and dependency identity.**  
   Include source/game/profile/transform/candidate fingerprints and conformance/replay evidence.

9. **Define compatibility drift and invalidation states.**  
   A pinned mapping may be deterministic and stale at the same time.

10. **Require NO_MATCH/abstention where candidate sets may be incomplete.**

11. **Define a calibration-probe interface.**  
   A wizard can measure only what the adapter legitimately exposes.

12. **Generate repetitive adapter contract glue mechanically.**  
   Keep irreducibly local hooks explicit and small.

13. **Use role/operation descriptors, not one universal convenience interface.**  
   Real targets should be allowed to falsify the descriptor.

14. **Treat resolver suggestions as advisory.**  
   Simulation-affecting misses should fail closed or require pinning.

15. **Make Forge local-first.**  
   Hosted resolver use should have an explicit export/privacy contract.

16. **Separate local approval from dataset contribution.**  
   Training contribution should be explicit, licensed, provenance-bearing, and invalidatable.

17. **Treat public translators as supply-chain software.**  
   Separate signature, current authorization, provenance, reproducibility, confinement, conformance, compatibility, and policy claims.

18. **Make integration mode part of the trust subject.**  
   In-process extension, gateway, sandboxed worker, reimplementation, and full application do not have the same confinement properties.

19. **Publish scoped interoperability evidence.**  
   Include independent implementation, pairwise execution, negative results, reproduction, and freshness.

20. **Separate governance and revocation powers.**  
   Protocol, profile, participant, directory, update-root, security, conformance, dataset/model, and policy authority should remain distinguishable.

21. **Replace broad anti-cheat wording with a narrower server-authority claim.**  
   Server authority prevents classes of direct client state forgery; it does not make all cheating impossible.

22. **Represent official integration policy as scoped project governance.**  
   Do not imply a universal legal conclusion from the official support boundary.

23. **Separate semantic acceptability from session preference.**  
   Profiles should declare which ordering/visibility models preserve their semantics; the session should publish the ordered preference used to select among the intersection.

24. **Make optional-profile activation deterministic and inspectable.**  
   Attempt optionals in explicit session preference order, rerun complete composition after each tentative addition, and publish activated/skipped profiles with exact reasons.

---

## 13. The boundaries worth preserving

These compact distinctions summarize the paper, but each is tied to a concrete technical test or authority rule above.

~~~text
SHARED_LABELS
!= SHARED_SEMANTICS

PINNED_CHOICE
!= CORRECT_CHOICE

SERVER_AUTHORITY
!= CHEAT_IMPOSSIBILITY

DECLARED_PERMISSION
!= ENFORCED_PERMISSION

VALID_SIGNATURE
!= CURRENT_AUTHORIZATION

PACKAGE_BYTES
!= EFFECTIVE_PROFILE

MODEL_SUGGESTION
!= AUTHORITY

LOCAL_APPROVAL
!= DATASET_CONTRIBUTION

SELF_CONFORMANCE
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= ECOSYSTEM_ADOPTION
~~~

The purpose of these invariants is not rhetorical caution.

Each one identifies a place where two states require different evidence or different authority.

---

## 14. Conclusion

Signet 2 has selected a strong scaling architecture.

Translating each game into shared meaning is a better long-term direction than accumulating pairwise bridges. Keeping authoritative effects on the server, local presentation on the client, adaptive resolution outside the real-time loop, and accepted mappings in deterministic profiles gives the project a clear architectural spine.

The remaining work is to make that spine explicit enough that independent teams can implement it without inheriting hidden assumptions.

Shared meaning needs stable semantic identity, modular profiles, and negotiated session semantics. Translation profiles need typed authority, ownership, provenance, and invalidation. Adapter tooling should generate stable contract glue while leaving local hooks visible. Calibration should fail to unknown rather than invent measurements. Resolver suggestions need abstention and remain advisory. Public translators need exact permission and confinement claims. Forge should be local-first and treat training contribution as an explicit act. Compatibility needs independent, reproducible, freshness-aware evidence. Governance should distinguish protocol evolution, package authorization, security revocation, conformance, datasets, and policy.

The resulting architecture is still recognizably Signet 2:

~~~text
local game
   |
adapter + pinned effective profile
   |
negotiated semantic contract
   |
authoritative shared simulation
   |
negotiated semantic contract
   |
adapter + pinned effective profile
   |
local presentation
~~~

The adaptive systems help create and review the edges.

They do not become hidden shared-world authority.

The practical target is not "an AI that can translate any game."

It is:

> **a translator ecosystem where independent implementers can agree on a small shared meaning, adaptive choices become inspectable artifacts, package authority is bounded, and compatibility claims can be reproduced and invalidated when the world changes.**

In compact form:

> **deterministic core + adaptive edge + explicit trust boundary + reproducible evidence**

That is a realistic path from a promising draft architecture to an open interoperability ecosystem.

---

## References

### Signet Protocol primary sources

1. **Signet Protocol.** *Signet 2: intents, archetypes and translation profiles.* Draft proposal v0.1, 2026-10-05.  
   https://github.com/kian-cx/signetprotocol/blob/main/docs/content/proposals/translation-profiles.mdx

2. **Signet Protocol.** *Signet Forge* design documentation, including architecture, calibration, translation-profile workflow, resolver behavior, reliability/limits, fine-tuning, and roadmap. Checked through commit 2ddb136ee941705d3e1c020eaddad93be65026f2.  
   https://github.com/kian-cx/signetprotocol/tree/main/docs/content/forge

3. **Signet Protocol.** Governance.  
   https://github.com/kian-cx/signetprotocol/blob/main/GOVERNANCE.md

### Semantic interoperability and validation

4. **NIST.** *Framework and Roadmap for Smart Grid Interoperability Standards, Release 2.0.* NIST SP 1108R2, 2012.  
   https://www.nist.gov/system/files/documents/smartgrid/NIST_Framework_Release_2-0_corr.pdf

5. **IETF.** RFC 5939, *Session Description Protocol (SDP) Capability Negotiation*, 2010.  
   https://www.rfc-editor.org/rfc/rfc5939.html

6. **W3C.** *Process Document — Implementation Experience.*  
   https://www.w3.org/policies/process/#implementation-experience

7. **IETF.** RFC 6410, *Reducing the Standards Track to Two Maturity Levels*, 2011.  
   https://www.rfc-editor.org/rfc/rfc6410.html

8. **IPv6 Ready Logo Committee.** Program and interoperability/conformance material.  
   https://www.ipv6ready.org/

9. **OpenID Foundation.** Interoperability/conformance reporting material, including attempted pairing coverage.  
   https://openid.net/

10. **NASA / SISO.** SpaceFOM V2 interoperability work, NASA NTRS 20260008222, 2026.  
    https://ntrs.nasa.gov/citations/20260008222

### Adapter engineering

11. **Godot Engine.** GDExtension documentation and machine-readable interface material.  
    https://docs.godotengine.org/

12. Hibino, H.; Fukuda, Y.; Yura, Y. Manufacturing Adapter / HLA distributed-simulation work, 2002.  
    DOI: 10.1299/jsmemsd.2002.99

13. El Kassis et al. *An HLA-based automated approach for the interoperable simulation of collaborative business processes.* Simulation Modelling Practice and Theory 135 (2024), 102977.  
    DOI: 10.1016/j.simpat.2024.102977

### Supply-chain and confinement references

14. **The Update Framework.**  
    https://theupdateframework.io/

15. **Sigstore.**  
    https://www.sigstore.dev/

16. **SLSA.**  
    https://slsa.dev/

17. **Reproducible Builds.**  
    https://reproducible-builds.org/

18. **Microsoft.** AppContainer / Windows application isolation documentation.  
    https://learn.microsoft.com/

19. **Linux kernel documentation.** Landlock userspace API.  
    https://docs.kernel.org/userspace-api/landlock.html

20. **Apple.** App Sandbox documentation.  
    https://developer.apple.com/documentation/security/app_sandbox

### Legal/policy scope

21. **European Union.** Directive 2009/24/EC on the legal protection of computer programs.

22. **Court of Justice of the European Union.** SAS Institute Inc. v World Programming Ltd., C-406/10.

These legal sources are included only to support the narrow proposition that interoperability analysis is action- and jurisdiction-specific. This paper does not offer a universal legal conclusion.

---

## Reproducibility material

The four supporting research lanes remain intact in Distributed-Minds/Fleet-Control-Public.

Exact lane heads observed for this synthesis revision:

- **L1 — Protocol & Semantic Interoperability:** research/signet-l1-protocol-semantics @ `b8f47a332fcb0aa641cfe676e22b49a801731628` (including executable deterministic pre-selection and optional-profile negotiation vectors)
- **L2 — Adapter Engineering & Adaptive Translation:** research/signet-l2-adapters-ai @ `a5a677f7bf3e80d2face8f2185b27fcb8e394ab3`
- **L3 — Trust, Distribution, Privacy, Legal & Governance:** research/signet-l3-trust-distribution @ `821d13fe33f293281c85dc9da3a32e2a3fb47148`
- **L4 — Validation, Ecosystem & Adoption:** research/signet-l4-validation-ecosystem @ `22cdaf37b08d940a518c4a6a9dcea51553cd7256`

Important supporting artifacts include:

- L1 semantic-contract and Signet 2 response analysis;
- L2 typed translation-profile, drift checker, generated adapter surfaces, Godot host-harness target, Minecraft RCON target, role-contract v1, and resolver pilot;
- L3 package-trust contract and publication-ready trust/privacy/governance sections;
- L4 validation paper, supplement, replay/evidence schemas, and evidence ladder.

The upstream Signet 2 proposal is explicitly a draft and not implemented Signet/1 behavior. Experimental claims in this response are scoped to the exact observed fixtures named above; they are not claims that Signet itself has implemented the proposed refinements.
