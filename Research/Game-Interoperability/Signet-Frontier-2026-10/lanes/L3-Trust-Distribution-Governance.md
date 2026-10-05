# L3 — Trust, Distribution, Privacy, Legal & Governance

**Continuation pass:** 2026-10-05  
**Lane:** L3  
**Evidence labels:** OBSERVED / DERIVED / PROPOSED / UNKNOWN  
**Scope:** public adapter/package ecosystem trust, runtime confinement, privacy, distribution policy, legal uncertainty, and governance. Gameplay semantics remain L1; adapter implementation mechanics remain L2; experimental validation remains L4.

## Mission

Determine how an open cross-game ecosystem can let strangers discover, install, execute, update, revoke, and govern adapters without turning convenience into an opaque software-supply-chain risk.

## Executive delta

This pass changes the recommended architecture in five material ways.

1. **Do not give an adapter ambient launcher authority.** Use a privileged broker plus a sandboxed adapter worker. The package asks for typed capabilities; the launcher resolves those capabilities to user-selected game installations, session endpoints, staging directories, and process actions.
2. **Do not use one “verified” badge.** Publisher identity, signature validity, update freshness, source provenance, reproducibility, malware scanning, runtime confinement, protocol conformance, game-version compatibility, and legal/integration-policy status are separate claims with separate evidence.
3. **Use complementary supply-chain layers.** TUF-like repository metadata protects current update state and rollback/freeze/key-compromise recovery; Sigstore can bind an artifact to an authenticated signing identity and transparency evidence; SLSA provenance describes how the artifact was built; reproducible builds let independent parties compare output. None of these alone proves safety.
4. **Legal status must be a scoped record, not a boolean.** EU and U.S. law both contain interoperability-specific software provisions, but copyright, anti-circumvention, contract/EULA, anti-cheat/service rules, redistribution, trademark, privacy, and jurisdiction remain separate questions.
5. **Separate governance roots now, even if one maintainer currently fills several roles.** Protocol semantics, conformance, package directory, update/revocation, trademark, and security response should be independently modelled so ecosystem growth does not require one all-powerful “official” role.

The result is a user-facing trust model that can truthfully answer:

> Who published this? Which bytes am I running? How were they built? What may they access? Which claims were independently reproduced? What can be revoked? What data leaves my machine? Which policy/legal questions remain unresolved?

---

# 1. Current project evidence

## 1.1 Signet

**OBSERVED:** current Signet governance says the young project has one maintainer with final say while the community forms. Protocol proposals remain open for at least 14 days; accepted Signet/1 changes are additive. Translator authors own their translator, license, and manifest.

Primary source:
- https://github.com/kian-cx/signetprotocol/blob/main/GOVERNANCE.md

**OBSERVED:** Signet's beta security policy says the reference server has no authentication and should not be exposed to the Internet unless that risk is accepted.

Primary source:
- https://github.com/kian-cx/signetprotocol/blob/main/SECURITY.md

**OBSERVED:** Signet explicitly prefers open engines, officially allowed mods, controlled gateways, and reimplementations; it rejects injection into online games, anti-cheat bypass, official-server modification, and redistribution of game files.

Primary sources:
- https://github.com/kian-cx/signetprotocol/blob/main/README.md
- https://github.com/kian-cx/signetprotocol/blob/main/docs/content/concepts/integration.mdx

**DERIVED:** this is a strong technical/policy baseline, but it does not yet define a package distribution trust root, runtime permission enforcement, compromised-publisher recovery, or independent directory governance.

## 1.2 Melty

**OBSERVED:** Melty currently demonstrates the opposite end of the UX spectrum: one-click discovery/install/launch, creator publication, automatic setup, multiplayer links, and coding-agent publication.

Primary sources:
- https://melty.gg/
- https://melty.gg/terms
- https://melty.gg/privacy

**OBSERVED:** Melty Terms updated 2026-10-03 state that its app may discover games, download dependencies/tools, write game and other files, build local components, copy owned-game files into working areas, launch games/programs/setup steps, and auto-update. The Terms also state that mashup programs run with ordinary program access and that Melty does not malware-scan mashups before publication beyond limited upload checks based on file names/types.

**OBSERVED:** Melty Privacy says installed-game detection and owned-game file discovery happen locally, while account/activity, install/play reports, multiplayer IP/port, crash/error data, and some website analytics are processed by the service.

**DERIVED:** Melty is useful evidence for the authority a convenient launcher wants, but its current trust model is not the target for an “official safe adapter ecosystem.” The useful lesson is to preserve the UX while reducing ambient authority.

---

# 2. Launcher/package threat model

A credible L3 design should assume any one of these can fail independently.

## 2.1 Adversaries and failure sources

- malicious translator/package author;
- compromised legitimate publisher account or signing identity;
- compromised package directory;
- compromised update metadata service;
- compromised CI/build runner;
- malicious or substituted dependency;
- stale but valid old package;
- game update that invalidates a previously safe adapter assumption;
- malicious world/session/server;
- parser exploit in an adapter;
- sandbox escape;
- launcher bug granting more authority than declared;
- directory moderator or protocol maintainer capture;
- compromised security-response account;
- legal/policy takedown pressure applied through the wrong governance layer;
- privacy leak through telemetry, crash data, paths, game inventory, IP addresses, or stable identifiers.

## 2.2 Protected assets

