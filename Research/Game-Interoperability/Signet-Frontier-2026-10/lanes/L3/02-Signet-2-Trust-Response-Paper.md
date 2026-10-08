# Signet 2 Needs a Trust Layer

## A response on translation profiles, Forge, distribution, privacy, and governance

**Status:** synthesis-ready L3 response draft  
**Date:** 2026-10-05  
**Scope:** trust, distribution, privacy, legal/policy boundaries, and governance  
**Responding to:** *Signet 2: intents, archetypes and translation profiles*, draft proposal v0.1, 2026-10-05  
**Relationship to Signet:** unaffiliated technical analysis

---

## Abstract

Signet 2 makes a strong architectural move: replace pairwise game-to-game translation with translation from each game into shared meaning and back out again. It keeps an authoritative neutral server, introduces intent and archetype vocabularies, records decisions in deterministic translation profiles, and keeps model inference out of the per-tick simulation path.

We agree with that direction.

The main missing layer is not another semantic abstraction. It is a **trust architecture for the artifacts that decide how semantics are applied**.

Once a translation profile can determine which local action becomes a simulation-affecting intent, the profile is no longer merely a convenience file. Once community pull requests can change those profiles, the repository becomes part of the behavioral supply chain. Once Signet Forge can turn reviewed choices into training data and improved models, Forge is outside the match loop but still inside the development trust boundary. Once translators are publicly installed and updated, package identity, permissions, sandbox enforcement, update freshness, revocation, provenance, and governance become protocol-adjacent safety requirements.

This response proposes a concrete extension:

> **deterministic core, adaptive edge, explicit trust boundary**

The key rule is that different kinds of authority must remain distinguishable:

~~~text
PLAYER_PIN
!= TRANSLATOR_PUBLISHER_PROFILE
!= RESOLVER_SUGGESTION
!= SAFE_DEFAULT
~~~

and different trust claims must remain distinguishable:

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
CURRENT_AUTHORIZATION != SAFE_ARTIFACT
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
COMPATIBILITY         != LEGAL_PERMISSION
PACKAGE_BYTES         != EFFECTIVE_PROFILE
MODEL_SUGGESTION      != AUTHORITY
~~~

The result is still simple enough for players: install a translator, calibrate once, and play. The additional machinery exists so that the software can truthfully answer who changed what, what may execute, what data leaves the machine, which evidence is current, and which authority can revoke or override each decision.

---

## 1. What Signet 2 gets right

### 1.1 Shared meaning is the correct scaling direction

**OBSERVED:** Signet 2 rejects pairwise game-to-game translation and instead proposes game -> meaning -> game. The server understands intents and archetypes; each game-specific translator maps local controls and representations to and from those shared semantics.

**DERIVED:** This is the right anti-N-squared architecture. It makes a new game responsible for one semantic boundary instead of a growing set of bilateral translators.

This is also consistent with the strongest prior-art pattern found in the broader interoperability research: independent participants should target a federation or mediator contract rather than directly encode every other participant.

### 1.2 The authoritative server is the correct place for shared truth

**OBSERVED:** Signet 2 keeps simulation, physics, rules, damage, and authoritative state on the server. Clients emit intents and receive neutral state/archetypes.

**DERIVED:** That sharply reduces the amount of game-specific authority that must be trusted. A translator should not be able to assert an arbitrary final position, damage value, or death event merely because its local game produced one.

One wording in the design document is stronger than the evidence supports: an authoritative server does not mean that “nobody can cheat by modifying their client.” It removes important classes of client-authoritative state cheating, but input automation, timing abuse, information exposure, malformed intent streams, collusion, and implementation vulnerabilities remain security problems.

The stronger claim is:

> **The server is authoritative, so a modified client does not become authoritative merely by reporting a different world state.**

That is a substantial property without pretending it eliminates cheating.

### 1.3 Pinned translation is stronger than runtime improvisation

**OBSERVED:** Signet 2 puts obvious controls in calibration, measurable movement into a motion profile, and complex cases behind a resolver whose decision is reviewed and then pinned. Simulation-affecting intents must come from pinned decisions. Model inference is not in the per-tick loop.

**DERIVED:** This is exactly the right boundary for AI assistance. The model can help create deterministic configuration, but the simulation does not need the model to reinterpret meaning continuously.

The useful formula remains:

~~~text
model suggests
human or policy resolves
profile records
runtime performs deterministic lookup
server decides shared state
~~~

### 1.4 The Signet Protocol / Signet Forge split is valuable

**OBSERVED:** The proposal distinguishes Signet Protocol as the standard/runtime surface from Signet Forge as a companion tool for suggestion review, profile editing, datasets, and model fine-tuning. Forge is not required to play.

**DERIVED:** This is a good product and security boundary. A player should not need a GPU model or a development application merely to participate in a match.

