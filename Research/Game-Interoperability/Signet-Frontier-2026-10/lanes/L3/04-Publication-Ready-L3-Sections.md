# Publication-Ready L3 Sections for the Signet 2 Response Paper

**Status:** synthesis-ready prose  
**Date:** 2026-10-05  
**Target paper:** Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md  
**Observed target state:** research/signet-l2-adapters-ai, blob 797984b006d7b42e6de9564e17974c0cb3ff067e  
**L3 role:** trust, distribution, privacy, legal/policy boundaries, governance

This file is prose to merge into the current public response. It should not be published as a competing paper.

---

# A. Abstract insertion

Insert after the current abstract explains that accepted mappings need provenance, fingerprints, conformance evidence, and invalidation conditions.

> The same principle extends beyond semantic correctness. A translation profile is authority-bearing configuration, and a publicly distributed translator is supply-chain software. A profile can change shared behavior without changing executable bytes; a valid signature can remain cryptographically valid after a publisher or release is revoked; an adapter can pass conformance while still having excessive local authority; and a locally approved mapping should not silently become shared training data. A public Signet ecosystem therefore also needs explicit profile ownership, package identity, permission enforcement, update freshness and revocation, local-first data boundaries, and governance powers that remain distinguishable even while the project is small.

Add this invariant near the current abstract thesis:

~~~text
VERIFIABLE_MAPPING
!= SAFE_PACKAGE
!= CURRENT_AUTHORIZATION
!= LEGAL_OR_POLICY_PERMISSION
~~~

This extends rather than replaces the current L2 argument.

---

# B. Insert after current Section 4: profile ownership is separate from profile contents

## Translation profiles need separate ownership layers

The current response correctly separates authority classes inside the profile:

- input binding;
- motion observation;
- semantic mapping;
- appearance mapping;
- session policy.

There is a second distinction that matters just as much:

> **Who owns the stored decision?**

A single mutable file beside the translator is too ambiguous once package updates, community pull requests, local calibration, and player overrides all exist.

A safer model is:

~~~text
publisher_profile
    immutable/versioned baseline shipped with a translator release

participant_overlay
    local user-owned pins and explicit overrides

session_resolution
    derived effective profile for one exact dependency cut
~~~

The publisher profile and participant overlay are authority sources.

The session resolution is a derived artifact.

### Why this matters

Suppose a player explicitly chooses:

~~~text
weapon.ranged -> minecraft:bow
~~~

and a later translator release changes its default to:

~~~text
weapon.ranged -> minecraft:crossbow
~~~

That is a legitimate publisher update.

It is not legitimate to rewrite the player's local pin and continue presenting the resulting value as a player decision.

The same rule applies more strongly to control mappings.

A package update that silently changes:

~~~text
local action X -> fire
~~~

to:

~~~text
local action X -> use
~~~

has changed behavior even if executable bytes are unchanged.

### Effective-profile identity

Compatibility evidence should bind the behavior actually used:

~~~yaml
effective_profile_basis:
  translator_package_digest: ...
  publisher_profile_digest: ...
  participant_overlay_digest: ...
  game_version_or_fingerprint: ...
  protocol_version: ...
  intent_vocabulary_revision: ...
  archetype_catalog_revision: ...
  calibration_revision: ...
  unresolved_entries: [...]
  effective_profile_digest: ...
~~~

The exact serialization is an implementation question.

The requirement is not:

> **Package bytes are not a sufficient identity for runtime behavior.**

~~~text
PACKAGE_BYTES != EFFECTIVE_PROFILE
~~~

A profile-only update can stale compatibility evidence.

A participant overlay can also become stale when the game, semantic definition, or calibration basis changes.

The correct state is then RECONFIRM_REQUIRED, not silent migration.

---

# C. Insert in current Section 7: clarify suggest mode and simulation authority

The Signet 2 proposal has two rules that need to be read together:

1. in suggest mode, a resolver's best candidate may be used provisionally;
2. simulation-affecting intents must come from pinned decisions.

