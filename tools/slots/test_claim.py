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
        code, _, error = self.cli("claim", "hetzner2", "1", "other", "different task")
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
            claim.list_claims(self.directory, claim.inventory(), later)
        rows = output.getvalue().splitlines()
        self.assertEqual(len(rows), 10)
        self.assertIn("lead: a task (since 2026-10-28T12:00:00+00:00); held 1d 02:03:04", rows[0])
        self.assertEqual(rows[-1], "hetzner7 3: free")

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

    def test_cli_claim_succeeds(self):
        code, output, error = self.cli("claim", "hetzner7", "3", "lead", "task description")
        self.assertEqual(code, 0, error)
        self.assertIn("hetzner7-3 claimed by lead: task description", output)

    def test_concurrent_claims_have_exactly_one_winner(self):
        environment = {**os.environ, "HOME": str(self.home)}
        commands = [[sys.executable, "-I", "-B", str(DIRECTORY / "claim"),
                     "claim", "hetzner2", "1", owner, "task"] for owner in ("first", "second")]
        children = [subprocess.Popen(argv, env=environment, stdin=subprocess.DEVNULL,
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                    for argv in commands]
        results = [child.communicate() for child in children]
        self.assertEqual(sorted(child.returncode for child in children), [0, 1])
        winner = json.loads(self.path.read_text())["owner"]
        self.assertTrue(any(f"held by {winner}: task" in errors for _, errors in results))

    def test_inventory_matches_assigned_builders(self):
        self.assertEqual(claim.inventory(), [
            ("hetzner2", "1"), ("hetzner2", "2"), ("hetzner3", "1"), ("hetzner3", "2"),
            ("hetzner6", "1"), ("hetzner6", "2"), ("hetzner6", "3"),
            ("hetzner7", "1"), ("hetzner7", "2"), ("hetzner7", "3")])


if __name__ == "__main__":
    unittest.main()