However, “not required during play” must not become “outside the trust model.” Forge can still produce artifacts that later control runtime behavior. That makes Forge part of the development supply chain even when it is not part of the runtime dependency graph.

### 1.5 The proposed legal scope is a useful project policy

**OBSERVED:** The proposal explicitly excludes online-game code injection and anti-cheat bypass and prefers open-source engines, permitted mods/APIs, and controlled servers.

**DERIVED:** This is a sensible official-ecosystem boundary. It lowers operational, security, and policy risk.

It should remain a **project distribution policy**, not be promoted into a universal statement that anything outside those integration paths is unlawful. Interoperability law is jurisdiction- and action-specific, and in the EU there are statutory software interoperability exceptions with conditions. A project may deliberately choose a narrower supported scope than the outer boundary of the law.

---

## 2. The central trust issue: a translation profile is authority-bearing configuration

The Signet 2 document describes a translation profile as a lock-like file in which pinned decisions “stay decided.” It also says that a profile can live in git and be improved by community pull requests.

Those two ideas are individually reasonable, but together they expose the central governance problem.

A profile cannot simultaneously mean all of the following without additional structure:

- an immutable record of what the player chose;
- a publisher-maintained baseline;
- a community-editable artifact;
- model-generated candidate history;
- a compatibility claim;
- the current runtime configuration.

These are different authorities.

### 2.1 Separate the four decision sources

**PROPOSED:** Represent the source of a mapping as one of four explicit authority classes.

| Authority | May affect simulation? | May be silently replaced by package update? | Typical owner |
|---|---:|---:|---|
| Player pin | yes | **no** | participant/user |
| Translator publisher profile | yes, after validation | only by an explicitly installed new release | translator publisher |
| Resolver suggestion | **no direct authority** | n/a | model/table/service |
| Safe default | only if the semantic contract defines it as safe | protocol/policy controlled | protocol/session policy |

The current priority order in Signet 2 is a good starting point:

~~~text
player pin
> translator profile
> resolver suggestion
> safe default
~~~

The missing requirement is that each tier must retain its identity and lifecycle. A package update must not rewrite a user-owned pin and then still describe the result as “the player pinned this.”

### 2.2 Use three stored layers, not one mutable lock file

**PROPOSED:**

~~~text
publisher_profile
    immutable and versioned with a translator release

participant_overlay
    local, user-owned pins and explicit overrides

session_resolution
    derived effective profile for this game/session/protocol context
~~~

The first two are stored authorities. The third is a derived result.

An effective profile should be identifiable by at least:

~~~text
translator package digest
publisher profile digest
participant overlay digest
game/version fingerprint
protocol version
intent vocabulary revision
archetype catalog revision
calibration/motion-profile revision
resolver identity/version where relevant
unresolved-entry set
~~~

The exact canonical serialization is still an implementation question, but the semantic requirement is not:

> **The evidence for compatibility must identify the behavior that was actually executed, not merely the executable package bytes.**

### 2.3 “Pinned” must name the subject

A pin only has meaning if the system can answer:

- who pinned it;
- which key was pinned;
- against which candidate set;
- for which game/version;
- against which semantic vocabulary/catalog revision;
- whether it affects shared simulation or local appearance;
- whether a later package/profile release invalidated the assumptions under which it was chosen.

A useful pin record therefore needs provenance, not just a boolean.

---

## 3. Ambiguities in the draft that should be resolved before implementation

The Signet 2 proposal is a draft, so these are not defects in a shipped protocol. They are places where implementation can easily diverge unless the trust semantics are made explicit now.

### 3.1 “Once pinned, always the same” vs community profile pull requests

**OBSERVED:** The document says pinned decisions are deterministic and do not change, while also inviting the community to improve translator profiles through pull requests.

**RISK:** A repository merge can change the publisher profile underneath a user and silently reinterpret “pinned.”

**PROPOSED:** Treat publisher profiles as immutable release artifacts. A merged PR creates a new profile revision/digest. Participant overlays survive package/profile updates unless an explicit migration or reconfirmation rule applies.

### 3.2 “Suggest” mode vs mandatory pins for simulation-affecting intents

**OBSERVED:** In suggest mode, a resolver's best candidate may be used provisionally. The same section says simulation-affecting intents must always be pinned.

**PROPOSED:** Make the boundary explicit:

- provisional suggestions may directly affect **local presentation** when the failure mode is cosmetic/local;
- a simulation-affecting unknown may be displayed to a user as a suggestion, but it must not emit an authoritative intent until pinned or otherwise deterministically resolved by a protocol-defined safe rule.

Useful safe outcomes include:

~~~text
NO_AUTHORITATIVE_ACTION
IGNORE
OBSERVE_ONLY
REQUIRE_PIN
~~~

A “best guess fire action” is not a safe default merely because the resolver returned the highest score.