Those rules are compatible if provisional authority is scoped correctly.

### Provisional presentation is different from provisional simulation

For a local appearance choice:

~~~text
weapon.ranged -> minecraft:bow
~~~

a provisional result may be harmless because the consequence is local presentation.

For a simulation-relevant mapping:

~~~text
local event -> fire
~~~

the resolver's top score must not become authoritative merely because it is the best candidate.

A useful default policy is:

~~~text
presentation miss:
    provisional local choice may be permitted

simulation-affecting miss:
    NO_AUTHORITATIVE_ACTION
    or IGNORE
    or OBSERVE_ONLY
    or REQUIRE_PIN
~~~

This is the trust consequence of the proposal's own determinism rule.

A closed candidate list bounds syntax.

It does not grant authority.

~~~text
MODEL_SUGGESTION != AUTHORITY
~~~

---

# D. Insert after current Section 8: local approval is not training consent

## Local approval and dataset contribution are different acts

The Signet 2 proposal makes an important privacy-preserving choice for Forge training data:

- names and identifiers only;
- no textures, sounds, models or other game files;
- no usernames;
- no input logs;
- open-license contribution.

Those are good export-minimization rules.

One additional boundary is needed.

A user who accepts a mapping locally is answering:

> **Should my translator use this choice?**

That is not automatically the same question as:

> **May this record be exported, licensed, retained, and used to train future shared models?**

Therefore:

~~~text
LOCAL_APPROVAL != DATASET_CONTRIBUTION
~~~

Training contribution should be an explicit operation.

A contributed example should identify at least:

~~~yaml
training_example:
  task_scope: ...
  game_or_adapter_basis: ...
  semantic_subject: ...
  candidate_set_revision: ...
  selected_candidate: ...
  contribution_license: ...
  provenance: ...
  created_at: ...
  invalidated_or_superseded_by: ...
~~~

### Invalidated mappings need lineage

Suppose an accepted mapping is later discovered to be wrong after a game update.

The package/profile system can mark the mapping stale.

The training system also needs to know that the example is no longer current positive evidence.

Otherwise the ecosystem can repeatedly relearn a mapping it already disproved.

A useful separation is:

~~~text
accepted_for_local_profile
accepted_for_publisher_profile
eligible_for_dataset
licensed_for_training
validated_training_example
invalidated_training_example
~~~

These states are related.

They are not interchangeable.

---

# E. New publication section: Public translators are supply-chain software

## Public translators are supply-chain software

Signet 2 is primarily an interoperability architecture.

A public ecosystem adds another question:

> **What authority does the code implementing that architecture receive on a stranger's machine?**

A translator, launcher, gateway, or setup tool may need to:

- inspect a game installation;
- read configuration or data exposed by the integration;
- create generated or staged files;
- start a game or controlled server;
- communicate with a local server;
- connect to a Signet session;
- update itself or its dependencies.

That is real local authority.

Passing a semantic conformance suite does not answer whether the package should receive it.

### Keep trust claims separate

The launcher or directory should never collapse these into one badge:

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
CURRENT_AUTHORIZATION != SAFE_ARTIFACT
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
COMPATIBILITY         != LEGAL_OR_POLICY_PERMISSION
~~~

Examples:

- a malicious artifact can have a valid publisher signature;
- an old release can retain a valid signature after the publisher key or release is revoked;
- a reproducible artifact can reproducibly contain a vulnerability;
- a protocol-conformant adapter can request excessive filesystem access;
- a sandboxed adapter can still be semantically incompatible;
- a technically interoperable integration can still be excluded from an official directory for policy reasons.

A useful user-facing interface therefore displays exact claims rather than a single score.

### Exact package subject

A public compatibility or trust record should identify the exact subject:

