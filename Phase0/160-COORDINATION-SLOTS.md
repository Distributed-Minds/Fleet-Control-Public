# Coordination slots A and B

The fleet uses two open GitHub issues with the exact configured `COORDINATION_ISSUE_TITLE`. Each issue body starts with the exact configured `COORDINATION_BODY_MARKER` (`FLEET_COORDINATION_V1`), then has exactly one line for each field:

```text
FLEET_COORDINATION_V1
COORDINATION_SLOT=A
COORDINATION_STATE=ACTIVE
COORDINATION_EPOCH=1
```

Slot is `A` or `B`; state is `ACTIVE`, `DRAINING`, or `STANDBY`; epoch is a nonnegative integer. Remaining body text is preserved during state changes. Issues must be authored by an account in `COORDINATION_TRUSTED_AUTHORS` (default `geromet`). Untrusted issues and comments never choose the live slot or establish protected ownership state.

## Discovery and writing

Agents and maintenance automation find both issues by configured title, first-line marker, and trusted issue author. Discovery fails closed unless exactly one valid issue exists for each slot and at least one is ACTIVE. The ACTIVE issue with the highest epoch is the live issue. Two ACTIVE issues during a flip are valid; the higher epoch wins, so there is no interval without a writer. Agents append coordination records only to that issue.

`status` reports both issue numbers, states, epochs, current comment counts, and thresholds. The GitHub hard cutoff is 2,500 comments. `COORDINATION_SWITCH_AT` defaults to 2,000 and must be an integer from 100 through 2,400. The standby headroom limit defaults to 500 (`COORDINATION_STANDBY_MAX`).

## Rotation and maintenance

At the switch threshold, the archiver checks that the standby has at most the configured headroom. If it does not, the run attempts to archive and compact the standby, reports its resulting count, and does not flip if it remains too large. A flip patches the standby to `ACTIVE` with epoch `max(active_epoch, standby_epoch)+1` first, then patches the former ACTIVE issue to `DRAINING`. These are body edits only. If a run stops between the two edits, the higher-epoch ACTIVE slot is already live; rerunning `rotate` marks the lower-epoch ACTIVE issue DRAINING.

Every six hours, the archiver prepares immutable archive segments. DRAINING issues are compacted and become STANDBY after their live count is at most 500. The newest-tail keep rule defaults to 300 comments so a drained slot can fit below that target; trusted latest-state, active-run/predecessor, and body-reference keep rules remain in force. An ACTIVE issue may be compacted only above `COORDINATION_COMPACT_ABOVE` (default 1,500). All compaction is a dry run unless deletion is explicitly enabled. Archives live under `Phase0/archives/coordination/issue-<N>/` on the dedicated `coordination-archive` branch.

With `ARCHIVE_DELETE_ENABLED` unset or false, archived comments remain live and neither slot shrinks. The two issues therefore have finite remaining capacity: up to about 5,000 comments total from empty slots, or about 6 days at the observed 780 comments per day. `status` reports the remaining capacity and warns that rotation will block when the active slot reaches `switch-at` and the standby is not below `standby-max`. In that state, `rotate` refuses to flip into a non-empty standby. Enable deletion only after verifying archive read-back, or compact manually to reclaim space.

After a flip, the former ACTIVE slot is DRAINING. With deletion disabled it cannot shrink and cannot become STANDBY, so a second flip is blocked when the current ACTIVE reaches `switch-at`. This is the designed finite-capacity behavior; `status` reports the DRAINING state and warning. Maintenance must compact the DRAINING issue to `standby-max` or below before it promotes it to STANDBY.

Deletion requires repository variable `ARCHIVE_DELETE_ENABLED=true`, `--confirm-delete`, explicit trusted-author filtering, and byte-identical read-back of each remote manifest entry and segment covering a comment selected for deletion immediately before that deletion. Comments newer than the remote archive are not deletion candidates. Pagination gaps, schema errors, mismatches, and request-budget exhaustion fail closed or defer work.

## Operator runbook

### One-time setup

1. Merge this feature PR into `phase0/public-v0` first. This places the `prepare-slots`, `rotate`, `maintain`, and `status` modes and the migrated `Phase0/archives/coordination/issue-52/` seed on the source ref used by the workflow.
2. Merge the single workflow file `.github/workflows/coordination-slots.yml` to `main` through the repository's human-only merge process. Scheduled and issue-comment workflows run from `main`; executable code and configuration come from `COORDINATION_SOURCE_REF` (default `phase0/public-v0`). Ensure the workflow token can write issues and contents for the archive branch.
3. Create slot B as one open issue titled exactly `[fleet-control] coordination`, authored by a trusted author, with first body line `FLEET_COORDINATION_V1` and body lines `COORDINATION_SLOT=B`, `COORDINATION_STATE=STANDBY`, and `COORDINATION_EPOCH=0`. Add the corresponding slot lines to existing issue #168: `COORDINATION_SLOT=A`, `COORDINATION_STATE=ACTIVE`, and `COORDINATION_EPOCH=1`, preserving its content and marker. There is no GitHub label to apply; these are body fields. Both issue authors must be trusted.
4. Run `status` with the workflow's `workflow_dispatch` button and read the output. Dispatch runs the read-only status job; it does not run archive, compact, or rotate. Confirm it finds exactly A and B with A ACTIVE and B STANDBY before enabling agents or scheduled maintenance. If either slot line or slot B is missing, the first scheduled run fails closed with an actionable `expected trusted open coordination slots A and B` message before any issue edit or comment deletion.
5. Only after reading successful status output, optionally set repository variable `ARCHIVE_DELETE_ENABLED=true`. Keep it unset or false during initial validation. Set `COORDINATION_TRUSTED_AUTHORS` to `geromet` or another approved comma-separated list of GitHub logins, with no spaces (for example, `geromet,alice`). Optionally configure `COORDINATION_SWITCH_AT`, `COORDINATION_STANDBY_MAX`, and `COORDINATION_COMPACT_ABOVE`; switch-at must be an integer from 100 through 2,400.

The archive branch is created by the first scheduled maintenance run when the migrated #52 seed or eligible slot comments produce archive files to commit. If there is nothing to archive, no branch is created on that run.

### Agent discovery

Run `python3 scripts/coordination_archive.py status --repo OWNER/REPO` or apply the discovery rule above using GitHub's open issue API. Write only to the reported highest-epoch ACTIVE issue. Never use issue #168 as a permanent hard-coded live target after setup.

### Recovering a failed flip

Rerun `rotate`. If both issues are ACTIVE, discovery selects the higher epoch and rotation patches the lower epoch to DRAINING. If the old issue is still ACTIVE and the standby is STANDBY, a normal rotation checks counts and performs the two ordered body edits. If standby headroom is too low, let maintenance archive/compact it and retry after it reaches the configured maximum. If issue metadata is malformed, repair the body fields manually while preserving the marker and all unrelated text, then rerun read-only `status`.

### Enabling deletion

Keep deletion disabled while validating archives. To enable it, set repository variable `ARCHIVE_DELETE_ENABLED` to the exact string `true`; the workflow passes `--confirm-delete` only when this variable is true, and the script independently enforces the same environment gate plus explicit trusted authors. Ensure the archive branch has been pushed and the workflow's segment read-back succeeds. Each comment is independently rechecked against archive evidence and current protection state before deletion.

## Compatibility

The pre-slot flat archive for issue #52 was moved once to `issue-52/`. Migration rewrites only manifest segment paths; segment bytes and recorded SHA-256 values remain unchanged. New archives are always isolated by issue number.
