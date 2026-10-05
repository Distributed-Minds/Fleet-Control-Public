# 05 — Security, Privacy, Legal, and Supply Chain

## Threat model first

Cross-game adapters are unusually privileged software.

They may need to inspect game installations, read proprietary formats, load mods/plugins, write configuration, start processes, connect to local servers, expose network ports, and translate multiplayer state.

That is enough authority to create real risk even without malicious intent.

## Preserve Signet's current safe boundary

Current Signet principles say:

- neutral data travels over the network;
- game files do not;
- translators read the player's own files;
- no cheating / anti-cheat evasion;
- use open engines, officially allowed mods, controlled gateways, or reimplementations.

Preserve that.

### Beta server boundary

The current reference server has no authentication.

Operational rule:

```text
NO AUTH
=> local/LAN/private test only
```

until an authenticated profile exists.

---

## Melty demonstrates the launcher supply-chain problem

Melty's Terms describe the authority a one-click mashup launcher may need:

- discover installed games;
- download dependencies/tools;
- write files;
- preserve replacements for uninstall;
- build local components;
- copy owned-game files into working areas;
- launch games/programs;
- auto-update.

Convenient, but also a supply-chain execution surface.

### Typed verification labels

Do not collapse trust into one green “verified” badge.

Prefer:

```text
SIGNED
SOURCE_REPRODUCIBLE
STATIC_SCAN_PASS
PROTOCOL_CONFORMANCE_PASS
INSTALL_SMOKE_PASS
PUBLISHER_ID_VERIFIED
LICENSE_METADATA_PRESENT
```

“Signed” is not “safe.”  
“Static scan passed” is not “legal.”  
“Protocol conformant” is not “sandboxed.”

---

## Translator package baseline

### Identity/integrity

- signed release manifest;
- content hash for executables/scripts;
- immutable translator version;
- repository + commit provenance;
- declared build recipe;
- dependency lockfile.

### Capability permissions

Example:

```yaml
permissions:
  read:
    - game_installation
  write:
    - translator_cache
    - generated_mod_directory
  network:
    - localhost
    - signet_session
  process:
    - launch_game
```

### Sandboxing

Prefer:

- separate working directory;
- no broad home-directory access;
- no credential/keychain/browser-cookie access;
- read-only game installation where possible;
- no unrestricted child process by default;
- explicit network scope.

---

## Integration-mode policy

Classify:

```text
OPEN_ENGINE
OFFICIAL_MOD
OFFICIAL_API
CONTROLLED_SERVER_GATEWAY
LOCAL_REIMPLEMENTATION
UNSUPPORTED_INJECTION
```

The last class should be outside the official ecosystem.

Technically possible is not equivalent to safe or distributable.

---

## Legal status is not universal

This research does not claim a universal rule for reverse engineering or modding.

Relevant variables include:

- jurisdiction;
- copyright/interoperability exceptions;
- contract/EULA;
- anti-circumvention law;
- anti-cheat terms;
- service/store terms;
- redistribution;
- trademark representation.

A robust protocol should not require questionable integration techniques to prove itself.

---

## Privacy model

Keep local by default:

- installed-game inventory;
- directory paths;
- asset contents;
- saves;
- screenshots/video;
- raw input;
- logs with usernames/pathnames;
- gameplay-derived embeddings/traces.

Transmit only what the shared simulation needs:

- session identity;
- neutral input intent;
- neutral state/events;
- capability/version metadata;
- minimal connection metadata.

Optional telemetry should be opt-in, purpose-bound, redacted, retention-bounded, schema-documented, and inspectable.

---

## Multiplayer IP exposure

Peer-hosted multiplayer may expose IP/port to peers. Make modes explicit:

- direct peer;
- relay;
- private invite;
- public lobby.

Users should know which mode exposes what.

---

## Generated code is supply-chain code

```text
AI GENERATED
does not imply
SAFE
LICENSED
CONFORMANT
MAINTAINABLE
```

Generated translators should record generator/tool version where known, source schema/version, test suite version, build inputs, maintainer/publisher, and exact conformance result.

---

## Security tests worth standardizing

- malformed-message fuzzing;
- oversized/deep JSON;
- NaN/infinite numeric inputs;
- event spam;
- sequence replay;
- duplicate/out-of-order commands;
- invalid capability claim;
- path traversal/symlink escape;
- dependency substitution;
- game-version drift;
- local-path redaction;
- sandbox escape attempts;
- auth downgrade once auth exists.

---

## Governance security

Protocol capture is also a security issue.

Avoid allowing:

- one vendor to silently redefine a common event;
- a model provider to change behavior without versioning;
- a directory operator to mark compatibility without evidence;
- a translator author to self-certify all properties.

Use open specs, public fixtures, versioned changes, independent implementations, reproducible evidence, and explicit uncertainty.
