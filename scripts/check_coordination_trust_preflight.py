#!/usr/bin/env python3
"""Read-only preflight for declared, Python-runtime and Actions coordination trust.

No API requests, environment mutation, slot edits, or permission changes. An
existing maintenance installation is NOT automatically blocked by this check.
Pass --actions-variable with the raw GitHub repository variable, or
--actions-unset if that variable is absent; do not supply the job's fallback.
"""
from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path
from unittest import mock

import coordination_archive as archive

JOBS = ("archive", "compact", "rotate")
MEMBERSHIP = "contains(format(',{0},', vars.COORDINATION_TRUSTED_AUTHORS || 'geromet'), format(',{0},', github.event.comment.user.login))"
COMMENT_GUARD = ("(github.event.issue.pull_request == null && " + MEMBERSHIP +
                 " && github.event.issue.comments >= 1000)")
ARCHIVE_COMPACT_IF = ("github.event_name != 'workflow_dispatch' && "
                      "(github.event_name != 'issue_comment' || " + COMMENT_GUARD + ")")
ROTATE_IF = ("$" + "{{ !cancelled() && needs.archive.result == 'success' && "
             "(github.event_name != 'issue_comment' || " + COMMENT_GUARD + ") }}")
EXPECTED_JOB_IF = {"archive": ARCHIVE_COMPACT_IF, "compact": ARCHIVE_COMPACT_IF,
                   "rotate": ROTATE_IF}
LOGIN = re.compile(r"[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?\Z")


def parse_authors(raw: str, label: str, errors: list[str]) -> set[str]:
    """Strict new-installation grammar; not a change to production trusted_authors()."""
    parts = raw.split(",")
    if (not raw or any(not LOGIN.fullmatch(p) or p.startswith("-") or p.endswith("-") for p in parts)
            or len({p.lower() for p in parts}) != len(parts)):
        errors.append(f"{label}: invalid comma-separated GitHub login list (no whitespace, empty or duplicate tokens)")
        return set()
    return {p.lower() for p in parts}


def declared_config(path: Path, errors: list[str]) -> set[str]:
    """Read a single versioned Markdown config block, not prose or examples."""
    lines = path.read_text(encoding="utf-8").splitlines()
    blocks: list[list[str]] = []
    active: list[str] | None = None
    for line in lines:
        if active is None and line == "```text":
            active = []
        elif active is not None and line == "```":
            if any(item.startswith("PHASE0_CONFIG_VERSION=") for item in active):
                blocks.append(active)
            active = None
        elif active is not None:
            active.append(line)
    if active is not None:
        errors.append("declared: unterminated text configuration fence")
        return set()
    if len(blocks) != 1:
        errors.append("declared: expected exactly one versioned text configuration fence")
        return set()
    config = blocks[0]
    versions = [line for line in config if line.startswith("PHASE0_CONFIG_VERSION=")]
    if versions != ["PHASE0_CONFIG_VERSION=1"]:
        errors.append("declared: unsupported or duplicate configuration version")
        return set()
    values = [line.split("=", 1)[1] for line in config
              if line.startswith("COORDINATION_TRUSTED_AUTHORS=")]
    if len(values) != 1:
        errors.append("declared: expected one COORDINATION_TRUSTED_AUTHORS= in configuration fence")
        return set()
    return parse_authors(values[0], "declared", errors)

def job_conditions(path: Path, errors: list[str]) -> dict[str, str]:
    jobs: dict[str, str] = {}
    current = None
    for line in path.read_text(encoding="utf-8").splitlines():
        m = re.fullmatch(r"  ([A-Za-z][\w-]*):\s*", line)
        if m:
            current = m.group(1)
        elif current in JOBS and line.startswith("    if: "):
            if current in jobs:
                errors.append(f"workflow: duplicate {current} job condition")
            jobs[current] = line[len("    if: "):]
    for job in JOBS:
        if jobs.get(job) != EXPECTED_JOB_IF[job]:
            errors.append(f"workflow: {job} admission expression changed; cannot certify parity")
    return jobs