~~~yaml
package_subject:
  namespace: ...
  name: ...
  version: ...
  artifact_digest: ...
  platform: ...
  architecture: ...

  publisher:
    identity: ...
    namespace_authorization: ...

  integration_mode: ...

  dependencies:
    lock_digest: ...

  publisher_profile_digest: ...

  requested_permissions: [...]

  update:
    channel: ...
    repository_target: ...
~~~

The specific schema may change.

The requirement is that evidence attaches to an exact package/profile subject rather than a human-readable project name.

---

# F. Signing is necessary but not sufficient

## Signing, current authorization, provenance, and reproducibility answer different questions

A public update system needs more than a signature check.

A correctly signed release can become inappropriate to run because of:

- publisher-key compromise;
- release-specific security revocation;
- dependency compromise;
- rollback to an older vulnerable target;
- frozen repository metadata;
- emergency supersession.

A TUF-shaped update model is useful because it explicitly separates root trust, target authorization, repository state, freshness, rollback, freeze, and key-compromise recovery.

Sigstore-style signing can answer a different question:

> Which human or CI/workload identity signed this release, and is that event transparently recorded?

SLSA-style provenance answers:

> Where and how was this artifact built?

Independent reproducibility answers:

> Can another party recreate the same artifact from the declared source/build conditions?

These compose:

~~~text
update currentness
+ signer identity
+ build provenance
+ independent reproduction
+ runtime confinement
+ conformance
+ compatibility evidence
~~~

They are not substitutes for one another.

A public directory can summarize them.

It should still preserve the individual states.

---

# G. New publication section: permissions should be brokered

## Permissions should be brokered, not merely declared

A package manifest can say:

~~~text
read game
write work directory
connect network
~~~

but a declaration is only useful if the runtime can constrain the package accordingly.

The portable interface should prefer symbolic resources:

~~~text
GAME_INSTALL_READ
GAME_INSTALL_WRITE_STAGED
TRANSLATOR_WORKDIR_RW
SIGNET_SESSION_NETWORK
CONTROLLED_SERVER_PROCESS
USER_SELECTED_FILE_READ
~~~

rather than package-selected arbitrary absolute paths.

The launcher resolves those symbols into host-specific resources after policy or user approval.

A useful architecture is:

~~~text
downloaded translator worker
        |
        | narrow broker RPC
        v
privileged launcher broker
        |
        +-- approved game resources
        +-- translator work directory
        +-- current Signet session
        +-- approved game/server process
~~~

The worker should not receive ambient access merely because the launcher itself has it.

### Report enforcement, not intent

Cross-platform implementations will differ.

The trust record should report what was actually enforced:

~~~text
ENFORCED
PARTIALLY_ENFORCED
DECLARED_ONLY
NOT_SUPPORTED
FAILED
UNKNOWN
~~~

and expose the runtime class:

~~~text
SEPARATE_SANDBOX_ENFORCED
HOST_API_CONFINED
HOST_PROCESS_CODE
FULL_APPLICATION
UNCONFINED
UNKNOWN
~~~

This makes one important fact visible:

> **Official integration and OS-isolated integration are different properties.**

---

# H. Real targets show why integration mode belongs in the trust model

The current response paper now contains two useful adapter experiments:

1. a Godot 4.7.2 GDExtension target;
2. Signet's documented Minecraft controlled-server/RCON gateway.

They are not merely implementation examples.

They demonstrate two very different trust surfaces.

## Godot GDExtension: host-process code

The Godot experiment correctly refuses to fabricate game-level capabilities from a generic engine extension.

That same restraint should apply to security claims.

A normal GDExtension executes inside the host application's process.

For trust classification, that is approximately:

~~~yaml
integration:
  mechanism: native_engine_extension
  confinement_class: HOST_PROCESS_CODE
~~~

It may be well-designed, signed, reproducible, and constrained by the host API.

It is not a separately kernel-sandboxed worker merely because the Signet package manifest requests few permissions.

If an ecosystem wants stronger isolation, it must deliberately move untrusted logic behind a process boundary rather than describe in-process code as sandboxed.

## Minecraft RCON gateway: controlled-server administration

