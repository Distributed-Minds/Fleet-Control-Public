#!/usr/bin/env python3
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures" / "merge-base-topology-spec2.json"


def canonical_bases(case):
    return sorted(set(case.get("bases", [])))


def topology(case):
    if not case.get("complete", False):
        return "UNPROVABLE"
    bases = canonical_bases(case)
    if not bases:
        return "NONE"
    if len(bases) == 1:
        return "UNIQUE"
    return "MULTIPLE"


def set_identity(case):
    payload = {"history_view": case["history_view"], "bases": canonical_bases(case)}
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def virtual_disposition(case):
    virtual = case.get("virtual")
    if topology(case) != "MULTIPLE":
        return None
    if not virtual or not virtual.get("supported", False):
        return "UNSUPPORTED"
    required = ("algorithm", "version", "options", "intermediates", "result")
    if any(k not in virtual for k in required):
        return "UNPROVABLE"
    return "SUPPORTED"


def computation_identity(case):
    if virtual_disposition(case) != "SUPPORTED":
        return None
    v = case["virtual"]
    payload = {
        "history_view": case["history_view"],
        "bases": canonical_bases(case),
        "algorithm": v["algorithm"],
        "version": v["version"],
        "options": v["options"],
        "intermediates": v["intermediates"],
        "result": v["result"],
    }
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def stale_against(case, previous):
    return (
        case["history_view"] != previous["history_view"]
        or canonical_bases(case) != sorted(set(previous.get("bases", [])))
    )


def run_git(repo, *args, input_text=None, check=True):
    env = os.environ.copy()
    env.update(
        {
            "GIT_AUTHOR_NAME": "Fleet topology fixture",
            "GIT_AUTHOR_EMAIL": "fleet-topology@example.invalid",
            "GIT_COMMITTER_NAME": "Fleet topology fixture",
            "GIT_COMMITTER_EMAIL": "fleet-topology@example.invalid",
            "GIT_AUTHOR_DATE": "2000-01-01T00:00:00+00:00",
            "GIT_COMMITTER_DATE": "2000-01-01T00:00:00+00:00",
        }
    )
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        input=input_text,
        text=True,
        capture_output=True,
        env=env,
    )
    if check and proc.returncode:
        raise RuntimeError(
            f"git {' '.join(args)} failed ({proc.returncode}): {proc.stderr.strip()}"
        )
    return proc.stdout.strip(), proc.returncode


def real_git_topology_errors():
    errors = []
    with tempfile.TemporaryDirectory(prefix="fleet-merge-base-") as raw:
        repo = Path(raw)
        run_git(repo, "init", "-q")
        tree, _ = run_git(repo, "mktree", input_text="")

        def commit(message, *parents):
            args = ["commit-tree", tree, "-m", message]
            for parent in parents:
                args.extend(["-p", parent])
            value, _ = run_git(repo, *args)
            return value

        root = commit("root")
        left = commit("left", root)
        right = commit("right", root)
        unique_tip = commit("unique-tip", left)

        unique, rc = run_git(repo, "merge-base", "--all", unique_tip, right, check=False)
        if rc != 0 or unique.splitlines() != [root]:
            errors.append("real-unique-base: expected exactly the root commit")

        merge_left = commit("merge-left", left, right)
        merge_right = commit("merge-right", right, left)
        criss, rc = run_git(repo, "merge-base", "--all", merge_left, merge_right, check=False)
        if rc != 0 or set(criss.splitlines()) != {left, right} or len(criss.splitlines()) != 2:
            errors.append("real-criss-cross: expected exactly two best merge bases")

        git_dir_text, _ = run_git(repo, "rev-parse", "--git-dir")
        git_dir = Path(git_dir_text)
        if not git_dir.is_absolute():
            git_dir = repo / git_dir
        shallow = git_dir / "shallow"
        shallow.write_text(left + "\n")
        shallow_result, shallow_rc = run_git(
            repo, "merge-base", "--all", unique_tip, right, check=False
        )
        shallow.unlink()
        if shallow_rc == 0 or shallow_result:
            errors.append("real-shallow-boundary: full-history base incorrectly survived")

        replacement_root = commit("replacement-root")
        replacement_left = commit("replacement-left", replacement_root)
        run_git(repo, "replace", left, replacement_left)
        replaced_result, replaced_rc = run_git(
            repo, "merge-base", "--all", unique_tip, right, check=False
        )
        if replaced_rc == 0 or replaced_result:
            errors.append("real-replacement-ancestry: original base incorrectly survived")

    return errors


def main():
    cases = json.loads(FIXTURES.read_text())
    by_name = {c["name"]: c for c in cases}
    errors = []

    for case in cases:
        if "expect" in case and topology(case) != case["expect"]:
            errors.append(f"{case['name']}: expected {case['expect']}, got {topology(case)}")
        if "expect_virtual" in case and virtual_disposition(case) != case["expect_virtual"]:
            errors.append(
                f"{case['name']}: expected virtual {case['expect_virtual']}, got {virtual_disposition(case)}"
            )
        if "same_set_as" in case and set_identity(case) != set_identity(by_name[case["same_set_as"]]):
            errors.append(f"{case['name']}: canonical base-set identity changed under reordering")
        if "same_computation_as" in case and computation_identity(case) != computation_identity(by_name[case["same_computation_as"]]):
            errors.append(f"{case['name']}: computation identity changed under equivalent reordering")
        if "different_computation_from" in case and computation_identity(case) == computation_identity(by_name[case["different_computation_from"]]):
            errors.append(f"{case['name']}: material computation drift aliased")
        if "stale_against" in case and not stale_against(case, case["stale_against"]):
            errors.append(f"{case['name']}: history/base drift was not stale")

    errors.extend(real_git_topology_errors())

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        raise SystemExit(1)
    print(f"PASS: {len(cases)} modeled cases + 4 real Git topology probes")


if __name__ == "__main__":
    main()
