"""Slot claims against disposable local state; no network or real claims."""

from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
import importlib.machinery
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
LOADER = importlib.machinery.SourceFileLoader("claim", str(DIRECTORY / "claim"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
claim = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(claim)
NOW = datetime(2026, 10, 28, 12, tzinfo=timezone.utc)


FIXTURE = {
    ("hetzner2", "1"): "lead/skein", ("hetzner2", "2"): "lead/skein", ("hetzner2", "3"): "pool",
    ("hetzner3", "1"): "pool", ("hetzner3", "2"): "pool", ("hetzner3", "3"): "pool",
    ("hetzner6", "1"): "lead/temper", ("hetzner6", "2"): "lead/smith", ("hetzner6", "3"): "lead/temper",
    ("hetzner7", "1"): "lead/jig", ("hetzner7", "2"): "lead/jig", ("hetzner7", "3"): "lead/jig"}


class ClaimTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT, prefix=".claim-test-")
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name)
        self.directory = self.home / "src/fabrig/claims"
        self.path = self.directory / "hetzner2-1"

    def create(self, owner="lead", what="a task"):
        with claim.state_lock(self.directory), redirect_stdout(io.StringIO()):
            claim.create_claim(self.path, owner, what, NOW)

    def cli(self, *args):
        with patch.object(sys, "argv", ["claim", *args]), patch.object(Path, "home", return_value=self.home), \
                redirect_stdout(io.StringIO()) as output, redirect_stderr(io.StringIO()) as errors:
            code = claim.main()
        return code, output.getvalue(), errors.getvalue()

    def test_claim_records_owner_work_utc_and_private_mode(self):
        self.create()
        self.assertEqual(json.loads(self.path.read_text()), {
            "owner": "lead", "what": "a task", "created": "2026-10-28T12:00:00+00:00"})
        self.assertEqual(self.path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(self.directory.stat().st_mode & 0o777, 0o700)

    def test_existing_claim_refuses_and_prints_holder(self):
        self.create()
        before = self.path.read_bytes()
        code, _, error = self.cli("claim", "hetzner2", "1", "other", "different task",
                                  "--lent-by", "lead/skein")
        self.assertEqual(code, 1)
        self.assertIn("held by lead: a task", error)
        self.assertEqual(self.path.read_bytes(), before)

    def test_only_holder_can_release(self):
        self.create()
        code, _, error = self.cli("release", "hetzner2", "1", "other")
        self.assertEqual(code, 1)
        self.assertIn("held by lead", error)
        self.assertTrue(self.path.exists())
        code, output, _ = self.cli("release", "hetzner2", "1", "lead")
        self.assertEqual(code, 0)
        self.assertIn("released by lead", output)
        self.assertFalse(self.path.exists())

    def test_list_shows_inventory_free_slots_holder_and_age(self):
        self.create()
        later = datetime(2026, 10, 29, 14, 3, 4, tzinfo=timezone.utc)
        with redirect_stdout(io.StringIO()) as output:
            claim.list_claims(self.directory, FIXTURE, later)
        rows = output.getvalue().splitlines()
        self.assertEqual(len(rows), 12)
        self.assertIn("lead: a task (since 2026-10-28T12:00:00+00:00); held 1d 02:03:04", rows[0])
        self.assertTrue(rows[0].startswith("hetzner2 1 [lead/skein]: lead: a task"))
        self.assertNotIn("lent by", rows[0])
        self.assertEqual(rows[2], "hetzner2 3 [pool]: free")
        self.assertEqual(rows[-1], "hetzner7 3 [lead/jig]: free")

    def test_missing_claim_release_fails(self):
        code, _, _ = self.cli("release", "hetzner2", "1", "lead")
        self.assertEqual(code, 1)

    def test_malformed_record_refuses_without_deleting_it(self):
        self.directory.mkdir(parents=True)
        self.path.write_text("[]")
        code, _, error = self.cli("release", "hetzner2", "1", "lead")
        self.assertEqual(code, 1)
        self.assertIn("invalid claim record", error)
        self.assertEqual(self.path.read_text(), "[]")

    def test_unknown_slot_refuses_before_creating_state(self):
        code, _, error = self.cli("claim", "../outside", "1", "lead", "task")
        self.assertEqual(code, 1)
        self.assertIn("unknown slot", error)
        self.assertFalse(self.directory.exists())

    def test_cli_claim_of_own_share_succeeds(self):
        code, output, error = self.cli("claim", "hetzner7", "3", "lead/jig", "task description")
        self.assertEqual(code, 0, error)
        self.assertIn("hetzner7-3 claimed by lead/jig: task description", output)

    def test_cli_claim_of_pool_succeeds(self):
        code, output, error = self.cli("claim", "hetzner3", "1", "lead/smith", "task")
        self.assertEqual(code, 0, error)
        self.assertIn("hetzner3-1 claimed by lead/smith: task", output)

    def test_another_share_requires_a_lend(self):
        code, _, error = self.cli("claim", "hetzner2", "1", "lead/jig", "task")
        self.assertEqual(code, 1)
        self.assertIn("belongs to share lead/skein", error)
        self.assertIn("--lent-by lead/skein", error)
        self.assertFalse(self.directory.exists())

    def test_lend_is_recorded_listed_and_released_by_holder(self):
        code, _, error = self.cli("claim", "hetzner2", "1", "lead/jig", "task",
                                  "--lent-by", "lead/skein")
        self.assertEqual(code, 0, error)
        self.assertEqual(json.loads(self.path.read_text())["lent_by"], "lead/skein")
        code, output, error = self.cli("list")
        self.assertEqual(code, 0, error)
        self.assertIn("hetzner2 1 [lead/skein]: lead/jig: task", output)
        self.assertIn("; lent by lead/skein", output)
        self.assertEqual(self.cli("release", "hetzner2", "1", "lead/skein")[0], 1)
        self.assertEqual(self.cli("release", "hetzner2", "1", "lead/jig")[0], 0)

    def test_wrong_lender_is_refused(self):
        for owner in ("lead/jig", "lead/skein"):
            with self.subTest(owner=owner):
                code, _, error = self.cli("claim", "hetzner2", "1", owner, "task",
                                          "--lent-by", "lead/smith")
                self.assertEqual(code, 1)
                self.assertIn("--lent-by lead/skein", error)
                self.assertFalse(self.path.exists())

    def test_free_lists_own_share_before_pool_and_omits_held_slots(self):
        self.create()
        self.assertEqual(self.cli("claim", "hetzner3", "2", "lead/smith", "task")[0], 0)
        with patch.object(claim, "inventory", return_value=FIXTURE):
            code, output, error = self.cli("list", "--free", "lead/skein")
        self.assertEqual(code, 0, error)
        self.assertEqual(output.splitlines(), [
            "hetzner2 2", "hetzner2 3", "hetzner3 1", "hetzner3 3"])
        with patch.object(claim, "inventory", return_value=FIXTURE):
            code, output, error = self.cli("list", "--free", "lead/smith")
        self.assertEqual(code, 0, error)
        self.assertEqual(output.splitlines(), [
            "hetzner6 2", "hetzner2 3", "hetzner3 1", "hetzner3 3"])

    def test_free_exits_zero_when_no_eligible_slots(self):
        with patch.object(claim, "inventory", return_value={("hetzner2", "1"): "lead/skein"}):
            code, output, error = self.cli("list", "--free", "lead/smith")
        self.assertEqual((code, output, error), (0, "", ""))

    def test_free_exits_zero_when_all_eligible_slots_are_held(self):
        self.create()
        with patch.object(claim, "inventory", return_value={("hetzner2", "1"): "lead/skein"}):
            code, output, error = self.cli("list", "--free", "lead/skein")
        self.assertEqual((code, output, error), (0, "", ""))

    def test_concurrent_claims_have_exactly_one_winner(self):
        environment = {**os.environ, "HOME": str(self.home)}
        commands = [[sys.executable, "-I", "-B", str(DIRECTORY / "claim"),
                     "claim", "hetzner3", "1", owner, "task"] for owner in ("first", "second")]
        children = [subprocess.Popen(argv, env=environment, stdin=subprocess.DEVNULL,
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                    for argv in commands]
        results = [child.communicate() for child in children]
        self.assertEqual(sorted(child.returncode for child in children), [0, 1])
        winner = json.loads((self.directory / "hetzner3-1").read_text())["owner"]
        self.assertTrue(any(f"held by {winner}: task" in errors for _, errors in results))

    def test_inventory_matches_assigned_builders(self):
        self.assertEqual(claim.inventory(), {
            ("hetzner2", "1"): "lead/skein", ("hetzner2", "2"): "lead/skein",
            ("hetzner2", "3"): "pool", ("hetzner2", "4"): "pool", ("hetzner2", "5"): "pool",
            ("hetzner3", "1"): "pool", ("hetzner3", "2"): "pool", ("hetzner3", "3"): "pool",
            ("hetzner3", "4"): "pool", ("hetzner3", "5"): "pool",
            ("hetzner6", "1"): "lead/temper", ("hetzner6", "2"): "lead/smith",
            ("hetzner6", "3"): "lead/temper",
            ("hetzner7", "1"): "lead/jig", ("hetzner7", "2"): "lead/jig",
            ("hetzner7", "3"): "lead/jig", ("hetzner7", "4"): "pool", ("hetzner7", "5"): "pool",
            ("hetzner7", "6"): "pool"})

    def test_inventory_accepts_comments_blank_lines_and_normalizes_numbers(self):
        path = self.home / "hosts"
        path.write_text("# shares\n\nhetzner2 01=lead/skein 2=pool # spare\n")
        self.assertEqual(claim.inventory(path), {
            ("hetzner2", "1"): "lead/skein", ("hetzner2", "2"): "pool"})

    def test_inventory_rejects_malformed_lines_and_duplicate_slots(self):
        path = self.home / "hosts"
        for content in ("hetzner2", "../host 1=pool", "hetzner2 1", "hetzner2 1=",
                        "hetzner2 =pool", "hetzner2 0=pool", "hetzner2 -1=pool",
                        "hetzner2 one=pool", "hetzner2 1=lead=jig", "hetzner2 1=lead\0jig",
                        "hetzner2 1=pool 1=pool", "hetzner2 1=pool\nhetzner2 01=lead/jig"):
            with self.subTest(content=content):
                path.write_text(content + "\n")
                with self.assertRaises(ValueError):
                    claim.inventory(path)


if __name__ == "__main__":
    unittest.main()