### 3.3 “The model cannot invent” does not mean “the model cannot be wrong”

**OBSERVED:** The resolver receives a closed candidate list.

**DERIVED:** This is a valuable syntactic safety property: the model cannot emit an arbitrary identifier outside the list.

It does not establish:

- semantic correctness;
- calibrated confidence;
- resistance to poisoned candidate metadata;
- resistance to biased training examples;
- correctness for an out-of-distribution game;
- safety of the candidate list itself.

A score such as 0.81 should not be displayed or stored as an 81% probability unless the model has actually been calibrated for that interpretation.

### 3.4 “AI stays outside the protocol” is true at runtime but incomplete for supply-chain trust

If a resolver produces a mapping, a person accepts it, and that accepted mapping is shipped in a translator profile, the model has influenced runtime behavior even though no inference runs during the match.

That is not a reason to reject the architecture. It is a reason to record lineage.

For accepted model-assisted mappings, the profile or associated evidence should be able to identify:

- resolver/model identifier and version;
- candidate set;
- accepted choice;
- accepting actor or review class;
- game/profile revision;
- whether the model output was later invalidated;
- training/example lineage if the choice later enters a dataset.

### 3.5 “Every approved choice is a training example” needs an explicit contribution boundary

The document's dataset rules are good: names/identifiers only, no game files, no usernames, no input logs, open license.

But the word “approved” is overloaded.

A user can approve a local mapping because they want their game to work. That action should not implicitly mean:

> upload this decision to a shared dataset and license it for model training.

**PROPOSED:**

~~~text
local approval != dataset contribution
~~~

Training contribution should be a separate explicit act with:

- an export preview/schema;
- the exact fields leaving the machine;
- source/profile/game revision;
- contribution license;
- provenance;
- an invalidation mechanism when the underlying mapping is later found wrong.

### 3.6 “Only meaning travels over the network” is an application-data goal, not a complete privacy claim

The neutral protocol can avoid transmitting textures, sounds, models, and other game assets. That is a strong privacy and licensing property.

The network still exposes or processes other data depending on topology:

- peer/server addresses;
- timing;
- session membership;
- game/translator capability metadata;
- possibly game/version identifiers;
- telemetry or crash reports if enabled;
- any remote resolver/Forge export if a shared service is used.

Privacy documentation should therefore distinguish:

~~~text
application payload
transport metadata
optional telemetry
development-tool export
training-data contribution
~~~

### 3.7 “Officially allowed” is a support-policy label, not a universal legal boolean

Signet is free to say that its official directory will only support specified integration modes. That is operationally useful.

The policy record should still distinguish technical actions and legal questions rather than store one value such as LEGAL=true.

For example:

~~~yaml
technical_actions:
  observe_or_test: true
  parse_file_formats: true
  decompile_code: false
  circumvent_access_control: false
  modify_game_process: false
  connect_official_online_service: false
  redistribute_game_assets: false
~~~

A directory can then decide ALLOW, REVIEW_REQUIRED, or EXCLUDE_OFFICIAL_DIRECTORY for a declared scope without pretending it has issued a universal legal judgment.

---

## 4. Package trust is a different layer from semantic correctness

Signet 2 mostly discusses how a translator should decide meaning. A public translator ecosystem also has to decide whether code should be installed and executed.

These questions must not collapse into one “verified” badge.

### 4.1 Minimum package subject

**PROPOSED:** Every installable translator release should have an exact subject including:

- package namespace and name;
- package version;
- artifact digest;
- platform/architecture;
- dependency lock/digests;
- publisher identity;
- integration mode;
- publisher profile digest;
- requested permissions;
- update channel/repository metadata.

### 4.2 Typed trust claims

The launcher or directory should render independent claims such as:

**Origin and authorization**
- artifact digest matches;
- signature valid;
- publisher identity verified;
- directory namespace authorized;
- update metadata current.

**Source and build**
- source reference known;
- provenance verified;
- reproducible build independently reproduced;
- dependencies locked.

**Runtime**
- requested permissions declared;
- sandbox actually enforced on this platform;
- install rollback available;
- undeclared network blocked.

**Compatibility**
- protocol conformance result;
- game-version compatibility result;
- exact profile subject tested;
- evidence age/currentness.

**Policy**
- integration mode;
- license metadata present;
- legal/policy review scope;
- unresolved questions.

These claims may all be true or false independently.

### 4.3 Signature validity is not current authorization

A correctly signed old artifact can become unsafe after:

- publisher key compromise;
- security revocation;
- dependency compromise;
- discovered malicious behavior;
- superseding emergency release.

That is why a public update system needs freshness and revocation semantics, not only artifact signatures.