The Minecraft experiment is qualitatively different.

The documented gateway can:

- construct or change the world through commands;
- inspect player or actor state;
- teleport or reconcile actors;
- apply damage;
- kill actors;
- observe selected gameplay signals.

That is not ordinary read-only adapter authority.

It is a controlled-server administration surface.

The package trust model should represent that honestly:

~~~yaml
integration:
  mechanism: rcon_gateway

authority:
  controlled_server_command: true
  world_mutation: true
  actor_reposition: true
  damage_application: true
  death_application: true
~~~

The exact operation names can follow the emerging L2 role contract.

The important L3 rule is:

> **The launcher must not turn gateway into an opaque trust category. The exact local/server authority should be inspectable.**

## RCON credentials are a separate secret

A gateway that requires an RCON password should not automatically receive every other launcher secret.

Prefer:

- a dedicated controlled-server credential;
- minimum necessary scope;
- local storage through the launcher or OS secret mechanism;
- no telemetry or log export;
- redaction from crash reports;
- rotation when practical;
- process-bound delivery rather than embedding in package files.

Where the protocol permits it, the launcher can mediate the privileged command channel rather than giving arbitrary package code the raw credential.

## Role contracts can help permission review, but they are not enforcement

The newer L2 role contract describes target operations under roles such as:

~~~text
HOST
IMPORTER
WORLD
INPUT
PRESENTATION
AUTHORITY
~~~

This is valuable input to trust review.

For example:

~~~text
WORLD + local RCON mechanism
-> likely controlled-server mutation authority

AUTHORITY + teleport/damage/kill operations
-> high-impact controlled-server authority

HOST + native GDExtension
-> in-process host-code risk
~~~

But:

~~~text
ROLE_CONTRACT != SANDBOX_POLICY
~~~

The operation descriptor can help generate or validate a permission manifest.

The launcher still needs an independent enforcement policy.

---

# I. Forge should be local-first

## Forge should be local-first, and remote execution should create an explicit data boundary

Signet 2 deliberately leaves the resolver location open:

- player's machine;
- shared service;
- development tool.

Those are not equivalent privacy models.

A good default is:

~~~text
runtime play:
    profile lookup only

calibration/review:
    local where practical

Forge development:
    local by default

remote/shared resolver:
    explicit opt-in with declared export schema

training contribution:
    separate explicit action
~~~

A remote resolver must not inherit access to everything that local calibration or game discovery can observe.

Example export declaration:

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

The exact fields can evolve.

The requirement is that remote execution creates a visible trust boundary.

### Identifiers only is minimization, not automatic anonymity

Game, mod, or item identifiers can reveal what software or content a person has installed.

Network endpoints can identify session participants.

Crash reports can accidentally contain local paths, tokens, account names, or environment data.

Therefore privacy documentation should distinguish:

~~~text
application payload
transport metadata
telemetry
crash reporting
remote Forge export
training-data contribution
~~~

Only meaning travels over the gameplay protocol can be true while other product surfaces still process personal or device data.

---

# J. Interoperability needs separated governance authorities

Signet's current governance is appropriately simple for a young project.

One maintainer currently has final say while the community forms, protocol proposals remain open for at least 14 days, and translator authors own their translator and license.

That is a workable bootstrap model.

The next architectural step is not necessarily to create a foundation immediately.

It is to name the powers separately.

A mature ecosystem contains at least:

### Protocol governance

Owns:

- wire contract;
- intent vocabulary;
- archetype catalog;
- semantic and profile versioning rules.

### Translator/profile governance

Owns:

- translator implementation;
- integration surface;
- publisher baseline profile;
- palette;
- calibration logic.

### Participant authority

Owns:

- local bindings;
- participant pins;
- local privacy and remote-Forge choices.

### Package-directory governance

Owns:

- listings;
- namespace authorization;
- evidence display;
- compatibility yanking;
- directory policy.

### Update trust-root authority

Owns:

- repository root keys and policy;
- publisher delegation;
- compromise recovery.

