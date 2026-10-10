"""Offline deterministic preflight tests; no GitHub credentials or network needed."""
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import check_coordination_trust_preflight as preflight
WORKFLOW = "jobs:\n" + "".join(f"  {job}:\n    if: {expr}\n" for job, expr in preflight.EXPECTED_JOB_IF.items())
CONFIG = (ROOT / "Phase0/05-FLEET-CONFIG.md").read_text()


def issue(number, slot, state, epoch, creator="geromet"):
    return {"number": number, "state": "open", "title": "[fleet-control] coordination",
            "user": {"login": creator}, "body": (f"FLEET_COORDINATION_V1\nCOORDINATION_SLOT={slot}\n"
            f"COORDINATION_STATE={state}\nCOORDINATION_EPOCH={epoch}\n")}


class TrustPreflightTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.policy = self.root / "config.md"
        self.workflow = self.root / "workflow.yml"
        self.policy.write_text(CONFIG)
        self.workflow.write_text(WORKFLOW)
        self.slots = self.root / "slots.json"
        self.slots.write_text(json.dumps([issue(168, "A", "DRAINING", 1), issue(186, "B", "ACTIVE", 2)]))

    def check(self, actions="geromet", runtime=None, slots=False):
        with mock.patch.dict(os.environ, {}, clear=True):
            return preflight.preflight(self.policy, self.workflow, actions, runtime,
                                       self.slots if slots else None)

    def test_valid_default_fallback_and_slot_epoch(self):
        value = self.check(actions=None, slots=True)
        self.assertEqual(value["status"], "TRUST_AND_OFFLINE_SLOTS_CONSISTENT")
        self.assertEqual(value["slots"], "VALID_ACTIVE_B_EPOCH_2")

    def test_consistent_config_does_not_certify_installation(self):
        self.assertEqual(self.check()["status"], "TRUST_CONFIG_CONSISTENT_ONLY")

    def test_markdown_only_change_is_blocked(self):
        self.policy.write_text(CONFIG.replace("AUTHORS=geromet", "AUTHORS=alice"))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_actions_only_change_is_blocked(self):
        self.assertEqual(self.check(actions="alice")["status"], "BLOCKED")

    def test_matching_alice_with_slots_can_be_admitted(self):
        self.policy.write_text(CONFIG.replace("AUTHORS=geromet", "AUTHORS=alice"))
        self.slots.write_text(json.dumps([issue(2,"A","ACTIVE",2,"alice"), issue(3,"B","STANDBY",1,"alice")]))
        self.assertEqual(self.check(actions="alice", runtime="alice", slots=True)["status"], "TRUST_AND_OFFLINE_SLOTS_CONSISTENT")

    def test_empty_actions_variable_uses_fallback(self):
        self.assertEqual(self.check(actions="")["status"], "TRUST_CONFIG_CONSISTENT_ONLY")

    def test_explicit_empty_python_override_fails_closed(self):
        self.assertEqual(self.check(runtime="")["status"], "BLOCKED")

    def test_whitespace_splits_python_and_actions_admission(self):
        self.policy.write_text(CONFIG.replace("AUTHORS=geromet", "AUTHORS=geromet, alice"))
        result = self.check(actions="geromet, alice", runtime="geromet, alice")
        self.assertEqual(result["status"], "BLOCKED")
        self.assertTrue(any("membership differs for alice" in x for x in result["errors"]))

    def test_invalid_tokens_and_duplicate_case_are_blocked(self):
        invalid = ["geromet,", ",geromet", "geromet,,alice", " geromet", "geromet ",
                   "geromet,alice ", "geromet,GEROMET", "geromet,geromet", "a@b", "-alice",
                   "alice-", "alice\tbob", "alice\nbob", "a" * 40]
        for raw in invalid:
            with self.subTest(raw=raw):
                self.policy.write_text(CONFIG.replace("AUTHORS=geromet", "AUTHORS=" + raw))
                self.assertEqual(self.check(actions=raw, runtime=raw)["status"], "BLOCKED")

    def test_multi_actor_and_case_insensitive_membership(self):
        self.policy.write_text(CONFIG.replace("AUTHORS=geromet", "AUTHORS=Geromet,ALICE"))
        result = self.check(actions="geromet,alice", runtime="Geromet,ALICE")
        self.assertEqual(result["status"], "TRUST_CONFIG_CONSISTENT_ONLY")
        self.assertEqual(result["declared"], ["alice", "geromet"])

    def test_untrusted_slot_creator_does_not_grant_authority(self):
        self.slots.write_text(json.dumps([issue(1,"A","ACTIVE",1,"intruder"),issue(2,"B","STANDBY",0)]))
        self.assertEqual(self.check(slots=True)["status"], "BLOCKED")

    def test_dual_active_equal_epoch_is_blocked(self):
        self.slots.write_text(json.dumps([issue(1,"A","ACTIVE",2),issue(2,"B","ACTIVE",2)]))
        self.assertEqual(self.check(slots=True)["status"], "BLOCKED")

    def test_duplicate_slot_is_blocked(self):
        self.slots.write_text(json.dumps([issue(1,"A","ACTIVE",1),issue(2,"B","ACTIVE",2),issue(3,"A","STANDBY",0)]))
        self.assertEqual(self.check(slots=True)["status"], "BLOCKED")

    def test_workflow_drift_blocks_verdict(self):
        self.workflow.write_text(WORKFLOW.replace(preflight.MEMBERSHIP, "true", 1))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_subtle_workflow_bypass_is_blocked(self):
        # Retaining the expected membership substring does not justify a pass
        # if the rest of a source predicate changes to admit arbitrary events.
        self.workflow.write_text(WORKFLOW.replace("    if: ", "    if: true || ", 1))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_missing_declared_author_field_is_blocked(self):
        self.policy.write_text("COORDINATION_ISSUE_TITLE=[fleet-control] coordination\n")
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_duplicate_declared_author_field_is_blocked(self):
        self.policy.write_text(CONFIG.replace("MISSION_TITLE_PREFIX=", "COORDINATION_TRUSTED_AUTHORS=alice\nMISSION_TITLE_PREFIX="))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_later_prose_alias_is_not_an_authoritative_declaration(self):
        self.policy.write_text(CONFIG + "\nCOORDINATION_TRUSTED_AUTHORS=alice\n")
        self.assertEqual(self.check()["status"], "TRUST_CONFIG_CONSISTENT_ONLY")

    def test_later_unversioned_example_fence_is_ignored(self):
        self.policy.write_text(CONFIG + "\n```text\nCOORDINATION_TRUSTED_AUTHORS=alice\n```\n")
        self.assertEqual(self.check()["status"], "TRUST_CONFIG_CONSISTENT_ONLY")

    def test_earlier_unversioned_example_fence_is_ignored(self):
        self.policy.write_text("```text\nCOORDINATION_TRUSTED_AUTHORS=alice\n```\n" + CONFIG)
        self.assertEqual(self.check()["status"], "TRUST_CONFIG_CONSISTENT_ONLY")

    def test_second_versioned_configuration_fence_is_blocked(self):
        self.policy.write_text(CONFIG + "\n```text\nPHASE0_CONFIG_VERSION=1\nCOORDINATION_TRUSTED_AUTHORS=alice\n```\n")
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_unterminated_configuration_fence_is_blocked(self):
        self.policy.write_text(CONFIG.replace("\n```\n", "\n"))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_unsupported_configuration_version_is_blocked(self):
        self.policy.write_text(CONFIG.replace("PHASE0_CONFIG_VERSION=1", "PHASE0_CONFIG_VERSION=999"))
        self.assertEqual(self.check()["status"], "BLOCKED")

    def test_duplicate_configuration_version_is_blocked(self):
        self.policy.write_text(CONFIG.replace("TARGET_REPOSITORY=", "PHASE0_CONFIG_VERSION=1\nTARGET_REPOSITORY="))
        self.assertEqual(self.check()["status"], "BLOCKED")
    def test_malformed_offline_slot_snapshot_is_blocked_not_traceback(self):
        # Offline JSON is caller-supplied, not an authenticated GitHub API result.
        # These nested shapes used to raise uncaught AttributeError in discovery.
        malformed = [
            ("body as object", "body", {"marker": "FLEET_COORDINATION_V1"}),
            ("body as integer", "body", 42),
            ("creator as string", "user", "geromet"),
            ("creator login as integer", "user", {"login": 42}),
            ("title as array", "title", []),
            ("state as array", "state", []),
            ("issue number as text", "number", "186"),
            ("issue number as bool", "number", True),
            ("missing creator", "user", None),
        ]
        for label, field, value in malformed:
            with self.subTest(label=label):
                bad_slot = issue(186, "B", "ACTIVE", 2)
                bad_slot[field] = value
                self.slots.write_text(json.dumps([issue(168, "A", "DRAINING", 1), bad_slot]))
                with mock.patch.dict(os.environ, {}, clear=True):
                    result = preflight.preflight(self.policy, self.workflow, "geromet",
                                                None, self.slots)
                    self.assertEqual(result["status"], "BLOCKED", result)
                    self.assertEqual(result["slots"], "INVALID", result)
                    self.assertTrue(any("malformed issue snapshot at index 1" in e
                                        for e in result["errors"]), result)
                    with mock.patch("sys.stdout", new_callable=io.StringIO) as output:
                        exit_code = preflight.main([
                            "--config", str(self.policy), "--workflow", str(self.workflow),
                            "--actions-variable", "geromet", "--slots-json", str(self.slots)])
                    self.assertEqual(exit_code, 2)
                    self.assertEqual(json.loads(output.getvalue())["status"], "BLOCKED")

    def test_present_malformed_user_must_not_fall_back_to_author_alias(self):
        # The offline author alias is only a fallback when user is absent.
        # A present but falsey malformed API field must fail closed.
        for bad_user in (None, False, 0, "", [], {}):
            with self.subTest(bad_user=bad_user):
                malformed = issue(186, "B", "ACTIVE", 2)
                malformed["user"] = bad_user
                malformed["author"] = {"login": "geromet"}
                self.slots.write_text(json.dumps([
                    issue(168, "A", "DRAINING", 1), malformed]))
                with mock.patch.dict(os.environ, {}, clear=True):
                    result = preflight.preflight(
                        self.policy, self.workflow, "geromet", None, self.slots)
                self.assertEqual(result["status"], "BLOCKED", result)
                self.assertEqual(result["slots"], "INVALID", result)
                self.assertTrue(any("malformed issue snapshot at index 1" in e
                                    for e in result["errors"]), result)

    def test_offline_author_fallback_preserves_valid_slot_selection(self):
        fallback = issue(186, "B", "ACTIVE", 2)
        fallback["author"] = fallback.pop("user")
        self.slots.write_text(json.dumps([issue(168, "A", "DRAINING", 1), fallback]))
        value = self.check(slots=True)
        self.assertEqual(value["status"], "TRUST_AND_OFFLINE_SLOTS_CONSISTENT")
        self.assertEqual(value["slots"], "VALID_ACTIVE_B_EPOCH_2")

    def test_json_cli_success_and_failure_status(self):
        with mock.patch.dict(os.environ, {}, clear=True):
            self.assertEqual(preflight.main(["--config", str(self.policy), "--workflow", str(self.workflow),
                                             "--actions-unset"]), 0)
            self.assertEqual(preflight.main(["--config", str(self.policy), "--workflow", str(self.workflow),
                                             "--actions-variable", "alice"]), 2)


if __name__ == "__main__":
    unittest.main()
