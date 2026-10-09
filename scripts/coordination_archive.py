#!/usr/bin/env python3
"""Lossless, fail-closed maintenance for a Phase0 coordination issue."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
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


def record_info(comments: list[dict[str, Any]], issue_body: str, tail_min: int,
                agents: set[str]) -> tuple[set[int], dict[str, int]]:
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
        if body.startswith("PHASE0 |"):
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
    if path.exists() and path.read_bytes() != payload:
        # Manifest is a versioned index and may grow; only segments are immutable.
        path.write_bytes(payload)
    else:
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
    protected, summary = record_info(comments, issue_obj.get("body") or "", args.tail_min, agents)
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
    protected, summary = record_info(comments, issue_body, args.tail_min, agents)
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
    obj, _ = api_request(repo, "GET", f"/repos/{repo}/contents/{urllib.parse.quote(path)}?{query}")
    if not isinstance(obj, dict) or obj.get("encoding") != "base64" or not isinstance(obj.get("content"), str):
        raise RuntimeError(f"remote archive file unavailable or unsupported: {path}@{ref}")
    try:
        encoded = "".join(obj["content"].split()).encode("ascii")
        return base64.b64decode(encoded, validate=True)
    except Exception as exc:
        raise RuntimeError(f"invalid base64 from archive read-back: {path}@{ref}") from exc


def verify_remote(repo: str, archive_dir: Path, manifest: dict[str, Any], ref: str) -> None:
    local_manifest = manifest_path(archive_dir).read_bytes()
    remote_manifest = remote_file_bytes(repo, (archive_dir / "manifest.json").as_posix(), ref)
    if remote_manifest != local_manifest:
        raise RuntimeError("remote manifest is not byte-identical; refusing deletion")
    for segment in manifest["segments"]:
        remote = remote_file_bytes(repo, segment["path"], ref)
        local = local_segment_path(archive_dir, segment["path"]).read_bytes()
        if remote != local or sha256(remote) != segment["sha256"]:
            raise RuntimeError(f"remote segment is not byte-identical or hash mismatched: {segment['path']}")


def compact(args: argparse.Namespace) -> int:
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
                                    configured_agents(args.config))
    delete_ids = [cid for cid in candidates if cid not in protected]
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
        # Revalidate exact manifest and every listed segment directly before each effect.
        verify_remote(args.repo, args.archive_dir, manifest, args.archive_ref)
        current = fetch_comments(args.repo, args.issue)
        current_issue = fetch_issue(args.repo, args.issue)
        match = next((item for item in current if item["id"] == cid), None)
        if match is None:
            continue
        if match != live[cid]:
            raise RuntimeError(f"live comment {cid} changed since planning; refusing deletion")
        now_protected, _ = record_info(current, current_issue.get("body") or "", args.tail_min,
                                       configured_agents(args.config))
        if cid in now_protected:
            raise RuntimeError(f"comment {cid} became protected before deletion")
        api_request(args.repo, "DELETE", f"/repos/{args.repo}/issues/comments/{cid}", allow_not_found=True)
        estimated_calls += calls_for_delete
        deleted += 1
        if args.delete_delay:
            time.sleep(args.delete_delay)
    print(f"compact: deleted={deleted} estimated_api_calls={estimated_calls}")
    return 0


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("prepare", "compact"))
    parser.add_argument("--repo")
    parser.add_argument("--issue", type=int)
    parser.add_argument("--archive-dir", type=Path, default=Path("Phase0/archives/coordination"))
    parser.add_argument("--archive-ref", default="main")
    parser.add_argument("--trigger-count", type=int, default=1500)
    parser.add_argument("--tail-min", type=int, default=750)
    parser.add_argument("--segment-size", type=int, default=200)
    parser.add_argument("--delete-delay", type=float, default=0.4)
    parser.add_argument("--request-budget", type=int, default=3000,
                        help="maximum estimated REST calls for this compaction run")
    parser.add_argument("--confirm-delete", action="store_true")
    parser.add_argument("--config", type=Path, default=Path("Phase0/05-FLEET-CONFIG.md"))
    parser.add_argument("--input-jsonl", type=Path)
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument("--issue-body", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = make_parser()
    args = parser.parse_args(argv)
    try:
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
        if not args.repo or not args.issue:
            raise RuntimeError("online modes require --repo and --issue")
        if args.mode == "prepare":
            return prepare_online(args)
        return compact(args)
    except (OSError, ValueError, RuntimeError, KeyError, TypeError) as exc:
        print(f"coordination-archive: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