### Security incident authority

Owns:

- emergency quarantine;
- security revocation;
- advisory publication;
- unquarantine criteria.

### Forge/dataset/model governance

Owns:

- dataset admission;
- contribution licensing;
- example invalidation;
- model release identity.

### Conformance authority

Owns:

- test-suite revisions;
- PASS semantics;
- reproducibility requirements.

Initially the same people may occupy several of these roles.

The system should still not collapse the roles into one statement such as:

> maintainer approved

because that could otherwise ambiguously mean:

~~~text
protocol-standardized
package-listed
publisher-authorized
security-current
conformant
compatible
training-eligible
policy-approved
~~~

Those are different claims.

## Profile pull requests are behavioral supply-chain changes

Signet 2 notes that community members can improve profiles through pull requests.

A profile PR that changes a simulation-affecting mapping or a calibration assumption is not merely a documentation edit.

Release tooling should identify:

- exact profile subject;
- semantic diff;
- whether shared simulation can change;
- compatibility evidence made stale;
- tests or replays required;
- reviewer and approval evidence;
- resulting immutable profile digest.

This makes community improvement safer without preventing it.

---

# K. Package lifecycle needs more than latest version

A public directory should distinguish at least:

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

Examples:

- SUPERSEDED: newer version exists;
- DEPRECATED: maintainers recommend migration;
- YANKED_COMPATIBILITY: known-bad for a target/version, not necessarily malicious;
- SECURITY_REVOKED: should not launch automatically even if already installed;
- PUBLISHER_REVOKED: publisher namespace or key is no longer currently authorized;
- LEGAL_OR_POLICY_BLOCKED: official directory declines distribution for a declared scope.

This prevents compatibility problems from being mislabeled as malware and security incidents from being treated as merely old releases.

---

# L. Official integration policy should be explicit without pretending to be universal law

Signet 2 sets a deliberately conservative boundary:

- open-source engines;
- permitted mods or plugin APIs;
- controlled servers;
- no injection into online games;
- no anti-cheat bypass.

That is a strong official-project policy.

The response should preserve it.

It should also avoid turning the boundary into:

~~~text
outside_supported_modes == illegal
~~~

Interoperability law depends on jurisdiction and the exact technical action.

European software law, for example, contains specific observation, study, testing, and interoperability provisions subject to conditions.

Contract terms, anti-circumvention rules, online-service rules, redistribution, privacy, and trademark questions are separate again.

The useful package and directory representation is therefore:

~~~yaml
integration_policy:
  integration_mode: CONTROLLED_SERVER_GATEWAY

  technical_actions:
    observe_or_test: true
    parse_file_formats: false
    decompile_code: false
    circumvent_access_control: false
    modify_game_process: false
    connect_official_online_service: false
    redistribute_game_assets: false

  unresolved: [...]

  directory_disposition: ALLOW
~~~

Possible official-directory outcomes:

~~~text
ALLOW
ALLOW_WITH_USER_WARNING
REVIEW_REQUIRED
EXCLUDE_OFFICIAL_DIRECTORY
UNKNOWN
~~~

This is a distribution decision for a declared scope.

It is not a universal court judgment.

---

# M. Replace or expand the current conflicting-profile answer

## How should conflicting community profiles be resolved?

First determine whether they are profiles of the same subject.

Identity should include at least:

~~~text
translator package
publisher or namespace
game plus relevant version or fingerprint
protocol version
semantic profile, vocabulary, and catalog revision
integration mode
calibration basis
~~~

Two records with different subjects can coexist without conflict.

For one exact translator release, use:

~~~text
effective profile
=
publisher baseline
+
participant-owned overlay
+
session-compatible derivation
~~~

A community pull request can improve the publisher baseline for a new release.

It should not rewrite an already-installed participant overlay.

When a profile changes, classify the change:

~~~text
presentation-only
input-binding
calibration or motion
simulation-affecting semantic mapping
vocabulary or catalog migration
resolver or evidence metadata only
~~~

