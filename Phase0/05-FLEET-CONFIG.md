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

There must be exactly one live coordination issue carrying `COORDINATION_BODY_MARKER`.

Do not depend on a fixed issue number.

## Stale ownership

Age alone never proves ownership is abandoned. The configured timeout is only the earliest point at which recovery investigation may begin.

## Default branch

`HUMAN_MERGE_ONLY` means agents may prepare and repair non-default branches and PRs but do not merge to the default branch.

Changing this policy should be an explicit human decision, not an inference by an agent.