A TUF-shaped repository model is a strong fit because it separates root, targets, snapshot, and timestamp responsibilities and explicitly models rollback/freeze/key-compromise threats. Sigstore-style identity/transparency can complement that by binding a release signature to a short-lived workload or human identity. SLSA provenance and reproducible builds answer different questions again.

The composition is:

~~~text
update metadata -> is this exact target currently authorized?
signature identity -> who/what signed it?
provenance -> where/how was it built?
reproducibility -> can an independent party recreate the artifact?
sandbox evidence -> what can it actually do on this machine?
conformance -> does it implement the Signet contract?
compatibility -> does this exact package/profile work with this exact game context?
~~~

No one item replaces the others.

---

## 5. Runtime permissions should be brokered, not merely declared

A translator can require serious authority: read game data, write generated files, start a controlled game/server process, or connect to a Signet session.

A manifest is useful only if the launcher can constrain those requests.

### 5.1 Symbolic resources

Instead of letting a package request arbitrary host paths, the package should request symbolic resources:

~~~text
GAME_INSTALL_READ
GAME_INSTALL_WRITE_STAGED
TRANSLATOR_WORKDIR_RW
SIGNET_SESSION_NETWORK
CONTROLLED_SERVER_PROCESS
USER_SELECTED_FILE_READ
~~~

The launcher resolves those symbols to host-specific paths or handles after user/policy approval.

### 5.2 Declaration and enforcement must be separate evidence

The same package may have different confinement properties on different platforms or integration modes.

Useful runtime classes include:

~~~text
SEPARATE_SANDBOX_ENFORCED
HOST_API_CONFINED
HOST_PROCESS_CODE
FULL_APPLICATION
UNCONFINED
UNKNOWN
~~~

and enforcement results such as:

~~~text
ENFORCED
PARTIALLY_ENFORCED
DECLARED_ONLY
NOT_SUPPORTED
FAILED
UNKNOWN
~~~

An official mod loaded directly into a game's process should not be presented as OS-sandboxed merely because its manifest asked for few permissions.

### 5.3 Platform implementation can vary without changing the contract

The portable contract can be implemented with platform-specific mechanisms:

- Windows: AppContainer / current Create Process In Sandbox capabilities where applicable;
- Linux: Flatpak-style mediation, Landlock and other kernel/runtime restrictions where available;
- macOS: App Sandbox for signed helper components and a deliberately narrow helper/XPC-style boundary where arbitrary downloaded translator logic cannot be safely granted broad application authority.

The user-facing claim should describe the observed enforcement property, not just the intended backend.

---

## 6. Signet Forge should be local-first and contribution-explicit

Signet 2 deliberately leaves the resolver location open: local machine, shared service, or development tool.

From the trust/privacy perspective these are not equivalent deployment choices.

### 6.1 Recommended default

**PROPOSED:**

1. **Runtime play:** profile lookup only; no resolver dependency.
2. **Local calibration/review:** local resolver where practical.
3. **Forge development:** local by default.
4. **Remote/shared resolver:** optional and explicit, with a declared export schema and purpose.
5. **Training contribution:** separate opt-in action from local profile approval.

This preserves the paper's strongest property: a model failure or service outage cannot change a running match.

### 6.2 Remote Forge needs an export contract

Enabling a remote resolver must not implicitly grant access to everything the local discovery/calibration process can see.

A remote export should declare fields such as:

~~~yaml
remote_resolver_export:
  game_identifier: true
  game_version: optional
  archetype_or_intent_id: true
  candidate_identifiers: true
  free_text_notes: false
  local_paths: false
  installed_game_inventory: false
  raw_input_events: false
  save_files: false
  game_assets: false
  usernames: false
~~~

The exact schema may evolve. The critical property is that remote execution adds a new trust boundary rather than inheriting local access.

### 6.3 Dataset governance is separate from package governance

A profile may be good enough to ship but unsuitable as a training example. A training example may be useful research data but not authoritative enough to ship.

Therefore maintain separate states for:

~~~text
profile accepted for package
example eligible for dataset
example licensed for training
example validated
example invalidated
model trained from example set
~~~

If a mapping is later found wrong, the system should be able to identify which dataset/model versions inherited it.

---

## 7. Resolving conflicting community profiles

One of Signet 2's explicit open questions is how to resolve conflicting community profiles for the same game.

The answer should not be “pick the newest,” “pick the most popular,” or “let the model merge them.”

### 7.1 First resolve identity, then conflict

Many apparent conflicts are actually different subjects.

A profile should identify:

- translator package;
- translator publisher/namespace;
- game and relevant version/fingerprint;
- protocol version;
- intent vocabulary revision;
- archetype catalog revision;
- integration mode;
- calibration assumptions.

Two profiles with different subjects are alternatives, not conflicting edits to one object.

### 7.2 Use a publisher baseline plus participant overlay

For one exact translator release:

~~~text
effective profile
=
publisher baseline
+
participant-owned overlay
+
session-compatible derivation
~~~