Simulation-affecting changes should:

- receive a new digest;
- stale the applicable compatibility evidence;
- rerun the required replay or conformance set;
- require reconfirmation where migration would replace a participant decision.

Multiple community or publisher profiles can coexist.

Popularity can help discovery.

It should not silently define semantic truth.

Do not synthesize a consensus profile unless the synthesis itself is deterministic, versioned, inspectable, evidence-bearing, and separately governed.

---

# N. Append to the current Proposed changes section

11. **Separate publisher profile from participant overlay.**
    - package updates do not silently overwrite player-owned pins;
    - effective-profile identity becomes part of compatibility evidence.

12. **Make simulation-affecting suggest-mode misses non-authoritative until pinned.**
    - provisional local appearance is different from provisional shared behavior.

13. **Make local approval distinct from training-data contribution.**
    - contribution is explicit, licensed, provenance-bearing, and invalidatable.

14. **Treat community profile PRs as behavioral release changes.**
    - semantic diff;
    - profile digest;
    - compatibility invalidation;
    - replay or conformance rerun where applicable.

15. **Add a public-package trust contract before one-click third-party installation scales.**
    - exact subject;
    - symbolic permissions;
    - enforcement evidence;
    - current update authorization;
    - revocation.

16. **Expose integration-specific confinement.**
    - in-process engine extension;
    - controlled-server gateway;
    - separate sandboxed worker;
    - full application or reimplementation;
    - unknown or unconfined.

17. **Require explicit data-export schemas for remote Forge or resolver services.**

18. **Separate protocol, package-directory, update-root, security, conformance, and dataset or model governance powers.**

19. **Distinguish compatibility yank, security revoke, publisher revoke, and policy block.**

20. **Represent official integration policy as scoped governance rather than one global legal Boolean.**

---

# O. Extend the concrete joint experiment with L3 proof

The existing paper's no-AI experiment is already the right starting point.

Add the smallest trust proof.

## Trust Step A — derive declared authority from the target role contract

For both real targets, record:

- integration mechanism;
- target operations;
- local or server resources required;
- expected confinement class.

Expected qualitative result:

~~~text
Godot GDExtension:
    HOST_PROCESS_CODE

Minecraft RCON gateway:
    separate gateway process
    + controlled-server command authority
    + protected RCON credential
~~~

## Trust Step B — enforce one negative permission

Examples:

- a separate helper or worker cannot read an unrelated user file;
- the Minecraft gateway cannot open undeclared arbitrary outbound Internet;
- package code cannot read the RCON secret from a general package file;
- a staged write cannot escape its approved resource.

The exact fixture should match the actual prototype architecture.

## Trust Step C — profile-only drift

Change one simulation-affecting profile mapping without changing executable bytes.

Expected:

~~~text
artifact_digest: unchanged
effective_profile_digest: changed
old compatibility evidence: SUSPECT or STALE
~~~

## Trust Step D — publisher update preserves participant overlay

Install publisher profile v1 plus a local participant pin.

Update to publisher profile v2.

Expected:

~~~text
publisher baseline: v2
participant overlay: preserved
effective profile: rederived
~~~

If semantic revisions no longer compose:

~~~text
RECONFIRM_REQUIRED
~~~

not silent overwrite.

## Trust Step E — local acceptance does not upload training data

Accept one resolver suggestion.

Expected default:

~~~text
local profile: updated
remote dataset: unchanged
~~~

Then perform an explicit contribution operation and inspect the export schema.

## Trust Step F — current authorization and revocation

Publish a test release and then security-revoke it in the test repository metadata.

Expected:

~~~text
signature validity: PASS
current authorization: REVOKED
automatic launch: BLOCK
~~~

This demonstrates the difference between cryptographic history and current policy.

---

# P. Joint invariants for the final paper

From the semantic, adapter, and validation work:

~~~text
SHARED_LABELS
!= SHARED_SEMANTICS