def valid_slot_snapshots(issues: object, errors: list[str]) -> bool:
    """Reject malformed offline GitHub issue objects before archive discovery.

    This is a read-only input boundary; it does not change installed
    coordination_archive discovery, trust, or live slot selection rules.
    """
    if not isinstance(issues, list) or not all(isinstance(item, dict) for item in issues):
        errors.append("slots: expected a list of GitHub issue snapshots")
        return False
    for index, item in enumerate(issues):
        creator = item.get("user") or item.get("author") or {}
        if (not isinstance(item.get("number"), int) or isinstance(item.get("number"), bool)
                or not isinstance(item.get("state"), str)
                or not isinstance(item.get("title"), str)
                or not isinstance(item.get("body"), (str, type(None)))
                or not isinstance(creator, dict)
                or not isinstance(creator.get("login"), str)):
            errors.append(f"slots: malformed issue snapshot at index {index}")
            return False
    return True


def preflight(config: Path, workflow: Path, actions_variable: str | None,
              runtime_override: str | None, slots_path: Path | None) -> dict:
    errors: list[str] = []
    declared = declared_config(config, errors)
    job_conditions(workflow, errors)
    # GitHub Actions expression uses its fallback for an unset or empty variable.
    actions_raw = actions_variable or "geromet"
    actions = parse_authors(actions_raw, "actions", errors)
    try:
        effective = archive.trusted_authors(argparse.Namespace(trusted_authors=runtime_override))
    except (RuntimeError, ValueError) as exc:
        errors.append(f"runtime: {exc}")
        effective = set()
    # Do not normalize a malformed runtime value merely because production does.
    runtime_raw = (runtime_override if runtime_override is not None else
                   os.environ.get("COORDINATION_TRUSTED_AUTHORS", "geromet"))
    parse_authors(runtime_raw, "runtime", errors)
    if declared != effective:
        errors.append("declared/runtime trusted authors differ")
    if actions != effective:
        errors.append("Actions event-admission/runtime trusted authors differ")
    if actions != declared:
        errors.append("Actions event-admission/declared trusted authors differ")
    # Membership model matches the exact expression required above. Refuse to
    # infer behavior after an Actions YAML predicate change.
    candidates = declared | effective | actions | {"untrusted", "geromet"}
    for who in sorted(candidates):
        action_member = f",{who.lower()}," in f",{actions_raw.lower()},"
        if action_member != (who in effective):
            errors.append(f"workflow/Python membership differs for {who}")
    slot_status = "NOT_CHECKED"
    if slots_path is not None:
        issues = json.loads(slots_path.read_text(encoding="utf-8"))
        if not valid_slot_snapshots(issues, errors):
            slot_status = "INVALID"
        else:
            try:
                # Reuse the installed Python discovery and epoch rules offline.
                with mock.patch.object(archive, "fetch_open_issues", return_value=issues):
                    slots = archive.discover_slots("offline/offline", config, effective)
                chosen = archive.live_slot(slots)
                slot_status = f"VALID_ACTIVE_{chosen['slot']}_EPOCH_{chosen['epoch']}"
            except (RuntimeError, KeyError, ValueError, TypeError, AttributeError) as exc:
                errors.append(f"slots: {exc}")
                slot_status = "INVALID"
    return {"status": "BLOCKED" if errors else ("TRUST_AND_OFFLINE_SLOTS_CONSISTENT" if slots_path else "TRUST_CONFIG_CONSISTENT_ONLY"),
            "declared": sorted(declared), "effective_runtime": sorted(effective),
            "actions_event": sorted(actions), "actions_source": ("fallback-unset" if actions_variable is None else "fallback-empty" if not actions_variable else "repository-variable"),
            "runtime_source": ("cli-override" if runtime_override is not None else "environment" if "COORDINATION_TRUSTED_AUTHORS" in os.environ else "fallback"),
            "slots": slot_status, "errors": sorted(set(errors))}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=Path("Phase0/05-FLEET-CONFIG.md"))
    parser.add_argument("--workflow", type=Path, default=Path(".github/workflows/coordination-slots.yml"))
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--actions-variable", help="raw Actions repository variable (not the fallback)")
    group.add_argument("--actions-unset", action="store_true")
    parser.add_argument("--runtime-override", default=None, help="actual Python --trusted-authors override, if any")
    parser.add_argument("--slots-json", type=Path, help="optional read-only recorded issue snapshots")
    args = parser.parse_args(argv)
    try:
        result = preflight(args.config, args.workflow, args.actions_variable,
                           args.runtime_override, args.slots_json)
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        result = {"status": "BLOCKED", "errors": [f"preflight input unavailable or invalid: {exc}"]}
    print(json.dumps(result, sort_keys=True))
    return 2 if result["status"] == "BLOCKED" else 0


if __name__ == "__main__":
    raise SystemExit(main())