A community PR can improve the publisher baseline for a future release. It should not rewrite an already-installed participant overlay.

### 7.3 Treat profile diffs as behavioral diffs

A profile-only update can alter shared behavior without changing executable bytes.

**PROPOSED:** classify profile changes:

- presentation-only;
- control-binding only;
- calibration/motion change;
- simulation-affecting semantic mapping;
- vocabulary/catalog migration;
- resolver metadata only.

Simulation-affecting changes should automatically stale relevant compatibility evidence and require replay/conformance reruns. Depending on severity, a participant pin may need reconfirmation rather than automatic migration.

### 7.4 Multiple community profiles can coexist

The directory can list alternatives with distinct publishers/namespaces/evidence. Users or translator maintainers can choose one deliberately.

The system should not silently synthesize a “consensus profile” unless there is a separately specified, deterministic process whose result has its own identity and evidence.

---

## 8. Governance: separate powers before the ecosystem depends on them

Signet's current governance document is appropriately simple for a young project: one maintainer has final say while the community forms, protocol proposals remain open for at least 14 days, and translator authors own their translator/license/manifest.

That is workable today.

The important next step is to distinguish governance authorities even if the same person temporarily holds several of them.

### 8.1 Distinct authorities

**PROPOSED:**

1. **Protocol governance**
   - wire contract;
   - intent vocabulary;
   - archetype catalog;
   - versioning rules.

2. **Translator/profile governance**
   - package implementation;
   - palette;
   - publisher profile;
   - calibration logic.

3. **Participant authority**
   - local bindings;
   - local pins;
   - local privacy choices.

4. **Directory governance**
   - listing;
   - namespace authorization;
   - evidence display;
   - compatibility yanking;
   - policy disposition.

5. **Update trust-root authority**
   - repository root;
   - delegated publisher namespaces;
   - compromise recovery.

6. **Security incident authority**
   - emergency quarantine/revocation;
   - advisory publication;
   - unquarantine criteria.

7. **Forge/dataset/model governance**
   - dataset admission;
   - contribution licensing;
   - example invalidation;
   - model release identity.

8. **Conformance authority**
   - test-suite versions;
   - reproducibility;
   - PASS semantics.

These powers should not be collapsed into “maintainer approval.”

### 8.2 A community profile pull request is a supply-chain event

A PR that changes weapon.ranged -> bow into weapon.ranged -> fishing_rod is not merely documentation.

The review surface should show:

- semantic diff;
- whether shared simulation is affected;
- profile subject;
- stale compatibility claims;
- tests/replays required;
- reviewer identity;
- resulting immutable profile digest.

### 8.3 Yank, revoke, and supersede are different operations

A package can be:

- superseded by a newer release;
- deprecated;
- compatibility-yanked;
- security-revoked;
- publisher-revoked;
- blocked from an official directory for policy/legal reasons.

These states have different meanings and should produce different launcher behavior.

A compatibility yank should not pretend a package is malicious. A security revoke should not be bypassed by treating the release as merely old.

---

## 9. Legal and policy boundaries should stay scoped and machine-readable

This research is not legal advice.

The useful engineering goal is to avoid encoding false legal certainty into the package system.

### 9.1 Keep integration mode explicit

A translator/package should declare its technical integration mode, for example:

~~~text
OPEN_ENGINE
OFFICIAL_MOD
OFFICIAL_API
CONTROLLED_SERVER_GATEWAY
LOCAL_REIMPLEMENTATION
UNSUPPORTED_INJECTION
~~~

The official Signet ecosystem can choose to exclude UNSUPPORTED_INJECTION and similar high-risk modes regardless of whether a specific jurisdiction might permit some underlying reverse-engineering activity.

### 9.2 Record actions and evidence, not “LEGAL=true”

For a declared game/product/jurisdiction scope, a policy record can state which technical actions are actually performed and which sources were reviewed.

This matters because EU software law, national implementations, contract terms, anti-circumvention rules, online-service terms, redistribution rights, and trademark questions are separate issues.

The project can then make a distribution decision such as:

~~~text
ALLOW
ALLOW_WITH_USER_WARNING
REVIEW_REQUIRED
EXCLUDE_OFFICIAL_DIRECTORY
UNKNOWN
~~~

That decision is governance for the official directory, not a universal legal judgment.

---

## 10. A trust-aware extension to the Signet 2 roadmap

The six-step roadmap in the Signet 2 paper is a good semantic/translator sequence:

1. vocabulary/catalog v0;
2. capabilities;
3. profile format + calibration;
4. no-AI end-to-end prototype;
5. resolver experiment;
6. first Forge.

The trust work can run in parallel rather than delaying the semantic prototype.

