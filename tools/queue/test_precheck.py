"""Pre-ready checks on disposable local Git repositories, without Cargo or services."""

from contextlib import redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
LOADER = importlib.machinery.SourceFileLoader("precheck", str(DIRECTORY / "precheck"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
precheck = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(precheck)
mq, ALWAYS, PATTERNS = precheck.policy()


def metadata(root):
    names = ["skein-core", "smith-agent", "jig-app", "temper-app", "unrelated", "always"]
    directories = ["skein/core", "smith/agent", "jig/app", "temper/app", "unrelated", "always"]
    edges = {"smith-agent": ["skein-core"], "jig-app": ["smith-agent"], "temper-app": ["jig-app"]}
    return {"workspace_root": str(root), "workspace_members": names,
            "packages": [{"id": name, "name": name, "version": "1.0.0", "source": None,
                          "manifest_path": str(root / directory / "Cargo.toml")}
                         for name, directory in zip(names, directories)],
            "resolve": {"nodes": [{"id": name, "dependencies": edges.get(name, [])}
                                  for name in names]}}


class Fixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=ROOT, prefix=".precheck-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / 'slot ; $(touch BAD) space'
        self.repo.mkdir()
        self.env = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1")
        self.git("init", "-b", "original")
        self.git("config", "user.name", "Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.write(".gitignore", "target/\n")
        self.write("README.md", "base\n")
        self.write("Cargo.lock", 'version = 4\n\n[[package]]\nname = "skein-core"\nversion = "1.0.0"\n')
        for directory in ("skein/core", "smith/agent", "jig/app", "temper/app", "unrelated", "always"):
            self.write(directory + "/Cargo.toml", "fixture\n")
            self.write(directory + "/src/lib.rs", "base\n")
        for tree in mq.TREES:
            self.write(tree + "/scripts/old_forms.py", "# fixture\n")
            self.write(tree + "/scripts/old-forms.txt", "ceiling 10\n")
        self.commit()
        self.ancestor = self.git("rev-parse", "HEAD")
        self.git("branch", "base")
        self.git("branch", "tip")
        self.write("target/keep", "warm target\n")
        self.calls = []
        self.codes = {}
        self.effect = None
        self.metadata_effect = None
        self.tool = None

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args], env=self.env,
                                       text=True, stderr=subprocess.DEVNULL).rstrip("\n")

    def write(self, path, body):
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body)

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-m", "fixture")
        return self.git("rev-parse", "HEAD")

    def change(self, branch, path, body):
        self.git("switch", branch)
        self.write(path, body)
        oid = self.commit()
        self.git("switch", "original")
        return oid

    def runner(self, argv, **options):
        self.calls.append((list(argv), options["cwd"]))
        self.assertEqual(options["stdin"], subprocess.DEVNULL)
        self.assertFalse(options.get("shell"))
        if self.effect:
            self.effect(argv)
        code = self.codes.get(argv[0], 0)
        if argv[0] == "heavy":
            code = self.codes.get(argv[2], code)
        if code == 2:
            options["stdout"].write("fixture below the disk floor\n")
        return subprocess.CompletedProcess(argv, code)

    def load_metadata(self, repo):
        if self.metadata_effect:
            self.metadata_effect()
        return metadata(repo)

    def run_tool(self, *, base="base", tip="tip", production_runner=False):
        self.tool = precheck.Precheck(self.repo, base, tip, mq=mq, always=["always"],
                                     read_by_tests=PATTERNS,
                                     runner=None if production_runner else self.runner,
                                     metadata=self.load_metadata, busy=lambda _: "fixture observation")
        with redirect_stdout(io.StringIO()) as output:
            result = self.tool.run()
        self.printed = output.getvalue()
        return result

    def restored(self):
        self.assertEqual(self.git("symbolic-ref", "HEAD"), "refs/heads/original")
        self.assertEqual(self.git("rev-parse", "HEAD"), self.ancestor)
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertFalse((self.repo / ".git/MERGE_HEAD").exists())
        self.assertFalse((self.repo / ".git/precheck.unresolved").exists())
        self.assertEqual((self.repo / "target/keep").read_text(), "warm target\n")
        self.assertTrue((self.tool.log_directory / "commands.log").exists())
        self.assertTrue(self.tool.record["restored"])