- user files outside selected game installations;
- credentials, SSH keys, browser/session tokens, wallets, keychains;
- saves and game configuration;
- game installations and anti-cheat-sensitive state;
- other processes;
- local network and Internet reachability;
- account/game ownership metadata;
- installed-game inventory;
- home paths and usernames;
- IP/port and session topology;
- package update trust roots;
- protocol meaning and conformance truth;
- package-directory listing state;
- historical security/revocation evidence.

## 2.3 Core invariant

**PROPOSED:**

> Package metadata can request authority; only the launcher/runtime policy can grant it.

A manifest declaring “read game files” does not itself grant filesystem access. A package declaring “network: localhost” does not itself prove confinement. A valid signature does not upgrade either.

---

# 3. L3.1 — Permission model

## 3.1 Brokered capability architecture

The portable design should not expose raw host paths or unrestricted process launch directly to adapter code.

**PROPOSED architecture:**

~~~text
user / launcher UI
        |
        v
privileged launcher broker
  - discovers installations
  - resolves user selections
  - verifies package/update evidence
  - creates sandbox
  - owns process launch
  - owns privileged file placement
  - owns session network handles
        |
 typed capability RPC
        |
        v
sandboxed adapter worker
  - parses selected game data
  - translates Signet state/intents
  - writes only staging/workdir
  - receives no ambient home/credential access
~~~

The broker should pass **resource handles** or symbolic capabilities, not package-selected absolute paths.

Bad:

~~~yaml
filesystem:
  read:
    - C:\Users\Alice
~~~

Preferred:

~~~yaml
permissions_version: 1

filesystem:
  - resource: selected_game_installation
    access: read
    recursive: true
  - resource: package_workdir
    access: read_write
  - resource: generated_mod_staging
    access: read_write

network:
  outbound:
    - resource: signet_session
  inbound: []

process:
  - action: request_game_launch
    executable: selected_game
    brokered: true

ipc:
  - channel: launcher_broker

ambient:
  home_directory: deny
  credential_stores: deny
  arbitrary_child_process: deny
  unrestricted_network: deny
~~~

The package declares the class of resource it needs. The user/launcher resolves “selected_game_installation” to a concrete installation and can display that exact resolved target before first use.

## 3.2 Why brokered staging matters

For an official mod that requires files inside a game installation, direct write access should not be the default.

Preferred flow:

~~~text
adapter generates files
-> package-specific staging directory
-> launcher validates declared output set
-> launcher snapshots/reconciles existing target state
-> launcher copies exact staged outputs into approved mod/config location
-> uninstall uses the recorded transaction
~~~

This does not make malicious generated output safe, but it prevents a translator from receiving arbitrary write authority over the whole game tree merely because one mod file must be installed.

## 3.3 Process authority

Treat these separately:

- run the adapter worker;
- request launch of a selected game;
- run a declared build tool;
- spawn arbitrary children;
- inject into another process;
- attach/debug another process.

**PROPOSED:** only the first three are potentially normal official-ecosystem capabilities. Arbitrary children are denied by default. Injection/debug/anti-cheat bypass remain outside official policy.

A game launch should be a broker action with:

~~~text
selected executable identity
allowed argument template
working directory
environment allowlist
session handle(s)
expected child/process tree policy
~~~

Do not pass the participant's full environment by default; environment variables can carry tokens, paths, proxy settings, and other secrets.

---

# 4. Cross-platform enforceability

The portable manifest should describe intent at a higher level than any one OS primitive. The launcher then reports which parts are **actually enforced** on the current platform.

## 4.1 Windows

**OBSERVED:** Microsoft documents AppContainer and Less-Privileged AppContainer as kernel-enforced isolation for process, filesystem/registry, network, credentials, devices, and other processes unless access is explicitly granted.

Sources:
- https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer
- https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation

**OBSERVED:** current Microsoft documentation also exposes Create Process In Sandbox APIs with an AppContainer option plus explicit read-only/read-write filesystem paths and network policy.

Source:
- https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox

**UNKNOWN:** minimum Windows build/version and production maturity for that newer sandbox API need an implementation-specific validation pass before it becomes a hard Signet requirement.

**PROPOSED:** Windows launcher should prefer AppContainer/LPAC or the current supported CreateProcessInSandbox path for adapter workers. Privileged game discovery and game launch remain in the broker.

## 4.2 Linux

**OBSERVED:** Flatpak's sandbox defaults deny host files, network, devices, outside processes, broad D-Bus, and many syscalls; permissions and portals selectively restore access.

Sources:
- https://docs.flatpak.org/en/latest/basic-concepts.html
- https://docs.flatpak.org/en/latest/sandbox-permissions.html

**OBSERVED:** Linux Landlock is an unprivileged stackable LSM for restricting ambient process rights. Current kernel documentation includes filesystem restrictions plus TCP and newer UDP network restrictions by ABI version.

Source:
- https://www.kernel.org/doc/html/latest/userspace-api/landlock.html

**PROPOSED:** a native Linux launcher does not need to require Flatpak packaging, but should reuse the same design principles: private mount/process/network namespaces where appropriate, seccomp, minimal bind mounts, and Landlock as an additional in-process restriction where supported. A Flatpak build can use portals for user-mediated resources.

## 4.3 macOS

**OBSERVED:** App Sandbox is kernel-enforced and uses entitlements for network and file access; user-selected file access and security-scoped resources provide a narrower model than blanket host access.

Sources:
- https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox
- https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.app-sandbox
- https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.network.client

**UNKNOWN:** a robust mechanism for taking an arbitrary downloaded native translator binary and dynamically imposing an App-Sandbox-equivalent policy without making it part of a signed app/helper architecture is not established by this pass.

