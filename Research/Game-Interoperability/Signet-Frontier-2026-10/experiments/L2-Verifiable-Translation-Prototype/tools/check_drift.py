#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

DEPENDENCIES = (
    "integration_surface_digest",
    "semantic_profile_digest",
    "transform_digest",
    "candidate_set_digest",
)

def load(path: str):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def semantic_entries(profile):
    for entry in profile["entries"]:
        if entry["authority_class"] == "SEMANTIC_MAPPING":
            yield entry

def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} PROFILE DRIFT_EVENT", file=sys.stderr)
        return 2

    profile = load(argv[1])
    drift = load(argv[2])
    current = drift["current"]
    game_version_changed = "game_version" in drift.get("changed", [])

    exit_code = 0
    for entry in semantic_entries(profile):
        declared = set(entry.get("invalidates_on", []))
        changed = []

        for dep in DEPENDENCIES:
            if dep in declared and dep in current and entry.get(dep) != current[dep]:
                changed.append(dep)

        if game_version_changed and "game_version" in declared:
            changed.append("game_version")

        status = "SUSPECT" if changed else entry.get("status", "VALID")
        print(f"semantic mapping {entry['id']}: {status}")
        for dep in changed:
            print(f"changed dependency: {dep}")

        if status == "SUSPECT":
            exit_code = max(exit_code, 10)

    return exit_code

if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