| Before/alongside | Add this trust milestone | Why |
|---|---|---|
| Step 1-2 | version vocabulary/catalog entries and governance process | profile evidence must bind exact semantics |
| Step 3 | define profile subject, source authority, publisher baseline, participant overlay | prevents “pinned” ambiguity |
| Step 3 | define package manifest + symbolic permissions | makes translator installation reviewable |
| Step 4 | emit sandbox/enforcement evidence for the prototype | proves declaration != enforcement |
| Step 4 | bind compatibility result to package + profile + game fingerprint | profile-only changes can invalidate PASS |
| Step 5 | define abstention/unknown policy for simulation-affecting mappings | highest score must not become authority |
| Step 5 | record resolver/model/version/candidate lineage for accepted suggestions | makes model-assisted mappings auditable |
| Step 6 | make local approval distinct from dataset contribution | prevents accidental training-data upload |
| Step 6 | define remote Forge export schema if a shared service is used | preserves local-first privacy |
| Before public auto-update | add freshness, rollback, revocation, compromise recovery | signatures alone are insufficient |
| Before official directory scale | separate directory, protocol, security, update-root, dataset authorities | limits governance capture and ambiguous revocation |

The key point is that this is not a demand to solve enterprise package security before demonstrating Signet 2.

It is a demand to prevent the prototype's accidental conventions from becoming the public ecosystem's security model.

---

## 11. Direct answers to the Signet 2 open questions that intersect L3

### 11.1 Who maintains the vocabulary and catalog?

**PROPOSED:** Protocol governance owns them, not the package directory, Forge model, or one translator publisher.

A practical early process can still use the current maintainer model, but entries should have:

- stable identifiers;
- version/revision history;
- proposal rationale;
- compatibility impact;
- explicit deprecation/migration rules;
- review from more than one engine perspective as the contributor base grows.

The exact semantic admission criteria belong to L1. The governance separation is the L3 requirement.

### 11.2 Where does the resolver run?

**PROPOSED default:** locally during calibration/development, never required in the per-tick match path.

A shared resolver service may be offered, but enabling it should create an explicit remote-data boundary with an export schema, retention/purpose statement, and no inherited access to local discovery state.

Forge can also host model training independently of player runtime.

### 11.3 How are conflicting community profiles resolved?

**PROPOSED:** do not resolve by silent merge.

1. identify exact profile subject;
2. keep publisher baseline immutable per translator release;
3. keep participant pins in a separate local overlay;
4. allow multiple publisher/community alternatives to coexist;
5. classify semantic diffs;
6. invalidate/re-run compatibility evidence when behavior changes;
7. require reconfirmation where a simulation-affecting migration would otherwise replace a participant decision.

### 11.4 What this paper does not answer

The minimum motion-profile measurements and calibration of a game that does not report position are not primarily L3 questions. They belong to protocol/adapter/validation work.

They should be answered with experiments, not by extending the trust model.

---

## 12. Falsifiable security and governance tests

A trust architecture is useful only if an implementation can fail tests.

At minimum, the eventual launcher/directory/profile stack should demonstrate the following:

1. **Signed executable, unsigned mutable profile:** the profile is not treated as authoritative merely because the executable is signed.
2. **Participant pin preservation:** a package/profile update cannot silently replace a local participant pin.
3. **Profile-only behavior change:** simulation-affecting profile change invalidates old compatibility evidence.
4. **Mixed rollback:** an old executable cannot be combined silently with a newer/older incompatible profile to recreate a previously revoked behavior.
5. **Revoked publisher:** a cryptographically valid old signature does not restore current authorization.
6. **Stale update metadata:** freeze/rollback conditions are detected according to policy.
7. **Undeclared filesystem access:** sandbox/broker denies it.
8. **Undeclared outbound network:** sandbox/broker denies it or reports that enforcement is unavailable.
9. **Unpinned simulation suggestion:** resolver miss for fire/move cannot emit an authoritative action.
10. **Malicious profile PR:** semantic diff identifies the simulation-affecting mapping and requires the correct review/test class.
11. **Remote Forge privacy:** local paths, installed-game inventory, raw input events, assets, and saves are not uploaded unless an explicit schema says otherwise.
12. **Training consent:** accepting a local mapping does not automatically contribute it to a shared dataset.
13. **Example invalidation:** a bad training example can be traced to affected dataset/model releases.
14. **Conflicting profiles:** selection is deterministic and user/publisher authority is visible; popularity does not silently rewrite local behavior.
15. **Compatibility yank vs security revoke:** launcher behavior differs appropriately.
16. **Legal/policy unknown:** an unresolved official-directory policy record does not become an implied legality guarantee.
17. **Server-authority security:** client-side modification cannot directly assert authoritative state, while abuse through allowed intent channels remains independently testable.
18. **Trust UI:** no single badge hides which exact claims are current, stale, unknown, or unverified.

These tests are suitable handoffs to L4.

