# L3 — Trust, Distribution, Privacy, Legal & Governance

## Mission

Determine how an open cross-game ecosystem can let strangers discover, install, execute, and govern adapters without turning convenience into an opaque software-supply-chain risk.

## Core questions

1. What authority does a launcher/translator actually need?
2. How should packages declare filesystem, process, and network permissions?
3. Which trust claims can be independently verified?
4. How should signing, provenance, reproducibility, and scanning be separated?
5. What data should remain local?
6. Which integration modes are suitable for an official ecosystem?
7. How should legal/EULA uncertainty be represented without pretending to universal legal advice?
8. What governance structure prevents protocol or directory capture?

## Primary evidence frontier

### Product systems

- Melty;
- mod managers/loaders;
- package managers;
- plugin marketplaces;
- game-server directories;
- reproducible-build ecosystems.

### Security / supply chain

- Sigstore;
- SLSA;
- TUF;
- package signing;
- sandboxing/capability systems;
- OS application permission models.

### Legal / policy

Research interoperability and reverse-engineering rules using jurisdiction-specific primary or authoritative sources.

At minimum distinguish:

- interoperability exceptions;
- copyright;
- anti-circumvention;
- contract/EULA;
- anti-cheat/service terms;
- redistribution;
- trademarks.

Do not flatten these into one global rule.

### Native-language lanes

Legal/policy research should include native-language material where jurisdictionally relevant, especially EU member-state implementations and non-English modding/reverse-engineering scholarship.

## Research tasks

### L3.1 — Permission model

Design explicit package permissions such as:

```yaml
filesystem:
  read:
    - game_installation
  write:
    - translator_workdir
network:
  - localhost
  - declared_session_hosts
process:
  - launch_game
```

Research enforceability on Windows/Linux/macOS.

### L3.2 — Trust claim taxonomy

Separate:

- publisher identity verified;
- signature valid;
- source available;
- reproducible build;
- static scan passed;
- sandbox profile satisfied;
- install smoke test passed;
- protocol conformance passed;
- game-version compatibility reproduced.

No generic “verified” label should imply all of them.

### L3.3 — Privacy model

Classify data:

- required for shared simulation;
- needed only locally;
- optional telemetry;
- sensitive/local identifiers;
- debugging data.

Research local-first patterns and retention/minimization.

### L3.4 — Package/update safety

Study:

- dependency pinning;
- update signatures;
- rollback;
- uninstall restoration;
- compromised publisher;
- revoked package;
- malicious dependency;
- lost signing key.

### L3.5 — Integration-policy taxonomy

Research official-ecosystem rules for:

```text
OPEN_ENGINE
OFFICIAL_MOD
OFFICIAL_API
CONTROLLED_SERVER_GATEWAY
LOCAL_REIMPLEMENTATION
UNSUPPORTED_INJECTION
```

Keep technical possibility separate from distribution policy.

### L3.6 — Governance split

Separate governance of:

- protocol semantics;
- conformance suite;
- package directory;
- compatibility registry;
- trademark/branding;
- security incident response.

Research foundations only when contributor diversity justifies them.

## Deliverables

- launcher/package threat model;
- permission schema;
- trust-label taxonomy;
- update/revocation design;
- privacy/data-flow matrix;
- jurisdiction-aware legal research map;
- protocol-vs-directory governance proposal;
- adversarial supply-chain fixture list for L4.

## Do not duplicate

Hand off to:

- **L1** for protocol semantics/authenticated wire details beyond trust boundaries.
- **L2** for adapter implementation mechanics.
- **L4** for reproducibility experiments, sandbox tests, registry metrics, and ecosystem adoption evidence.

## Success condition

L3 is useful when a user can understand exactly what an adapter is allowed to do, what evidence supports each trust claim, and what remains uncertain.
