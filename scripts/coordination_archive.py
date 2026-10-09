#!/usr/bin/env python3
"""Lossless, fail-closed maintenance for a Phase0 coordination issue."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

API = "https://api.github.com"
ACTIVE_STATES = {"OWNED", "WORKING"}
VALID_STATES = {"INTENT", "OWNED", "WORKING", "HANDOFF", "RELEASE", "YIELD", "RECOVERED"}
SCHEMA = 1
HARD_COMMENT_CAP = 2500
MAX_SWITCH_AT = 2400
DEFAULT_TRUSTED_AUTHORS = {"geromet"}


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def token() -> str:
    value = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if not value:
        raise RuntimeError("GH_TOKEN/GITHUB_TOKEN is required")
    return value


def api_request(repo: str, method: str, path: str, body: Any = None,
                allow_not_found: bool = False, retries: int = 5):
    data = None if body is None else json.dumps(body).encode("utf-8")
    for attempt in range(retries):
        req = urllib.request.Request(
            f"{API}{path}", data=data, method=method,
            headers={"Accept": "application/vnd.github+json",
                     "Authorization": f"Bearer {token()}",
                     "X-GitHub-Api-Version": "2022-11-28",
                     "User-Agent": "fleet-control-coordination-archive"})
        try:
            with urllib.request.urlopen(req, timeout=60) as response:
                raw = response.read()
                return (json.loads(raw.decode("utf-8")) if raw else None,
                        dict(response.headers.items()))
        except urllib.error.HTTPError as exc:
            if exc.code == 404 and allow_not_found:
                return None, dict(exc.headers.items())
            retry_after = exc.headers.get("Retry-After")
            remaining = exc.headers.get("X-RateLimit-Remaining")
            reset = exc.headers.get("X-RateLimit-Reset")
            exhausted = exc.code == 403 and remaining == "0"
            retryable = exc.code in (403, 429, 500, 502, 503, 504)
            if retryable and attempt + 1 < retries:
                if retry_after and retry_after.isdigit():
                    delay = int(retry_after)
                elif exhausted and reset and reset.isdigit():
                    delay = max(1, int(reset) - int(time.time()) + 1)
                elif exc.code in (403, 429):
                    delay = min(60 * (attempt + 1), 300)
                else:
                    delay = min(2 ** attempt, 30)
                time.sleep(delay)
                continue
            detail = exc.read().decode("utf-8", "replace")
            raise RuntimeError(f"GitHub API {method} {path} failed: {exc.code} {detail}") from exc
        except (urllib.error.URLError, TimeoutError) as exc:
            if attempt + 1 < retries:
                time.sleep(min(2 ** attempt, 30))
                continue
            raise RuntimeError(f"GitHub API {method} {path} exhausted retries: {exc}") from exc
    raise RuntimeError("GitHub API retry budget exhausted")


def validate_comment(comment: Any) -> None:
    if not isinstance(comment, dict):
        raise RuntimeError("unsupported comment schema: expected object")
    required = {"id", "body", "user", "created_at", "updated_at"}
    if not required.issubset(comment):
        raise RuntimeError("unsupported comment schema: required field missing")
    if not isinstance(comment["id"], int) or isinstance(comment["id"], bool):
        raise RuntimeError("unsupported comment schema: id must be an integer")
    if comment["body"] is not None and not isinstance(comment["body"], str):
        raise RuntimeError("unsupported comment schema: body must be text or null")
    if comment["user"] is not None and not isinstance(comment["user"], dict):
        raise RuntimeError("unsupported comment schema: user must be an object or null")


def fetch_comments(repo: str, issue: int) -> list[dict[str, Any]]:
    all_comments: list[dict[str, Any]] = []
    seen: set[int] = set()
    page = 1
    while True:
        batch, headers = api_request(
            repo, "GET", f"/repos/{repo}/issues/{issue}/comments?per_page=100&page={page}")
        if not isinstance(batch, list):
            raise RuntimeError(f"unsupported comments response on page {page}")
        if not batch:
            # Follow Link pagination if present. An empty page with a subsequent page
            # means a gap or unstable frontier, so never treat it as complete.
            if re.search(r'<[^>]+>;\s*rel="next"', headers.get("Link", "")):
                raise RuntimeError(f"pagination gap: empty page {page} advertises a next page")
            break
        for item in batch:
            validate_comment(item)
            if item["id"] in seen:
                raise RuntimeError(f"pagination overlap/gap: duplicate comment id {item['id']}")
            seen.add(item["id"])
        if all_comments and batch[0]["id"] <= all_comments[-1]["id"]:
            raise RuntimeError("pagination gap or unstable ordering: comment IDs are not increasing")
        all_comments.extend(batch)
        link = headers.get("Link", "")
        has_next = bool(re.search(r'<[^>]+>;\s*rel="next"', link))
        if len(batch) < 100:
            if has_next:
                raise RuntimeError(f"pagination gap: short page {page} advertises a next page")
            break
        # GitHub may omit Link on the last full page; request the next page and
        # require either a full/short valid page or a clean empty terminus.
        page += 1
    return all_comments


def fetch_issue(repo: str, issue: int) -> dict[str, Any]:
    obj, _ = api_request(repo, "GET", f"/repos/{repo}/issues/{issue}")
    if not isinstance(obj, dict) or not isinstance(obj.get("body", ""), (str, type(None))):
        raise RuntimeError("unsupported issue response schema")
    return obj


def fields(body: str) -> dict[str, str]:
    result = {}
    for part in body.split("|"):
        part = part.strip()
        if "=" in part:
            key, value = part.split("=", 1)
            result[key.strip()] = value.strip()
    return result


def configured_agents(config_path: Path) -> set[str]:
    for line in config_path.read_text(encoding="utf-8").splitlines():
        if line.strip().startswith("AGENTS="):
            agents = {item.strip() for item in line.split("=", 1)[1].split(",") if item.strip()}
            if agents:
                return agents
    raise RuntimeError(f"AGENTS not found in {config_path}")


def trusted_authors(args: argparse.Namespace) -> set[str]:
    configured = getattr(args, "trusted_authors", None)
    if configured is not None:
        raw = configured
    elif "COORDINATION_TRUSTED_AUTHORS" in os.environ:
        raw = os.environ["COORDINATION_TRUSTED_AUTHORS"]
    else:
        return set(DEFAULT_TRUSTED_AUTHORS)
    parts = [item.strip() for item in raw.split(",")]
    if not parts or any(not item for item in parts):
        raise RuntimeError("COORDINATION_TRUSTED_AUTHORS must be a non-empty comma-separated list of logins")
    return {item.lower() for item in parts}


def coordination_config(config_path: Path) -> dict[str, str]:
    wanted = {"COORDINATION_ISSUE_TITLE", "COORDINATION_BODY_MARKER"}
    values = {}
    for line in config_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        for key in wanted:
            if line.startswith(key + "="):
                values[key] = line.split("=", 1)[1]
    if wanted - values.keys():
        raise RuntimeError("coordination title or body marker missing from config")
    return values


def slot_fields(body: str) -> dict[str, str]:
    result = {}
    for line in (body or "").splitlines():
        match = re.fullmatch(r"(COORDINATION_SLOT|COORDINATION_STATE|COORDINATION_EPOCH)=(.*)", line.strip())
        if match:
            if match.group(1) in result:
                raise RuntimeError(f"duplicate {match.group(1)} coordination field")
            result[match.group(1)] = match.group(2)
    return result


def discover_slots(repo: str, config_path: Path, trusted: set[str] | None = None) -> dict[str, dict[str, Any]]:
    config = coordination_config(config_path)
    trusted = {x.lower() for x in (trusted if trusted is not None else DEFAULT_TRUSTED_AUTHORS)}
    if not trusted:
        raise RuntimeError("COORDINATION_TRUSTED_AUTHORS must contain at least one author")
    candidates = fetch_open_issues(repo)
    slots: dict[str, dict[str, Any]] = {}
    for issue in candidates:
        body = issue.get("body") or ""
        author = ((issue.get("user") or issue.get("author") or {}).get("login") or "").lower()
        if (issue.get("state") != "open" or issue.get("title") != config["COORDINATION_ISSUE_TITLE"]
                or body.splitlines()[:1] != [config["COORDINATION_BODY_MARKER"]]
                or author not in trusted):
            continue
        info = slot_fields(body)
        slot, state, epoch = info.get("COORDINATION_SLOT"), info.get("COORDINATION_STATE"), info.get("COORDINATION_EPOCH", "")
        if slot not in {"A", "B"} or state not in {"ACTIVE", "DRAINING", "STANDBY"} or not epoch.isdigit():
            raise RuntimeError(f"trusted coordination issue #{issue.get('number')} has invalid slot metadata")
        if slot in slots:
            raise RuntimeError(f"multiple trusted coordination issues claim slot {slot}")
        slots[slot] = {**issue, "slot": slot, "slot_state": state, "epoch": int(epoch)}
    if set(slots) != {"A", "B"}:
        raise RuntimeError(f"expected trusted open coordination slots A and B; found {', '.join(sorted(slots)) or 'none'}")
    active = [x for x in slots.values() if x["slot_state"] == "ACTIVE"]
    if not active:
        raise RuntimeError("no ACTIVE coordination slot; refusing to proceed")
    if len(active) == 2 and active[0]["epoch"] == active[1]["epoch"]:
        raise RuntimeError("two ACTIVE coordination slots have the same epoch")
    return slots


def fetch_open_issues(repo: str) -> list[dict[str, Any]]:
    result: list[dict[str, Any]] = []
    page = 1
    while True:
        batch, headers = api_request(repo, "GET", f"/repos/{repo}/issues?state=open&per_page=100&page={page}")
        if not isinstance(batch, list):
            raise RuntimeError(f"unsupported open issues response on page {page}")
        if not batch:
            if re.search(r'<[^>]+>;\s*rel="next"', headers.get("Link", "")):
                raise RuntimeError(f"pagination gap: empty issue page {page} advertises a next page")
            break
        result.extend(x for x in batch if "pull_request" not in x)
        if len(batch) < 100:
            if re.search(r'<[^>]+>;\s*rel="next"', headers.get("Link", "")):
                raise RuntimeError(f"pagination gap: short issue page {page} advertises a next page")
            break
        page += 1
    return result


def live_slot(slots: dict[str, dict[str, Any]]) -> dict[str, Any]:
    return max((x for x in slots.values() if x["slot_state"] == "ACTIVE"), key=lambda x: x["epoch"])


def issue_archive_dir(base: Path, issue: int) -> Path:
    name = f"issue-{issue}"
    if base.name == name:
        return base
    if re.fullmatch(r"issue-\d+", base.name):
        raise RuntimeError(f"archive-dir {base} is scoped to a different issue; pass the coordination archive base directory")
    return base / name


def replace_body_fields(body: str, updates: dict[str, str]) -> str:
    # Split only at LF. splitlines() also treats CR, U+2028, form feed, and
    # other characters as line boundaries and would rewrite operator text.
    lines = (body or "").split("\n")
    for key, value in updates.items():
        prefix = key + "="
        indexes = [i for i, line in enumerate(lines) if line.startswith(prefix)]
        if len(indexes) != 1:
            raise RuntimeError(f"coordination body must contain exactly one {key} line")
        old = lines[indexes[0]]
        if "\r" in old[:-1]:
            raise RuntimeError(f"coordination body {key} line has ambiguous lone-CR separators; refusing to edit")
        lines[indexes[0]] = prefix + value + ("\r" if old.endswith("\r") else "")
    return "\n".join(lines)


def patch_issue_body(repo: str, issue: dict[str, Any], updates: dict[str, str]) -> None:
    body = replace_body_fields(issue.get("body") or "", updates)
    api_request(repo, "PATCH", f"/repos/{repo}/issues/{issue['number']}", {"body": body})
    issue["body"] = body
    info = slot_fields(body)
    issue["slot_state"] = info["COORDINATION_STATE"]
    issue["epoch"] = int(info["COORDINATION_EPOCH"])


def record_info(comments: list[dict[str, Any]], issue_body: str, tail_min: int,
                agents: set[str], trusted: set[str] | None = None) -> tuple[set[int], dict[str, int]]:
    if tail_min < 0:
        raise RuntimeError("tail-min must be nonnegative")
    ordered = comments
    ids = [item["id"] for item in ordered]
    if ids != sorted(ids) or len(set(ids)) != len(ids):
        raise RuntimeError("comments are not in unique authoritative increasing ID order")
    keep_tail = {item["id"] for item in ordered[-tail_min:]} if tail_min else set()
    keep_agents: dict[str, int] = {}
    latest_run: dict[str, tuple[str, int]] = {}
    run_scope: dict[str, dict[str, str]] = {}
    run_ids: dict[str, list[int]] = {}
    by_id = {item["id"]: item for item in ordered}
    for item in ordered:
        body = item.get("body") or ""
        author = ((item.get("user") or {}).get("login") or "").lower()
        # Anyone can comment on a public issue: only records by trusted authors are authoritative.
        # Untrusted records are still archived losslessly but never credited as state or ownership.
        if body.startswith("PHASE0 |") and (trusted is None or author in trusted):
            parsed = fields(body)
            agent = parsed.get("agent")
            run = parsed.get("run")
            state = parsed.get("state")
            # Protect the latest valid record of EVERY observed agent identity, not only the configured
            # AGENTS list: real deployments use free-form identities (e.g. manual.*.operator).
            if agent and run and state in VALID_STATES and parsed.get("seq", "").isdigit():
                keep_agents[agent] = item["id"]
            if run and state in VALID_STATES:
                latest_run[run] = (state, item["id"])
                run_scope[run] = parsed
                run_ids.setdefault(run, []).append(item["id"])
    protected = set(keep_tail) | set(keep_agents.values())
    election_predecessors: set[int] = set()
    for run, (state, _cid) in latest_run.items():
        if state in ACTIVE_STATES:
            protected.update(run_ids.get(run, []))
            scope = run_scope[run]
            scope_keys = ("mission", "issue", "pr", "branch", "seam")
            for candidate in ordered:
                body = candidate.get("body") or ""
                if not body.startswith("PHASE0 |") or candidate["id"] >= _cid:
                    continue
                prior = fields(body)
                same_scope = any(scope.get(key) for key in scope_keys) and all(
                    prior.get(key, "") == scope.get(key, "") for key in scope_keys)
                if same_scope and prior.get("state") == "INTENT":
                    election_predecessors.add(candidate["id"])
                    protected.add(candidate["id"])
    # Follow explicit prev links from protected active-run records to retain the
    # election/intent chain even when records are unusually interleaved.
    queue = list(protected)
    predecessor_links: set[int] = set()
    while queue:
        cid = queue.pop()
        body = by_id.get(cid, {}).get("body") or ""
        prev = fields(body).get("prev")
        if prev and prev.isdigit() and int(prev) in by_id and int(prev) not in protected:
            predecessor_links.add(int(prev))
            protected.add(int(prev))
            queue.append(int(prev))
    body = issue_body or ""
    issue_refs: set[int] = set()
    for pattern in (r"issuecomment-(\d+)", r"\bprev=(\d+)\b", r"\bcomment[_ -]?id=(\d+)\b"):
        issue_refs.update(int(match.group(1)) for match in re.finditer(pattern, body))
    protected.update(issue_refs)
    return protected, {"tail": len(keep_tail), "latest_agent": len(keep_agents),
                       "active_run_records": sum(len(run_ids[r]) for r, (s, _) in latest_run.items() if s in ACTIVE_STATES),
                       "election_predecessors": len(election_predecessors),
                       "predecessor_links": len(predecessor_links),
                       "issue_body_references": len(issue_refs.intersection(ids))}


def encode_api_records(records: list[dict[str, Any]]) -> bytes:
    return b"".join((json.dumps(item, sort_keys=True, ensure_ascii=False,
                                  separators=(",", ":")) + "\n").encode("utf-8") for item in records)


def load_jsonl(path: Path) -> tuple[list[dict[str, Any]], list[bytes], bytes]:
    raw = path.read_bytes()
    if raw and not raw.endswith(b"\n"):
        raise RuntimeError("input JSONL must end with a newline for lossless segment reconstruction")
    lines = raw.splitlines(keepends=True)
    comments = []
    for number, line in enumerate(lines, 1):
        if not line.strip():
            raise RuntimeError(f"blank JSONL line at {number}")
        try:
            item = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise RuntimeError(f"invalid JSONL at line {number}: {exc}") from exc
        validate_comment(item)
        comments.append(item)
    ids = [item["id"] for item in comments]
    if ids != sorted(ids) or len(set(ids)) != len(ids):
        raise RuntimeError("input comments are not in unique increasing ID order")
    return comments, lines, raw


def manifest_path(archive_dir: Path) -> Path:
    return archive_dir / "manifest.json"


def load_manifest(archive_dir: Path, issue: int | None) -> dict[str, Any]:
    path = manifest_path(archive_dir)
    if not path.exists():
        return {"schema": SCHEMA, "issue": issue, "segments": []}
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"invalid archive manifest: {exc}") from exc
    if (not isinstance(manifest, dict) or manifest.get("schema") != SCHEMA
            or manifest.get("issue") != issue or not isinstance(manifest.get("segments"), list)):
        raise RuntimeError("unsupported or corrupt coordination archive manifest")
    return manifest


def read_segment(path: Path, expected_sha: str) -> tuple[list[dict[str, Any]], bytes]:
    raw = path.read_bytes()
    if sha256(raw) != expected_sha:
        raise RuntimeError(f"segment hash mismatch: {path}")
    records, _lines, _raw = load_jsonl(path)
    return records, raw


def local_segment_path(archive_dir: Path, relative_path: str) -> Path:
    rel = Path(relative_path)
    if rel.is_absolute() or ".." in rel.parts:
        raise RuntimeError("unsafe segment path in manifest")
    local = archive_dir / rel
    if local.exists():
        return local
    for ancestor in archive_dir.parents:
        if ancestor.name == rel.parts[0]:
            candidate = ancestor.parent / rel
            if candidate.exists():
                return candidate
    # Online manifests are repository-root-relative, while offline manifests
    # use names relative to their isolated --out-dir.
    return Path.cwd() / rel


def archived_index(archive_dir: Path, manifest: dict[str, Any]) -> dict[int, dict[str, Any]]:
    index: dict[int, dict[str, Any]] = {}
    paths: set[str] = set()
    for segment in manifest["segments"]:
        if not isinstance(segment, dict) or not {"path", "count", "first_comment_id", "last_comment_id", "sha256"}.issubset(segment):
            raise RuntimeError("unsupported segment manifest schema")
        rel = Path(segment["path"])
        if rel.is_absolute() or ".." in rel.parts or rel.as_posix() in paths:
            raise RuntimeError("unsafe or duplicate segment path in manifest")
        paths.add(rel.as_posix())
        records, _ = read_segment(local_segment_path(archive_dir, rel.as_posix()), segment["sha256"])
        if len(records) != segment["count"]:
            raise RuntimeError(f"segment count mismatch: {rel}")
        if records and (records[0]["id"] != segment["first_comment_id"] or records[-1]["id"] != segment["last_comment_id"]):
            raise RuntimeError(f"segment range mismatch: {rel}")
        for record in records:
            cid = record["id"]
            if cid in index and index[cid] != record:
                raise RuntimeError(f"conflicting archived comment id {cid}")
            index[cid] = record
    return index


def write_manifest(archive_dir: Path, manifest: dict[str, Any]) -> None:
    manifest = dict(manifest)
    manifest["segments"] = sorted(manifest["segments"], key=lambda s: (s["first_comment_id"], s["last_comment_id"], s["path"]))
    manifest["archived_comment_count"] = sum(segment["count"] for segment in manifest["segments"])
    path = manifest_path(archive_dir)
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = (json.dumps(manifest, sort_keys=True, indent=2, ensure_ascii=False) + "\n").encode("utf-8")
    # Manifest is a versioned index and may grow; only segments are immutable.
    path.write_bytes(payload)


def append_segments(archive_dir: Path, manifest: dict[str, Any], comments: list[dict[str, Any]],
                    raw_lines: list[bytes], segment_size: int) -> int:
    archived = archived_index(archive_dir, manifest) if manifest["segments"] else {}
    for item in comments:
        if item["id"] in archived and archived[item["id"]] != item:
            raise RuntimeError(f"live comment {item['id']} differs from its archived record")
    new = [(item, raw) for item, raw in zip(comments, raw_lines) if item["id"] not in archived]
    created = 0
    for offset in range(0, len(new), segment_size):
        chunk = new[offset:offset + segment_size]
        records = [item for item, _ in chunk]
        raw = b"".join(line for _, line in chunk)
        first_id, last_id = records[0]["id"], records[-1]["id"]
        filename = f"comments-{first_id}-{last_id}.jsonl"
        rel = filename if archive_dir.is_absolute() else f"{archive_dir.as_posix()}/{filename}"
        path = archive_dir / filename if archive_dir.is_absolute() else Path(rel)
        if path.exists():
            raise RuntimeError(f"refusing to overwrite existing segment {path}")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw)
        manifest["segments"].append({"path": rel, "count": len(records),
                                     "first_comment_id": first_id, "last_comment_id": last_id,
                                     "sha256": sha256(raw)})
        created += len(records)
    write_manifest(archive_dir, manifest)
    return created


def prepare_online(args: argparse.Namespace) -> int:
    comments = fetch_comments(args.repo, args.issue)
    issue_obj = fetch_issue(args.repo, args.issue)
    if len(comments) < args.trigger_count:
        print(f"prepare: below trigger; live={len(comments)} trigger={args.trigger_count}")
        return 0
    agents = configured_agents(args.config)
    protected, summary = record_info(comments, issue_obj.get("body") or "", args.tail_min, agents, trusted_authors(args))
    args.archive_dir = issue_archive_dir(args.archive_dir, args.issue)
    manifest = load_manifest(args.archive_dir, args.issue)
    raw_lines = [line for line in encode_api_records(comments).splitlines(keepends=True)]
    created = append_segments(args.archive_dir, manifest, comments, raw_lines, args.segment_size)
    eligible = sum(1 for comment in comments if comment["id"] not in protected)
    print(f"prepare: archived_new={created} live={len(comments)} keep_live={len(protected)} archivable={eligible} keep_rules={json.dumps(summary, sort_keys=True)}")
    return 0


def prepare_offline(args: argparse.Namespace) -> int:
    comments, lines, raw = load_jsonl(args.input_jsonl)
    agents = configured_agents(args.config)
    # The export cannot know current issue-body references; caller may supply an
    # offline body file for consistent count-only classification.
    issue_body = args.issue_body.read_text(encoding="utf-8") if args.issue_body else ""
    protected, summary = record_info(comments, issue_body, args.tail_min, agents, trusted_authors(args))
    if args.out_dir.exists() and any(args.out_dir.iterdir()):
        raise RuntimeError(f"offline output directory must be empty: {args.out_dir}")
    manifest = {"schema": SCHEMA, "issue": None, "source": "offline-jsonl-export",
                "source_sha256": sha256(raw), "segments": []}
    created = append_segments(args.out_dir, manifest, comments, lines, args.segment_size)
    reassembled = b"".join(local_segment_path(args.out_dir, item["path"]).read_bytes()
                           for item in manifest["segments"])
    if reassembled != raw:
        raise RuntimeError("offline segment reconstruction differs byte-for-byte from input")
    eligible = sum(1 for comment in comments if comment["id"] not in protected)
    print(f"offline: comments={len(comments)} segments={len(manifest['segments'])} keep_live={len(protected)} archivable={eligible} archived_records={created} sha256={sha256(reassembled)}")
    print("offline: keep_rule_counts=" + json.dumps(summary, sort_keys=True))
    return 0


def remote_file_bytes(repo: str, path: str, ref: str) -> bytes:
    query = urllib.parse.urlencode({"ref": ref})
    obj, _ = api_request(repo, "GET", f"/repos/{repo}/contents/{urllib.parse.quote(path)}?{query}", allow_not_found=True)
    if obj is None:
        raise FileNotFoundError(f"remote archive file not found: {path}@{ref}")
    if not isinstance(obj, dict) or obj.get("encoding") != "base64" or not isinstance(obj.get("content"), str):
        raise RuntimeError(f"remote archive file unavailable or unsupported: {path}@{ref}")
    try:
        encoded = "".join(obj["content"].split()).encode("ascii")
        return base64.b64decode(encoded, validate=True)
    except Exception as exc:
        raise RuntimeError(f"invalid base64 from archive read-back: {path}@{ref}") from exc


def remote_manifest_path(archive_dir: Path, manifest: dict[str, Any]) -> str:
    if manifest.get("segments"):
        return (Path(manifest["segments"][0]["path"]).parent / "manifest.json").as_posix()
    return (archive_dir / "manifest.json").as_posix()


def remote_manifest(repo: str, archive_dir: Path, ref: str,
                    manifest: dict[str, Any] | None = None) -> dict[str, Any]:
    raw = remote_file_bytes(repo, remote_manifest_path(archive_dir, manifest or {}), ref)
    try:
        obj = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise RuntimeError("remote archive manifest is invalid; refusing deletion") from exc
    if not isinstance(obj, dict) or not isinstance(obj.get("segments"), list):
        raise RuntimeError("remote archive manifest is unsupported; refusing deletion")
    return obj


def matching_remote_segments(local_manifest: dict[str, Any], remote: dict[str, Any], ids: set[int]) -> list[dict[str, Any]]:
    selected = []
    for segment in local_manifest["segments"]:
        if not any(segment["first_comment_id"] <= cid <= segment["last_comment_id"] for cid in ids):
            continue
        other = next((item for item in remote["segments"] if item.get("path") == segment["path"]), None)
        if other is None:
            continue
        if other != segment:
            raise RuntimeError(f"remote manifest entry differs from local segment metadata: {segment['path']}")
        selected.append(segment)
    return selected


def verify_remote(repo: str, archive_dir: Path, manifest: dict[str, Any], ref: str,
                  required_ids: set[int]) -> None:
    remote = remote_manifest(repo, archive_dir, ref, manifest)
    segments = matching_remote_segments(manifest, remote, required_ids)
    covered = {cid for segment in segments
               for cid in range(segment["first_comment_id"], segment["last_comment_id"] + 1)}
    missing = required_ids - covered
    if missing:
        raise RuntimeError(f"remote archive does not contain segment entries for comments {sorted(missing)[:10]}")
    for segment in segments:
        remote_bytes = remote_file_bytes(repo, segment["path"], ref)
        local_bytes = local_segment_path(archive_dir, segment["path"]).read_bytes()
        if remote_bytes != local_bytes or sha256(remote_bytes) != segment["sha256"]:
            raise RuntimeError(f"remote segment is not byte-identical or hash mismatched: {segment['path']}")


def verify_archive_append(args: argparse.Namespace) -> int:
    repo_dir = args.repo_dir
    archive_rel = args.archive_dir.as_posix().rstrip("/")
    listing = subprocess.run(["git", "-C", str(repo_dir), "ls-tree", "-r", "--name-only", "HEAD", "--", archive_rel],
                             check=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if listing.returncode and not args.allow_empty:
        raise RuntimeError("could not inspect existing archive branch HEAD; refusing append")
    if listing.returncode == 0:
        manifest_paths = [name for name in listing.stdout.splitlines()
                          if name.startswith(archive_rel + "/") and Path(name).name == "manifest.json"]
    else:
        manifest_paths = []
    if manifest_paths:
        for manifest_rel in manifest_paths:
            existing = subprocess.run(["git", "-C", str(repo_dir), "show", f"HEAD:{manifest_rel}"],
                                      check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                previous = json.loads(existing.stdout)
            except (UnicodeDecodeError, json.JSONDecodeError) as exc:
                raise RuntimeError(f"existing archive manifest is invalid: {manifest_rel}") from exc
            current_path = repo_dir / manifest_rel
            try:
                current = json.loads(current_path.read_bytes())
            except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
                raise RuntimeError(f"working archive manifest is missing or invalid: {manifest_rel}") from exc
            current_by_path = {item.get("path"): item for item in current.get("segments", [])}
            for segment in previous.get("segments", []):
                path = repo_dir / segment["path"]
                if not path.is_file() or sha256(path.read_bytes()) != segment["sha256"]:
                    raise RuntimeError(f"existing archive segment missing or hash mismatched: {segment['path']}")
                if current_by_path.get(segment["path"]) != segment:
                    raise RuntimeError(f"working manifest removed or changed existing segment: {segment['path']}")
    staged = subprocess.run(["git", "-C", str(repo_dir), "diff", "--cached", "--name-status"],
                            check=True, stdout=subprocess.PIPE, text=True).stdout.splitlines()
    forbidden = []
    for row in staged:
        fields = row.split("\t")
        status, paths = fields[0], fields[1:]
        path = paths[-1] if paths else ""
        allowed_segment_add = (status == "A" and path.startswith(archive_rel + "/")
                               and path.endswith(".jsonl") and Path(path).name.startswith("comments-"))
        allowed_manifest_update = (status in {"A", "M"} and path.startswith(archive_rel + "/")
                                   and Path(path).name == "manifest.json")
        if not (allowed_segment_add or allowed_manifest_update):
            forbidden.append(row)
    if forbidden:
        raise RuntimeError("append-only archive update permits only new segments and manifest updates; refused: "
                           + ", ".join(forbidden))
    print("verify-archive-append: existing segments verified; staged update is append-only")
    return 0


def switch_at_value(raw: str) -> int:
    try:
        value = int(raw)
    except (TypeError, ValueError) as exc:
        raise RuntimeError("switch-at must be an integer from 100 through 2400") from exc
    if not 100 <= value <= MAX_SWITCH_AT:
        raise RuntimeError(f"switch-at must be from 100 through {MAX_SWITCH_AT}")
    return value


def compact(args: argparse.Namespace) -> int:
    explicit_authors = (getattr(args, "trusted_authors", None) is not None
                        or "COORDINATION_TRUSTED_AUTHORS" in os.environ)
    if args.confirm_delete:
        if os.environ.get("ARCHIVE_DELETE_ENABLED") != "true":
            raise SystemExit("compact deletion refused: set ARCHIVE_DELETE_ENABLED=true after verifying archive read-back")
        if not explicit_authors or not trusted_authors(args):
            raise SystemExit("compact deletion refused: provide explicit non-empty --trusted-authors or COORDINATION_TRUSTED_AUTHORS")
    args.archive_dir = issue_archive_dir(args.archive_dir, args.issue)
    manifest = load_manifest(args.archive_dir, args.issue)
    if not manifest["segments"]:
        print("compact: no archive segments")
        return 0
    archived = archived_index(args.archive_dir, manifest)
    comments = fetch_comments(args.repo, args.issue)
    issue_obj = fetch_issue(args.repo, args.issue)
    live = {item["id"]: item for item in comments}
    candidates = sorted(cid for cid in archived if cid in live)
    mismatched = [cid for cid in candidates if archived[cid] != live[cid]]
    if mismatched:
        raise RuntimeError(f"live comments differ from archived evidence (sample IDs: {mismatched[:10]})")
    protected, summary = record_info(comments, issue_obj.get("body") or "", args.tail_min,
                                    configured_agents(args.config), trusted_authors(args))
    delete_ids = [cid for cid in candidates if cid not in protected]
    if args.confirm_delete and delete_ids:
        try:
            remote = remote_manifest(args.repo, args.archive_dir, args.archive_ref, manifest)
        except FileNotFoundError as exc:
            print(f"compact: skipping deletion for issue #{args.issue}; remote archive manifest is not available yet ({exc})")
            return 0
        remote_segments = matching_remote_segments(manifest, remote, set(delete_ids))
        remotely_archived = {cid for segment in remote_segments
                             for cid in range(segment["first_comment_id"], segment["last_comment_id"] + 1)}
        delete_ids = [cid for cid in delete_ids if cid in remotely_archived]
    print(f"compact: live={len(comments)} archived_live={len(candidates)} keep={len(candidates)-len(delete_ids)} would_delete={len(delete_ids)}")
    print("compact: keep_rule_counts=" + json.dumps(summary, sort_keys=True))
    if not args.confirm_delete:
        print("compact: dry run; no comments deleted")
        if delete_ids:
            print("compact: delete_ids=" + ",".join(map(str, delete_ids)))
        return 0
    # Reserve headroom under GitHub's shared 5,000/hour REST budget.
    estimated_calls = max(1, (len(comments) + 99) // 100) + 1
    deleted = 0
    for cid in delete_ids:
        live_pages = max(1, (len(comments) + 99) // 100)
        calls_for_delete = 1 + len(manifest["segments"]) + live_pages + 2
        if estimated_calls + calls_for_delete > args.request_budget:
            print(f"compact: request budget reached; deferred={len(delete_ids) - deleted} estimated_calls={estimated_calls} budget={args.request_budget}")
            break
        current = fetch_comments(args.repo, args.issue)
        current_issue = fetch_issue(args.repo, args.issue)
        match = next((item for item in current if item["id"] == cid), None)
        if match is None:
            continue
        if match != live[cid]:
            raise RuntimeError(f"live comment {cid} changed since planning; refusing deletion")
        now_protected, _ = record_info(current, current_issue.get("body") or "", args.tail_min,
                                       configured_agents(args.config), trusted_authors(args))
        if cid in now_protected:
            raise RuntimeError(f"comment {cid} became protected before deletion")
        # Read back the relevant immutable evidence directly before each effect.
        try:
            verify_remote(args.repo, args.archive_dir, manifest, args.archive_ref, {cid})
        except FileNotFoundError as exc:
            print(f"compact: remote archive manifest disappeared; skipping remaining deletions for issue #{args.issue} ({exc})")
            break
        api_request(args.repo, "DELETE", f"/repos/{args.repo}/issues/comments/{cid}", allow_not_found=True)
        estimated_calls += calls_for_delete
        deleted += 1
        if args.delete_delay:
            time.sleep(args.delete_delay)
    print(f"compact: deleted={deleted} estimated_api_calls={estimated_calls}")
    return 0


def status(args: argparse.Namespace) -> int:
    slots = discover_slots(args.repo, args.config, trusted_authors(args))
    active = live_slot(slots)
    print(f"active=slot-{active['slot']} issue=#{active['number']} epoch={active['epoch']}")
    counts = {}
    for slot in ("A", "B"):
        issue = slots[slot]
        count = len(fetch_comments(args.repo, issue["number"]))
        counts[slot] = count
        print(f"slot={slot} issue=#{issue['number']} state={issue['slot_state']} epoch={issue['epoch']} comments={count} switch_at={args.switch_at} compact_above={args.compact_above} standby_max={args.standby_max} hard_cap={HARD_COMMENT_CAP}")
    if any(issue["slot_state"] == "DRAINING" for issue in slots.values()) and os.environ.get("ARCHIVE_DELETE_ENABLED") != "true":
        print("WARNING: deletion is disabled; a DRAINING slot cannot be compacted to STANDBY, so the next rotation will block until deletion is enabled and maintenance drains it")
    if os.environ.get("ARCHIVE_DELETE_ENABLED") != "true":
        remaining = sum(max(0, HARD_COMMENT_CAP - count) for count in counts.values())
        active_remaining = max(0, HARD_COMMENT_CAP - counts[active["slot"]])
        standby = slots["B" if active["slot"] == "A" else "A"]
        standby_remaining = max(0, args.standby_max - counts[standby["slot"]])
        print(f"WARNING: deletion is disabled; finite remaining comment capacity is {remaining} across both slots "
              f"(active cap headroom={active_remaining}, standby headroom to standby-max={standby_remaining}); "
              "rotation will block when the active slot reaches switch-at and standby is not below standby-max")
    return 0


def rotate(args: argparse.Namespace) -> int:
    slots = discover_slots(args.repo, args.config, trusted_authors(args))
    active = live_slot(slots)
    other = slots["B" if active["slot"] == "A" else "A"]
    # An ACTIVE+ACTIVE pair is the recoverable crash point: the higher epoch is
    # already the writer, so finish draining the lower epoch without another flip.
    if active["slot_state"] == "ACTIVE" and other["slot_state"] == "ACTIVE":
        patch_issue_body(args.repo, other, {"COORDINATION_STATE": "DRAINING"})
        print(f"rotate: resumed flip; slot {active['slot']} issue #{active['number']} remains ACTIVE epoch={active['epoch']}")
        return 0
    active_count = len(fetch_comments(args.repo, active["number"]))
    if active_count < args.switch_at:
        print(f"rotate: below switch threshold; active={active_count} switch_at={args.switch_at}")
        return 0
    standby_count = len(fetch_comments(args.repo, other["number"]))
    if other["slot_state"] != "STANDBY":
        remedy = (" Enable ARCHIVE_DELETE_ENABLED after verifying archive read-back, or compact manually." if
                  os.environ.get("ARCHIVE_DELETE_ENABLED") != "true" else " Verify archive read-back and compact the draining slot manually.")
        raise RuntimeError(f"expected inactive slot {other['slot']} to be STANDBY, found {other['slot_state']}; "
                           f"rotation cannot safely flip into a non-empty standby.{remedy}")
    if standby_count > args.standby_max:
        print(f"rotate: standby slot {other['slot']} has {standby_count} comments (> {args.standby_max}); attempting archive/compaction before flip")
        maintenance = argparse.Namespace(**vars(args))
        maintenance.issue = other["number"]
        maintenance.trigger_count = 0
        prepare_online(maintenance)
        compact(maintenance)
        standby_count = len(fetch_comments(args.repo, other["number"]))
        if standby_count > args.standby_max:
            remedy = ("Enable ARCHIVE_DELETE_ENABLED after verifying archive read-back, or compact manually." if
                      os.environ.get("ARCHIVE_DELETE_ENABLED") != "true" else "Verify archive read-back and compact the standby manually.")
            raise RuntimeError(f"rotation refused: standby still has {standby_count} comments (> {args.standby_max}); "
                               f"it will not flip into a non-empty standby. {remedy}")
    # Writer availability is maintained by making the new ACTIVE visible first.
    next_epoch = max(active["epoch"], other["epoch"]) + 1
    patch_issue_body(args.repo, other, {"COORDINATION_STATE": "ACTIVE", "COORDINATION_EPOCH": str(next_epoch)})
    patch_issue_body(args.repo, active, {"COORDINATION_STATE": "DRAINING"})
    print(f"rotate: slot {other['slot']} issue #{other['number']} ACTIVE epoch={next_epoch}; slot {active['slot']} is DRAINING")
    return 0


def maintain_slots(args: argparse.Namespace) -> int:
    """Archive draining slots and eligible active history; re-enable drained slots."""
    slots = discover_slots(args.repo, args.config, trusted_authors(args))
    active = live_slot(slots)
    for issue in slots.values():
        count = len(fetch_comments(args.repo, issue["number"]))
        if (issue["slot_state"] == "DRAINING"
                or (issue["number"] == active["number"] and count > args.compact_above)
                or (issue["slot_state"] == "STANDBY" and count > args.standby_max)):
            maintenance = argparse.Namespace(**vars(args))
            maintenance.archive_dir = args.archive_dir
            maintenance.issue = issue["number"]
            maintenance.trigger_count = 0
            prepare_online(maintenance)
            compact(maintenance)
            count = len(fetch_comments(args.repo, issue["number"]))
            if issue["slot_state"] == "DRAINING" and count <= args.standby_max:
                patch_issue_body(args.repo, issue, {"COORDINATION_STATE": "STANDBY"})
                print(f"maintain: slot {issue['slot']} issue #{issue['number']} is now STANDBY")
    return 0


def prepare_slots(args: argparse.Namespace) -> int:
    """Create archive evidence for any issue eligible for this maintenance pass."""
    slots = discover_slots(args.repo, args.config, trusted_authors(args))
    active = live_slot(slots)
    for issue in slots.values():
        count = len(fetch_comments(args.repo, issue["number"]))
        if (issue["slot_state"] == "DRAINING"
                or (issue["number"] == active["number"] and count > args.compact_above)
                or (issue["slot_state"] == "STANDBY" and count > args.standby_max)):
            maintenance = argparse.Namespace(**vars(args))
            maintenance.archive_dir = args.archive_dir
            maintenance.issue = issue["number"]
            maintenance.trigger_count = 0
            prepare_online(maintenance)
    return 0


def migrate_layout(source_dir: Path, target_dir: Path, issue: int) -> dict[str, Any]:
    """Move a legacy flat archive, rewriting only manifest paths, never segment bytes."""
    manifest_file = source_dir / "manifest.json"
    if not manifest_file.exists():
        raise RuntimeError(f"legacy manifest missing: {manifest_file}")
    manifest = json.loads(manifest_file.read_text(encoding="utf-8"))
    if manifest.get("issue") != issue or not isinstance(manifest.get("segments"), list):
        raise RuntimeError(f"legacy manifest does not describe issue #{issue}")
    target_dir.mkdir(parents=True, exist_ok=True)
    rewritten = dict(manifest)
    rewritten["segments"] = []
    for segment in manifest["segments"]:
        old_rel = Path(segment["path"])
        filename = old_rel.name
        old_file = source_dir / filename
        if sha256(old_file.read_bytes()) != segment["sha256"]:
            raise RuntimeError(f"legacy segment hash mismatch: {old_file}")
        new_file = target_dir / filename
        if new_file.exists():
            raise RuntimeError(f"refusing to overwrite existing segment {new_file}")
        new_file.write_bytes(old_file.read_bytes())
        new_segment = dict(segment)
        new_segment["path"] = (old_rel.parent / target_dir.name / filename).as_posix()
        rewritten["segments"].append(new_segment)
    write_manifest(target_dir, rewritten)
    # Source files are only removed after every copied byte and manifest is validated.
    archived_index(target_dir, load_manifest(target_dir, issue))
    for segment in manifest["segments"]:
        (source_dir / Path(segment["path"]).name).unlink()
    manifest_file.unlink()
    return rewritten


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("prepare", "compact", "status", "rotate", "maintain", "prepare-slots", "migrate-layout", "verify-archive-append"))
    parser.add_argument("--repo")
    parser.add_argument("--issue", type=int)
    parser.add_argument("--archive-dir", type=Path, default=Path("Phase0/archives/coordination"))
    parser.add_argument("--archive-ref", default="main")
    parser.add_argument("--trigger-count", type=int, default=1500)
    parser.add_argument("--switch-at", type=int, default=os.environ.get("COORDINATION_SWITCH_AT", "2000"))
    parser.add_argument("--standby-max", type=int, default=int(os.environ.get("COORDINATION_STANDBY_MAX", "500")))
    parser.add_argument("--compact-above", type=int, default=int(os.environ.get("COORDINATION_COMPACT_ABOVE", "1500")))
    # Must fit below the 500-comment standby target so a fully compacted
    # draining slot can become a standby again.
    parser.add_argument("--tail-min", type=int, default=300)
    parser.add_argument("--segment-size", type=int, default=200)
    parser.add_argument("--delete-delay", type=float, default=0.4)
    parser.add_argument("--request-budget", type=int, default=3000,
                        help="maximum estimated REST calls for this compaction run")
    parser.add_argument("--trusted-authors", default=None,
                        help="comma-separated GitHub logins whose PHASE0 records are authoritative "
                             "(or env COORDINATION_TRUSTED_AUTHORS); REQUIRED for compact --confirm-delete")
    parser.add_argument("--confirm-delete", action="store_true")
    parser.add_argument("--config", type=Path, default=Path("Phase0/05-FLEET-CONFIG.md"))
    parser.add_argument("--input-jsonl", type=Path)
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument("--issue-body", type=Path)
    parser.add_argument("--source-dir", type=Path)
    parser.add_argument("--target-dir", type=Path)
    parser.add_argument("--repo-dir", type=Path, default=Path("."))
    parser.add_argument("--allow-empty", action="store_true", help="allow an archive repo with no existing HEAD")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = make_parser()
    args = parser.parse_args(argv)
    try:
        args.switch_at = switch_at_value(args.switch_at)
        if not 1 <= args.segment_size <= 500:
            raise RuntimeError("segment-size out of range")
        if args.tail_min < 0 or args.trigger_count < 0:
            raise RuntimeError("invalid trigger/tail values")
        if args.request_budget < 1:
            raise RuntimeError("request-budget must be positive")
        if args.mode == "prepare" and args.input_jsonl:
            if not args.out_dir:
                raise RuntimeError("offline prepare requires --out-dir")
            return prepare_offline(args)
        if args.switch_at < 1 or args.standby_max < 0 or args.compact_above < 0:
            raise RuntimeError("invalid slot thresholds")
        if args.switch_at > MAX_SWITCH_AT:
            raise RuntimeError(f"switch-at must be <= {MAX_SWITCH_AT} (hard comment cap is {HARD_COMMENT_CAP})")
        if args.mode == "migrate-layout":
            if not args.source_dir or not args.target_dir or not args.issue:
                raise RuntimeError("migrate-layout requires --source-dir, --target-dir, and --issue")
            migrate_layout(args.source_dir, args.target_dir, args.issue)
            return 0
        if args.mode == "verify-archive-append":
            return verify_archive_append(args)
        if not args.repo:
            raise RuntimeError("online modes require --repo")
        if args.mode in {"status", "rotate", "maintain", "prepare-slots"}:
            if args.mode == "status":
                return status(args)
            if args.mode == "rotate":
                return rotate(args)
            if args.mode == "prepare-slots":
                return prepare_slots(args)
            return maintain_slots(args)
        if not args.issue:
            raise RuntimeError("online modes require --repo and --issue")
        if args.mode == "prepare":
            return prepare_online(args)
        return compact(args)
    except (OSError, ValueError, RuntimeError, KeyError, TypeError) as exc:
        print(f"coordination-archive: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