---

## 13. Proposed user-facing result

The end user should not have to understand TUF, SLSA, Sigstore, Landlock, AppContainer, profile provenance, or model lineage.

The launcher can summarize the exact evidence:

~~~text
Publisher
  Identity: verified for org.example
  Current release authorization: current

Build
  Provenance: verified
  Independent rebuild: not yet reproduced

Permissions
  Game installation: read-only
  Work directory: read/write
  Network: current Signet session only
  Runtime confinement: kernel-enforced on this system

Compatibility
  Signet profile: PASS
  Game version: reproduced
  Evidence age: 3 days

Translation profile
  Publisher baseline: current
  Your local overrides: 2
  Simulation-affecting changes since previous release: none

Forge
  Runtime dependency: none
  Remote resolver: disabled
  Training contribution: disabled

Policy
  Integration mode: controlled server gateway
  Official directory: allowed for declared scope
  Legal status: scoped review; no universal legality guarantee
~~~

That is more information internally, but less ambiguity externally.

---

## 14. The paper's strongest architecture survives this critique

The purpose of this response is not to replace Signet 2.

The proposal's core remains intact:

- shared intents and archetypes instead of pairwise translation;
- authoritative common simulation;
- capabilities for heterogeneous games;
- calibration for obvious/measurable mappings;
- a closed candidate set for hard mappings;
- human confirmation;
- pinned deterministic runtime lookup;
- Forge separated from the play dependency path.

The trust layer adds one more separation:

~~~text
semantic meaning
!= authority to change the mapping
!= authority to install code
!= authority to export data
!= authority to declare compatibility
!= authority to declare policy/legal status
~~~

That separation becomes more important, not less, as the project succeeds.

A two-game prototype can survive informal trust.

A public ecosystem of community translators, automated updates, profile pull requests, local calibration, optional remote resolvers, training datasets, and one-click launchers cannot.

---

## Conclusion

Signet 2's game -> meaning -> game architecture is a credible answer to the scaling problem that motivated this research. Its strongest decision is to make adaptive translation produce deterministic artifacts instead of turning the model into a runtime authority.

The next design step is to recognize that those deterministic artifacts become **authority-bearing supply-chain objects**.

The translation profile needs an exact subject and provenance. Player pins need a separate ownership layer from publisher defaults. Resolver suggestions need to remain advisory until explicitly promoted. Community profile pull requests need behavioral diffs and compatibility invalidation. Forge needs local-first defaults and an explicit contribution boundary. Public translator packages need permissions, sandbox evidence, current update authorization, revocation, and typed trust claims. Governance should separate protocol, package directory, update trust root, security response, conformance, and dataset/model powers even while a young project has few maintainers.

The compact version is:

> **Keep Signet 2's deterministic core and adaptive edge. Add explicit authority, provenance, and revocation everywhere the edge can become durable behavior.**

That produces a system that can scale not only from two games to twenty games, but from a trusted prototype community to strangers safely installing and improving each other's translators.

---

## References

1. **Signet Protocol**, *Signet 2: intents, archetypes and translation profiles*, draft proposal v0.1, 2026-10-05.  
   https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/content/proposals/translation-profiles.mdx  
   PDF: https://github.com/kian-cx/signetprotocol/blob/490dfa9423841a45f2917d8d013e010ca0eb5548/docs/public/signet-2-architecture.pdf

2. **Signet Protocol**, *Governance*.  
   https://github.com/kian-cx/signetprotocol/blob/main/GOVERNANCE.md

3. **Signet Protocol**, *Security policy*.  
   https://github.com/kian-cx/signetprotocol/blob/main/SECURITY.md

4. **The Update Framework**, metadata roles and security model.  
   https://theupdateframework.io/docs/metadata/  
   https://theupdateframework.io/docs/security/

5. **Sigstore**, signing and verification documentation.  
   https://docs.sigstore.dev/cosign/signing/overview/  
   https://docs.sigstore.dev/cosign/verifying/verify/

6. **SLSA**, specification v1.2.  
   https://slsa.dev/spec/v1.2/

7. **Reproducible Builds**, definition.  
   https://reproducible-builds.org/docs/definition/

8. **Microsoft**, AppContainer and Create Process In Sandbox documentation.  
   https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation  
   https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox

9. **Linux kernel documentation**, Landlock userspace API.  
   https://www.kernel.org/doc/html/latest/userspace-api/landlock.html

10. **Apple**, App Sandbox documentation.  
    https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox

11. **European Data Protection Board**, basic data-protection principles.  
    https://www.edpb.europa.eu/topics/key-gdpr-concepts/basic-principles_en

12. **Directive 2009/24/EC on the legal protection of computer programs**, especially Articles 5 and 6.  
    https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32009L0024

