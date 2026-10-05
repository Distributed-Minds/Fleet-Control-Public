# L3 Merge Notes for the Public Signet 2 Response Paper

**Status:** synthesis handoff  
**Date:** 2026-10-05  
**L3 source paper:** 02-Signet-2-Trust-Response-Paper.md  
**Observed public response draft:** Publications/Game-Interoperability/RESPONSE-TO-SIGNET-2-ARCHITECTURE-DRAFT.md on research/signet-l2-adapters-ai, blob e1d493187779b6fd41d6c4620dc399437fb93da7  
**Observed L4 response:** Research/Game-Interoperability/Signet-Frontier-2026-10/Signet-2-Validation-Response.md in draft PR #53

---

## Purpose

The L2 branch already contains the strongest candidate for the eventual public paper:

**From Shared Meaning to Verifiable Translation**

The L4 branch separately contains a validation/evidence response, and L1 contains a semantic-contract response analysis.

L3 should therefore **not create a competing public canonical paper**.

The L3 response paper in this branch is a source contribution for the eventual merged paper.

This file records the exact L3 material that should survive synthesis.

---

## 1. Preserve the current public paper's core thesis

The L2 public draft should remain the main engineering spine:

- game -> shared meaning -> game is the correct scaling direction;
- closed candidates are better than free generation;
- calibration should solve deterministic/measurable work before AI;
- profile authority classes should be explicit;
- pinned mappings need evidence and invalidation conditions;
- generated contract glue + thin target-specific hooks is a strong adapter-engineering target;
- NO_MATCH / abstention is necessary;
- the 30-case resolver experiment is a pilot, not final model validation;
- replay and drift detection should make compatibility claims stateful.

L3 agrees with this.

---

## 2. Add one trust paragraph to the abstract

After the public draft explains that the translation boundary must become verifiable, add the ecosystem trust consequence:

> A verifiable translation boundary is still insufficient if strangers cannot safely install or update the translator that implements it. Translation profiles are authority-bearing configuration, community profile pull requests are behavioral supply-chain changes, and Forge can influence shipped runtime behavior even when no model runs during a match. The public ecosystem therefore also needs explicit package identity, permission enforcement, update freshness and revocation, participant-owned profile overlays, local-first data handling, and separated governance authorities.

Then add the compact invariant:

~~~text
VERIFIABLE_MAPPING
!= SAFE_PACKAGE
!= CURRENT_AUTHORIZATION
!= LEGAL_OR_POLICY_PERMISSION
~~~

This extends rather than replaces the L2 thesis.

---

## 3. Strengthen the existing profile-authority section

The L2 paper already distinguishes:

- input binding;
- motion observation;
- semantic mapping;
- appearance mapping;
- session capability policy.

Add the L3 ownership split:

~~~text
publisher_profile
    immutable/versioned translator baseline

participant_overlay
    local user-owned pins/overrides

session_resolution
    derived effective profile
~~~

Important rule:

> A package/profile update must not silently rewrite a participant-owned pin and continue to label the result as a player decision.

The effective profile evidence should bind:

- translator artifact digest;
- publisher profile digest;
- participant overlay digest;
- game/version fingerprint;
- protocol version;
- vocabulary/catalog revisions;
- calibration revision;
- unresolved/default dispositions.

This is the L3 addition to L2's evidence-bearing lock design.

---

## 4. Resolve the draft proposal's suggest-mode ambiguity explicitly

The Signet 2 proposal allows a provisional resolver result in suggest mode but separately says simulation-affecting intents must always be pinned.

The merged paper should state:

- provisional resolver output is acceptable for local presentation when the consequence is local/cosmetic;
- a simulation-affecting miss may be suggested to a reviewer, but it must not emit an authoritative intent before pinning;
- safe simulation outcomes include NO_AUTHORITATIVE_ACTION, IGNORE, OBSERVE_ONLY, or REQUIRE_PIN.

This aligns naturally with L2's NO_MATCH / abstention requirement.

---

## 5. Correct the “server authority means nobody can cheat” wording

The Signet 2 paper says the server is the only authority, “so nobody can cheat by modifying their client.”

The merged response should preserve the architectural point but narrow the security claim:

> Server authority prevents a modified client from becoming authoritative merely by reporting different world state. It does not eliminate cheating through automated inputs, timing abuse, information exposure, malformed intent streams, collusion, or implementation vulnerabilities.

Recommended compact invariant:

~~~text
SERVER_AUTHORITY
!= CHEAT_IMPOSSIBILITY
~~~

L4 independently reaches the same correction, so this is a strong synthesis point.

---

## 6. Add a package/distribution section after adapter architecture

Suggested heading:

# Public translators are supply-chain software

Core argument:

A public translator/launcher may:

- read game installations;
- write staged modifications;
- execute helper processes;
- launch games/servers;
- open network connections;
- update itself and dependencies.

Therefore a semantic/profile PASS does not answer whether the artifact is safe to install.

Keep these claims separate:

~~~text
DECLARED_PERMISSION   != ENFORCED_PERMISSION
VALID_SIGNATURE       != CURRENT_AUTHORIZATION
CURRENT_AUTHORIZATION != SAFE_ARTIFACT
SAFE_ARTIFACT         != PROTOCOL_CONFORMANCE
CONFORMANCE           != GAME_COMPATIBILITY
~~~

Recommended package subject:

- namespace/name/version;
- artifact digest;
- platform/architecture;
- publisher identity;
- integration mode;
- dependency lock/digests;
- publisher profile digest;
- requested permissions;
- update channel metadata.

Recommended trust layers:

- TUF-like update freshness/rollback/revocation;
- Sigstore-style signer/workload identity and transparency;
- SLSA provenance;
- independent reproducibility;
- sandbox/enforcement evidence;
- conformance and compatibility evidence as separate claims.

Do not turn these into one “verified” badge.

---

## 7. Add a short brokered-permission subsection

Suggested compact design:

~~~text
downloaded translator worker
        |
        | typed broker RPC
        v
privileged launcher broker
        |
        +-- selected game resource
        +-- translator work directory
        +-- current Signet session network
        +-- approved game/server process
~~~

Packages request symbolic resources rather than arbitrary host paths.

Example vocabulary:

~~~text
GAME_INSTALL_READ
GAME_INSTALL_WRITE_STAGED
TRANSLATOR_WORKDIR_RW
SIGNET_SESSION_NETWORK
CONTROLLED_SERVER_PROCESS
USER_SELECTED_FILE_READ
~~~

The launcher reports what the platform actually enforced.

Useful confinement classes:

~~~text
SEPARATE_SANDBOX_ENFORCED
HOST_API_CONFINED
HOST_PROCESS_CODE
FULL_APPLICATION
UNCONFINED
UNKNOWN
~~~

This avoids calling an in-process official mod “sandboxed” merely because its manifest is narrow.

---

## 8. Expand the Forge/training-data section with an explicit privacy boundary

The L2 draft already distinguishes label scope and candidate context.

Add the L3 rule:

~~~text
local mapping approval
!= training-data contribution
~~~

The Signet 2 paper's dataset rules — names/identifiers only, no game assets, no usernames, no input logs, open license — are a good **export minimization** baseline.

They do not by themselves establish end-to-end privacy.

The merged paper should distinguish:

- local calibration data;
- local profile state;
- remote resolver export;
- telemetry/crash reports;
- explicit training-data contribution.

If a shared Forge/resolver service exists, require a visible export schema. It should not inherit access to:

- local paths;
- full installed-game inventory;
- raw input streams;
- game assets;
- saves;
- usernames/account identifiers.

Every contributed example should bind provenance, license, source/profile revision, and invalidation lineage.

---

## 9. Add a profile-conflict answer that combines L2 and L3

The L2 answer correctly says that conflict resolution should distinguish profile class and prefer current reproducible evidence for translator-level mappings.

Add:

1. first identify the exact profile subject;
2. publisher baseline is immutable per translator release;
3. participant pins live in a separate local overlay;
4. multiple community/publisher alternatives may coexist;
5. profile-only semantic changes receive new digests and stale prior compatibility evidence;
6. popularity may aid discovery but must not silently rewrite local behavior;
7. do not silently synthesize a “consensus profile” unless that synthesis algorithm is itself specified, deterministic, versioned, and evidenced.

---

## 10. Add one governance section

Suggested heading:

# Interoperability needs separated governance authorities

Early Signet governance can remain lightweight, but the paper should distinguish powers even if current personnel overlap:

- protocol/vocabulary/catalog governance;
- translator/profile governance;
- participant/local-pin authority;
- package-directory governance;
- update trust-root authority;
- security incident/quarantine authority;
- Forge/dataset/model governance;
- conformance-suite authority.

This prevents statements such as “maintainer approved” from ambiguously meaning:

- protocol-standardized;
- package safe;
- directory listed;
- security current;
- model-training eligible;
- interoperable.

A community PR that changes a simulation-affecting profile mapping is a behavioral supply-chain event, not merely a text edit.

---

## 11. Add legal/policy scope without turning the paper into legal advice

Keep Signet's official boundary against online-game injection and anti-cheat bypass.

Add one qualification:

> The official ecosystem may deliberately support only open engines, permitted mods/APIs, controlled servers, and other low-risk integration modes. That is a project support/distribution policy. It should not be encoded as a universal claim that all other interoperability work is unlawful.

Represent technical integration mode separately from legal/policy disposition.

Possible modes:

~~~text
OPEN_ENGINE
OFFICIAL_MOD
OFFICIAL_API
CONTROLLED_SERVER_GATEWAY
LOCAL_REIMPLEMENTATION
UNSUPPORTED_INJECTION
~~~

Possible official-directory dispositions:

~~~text
ALLOW
ALLOW_WITH_USER_WARNING
REVIEW_REQUIRED
EXCLUDE_OFFICIAL_DIRECTORY
UNKNOWN
~~~

A directory decision is scoped governance, not a universal legality judgment.

---

## 12. Merge L4's evidence thesis directly

L4's current response paper supplies the empirical half of the combined argument.

Preserve these synthesis invariants:

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

L3 adds:

~~~text
VALID_SIGNATURE
!= CURRENT_AUTHORIZATION

PACKAGE_BYTES
!= EFFECTIVE_PROFILE

MODEL_SUGGESTION
!= AUTHORITY

LOCAL_APPROVAL
!= DATASET_CONTRIBUTION
~~~

Together these form a coherent paper rather than four unrelated lane reports.

---

## 13. Recommended final-paper structure

A merged public response can use this order:

1. Abstract
2. Why Signet 2's shared-meaning architecture is the right direction
3. Semantic contract: shared labels are not yet shared semantics — L1
4. Translation/profile engineering: authority classes, evidence, calibration, abstention, generated glue — L2
5. Trust/distribution: packages, permissions, current authorization, profile ownership — L3
6. Forge and adaptive learning: bounded suggestions, privacy, contribution/provenance — L2 + L3
7. Validation: pinned is not correct, pairwise and independent evidence — L4
8. Governance: protocol/profile/directory/security/conformance/dataset separation — L3 + L1/L4
9. Direct answers to the five Signet 2 open questions — synthesized
10. A joint staged roadmap and experiments
11. Proposed changes to the Signet 2 draft
12. Conclusion
13. References / reproducibility package

Do not present the lane boundaries to the external reader. The final paper should read as one argument.

---

## 14. Direct-answer table for the five upstream open questions

| Signet 2 open question | Synthesis answer |
|---|---|
| Minimum motion profile? | Profile/ruleset dependent. Measure only what is needed for normalization/fidelity/compatibility; do not let local calibration redefine server physics. |
| Game does not report position? | Use legitimate observability surfaces; otherwise mark UNKNOWN/UNSUPPORTED rather than manufacture precision. |
| Who maintains vocabulary/catalog? | Protocol governance, with stable IDs, versioned profiles/namespaces, proposal/review, conformance vectors, and deprecation rules. |
| Where does resolver run? | Development/calibration time, local by default; hosted service optional with explicit export contract; never required in the per-tick path. |
| Conflicting community profiles? | Resolve exact subject first; immutable publisher baseline + participant overlay; allow alternatives; evidence and semantic diff determine compatibility, not silent popularity merge. |

---

## 15. Joint experiment to emphasize

The final paper should converge on one small, falsifiable milestone:

1. freeze a tiny versioned semantic profile;
2. implement two materially different adapters/integration modes;
3. generate repetitive contract/binding glue;
4. calibrate without AI;
5. emit evidence-bearing package/profile records;
6. run replay + pairwise interoperability tests;
7. enforce a minimal brokered permission model;
8. change one game/integration/profile dependency and prove the old evidence becomes stale;
9. only then run the 30-case resolver experiment with NO_MATCH;
10. publish raw compatibility evidence so an outsider can reproduce it.

This single demonstration exercises L1 semantics, L2 adapter engineering, L3 trust/distribution, and L4 validation without pretending to solve every game.

---

## 16. L3 sources to add to the final references

- Signet 2 proposal/PDF
- Signet governance
- Signet security policy
- The Update Framework metadata/security model
- Sigstore signing/verification/transparency
- SLSA v1.2
- Reproducible Builds definition
- Microsoft AppContainer / Create Process In Sandbox
- Linux Landlock
- Apple App Sandbox
- EDPB basic principles
- EU Software Directive 2009/24/EC
- CJEU SAS Institute v World Programming

Use these for the exact claims they support; do not imply that adopting the standards names automatically produces security.

---

## Synthesis stop condition

L3 is ready to merge into the public response when the public paper can answer all of these without one ambiguous “trusted/verified/compatible” word:

1. What meaning was mapped?
2. Who had authority to pin/change it?
3. What exact package/profile behavior was tested?
4. What can the package do on this machine?
5. Who currently authorizes this release?
6. What data leaves the machine and why?
7. Which compatibility evidence is still current?
8. Which governance body can revoke, appeal, or change each state?
9. Which legal/policy claim is actually being made, for what scope?

The public paper should remain supportive of Signet 2's architecture while being precise about what must be added for a public ecosystem.