**PROPOSED:** design the macOS launcher around sandboxed signed helper targets / XPC-style separation rather than assuming a Linux-like “run arbitrary binary under policy file” primitive exists.

## 4.4 Enforcement disclosure

Every runtime should produce an enforcement result such as:

~~~yaml
sandbox_evidence:
  profile_version: 1
  platform: linux
  launcher_version: 0.2.0
  package_digest: sha256:...
  requested:
    filesystem: [selected_game_installation:read, package_workdir:read_write]
    network: [signet_session]
    child_process: deny
  enforced:
    filesystem: kernel_enforced
    network: kernel_enforced
    process_isolation: kernel_enforced
    credential_store: denied_by_namespace_and_path_policy
  gaps: []
  observed_at: ...
~~~

If a platform cannot enforce a requested boundary, the UI should say **DECLARED_ONLY** or **PARTIALLY_ENFORCED**, not silently present the same trust label.

---

# 5. Integration mode changes the sandbox claim

A single launcher policy cannot honestly claim the same containment for every Signet integration mode.

| Integration mode | Typical execution | Strong separate adapter sandbox? | Official default |
|---|---|---:|---|
| LOCAL_REIMPLEMENTATION | separate viewer/runtime reading owned files | yes, strongest fit | allow when package/evidence policy passes |
| CONTROLLED_SERVER_GATEWAY | adapter talks to user-controlled game/server API | usually yes | allow when exact network/process scope passes |
| OFFICIAL_API | separate client of documented API | usually yes | allow when API/terms scope passes |
| OFFICIAL_MOD | code executes inside game/mod host | often no separate OS boundary | allow only with explicit host-process-code warning/policy |
| OPEN_ENGINE | modified/open engine may itself be the runtime | package is effectively an application | treat as app distribution, not “sandboxed plugin” by default |
| UNSUPPORTED_INJECTION | process injection/hooking outside official support | no acceptable official boundary | exclude from official directory |

**PROPOSED:** add a trust dimension:

~~~text
RUNTIME_CONFINEMENT =
  SEPARATE_SANDBOX_ENFORCED
  HOST_API_CONFINED
  HOST_PROCESS_CODE
  FULL_APPLICATION
  UNCONFINED
  UNKNOWN
~~~

A package can be signed and reproducible while still being HOST_PROCESS_CODE.

---

# 6. L3.2 — Trust claim taxonomy

## 6.1 No generic “verified”

Flathub provides a useful naming example: its “verified” status specifically means the developer confirmed ownership/authorization for the app identity; it does not mean every other property is proven.

Sources:
- https://docs.flathub.org/docs/for-users/verification
- https://docs.flathub.org/docs/for-app-authors/verification

For Signet-like packages, expose independent claims.

### Origin / identity

- ARTIFACT_DIGEST_MATCH
- SIGNATURE_VALID
- PUBLISHER_IDENTITY_VERIFIED
- DIRECTORY_NAMESPACE_AUTHORIZED

### Source / build

- SOURCE_REVISION_DECLARED
- SOURCE_AVAILABLE
- BUILD_PROVENANCE_VERIFIED
- REPRODUCIBLE_BUILD_CLAIMED
- REPRODUCIBLE_BUILD_INDEPENDENTLY_VERIFIED
- DEPENDENCY_SET_LOCKED

### Runtime

- PERMISSIONS_DECLARED
- SANDBOX_PROFILE_ENFORCED
- NO_UNDECLARED_NETWORK_OBSERVED
- INSTALL_TRANSACTION_ROLLBACK_TESTED

### Compatibility / protocol

- PROTOCOL_CONFORMANCE_PASS
- GAME_VERSION_COMPATIBILITY_REPRODUCED
- OS_PLATFORM_COMPATIBILITY_REPRODUCED
- FALLBACKS_DECLARED

### Policy / legal

- OFFICIAL_INTEGRATION_MODE_ALLOWED
- REDISTRIBUTION_METADATA_PRESENT
- LEGAL_SCOPE_REVIEWED
- LEGAL_REVIEW_REQUIRED
- PROVIDER_OR_GAME_TERMS_UNRESOLVED

### Advisory signals

- STATIC_SCAN_PASS
- MALWARE_SCAN_PASS
- COMMUNITY_REPORTS_CLEAR

Advisory signals must never be rendered as equivalent to deterministic confinement or provenance.

## 6.2 Claim record

**PROPOSED:**

~~~yaml
trust_claim:
  claim_type: SANDBOX_PROFILE_ENFORCED
  subject:
    package_id: org.example.adapter
    package_version: 1.4.2
    artifact_digest: sha256:...
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
  status: CURRENT
~~~

Useful status vocabulary:

~~~text
ASSERTED
PASS
FAIL
UNKNOWN
STALE
SUPERSEDED
REVOKED
NOT_APPLICABLE
~~~

A UI can then group evidence without inventing a total order such as “92% trusted.”

---

# 7. Supply-chain composition: TUF + Sigstore + SLSA + reproducibility

## 7.1 TUF-shaped repository/update layer

**OBSERVED:** TUF defines separate Root, Targets, Snapshot, and Timestamp roles. Root controls role keys/thresholds; Targets binds target hashes/sizes and delegation; Snapshot gives a consistent repository view; Timestamp is short-lived and helps clients detect stale/frozen metadata. TUF explicitly designs for key compromise, rollback, and repository attacks.

Sources:
- https://theupdateframework.io/docs/metadata/
- https://theupdateframework.io/docs/security/
- https://theupdateframework.io/docs/faq/

**PROPOSED use:**

