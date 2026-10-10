# Phase0 Fleet Configuration

This file is intentionally simple Markdown so both humans and agents can inspect it.

Edit these values when installing the fleet.

```text
PHASE0_CONFIG_VERSION=1
TARGET_REPOSITORY=SELF
FLEET_SIZE=5
AGENTS=A1,A2,A3,A4,A5
COORDINATION_ISSUE_TITLE=[fleet-control] coordination
COORDINATION_BODY_MARKER=FLEET_COORDINATION_V1
COORDINATION_TRUSTED_AUTHORS=geromet
COORDINATION_SWITCH_AT=2000
COORDINATION_STANDBY_MAX=500
COORDINATION_COMPACT_ABOVE=1500
MISSION_TITLE_PREFIX=[fleet-mission]
DEFAULT_BRANCH_POLICY=HUMAN_MERGE_ONLY
STALE_OWNERSHIP_MINUTES=90
DEFAULT_PR_POLICY=OPEN_WHEN_IMPLEMENTATION_EXISTS
```

## TARGET_REPOSITORY

`SELF` means the repository containing this Phase0 installation.

If a deployment intentionally controls another repository, replace it with exact `owner/repo` and ensure current human authority permits that scope.

## FLEET_SIZE / AGENTS

Every scheduled automation has one persistent identity listed in `AGENTS`.

The IDs are identities, not permanent job titles.

`50-AGENT-STATE-MACHINES.md` adapts behavior for one, two, or three-or-more agents.

## Coordination issue

There are exactly two trusted open coordination issues carrying `COORDINATION_BODY_MARKER`, one per slot A and B. `COORDINATION_TRUSTED_AUTHORS` defaults to `geromet`; the switch threshold defaults to 2,000 and must not exceed 2,400. See `160-COORDINATION-SLOTS.md`.

Do not depend on a fixed issue number.

## Trusted-author source and new-installation preflight

**Declaration is not enforcement.** The `COORDINATION_TRUSTED_AUTHORS` line above documents intended issue authors; editing this Markdown alone **does not** change the principals admitted by the running GitHub Actions workflow or the Python archiver. Do not regard a changed config file, two apparently valid issue bodies, or a successful `status` command as complete authorization to schedule the fleet. See [issue #349](https://github.com/Distributed-Minds/Fleet-Control-Public/issues/349) for the separately gated runtime preflight implementation.

- The existing `scripts/coordination_archive.py` chooses trusted authors from an explicit `--trusted-authors` option, then the process `COORDINATION_TRUSTED_AUTHORS` environment variable, then its built-in `geromet` fallback. An explicitly supplied empty CLI/environment value is invalid in Python; absent input uses the fallback. This selection does **not** read the trusted-author line in this Markdown file.
- The default-branch `.github/workflows/coordination-slots.yml` passes `vars.COORDINATION_TRUSTED_AUTHORS || 'geromet'` into Python. Its three `issue_comment` job predicates also use the repository variable in a comma-delimited membership expression **without trimming whitespace**. Python does trim comma fields. For a new installation, require a nonempty, comma-separated list of case-insensitively distinct GitHub logins **with no spaces, empty elements, or duplicate logins**; for example, `geromet,alice`, not `geromet, alice`. A value accepted by one layer is not necessarily admitted by the other.
- Before enabling a new fleet, inspect the **actual default-branch workflow**, its `COORDINATION_SOURCE_REF` checkout target, the repository Actions variable (including unset/empty fallback), any explicit CLI override, and this declaration. Compare normalized intended authors against effective runtime authors **and** event-trigger admission. Obtain the actual GitHub issue creators for both open, marker-matching A/B slots; issue body text cannot appoint a trusted author. Any malformed value, mismatch, untrusted creator, missing/duplicate slot, or ambiguous effective source means **BLOCKED for new-installation readiness** pending an explicit operator correction.
- The existing read-only command `python3 scripts/coordination_archive.py status --repo OWNER/REPO --config Phase0/05-FLEET-CONFIG.md --trusted-authors geromet` checks slot status **for that supplied author set only**. It neither reads this declaration as an authority source nor proves that the default-branch event predicates agree. Never infer overall preflight READY from `status` alone.

This paragraph is **guidance, not an installed runtime guard**: current production `rotate`/`maintain` behavior is unchanged. Do not automatically rewrite GitHub variables, extend trust, interrupt existing archive recovery, enable deletion, or change the default-branch workflow to enforce this check. Migration and a fail-closed executable preflight require #349's separately verified implementation, safety review, and human-owned integration.

## Stale ownership

Age alone never proves ownership is abandoned. The configured timeout is only the earliest point at which recovery investigation may begin.

## Default branch

`HUMAN_MERGE_ONLY` means agents may prepare and repair non-default branches and PRs but do not merge to the default branch.

Changing this policy should be an explicit human decision, not an inference by an agent.