class CandidateTests(Fixture):
    def test_checks_exact_merged_worktree_and_reverse_dependencies(self):
        base = self.change("base", "unrelated/src/lib.rs", "main API\n")
        tip = self.change("tip", "skein/core/src/lib.rs", "branch API\n")
        observations = []
        def inspect(argv):
            if argv[0] == "heavy":
                observations.append(((self.repo / "unrelated/src/lib.rs").read_text(),
                                     (self.repo / "skein/core/src/lib.rs").read_text(),
                                     self.git("rev-parse", "HEAD")))
        self.effect = inspect
        refs = self.git("show-ref", "--heads")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["base"], base)
        self.assertEqual(self.tool.record["tip"], tip)
        self.assertEqual(observations, [("main API\n", "branch API\n", base)] * 2)
        self.assertEqual(self.tool.record["selection"], ["always", "jig-app", "skein-core", "smith-agent", "temper-app"])
        fmt = next(argv for argv, _ in self.calls if argv[:2] == ["cargo", "fmt"])
        self.assertEqual(fmt, ["cargo", "fmt", "--check", "-p", "skein-core"])
        self.assertEqual(refs, self.git("show-ref", "--heads"))
        self.restored()

    def test_cross_branch_lint_failure_is_not_hidden_by_tip_only_checks(self):
        self.change("base", "unrelated/src/lib.rs", "main API\n")
        self.change("tip", "skein/core/src/lib.rs", "branch API\n")
        def interaction(argv):
            if argv[:3] == ["heavy", "cargo", "clippy"]:
                if ((self.repo / "unrelated/src/lib.rs").read_text() == "main API\n" and
                        (self.repo / "skein/core/src/lib.rs").read_text() == "branch API\n"):
                    self.codes["clippy"] = 1
        self.effect = interaction
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "failed")
        self.restored()

    def test_merge_lockfile_is_selected_without_reading_unchanged_head(self):
        self.change("tip", "Cargo.lock", 'version = 4\n')
        seen = []
        real = mq.lock_changes
        def lock_changes(before, after, data):
            seen.append((before, after))
            return real(before, after, data)
        with patch.object(mq, "lock_changes", side_effect=lock_changes):
            self.assertEqual(self.run_tool(), 0)
        self.assertIn('name = "skein-core"', seen[0][0])
        self.assertEqual(seen[0][1], "version = 4\n")
        self.assertEqual(self.tool.record["selection"], ["always", "jig-app", "skein-core", "smith-agent", "temper-app"])
        self.assertFalse(any(argv[:2] == ["cargo", "fmt"] for argv, _ in self.calls))
        self.restored()

    def test_metadata_comes_from_merged_manifests(self):
        self.change("tip", "smith/agent/Cargo.toml", "merged manifest\n")
        def inspect():
            self.assertEqual((self.repo / "smith/agent/Cargo.toml").read_text(), "merged manifest\n")
        self.metadata_effect = inspect
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], ["always", "jig-app", "smith-agent", "temper-app"])
        self.restored()

    def test_full_selection_uses_tools_fmt_and_plain_heavy_only(self):
        self.change("tip", "Cargo.toml", "workspace fixture\n")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "full")
        commands = [argv for argv, _ in self.calls]
        self.assertIn(["cargo", "fmt", "--all", "--check"], commands)
        self.assertIn(["heavy", "cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], commands)
        self.assertIn(["heavy", "cargo", "nextest", "run", "--workspace", "--no-run"], commands)
        self.assertFalse(any("--gate" in argv or "fuzzy" in argv or "default" in argv for argv in commands))
        self.restored()

    def test_docs_and_contained_tips_have_no_build_steps(self):
        for branch, body in (("tip", "updated docs\n"), ("base", "updated docs\n")):
            self.change(branch, "README.md", body)
        self.metadata_effect = lambda: self.fail("docs must not request Cargo metadata")
        self.assertEqual(self.run_tool(tip="base"), 0)
        self.assertEqual(self.calls, [])
        self.assertEqual(self.tool.record["paths"], [])
        self.assertIn("No tests executed beyond selected tools suites", self.printed)
        self.restored()

    def test_docs_changes_are_empty_selection(self):
        self.change("tip", "docs/new.md", "unread\n")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "docs")
        self.assertEqual(self.calls, [])
        self.restored()

    def test_read_by_tests_forces_full(self):
        self.change("tip", "temper/docs/help/new.md", "read by tests\n")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "full")
        self.restored()

    def test_policy_lists_are_frozen_before_checkout_changes(self):
        self.change("tip", "tools/queue/read-by-tests.txt", "README.md\n")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "docs")
        self.restored()

    def test_tools_discovery_and_exact_manual_recovery_exclusion(self):
        self.git("switch", "tip")
        for name in ("tools/test_python.py", "tools/bash/test-bash", "tools/python/test-python",
                     "tools/heavy/test-heavy-recovery", "tools/other/test-heavy-recovery", "tools/test-noexec"):
            self.write(name, "#!/bin/sh\nexit 0\n")
            if name != "tools/test-noexec":
                (self.repo / name).chmod(0o755)
        self.commit()
        self.git("switch", "original")
        self.assertEqual(self.run_tool(), 0)
        commands = [argv for argv, _ in self.calls]
        self.assertEqual(commands, [["./tools/bash/test-bash"], ["./tools/other/test-heavy-recovery"],
                                    ["./tools/python/test-python"], ["python3", "-I", "-B", "tools/test_python.py"]])
        self.assertTrue(all(step["timeout"] == mq.TOOLS_TIMEOUT for step in self.tool.record["plan"]))
        self.restored()

    def test_all_tools_run_after_failure_and_cargo_is_stopped(self):
        self.change("tip", "Cargo.toml", "full\n")
        self.change("tip", "tools/test_a.py", "# a\n")
        self.change("tip", "tools/test_b.py", "# b\n")
        self.codes["python3"] = 1
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(len(self.calls), 2)
        self.assertFalse(any(argv[0] == "heavy" for argv, _ in self.calls))
        self.restored()

    def test_baseline_only_conflict_uses_base_and_recounts_without_commits(self):
        self.change("base", "skein/scripts/old-forms.txt", "ceiling 8\n")
        self.change("tip", "skein/scripts/old-forms.txt", "ceiling 9\n")
        def recount(argv):
            if "--write" in argv:
                self.assertEqual((self.repo / "skein/scripts/old-forms.txt").read_text(), "ceiling 8\n")
                self.write("skein/scripts/old-forms.txt", "ceiling 7\n")
        self.effect = recount
        self.assertEqual(self.run_tool(), 0)
        self.assertIn("skein/scripts/old-forms.txt", self.tool.record["paths"])
        self.assertTrue(any(step["name"] == "ratchet:skein" for step in self.tool.record["steps"]))
        self.restored()

    def test_baseline_recount_raise_rejects_and_restores(self):
        self.change("base", "skein/scripts/old-forms.txt", "ceiling 8\n")
        self.change("tip", "skein/scripts/old-forms.txt", "ceiling 9\n")
        self.codes["python3"] = 1
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["error"], "ratchet:skein failed")
        self.restored()

    def test_regular_merge_conflict_restores_without_checks(self):
        self.change("base", "README.md", "main\n")
        self.change("tip", "README.md", "tip\n")
        self.assertEqual(self.run_tool(), 1)
        self.assertIn("merge conflict", self.tool.record["error"])
        self.assertEqual(self.calls, [])
        self.restored()

    def test_detached_head_and_original_index_are_restored(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        self.git("switch", "--detach", self.ancestor)
        original = (self.repo / ".git/index").read_bytes()
        self.codes["clippy"] = 1
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.git("rev-parse", "HEAD"), self.ancestor)
        self.assertEqual(self.git("rev-parse", "--abbrev-ref", "HEAD"), "HEAD")
        self.assertEqual((self.repo / ".git/index").read_bytes(), original)
        self.assertTrue(self.tool.record["restored"])

    def test_nextest_exit_four_is_green_for_build_only(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        self.codes["nextest"] = 4
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["steps"][-1]["returncode"], 4)
        self.restored()

    def test_disk_floor_is_infrastructure_error_with_log(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        self.codes["clippy"] = 2
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.assertIn("below the disk floor", (self.tool.log_directory / "commands.log").read_text())
        self.restored()

    def test_metadata_errors_are_infrastructure_errors(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        def fail():
            raise RuntimeError("fixture invalid metadata")
        self.metadata_effect = fail
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.restored()

    def test_incomplete_plan_cannot_report_success(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        with patch.object(mq, "check_commands", return_value=[]):
            self.assertEqual(self.run_tool(), 1)
        self.assertIn("incomplete", self.tool.record["error"])
        self.restored()

    def test_source_modification_retains_candidate_and_marker(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        def modify(argv):
            if argv[:2] == ["cargo", "fmt"]:
                self.write("README.md", "unexpected writer\n")
        self.effect = modify
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "unresolved")
        self.assertEqual((self.repo / "README.md").read_text(), "unexpected writer\n")
        self.assertTrue((self.repo / ".git/precheck.unresolved").exists())

    def test_unexpected_commit_cannot_report_success_or_reset_branch(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        def commit(argv):
            if argv[:2] == ["cargo", "fmt"]:
                self.git("commit", "-am", "unexpected commit")
        self.effect = commit
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "unresolved")
        self.assertNotEqual(self.git("rev-parse", "HEAD"), self.tool.base)
        self.assertEqual(self.git("rev-parse", "original"), self.ancestor)

    def test_restoration_failure_cannot_report_success(self):
        with patch.object(precheck.Precheck, "restore", side_effect=RuntimeError("restore failed")):
            self.assertEqual(self.run_tool(), 1)
        self.assertFalse(self.tool.record["restored"])
        self.assertTrue((self.repo / ".git/precheck.unresolved").exists())

    def test_private_logs_survive_restoration(self):
        self.assertEqual(self.run_tool(), 0)
        self.assertFalse(self.tool.log_directory.is_relative_to(self.repo))
        self.assertEqual(self.tool.log_directory.stat().st_mode & 0o777, 0o700)
        for name in ("original.index", "commands.log", "result.json"):
            self.assertEqual((self.tool.log_directory / name).stat().st_mode & 0o777, 0o600)
        result = json.loads((self.tool.log_directory / "result.json").read_text())
        self.assertEqual((result["outcome"], result["restored"]), ("passed", True))
        self.assertFalse((self.repo / "BAD").exists())

    def test_raw_mode_changes_cannot_hide_behind_git_filemode_policy(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        self.git("config", "core.filemode", "false")
        def modify(argv):
            if argv[:2] == ["cargo", "fmt"]:
                (self.repo / "README.md").chmod(0o755)
        self.effect = modify
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "unresolved")

    def test_ratchet_changes_are_confined_to_its_baseline(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        def modify(argv):
            if "--write" in argv:
                self.write("README.md", "unexpected\n")
        self.effect = modify
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "unresolved")
        self.assertIn("outside its baseline", self.tool.record["error"])

    def test_full_and_lockfile_changes_share_merged_metadata(self):
        self.change("tip", "Cargo.toml", "full workspace\n")
        self.change("tip", "Cargo.lock", "version = 4\n")
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record["selection"], "full")
        self.restored()

    def test_ignored_target_collision_is_not_overwritten_by_merge(self):
        self.git("switch", "tip")
        self.write("target/keep", "tracked on tip\n")
        self.git("add", "-f", "target/keep")
        self.git("commit", "-m", "fixture collision")
        self.git("switch", "original")
        self.write("target/keep", "warm target\n")
        with self.assertRaisesRegex(RuntimeError, "existing ignored path"):
            self.run_tool()
        self.assertEqual((self.repo / "target/keep").read_text(), "warm target\n")
        self.assertEqual(self.git("symbolic-ref", "HEAD"), "refs/heads/original")

    def test_ignored_target_collision_is_not_overwritten_by_base_switch(self):
        self.git("switch", "base")
        self.write("target/keep", "tracked on base\n")
        self.git("add", "-f", "target/keep")
        self.git("commit", "-m", "fixture collision")
        self.git("switch", "original")
        self.write("target/keep", "warm target\n")
        with self.assertRaisesRegex(RuntimeError, "existing ignored path"):
            self.run_tool()
        self.assertEqual((self.repo / "target/keep").read_text(), "warm target\n")
        self.assertEqual(self.git("symbolic-ref", "HEAD"), "refs/heads/original")

    def test_executable_suites_run_their_shebangs(self):
        self.change("tip", "tools/test_python.py", "from pathlib import Path\nPath('target/python').write_text('python')\n")
        self.git("switch", "tip")
        self.write("tools/test-shell", "#!/bin/bash\nprintf bash >target/shell\n")
        (self.repo / "tools/test-shell").chmod(0o755)
        self.commit()
        self.git("switch", "original")
        self.assertEqual(self.run_tool(production_runner=True), 0)
        self.assertEqual((self.repo / "target/shell").read_text(), "bash")
        self.assertEqual((self.repo / "target/python").read_text(), "python")
        self.restored()

    def test_cli_default_base_is_pinned_locally_without_fetch(self):
        self.change("tip", "README.md", "docs\n")
        self.git("update-ref", "refs/remotes/origin/main", self.ancestor)
        fake = self.root / "bin"
        fake.mkdir()
        systemctl = fake / "systemctl"
        systemctl.write_text("#!/bin/sh\nexit 0\n")
        systemctl.chmod(0o755)
        result = subprocess.run([str(DIRECTORY / "precheck"), str(self.repo), "tip"],
                                env=dict(self.env, PATH=str(fake) + os.pathsep + os.environ["PATH"]),
                                text=True, capture_output=True, stdin=subprocess.DEVNULL)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("base: " + self.ancestor, result.stdout)
        self.assertIn('"restored": true', result.stdout)
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertEqual(self.git("symbolic-ref", "HEAD"), "refs/heads/original")

    def test_policy_code_and_lists_survive_switching_their_own_checkout(self):
        import shutil
        self.git("switch", "original")
        for name in ("precheck", "mq", "always.txt", "read-by-tests.txt"):
            target = self.repo / "tools/queue" / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(DIRECTORY / name, target)
        self.ancestor = self.commit()
        self.git("branch", "-f", "base", self.ancestor)
        self.git("branch", "-f", "tip", self.ancestor)
        self.change("tip", "tools/queue/mq", "invalid candidate policy code!\n")
        self.change("tip", "tools/queue/read-by-tests.txt", "README.md\n")
        self.change("tip", "tools/queue/always.txt", "missing-package\n")
        self.change("tip", "README.md", "docs\n")
        fake = self.root / "bin"
        fake.mkdir()
        self.write("target/keep", "warm target\n")
        (fake / "systemctl").write_text("#!/bin/sh\nexit 0\n")
        (fake / "systemctl").chmod(0o755)
        result = subprocess.run([str(self.repo / "tools/queue/precheck"), "--base", "base", str(self.repo), "tip"],
                                env=dict(self.env, PATH=str(fake) + os.pathsep + os.environ["PATH"]),
                                text=True, capture_output=True, stdin=subprocess.DEVNULL)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("selection: docs", result.stdout)
        self.assertEqual(self.git("status", "--porcelain"), "")

    def test_original_file_permissions_are_restored_under_private_umask(self):
        self.change("tip", "skein/core/src/lib.rs", "candidate\n")
        original = self.repo / "skein/core/src/lib.rs"
        original.chmod(0o644)
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(original.stat().st_mode & 0o777, 0o644)
        self.restored()

    def test_raw_clean_crlf_contents_are_restored(self):
        self.git("config", "core.autocrlf", "input")
        self.change("tip", "README.md", "candidate\n")
        original = self.repo / "README.md"
        original.write_bytes(b"base\r\n")
        self.git("add", "README.md")
        self.assertEqual(self.git("status", "--porcelain"), "")
        code = self.run_tool()
        self.assertEqual(code, 0, self.printed)
        self.assertEqual(original.read_bytes(), b"base\r\n")
        self.restored()

    def test_detached_success_restores_exact_index(self):
        self.change("tip", "README.md", "docs\n")
        self.git("switch", "--detach", self.ancestor)
        original = (self.repo / ".git/index").read_bytes()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.git("rev-parse", "--abbrev-ref", "HEAD"), "HEAD")
        self.assertEqual((self.repo / ".git/index").read_bytes(), original)
        self.assertTrue(self.tool.record["restored"])

    def test_tools_exit_four_is_failure(self):
        self.change("tip", "tools/test_suite.py", "# fixture\n")
        self.codes["python3"] = 4
        self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record["outcome"], "failed")
        self.restored()

    def test_directory_rename_merge_does_not_consume_ignored_file(self):
        self.git("switch", "base")
        self.git("mv", "unrelated", "moved")
        self.commit()
        self.git("switch", "original")
        self.change("tip", "unrelated/added.rs", "tip addition\n")
        # An exclude local to the fixture leaves its committed branch histories intact.
        (self.repo / ".git/info/exclude").write_text("moved/added.rs\n")
        self.write("moved/added.rs", "ignored data\n")
        with self.assertRaisesRegex(RuntimeError, "ignored path"):
            self.run_tool()
        self.assertEqual((self.repo / "moved/added.rs").read_text(), "ignored data\n")
        self.assertEqual(self.git("symbolic-ref", "HEAD"), "refs/heads/original")


