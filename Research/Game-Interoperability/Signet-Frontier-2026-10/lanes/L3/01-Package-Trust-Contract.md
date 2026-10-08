# L3 Package Trust Contract — draft v0.1

**Status:** PROPOSED implementation contract  
**Lane:** L3 — Trust, Distribution, Privacy, Legal & Governance  
**Date:** 2026-10-05  
**Purpose:** turn the L3 research findings into one machine-checkable package/update/runtime trust contract that L2 can implement and L4 can test.

This is not a final Signet specification. It is a research contract for evaluating a future public adapter/package ecosystem.

---

## 1. Core invariants

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
CURRENT_AUTHORIZATION != SAFE_ARTIFACT
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
COMPATIBILITY         != LEGAL_PERMISSION
PACKAGE_BYTES         != EFFECTIVE_PROFILE
PUBLISHER_PROFILE     != PARTICIPANT_PIN
MODEL_SUGGESTION      != AUTHORITY
~~~

A package can pass one claim and fail another.

The official directory/launcher should never emit a generic \`verified=true\` field.

---

## 2. Trust subject identity

Every consequential trust claim binds one exact subject.

~~~yaml
package_subject:
  package_id: org.example.minecraft
  package_version: 1.4.2
  artifact_digest: sha256:...
  manifest_digest: sha256:...
  publisher_namespace: org.example
  integration_mode: CONTROLLED_SERVER_GATEWAY
~~~

When a translation profile changes behavior, the effective trust subject also binds:

~~~yaml
profile_subject:
  publisher_profile_digest: sha256:...
  participant_overlay_digest: sha256:...
  effective_profile_digest: sha256:...
  protocol_version: Signet/2-draft
  intent_vocabulary_revision: ...
  archetype_catalog_revision: ...
  game_version_or_fingerprint: ...
  calibration_result_digest: sha256:...
~~~

A claim over artifact A cannot be reused for artifact B merely because the package ID/version string looks equal.

---

## 3. Package manifest

Recommended v0.1 shape:

~~~yaml
schema: signet.package/v0.1

package:
  id: org.example.minecraft
  name: Example Minecraft translator
  version: 1.4.2
  integration_mode: CONTROLLED_SERVER_GATEWAY

publisher:
  namespace: org.example
  source_repository: https://example.invalid/repo
  source_revision: 0123456789abcdef
  license: Apache-2.0

artifacts:
  - path: bin/translator
    digest: sha256:...
    platform:
      os: linux
      arch: x86_64

dependencies:
  lock_digest: sha256:...
  entries:
    - id: example-lib
      version: 2.3.1
      digest: sha256:...

permissions:
  filesystem:
    - resource: selected_game_installation
      access: read
      recursive: true
    - resource: package_workdir
      access: read_write
      recursive: true
    - resource: generated_mod_staging
      access: read_write
      recursive: true

  network:
    outbound:
      - resource: signet_session
    inbound: []

  process:
    - action: request_game_launch
      target: selected_game
      brokered: true

  ambient:
    home_directory: deny
    credential_stores: deny
    arbitrary_child_process: deny
    unrestricted_network: deny
    process_injection: deny
    debugging_other_processes: deny

profile:
  publisher_profile_path: profiles/default.json
  publisher_profile_digest: sha256:...
  profile_format: signet.translation-profile/v0.1

privacy:
  local_only:
    - installed_game_inventory
    - absolute_game_paths
    - game_assets
    - raw_input_logs
    - save_files
  optional_exports:
    - compatibility_result
    - redacted_crash_report
  telemetry_default: disabled

update:
  channel: stable
  repository_role: org.example
  rollback_allowed: false

evidence_refs:
  provenance: ...
  signature: ...
  source: ...
~~~

---

## 4. Manifest rules

1. Unknown required top-level schema version -> reject.
2. Duplicate object keys -> reject before semantic interpretation.
3. Artifact digest mismatch -> reject.
4. Dependency digest mismatch -> reject.
5. Raw absolute package-selected host paths are not accepted as portable capability requests.
6. \`selected_game_installation\`, \`package_workdir\`, \`generated_mod_staging\`, and \`signet_session\` are symbolic resources resolved by the launcher.
7. A package cannot upgrade \`deny\` ambient capabilities by adding another field elsewhere.
8. Unsupported permission semantics -> \`UNENFORCEABLE\`, not silently ignored.
9. Integration mode is part of trust evaluation.
10. A material profile change changes the effective behavioral subject even when executable bytes remain unchanged.

---

## 5. Resource resolution

Before launch, the broker resolves symbolic resources.

~~~yaml
resolved_resources:
  selected_game_installation:
    opaque_id: game-install-7
    display_name: Minecraft Java 1.21.x
    host_path: <launcher-private>
    access: read

  package_workdir:
    opaque_id: workdir-badf00d
    host_path: <launcher-private>
    access: read_write

  signet_session:
    opaque_id: session-4
    destination_class: exact_session_endpoint
~~~

The adapter receives handles, mounted paths, file descriptors, RPC endpoints, or platform equivalents.

The adapter should not need to know unrelated host filesystem topology.

---

## 6. Runtime confinement classes

~~~text
SEPARATE_SANDBOX_ENFORCED
HOST_API_CONFINED
HOST_PROCESS_CODE
FULL_APPLICATION
UNCONFINED
UNKNOWN
~~~

Mapping guidance:

| Integration mode | Default confinement class |
|---|---|
| LOCAL_REIMPLEMENTATION | SEPARATE_SANDBOX_ENFORCED when supported |
| CONTROLLED_SERVER_GATEWAY | SEPARATE_SANDBOX_ENFORCED when supported |
| OFFICIAL_API | SEPARATE_SANDBOX_ENFORCED when supported |
| OFFICIAL_MOD | HOST_API_CONFINED or HOST_PROCESS_CODE |
| OPEN_ENGINE | FULL_APPLICATION |
| UNSUPPORTED_INJECTION | UNCONFINED / official ecosystem excluded |

The launcher must report the actual enforced result, not only the intended mode.

---

## 7. Enforcement evidence

~~~yaml
sandbox_evidence:
  evidence_version: 1
  package_subject: ...
  launcher_version: ...
  platform:
    os: linux
    kernel: ...
  mechanism:
    - mount_namespace
    - seccomp
    - landlock_abi_10

  requested:
    filesystem:
      selected_game_installation: read
      package_workdir: read_write
    network:
      - signet_session
    arbitrary_child_process: deny

  result:
    filesystem: ENFORCED
    network: ENFORCED
    process: ENFORCED
    credential_store: ENFORCED

  gaps: []
  observed_at: ...
~~~

Allowed result vocabulary:

~~~text
ENFORCED
PARTIALLY_ENFORCED
DECLARED_ONLY
NOT_SUPPORTED
FAILED
UNKNOWN
~~~

A missing enforcement mechanism is not a PASS.

---

## 8. Trust claim record

~~~yaml
trust_claim:
  claim_id: ...
  claim_type: SANDBOX_PROFILE_ENFORCED

  subject:
    package_id: ...
    package_version: ...
    artifact_digest: sha256:...
    effective_profile_digest: sha256:...

  scope:
    os: linux
    launcher_profile: sandbox-v1

  result: PASS

  evidence_refs:
    - ...

  verifier:
    identity: ...
    tool_version: ...

  observed_at: ...
  expires_at: ...
  supersedes: ...

  state: CURRENT
~~~

### Claim types

Origin / identity:

~~~text
ARTIFACT_DIGEST_MATCH
SIGNATURE_VALID
PUBLISHER_IDENTITY_VERIFIED
DIRECTORY_NAMESPACE_AUTHORIZED
UPDATE_METADATA_CURRENT
~~~

Source / build:

~~~text
SOURCE_REVISION_DECLARED
SOURCE_AVAILABLE
BUILD_PROVENANCE_VERIFIED
REPRODUCIBLE_BUILD_CLAIMED
REPRODUCIBLE_BUILD_INDEPENDENTLY_VERIFIED
DEPENDENCY_SET_LOCKED
~~~

Runtime:

~~~text
PERMISSIONS_DECLARED
SANDBOX_PROFILE_ENFORCED
INSTALL_TRANSACTION_ROLLBACK_TESTED
UNDECLARED_NETWORK_TEST_PASS
~~~

Compatibility:

~~~text
PROTOCOL_CONFORMANCE_PASS
GAME_VERSION_COMPATIBILITY_REPRODUCED
PLATFORM_COMPATIBILITY_REPRODUCED
PROFILE_CONFORMANCE_PASS
~~~

Policy / legal:

~~~text
OFFICIAL_INTEGRATION_MODE_ALLOWED
LICENSE_METADATA_PRESENT
LEGAL_SCOPE_REVIEWED
LEGAL_REVIEW_REQUIRED
PROVIDER_TERMS_REVIEW_REQUIRED
~~~

Advisory only:

~~~text
STATIC_SCAN_PASS
MALWARE_SCAN_PASS
COMMUNITY_REPORT_STATUS
~~~

---

## 9. Claim state

~~~text
ASSERTED
CURRENT
STALE
SUPERSEDED
REVOKED
FAILED
UNKNOWN
NOT_APPLICABLE
~~~

State transitions are claim-specific.

Examples:

~~~text
CURRENT compatibility
-> STALE after game fingerprint movement

CURRENT publisher identity
-> REVOKED after identity compromise

CURRENT sandbox claim
-> STALE after launcher/runtime policy change

CURRENT profile conformance
-> STALE after publisher profile digest changes
~~~

---

## 10. Update metadata contract

The implementation may use TUF directly or a mechanically equivalent model.

Required security properties:

- rooted trust;
- delegated package namespaces;
- exact target digests;
- consistent repository snapshot;
- short-lived freshness evidence;
- rollback detection;
- freeze detection;
- compromised-role rotation/revocation;
- threshold/offline-root recovery where policy requires it.

Minimal conceptual state:

~~~yaml
update_basis:
  root_generation: ...
  root_digest: ...
  targets_generation: ...
  targets_digest: ...
  snapshot_generation: ...
  snapshot_digest: ...
  timestamp_generation: ...
  timestamp_expires_at: ...
  delegated_namespace: org.example
  exact_target_digest: sha256:...
~~~

A timeless package signature does not replace update currentness.

---

## 11. Package lifecycle

~~~text
ACTIVE
DEPRECATED
YANKED_COMPATIBILITY
COMPATIBILITY_INVALIDATED
SECURITY_REVOKED
PUBLISHER_REVOKED
LEGAL_OR_POLICY_BLOCKED
SUPERSEDED
~~~

Semantics:

### DEPRECATED

- new installs discouraged;
- execution may remain allowed.

### YANKED_COMPATIBILITY

- not selected for new compatible installs;
- exact historical reproduction may remain available;
- not equivalent to compromise.

### COMPATIBILITY_INVALIDATED

- evidence no longer supports the bound game/platform/profile;
- package may remain installable for other scopes.

### SECURITY_REVOKED

- automatic launch blocked for affected subject;
- reinstall blocked;
- evidence/history retained;
- uninstall/rollback remains available.

### PUBLISHER_REVOKED

- old publisher generation cannot authorize new current releases;
- historical signatures remain historical evidence.

### LEGAL_OR_POLICY_BLOCKED

- official directory/launcher blocks declared scope;
- state does not assert universal illegality.

---

## 12. Compromised publisher recovery

~~~text
compromise detected
-> revoke publisher generation in current directory/update authority
-> advance trusted update metadata
-> quarantine affected versions according to incident scope
-> authorize successor publisher identity from current root
-> require new release/provenance for current trust
~~~

A publisher cannot self-unrevoke by:

- uploading identical bytes under a new version;
- creating a new key without directory authorization;
- editing its own profile;
- changing package namespace spelling.

---

## 13. Install transaction

Installer-owned mutation must be a transaction-like record.

~~~yaml
install_transaction:
  transaction_id: ...
  package_subject: ...
  resolved_targets:
    - resource: generated_mod_target
      target_ref: ...
  pre_state:
    - target_ref: ...
      digest_or_absence: ...
  staged_outputs:
    - source_ref: ...
      digest: ...
  applied_outputs:
    - target_ref: ...
      digest: ...
  backup_refs:
    - ...
  outcome: COMMITTED
  rollback_plan: ...
~~~

Rules:

- package writes to staging;
- broker applies approved outputs;
- existing state is inventoried first;
- cleanup/rollback only touches transaction-owned targets;
- ambiguous application outcome is reconciled before retry;
- uninstall does not broadly delete directories it did not create.

---

## 14. Translation profile authority

Store publisher baseline and participant choices separately.

~~~text
publisher_profile
+ participant_overlay
-> derived effective_profile
~~~

### Publisher profile

- package/release controlled;
- signed/current under package policy;
- subject to profile conformance and compatibility testing.

### Participant overlay

- user controlled;
- local by default;
- package update cannot silently replace it;
- not automatically contributed to training.

### Resolver suggestion

- non-authoritative;
- exact model/tool identity retained when a suggestion is accepted.

### Effective profile

Derived for one exact dependency cut.

~~~yaml
effective_profile:
  package_subject: ...
  publisher_profile_digest: ...
  participant_overlay_digest: ...
  game_fingerprint: ...
  vocabulary_revision: ...
  archetype_catalog_revision: ...
  calibration_result_digest: ...
  unresolved_entries: []
  effective_profile_digest: sha256:...
~~~

If any required component is incompatible or stale, effective-profile derivation fails.

---

## 15. Profile update semantics

A profile-only change is a behavioral update.

A release pipeline must detect at least:

~~~text
simulation-affecting mapping changed
appearance-only mapping changed
fallback changed
calibration basis changed
profile dependency revision changed
provisional -> pinned
pinned -> removed
~~~

Simulation-affecting change:

~~~text
old compatibility PASS
-> STALE
-> rerun applicable replay/conformance
-> publish new claim
~~~

User-facing update UI should distinguish:

~~~text
EXECUTABLE_CHANGED
PERMISSIONS_CHANGED
SIMULATION_MAPPING_CHANGED
APPEARANCE_MAPPING_CHANGED
DEPENDENCY_CHANGED
NO_MATERIAL_BEHAVIOR_CHANGE
UNKNOWN
~~~

---

## 16. Forge / training-data boundary

Local participant state is not training data by default.

A contributed mapping example should bind:

~~~yaml
mapping_example:
  example_id: ...
  semantic_subject: ...
  candidate_set_revision: ...
  selected_candidate: ...
  game_or_translator_basis: ...
  profile_revision: ...
  approval_source: HUMAN
  contribution_source_ref: ...
  contribution_license: ...
  privacy_projection: ...
  invalidated_by: ...
~~~

Training eligibility requires:

- explicit contribution;
- applicable license;
- provenance;
- privacy-minimized record;
- current/not-invalidated semantic basis, or explicit negative/historical labeling.

Conflicting examples may coexist.

Popularity is not semantic correctness.

---

## 17. Privacy declaration

~~~yaml
privacy_contract:
  schema: signet.privacy/v0.1

  local_only:
    - installed_game_inventory
    - absolute_paths
    - game_assets
    - save_files
    - raw_input_logs

  session_required:
    - session_identity
    - neutral_intents
    - neutral_state_events
    - negotiated_profile_metadata

  optional_telemetry:
    default: disabled
    fields:
      - adapter_version
      - game_version
      - conformance_result
    purpose: compatibility_improvement
    retention: bounded

  crash_reports:
    default: ask_or_opt_in
    redaction:
      - home_path
      - username
      - token_like_strings
      - cookies
      - raw_environment
~~~

Remote Forge/model execution must declare an additional export schema instead of inheriting local discovery access.

---

## 18. Network privacy mode

~~~yaml
network_privacy:
  mode: DIRECT_PEER
  disclosed_to:
    - peers
  relay_present: false
~~~

Allowed mode vocabulary:

~~~text
DIRECT_PEER
RELAYED
PRIVATE_SERVER
PUBLIC_LOBBY
LOCAL_ONLY
~~~

The UI describes audiences, not marketing labels.

Example:

> Direct peer: players you connect to can learn the connection address used by the session.

---

## 19. Integration policy record

~~~yaml
integration_policy:
  game_or_product: ...
  package_id: ...
  integration_mode: LOCAL_REIMPLEMENTATION

  jurisdiction_profiles:
    - EU
    - NL

  technical_actions:
    observe_or_test: true
    parse_file_formats: true
    decompile_code: false
    circumvent_access_control: false
    modify_game_process: false
    connect_official_online_service: false
    redistribute_game_assets: false

  evidence_refs:
    copyright: [...]
    contract_or_eula: [...]
    anti_cheat: [...]
    store_or_platform: [...]
    trademark: [...]

  unresolved: [...]

  directory_disposition: ALLOW
  evidence_cutoff: ...
~~~

Allowed directory dispositions:

~~~text
ALLOW
ALLOW_WITH_USER_WARNING
REVIEW_REQUIRED
EXCLUDE_OFFICIAL_DIRECTORY
UNKNOWN
~~~

This is a distribution decision for a declared scope, not a universal legal judgment.

---

## 20. User-facing trust summary

The launcher may derive a compact display, but it must preserve exact distinctions.

Example:

~~~text
Publisher
  Identity: verified for org.example
  Current release authorization: current

Build
  Provenance: verified
  Independent reproducibility: not yet reproduced

Permissions
  Game installation: read-only
  Work directory: read/write
  Network: current Signet session only
  Runtime confinement: kernel-enforced on this system

Compatibility
  Signet profile: PASS
  Game 1.21.x: reproduced
  Evidence age: 3 days

Profile
  Publisher profile: current
  Your local overrides: 2
  Simulation-affecting changes since last version: none

Policy
  Integration mode: controlled server gateway
  Official directory: allowed for declared scope
  Legal status: scope reviewed; not a universal legality guarantee
~~~

No total trust score.

---

## 21. Verification algorithm sketch

~~~text
resolve exact package subject

verify current update root
verify delegated namespace
verify exact target digest

verify publisher identity/signature policy
verify manifest/artifact digests
verify dependency lock/digests
verify build provenance if claimed

resolve symbolic resources
calculate requested permission set
construct platform sandbox
measure actual enforcement result

resolve current publisher profile
resolve compatible participant overlay
derive effective profile
bind game/protocol/catalog/calibration revisions

load exact trust claims
invalidate claims whose subject/currentness moved

evaluate integration/directory policy

derive launch decision:
  ALLOW
  ALLOW_WITH_WARNING
  BLOCK_STALE
  BLOCK_REVOKED
  BLOCK_UNENFORCEABLE
  BLOCK_POLICY
  BLOCK_AMBIGUOUS
~~~

---

## 22. Deterministic fixture minimum

### Package/update

1. artifact digest mismatch -> block;
2. dependency digest mismatch -> block;
3. valid old signature after publisher revocation -> block current authorization;
4. stale timestamp metadata -> stale/fail according to policy;
5. repository rollback -> reject;
6. package security revoked -> no automatic launch;
7. compatibility-yanked package -> distinguish from security revoke;
8. lost publisher key -> successor requires current root authorization.

### Permission

9. home-directory read attempt -> denied;
10. credential-store read -> denied;
11. undeclared outbound network -> denied;
12. undeclared child process -> denied;
13. symlink/reparse escape -> denied;
14. staging output targets unrelated file -> broker rejects;
15. platform cannot enforce declared network boundary -> no sandbox PASS.

### Profile

16. profile-only simulation mapping change -> old compatibility stale;
17. package update overwrites participant overlay -> rejected;
18. stale overlay semantic revision -> reconfirm required;
19. executable/profile mixed rollback -> fail coherent-cut check;
20. unsigned mutable profile with signed executable -> profile non-authoritative;
21. resolver suggestion for simulation intent without pin -> no authoritative action;
22. game version moves -> affected profile/compatibility claim stale.

### Privacy

23. installed-game inventory -> no default telemetry export;
24. crash report contains token/home path -> redacted before upload;
25. direct-peer mode -> endpoint audience declared;
26. remote Forge enabled -> exported field schema displayed.

### Governance/policy

27. directory removes package -> protocol semantics unchanged;
28. protocol changes -> directory listing does not imply compatibility;
29. package author self-asserts PASS -> no verifier claim created;
30. policy record unresolved -> REVIEW_REQUIRED / fail closed for official distribution.

---

## 23. L2 handoff

L2 can implement against this contract by prototyping:

1. one brokered local reimplementation worker;
2. one controlled-server gateway;
3. symbolic resource resolution;
4. a platform enforcement report;
5. publisher-profile + participant-overlay derivation;
6. install staging/transaction recording.

The prototype should not invent protocol semantics that belong to L1.

---

## 24. L4 handoff

L4 can validate this contract with:

- exact package fixture corpus;
- rollback/freeze/revocation simulation;
- cross-platform sandbox escape tests;
- profile poisoning/change tests;
- privacy canaries;
- independent rebuild/provenance checks;
- stale evidence invalidation.

A PASS should always identify the exact claim type and exact subject.

---

## 25. Open implementation questions

- exact canonical serialization for package/profile manifests;
- whether TUF is adopted directly or via an equivalent repository format;
- exact Sigstore identity policies for human releases versus CI releases;
- minimum Windows versions for the preferred sandbox implementation;
- Linux fallback policy for older Landlock ABIs;
- macOS helper architecture for untrusted translator logic;
- how an official mod expresses host-process-code risk without implying OS isolation;
- exact UI semantics for offline trust freshness;
- who owns high-impact package security revocation initially;
- how package/profile identity composes with Signet 2 negotiation on the wire.

These are explicit unknowns, not implementation defaults.