~~~text
root trust
  -> directory/update role keys
  -> publisher namespace delegations
  -> exact target package digest/version
  -> snapshot consistency
  -> timestamp freshness
~~~

This is the right layer for “is this still a currently authorized package/update?” rather than relying on a timeless package signature.

## 7.2 Sigstore-shaped publisher/build identity

**OBSERVED:** Sigstore keyless signing uses OIDC identity, short-lived certificates, and transparency-log inclusion. Verification can bind the artifact signature to an expected identity and trust root.

Sources:
- https://docs.sigstore.dev/cosign/signing/overview/
- https://docs.sigstore.dev/cosign/verifying/verify/
- https://docs.sigstore.dev/logging/overview/

**PROPOSED:** a package directory may accept Sigstore bundles as evidence for a specific expected maintainer/workflow identity, but “signed by some valid OIDC identity” is not enough. Directory policy must bind the expected identity for that package namespace.

## 7.3 SLSA-shaped build provenance

**OBSERVED:** SLSA 1.2 describes provenance that tracks artifacts back through build processes and source, with Build and Source tracks.

Source:
- https://slsa.dev/spec/v1.2/

**PROPOSED:** package metadata should link to provenance without claiming provenance proves source correctness. It proves/attests how an artifact was produced under the stated builder model.

## 7.4 Reproducibility

**OBSERVED:** Reproducible Builds defines a reproducible build as one where parties can recreate bit-identical specified artifacts from the same source, environment, and instructions.

Source:
- https://reproducible-builds.org/docs/definition/

**DERIVED:** reproducibility is stronger when an independent rebuilder actually reproduces the released artifact. “The build is designed to be reproducible” and “two independent rebuilders matched this digest” are different claims.

## 7.5 Composition rule

~~~text
TUF currentness
!= publisher identity
!= build provenance
!= reproducibility
!= malware scan
!= sandbox confinement
!= protocol conformance
!= compatibility
!= legality
~~~

The strongest ecosystem should make these composable, not collapse them.

---

# 8. L3.4 — Package update, yank, revoke, and recovery

## 8.1 Distinguish lifecycle states

**PROPOSED:**

~~~text
ACTIVE
DEPRECATED
YANKED_COMPATIBILITY
SECURITY_REVOKED
PUBLISHER_REVOKED
LEGAL_OR_POLICY_BLOCKED
COMPATIBILITY_INVALIDATED
SUPERSEDED
~~~

PyPI's yanking model is useful evidence that “discouraged from new selection” and “deleted” are different: yanked releases can remain available and may still satisfy exact pins.

Source:
- https://docs.pypi.org/project-management/yanking/
- https://peps.python.org/pep-0592/

For an executable adapter ecosystem, a security revocation must be stronger than ordinary yanking.

## 8.2 Security-revoked package behavior

**PROPOSED:**

- stop automatic launch;
- stop automatic reinstall;
- retain exact artifact/evidence metadata for incident reconstruction;
- allow uninstall/rollback;
- show the reason and affected versions;
- require a new non-revoked version or an explicitly separate developer/unsafe override path;
- never silently “unrevoke” because a publisher republishes identical bytes under a new version.

## 8.3 Compromised publisher

Recovery should be directory/update-root controlled, not publisher-self-certified.

~~~text
publisher key/identity compromised
-> directory marks affected identity generation revoked
-> update metadata advances
-> affected target versions become revoked/quarantined
-> replacement identity must be authorized by current directory root
-> new package version/provenance required
~~~

Sigstore reduces long-lived package-key management but does not remove identity-account compromise. TUF-style root/delegation control remains useful for current authorization.

## 8.4 Lost signing key

Lost key is not evidence of compromise.

Model separately:

~~~text
LOST
SUSPECTED_COMPROMISE
CONFIRMED_COMPROMISE
ROTATED_ROUTINE
REVOKED_POLICY
~~~

Recovery must preserve old package history while changing who may sign future current targets.

## 8.5 Offline behavior

This is an explicit availability/safety choice.

**PROPOSED:** every trust profile carries a freshness policy.

~~~yaml
update_freshness:
  last_verified_snapshot: ...
  last_verified_timestamp: ...
  expires_at: ...
  offline_disposition:
    low_risk_reimplementation: ALLOW_WITH_STALE_WARNING
    host_process_code: BLOCK_AFTER_EXPIRY
~~~

Exact values require product policy and L4 testing; the important point is to avoid pretending cached trust remains current forever.

---

# 9. L3.3 — Privacy and data-flow model

## 9.1 Local-first baseline

Keep local by default:

- installed-game inventory;
- absolute game/library paths;
- game asset contents;
- save files;
- screenshots/video;
- raw controller/keyboard/mouse events;
- usernames embedded in paths/logs;
- crash dumps containing process memory;
- private launcher tokens;
- package build caches;
- cloud-model prompts containing local code/assets unless explicitly selected.

Transmit only what a shared session needs:

- protocol/session version;
- neutral intent;
- neutral authoritative state/events;
- capability/profile metadata;
- minimum connection/routing metadata.

## 9.2 Data-flow matrix

