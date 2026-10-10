import hashlib
import json
import tempfile
import unittest
import subprocess
import shutil
import os
from pathlib import Path
from unittest import mock

import scripts.coordination_archive as archive


ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "Phase0/05-FLEET-CONFIG.md"


def comment(cid, body="ordinary", author="tester"):
    return {"id": cid, "body": body, "user": {"login": author},
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z",
            "html_url": f"https://example.invalid/issues/1#issuecomment-{cid}"}


def phase(cid, run, agent="A1", state="INTENT", prev=None):
    suffix = f" | prev={prev}" if prev else ""
    return comment(cid, f"PHASE0 | seq={cid} | run={run} | agent={agent} | state={state}{suffix}")


def slot_issue(number, slot, state, epoch, author="tester"):
    return {"number": number, "state": "open", "title": "[fleet-control] coordination",
            "user": {"login": author},
            "body": f"FLEET_COORDINATION_V1\nCOORDINATION_SLOT={slot}\nCOORDINATION_STATE={state}\nCOORDINATION_EPOCH={epoch}\nAdditional operator text\n"}


class CoordinationArchiveTests(unittest.TestCase):
    def test_offline_segments_are_deterministic_hashed_and_byte_exact(self):
        comments = [comment(100 + i, f"line {i} | unicode=é") for i in range(5)]
        raw = b"".join((json.dumps(x, ensure_ascii=False, separators=(",", ":")) + "\n").encode() for x in comments)
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / "input.jsonl"
            source.write_bytes(raw)
            dirs = [Path(tmp) / "one", Path(tmp) / "two"]
            for out in dirs:
                self.assertEqual(archive.main(["prepare", "--input-jsonl", str(source), "--out-dir", str(out),
                                               "--config", str(CONFIG), "--tail-min", "2", "--segment-size", "2"]), 0)
            manifests = [(out / "manifest.json").read_bytes() for out in dirs]
            self.assertEqual(manifests[0], manifests[1])
            data = json.loads(manifests[0])
            self.assertEqual([s["count"] for s in data["segments"]], [2, 2, 1])
            self.assertEqual(b"".join((dirs[0] / s["path"]).read_bytes() for s in data["segments"]), raw)
            for seg in data["segments"]:
                self.assertEqual(hashlib.sha256((dirs[0] / seg["path"]).read_bytes()).hexdigest(), seg["sha256"])
            self.assertEqual(data["source_sha256"], hashlib.sha256(raw).hexdigest())

    def test_keep_rules_public_format(self):
        records = [phase(1, "run-old", state="INTENT"),
                   phase(2, "run-old", state="OWNED", prev=1),
                   comment(3, "PHASE0 | seq=3 | run=other | agent=A2 | state=RELEASE"),
                   comment(4, "PHASE0 | seq=4 | run=active | agent=A3 | state=INTENT"),
                   phase(5, "active", agent="A3", state="WORKING", prev=4),
                   phase(6, "state", agent="A4", state="RELEASE"),
                   comment(7, "ordinary")]
        kept, summary = archive.record_info(records, "See #coordination issuecomment-3", 1, {"A1", "A2", "A3", "A4", "A5"})
        self.assertTrue({1, 2, 3, 4, 5, 6, 7}.issubset(kept))
        self.assertEqual(summary["tail"], 1)
        self.assertEqual(summary["latest_agent"], 4)
        # A terminal latest record for run-old supersedes its active status.
        records.append(phase(8, "run-old", state="HANDOFF"))
        kept2, _ = archive.record_info(records, "", 1, {"A1", "A2", "A3", "A4", "A5"})
        self.assertNotIn(1, kept2)
        self.assertNotIn(2, kept2)

    def test_latest_state_is_kept_for_free_form_agent_identities(self):
        records = [phase(1, "r1", agent="manual.cleanup.operator", state="HANDOFF"),
                   phase(2, "r2", agent="manual.cleanup.operator", state="HANDOFF"),
                   phase(3, "r3", agent="git.integration.consolidator", state="RELEASE"),
                   comment(4, "ordinary"), comment(5, "ordinary")]
        kept, summary = archive.record_info(records, "", 1, {"A1", "A2", "A3", "A4", "A5"})
        self.assertIn(2, kept)
        self.assertIn(3, kept)
        self.assertNotIn(1, kept)
        self.assertEqual(summary["latest_agent"], 2)

    def test_manifest_round_trip_and_hash_mismatch_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            arch_dir = root / "Phase0/archives/coordination/issue-9"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            seg = arch_dir / "comments-10-10.jsonl"
            seg.write_bytes(raw)
            manifest = {"schema": 1, "issue": 9, "segments": [{"path": "Phase0/archives/coordination/issue-9/comments-10-10.jsonl",
                       "count": 1, "first_comment_id": 10, "last_comment_id": 10, "sha256": archive.sha256(raw)}]}
            archive.write_manifest(arch_dir, manifest)
            loaded = archive.load_manifest(arch_dir, 9)
            self.assertEqual(archive.archived_index(arch_dir, loaded)[10], item)
            seg.write_bytes(raw + b" ")
            with self.assertRaisesRegex(RuntimeError, "hash mismatch"):
                archive.archived_index(arch_dir, loaded)

    def test_dry_run_never_deletes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            arch_dir = root / "Phase0/archives/coordination/issue-12"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            (arch_dir / "comments-10-10.jsonl").write_bytes(raw)
            archive.write_manifest(arch_dir, {"schema": 1, "issue": 12, "segments": [{
                "path": "Phase0/archives/coordination/issue-12/comments-10-10.jsonl", "count": 1,
                "first_comment_id": 10, "last_comment_id": 10, "sha256": archive.sha256(raw)}]})
            args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12",
                                                     "--archive-dir", str(arch_dir), "--tail-min", "0", "--config", str(CONFIG)])
            with mock.patch.object(archive, "fetch_comments", return_value=[item]), \
                 mock.patch.object(archive, "fetch_issue", return_value={"body": ""}), \
                 mock.patch.object(archive, "api_request", side_effect=AssertionError("delete attempted")):
                self.assertEqual(archive.compact(args), 0)

    def test_confirm_delete_refuses_without_identical_remote_readback(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            arch_dir = root / "Phase0/archives/coordination/issue-12"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            (arch_dir / "comments-10-10.jsonl").write_bytes(raw)
            archive.write_manifest(arch_dir, {"schema": 1, "issue": 12, "segments": [{
                "path": "Phase0/archives/coordination/issue-12/comments-10-10.jsonl", "count": 1,
                "first_comment_id": 10, "last_comment_id": 10, "sha256": archive.sha256(raw)}]})
            args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12", "--confirm-delete", "--trusted-authors", "tester",
                                                     "--archive-dir", str(arch_dir), "--tail-min", "0", "--config", str(CONFIG)])
            with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true", "COORDINATION_TRUSTED_AUTHORS": "tester"}), \
                 mock.patch.object(archive, "fetch_comments", return_value=[item]), \
                 mock.patch.object(archive, "fetch_issue", return_value={"body": ""}), \
                 mock.patch.object(archive, "remote_manifest", return_value={"segments": json.loads((arch_dir / "manifest.json").read_bytes())["segments"]}), \
                 mock.patch.object(archive, "verify_remote", side_effect=RuntimeError("remote manifest is not byte-identical")), \
                 mock.patch.object(archive, "api_request") as request:
                with self.assertRaisesRegex(RuntimeError, "byte-identical"):
                    archive.compact(args)
                request.assert_not_called()

    def test_confirm_delete_requires_trusted_authors(self):
        args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12", "--confirm-delete"])
        with mock.patch.dict("os.environ", {"ARCHIVE_DELETE_ENABLED": "true"}, clear=False) as env:
            env.pop("COORDINATION_TRUSTED_AUTHORS", None)
            with mock.patch.object(archive, "api_request") as request:
                with self.assertRaisesRegex(SystemExit, "trusted-authors"):
                    archive.compact(args)
                request.assert_not_called()

    def test_forged_records_from_untrusted_authors_are_not_credited(self):
        records = [phase(1, "forged", agent="evil", state="OWNED"),
                   comment(2, "PHASE0 | seq=2 | run=forged | agent=evil | state=WORKING", author="stranger"),
                   phase(3, "real", agent="op", state="HANDOFF"),
                   comment(4), comment(5)]
        records[0]["user"]["login"] = "stranger"
        kept, summary = archive.record_info(records, "", 1, set(), {"tester"})
        self.assertNotIn(1, kept)
        self.assertNotIn(2, kept)
        self.assertIn(3, kept)
        self.assertEqual(summary["latest_agent"], 1)
        # Without an allowlist (offline dry run only) the forged active run would be protected: the gap the allowlist closes.
        kept_all, _ = archive.record_info(records, "", 1, set(), None)
        self.assertIn(1, kept_all)

    def test_pagination_gap_fails_closed(self):
        first = [comment(i + 1) for i in range(100)]
        with mock.patch.object(archive, "api_request", side_effect=[(first, {"Link": '<https://api.example/page=2>; rel="next"'}),
                                                                     ([comment(i) for i in range(101, 200)], {"Link": '<https://api.example/page=3>; rel="next"'})]):
            with self.assertRaisesRegex(RuntimeError, "short page.*next"):
                archive.fetch_comments("o/r", 1)

    def test_offline_mode_does_not_call_network(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "in.jsonl"
            source.write_bytes(archive.encode_api_records([comment(1)]))
            with mock.patch.object(archive, "api_request", side_effect=AssertionError("network used")):
                self.assertEqual(archive.main(["prepare", "--input-jsonl", str(source), "--out-dir", str(root / "out"),
                                               "--config", str(CONFIG), "--tail-min", "0"]), 0)

    def test_discovery_uses_highest_epoch_and_ignores_untrusted_issue(self):
        issues = [slot_issue(11, "A", "ACTIVE", 4), slot_issue(12, "B", "ACTIVE", 5),
                  slot_issue(99, "A", "ACTIVE", 900, author="stranger")]
        with mock.patch.object(archive, "fetch_open_issues", return_value=issues):
            slots = archive.discover_slots("o/r", CONFIG, {"tester"})
        self.assertEqual(archive.live_slot(slots)["number"], 12)

    def test_rotate_patch_order_and_crash_resume(self):
        issues = [slot_issue(11, "A", "ACTIVE", 1), slot_issue(12, "B", "STANDBY", 0)]
        writes = []
        def request(repo, method, path, body=None, **kwargs):
            if method == "PATCH":
                writes.append((path, body["body"]))
                issue_num = int(path.rsplit("/", 1)[-1])
                next(x for x in issues if x["number"] == issue_num)["body"] = body["body"]
                # A flip must always expose at least one ACTIVE issue after each body edit.
                active = [x for x in issues if archive.slot_fields(x["body"]).get("COORDINATION_STATE") == "ACTIVE"]
                self.assertTrue(active)
            return None, {}
        args = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester", "--switch-at", "100"])
        # Keep synthetic counts above the validated minimum threshold.
        counts_for_test = [comment(i) for i in range(1, 102)]
        with mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, n: counts_for_test), \
             mock.patch.object(archive, "api_request", side_effect=request):
            self.assertEqual(archive.rotate(args), 0)
            self.assertEqual([x[0].rsplit("/", 1)[-1] for x in writes], ["12", "11"])
            self.assertEqual(archive.slot_fields(issues[1]["body"])["COORDINATION_EPOCH"], "2")
            # Simulate process death after the first PATCH: two ACTIVE, different epochs.
            issues[0]["body"] = archive.replace_body_fields(issues[0]["body"], {"COORDINATION_STATE": "ACTIVE"})
            issues[1]["body"] = archive.replace_body_fields(issues[1]["body"], {"COORDINATION_STATE": "ACTIVE"})
            writes.clear()
            self.assertEqual(archive.rotate(args), 0)
            self.assertEqual(len(writes), 1)
            self.assertIn("COORDINATION_STATE=DRAINING", writes[0][1])

    def test_rotate_refuses_full_standby_and_attempts_compaction(self):
        issues = [slot_issue(11, "A", "ACTIVE", 1), slot_issue(12, "B", "STANDBY", 0)]
        args = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester", "--switch-at", "100", "--standby-max", "1"])
        counts = {11: [comment(i) for i in range(100)], 12: [comment(201), comment(202)]}
        with mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, n: counts[n]), \
             mock.patch.object(archive, "prepare_online") as prep, \
             mock.patch.object(archive, "compact") as compact, \
             mock.patch.object(archive, "api_request") as request:
            with self.assertRaisesRegex(RuntimeError, "Enable ARCHIVE_DELETE_ENABLED after verifying archive read-back, or compact manually"):
                archive.rotate(args)
            prep.assert_called_once()
            compact.assert_called_once()
            request.assert_not_called()

    def test_switch_at_over_2400_is_rejected_before_api(self):
        self.assertEqual(archive.main(["rotate", "--repo", "o/r", "--switch-at", "2401"]), 2)

    def test_switch_at_environment_must_be_integer_and_in_range(self):
        for raw in ("nope", "0", "99", "2401"):
            with self.subTest(raw=raw), mock.patch.dict(os.environ, {"COORDINATION_SWITCH_AT": raw}):
                if raw == "nope":
                    with self.assertRaises(SystemExit):
                        archive.main(["status", "--repo", "o/r"])
                else:
                    self.assertEqual(archive.main(["status", "--repo", "o/r"]), 2)

    def test_body_field_replacement_preserves_all_non_machine_bytes(self):
        body = ("FLEET_COORDINATION_V1\r\nCOORDINATION_SLOT=A\r\n"
                "operator\u2028paragraph\x0ccontinued\r\nCOORDINATION_STATE=ACTIVE\r\n"
                "COORDINATION_EPOCH=1\r\ntrailing\u2028text")
        changed = archive.replace_body_fields(body, {"COORDINATION_STATE": "DRAINING"})
        self.assertEqual(changed, body.replace("COORDINATION_STATE=ACTIVE", "COORDINATION_STATE=DRAINING"))

    def test_body_field_replacement_refuses_ambiguous_lone_cr_machine_line(self):
        body = "FLEET_COORDINATION_V1\nCOORDINATION_STATE=ACTIVE\rCOORDINATION_EPOCH=1\r"
        with self.assertRaisesRegex(RuntimeError, "ambiguous lone-CR"):
            archive.replace_body_fields(body, {"COORDINATION_STATE": "DRAINING"})

    def test_trusted_author_empty_configuration_is_rejected(self):
        args = archive.make_parser().parse_args(["status", "--repo", "o/r", "--trusted-authors", ""])
        with self.assertRaisesRegex(RuntimeError, "non-empty comma-separated"):
            archive.trusted_authors(args)
        with mock.patch.object(archive, "fetch_open_issues", side_effect=AssertionError("discovery should reject first")):
            with self.assertRaisesRegex(RuntimeError, "non-empty comma-separated"):
                archive.discover_slots("o/r", CONFIG, archive.trusted_authors(args))

    def test_trusted_authors_normalize_spaces_and_reject_empty_entries(self):
        args = archive.make_parser().parse_args(["status", "--repo", "o/r", "--trusted-authors", " Geromet , ALICE "])
        self.assertEqual(archive.trusted_authors(args), {"geromet", "alice"})
        for raw in (",", "geromet,", ",alice", "geromet,,alice"):
            with self.subTest(raw=raw):
                args = archive.make_parser().parse_args(["status", "--repo", "o/r", "--trusted-authors", raw])
                with self.assertRaisesRegex(RuntimeError, "non-empty comma-separated"):
                    archive.trusted_authors(args)

    def test_issue_archive_dir_rejects_mismatched_issue_scoped_path(self):
        with self.assertRaisesRegex(RuntimeError, "different issue"):
            archive.issue_archive_dir(Path("archives/issue-11"), 12)

    def test_script_enforces_delete_environment_gate(self):
        args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12",
                                                 "--confirm-delete", "--trusted-authors", "tester"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "false"}), \
             mock.patch.object(archive, "fetch_comments", side_effect=AssertionError("must refuse before API")):
            with self.assertRaisesRegex(SystemExit, "ARCHIVE_DELETE_ENABLED=true"):
                archive.compact(args)

    def test_append_verifier_rejects_staged_segment_deletion(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            segment = repo / "Phase0/archives/coordination/issue-5/comments-1-1.jsonl"
            segment.parent.mkdir(parents=True)
            raw = archive.encode_api_records([comment(1)])
            segment.write_bytes(raw)
            rel = "Phase0/archives/coordination/issue-5/comments-1-1.jsonl"
            manifest = {"schema": 1, "issue": 5, "segments": [{"path": rel, "count": 1,
                        "first_comment_id": 1, "last_comment_id": 1, "sha256": archive.sha256(raw)}]}
            (repo / "Phase0/archives/coordination/issue-5/manifest.json").write_text(json.dumps(manifest))
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            subprocess.run(["git", "-C", str(repo), "config", "user.email", "test@example.invalid"], check=True)
            subprocess.run(["git", "-C", str(repo), "config", "user.name", "Test"], check=True)
            subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
            subprocess.run(["git", "-C", str(repo), "commit", "-qm", "baseline"], check=True)
            subprocess.run(["git", "-C", str(repo), "rm", "--cached", "-q", rel], check=True)
            self.assertEqual(archive.main(["verify-archive-append", "--repo-dir", str(repo),
                                           "--archive-dir", "Phase0/archives/coordination"]), 2)

    def test_append_verifier_allows_new_segment_and_manifest_append(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            issue_dir = repo / "Phase0/archives/coordination/issue-5"
            issue_dir.mkdir(parents=True)
            first = archive.encode_api_records([comment(1)])
            second = archive.encode_api_records([comment(2)])
            (issue_dir / "comments-1-1.jsonl").write_bytes(first)
            first_entry = {"path": "Phase0/archives/coordination/issue-5/comments-1-1.jsonl", "count": 1,
                           "first_comment_id": 1, "last_comment_id": 1, "sha256": archive.sha256(first)}
            manifest_path = issue_dir / "manifest.json"
            manifest_path.write_text(json.dumps({"schema": 1, "issue": 5, "segments": [first_entry]}))
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            subprocess.run(["git", "-C", str(repo), "config", "user.email", "test@example.invalid"], check=True)
            subprocess.run(["git", "-C", str(repo), "config", "user.name", "Test"], check=True)
            subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
            subprocess.run(["git", "-C", str(repo), "commit", "-qm", "baseline"], check=True)
            (issue_dir / "comments-2-2.jsonl").write_bytes(second)
            second_entry = {"path": "Phase0/archives/coordination/issue-5/comments-2-2.jsonl", "count": 1,
                            "first_comment_id": 2, "last_comment_id": 2, "sha256": archive.sha256(second)}
            manifest_path.write_text(json.dumps({"schema": 1, "issue": 5, "segments": [first_entry, second_entry]}))
            subprocess.run(["git", "-C", str(repo), "add", "Phase0/archives/coordination"], check=True)
            self.assertEqual(archive.main(["verify-archive-append", "--repo-dir", str(repo),
                                           "--archive-dir", "Phase0/archives/coordination"]), 0)

    def test_compact_uses_remote_segments_without_whole_manifest_equality(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            arch_dir = root / "Phase0/archives/coordination/issue-12"
            arch_dir.mkdir(parents=True)
            old_items = [comment(10), comment(11)]
            new_items = [comment(12)]
            old_raw = archive.encode_api_records(old_items)
            new_raw = archive.encode_api_records(new_items)
            (arch_dir / "comments-10-11.jsonl").write_bytes(old_raw)
            (arch_dir / "comments-12-12.jsonl").write_bytes(new_raw)
            old_seg = {"path": "Phase0/archives/coordination/issue-12/comments-10-11.jsonl", "count": 2,
                       "first_comment_id": 10, "last_comment_id": 11, "sha256": archive.sha256(old_raw)}
            new_seg = {"path": "Phase0/archives/coordination/issue-12/comments-12-12.jsonl", "count": 1,
                       "first_comment_id": 12, "last_comment_id": 12, "sha256": archive.sha256(new_raw)}
            local_manifest = {"schema": 1, "issue": 12, "segments": [old_seg, new_seg]}
            remote_manifest = {"schema": 1, "issue": 12, "segments": [old_seg]}
            archive.write_manifest(arch_dir, local_manifest)
            args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12", "--confirm-delete",
                    "--trusted-authors", "tester", "--archive-dir", str(arch_dir), "--tail-min", "0", "--config", str(CONFIG)])
            deleted = []
            def remote_bytes(_repo, path, _ref):
                if path.endswith("manifest.json"):
                    return json.dumps(remote_manifest).encode()
                return old_raw
            def request(_repo, method, path, *a, **kw):
                if method == "DELETE":
                    deleted.append(int(path.rsplit("/", 1)[-1]))
                return None, {}
            with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true", "COORDINATION_TRUSTED_AUTHORS": "tester"}), \
                 mock.patch.object(archive, "fetch_comments", return_value=old_items + new_items), \
                 mock.patch.object(archive, "fetch_issue", return_value={"body": ""}), \
                 mock.patch.object(archive, "remote_file_bytes", side_effect=remote_bytes), \
                 mock.patch.object(archive, "api_request", side_effect=request):
                self.assertEqual(archive.compact(args), 0)
            self.assertEqual(deleted, [10, 11])

    def test_rotation_epoch_uses_maximum_existing_epoch(self):
        issues = [slot_issue(11, "A", "ACTIVE", 5), slot_issue(12, "B", "STANDBY", 100)]
        args = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester", "--switch-at", "100"])
        writes = []
        def request(_repo, method, path, body=None, **kwargs):
            if method == "PATCH":
                writes.append((path, body["body"]))
                target = next(x for x in issues if x["number"] == int(path.rsplit("/", 1)[-1]))
                target["body"] = body["body"]
            return None, {}
        with mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, _n: [comment(i) for i in range(100)]), \
             mock.patch.object(archive, "api_request", side_effect=request):
            archive.rotate(args)
        self.assertEqual(archive.slot_fields(issues[1]["body"])["COORDINATION_EPOCH"], "101")

    def test_rotation_explains_deletion_remedy_for_draining_standby(self):
        issues = [slot_issue(11, "A", "ACTIVE", 1), slot_issue(12, "B", "DRAINING", 0)]
        args = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester", "--switch-at", "100"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "false"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, _n: [comment(i) for i in range(100)]):
            with self.assertRaisesRegex(RuntimeError, "Enable ARCHIVE_DELETE_ENABLED after verifying archive read-back, or compact manually"):
                archive.rotate(args)

    def test_status_warns_of_finite_capacity_when_deletion_is_disabled(self):
        issues = [slot_issue(11, "A", "ACTIVE", 1), slot_issue(12, "B", "STANDBY", 0)]
        args = archive.make_parser().parse_args(["status", "--repo", "o/r", "--trusted-authors", "tester"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "false"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, n: [comment(i) for i in range(100 if n == 11 else 20)]), \
             mock.patch("builtins.print") as output:
            self.assertEqual(archive.status(args), 0)
        rendered = " ".join(str(call.args[0]) for call in output.call_args_list)
        self.assertIn("finite remaining comment capacity is 4880", rendered)
        self.assertIn("rotation will block when the active slot reaches switch-at and standby is not below standby-max", rendered)

    def test_status_explains_draining_slot_blocks_second_flip_when_deletion_is_off(self):
        issues = [slot_issue(11, "A", "DRAINING", 1), slot_issue(12, "B", "ACTIVE", 2)]
        args = archive.make_parser().parse_args(["status", "--repo", "o/r", "--trusted-authors", "tester"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "false"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", return_value=[]), \
             mock.patch("builtins.print") as output:
            self.assertEqual(archive.status(args), 0)
        rendered = " ".join(str(call.args[0]) for call in output.call_args_list)
        self.assertIn("a DRAINING slot cannot be compacted to STANDBY", rendered)
        self.assertIn("next rotation will block", rendered)

    def test_status_warns_of_protected_capacity_even_with_delete_enabled(self):
        # Each one-shot manual actor's latest terminal record also protects
        # its OWNED/INTENT predecessors. Fully archived bytes are not enough
        # to reclaim this live slot to standby size.
        records = []
        for index in range(8):
            first = 3 * index + 1
            run = f"manual-run-{index}"
            agent = f"manual.v8.gpt6.{index}"
            records.extend([
                phase(first, run, agent=agent, state="INTENT"),
                phase(first + 1, run, agent=agent, state="OWNED", prev=first),
                phase(first + 2, run, agent=agent, state="RELEASE", prev=first + 1),
            ])
        issues = [slot_issue(11, "A", "ACTIVE", 1),
                  slot_issue(12, "B", "STANDBY", 0)]
        args = archive.make_parser().parse_args([
            "status", "--repo", "o/r", "--trusted-authors", "tester",
            "--tail-min", "1", "--standby-max", "5"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments",
                               side_effect=lambda _r, n: records if n == 11 else []), \
             mock.patch("builtins.print") as output:
            self.assertEqual(archive.status(args), 0)
        rendered = " ".join(str(call.args[0]) for call in output.call_args_list)
        self.assertIn("slot=A minimum_retained=24 maximum_retention_eligible=0", rendered)
        self.assertIn("cannot reach standby_max=5", rendered)
        self.assertIn("not permission to delete", rendered)

    def test_status_does_not_warn_when_retention_alone_allows_headroom(self):
        # Reusing one persistent actor identity leaves only its latest
        # completed run's three linked records protected by this rule.
        records = []
        for index in range(8):
            first = 3 * index + 1
            run = f"stable-run-{index}"
            records.extend([
                phase(first, run, agent="manual.stable", state="INTENT"),
                phase(first + 1, run, agent="manual.stable", state="OWNED", prev=first),
                phase(first + 2, run, agent="manual.stable", state="RELEASE", prev=first + 1),
            ])
        issues = [slot_issue(11, "A", "ACTIVE", 1),
                  slot_issue(12, "B", "STANDBY", 0)]
        args = archive.make_parser().parse_args([
            "status", "--repo", "o/r", "--trusted-authors", "tester",
            "--tail-min", "1", "--standby-max", "5"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments",
                               side_effect=lambda _r, n: records if n == 11 else []), \
             mock.patch("builtins.print") as output:
            self.assertEqual(archive.status(args), 0)
        rendered = " ".join(str(call.args[0]) for call in output.call_args_list)
        self.assertIn("slot=A minimum_retained=3 maximum_retention_eligible=21", rendered)
        self.assertNotIn("cannot reach standby_max=5", rendered)
        self.assertIn("not permission to delete", rendered)

    def test_per_issue_archives_are_isolated(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            a, b = root / "coordination/issue-11", root / "coordination/issue-12"
            for issue, directory in ((11, a), (12, b)):
                item = comment(issue)
                raw = archive.encode_api_records([item])
                archive.append_segments(directory, {"schema": 1, "issue": issue, "segments": []}, [item], [raw], 10)
            self.assertEqual(json.loads((a / "manifest.json").read_text())["issue"], 11)
            self.assertEqual(json.loads((b / "manifest.json").read_text())["issue"], 12)
            self.assertTrue((a / "comments-11-11.jsonl").exists())
            self.assertTrue((b / "comments-12-12.jsonl").exists())

    def test_layout_migration_preserves_segment_bytes_and_hash(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "Phase0/archives/coordination"
            target = source / "issue-52"
            source.mkdir(parents=True)
            item = comment(81, "immutable")
            raw = archive.encode_api_records([item])
            (source / "comments-81-81.jsonl").write_bytes(raw)
            archive.write_manifest(source, {"schema": 1, "issue": 52, "segments": [{
                "path": "Phase0/archives/coordination/comments-81-81.jsonl", "count": 1,
                "first_comment_id": 81, "last_comment_id": 81, "sha256": archive.sha256(raw)}]})
            old_hash = archive.sha256((source / "comments-81-81.jsonl").read_bytes())
            migrated = archive.migrate_layout(source, target, 52)
            self.assertEqual(migrated["segments"][0]["sha256"], old_hash)
            self.assertEqual((target / "comments-81-81.jsonl").read_bytes(), raw)
            self.assertFalse((source / "comments-81-81.jsonl").exists())
            self.assertEqual(migrated["segments"][0]["path"], "Phase0/archives/coordination/issue-52/comments-81-81.jsonl")

    def test_workflow_yaml_and_trigger_safety(self):
        workflow = ROOT / ".github/workflows/coordination-slots.yml"
        source = workflow.read_text()
        self.assertIn("issue_comment:", source)
        self.assertIn("types: [created]", source)
        self.assertGreaterEqual(source.count("github.event.issue.comments >= 1000"), 3)
        self.assertNotIn("fromJSON(vars.COORDINATION_SWITCH_AT", source)
        self.assertIn("compact:\n    needs: archive", source)
        self.assertIn("rotate:\n    needs: [archive, compact]", source)
        self.assertIn("!cancelled() && needs.archive.result == 'success'", source)
        self.assertIn("status:\n    if: github.event_name == 'workflow_dispatch'", source)
        runbook = (ROOT / "Phase0/160-COORDINATION-SLOTS.md").read_text()
        ordered_setup = [
            "Merge this feature PR into `phase0/public-v0` first",
            "Merge the single workflow file",
            "Create slot B as one open issue",
            "Run `status` with the workflow's `workflow_dispatch` button",
            "Only after reading successful status output",
        ]
        self.assertEqual(sorted(ordered_setup, key=runbook.index), ordered_setup)
        self.assertIn("comma-separated list of GitHub logins, with no spaces", runbook)
        self.assertGreaterEqual(source.count("github.event.issue.pull_request == null"), 3)
        self.assertGreaterEqual(source.count("github.event.comment.user.login"), 3)
        self.assertGreaterEqual(source.count("contains(format(',{0},', vars.COORDINATION_TRUSTED_AUTHORS || 'geromet')"), 3)
        self.assertNotIn("continue-on-error: true", source)
        self.assertNotIn("persist-credentials: true", source)
        self.assertIn("verify-archive-append", source)
        self.assertNotIn("cp -a archive-state/Phase0/archives/coordination", source)
        self.assertNotIn("cp -a Phase0/archives/coordination/. archive-state/Phase0/archives/coordination", source)
        in_run = False
        indent = 0
        for line in source.splitlines():
            if line.strip() == "run: |":
                in_run = True
                indent = len(line) - len(line.lstrip())
                continue
            if in_run:
                current = len(line) - len(line.lstrip())
                if line.strip() and current <= indent:
                    in_run = False
                elif "${{" in line:
                    self.fail("workflow expression appears inside a shell run block")
        if shutil.which("ruby"):
            subprocess.run(["ruby", "-e", "require 'yaml'; YAML.parse_file(ARGV.fetch(0))", str(workflow)], check=True)

    def test_first_scheduled_prepare_without_two_slots_fails_closed_without_writes(self):
        only_168 = [{"number": 168, "state": "open", "title": "[fleet-control] coordination",
                     "user": {"login": "tester"}, "body": "Existing notes without slot metadata"}]
        args = archive.make_parser().parse_args(["prepare-slots", "--repo", "o/r", "--trusted-authors", "tester", "--config", str(CONFIG)])
        writes = []
        def api(_repo, method, path, body=None, **kwargs):
            if method in {"PATCH", "DELETE", "POST", "PUT"}:
                writes.append((method, path, body))
            return None, {}
        with mock.patch.object(archive, "fetch_open_issues", return_value=only_168), \
             mock.patch.object(archive, "api_request", side_effect=api), \
             mock.patch.object(archive, "fetch_comments", side_effect=AssertionError("must fail before reading/deleting comments")):
            with self.assertRaisesRegex(RuntimeError, "expected trusted open coordination slots A and B; found none"):
                archive.prepare_slots(args)
        self.assertEqual(writes, [])

    def test_compact_skips_issue_without_remote_manifest_and_does_not_delete(self):
        with tempfile.TemporaryDirectory() as tmp:
            arch_dir = Path(tmp) / "Phase0/archives/coordination/issue-12"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            (arch_dir / "comments-10-10.jsonl").write_bytes(raw)
            archive.write_manifest(arch_dir, {"schema": 1, "issue": 12, "segments": [{
                "path": "Phase0/archives/coordination/issue-12/comments-10-10.jsonl", "count": 1,
                "first_comment_id": 10, "last_comment_id": 10, "sha256": archive.sha256(raw)}]})
            args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12", "--confirm-delete",
                    "--trusted-authors", "tester", "--archive-dir", str(arch_dir), "--tail-min", "0", "--config", str(CONFIG)])
            with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true", "COORDINATION_TRUSTED_AUTHORS": "tester"}), \
                 mock.patch.object(archive, "fetch_comments", return_value=[item]), \
                 mock.patch.object(archive, "fetch_issue", return_value={"body": ""}), \
                 mock.patch.object(archive, "remote_manifest", side_effect=FileNotFoundError("manifest missing")), \
                 mock.patch.object(archive, "api_request") as request, \
                 mock.patch("builtins.print") as output:
                self.assertEqual(archive.compact(args), 0)
                request.assert_not_called()
            self.assertIn("skipping deletion for issue #12", " ".join(str(c.args[0]) for c in output.call_args_list))

    def test_full_simulated_two_slot_lifecycle(self):
        issues = [slot_issue(11, "A", "ACTIVE", 1), slot_issue(12, "B", "STANDBY", 0)]
        counts = {11: 2000, 12: 20}
        observed_epochs = []
        def comments(_repo, number):
            return [comment(i + 1) for i in range(counts[number])]
        def api(repo, method, path, body=None, **kwargs):
            if method == "GET" and path.endswith("/issues?state=open&per_page=100&page=1"):
                return issues, {}
            if method == "PATCH":
                number = int(path.rsplit("/", 1)[-1])
                item = next(x for x in issues if x["number"] == number)
                item["body"] = body["body"]
                states = [archive.slot_fields(x["body"]) for x in issues]
                actives = [x for x in states if x["COORDINATION_STATE"] == "ACTIVE"]
                self.assertTrue(actives, "a PATCH created a no-ACTIVE interval")
                observed_epochs.append(max(int(x["COORDINATION_EPOCH"]) for x in actives))
                return item, {}
            raise AssertionError(f"unexpected API call: {method} {path}")
        args = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester", "--switch-at", "2000", "--standby-max", "500"])
        with mock.patch.object(archive, "api_request", side_effect=api), \
             mock.patch.object(archive, "fetch_comments", side_effect=comments), \
             mock.patch.object(archive, "prepare_online", return_value=0), \
             mock.patch.object(archive, "compact", side_effect=lambda a: counts.__setitem__(a.issue, 400 if a.issue == 11 else 300) or 0):
            archive.rotate(args)
            self.assertEqual(archive.live_slot(archive.discover_slots("o/r", CONFIG, {"tester"}))["number"], 12)
            maintain = archive.make_parser().parse_args(["maintain", "--repo", "o/r", "--trusted-authors", "tester"])
            archive.maintain_slots(maintain)
            self.assertEqual(archive.slot_fields(issues[0]["body"])["COORDINATION_STATE"], "STANDBY")
            counts[12] = 2000
            archive.rotate(args)
            self.assertEqual(archive.live_slot(archive.discover_slots("o/r", CONFIG, {"tester"}))["number"], 11)
            archive.maintain_slots(maintain)
            self.assertEqual(archive.slot_fields(issues[1]["body"])["COORDINATION_STATE"], "STANDBY")
            epochs = [int(archive.slot_fields(x["body"])["COORDINATION_EPOCH"]) for x in issues]
            self.assertEqual(max(epochs), 3)
            self.assertTrue(observed_epochs)
            self.assertEqual(observed_epochs, sorted(observed_epochs))
            active_epoch_advances = sorted(set(observed_epochs))
            self.assertEqual(active_epoch_advances, [2, 3])
            self.assertTrue(all(left < right for left, right in zip(active_epoch_advances, active_epoch_advances[1:])))

    def test_deletion_enabled_maintenance_promotes_draining_before_next_flip(self):
        issues = [slot_issue(11, "A", "DRAINING", 1), slot_issue(12, "B", "ACTIVE", 2)]
        counts = {11: 600, 12: 2000}
        events = []
        def api(_repo, method, path, body=None, **kwargs):
            if method == "PATCH":
                number = int(path.rsplit("/", 1)[-1])
                item = next(x for x in issues if x["number"] == number)
                item["body"] = body["body"]
                events.append(("patch", number, archive.slot_fields(item["body"])["COORDINATION_STATE"]))
                return item, {}
            raise AssertionError(f"unexpected API call: {method} {path}")
        def compact(issue_args):
            self.assertTrue(issue_args.confirm_delete)
            events.append(("compact", issue_args.issue))
            counts[issue_args.issue] = 300
            return 0
        maintain = archive.make_parser().parse_args(["maintain", "--repo", "o/r", "--trusted-authors", "tester",
                    "--compact-above", "2400", "--standby-max", "500", "--confirm-delete"])
        rotate = archive.make_parser().parse_args(["rotate", "--repo", "o/r", "--trusted-authors", "tester",
                    "--switch-at", "2000", "--standby-max", "500", "--confirm-delete"])
        with mock.patch.dict(os.environ, {"ARCHIVE_DELETE_ENABLED": "true"}), \
             mock.patch.object(archive, "fetch_open_issues", return_value=issues), \
             mock.patch.object(archive, "fetch_comments", side_effect=lambda _r, n: [comment(i) for i in range(counts[n])]), \
             mock.patch.object(archive, "prepare_online", return_value=0), \
             mock.patch.object(archive, "compact", side_effect=compact), \
             mock.patch.object(archive, "api_request", side_effect=api):
            archive.maintain_slots(maintain)
            self.assertEqual(archive.slot_fields(issues[0]["body"])["COORDINATION_STATE"], "STANDBY")
            archive.rotate(rotate)
        self.assertLess(events.index(("compact", 11)), next(i for i, event in enumerate(events) if event[0] == "patch" and event[1] == 12))
        self.assertEqual(archive.slot_fields(issues[0]["body"])["COORDINATION_STATE"], "ACTIVE")
        self.assertEqual(archive.slot_fields(issues[1]["body"])["COORDINATION_STATE"], "DRAINING")


if __name__ == "__main__":
    unittest.main()