13. **CJEU, SAS Institute Inc. v World Programming Ltd., C-406/10**.  
    https://eur-lex.europa.eu/legal-content/EN/ALL/?uri=celex:62010CJ0406

---

## Cross-lane handoffs

~~~yaml
handoff:
  from_lane: L3
  to_lane: L1
  finding: Translation-profile identity and compatibility evidence need to bind exact intent-vocabulary and archetype-catalog revisions.
  why_material: A profile can remain byte-identical while its semantic referents change.
  requested_followup: Define stable semantic identifiers, revision compatibility, and the minimum session-visible effective-profile identity.

handoff:
  from_lane: L3
  to_lane: L2
  finding: Resolver suggestions must remain advisory; accepted mappings need model/candidate/reviewer lineage, and simulation-affecting unknowns need abstention rather than provisional authority.
  why_material: Closed-list ranking bounds output syntax but not correctness or confidence.
  requested_followup: Prototype resolver abstention/OOD behavior and evidence-bearing profile writes.

handoff:
  from_lane: L3
  to_lane: L4
  finding: Package/profile trust needs deterministic adversarial fixtures, especially profile-only behavior changes, participant-pin preservation, revocation, remote-Forge privacy, and training-consent boundaries.
  why_material: These claims are only useful if independent implementations can reproduce PASS/fail behavior.
  requested_followup: Add these cases to the conformance/compatibility validation matrix.
~~~


---

## 15. Real-target trust consequence: integration mode changes the security claim

The current L2 implementation pass materially strengthens one L3 conclusion.

The public response now includes two real-target experiments:

1. a Godot 4.7.2 GDExtension ABI target;
2. Signet's documented Minecraft controlled-server/RCON gateway.

These targets implement different integration mechanisms and therefore have different trust surfaces.

### Godot GDExtension

The generic Godot experiment correctly claims only the host/version boundary and zero game-specific gameplay capabilities.

That same discipline should apply to confinement.

A native engine extension executes in the host application's process. Its useful trust classification is therefore approximately:

~~~text
HOST_PROCESS_CODE
~~~

rather than SEPARATE_SANDBOX_ENFORCED.

The extension may still be:

- signed;
- reproducibly built;
- provenance-bearing;
- reviewed;
- constrained by a narrow host API.

Those are real claims.

They do not create a kernel/process isolation boundary that is not present.

### Minecraft controlled-server/RCON gateway

The documented Minecraft gateway can issue commands that:

- construct or modify the world;
- inspect player/actor state;
- teleport or reconcile actors;
- apply damage;
- kill actors;
- observe selected gameplay signals.

This is a high-impact controlled-server administration surface.

The trust record should therefore expose the actual authority instead of treating gateway as one generic safe category.

Illustratively:

~~~yaml
integration:
  mechanism: RCON_GATEWAY

authority:
  controlled_server_command: true
  world_mutation: true
  actor_reposition: true
  damage_application: true
  death_application: true
~~~

The gateway's RCON credential is also a separate secret-bearing boundary.

Where practical:

- use a dedicated credential;
- keep it out of package files;
- redact it from logs/crash reports;
- deliver it only to the process or broker that needs it;
- do not let possession of the RCON secret imply access to unrelated launcher credentials.

### Link to the new L2 role contract

The revised L2 descriptor classifies target operations under roles such as:

~~~text
HOST
IMPORTER
WORLD
INPUT
PRESENTATION
AUTHORITY
~~~

This is a useful source for generating or reviewing requested permissions.

It is not itself an enforcement system.

~~~text
ROLE_CONTRACT != SANDBOX_POLICY
~~~

A useful implementation pipeline is:

~~~text
target role contract
    -> declared authority/permission request
    -> launcher/broker resolution
    -> platform enforcement
    -> measured enforcement evidence
~~~

This gives the real-target experiments a direct trust consequence:

> **Integration mode is part of the trust subject. Two adapters can be equally conformant while requiring radically different local authority.**

That claim should survive into the final public paper.

---

## 16. Response-paper synthesis direction

The emerging response is now strongest when treated as one argument rather than four lane reports:

~~~text
L1:
  shared labels need versioned/testable semantic meaning

L2:
  mappings need typed authority, evidence, drift detection,
  generated contract glue and explicit local hooks

L3:
  durable mappings and installable translators need
  ownership, permissions, current authorization,
  privacy boundaries and separated governance

L4:
  all compatibility claims need independent,
  replayable and freshness-bound evidence
~~~

The resulting thesis is:

> **Signet 2 has selected the right scaling architecture. The remaining work is to make every semantic, behavioral, trust and evidence boundary explicit enough that independent implementations can reproduce or falsify the claim.**

The compact architecture is:

~~~text
deterministic core
+ adaptive edge
+ explicit trust boundary
+ reproducible evidence
~~~

This is now a stronger target than any lane-specific paper alone.