| Data class | Default location | Network exposure | Retention default | Notes |
|---|---|---|---|---|
| installed-game inventory | local | none | local current state | should not become analytics by default |
| absolute install paths | local | none | ephemeral/local | redact from logs |
| owned game assets | local | none | local | never use package registry as redistribution channel |
| game version | local + optional compatibility report | exact version only | bounded evidence | useful for compatibility registry |
| session pseudonym | local/server | session participants/server | session/bounded | do not reuse as global identity without need |
| peer IP/port | network topology | peer or relay/server depending mode | minimal | audience must be explicit |
| neutral intents/state | session | session/server | session/replay policy | protocol data |
| optional telemetry | opt-in | telemetry service | purpose-bounded | schema-visible and inspectable |
| crash/error report | opt-in or clearly disclosed | support service | bounded | aggressive redaction before upload |
| raw crash dump | local by default | none | short/local | can contain secrets and proprietary data |
| adapter-generation prompt/context | local by default | cloud only by explicit model choice | provider-dependent | separate L2 mechanics; L3 owns disclosure boundary |

## 9.3 IP exposure

**OBSERVED:** Melty currently tells users that multiplayer hosting may expose the host's IP address and port to players in that game.

Source:
- https://melty.gg/privacy
- https://melty.gg/terms

**OBSERVED:** EU case law has treated a dynamic IP address as personal data for an operator when that operator has legal means to identify the person using additional data.

Source:
- https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=celex:62014CJ0582

**PROPOSED:** multiplayer mode presentation should say exactly which parties learn the network address.

~~~text
DIRECT_PEER      -> peers can learn host address
RELAYED          -> relay learns endpoints; peers need not
PRIVATE_SERVER   -> server operator learns connecting endpoints
PUBLIC_LOBBY     -> discovery metadata becomes public/semi-public; endpoint policy separate
~~~

“Uses relay” is not “anonymous.”

## 9.4 Privacy by design

**OBSERVED:** EDPB guidance treats data minimisation, purpose limitation, storage limitation, integrity/confidentiality, and privacy by design/default as core GDPR principles.

Sources:
- https://www.edpb.europa.eu/topics/key-gdpr-concepts/basic-principles_en
- https://www.edpb.europa.eu/documents/guideline/guidelines-42019-on-article-25-data-protection-by-design-and-by-default_en

**PROPOSED design consequence:** compatibility evidence should prefer minimized facts such as “game version X / adapter Y / suite Z passed on platform P” rather than uploading paths, inventory, account IDs, or raw logs.

---

# 10. L3.5 — Jurisdiction-aware legal/integration policy

This is research architecture, not legal advice.

## 10.1 Do not model “legal = true”

A package or integration may have different answers for:

~~~text
copyright_interoperability
decompilation_or_reverse_engineering
anti_circumvention
contract_or_EULA
anti_cheat_or_online_service_rules
redistribution_of_game_assets
trademark_and_passing_off
privacy_and_telemetry
export_or_sanctions
publisher_or_store_policy
jurisdiction
~~~

One favorable answer does not erase another blocker.

## 10.2 EU baseline

**OBSERVED:** Directive 2009/24/EC Article 5(3) allows a person entitled to use a program to observe, study, or test its functioning during lawful acts to determine underlying ideas/principles. Article 6 creates a constrained decompilation route when indispensable to obtain information necessary for interoperability of an independently created program and the information is not readily available, with use/disclosure limits.

Primary source:
- https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32009L0024

**OBSERVED:** the directive also states that contractual provisions contrary to the protected backup/observation/decompilation exceptions identified there are null and void.

**OBSERVED:** CJEU SAS Institute v World Programming (C-406/10) is a major interoperability boundary: the judgment addresses reproduction of functionality by a second program without source-code access and the protection boundary around functionality, programming languages, and data-file formats.

Primary source:
- https://eur-lex.europa.eu/legal-content/EN/ALL/?uri=celex:62010CJ0406

**DERIVED:** EU interoperability law is materially friendlier to clean independent compatibility work than a simplistic “reverse engineering is illegal” statement, but it does not create a blanket right to bypass anti-cheat, redistribute copyrighted assets, violate unrelated service rules, or ignore privacy/trademark law.

## 10.3 Native EU implementation examples

### Netherlands

**OBSERVED:** Dutch Auteurswet Article 45l permits an authorized user, during authorized acts, to observe, study, and test a program to determine underlying ideas/principles. Article 45m implements a constrained interoperability decompilation exception.

Primary source:
- https://wetten.overheid.nl/BWBR0001886/

Native discovery tokens:
- interoperabiliteit
- waarnemen, bestuderen en testen
- vertalen van de codevorm
- rechtmatige gebruiker / rechtmatig verkregen exemplaar

### Germany

**OBSERVED:** UrhG §69e permits code reproduction/translation when indispensable to obtain information necessary for interoperability of an independently created program, subject to lawful-user, availability, necessity, purpose, disclosure, and infringement limits.

Primary source:
- https://www.gesetze-im-internet.de/urhg/__69e.html

Native discovery tokens:
- Dekompilierung
- Interoperabilität
- unerläßlich
- zur Verwendung eines Vervielfältigungsstücks berechtigte Person

### France

**OBSERVED:** Code de la propriété intellectuelle L122-6-1 III permits an authorized user to observe/study/test software functioning to determine underlying ideas/principles during authorized acts; IV contains the interoperability decompilation conditions.

Primary source:
- https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000044365559/

Native discovery tokens:
- observer, étudier ou tester
- interopérabilité
- reproduction du code
- traduction de la forme du code

## 10.4 United States baseline

**OBSERVED:** 17 U.S.C. §1201(f) contains a specific reverse-engineering/interoperability exception allowing a person who lawfully obtained the right to use a copy of a program to circumvent access controls for the limited purpose of identifying/analyzing elements necessary to achieve interoperability of an independently created program, subject to statutory conditions.

Primary source:
- https://www.govinfo.gov/content/pkg/USCODE-2021-title17/html/USCODE-2021-title17-chap12-sec1201.htm

