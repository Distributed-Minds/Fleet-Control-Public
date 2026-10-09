import hashlib
import json
import tempfile
import unittest
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
            arch_dir = root / "Phase0/archives/coordination"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            seg = arch_dir / "comments-10-10.jsonl"
            seg.write_bytes(raw)
            manifest = {"schema": 1, "issue": 9, "segments": [{"path": "Phase0/archives/coordination/comments-10-10.jsonl",
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
            arch_dir = root / "Phase0/archives/coordination"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            (arch_dir / "comments-10-10.jsonl").write_bytes(raw)
            archive.write_manifest(arch_dir, {"schema": 1, "issue": 12, "segments": [{
                "path": "Phase0/archives/coordination/comments-10-10.jsonl", "count": 1,
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
            arch_dir = root / "Phase0/archives/coordination"
            arch_dir.mkdir(parents=True)
            item = comment(10)
            raw = archive.encode_api_records([item])
            (arch_dir / "comments-10-10.jsonl").write_bytes(raw)
            archive.write_manifest(arch_dir, {"schema": 1, "issue": 12, "segments": [{
                "path": "Phase0/archives/coordination/comments-10-10.jsonl", "count": 1,
                "first_comment_id": 10, "last_comment_id": 10, "sha256": archive.sha256(raw)}]})
            args = archive.make_parser().parse_args(["compact", "--repo", "o/r", "--issue", "12", "--confirm-delete",
                                                     "--archive-dir", str(arch_dir), "--tail-min", "0", "--config", str(CONFIG)])
            with mock.patch.object(archive, "fetch_comments", return_value=[item]), \
                 mock.patch.object(archive, "fetch_issue", return_value={"body": ""}), \
                 mock.patch.object(archive, "verify_remote", side_effect=RuntimeError("remote manifest is not byte-identical")), \
                 mock.patch.object(archive, "api_request") as request:
                with self.assertRaisesRegex(RuntimeError, "byte-identical"):
                    archive.compact(args)
                request.assert_not_called()

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


if __name__ == "__main__":
    unittest.main()