class RefusalTests(Fixture):
    def refuse(self, expected):
        head = self.git("rev-parse", "HEAD")
        index = (self.repo / ".git/index").read_bytes()
        with self.assertRaisesRegex((RuntimeError, OSError), expected):
            self.run_tool()
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.assertEqual((self.repo / ".git/index").read_bytes(), index)
        self.assertEqual(self.calls, [])

    def test_dirty_staged_untracked_refused_without_mutation(self):
        for path, staged in (("README.md", False), ("README.md", True), ("new", False)):
            with self.subTest(path=path, staged=staged):
                self.write(path, "dirty\n")
                if staged:
                    self.git("add", path)
                self.refuse("dirty")
                self.git("reset", "--hard", self.ancestor)
                if path == "new":
                    (self.repo / path).unlink()

    def test_in_progress_states_are_refused(self):
        for name in ("MERGE_HEAD", "CHERRY_PICK_HEAD", "rebase-merge", "sequencer", "index.lock"):
            with self.subTest(name=name):
                path = self.repo / ".git" / name
                path.write_text(self.ancestor + "\n")
                self.refuse("in-progress")
                path.unlink()

    def test_skip_worktree_and_assume_unchanged_are_refused(self):
        for flag, undo in (("--skip-worktree", "--no-skip-worktree"),
                           ("--assume-unchanged", "--no-assume-unchanged")):
            self.git("update-index", flag, "README.md")
            self.refuse("index flags")
            self.git("update-index", undo, "README.md")

    def test_symlink_and_subdirectory_roots_are_refused(self):
        repo = self.repo
        for supplied in (repo / "skein", self.root / "symlink"):
            if supplied.name == "symlink":
                supplied.symlink_to(repo)
            self.repo = supplied
            with self.assertRaisesRegex(RuntimeError, "canonical checkout root|symlink"):
                self.run_tool()
        self.repo = repo

    def test_option_like_ref_and_non_commit_are_refused(self):
        for ref in ("--help", "HEAD:README.md", "missing"):
            with self.assertRaises(RuntimeError):
                self.run_tool(tip=ref)
        self.assertEqual(self.git("rev-parse", "HEAD"), self.ancestor)

    def test_durable_marker_refuses_reuse(self):
        (self.repo / ".git/precheck.unresolved").write_text("retained\n")
        self.refuse("unresolved")

    def assert_worker_marker_refused(self, marker):
        branch = self.git("symbolic-ref", "HEAD")
        head = self.git("rev-parse", "HEAD")
        index = (self.repo / ".git/index").read_bytes()
        contents = (self.repo / "README.md").read_bytes()
        with patch.object(precheck.Precheck, "prepare") as prepare, \
                patch.object(precheck.Precheck, "restore") as restore, \
                patch.object(precheck.Precheck, "protect_ignored") as preview, \
                patch.object(precheck.Precheck, "save") as save:
            with self.assertRaisesRegex(RuntimeError, "mq-worker-unresolved.json"):
                self.run_tool()
        for operation in (prepare, restore, preview, save):
            operation.assert_not_called()
        self.assertIsNone(self.tool.log_directory)
        self.assertFalse(self.repo.with_name(self.repo.name + ".precheck-logs").exists())
        self.assertFalse((self.repo / ".git/precheck.unresolved").exists())
        self.assertEqual(self.git("symbolic-ref", "HEAD"), branch)
        self.assertEqual(self.git("rev-parse", "HEAD"), head)
        self.assertEqual((self.repo / ".git/index").read_bytes(), index)
        self.assertEqual((self.repo / "README.md").read_bytes(), contents)
        self.assertTrue(marker.exists() or marker.is_symlink())
        self.assertEqual(self.calls, [])

    def test_existing_worker_marker_refuses_before_preparation_or_logging(self):
        marker = self.repo / ".git/mq-worker-unresolved.json"
        body = json.dumps({"attempt": "fixture-attempt", "generation": 1,
                           "artifacts": str(self.root / "job"), "candidate": self.ancestor}) + "\n"
        marker.write_text(body)
        self.assert_worker_marker_refused(marker)
        self.assertEqual(marker.read_text(), body)

    def test_dangling_worker_marker_refuses_without_following_or_removing_link(self):
        marker = self.repo / ".git/mq-worker-unresolved.json"
        marker.symlink_to("missing-worker-evidence")
        self.assert_worker_marker_refused(marker)
        self.assertEqual(marker.readlink(), Path("missing-worker-evidence"))
        self.assertFalse(marker.exists())

    def test_competing_separate_process_locks_refuse(self):
        for path in (self.repo.with_name(self.repo.name + ".queue.lock"),
                     self.repo.with_name(self.repo.name + ".worker.lock"),
                     self.repo / ".git/precheck.lock", self.repo / ".git/worker.lock"):
            with self.subTest(path=path):
                program = ('import fcntl,sys; f=open(sys.argv[1],"a"); '
                           'fcntl.flock(f,fcntl.LOCK_EX); print("locked",flush=True); sys.stdin.read()')
                child = subprocess.Popen([sys.executable, "-c", program, str(path)],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
                try:
                    self.assertEqual(child.stdout.readline(), "locked\n")
                    self.refuse("ownership conflict")
                finally:
                    child.communicate("")


class LifetimeTests(Fixture):
    def test_lost_heavy_client_retains_inputs_and_marker(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        for code in (137, -signal.SIGKILL):
            with self.subTest(code=code):
                class Child:
                    def wait(self, timeout=None):
                        return code
                real_popen = subprocess.Popen
                def launch(argv, **kwargs):
                    return Child() if argv[0] == "heavy" else real_popen(argv, **kwargs)
                with patch.object(precheck.subprocess, "Popen", side_effect=launch):
                    self.assertEqual(self.run_tool(production_runner=True), 1)
                self.assertFalse(self.tool.record["restored"])
                self.assertEqual(self.tool.record["outcome"], "unresolved")
                self.assertEqual((self.repo / "unrelated/Cargo.toml").read_text(), "candidate\n")
                self.assertTrue((self.repo / ".git/precheck.unresolved").exists())
                self.assertIn("heavy --cancel TOKEN", self.tool.record["reconciliation"])
                # Fixture-only cleanup after the asserted unknown completion; no real child exists.
                (self.repo / ".git/precheck.unresolved").unlink()
                self.git("reset", "--hard", self.ancestor)
                self.git("switch", "original")

    def test_heavy_wait_exception_does_not_restore(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        class Child:
            def wait(self, timeout=None):
                raise OSError("lost wait")
        real_popen = subprocess.Popen
        def launch(argv, **kwargs):
            return Child() if argv[0] == "heavy" else real_popen(argv, **kwargs)
        with patch.object(precheck.subprocess, "Popen", side_effect=launch):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertFalse(self.tool.record["restored"])
        self.assertEqual((self.repo / "unrelated/Cargo.toml").read_text(), "candidate\n")

    def test_launch_error_is_infrastructure_and_restores(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        real_popen = subprocess.Popen
        def launch(argv, **kwargs):
            if argv[0] == "heavy":
                raise FileNotFoundError("no heavy")
            return real_popen(argv, **kwargs)
        with patch.object(precheck.subprocess, "Popen", side_effect=launch):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertEqual(self.tool.record["outcome"], "infrastructure-error")
        self.restored()

    def test_signal_drains_known_child_before_restoring(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        real_popen = subprocess.Popen
        waited = []
        def launch(argv, **kwargs):
            if argv[0] != "heavy":
                return real_popen(argv, **kwargs)
            owner = self.tool
            class Child:
                def wait(self, timeout=None):
                    os.kill(os.getpid(), signal.SIGTERM)
                    self_candidate = (owner.repo / "unrelated/Cargo.toml").read_text()
                    self_head = owner.git("rev-parse", "HEAD")
                    waited.append((self_candidate, self_head, owner.record["restored"]))
                    return 0
            return Child()
        with patch.object(precheck.subprocess, "Popen", side_effect=launch):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertEqual(waited, [("candidate\n", self.tool.base, False)])
        self.assertIn("child drained", self.tool.record["error"])
        self.restored()

    def test_hard_death_leaves_durable_marker_and_rejects_reuse(self):
        self.change("tip", "unrelated/Cargo.toml", "candidate\n")
        script = '''import importlib.machinery,importlib.util,os,sys
loader=importlib.machinery.SourceFileLoader("pc",sys.argv[1])
spec=importlib.util.spec_from_loader(loader.name,loader)
p=importlib.util.module_from_spec(spec);loader.exec_module(p)
mq,a,r=p.policy()
def runner(argv,**kwargs):
    print("prepared",flush=True)
    os.kill(os.getpid(),9)
tool=p.Precheck(sys.argv[2],"base","tip",mq=mq,always=[],read_by_tests=[],runner=runner,
    metadata=lambda repo: {},busy=lambda repo:"fixture")
tool.run()
'''
        # Use a full input to avoid needing a graph in the disposable process.
        self.change("tip", "Cargo.toml", "full\n")
        child = subprocess.run([sys.executable, "-I", "-B", "-c", script, str(DIRECTORY / "precheck"), str(self.repo)],
                               text=True, capture_output=True, stdin=subprocess.DEVNULL)
        self.assertEqual(child.returncode, -9, child.stderr)
        self.assertTrue((self.repo / ".git/precheck.unresolved").exists())
        self.assertEqual((self.repo / "unrelated/Cargo.toml").read_text(), "candidate\n")
        with self.assertRaisesRegex(RuntimeError, "unresolved"):
            self.run_tool()


class ObservationTests(unittest.TestCase):
    LISTING = ("fixture-heavy-reconcile.timer loaded active waiting Heavy reconciliation\n"
               "fixture-heavy-build.service loaded active running Heavy build\n"
               "fixture-heavy-wrapper.service loaded activating start Heavy wrapper\n"
               "unrelated.service loaded active running Unrelated service\n")

    def test_unit_observation_uses_working_directory(self):
        results = [subprocess.CompletedProcess([], 0, self.LISTING, ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/slot\nActiveState=active\n", "")]
        with patch.object(subprocess, "run", side_effect=results) as run:
            with self.assertRaisesRegex(RuntimeError, "live heavy.*build.service"):
                precheck.heavy_busy(Path("/slot"))
        self.assertIn("--type=service", run.call_args_list[0].args[0])
        self.assertEqual(run.call_args_list[1].args[0][-1], "fixture-heavy-build.service")
        self.assertEqual(run.call_count, 2)

    def test_timer_listing_without_services_needs_no_working_directory(self):
        listing = "fixture-heavy-reconcile.timer loaded active waiting Heavy reconciliation\n"
        with patch.object(subprocess, "run", return_value=subprocess.CompletedProcess([], 0, listing, "")) as run:
            self.assertIn("no active heavy service observed", precheck.heavy_busy(Path("/slot")))
        run.assert_called_once()
        self.assertIn("--type=service", run.call_args.args[0])

    def test_wrapper_in_slot_is_refused_after_unrelated_payload(self):
        results = [subprocess.CompletedProcess([], 0, self.LISTING, ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/other\nActiveState=active\n", ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/slot/child\nActiveState=activating\n", "")]
        with patch.object(subprocess, "run", side_effect=results) as run:
            with self.assertRaisesRegex(RuntimeError, "live heavy.*wrapper.service"):
                precheck.heavy_busy(Path("/slot"))
        self.assertEqual(run.call_count, 3)

    def test_unloaded_inactive_service_is_recorded_as_vanished(self):
        results = [subprocess.CompletedProcess([], 0, self.LISTING, ""),
                   subprocess.CompletedProcess([], 0, "LoadState=not-found\nActiveState=inactive\n", ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/other\nActiveState=active\n", "")]
        with patch.object(subprocess, "run", side_effect=results):
            observation = precheck.heavy_busy(Path("/slot"))
        self.assertIn("1 unloaded inactive service", observation)
        self.assertIn("slot ownership is required", observation)

    def test_vanished_payload_does_not_hide_live_wrapper(self):
        results = [subprocess.CompletedProcess([], 0, self.LISTING, ""),
                   subprocess.CompletedProcess([], 0, "LoadState=not-found\nActiveState=inactive\n", ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/slot\nActiveState=active\n", "")]
        with patch.object(subprocess, "run", side_effect=results):
            with self.assertRaisesRegex(RuntimeError, "live heavy.*wrapper.service"):
                precheck.heavy_busy(Path("/slot"))

    def test_failed_or_incomplete_inspection_is_not_quiescence(self):
        listing = subprocess.CompletedProcess([], 0, "fixture-heavy-build.service loaded active running\n", "")
        for code, body in ((1, "LoadState=not-found\nActiveState=inactive\n"),
                           (0, "WorkingDirectory=/slot\n"),
                           (0, "LoadState=not-found\nActiveState=active\n"),
                           (0, "LoadState=loaded\nWorkingDirectory=\nActiveState=active\n")):
            with self.subTest(code=code, body=body), patch.object(subprocess, "run", side_effect=[
                    listing, subprocess.CompletedProcess([], code, body, "inspection unavailable")]):
                with self.assertRaisesRegex(RuntimeError, "cannot inspect heavy unit"):
                    precheck.heavy_busy(Path("/slot"))

    def test_stopping_service_in_slot_remains_busy(self):
        results = [subprocess.CompletedProcess([], 0, self.LISTING, ""),
                   subprocess.CompletedProcess([], 0, "LoadState=loaded\nWorkingDirectory=/slot\nActiveState=deactivating\n", "")]
        with patch.object(subprocess, "run", side_effect=results):
            with self.assertRaisesRegex(RuntimeError, "live heavy"):
                precheck.heavy_busy(Path("/slot"))

    def test_missing_bus_is_explicitly_unavailable_not_quiescent(self):
        with patch.object(subprocess, "run", return_value=subprocess.CompletedProcess(
                [], 1, "", "Failed to connect to bus: No medium found")):
            self.assertIn("unavailable", precheck.heavy_busy(Path("/slot")))


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