**DERIVED:** this is not a universal modding safe harbor. The statute itself limits the exception and does not make copyright infringement or other applicable law disappear.

## 10.5 Registry legal record

**PROPOSED:**

~~~yaml
integration_policy_record:
  game_or_product: ...
  adapter_id: ...
  integration_mode: LOCAL_REIMPLEMENTATION
  jurisdiction_profile:
    - EU
    - NL
  lawful_copy_required: true

  technical_actions:
    observe_test: true
    parse_file_formats: true
    decompile_code: false
    circumvent_access_control: false
    modify_game_process: false
    connect_online_service: false
    redistribute_game_assets: false

  policy_sources:
    copyright: [...]
    eula_or_terms: [...]
    anti_cheat: [...]
    store_or_platform: [...]
    trademark: [...]

  unresolved:
    - ...

  official_directory_disposition:
    ALLOW
    # or REVIEW_REQUIRED / EXCLUDE

  evidence_cutoff: ...
~~~

The record should describe why the official directory allows/excludes a package. It should never claim universal legality.

## 10.6 Official ecosystem conservative rule

**PROPOSED:**

- OPEN_ENGINE: normally admissible subject to licenses/trademarks/package policy.
- OFFICIAL_MOD: admissible when the mod path is actually supported and package policy passes.
- OFFICIAL_API: admissible under exact API/service terms.
- CONTROLLED_SERVER_GATEWAY: admissible for user-controlled servers/interfaces; public-service interaction must be separately assessed.
- LOCAL_REIMPLEMENTATION: admissible where owned-file use and applicable interoperability/legal conditions are satisfied.
- UNSUPPORTED_INJECTION: excluded from the official directory even if technically possible.

This is a project distribution policy, not a claim that excluded techniques are universally unlawful.

---

# 11. L3.6 — Governance split

## 11.1 Current Signet governance reality

**OBSERVED:** Signet is presently single-maintainer governance with a public proposal process and a stated intention to broaden maintainership as the community grows.

That is honest and workable for a young protocol. The mistake would be to pretend that a future package directory, security revocation system, conformance registry, and trademark program are already independently governed.

## 11.2 Model independent governance roots now

**PROPOSED roles:**

### Protocol semantics authority

Owns:
- wire/core/profile meaning;
- versioning rules;
- protocol proposals.

Does not own:
- package listing;
- publisher identity;
- security quarantine;
- trademark enforcement.

### Conformance authority

Owns:
- suite versions;
- fixtures;
- normative expected results;
- certification evidence format.

Does not own:
- package installation policy;
- protocol proposal acceptance by itself.

### Package directory authority

Owns:
- package namespace/listing;
- metadata requirements;
- publisher authorization;
- directory moderation;
- package yank/quarantine/revocation status.

Does not own:
- protocol meaning.

### Update trust-root authority

Owns:
- TUF-like root/delegations;
- repository signing-role rotation;
- compromise recovery.

This may initially be operated by the same humans as the directory, but it is a distinct high-risk authority.

### Security incident authority

Owns:
- confidential intake;
- temporary quarantine;
- coordinated vulnerability/revocation handling;
- incident advisories.

A temporary security quarantine must not silently rewrite protocol semantics or erase history.

### Trademark/official-brand authority

Owns:
- official Signet naming/branding;
- claims of official endorsement.

“Compatible with Signet/1” should remain tied to conformance, not to directory politics.

## 11.3 Phase-gated governance

Avoid both founder absolutism forever and premature foundation bureaucracy.

**PROPOSED:**

### G0 — young project

- one/few maintainers may fill several roles;
- role boundaries are documented even if personnel overlap;
- protocol proposals and directory actions are public/reasoned;
- security matters may be confidential until disclosure;
- package history/revocations are auditable.

### G1 — real independent ecosystem

Trigger candidates:
- at least 3 actively maintained third-party translators;
- contributors from at least 2 unrelated organizations/communities;
- recurring protocol proposals from outside the founding project.

Then:
- add independent maintainers;
- require at least two independent maintainers for high-impact update-root or security-unquarantine changes;
- add directory appeal/reason codes;
- separate protocol proposal decision from package moderation.

### G2 — neutral stewardship

Consider a foundation/neutral organization only when:
- independent implementations depend on stable shared governance;
- trademark/assets/servers require institutional ownership;
- contributor diversity makes one-person ownership a material ecosystem risk.

The trigger should be ecosystem dependency, not prestige.

## 11.4 Capture-resistant invariants

**PROPOSED:**

- directory removal cannot redefine protocol semantics;
- protocol compatibility cannot force directory listing;
- package publisher identity does not grant protocol voting authority;
- maintainer status does not self-certify package safety;
- conformance results bind exact suite/package/game versions;
- security quarantine has reason, scope, timestamp, owner, and recovery condition;
- high-impact root rotation is separately recorded;
- compatibility evidence remains reproducible by outsiders;
- historical yanks/revocations are retained rather than rewritten;
- governance changes themselves are versioned and publicly explainable.

---

# 12. Adversarial fixture list for L4

These are handed to L4 for executable validation.

## Package / update

1. valid publisher signature, malicious artifact -> signature claim passes; safety does not.
2. package v2 signed by compromised old key after key rotation -> current update policy rejects.
3. mirror serves v1 after v2/current metadata -> rollback rejected.
4. mirror freezes old timestamp metadata -> freshness failure.
5. dependency lock names package but digest changes -> install rejected.
6. package digest matches but SLSA provenance references wrong source revision -> provenance failure.
7. provenance passes but independent rebuild differs -> reproducibility failure.
8. security-revoked installed package remains on disk -> launcher refuses automatic execution but permits forensic/export/uninstall path.
9. yanked compatibility release pinned exactly -> behavior follows explicit policy, not confused with security revoke.
10. lost publisher key -> recovery authorizes successor identity without rewriting old artifact provenance.

