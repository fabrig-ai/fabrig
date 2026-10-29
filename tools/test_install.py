"""Exercise installation against local fixture repositories, never fleet state."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


TOOLS = Path(__file__).resolve().parent


class InstallTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="test-install-", dir=TOOLS)
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.home = self.base / "home"
        self.home.mkdir()
        self.env = dict(os.environ, HOME=str(self.home), GIT_CONFIG_GLOBAL="/dev/null",
                        GIT_CONFIG_NOSYSTEM="1")
        self.source = self.base / "source"
        self.source.mkdir()
        self.git(self.source, "init", "-b", "main")
        self.git(self.source, "config", "user.name", "Fixture")
        self.git(self.source, "config", "user.email", "fixture@example.invalid")
        self.git(self.source, "remote", "add", "origin", str(self.source))
        (self.source / "tools/codex-run").mkdir(parents=True)
        shutil.copy2(TOOLS / "install", self.source / "tools/install")
        shutil.copy2(TOOLS / "installed", self.source / "tools/installed")
        self.tool = self.source / "tools/codex-run/codex-run"
        self.tool.write_text("#!/bin/bash\necho fixture\n")
        self.tool.chmod(0o700)
        (self.source / "tools/slots").mkdir()
        self.claim_tool = self.source / "tools/slots/claim"
        self.claim_tool.write_text("#!/bin/bash\necho claim-fixture\n")
        self.claim_tool.chmod(0o700)
        (self.source / "tools/queue").mkdir()
        self.precheck_tool = self.source / "tools/queue/precheck"
        self.precheck_tool.write_text("#!/usr/bin/env python3\nprint('precheck-fixture')\n")
        self.precheck_tool.chmod(0o700)
        (self.source / "tools/heavy").mkdir()
        self.heavy_tool = self.source / "tools/heavy/heavy"
        self.heavy_tool.write_text("#!/bin/bash\necho heavy-fixture\n")
        self.heavy_tool.chmod(0o700)
        self.commit()
        self.checkout = self.home / "src/fabrig/main"
        self.link = self.home / ".local/bin/codex-run"
        self.claim_link = self.home / ".local/bin/claim"
        self.precheck_link = self.home / ".local/bin/precheck"
        self.heavy_link = self.home / ".local/bin/heavy"

    def git(self, directory, *args):
        return subprocess.check_output(["git", "-C", str(directory), *args],
                                       env=self.env, text=True, stderr=subprocess.DEVNULL).strip()

    def commit(self):
        self.git(self.source, "add", ".")
        self.git(self.source, "commit", "-m", "Fixture tool")

    def install(self, *args, success=True):
        result = subprocess.run(["python3", "-I", str(self.source / "tools/install"), *args],
                                env=self.env, text=True, capture_output=True,
                                stdin=subprocess.DEVNULL)
        self.assertEqual(result.returncode == 0, success, result.stdout + result.stderr)
        return result.stdout + result.stderr

    def test_dry_run_creates_nothing(self):
        output = self.install("--dry-run")
        self.assertIn("git clone", output)
        self.assertIn("(absent) -> " + str(self.checkout / "tools/codex-run/codex-run"), output)
        self.assertIn("(absent) -> " + str(self.checkout / "tools/slots/claim"), output)
        self.assertIn("(absent) -> " + str(self.checkout / "tools/queue/precheck"), output)
        self.assertIn("(absent) -> " + str(self.checkout / "tools/heavy/heavy"), output)
        self.assertFalse((self.home / "src").exists())
        self.assertFalse((self.home / ".local").exists())

    def test_clone_update_and_detached_main(self):
        self.link.parent.mkdir(parents=True)
        self.link.symlink_to("/fixture/old-codex-run")
        self.claim_link.symlink_to("/fixture/old-claim")
        self.precheck_link.symlink_to("/fixture/old-precheck")
        self.heavy_link.symlink_to("/fixture/old-heavy")
        output = self.install()
        self.assertIn("/fixture/old-codex-run -> ", output)
        self.assertIn("/fixture/old-claim -> ", output)
        self.assertIn("/fixture/old-precheck -> ", output)
        self.assertIn("/fixture/old-heavy -> ", output)
        self.assertEqual(self.link.readlink(), self.checkout / "tools/codex-run/codex-run")
        self.assertEqual(self.claim_link.readlink(), self.checkout / "tools/slots/claim")
        self.assertEqual(self.precheck_link.readlink(), self.checkout / "tools/queue/precheck")
        self.assertEqual(self.heavy_link.readlink(), self.checkout / "tools/heavy/heavy")
        self.assertEqual(self.git(self.checkout, "rev-parse", "--abbrev-ref", "HEAD"), "HEAD")
        before = self.git(self.checkout, "rev-parse", "HEAD")
        self.tool.write_text("#!/bin/bash\necho updated-fixture\n")
        self.claim_tool.write_text("#!/bin/bash\necho updated-claim-fixture\n")
        self.precheck_tool.write_text("#!/usr/bin/env python3\nprint('updated-precheck-fixture')\n")
        self.heavy_tool.write_text("#!/bin/bash\necho updated-heavy-fixture\n")
        self.commit()
        self.install("--dry-run")
        self.assertEqual(self.git(self.checkout, "rev-parse", "HEAD"), before)
        self.install()
        self.assertEqual(self.link.read_text(), self.tool.read_text())
        self.assertEqual(self.claim_link.read_text(), self.claim_tool.read_text())
        self.assertEqual(self.precheck_link.read_text(), self.precheck_tool.read_text())
        self.assertEqual(self.heavy_link.read_text(), self.heavy_tool.read_text())
        self.assertEqual(self.git(self.checkout, "rev-parse", "HEAD"),
                         self.git(self.source, "rev-parse", "main"))

    def test_precheck_executable_link(self):
        self.install()
        self.assertEqual(self.precheck_link.readlink(), self.checkout / "tools/queue/precheck")
        self.assertTrue(os.access(self.precheck_link, os.X_OK))
        result = subprocess.run([str(self.precheck_link)], text=True, capture_output=True,
                                stdin=subprocess.DEVNULL)
        self.assertEqual((result.returncode, result.stdout), (0, "precheck-fixture\n"))

    def test_dirty_clone_is_preserved(self):
        self.install()
        self.link.write_text("local changes\n")
        output = self.install(success=False)
        self.assertIn("dedicated clone is dirty", output)
        self.assertEqual(self.link.read_text(), "local changes\n")

    def test_checkout_symlink_is_refused(self):
        self.checkout.parent.mkdir(parents=True)
        self.checkout.symlink_to(self.source, target_is_directory=True)
        self.assertIn("refusing a symlink checkout", self.install(success=False))
        self.assertFalse(self.link.exists())

    def test_regular_program_is_preserved(self):
        self.link.parent.mkdir(parents=True)
        self.link.write_text("keep program\n")
        self.assertIn("refusing to replace a non-symlink", self.install(success=False))
        self.assertEqual(self.link.read_text(), "keep program\n")

    def test_bad_manifest_cannot_escape_tools(self):
        (self.source / "tools/installed").write_text("codex-run ../escape\n")
        self.commit()
        self.assertIn("invalid tools/installed entry", self.install(success=False))
        self.assertFalse(self.link.exists())

    def test_second_regular_program_preserves_both_links(self):
        self.link.parent.mkdir(parents=True)
        self.link.symlink_to("/fixture/old-codex-run")
        self.claim_link.write_text("keep claim program\n")
        self.assertIn("refusing to replace a non-symlink", self.install(success=False))
        self.assertEqual(self.claim_link.read_text(), "keep claim program\n")
        self.assertEqual(self.link.readlink(), Path("/fixture/old-codex-run"))

    def test_fourth_regular_program_preserves_all_links(self):
        self.link.parent.mkdir(parents=True)
        self.link.symlink_to("/fixture/old-codex-run")
        self.claim_link.symlink_to("/fixture/old-claim")
        self.precheck_link.symlink_to("/fixture/old-precheck")
        self.heavy_link.write_text("keep heavy program\n")
        self.assertIn("refusing to replace a non-symlink", self.install(success=False))
        self.assertEqual(self.heavy_link.read_text(), "keep heavy program\n")
        self.assertEqual(self.link.readlink(), Path("/fixture/old-codex-run"))
        self.assertEqual(self.claim_link.readlink(), Path("/fixture/old-claim"))
        self.assertEqual(self.precheck_link.readlink(), Path("/fixture/old-precheck"))


if __name__ == "__main__":
    unittest.main()