PINNED_CHOICE
!= CORRECT_CHOICE

SERVER_AUTHORITY
!= CHEAT_IMPOSSIBILITY

ONE_TRANSLATOR
!= UNIVERSAL_COMPATIBILITY

SELF_CONFORMANCE
!= PAIRWISE_INTEROPERABILITY
!= INDEPENDENT_REPRODUCTION
!= ECOSYSTEM_ADOPTION
~~~

From L3:

~~~text
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

COMPATIBILITY
!= LEGAL_OR_POLICY_PERMISSION
~~~

Each distinction names a separate evidence or authority boundary.

---

# Q. Revised final-paper thesis

A coherent final thesis is:

> **Signet 2 has selected the right scaling architecture: translate each game into shared meaning, keep shared simulation server-authoritative, resolve difficult mappings outside the real-time loop, and pin accepted decisions. The next step is to make every boundary around that architecture explicit and testable: semantic identity, mapping authority, package authority, compatibility evidence, data export, revocation, and governance.**

A compact version:

> **deterministic core, adaptive edge, explicit trust boundary, reproducible evidence**

The strongest recommendation is not replace Signet 2.

It is:

> **Finish the architecture by specifying who may change each durable decision, what exact behavior was tested, what code may do locally, and how every claim becomes stale or revoked.**

---

# R. Suggested revised conclusion paragraph

Use after the existing adapter-engineering conclusion:

> The same discipline should extend from translation correctness to ecosystem trust. A profile that changes shared behavior is a release artifact even when executable bytes stay the same. A resolver suggestion is evidence, not authority. A player's local approval is not implicit consent to contribute training data. A package signature records origin but does not prove that a release remains currently authorized or safe. And an integration that runs inside a game's process has a different confinement model from a separate controlled-server gateway or sandboxed worker. Making those boundaries explicit now lets Signet keep the simplicity of “calibrate once and play” for users while avoiding an ecosystem that depends on hidden mutable state and one overloaded “verified” label.

Finish with:

> **Shared meaning scales the protocol. Explicit authority and reproducible evidence are what let the ecosystem scale with it.**

---

# S. References to merge into the public paper

## Upstream Signet

- Signet 2: intents, archetypes and translation profiles, draft proposal v0.1, 2026-10-05.
- Signet Governance.
- Signet Security Policy.

## Supply chain

- The Update Framework — metadata roles and security model.
- Sigstore — signing, verification, transparency.
- SLSA v1.2.
- Reproducible Builds definition.

## Runtime confinement

- Microsoft AppContainer and Create Process In Sandbox.
- Linux Landlock userspace API.
- Apple App Sandbox.

## Privacy

- European Data Protection Board — basic data-protection principles.
- CJEU Breyer where endpoint or IP identifiability becomes material.

## Software interoperability and legal scope

- Directive 2009/24/EC.
- CJEU SAS Institute v World Programming.
- jurisdiction-specific implementation sources only where the final paper makes jurisdiction-specific claims.

These sources should support narrow claims.

Do not present adoption of a named framework as proof that the resulting implementation is secure.

---

# T. Synthesis acceptance criteria

The merged public paper is ready for external review when an outside reader can answer:

1. What exact shared meaning is being claimed?
2. What exact local operation maps to it?
3. Who owns or may change that mapping?
4. What exact package or profile behavior was tested?
5. Which evidence became stale after a game or profile change?
6. What authority does the translator receive locally?
7. Which parts are actually sandbox-enforced on this platform?
8. Who currently authorizes the installed release?
9. How is a compromised release revoked?
10. What data leaves the machine when Forge is local versus remote?
11. Did a local mapping approval become training data, and under what explicit contribution and license?
12. Which governance authority controls protocol semantics, directory listing, security revocation, conformance, and dataset or model admission?
13. Which official integration-policy claim is being made, and what remains legally unresolved?

If the paper can answer those questions while preserving Signet 2's simple end-user flow, the trust layer is doing its job.