## Permission / sandbox

11. adapter requests only selected game read; tries home-directory read -> denied.
12. tries SSH/private-key read -> denied.
13. tries browser-cookie/keychain access -> denied.
14. tries undeclared arbitrary outbound Internet -> denied.
15. tries undeclared child process -> denied.
16. tries path traversal from selected game tree -> cannot escape resolved resource boundary.
17. symlink inside selected tree points outside -> policy handles without granting ambient escape.
18. package writes staged mod output then targets unrelated game file -> broker rejects application.
19. package declares localhost only, tries public IP -> denied.
20. operator/launcher UI says sandboxed but runtime cannot enforce network on platform -> result must be PARTIALLY_ENFORCED, not PASS.

## Privacy

21. crash report contains username/home path/token-like string -> local redaction removes it before upload.
22. installed-game inventory populated -> no telemetry event contains full inventory by default.
23. direct peer mode -> peer sees expected endpoint and UI declared it.
24. relay mode -> peer does not receive endpoint but relay-side collection remains documented.
25. compatibility report -> exact game/adapter/suite version transmitted without absolute path/account ID.

## Governance / policy

26. package-directory moderator removes package -> protocol conformance meaning unchanged.
27. protocol maintainer changes spec -> existing package listing does not automatically become compatible.
28. translator author self-asserts all trust labels -> only supported evidence claims render as PASS.
29. security responder quarantines one version -> unrelated publisher namespace unaffected.
30. one maintainer controls all roles in G0 -> UI/docs disclose governance reality rather than calling system decentralized.

---

# 13. Cross-lane handoffs

~~~yaml
handoff:
  from_lane: L3
  to_lane: L1
  finding: Public-network Signet needs authenticated server/session identity, replay protection, and downgrade semantics, but L3 only owns the trust boundary and distribution consequences.
  evidence:
    - Signet SECURITY.md currently says beta reference server has no authentication.
  why_material: Package trust cannot compensate for unauthenticated public session semantics.
  requested_followup: Define the smallest authenticated public-server/session profile and protocol evidence L3 can reference.
~~~

~~~yaml
handoff:
  from_lane: L3
  to_lane: L2
  finding: Portable permission enforcement is materially easier with a privileged launcher broker and sandboxed adapter worker using typed resource handles instead of raw host paths.
  evidence:
    - Windows AppContainer/LPAC
    - Flatpak/portals and Landlock
    - macOS App Sandbox entitlements/user-selected resources
  why_material: Adapter SDK/launcher mechanics must expose brokered file/process/network operations rather than assuming ambient process access.
  requested_followup: Prototype the smallest adapter-worker/broker RPC and identify which current translators require host-process execution.
~~~

~~~yaml
handoff:
  from_lane: L3
  to_lane: L4
  finding: Trust must be validated as independent claims rather than one verified badge.
  evidence:
    - TUF role/currentness model
    - Sigstore identity/transparency
    - SLSA provenance
    - reproducible-build definition
    - OS sandbox enforcement evidence
  why_material: A compatibility registry should store exact evidence for each claim and reproduce it independently.
  requested_followup: Execute the 30 L3 fixtures above, beginning with sandbox escape, update rollback/freeze, and independent reproducibility.
~~~

---

# 14. New claims from this continuation pass

1. **DERIVED:** the portable permission schema should use symbolic resources resolved by a privileged broker, not package-declared raw host paths.
2. **DERIVED:** integration mode is part of the runtime trust claim; official mods and open-engine forks cannot honestly inherit the same “sandboxed translator” claim as a separate reimplementation worker.
3. **DERIVED:** TUF, Sigstore, SLSA provenance, reproducible builds, scanning, sandboxing, conformance, and legal review are complementary and must remain separately visible.
4. **PROPOSED:** package lifecycle needs hard security revocation distinct from compatibility yanking/deprecation.
5. **DERIVED:** direct-vs-relay networking is a privacy audience decision, not merely a connectivity option.
6. **DERIVED:** EU interoperability provisions are strong enough that “closed-source game” must not automatically mean “officially impossible,” but exact integration actions, jurisdiction, contract/service rules, redistribution, and anti-circumvention remain separate.
7. **PROPOSED:** model independent governance authorities now even if the same founder initially operates several of them; split personnel as ecosystem dependence grows.

# 15. Contradictions / corrections to the baseline

- The earlier sample permission manifest was too close to a static declaration. **Correction:** declaration and enforcement evidence are separate, and host paths should be broker-resolved.
- “Signed release manifest” was underspecified. **Correction:** an update system also needs current authorization, freshness, rollback/freeze handling, and compromise recovery.
- “Verified” is too ambiguous even when typed labels are listed elsewhere. **Correction:** every rendered trust claim should have a claim type, exact subject, evidence, verifier, scope, time/currentness, and status.
- “Sandbox the translator” is not uniformly realizable across integration modes. **Correction:** expose runtime confinement class explicitly.
- “Legal status varies by jurisdiction” remains true but too coarse. **Correction:** represent distinct legal/policy dimensions and integration actions; do not store one legal boolean.

# 16. No-delta evidence

- Signet's existing rejection of online-game injection/anti-cheat bypass remains a strong official-ecosystem boundary; this pass did not find evidence requiring it to be weakened.
- The existing local-first rule for owned game files remains correct.
- The existing recommendation to separate protocol governance from package-directory governance remains correct; this pass makes the authority split concrete rather than reversing it.
- The existing warning that signatures/scans/conformance are not equivalent remains correct; current standards research strengthens that conclusion.

# 17. Next recursion

## Security / distribution

- TUF delegations for per-publisher namespaces and exact root-rotation ceremony;
- Sigstore bundle verification policy for GitHub Actions vs human release identities;
- SLSA Source Track applicability to community translators;
- independent rebuilder designs for Windows/macOS native artifacts;
- SBOM/VEX relationship to the trust ledger without adding another misleading badge;
- dependency-confusion and package-name takeover policy.

## Sandbox

- exact Windows CreateProcessInSandbox supported-build matrix and limitations;
- localhost/AppContainer networking implications for a Signet session;
- Linux Landlock ABI fallback rules and UDP/session needs;
- macOS helper/XPC architecture for downloaded translator logic;
- game-launch broker protocol and environment-variable minimization;
- symlink/reparse-point/path canonicalization fixtures across OSes.

## Legal / policy

- actual EULA/anti-cheat/store terms for candidate first integrations;
- EU anti-circumvention interaction with Software Directive interoperability exceptions;
- U.S. Sega/Connectix/BnetD/MDY lines as case-specific evidence, not slogans;
- additional native member-state implementations where a candidate game/community makes them material;
- trademark/nominative-use rules for directory naming;
- package-directory notice/takedown obligations once user uploads become material.

## Governance

- directory appeal protocol;
- security quarantine and unquarantine authority;
- independent maintainer trigger metrics;
- transparent but privacy-safe security advisory history;
- protocol proposal and conformance-suite version coupling.

---

# 18. L3 source additions

## Project/product

- Signet governance: https://github.com/kian-cx/signetprotocol/blob/main/GOVERNANCE.md
- Signet security: https://github.com/kian-cx/signetprotocol/blob/main/SECURITY.md
- Signet integration modes: https://github.com/kian-cx/signetprotocol/blob/main/docs/content/concepts/integration.mdx
- Melty Terms: https://melty.gg/terms
- Melty Privacy: https://melty.gg/privacy

## Supply-chain security

- TUF metadata roles: https://theupdateframework.io/docs/metadata/
- TUF security model: https://theupdateframework.io/docs/security/
- TUF FAQ / compromise recovery: https://theupdateframework.io/docs/faq/
- Sigstore signing overview: https://docs.sigstore.dev/cosign/signing/overview/
- Sigstore verification: https://docs.sigstore.dev/cosign/verifying/verify/
- Rekor transparency log: https://docs.sigstore.dev/logging/overview/
- SLSA 1.2: https://slsa.dev/spec/v1.2/
- Reproducible Builds definition: https://reproducible-builds.org/docs/definition/

## Sandbox / permissions

- Windows AppContainer launch/isolation:
  - https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer
  - https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation
- Windows Create Process In Sandbox:
  - https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox
- Flatpak sandbox:
  - https://docs.flatpak.org/en/latest/basic-concepts.html
  - https://docs.flatpak.org/en/latest/sandbox-permissions.html
- Linux Landlock:
  - https://www.kernel.org/doc/html/latest/userspace-api/landlock.html
- macOS App Sandbox:
  - https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox
  - https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.app-sandbox
  - https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.network.client

## Package-ecosystem analogues

- Flathub verification:
  - https://docs.flathub.org/docs/for-users/verification
  - https://docs.flathub.org/docs/for-app-authors/verification
- Flathub permissions:
  - https://docs.flathub.org/docs/for-users/permissions
- PyPI yanking:
  - https://docs.pypi.org/project-management/yanking/
  - https://peps.python.org/pep-0592/

## Privacy

- EDPB basic principles:
  - https://www.edpb.europa.eu/topics/key-gdpr-concepts/basic-principles_en
- EDPB Article 25 guidance:
  - https://www.edpb.europa.eu/documents/guideline/guidelines-42019-on-article-25-data-protection-by-design-and-by-default_en
- CJEU Breyer, C-582/14:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=celex:62014CJ0582

## Legal / interoperability

- EU Software Directive 2009/24/EC:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32009L0024
- CJEU SAS Institute v World Programming, C-406/10:
  - https://eur-lex.europa.eu/legal-content/EN/ALL/?uri=celex:62010CJ0406
- Netherlands Auteurswet 45l/45m:
  - https://wetten.overheid.nl/BWBR0001886/
- Germany UrhG §69e:
  - https://www.gesetze-im-internet.de/urhg/__69e.html
- France CPI L122-6-1:
  - https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000044365559/
- U.S. 17 U.S.C. §1201(f):
  - https://www.govinfo.gov/content/pkg/USCODE-2021-title17/html/USCODE-2021-title17-chap12-sec1201.htm

---

## Success condition

L3 is useful when a user can understand:

1. exactly what an adapter is allowed to do;
2. which parts are enforced by the current OS/runtime rather than merely declared;
3. who published the exact artifact and whether it is still currently authorized;
4. how that artifact was built and whether anyone independently reproduced it;
5. which compatibility/conformance claims were actually reproduced;
6. what data leaves the machine and which parties receive it;
7. which game/integration policy applies;
8. which legal questions were scoped and which remain unresolved; and
9. which governance authority can change, revoke, or appeal each of those states.

The target is not “trusted software.” The target is **specific, auditable claims with bounded authority and explicit unknowns**.
