"""Queue policy and effects against disposable local repositories; no real posts."""

import argparse
from contextlib import redirect_stderr, redirect_stdout
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
from unittest.mock import Mock, patch


DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
LOADER = importlib.machinery.SourceFileLoader("mq", str(DIRECTORY / "mq"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
mq = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(mq)
RESTART_LOADER = importlib.machinery.SourceFileLoader("restart", str(DIRECTORY / "restart"))
RESTART_SPEC = importlib.util.spec_from_loader(RESTART_LOADER.name, RESTART_LOADER)
restart = importlib.util.module_from_spec(RESTART_SPEC)
RESTART_LOADER.exec_module(restart)


class StartupTests(unittest.TestCase):
    def test_missing_path_tools_refuse_before_any_effect(self):
        for module, argv in ((mq, ["mq", "run"]), (restart, ["restart"])):
            for missing in ("cargo", "heavy", "post", "git"):
                with self.subTest(module=module.__name__, missing=missing):
                    errors = io.StringIO()
                    with patch.object(sys, "argv", argv), redirect_stderr(errors), \
                            patch.object(module.shutil, "which", side_effect=lambda name: (
                                None if name == missing else "/fake/" + name)), \
                            patch.object(subprocess, "run") as run:
                        self.assertEqual(module.main(), 2)
                    self.assertIn(f"missing {missing}", errors.getvalue())
                    self.assertIn("bash -lc", errors.getvalue())
                    run.assert_not_called()


def copy_runtime(directory):
    directory.mkdir(parents=True, exist_ok=True)
    for name in restart.RUNTIME_FILES:
        (directory / name).write_bytes((DIRECTORY / name).read_bytes())


class RestartTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".restart-test-")
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name)
        self.repo = self.home / "src/fabrig/queue"
        self.repo.mkdir(parents=True, mode=0o700)
        self.repo.parent.chmod(0o700)
        (self.repo / ".git").mkdir(mode=0o700)
        self.stop = self.home / "src/fabrig/queue.stop"
        copy_runtime(self.repo / "tools/queue")

    def run_restart(self, *, ready=True, returncode=None, batch=8):
        calls = []

        def command(argv, repo, **ignored):
            calls.append(argv)
            if argv[1] in ("fetch", "switch"):
                self.assertTrue(self.stop.exists())
            return str(repo) if "--show-toplevel" in argv else "main-commit"

        def launch(argv, **kwargs):
            self.assertFalse(self.stop.exists())
            expected = ["python3", "-B", "-u", str(self.repo / "tools/queue/mq"), "run", "--repo", str(self.repo)]
            expected.extend(["--authority-dir", str(self.home / "src/fabrig/queue-authority"),
                             "--authority-fd", str(kwargs["pass_fds"][0]),
                             "--startup-fd", str(kwargs["pass_fds"][1])])
            os.write(kwargs["pass_fds"][1], b"READY\n")
            if batch != 8:
                expected.extend(["--batch", str(batch)])
            self.assertEqual(argv, expected)
            self.assertEqual(kwargs["cwd"], self.repo)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            self.assertTrue(kwargs["start_new_session"])
            self.assertIs(kwargs["stdout"], kwargs["stderr"])
            kwargs["stdout"].write("new output\n")
            return Mock(pid=42, returncode=returncode, poll=lambda: returncode)

        close_fd = os.close
        with patch.object(restart, "command", side_effect=command), \
                patch.object(restart, "queue_pid", return_value=41), \
                patch.object(os, "pidfd_open", return_value=123), \
                patch.object(os, "close", side_effect=lambda fd: None if fd == 123 else close_fd(fd)) as close, \
                patch.object(restart.select, "select", side_effect=lambda fds, *args: (
                    (fds if ready else [], [], []))), \
                patch.object(subprocess, "Popen", side_effect=launch) as popen, \
                patch.object(restart.time, "sleep") as sleep, redirect_stdout(io.StringIO()) as output:
            restart.restart(self.repo, 10, self.home, batch=batch)
        close.assert_any_call(123)
        sleep.assert_not_called()
        popen.assert_called_once()
        return calls, output.getvalue()

    def test_restart_waits_updates_detaches_and_appends(self):
        output = self.home / "src/fabrig/notes/queue.out"
        output.parent.mkdir()
        output.write_text("old output\n")
        calls, printed = self.run_restart()
        self.assertEqual(calls, [["git", "rev-parse", "--show-toplevel"], ["git", "fetch", "origin"],
                                 ["git", "switch", "--detach", "origin/main"],
                                 ["git", "rev-parse", "HEAD"]])
        self.assertEqual(output.read_text(), "old output\nnew output\n")
        self.assertEqual(output.stat().st_mode & 0o777, 0o600)
        self.assertIn("pid 41", printed)
        self.assertIn("pid 42, main main-commit", printed)

    def test_timeout_retains_stop_and_does_not_update(self):
        with self.assertRaisesRegex(RuntimeError, "timed out"):
            self.run_restart(ready=False)
        self.assertTrue(self.stop.exists())

    def test_nondefault_batch_survives_the_drain_and_launch(self):
        self.run_restart(batch=16)

    def test_invalid_batch_is_rejected_before_restart(self):
        for value in ("0", "-1", "1.5", "nan"):
            with self.subTest(value=value), patch.object(sys, "argv", ["restart", "--batch", value]), \
                    patch.object(restart, "restart") as effect, redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    restart.main()
                self.assertEqual(error.exception.code, 2)
                effect.assert_not_called()

    def test_child_early_exit_reports_log(self):
        with self.assertRaisesRegex(RuntimeError, "exited 2.*queue.out"):
            self.run_restart(returncode=2)

    def test_failed_update_retains_stop_and_does_not_spawn(self):
        with patch.object(restart, "command", side_effect=[str(self.repo), RuntimeError("fetch failed")]), \
                patch.object(restart, "queue_pid", return_value=41), \
                patch.object(os, "pidfd_open", side_effect=ProcessLookupError), \
                patch.object(subprocess, "Popen") as popen, redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(RuntimeError, "fetch failed"):
                restart.restart(self.repo, 10, self.home)
        self.assertTrue(self.stop.exists())
        popen.assert_not_called()

    def test_stop_unlink_directory_fsync_failure_restores_stop_before_launch(self):
        sync = restart.authority.durable.sync_directory
        failed, updated = [], []
        def command(argv, repo, **ignored):
            if argv[:2] == ["git", "fetch"]:
                self.assertTrue(self.stop.exists())
                updated.append(True)
            return str(repo)
        def fail_after_unlink(path):
            if updated and path == self.stop.parent and not self.stop.exists() and not failed:
                failed.append(True)
                raise OSError("stop directory fsync failed")
            sync(path)
        with patch.object(restart, "command", side_effect=command), \
                patch.object(restart, "queue_pid", return_value=None), \
                patch.object(restart.authority.durable, "sync_directory", side_effect=fail_after_unlink), \
                patch.object(subprocess, "Popen") as popen, redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(OSError, "stop directory fsync"):
                restart.restart(self.repo, 0, self.home)
        self.assertEqual(failed, [True])
        self.assertTrue(self.stop.exists())
        popen.assert_not_called()

    def test_lock_holder_is_preferred_to_pgrep(self):
        lock = self.repo.with_name("queue.queue.lock")
        lock.touch()
        info = lock.stat()
        record = f"1: FLOCK ADVISORY WRITE 41 {os.major(info.st_dev):x}:{os.minor(info.st_dev):x}:{info.st_ino} 0 EOF\n"
        with patch.object(Path, "read_text", return_value=record), patch.object(subprocess, "run") as run:
            self.assertEqual(restart.queue_pid(self.repo), 41)
        run.assert_not_called()

    def test_absent_holder_does_not_use_broad_process_fallback(self):
        with patch.object(subprocess, "run") as run:
            self.assertIsNone(restart.queue_pid(self.repo))
        run.assert_not_called()

    def test_wait_rejects_negative_and_nonfinite_values(self):
        for value in ("-1", "inf", "nan"):
            with self.subTest(value=value), self.assertRaises(argparse.ArgumentTypeError):
                restart.seconds(value)


def metadata(root=Path("/workspace")):
    names = ("core", "normal", "build", "dev", "transitive", "unrelated", "always", "nested")
    names += tuple(mq.lines(DIRECTORY / "always.txt"))
    packages = [{"id": name, "name": name, "version": "1.0.0", "source": None,
                 "manifest_path": str(root / name / "Cargo.toml")}
                for name in names]
    packages[7]["manifest_path"] = str(root / "core/examples/nested/Cargo.toml")
    edges = {"normal": ["core"], "build": ["core"], "dev": ["core"], "transitive": ["dev"]}
    return {"workspace_root": str(root), "workspace_members": list(names), "packages": packages,
            "resolve": {"nodes": [{"id": name, "dependencies": edges.get(name, [])}
                                   for name in names]}}


def lockfile(packages):
    return "version = 4\n" + "".join(
        "\n[[package]]\n" + "\n".join(f"{key} = {json.dumps(value)}" for key, value in package.items()) + "\n"
        for package in packages)


def resolved_metadata(root=Path("/workspace")):
    data = metadata(root)
    data["packages"] += [
        {"id": "external", "name": "external", "version": "2.0.0", "source": "registry+fixture"},
        {"id": "middle", "name": "middle", "version": "1.0.0", "source": "registry+fixture"}]
    data["resolve"]["nodes"][0]["dependencies"] = ["middle"]
    data["resolve"]["nodes"] += [{"id": "middle", "dependencies": ["external"]},
                                   {"id": "external", "dependencies": []}]
    return data


class SelectionTests(unittest.TestCase):
    def select(self, paths):
        return mq.selection(paths, metadata(), always=["always"], read_by_tests=["docs/contract/**"])

    def test_source_selects_owner_reverse_closure_and_always(self):
        self.assertEqual(self.select(["core/src/lib.rs"]),
                         {"core", "normal", "build", "dev", "transitive", "always"})

    def test_test_bench_example_and_build_script_select_owner(self):
        for path in ("tests/check.rs", "benches/speed.rs", "examples/demo.rs", "build.rs"):
            with self.subTest(path=path):
                self.assertEqual(self.select(["unrelated/" + path]), {"unrelated", "always"})

    def test_longest_manifest_directory_owns_the_change(self):
        self.assertEqual(self.select(["core/examples/nested/src/lib.rs"]), {"nested", "always"})

    def test_global_config_script_fixture_and_unknown_select_full(self):
        for path in ("Cargo.toml", ".config/nextest.toml", "rust-toolchain.toml", "rust-toolchain",
                     "rustfmt.toml", "core/clippy.toml", "core/src/rustfmt.toml",
                     "core/scripts/check.py", "core/fixtures/input", "outside/source.rs"):
            with self.subTest(path=path):
                self.assertEqual(self.select([path]), "full")

    def test_member_manifest_selects_owner_and_reverse_closure(self):
        self.assertEqual(self.select(["core/Cargo.toml"]),
                         {"core", "normal", "build", "dev", "transitive", "always"})

    def test_lock_change_reaches_workspace_through_external_dependencies(self):
        before = lockfile([{"name": "external", "version": "1.0.0", "source": "registry+fixture"}])
        after = lockfile([{"name": "external", "version": "2.0.0", "source": "registry+fixture"}])
        data = resolved_metadata()
        changed = mq.lock_changes(before, after, data)
        self.assertEqual(mq.selection(["Cargo.lock"], data, always=["always"], locked=changed),
                         {"core", "normal", "build", "dev", "transitive", "always"})

    def test_lock_entry_fields_select_and_checksum_and_order_do_not(self):
        base = {"name": "external", "version": "2.0.0", "source": "registry+fixture",
                "dependencies": ["one", "two"], "checksum": "before"}
        rows = [({"version": "1.0.0"}, {"external"}),
                ({"source": "git+fixture"}, {"external"}),
                ({"name": "other"}, {"external"}),
                ({"dependencies": ["one"]}, {"external"}),
                ({"checksum": "after", "dependencies": ["two", "one"]}, set())]
        for updates, expected in rows:
            with self.subTest(updates=updates):
                self.assertEqual(mq.lock_changes(lockfile([{**base, **updates}]), lockfile([base]),
                                                 resolved_metadata()), expected)

    def test_removed_dependency_selects_parent_and_added_package_selects_itself(self):
        rows = [([{"name": "core", "version": "1.0.0", "dependencies": ["removed"]},
                  {"name": "removed", "version": "1.0.0"}],
                 [{"name": "core", "version": "1.0.0"}], {"core"}),
                ([], [{"name": "unrelated", "version": "1.0.0"}], {"unrelated"})]
        for before, after, expected in rows:
            with self.subTest(after=after):
                self.assertEqual(mq.lock_changes(lockfile(before), lockfile(after), metadata()), expected)

    def test_duplicate_versions_and_sources_select_only_matching_node(self):
        data = resolved_metadata()
        data["packages"] += [{"id": "other-version", "name": "external", "version": "1.0.0",
                               "source": "registry+fixture"},
                              {"id": "other-source", "name": "external", "version": "2.0.0",
                               "source": "git+fixture"}]
        entry = {"name": "external", "version": "2.0.0", "source": "registry+fixture"}
        self.assertEqual(mq.lock_changes(lockfile([]), lockfile([entry]), data), {"external"})

    def test_lock_requires_comparison_and_unchanged_lock_keeps_always(self):
        with self.assertRaisesRegex(ValueError, "base and HEAD"):
            self.select(["Cargo.lock"])
        self.assertEqual(mq.selection(["Cargo.lock"], metadata(), always=["always"], locked=set()),
                         {"always"})

    def test_markdown_outside_test_inputs_is_docs(self):
        self.assertEqual(self.select(["README.md", "docs/guide.md"]), "docs")

    def test_tools_and_unread_markdown_contribute_no_packages(self):
        self.assertEqual(self.select(["tools/y.py", "agents/x.md", "core/README.md"]), "docs")
        self.assertEqual(self.select(["core/src/lib.rs", "agents/x.md", "tools/y.py"]),
                         {"core", "normal", "build", "dev", "transitive", "always"})

    def test_markdown_under_package_source_directories_selects_owner(self):
        for directory in ("src", "tests", "benches", "examples"):
            with self.subTest(directory=directory):
                self.assertEqual(self.select([f"unrelated/{directory}/input.md"]), {"unrelated", "always"})
        self.assertEqual(self.select(["core/examples/nested/src/input.md"]), {"nested", "always"})

    def test_markdown_read_by_tests_is_full(self):
        self.assertEqual(self.select(["docs/contract/store.md"]), "full")

    def test_mixed_source_and_read_markdown_is_full(self):
        self.assertEqual(self.select(["core/src/lib.rs", "docs/contract/store.md"]), "full")

    def test_root_package_and_directory_prefix_are_distinct(self):
        data = metadata()
        data["packages"][0]["manifest_path"] = "/workspace/Cargo.toml"
        self.assertIn("core", mq.selection(["src/lib.rs"], data, always=[]))
        self.assertEqual(self.select(["core-other/src/lib.rs"]), "full")

    def test_missing_always_package_is_reported(self):
        with self.assertRaisesRegex(ValueError, "missing workspace packages"):
            mq.selection(["core/src/lib.rs"], metadata(), always=["absent"])

    def test_default_read_globs_cover_the_known_markdown_inputs(self):
        paths = ("temper/docs/help/reference/commands.md", "jig/docs/design/protocol/store.md",
                 "smith/docs/design/shell.md", "smith/benchmarks/README.md",
                 "smith/benchmarks/tests/fixtures/seed/README.md",
                 "smith/benchmarks/tests/recorded/tasks/valid/seed/README.md",
                 "smith/benchmarks/tasks/fixtures/seed/README.md",
                 "skein/crates/skein-ui/src/tests/markdown/01.md", "skein/docs/design/forge-github.md")
        for path in paths:
            with self.subTest(path=path):
                self.assertEqual(mq.selection([path], metadata()), "full")


class CheckTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".mq-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.repo = self.directory / "caller-independent"
        self.repo.mkdir()
        mq.git(self.repo, "init", "-b", "fixture")
        mq.git(self.repo, "config", "user.name", "Queue fixture")
        mq.git(self.repo, "config", "user.email", "queue@example.test")
        mq.git(self.repo, "config", "commit.gpgsign", "false")
        baseline = self.repo / "jig/scripts/old-forms.txt"
        baseline.parent.mkdir(parents=True)
        baseline.write_text("10\n")
        mq.git(self.repo, "add", ".")
        mq.git(self.repo, "commit", "-m", "fixture baseline")

    def test_package_scope_and_ratchets(self):
        steps = mq.check_commands(self.repo, {"core", "normal"}, ["core/src/lib.rs", "skein/docs/x.md"],
                                  metadata=metadata(self.repo))
        self.assertEqual([name for name, _, _ in steps],
                         ["fmt", "clippy", "build", "default", "fuzzy"])
        self.assertEqual(steps[0][1], ["cargo", "fmt", "--check", "-p", "core"])
        for _, argv, _ in steps[1:5]:
            self.assertEqual(argv[:3], ["heavy", "--gate", "cargo"])
            self.assertEqual(argv[argv.index("-p"):argv.index("-p") + 4], ["-p", "core", "-p", "normal"])
        self.assertEqual(steps[1][1][-4:], ["--all-targets", "--", "-D", "warnings"])
        # nextest refuses --no-fail-fast with --no-run: the build step takes the run's scope only.
        self.assertEqual(steps[2][1], [arg for arg in steps[3][1] if arg != "--no-fail-fast"] + ["--no-run"])
        self.assertNotIn("--no-fail-fast", steps[2][1])
        self.assertIn("--no-fail-fast", steps[3][1])
        self.assertEqual(steps[4][1][-2:], ["--profile", "fuzzy"])
        ratchets = mq.ratchet_commands(self.repo, ["core/src/lib.rs", "skein/docs/x.md"])
        self.assertEqual(ratchets, [("ratchet:skein", ["python3", "-I", "-B",
                                                     str(self.repo / "skein/scripts/old_forms.py"), "--write"],
                                    self.repo / "skein")])

    def test_full_without_tools_keeps_all_ordinary_checks(self):
        for paths in (["jig/src/lib.rs"], ["agents/policy.md"], ["Cargo.toml"],
                      ["Cargo.lock"], [".config/nextest.toml"], []):
            with self.subTest(paths=paths), patch.object(
                    mq, "tools_commands", side_effect=AssertionError("tools discovery without tools change")):
                calls = []
                def runner(argv, **options):
                    calls.append(argv)
                    return subprocess.CompletedProcess(argv, 0)
                steps = mq.run_checks(self.repo, "full", paths, self.directory / "full.log", runner=runner)
                expected = [*mq.ratchet_commands(self.repo, paths),
                            *mq.check_commands(self.repo, "full", paths)]
                self.assertEqual([step["name"] for step in steps], [name for name, _, _ in expected])
                self.assertEqual(calls, [argv for _, argv, _ in expected])
                self.assertEqual([name for name, _, _ in expected][-5:],
                                 ["fmt", "clippy", "build", "default", "fuzzy"])

    def test_changed_tools_run_and_fail_closed_for_every_selection(self):
        commands = [("tools/test_red.py", ["python3", "-I", "-B", "tools/test_red.py"], self.repo),
                    ("tools/test_green.py", ["python3", "-I", "-B", "tools/test_green.py"], self.repo)]
        for selected in ("docs", {"core"}, "full"):
            with self.subTest(selected=selected), patch.object(mq, "tools_commands", return_value=commands):
                calls = []
                def runner(argv, **options):
                    calls.append(argv)
                    self.assertEqual(options["timeout"], 60)
                    return subprocess.CompletedProcess(argv, int("tools/test_red.py" in argv))
                steps = mq.run_checks(self.repo, selected, ["tools/docs/guide.md"],
                                      self.directory / "red.log", runner=runner)
                self.assertEqual(calls, [argv for _, argv, _ in commands])
                self.assertEqual([step["name"] for step in steps], ["tools"])
                self.assertFalse(steps[0]["green"])
                self.assertEqual([suite["returncode"] for suite in steps[0]["suites"]], [1, 0])

    def test_tools_path_changes_include_deletes_and_both_rename_sides(self):
        for operation in ("add", "edit", "delete", "rename-in", "rename-out"):
            with self.subTest(operation=operation):
                repo = self.directory / operation
                repo.mkdir()
                mq.git(repo, "init", "-b", "fixture")
                mq.git(repo, "config", "user.name", "Fixture")
                mq.git(repo, "config", "user.email", "fixture@example.invalid")
                (repo / "tools").mkdir()
                (repo / "tools/source.md").write_text("tool source\n")
                (repo / "outside.md").write_text("outside source\n")
                mq.git(repo, "add", ".")
                mq.git(repo, "commit", "-m", "base")
                base = mq.git(repo, "rev-parse", "HEAD").stdout.strip()
                if operation == "add":
                    (repo / "tools/new.md").write_text("new tool\n")
                elif operation == "edit":
                    (repo / "tools/source.md").write_text("edited tool\n")
                elif operation == "delete":
                    (repo / "tools/source.md").unlink()
                elif operation == "rename-in":
                    mq.git(repo, "mv", "outside.md", "tools/moved.md")
                else:
                    mq.git(repo, "mv", "tools/source.md", "moved.md")
                mq.git(repo, "add", "-A")
                mq.git(repo, "commit", "-m", operation)
                paths = mq.changed_paths(repo, base=base)
                expected = {"add": ["tools/new.md"], "edit": ["tools/source.md"],
                            "delete": ["tools/source.md"], "rename-in": ["outside.md", "tools/moved.md"],
                            "rename-out": ["moved.md", "tools/source.md"]}
                self.assertEqual(paths, expected[operation])
                self.assertTrue(mq.tools_required(paths))
                self.assertFalse(mq.tools_required(["toolsmith/source.md", "docs/tools/guide.md"]))

    def test_full_formats_workspace_and_docs_only_runs_ratchets(self):
        self.assertEqual(mq.check_commands(self.repo, "full", [])[0][1], ["cargo", "fmt", "--all", "--check"])
        for _, argv, _ in mq.check_commands(self.repo, "full", [])[1:]:
            self.assertIn("--workspace", argv)
            self.assertNotIn("-p", argv)
        self.assertEqual(mq.check_commands(self.repo, "docs", ["jig/README.md"]), [])
        self.assertEqual([name for name, _, _ in mq.ratchet_commands(self.repo, ["jig/README.md"])],
                         ["ratchet:jig"])

    def test_fmt_uses_only_changed_rust_owners_sorted_once(self):
        paths = ["unrelated/src/lib.rs", "core/src/lib.rs", "core/tests/check.rs",
                 "core/examples/nested/src/lib.rs"]
        steps = mq.check_commands(self.repo, {"core", "normal", "nested", "unrelated", "always"}, paths,
                                  metadata=metadata(self.repo))
        self.assertEqual(steps[0][1], ["cargo", "fmt", "--check", "-p", "core", "-p", "nested",
                                      "-p", "unrelated"])

    def test_no_rust_changes_skip_fmt_and_ownership_metadata(self):
        for selected, paths in (({"core"}, ["Cargo.lock"]), ({"core"}, ["core/Cargo.toml"]),
                                ("docs", ["README.md"]), ({"core"}, ["core/tests/data.json"]),
                                (set(), ["Cargo.lock"])):
            with self.subTest(paths=paths, selected=selected), patch.object(mq, "load_workspace_metadata") as loader:
                self.assertNotIn("fmt", [name for name, _, _ in mq.check_commands(self.repo, selected, paths)])
                loader.assert_not_called()

    def test_fmt_loads_ownership_from_no_deps_metadata(self):
        with patch.object(mq, "load_workspace_metadata", return_value=metadata(self.repo)) as loader:
            steps = mq.check_commands(self.repo, {"core"}, ["core/src/lib.rs"])
        loader.assert_called_once_with(self.repo)
        self.assertEqual(steps[0][1], ["cargo", "fmt", "--check", "-p", "core"])

    def test_empty_nextest_is_green_and_steps_have_durations(self):
        calls = []

        def runner(argv, **kwargs):
            calls.append(argv)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            return subprocess.CompletedProcess(argv, 4 if "nextest" in argv else 0)

        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory:
            log = Path(directory) / "checks.log"
            times = iter(range(8))
            steps = mq.run_checks(self.repo, {"core"}, [], log, runner=runner, clock=lambda: next(times))
            self.assertEqual(len(calls), 4)
            self.assertTrue(all(step["green"] and step["duration"] == 1 for step in steps))
            self.assertEqual(log.stat().st_mode & 0o777, 0o600)

    def test_build_default_and_fuzzy_have_independent_durations(self):
        times = iter([0, 2, 10, 18, 20, 23, 30, 35])
        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory:
            steps = mq.run_checks(self.repo, {"core"}, [], Path(directory) / "checks.log",
                                  runner=lambda *args, **kwargs: subprocess.CompletedProcess(args, 0),
                                  clock=lambda: next(times))
        self.assertEqual([(step["name"], step["duration"]) for step in steps],
                         [("clippy", 2), ("build", 8), ("default", 3), ("fuzzy", 5)])

    def test_failure_stops_checks_and_four_is_not_green_for_fmt(self):
        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory, \
                patch.object(mq, "tools_commands", return_value=[]):
            steps = mq.run_checks(self.repo, "full", [], Path(directory) / "checks.log",
                                  runner=lambda *args, **kwargs: subprocess.CompletedProcess(args, 4))
            self.assertEqual([step["name"] for step in steps], ["fmt"])
            self.assertFalse(steps[-1]["green"])

    def test_tools_timeout_is_red_and_other_suites_still_run(self):
        commands = [("tools/test_first.py", ["python3", "-I", "-B", "tools/test_first.py"], self.repo),
                    ("tools/test-second", ["bash", "tools/test-second"], self.repo)]
        calls = []

        def runner(argv, **kwargs):
            calls.append(argv)
            self.assertEqual(kwargs["timeout"], mq.TOOLS_TIMEOUT)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            self.assertEqual(kwargs["cwd"], self.repo)
            if argv[0] == "python3":
                raise subprocess.TimeoutExpired(argv, mq.TOOLS_TIMEOUT)
            return subprocess.CompletedProcess(argv, 0)

        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory, \
                patch.object(mq, "tools_commands", return_value=commands):
            log = Path(directory) / "checks.log"
            steps = mq.run_checks(self.repo, "full", ["tools/changed.py"], log, runner=runner)
            self.assertIn(f"Timed out after {mq.TOOLS_TIMEOUT} seconds", log.read_text())
        self.assertEqual(calls, [argv for _, argv, _ in commands])
        self.assertEqual([step["name"] for step in steps], ["tools"])
        self.assertFalse(steps[0]["green"])
        self.assertEqual([suite["returncode"] for suite in steps[0]["suites"]], [124, 0])

    def test_tools_exit_four_is_red_and_os_errors_are_logged(self):
        commands = [("tools/test_first.py", ["python3", "-I", "-B", "tools/test_first.py"], self.repo)]
        for result in (subprocess.CompletedProcess([], 4), FileNotFoundError("missing interpreter")):
            with self.subTest(result=result), \
                    tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory, \
                    patch.object(mq, "tools_commands", return_value=commands):
                log = Path(directory) / "checks.log"
                runner = Mock(side_effect=result) if isinstance(result, OSError) else Mock(return_value=result)
                steps = mq.run_checks(self.repo, "docs", ["tools/y.py"], log, runner=runner)
                self.assertFalse(steps[0]["green"])
                self.assertEqual(steps[0]["suites"][0]["returncode"], 127 if isinstance(result, OSError) else 4)
                if isinstance(result, OSError):
                    self.assertIn("missing interpreter", log.read_text())

    def test_tools_and_docs_keep_touched_tree_ratchets_without_cargo(self):
        before = mq.git(self.repo, "rev-parse", "HEAD").stdout.strip()
        (self.repo / "jig/scripts/old-forms.txt").write_text("9\n")
        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory, \
                patch.object(mq, "tools_commands", return_value=[]):
            steps = mq.run_checks(self.repo, "docs", ["tools/x.py", "jig/README.md"],
                                  Path(directory) / "checks.log",
                                  runner=lambda *a, **kw: subprocess.CompletedProcess(a, 0))
        self.assertEqual([step["name"] for step in steps], ["ratchet:jig", "tools"])
        self.assertEqual(steps[0]["command"][-1], "--write")
        self.assertEqual(mq.git(self.repo, "show", "HEAD:jig/scripts/old-forms.txt").stdout, "9\n")
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s").stdout.strip(),
                         "Lower jig old-form baselines")
        self.assertEqual(mq.git(self.repo, "rev-parse", "HEAD^").stdout.strip(), before)
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def test_check_only_ratchets_never_stage_or_commit_and_observe_each_step(self):
        observed = []
        outcomes = []
        with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory, \
                patch.object(mq, "git", side_effect=AssertionError("check-only mutated git")):
            steps = mq.run_checks(self.repo, "docs", ["jig/README.md"], Path(directory) / "checks.log",
                                  runner=lambda *a, **kw: subprocess.CompletedProcess(a, 0),
                                  check_only=True, observer=observed.append, outcomes=outcomes)
        self.assertIs(steps, outcomes)
        self.assertEqual(observed, steps)
        self.assertEqual(steps[0]["command"][-1], str(self.repo / "jig/scripts/old_forms.py"))
        self.assertEqual(steps[0]["cwd"], str(self.repo / "jig"))

    def test_only_heavy_exit_two_with_disk_floor_text_is_a_refusal(self):
        for code, output, refused in ((2, "builder: 39 GB below the disk floor", True),
                                      (1, "below the disk floor", False),
                                      (2, "test failed", False)):
            with self.subTest(code=code, output=output):
                def runner(argv, **kwargs):
                    heavy = argv[0] == "heavy"
                    if heavy:
                        kwargs["stdout"].write(output)
                    return subprocess.CompletedProcess(argv, code if heavy else 0)

                with tempfile.TemporaryDirectory(dir=self.directory, prefix=".mq-test-") as directory:
                    steps = mq.run_checks(self.repo, "full", [], Path(directory) / "checks.log", runner=runner)
                self.assertEqual("error" in steps[-1], refused)
                self.assertEqual([step["name"] for step in steps], ["fmt", "clippy"])


class CleanupTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".mq-test-")
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        self.deps = self.repo / "target/debug/deps"
        self.deps.mkdir(parents=True)
        self.batch_start = 10_000_000_000

    def artifact(self, name, *, mtime=1_000_000_000, executable=True):
        path = self.deps / name
        path.write_bytes(b"test binary")
        path.chmod(0o700 if executable else 0o600)
        os.utime(path, ns=(mtime, mtime))
        return path

    def test_prunes_oldest_first_until_floor_plus_forty_is_free(self):
        oldest = self.artifact("oldest", mtime=1_000_000_000)
        middle = self.artifact("middle", mtime=2_000_000_000)
        newest = self.artifact("newest", mtime=3_000_000_000)
        with patch.object(mq, "free_gb", side_effect=[84, 84, 85]):
            report = mq.prune_test_executables(self.repo, 45, self.batch_start)
        self.assertFalse(oldest.exists())
        self.assertTrue(middle.exists() and newest.exists())
        self.assertEqual(report, {"count": 1, "gb": 11 / mq.GB})

    def test_protects_libraries_nonexecutables_and_batch_outputs(self):
        protected = [self.artifact("fresh", mtime=self.batch_start),
                     self.artifact("newer", mtime=self.batch_start + 1),
                     self.artifact("ordinary", executable=False)]
        protected += [self.artifact("library" + suffix) for suffix in (".rlib", ".rmeta", ".so", ".d")]
        old = self.artifact("old")
        (self.deps / "symlink").symlink_to(old)
        (self.deps / "directory").mkdir()
        with patch.object(mq, "free_gb", return_value=0):
            report = mq.prune_test_executables(self.repo, 45, self.batch_start)
        self.assertEqual(report["count"], 1)
        self.assertTrue(all(path.exists() for path in protected))
        self.assertTrue((self.deps / "symlink").is_symlink())
        self.assertTrue((self.deps / "directory").is_dir())

    def test_no_cleanup_at_space_goal(self):
        old = self.artifact("old")
        with patch.object(mq, "free_gb", return_value=85):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start),
                             {"count": 0, "gb": 0.0})
        self.assertTrue(old.exists())

    def test_rewritten_candidate_is_preserved(self):
        old = self.artifact("old")
        calls = []

        def space(repo):
            calls.append(repo)
            if len(calls) == 2:
                os.utime(old, ns=(self.batch_start, self.batch_start))
            return 0

        with patch.object(mq, "free_gb", side_effect=space):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start)["count"], 0)
        self.assertTrue(old.exists())

    def test_symlinked_deps_directory_is_preserved(self):
        old = self.artifact("old")
        moved = self.deps.with_name("other")
        self.deps.rename(moved)
        self.deps.symlink_to(moved, target_is_directory=True)
        with patch.object(mq, "free_gb", return_value=0):
            self.assertEqual(mq.prune_test_executables(self.repo, 45, self.batch_start)["count"], 0)
        self.assertTrue(old.exists())


class BisectTests(unittest.TestCase):
    def test_one_red_is_reported_and_green_groups_land_in_order(self):
        landed = []

        def check(group):
            if "red" in group:
                return False
            landed.extend(group)
            return True

        green, red = mq.bisect(["a", "b", "red", "c", "d"], check)
        self.assertEqual(green, ["a", "b", "c", "d"])
        self.assertEqual(landed, green)
        self.assertEqual(red, ["red"])

    def test_single_red_is_rejected_once(self):
        calls = []
        self.assertEqual(mq.bisect(["red"], lambda group: calls.append(group) or False), ([], ["red"]))
        self.assertEqual(calls, [["red"]])

    def test_interacting_green_branches_are_checked_after_previous_landing(self):
        landed = []

        def check(group):
            if "a" in landed and "b" in group or {"a", "b"}.issubset(group):
                return False
            landed.extend(group)
            return True

        self.assertEqual(mq.bisect(["a", "b"], check), (["a"], ["b"]))


class RepositoryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".mq-test-")
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.repo = self.directory / "queue"
        self.origin = self.directory / "origin.git"
        self.repo.mkdir()
        mq.git(self.repo, "init", "-b", "work")
        mq.git(self.repo, "config", "user.name", "Queue fixture")
        mq.git(self.repo, "config", "user.email", "queue@example.test")
        mq.git(self.repo, "config", "commit.gpgsign", "false")
        self.write("shared", "base\n")
        self.commit("base")
        self.base = self.tip()
        mq.git(self.repo, "init", "--bare", str(self.origin))
        mq.git(self.repo, "remote", "add", "origin", str(self.origin))
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        self.posts = []
        self.checks = []

    def write(self, path, text):
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)

    def commit(self, subject):
        mq.git(self.repo, "add", ".")
        mq.git(self.repo, "commit", "-m", subject)

    def tip(self):
        return mq.git(self.repo, "rev-parse", "HEAD").stdout.strip()

    def branch(self, task, path=None, text="change\n", *, publish=False):
        # Only fixture repositories receive ready refs; the enclosing slot never does.
        mq.git(self.repo, "switch", "--detach", self.base)
        self.write(path or task, text)
        self.commit(task)
        branch = {"name": "ready/coord/" + task, "tip": self.tip(), "timestamp": 1}
        if publish:
            mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        return branch

    def checker(self, repo, selected, paths, log):
        self.checks.append((selected, paths, self.tip()))
        return [{"name": "fixture", "green": not (repo / "red").exists(), "duration": 0.0}]

    def ratchet_checker(self, repo, selected, paths, log):
        def runner(argv, **kwargs):
            if argv[0] == "python3":
                return subprocess.run(argv, **kwargs)
            self.checks.append(argv)
            self.assertIn(argv[0], ("cargo", "heavy"))
            return subprocess.CompletedProcess(argv, int((repo / "red").exists()))

        return mq.run_checks(repo, selected, paths, log, runner=runner)

    def poster(self, address, subject, body):
        self.assertLess(len(body.splitlines()), 15)
        self.posts.append((address, subject, body))
        return {"sent": True, "returncode": 3}

    def queue(self, **kwargs):
        kwargs.setdefault("disk_floor", 0)
        return mq.Queue(self.repo, state_path=self.directory / "state.json",
                        writer_guard=lambda: mq.authority.checkout_guard(self.repo),
                        log_path=self.directory / "queue.jsonl", checker=self.checker,
                        poster=self.poster, metadata_loader=lambda repo: metadata(repo), **kwargs)

    def records(self, *, effects=False):
        rows = [json.loads(line) for line in (self.directory / "queue.jsonl").read_text().splitlines()]
        return rows if effects else [row for row in rows if row.get("event") != "publication-effects"]

    def run_cycle(self, queue):
        with redirect_stdout(io.StringIO()):
            return queue.cycle()

    def test_conflict_is_skipped_and_the_rest_are_merged(self):
        first = self.branch("first", "shared", "first\n")
        conflict = self.branch("conflict", "shared", "other\n")
        last = self.branch("last")
        merged, conflicts = mq.build_batch(self.repo, [first, conflict, last])
        self.assertEqual([b["name"] for b in merged], [first["name"], last["name"]])
        self.assertEqual([b["name"] for b in conflicts], [conflict["name"]])
        self.assertEqual((self.repo / "shared").read_text(), "first\n")
        self.assertTrue((self.repo / "last").exists())
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def ratchet_base(self, trees=("skein",)):
        for tree in trees:
            self.write(f"{tree}/scripts/old-forms.txt", "3\n")
            self.write(f"{tree}/src/a.rs", "old old\n")
            self.write(f"{tree}/src/b.rs", "old\n")
            self.write(f"{tree}/scripts/old_forms.py", r"""from pathlib import Path
import sys
assert sys.argv[1:] == ['--write']
baseline = Path('scripts/old-forms.txt')
count = sum(p.read_text().count('old') for p in Path('src').glob('*.rs'))
if count > int(baseline.read_text()):
    print('count rose above HEAD baseline')
    sys.exit(1)
baseline.write_text(str(count) + '\n')
""")
        self.commit("ratchet fixture")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")

    def ratchet_branch(self, task, source, baseline, trees=("skein",), *, publish=False):
        mq.git(self.repo, "switch", "--detach", self.base)
        for tree in trees:
            self.write(f"{tree}/src/{source}.rs", "")
            self.write(f"{tree}/scripts/old-forms.txt", f"{baseline}\n")
        self.commit(task)
        branch = {"name": "ready/coord/" + task, "tip": self.tip(), "timestamp": 1}
        if publish:
            mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        return branch

    def test_ratchet_only_conflicts_recount_and_amend_each_tree(self):
        trees = ("skein", "smith")
        self.ratchet_base(trees)
        first = self.ratchet_branch("a", "a", 1, trees)
        second = self.ratchet_branch("b", "b", 2, trees)
        merged, conflicts = mq.build_batch(self.repo, [first, second])
        self.assertEqual(conflicts, [])
        self.assertEqual([branch["name"] for branch in merged], [first["name"], second["name"]])
        for tree in trees:
            self.assertEqual((self.repo / tree / "scripts/old-forms.txt").read_text(), "0\n")
            self.assertEqual(mq.git(self.repo, "show", f"HEAD:{tree}/scripts/old-forms.txt").stdout, "0\n")
        self.assertEqual(len(mq.git(self.repo, "show", "-s", "--format=%P").stdout.split()), 2)
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s").stdout.strip(),
                         "Merge ready/coord/b")
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def test_lower_count_is_committed_on_top_of_merges_and_pushed(self):
        self.ratchet_base()
        branch = self.branch("lower", "skein/src/a.rs", "", publish=True)
        queue = self.queue()
        queue.checker = self.ratchet_checker
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["outcome"], "green")
        self.assertEqual(mq.git(self.repo, "show", "origin/main:skein/scripts/old-forms.txt").stdout, "1\n")
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s", "origin/main").stdout.strip(),
                         "Lower skein old-form baselines")
        merge = mq.git(self.repo, "rev-parse", "origin/main^").stdout.strip()
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s", merge).stdout.strip(),
                         "Merge " + branch["name"])
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%an <%ae>", "origin/main").stdout.strip(),
                         "Queue fixture <queue@example.test>")
        self.assertEqual(record["steps"][0]["name"], "ratchet:skein")
        self.assertEqual(record["steps"][0]["command"][-1], "--write")
        self.assertGreaterEqual(record["steps"][0]["duration"], 0)
        self.assertTrue(any(f"Merge commit: {merge}" in body for _, _, body in self.posts))
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin", "refs/heads/main").stdout.split()[0],
                         record["commit"])
        self.assertEqual(mq.git(self.repo, "status", "--porcelain").stdout, "")

    def test_branches_lowering_same_baseline_land_combined_count(self):
        self.ratchet_base()
        first = self.branch("a", "skein/src/a.rs", "old\n")
        self.write("skein/scripts/old-forms.txt", "2\n")
        self.commit("lower first baseline")
        first["tip"] = self.tip()
        second = self.ratchet_branch("b", "b", 2)
        queue = self.queue()
        queue.checker = self.ratchet_checker
        self.assertTrue(queue.attempt([first, second]))
        self.assertEqual(mq.git(self.repo, "show", "origin/main:skein/scripts/old-forms.txt").stdout, "1\n")
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s", "origin/main").stdout.strip(),
                         "Lower skein old-form baselines")
        subjects = mq.git(self.repo, "log", "--first-parent", "--format=%s", f"{self.base}..origin/main").stdout.splitlines()
        self.assertEqual(subjects, ["Lower skein old-form baselines", "Merge ready/coord/b", "Merge ready/coord/a"])

    def test_count_rise_rejects_single_batch_before_other_checks(self):
        self.ratchet_base()
        branch = self.branch("raise", "skein/src/a.rs", "old old old\n", publish=True)
        queue = self.queue()
        queue.checker = self.ratchet_checker
        with patch.object(mq, "tools_commands", side_effect=AssertionError("tools ran")), \
                patch.object(mq, "check_commands", side_effect=AssertionError("cargo ran")):
            self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["outcome"], "red")
        self.assertEqual([step["name"] for step in record["steps"]], ["ratchet:skein"])
        self.assertEqual(record["steps"][0]["returncode"], 1)
        self.assertIn("count rose above HEAD baseline", Path(record["log"]).read_text())
        self.assertTrue(any("failing step: ratchet:skein" in body for _, _, body in self.posts))
        self.assertEqual(queue.state["tried"][branch["name"]], branch["tip"])
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)
        self.assertEqual(self.checks, [])

    def test_batch_touching_no_tree_creates_no_baseline_commit(self):
        branch = self.branch("docs", "guide.md", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = self.ratchet_checker
        self.run_cycle(queue)
        self.assertEqual(self.records()[0]["steps"], [])
        self.assertEqual(mq.git(self.repo, "show", "-s", "--format=%s", "origin/main").stdout.strip(),
                         "Merge " + branch["name"])
        self.assertEqual(mq.git(self.repo, "ls-tree", "-r", "--name-only", "origin/main").stdout.splitlines(),
                         ["guide.md", "shared"])

    def test_docs_only_recounts_only_touched_trees_and_commits_slack(self):
        self.ratchet_base(("skein", "smith", "jig"))
        self.write("skein/src/a.rs", "")
        self.write("smith/src/a.rs", "")
        self.write("jig/src/a.rs", "")
        self.commit("slack on main")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        branch = self.branch("docs", "skein/README.md", publish=True)
        self.write("smith/README.md", "docs\n")
        self.commit("second tree docs")
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = self.ratchet_checker
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["selection"], "docs")
        self.assertEqual([step["name"] for step in record["steps"]], ["ratchet:skein", "ratchet:smith"])
        self.assertTrue(all(step["command"][-1] == "--write" for step in record["steps"]))
        self.assertEqual(mq.git(self.repo, "show", "origin/main:jig/scripts/old-forms.txt").stdout, "3\n")
        for tree in ("skein", "smith"):
            self.assertEqual(mq.git(self.repo, "show", f"origin/main:{tree}/scripts/old-forms.txt").stdout, "1\n")
        self.assertEqual(self.checks, [])

    def test_bisection_recomputes_baselines_from_each_halfs_merges(self):
        self.ratchet_base()
        self.branch("a", "skein/src/a.rs", "", publish=True)
        self.branch("b", "skein/src/b.rs", "", publish=True)
        self.branch("red", publish=True)
        queue = self.queue()
        queue.checker = self.ratchet_checker
        self.run_cycle(queue)
        records = self.records()
        self.assertEqual([record["outcome"] for record in records], ["red", "green", "red", "green", "red"])
        green = [record for record in records if record["outcome"] == "green"]
        self.assertEqual([mq.git(self.repo, "show", f"{record['commit']}:skein/scripts/old-forms.txt").stdout
                          for record in green], ["1\n", "0\n"])
        self.assertEqual(mq.git(self.repo, "show", "origin/main:skein/scripts/old-forms.txt").stdout, "0\n")
        self.assertNotIn("red", mq.git(self.repo, "ls-tree", "--name-only", "origin/main").stdout.splitlines())

    def test_ratchet_rise_rejects_branch_and_restores_batch_so_far(self):
        self.ratchet_base()
        first = self.ratchet_branch("a", "a", 1, publish=True)
        mq.git(self.repo, "switch", "--detach", self.base)
        self.write("skein/scripts/old-forms.txt", "2\n")
        self.write("skein/src/b.rs", "old old\n")
        self.commit("raise combined count")
        second = {"name": "ready/coord/b", "tip": self.tip(), "timestamp": 1}
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{second['name']}")
        queue = self.queue()
        self.run_cycle(queue)
        self.assertEqual((self.repo / "skein/src/a.rs").read_text(), "")
        self.assertEqual((self.repo / "skein/src/b.rs").read_text(), "old\n")
        record = self.records()[0]
        self.assertEqual(record["conflicts"][0]["step"], "ratchet:skein")
        self.assertIn("count rose above HEAD baseline", record["conflicts"][0]["error"])
        self.assertTrue(any("failing step: ratchet:skein" in body for _, _, body in self.posts))
        self.assertEqual(queue.state["tried"][second["name"]], second["tip"])
        self.assertEqual(record["branches"][0]["tip"], first["tip"])

    def test_mixed_ratchet_and_source_conflict_is_aborted(self):
        self.ratchet_base()
        first = self.ratchet_branch("a", "a", 1)
        second = self.ratchet_branch("b", "b", 2)
        self.write("skein/src/a.rs", "other\n")
        self.commit("source conflict")
        second["tip"] = self.tip()
        with patch.object(mq, "merge_ratchets", side_effect=AssertionError("mixed conflict")):
            merged, conflicts = mq.build_batch(self.repo, [first, second])
        self.assertEqual(len(merged), 1)
        self.assertNotIn("step", conflicts[0])
        self.assertEqual((self.repo / "skein/scripts/old-forms.txt").read_text(), "1\n")

    def test_merge_commits_name_the_pinned_branch(self):
        branches = [self.branch("first"), self.branch("last")]
        merged, _ = mq.build_batch(self.repo, branches)
        self.assertEqual([mq.git(self.repo, "show", "-s", "--format=%s", branch["commit"]).stdout.strip()
                          for branch in merged], ["Merge ready/coord/first", "Merge ready/coord/last"])

    def test_candidates_are_pinned_ordered_and_skip_only_the_tried_tip(self):
        a = self.branch("a")
        b = self.branch("b")
        for branch in (a, b):
            mq.git(self.repo, "update-ref", "refs/remotes/origin/" + branch["name"], branch["tip"])
        listed = mq.list_candidates(self.repo, {a["name"]: a["tip"]})
        self.assertEqual([branch["name"] for branch in listed], [b["name"]])
        self.assertEqual(listed[0]["tip"], b["tip"])
        self.assertEqual([branch["name"] for branch in mq.list_candidates(self.repo, {a["name"]: "old"})],
                         [a["name"], b["name"]])

    def test_cycle_bisects_red_and_lands_green_without_retrying_red_tip(self):
        for task in ("a", "red", "z"):
            self.branch(task, publish=True)
        queue = self.queue()
        self.run_cycle(queue)
        mq.git(self.repo, "switch", "--detach", "origin/main")
        self.assertTrue((self.repo / "a").exists() and (self.repo / "z").exists())
        self.assertFalse((self.repo / "red").exists())
        self.assertEqual(mq.git(self.repo, "ls-remote", "--heads", "origin", "refs/heads/ready/*").stdout.split()[1],
                         "refs/heads/ready/coord/red")
        self.assertEqual(len([post for post in self.posts if "rejected" in post[1]]), 1)
        self.assertEqual(len([post for post in self.posts if "landed" in post[1]]), 2)
        checked = len(self.checks)
        self.assertFalse(self.run_cycle(self.queue()))
        self.assertEqual(len(self.checks), checked)
        records = self.records()
        self.assertTrue(all(record["start"] and record["end"] and record["steps"] for record in records))
        self.assertEqual(sum(record["outcome"] == "green" for record in records), 2)

    def test_conflict_is_reported_and_other_branch_lands(self):
        self.branch("a", "shared", "one\n", publish=True)
        self.branch("b", "shared", "two\n", publish=True)
        self.branch("z", publish=True)
        self.run_cycle(self.queue())
        self.assertEqual(len([post for post in self.posts if "rejected" in post[1]]), 1)
        self.assertTrue(any("merge conflict" in body for _, _, body in self.posts))
        self.assertTrue(any(record["conflicts"] for record in self.records()))
        self.assertTrue((self.repo / "z").exists())

    def test_dry_checks_and_restores_checkout_without_remote_posts_or_state_changes(self):
        branch = self.branch("a", publish=True)
        mq.git(self.repo, "switch", "work")
        before = mq.git(self.repo, "ls-remote", "origin").stdout
        queue = self.queue(dry=True)
        self.run_cycle(queue)
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin").stdout, before)
        self.assertEqual(mq.git(self.repo, "branch", "--show-current").stdout.strip(), "work")
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(self.posts, [])
        self.assertFalse(queue.state_path.exists())
        self.assertEqual(len(self.checks), 1)
        self.assertEqual(self.records()[0]["branches"][0]["tip"], branch["tip"])

    def test_plan_has_no_merge_push_post_or_state_write(self):
        self.branch("a", publish=True)
        mq.git(self.repo, "switch", "work")
        before = mq.git(self.repo, "ls-remote", "origin").stdout
        output = io.StringIO()
        queue = self.queue()
        with redirect_stdout(output):
            queue.plan()
        self.assertEqual(json.loads(output.getvalue())["selection"], "full")
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin").stdout, before)
        self.assertEqual(self.posts, [])
        self.assertEqual(self.checks, [])
        self.assertFalse(queue.state_path.exists())

    def test_plan_conservatively_previews_unmerged_lock_or_manifest(self):
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        with patch.object(queue, "metadata_loader", side_effect=AssertionError("unmerged metadata")):
            for path in ("Cargo.lock", "core/Cargo.toml"):
                with self.subTest(path=path):
                    self.assertEqual(queue.choose([path], preview=True), "full")

    def test_tools_manifest_preview_needs_no_cargo_graph(self):
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        with patch.object(queue, "metadata_loader", side_effect=AssertionError("tools metadata")):
            self.assertEqual(queue.choose(["tools/fixture/Cargo.toml"], preview=True), "docs")

    def test_post_failure_is_logged_and_does_not_fail_landing(self):
        self.branch("a", publish=True)
        queue = self.queue()
        queue.poster = lambda *args: {"sent": False, "returncode": 2, "output": "not delivered"}
        self.run_cycle(queue)
        record = self.records(effects=True)[-1]
        self.assertEqual(record["outcome"], "green")
        self.assertEqual(record["posts"][0]["returncode"], 2)

    def test_rejected_branch_is_retried_when_its_tip_changes(self):
        branch = self.branch("red", publish=True)
        self.run_cycle(self.queue())
        self.assertEqual(len(self.checks), 1)
        mq.git(self.repo, "switch", "--detach", branch["tip"])
        mq.git(self.repo, "rm", "red")
        self.write("fixed", "fixed\n")
        self.commit("remove failing input")
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        self.run_cycle(self.queue())
        self.assertEqual(len(self.checks), 2)
        self.assertTrue((self.repo / "fixed").exists())
        self.assertEqual(self.records()[-1]["outcome"], "green")

    def test_selected_check_and_periodic_full_are_applied_to_real_batches(self):
        self.branch("a", "core/src/lib.rs", publish=True)
        queue = self.queue(now=lambda: 10000, full_every=2)
        queue.state["last_full"] = 9999
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0],
                         {"core", "normal", "build", "dev", "transitive"} | set(mq.lines(DIRECTORY / "always.txt")))
        self.branch("z", "unrelated/src/lib.rs", publish=True)
        self.run_cycle(queue)
        self.assertEqual(self.checks[1][0], "full")
        self.assertEqual(queue.state["batches_since_full"], 0)

    def test_docs_batch_does_not_need_cargo_metadata(self):
        self.branch("a", "guide.md", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999

        def no_metadata(repo):
            self.fail("docs-only selection should not invoke cargo metadata")

        queue.metadata_loader = no_metadata
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0], "docs")

    def test_docs_batch_without_checks_is_green_and_lands(self):
        self.branch("docs", "README.md", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = mq.run_checks
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual((record["outcome"], record["steps"]), ("green", []))
        self.assertTrue((self.repo / "README.md").exists())

    def test_tools_discovery_uses_tracked_names_and_executable_modes(self):
        for path in ("tools/test_root.py", "tools/nested/test_input.py", "tools/nested/test-shell",
                     "tools/nested/test-plain", "tools/nested/ordinary.py", "elsewhere/test_other.py"):
            self.write(path, "fixture\n")
        (self.repo / "tools/nested/test-shell").chmod(0o700)
        self.commit("tracked tools suites")
        self.write("tools/test_untracked.py", "untracked\n")
        self.assertEqual(mq.tools_commands(self.repo), [
            ("tools/nested/test-shell", ["./tools/nested/test-shell"], self.repo),
            ("tools/nested/test_input.py", ["python3", "-I", "-B", "tools/nested/test_input.py"], self.repo),
            ("tools/test_root.py", ["python3", "-I", "-B", "tools/test_root.py"], self.repo)])

    def test_executable_python_and_bash_suites_use_their_shebangs(self):
        self.write("tools/with spaces/test-python", """#!/usr/bin/env python3
import pathlib, sys
assert sys.stdin.read() == ''
assert pathlib.Path('shared').read_text() == 'base\\n'
print('Python shebang passed')
""")
        self.write("tools/test-bash", """#!/bin/bash
set -eu
words=(Bash shebang passed)
[[ $(<shared) == base && -z $(cat) ]]
printf '%s\\n' "${words[*]}"
""")
        for path in ("tools/with spaces/test-python", "tools/test-bash"):
            (self.repo / path).chmod(0o700)
        self.commit("executable suites with different interpreters")
        log_path = self.directory / "checks.log"
        with log_path.open("a+") as log:
            result = mq.run_tools(self.repo, log)
        self.assertTrue(result["green"], log_path.read_text())
        self.assertEqual(len(result["suites"]), 2)
        self.assertIn("Python shebang passed", log_path.read_text())
        self.assertIn("Bash shebang passed", log_path.read_text())

    def test_heavy_discovery_excludes_only_the_systemd_recovery_suite(self):
        offline = ["tools/heavy/test-gate-policy", "tools/heavy/test-heavy-plain",
                   "tools/heavy/test-receipt-inputs", "tools/other/test-heavy-recovery"]
        recovery = "tools/heavy/test-heavy-recovery"
        for path in [*offline, recovery]:
            body = "raise AssertionError('systemd recovery must not run in the gate')" if path == recovery else "print('offline fixture passed')"
            self.write(path, "#!/usr/bin/env python3\n" + body + "\n")
            (self.repo / path).chmod(0o700)
        self.commit("offline and manual recovery suites")
        self.assertEqual([name for name, _, _ in mq.tools_commands(self.repo)], offline)
        log_path = self.directory / "checks.log"
        with log_path.open("a+") as log:
            result = mq.run_tools(self.repo, log)
        self.assertTrue(result["green"], log_path.read_text())
        self.assertEqual([suite["name"] for suite in result["suites"]], offline)

    def test_tools_only_batch_runs_tools_without_cargo_and_lands(self):
        self.branch("tools", "tools/test_root.py", """import pathlib, sys
assert sys.flags.isolated and sys.flags.dont_write_bytecode
assert pathlib.Path('shared').read_text() == 'base\\n'
assert sys.stdin.read() == ''
print('tools fixture passed')
""", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = mq.run_checks
        queue.metadata_loader = lambda repo: self.fail("tools-only selection needs no cargo metadata")
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual((record["selection"], record["outcome"]), ("docs", "green"))
        self.assertEqual([step["name"] for step in record["steps"]], ["tools"])
        tools = record["steps"][0]
        self.assertEqual([suite["name"] for suite in tools["suites"]], ["tools/test_root.py"])
        self.assertGreater(tools["duration"], 0)
        self.assertGreater(tools["suites"][0]["duration"], 0)
        self.assertIn("tools fixture passed", Path(record["log"]).read_text())
        self.assertTrue((self.repo / "tools/test_root.py").exists())

    def test_failing_tools_suite_rejects_single_branch(self):
        branch = self.branch("tools", "tools/test_red.py", "raise SystemExit('tools fixture failed')\n", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = mq.run_checks
        self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual((record["selection"], record["outcome"]), ("docs", "red"))
        self.assertEqual([step["name"] for step in record["steps"]], ["tools"])
        self.assertEqual(record["steps"][0]["suites"][0]["returncode"], 1)
        self.assertIn("tools fixture failed", Path(record["log"]).read_text())
        self.assertIn("failing step: tools", self.posts[0][2])
        self.assertEqual(queue.state["tried"], {branch["name"]: branch["tip"]})
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)

    def test_tools_failure_bisects_and_lands_green_branch(self):
        self.branch("a", "tools/test_green.py", "print('green fixture')\n", publish=True)
        self.branch("z", "tools/test_red.py", "raise SystemExit(1)\n", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        queue.checker = mq.run_checks
        self.run_cycle(queue)
        self.assertEqual([record["outcome"] for record in self.records()], ["red", "green", "red"])
        mq.git(self.repo, "switch", "--detach", "origin/main")
        self.assertTrue((self.repo / "tools/test_green.py").exists())
        self.assertFalse((self.repo / "tools/test_red.py").exists())
        self.assertEqual([subject for _, subject, _ in self.posts],
                         ["landed ready/coord/a", "rejected ready/coord/z"])

    def test_mixed_code_docs_and_tools_select_only_code_packages(self):
        branch = self.branch("mixed", "core/src/lib.rs")
        self.write("agents/x.md", "agent guidance\n")
        self.write("tools/y.py", "tool source\n")
        self.write("tools/test_green.py", "print('green fixture')\n")
        self.commit("mixed inputs")
        branch["tip"] = self.tip()
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        calls = []

        def runner(argv, **kwargs):
            calls.append(argv)
            return subprocess.CompletedProcess(argv, 0)

        queue.checker = lambda *args: mq.run_checks(*args, runner=runner)
        with patch.object(mq, "load_workspace_metadata", return_value=metadata(self.repo)):
            self.assertTrue(queue.attempt([branch]))
        record = self.records()[0]
        self.assertEqual(record["selection"], sorted(
            {"core", "normal", "build", "dev", "transitive"} | set(mq.lines(DIRECTORY / "always.txt"))))
        self.assertEqual([step["name"] for step in record["steps"]],
                         ["tools", "fmt", "clippy", "build", "default", "fuzzy"])
        self.assertEqual(calls[0], ["python3", "-I", "-B", "tools/test_green.py"])
        self.assertTrue(all("--workspace" not in argv for argv in calls))

    def test_source_markdown_batch_loads_ownership_and_selects_package(self):
        self.branch("source-doc", "core/src/input.md", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0],
                         {"core", "normal", "build", "dev", "transitive"} | set(mq.lines(DIRECTORY / "always.txt")))

    def test_batch_record_keeps_build_and_default_durations_separate(self):
        branch = self.branch("timed")
        queue = self.queue()
        times = iter([0, 1, 10, 12, 20, 27, 30, 33, 40, 44])
        queue.checker = lambda *args: mq.run_checks(
            *args, runner=lambda *a, **kw: subprocess.CompletedProcess(a, 0), clock=lambda: next(times))
        queue.attempt([branch])
        self.assertEqual([(step["name"], step["duration"]) for step in self.records()[0]["steps"]],
                         [("fmt", 1), ("clippy", 2), ("build", 7), ("default", 3), ("fuzzy", 4)])

    def test_failed_build_stops_tests_and_names_build_in_rejection(self):
        branch = self.branch("unbuildable")
        queue = self.queue()

        def runner(argv, **kwargs):
            return subprocess.CompletedProcess(argv, 1 if "--no-run" in argv else 0)

        queue.checker = lambda *args: mq.run_checks(*args, runner=runner)
        queue.attempt([branch])
        record = self.records()[0]
        self.assertEqual(record["outcome"], "red")
        self.assertEqual([step["name"] for step in record["steps"]], ["fmt", "clippy", "build"])
        self.assertIn("failing step: build", self.posts[0][2])

    def test_manifest_batch_does_not_need_valid_cargo_metadata(self):
        self.branch("a", "Cargo.toml", "not a valid manifest\n", publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999

        def no_metadata(repo):
            self.fail("a manifest change must select full before metadata is loaded")

        queue.metadata_loader = no_metadata
        self.run_cycle(queue)
        self.assertEqual(self.checks[0][0], "full")

    def test_lock_batch_uses_base_blob_and_merged_head_without_checkout(self):
        self.write("Cargo.lock", lockfile([{"name": "external", "version": "1.0.0",
                                            "source": "registry+fixture"}]))
        self.commit("base lock")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        self.branch("lock", "Cargo.lock", lockfile([{"name": "external", "version": "2.0.0",
                                                      "source": "registry+fixture"}]), publish=True)
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        loaded = []

        def loader(repo):
            loaded.append(self.tip())
            return resolved_metadata(repo)

        queue.metadata_loader = loader
        self.run_cycle(queue)
        self.assertEqual(loaded, [self.checks[0][2]])
        self.assertEqual(self.records()[0]["selection"], sorted(
            {"core", "normal", "build", "dev", "transitive"} | set(mq.lines(DIRECTORY / "always.txt"))))
        self.assertEqual(self.records()[0]["changed_paths"], ["Cargo.lock"])

    def test_successor_paths_use_parent_and_selection_uses_candidate_graph(self):
        self.write("core/src/first.rs", "first batch\n")
        self.write("tools/fixture/metadata.json", json.dumps(metadata(self.repo)))
        self.commit("C1 source and graph")
        parent = self.tip()
        self.write("unrelated/src/second.rs", "second batch\n")
        data = metadata(self.repo)
        next(node for node in data["resolve"]["nodes"] if node["id"] == "normal")["dependencies"] = ["unrelated"]
        self.write("tools/fixture/metadata.json", json.dumps(data))
        self.commit("C2 source and graph")
        candidate = self.tip()
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        loaded = []

        def loader(repo):
            loaded.append(self.tip())
            return json.loads((repo / "tools/fixture/metadata.json").read_text())

        queue.metadata_loader = loader
        paths = mq.changed_paths(self.repo, candidate, base=parent)
        self.assertEqual(paths, ["tools/fixture/metadata.json", "unrelated/src/second.rs"])
        self.assertEqual(queue.choose(paths, base=parent, candidate=candidate),
                         {"unrelated", "normal"} | set(mq.lines(DIRECTORY / "always.txt")))
        self.assertEqual(loaded, [candidate])
        self.assertIn("core/src/first.rs", mq.changed_paths(self.repo))
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)
        self.assertEqual(self.tip(), candidate)
        # Explicit object inputs work even when HEAD is elsewhere, without changing it.
        mq.git(self.repo, "switch", "--detach", self.base)
        self.assertEqual(mq.changed_paths(self.repo, candidate, base=parent), paths)
        self.assertEqual(self.tip(), self.base)

    def test_successor_lock_compares_parent_and_candidate_blobs(self):
        external = {"name": "external", "version": "1.0.0", "source": "registry+fixture"}
        unrelated = {"name": "unrelated", "version": "1.0.0"}
        self.write("Cargo.lock", lockfile([external, unrelated]))
        self.commit("P lock")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        self.write("core/src/first.rs", "first batch\n")
        external["version"] = "2.0.0"
        self.write("Cargo.lock", lockfile([external, unrelated]))
        self.commit("C1 changes external dependency")
        parent = self.tip()
        unrelated["version"] = "2.0.0"
        self.write("Cargo.lock", lockfile([external, unrelated]))
        self.commit("C2 changes unrelated member")
        candidate = self.tip()
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        loaded = []

        def loader(repo):
            loaded.append(self.tip())
            data = resolved_metadata(repo)
            entries = mq.tomllib.loads((repo / "Cargo.lock").read_text())["package"]
            version = next(entry["version"] for entry in entries if entry["name"] == "unrelated")
            next(package for package in data["packages"] if package["id"] == "unrelated")["version"] = version
            return data

        queue.metadata_loader = loader
        paths = mq.changed_paths(self.repo, base=parent)
        self.assertEqual(paths, ["Cargo.lock"])
        with patch.object(mq, "git", wraps=mq.git) as git:
            explicit = queue.choose(paths, base=parent, candidate=candidate)
        shows = [call.args[2] for call in git.call_args_list if call.args[1] == "show"]
        self.assertEqual(shows, [f"{parent}:Cargo.lock", f"{candidate}:Cargo.lock"])
        self.assertEqual(explicit, {"unrelated"} | set(mq.lines(DIRECTORY / "always.txt")))
        self.assertEqual(queue.choose(paths, base=parent), explicit)
        # One-gate defaults still compare origin/main and HEAD, including C1's lock change.
        self.assertEqual(queue.choose(paths), explicit | {"core", "normal", "build", "dev", "transitive"})
        self.assertEqual(loaded, [candidate] * 3)
        self.assertEqual(self.tip(), candidate)
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)

    def test_checked_selection_refuses_graph_from_another_candidate(self):
        self.write("core/src/lib.rs", "candidate\n")
        self.commit("candidate source")
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        with patch.object(queue, "metadata_loader") as loader:
            with self.assertRaisesRegex(ValueError, "candidate checkout"):
                queue.choose(["core/src/lib.rs"], candidate=self.base)
        loader.assert_not_called()

    def test_plan_uses_merge_base_while_checked_paths_compare_trees(self):
        branch = self.branch("candidate", "unrelated/src/lib.rs", publish=True)
        self.branch("main", "core/src/lib.rs")
        parent = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        self.assertEqual(mq.changed_paths(self.repo, branch["tip"], base=parent),
                         ["core/src/lib.rs", "unrelated/src/lib.rs"])
        queue = self.queue(now=lambda: 10000)
        queue.state["last_full"] = 9999
        with redirect_stdout(io.StringIO()) as output:
            queue.plan()
        plan = json.loads(output.getvalue())
        self.assertEqual(plan["changed_paths"], ["unrelated/src/lib.rs"])
        self.assertEqual(plan["selection"], sorted({"unrelated"} | set(mq.lines(DIRECTORY / "always.txt"))))
        self.assertEqual(self.tip(), parent)
        self.assertEqual(self.checks, [])
        self.assertEqual(self.posts, [])
        self.assertFalse(queue.state_path.exists())

    def test_parent_relative_paths_include_rename_owners_and_deletions(self):
        self.write("core/src/lib.rs", "move\n")
        self.write("dev/src/lib.rs", "delete\n")
        self.commit("C1 source")
        parent = self.tip()
        (self.repo / "unrelated/src").mkdir(parents=True)
        mq.git(self.repo, "mv", "core/src/lib.rs", "unrelated/src/lib.rs")
        mq.git(self.repo, "rm", "dev/src/lib.rs")
        self.commit("C2 moves and deletes sources")
        self.assertEqual(mq.changed_paths(self.repo, base=parent),
                         ["core/src/lib.rs", "dev/src/lib.rs", "unrelated/src/lib.rs"])

    def test_recent_batch_shape_fixtures_select_instead_of_forcing_full(self):
        # The task describes eight lock/manifest/source batches; exact logs are not supplied.
        fixture_count = 8
        records = [{"selection": "full", "changed_paths": ["Cargo.lock", "core/Cargo.toml", "core/src/lib.rs"]}
                   for _ in range(fixture_count)]
        fixture = self.directory / "recent-batches.jsonl"
        fixture.write_text("".join(json.dumps(record) + "\n" for record in records))
        selections = [mq.selection(json.loads(line)["changed_paths"], resolved_metadata(),
                                   locked={"external"}, always=["always"])
                      for line in fixture.read_text().splitlines()]
        self.assertEqual(sum(isinstance(selected, set) for selected in selections), fixture_count)
        self.assertTrue(all(selected == {"core", "normal", "build", "dev", "transitive", "always"}
                            for selected in selections))

    def test_full_failure_is_bisected_with_full_checks(self):
        self.branch("a", "core/src/good.rs", publish=True)
        self.branch("z", "unrelated/src/red.rs", publish=True)
        queue = self.queue()
        selections = []

        def check(repo, selected, paths, log):
            selections.append(selected)
            green = selected != "full" or not (repo / "unrelated/src/red.rs").exists()
            return [{"name": "fixture", "green": green, "duration": 0.0}]

        queue.checker = check
        self.run_cycle(queue)
        self.assertTrue(all(selected == "full" for selected in selections))
        self.assertTrue((self.repo / "core/src/good.rs").exists())
        mq.git(self.repo, "switch", "--detach", "origin/main")
        self.assertFalse((self.repo / "unrelated/src/red.rs").exists())
        self.assertTrue(any("rejected ready/coord/z" == subject for _, subject, _ in self.posts))

    def test_dry_bisection_checks_later_groups_against_prior_green_groups(self):
        for task in ("a", "red", "z"):
            self.branch(task, publish=True)
        queue = self.queue(dry=True)
        observed = []

        def check(repo, selected, paths, log):
            observed.append({name for name in ("a", "red", "z") if (repo / name).exists()})
            return self.checker(repo, selected, paths, log)

        queue.checker = check
        self.run_cycle(queue)
        self.assertEqual(observed[-1], {"a", "z"})
        self.assertEqual(self.posts, [])

    def test_checks_cannot_leave_uncommitted_inputs_and_land(self):
        self.branch("a", publish=True)
        queue = self.queue()

        def dirty_check(repo, selected, paths, log):
            self.write("shared", "modified by check\n")
            return [{"name": "fixture", "green": True, "duration": 0.0}]

        queue.checker = dirty_check
        with self.assertRaisesRegex(RuntimeError, "uncommitted or untracked"):
            self.run_cycle(queue)
        self.assertEqual(self.records()[0]["outcome"], "error")
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)

    def test_merge_setup_error_is_logged_without_rejecting_a_branch(self):
        branch = self.branch("a", publish=True)
        queue = self.queue()
        with patch.object(mq, "build_batch", side_effect=RuntimeError("merge infrastructure failure")):
            with self.assertRaisesRegex(RuntimeError, "merge infrastructure failure"):
                self.run_cycle(queue)
        record = self.records()[0]
        self.assertEqual(record["branches"][0]["name"], branch["name"])
        self.assertEqual(record["outcome"], "error")
        self.assertIn("merge infrastructure failure", Path(record["log"]).read_text())
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(self.posts, [])

    def run_cycles(self, queue, count):
        stop = self.directory / "stop"
        sleeps = []

        def sleeper(seconds):
            sleeps.append(seconds)
            if len(sleeps) == count:
                stop.touch()

        with redirect_stdout(io.StringIO()):
            queue.run(stop, sleeper=sleeper)
        return sleeps

    def test_run_recovers_errors_and_reports_each_distinct_text_once(self):
        branch = self.branch("a", publish=True)
        queue = self.queue()
        failures = iter(["gate guard", "gate guard", "git failed"])

        def fail(repo, selected, paths, log):
            raise RuntimeError(next(failures))

        queue.checker = fail
        self.assertEqual(self.run_cycles(queue, 3), [60, 60, 60])
        self.assertEqual(self.tip(), self.base)
        self.assertEqual(mq.git(self.repo, "branch", "--show-current").stdout.strip(), "")
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual([body.splitlines()[0] for _, _, body in self.posts], ["gate guard", "git failed"])
        batches = [record for record in self.records() if "event" not in record]
        self.assertEqual([record["outcome"] for record in batches], ["error"] * 3)
        self.assertTrue(all(record["branches"][0]["tip"] == branch["tip"] for record in batches))
        self.assertTrue(self.run_cycle(self.queue()))
        self.assertTrue((self.repo / "a").exists())

    def test_run_lands_batch_on_next_cycle_after_error(self):
        self.branch("a", publish=True)
        queue = self.queue()
        calls = []
        stop = self.directory / "stop"

        def checker(*args):
            calls.append(args)
            if len(calls) == 1:
                raise RuntimeError("temporary git failure")
            stop.touch()
            return self.checker(*args)

        queue.checker = checker
        with redirect_stdout(io.StringIO()):
            queue.run(stop, sleeper=lambda seconds: None)
        self.assertEqual(len(calls), 2)
        self.assertTrue((self.repo / "a").exists())
        self.assertEqual(self.records()[-1]["outcome"], "green")
        self.assertEqual([address for address, subject, _ in self.posts if subject == "queue error"], ["ops@hel8"])

    def test_same_error_alerts_again_after_completed_batch(self):
        queue = self.queue()
        outcomes = iter([RuntimeError("repeated dirty tree"), True,
                         RuntimeError("repeated dirty tree")])

        def cycle():
            outcome = next(outcomes)
            if isinstance(outcome, Exception):
                raise outcome
            return outcome

        queue.cycle = cycle
        self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        alerts = [body for _, subject, body in self.posts if subject == "queue error"]
        self.assertEqual(len(alerts), 2)
        self.assertTrue(all(body.startswith("repeated dirty tree") for body in alerts))

    def test_idle_recovery_clears_persisted_error_episode(self):
        queue = self.queue()
        outcomes = iter([RuntimeError("repeated dirty tree"), False])

        def cycle():
            outcome = next(outcomes)
            if isinstance(outcome, Exception):
                raise outcome
            return outcome

        queue.cycle = cycle
        self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        reloaded = self.queue()
        self.assertEqual(reloaded.state.get("reported_errors"), [])
        (self.directory / "stop").unlink()
        reloaded.cycle = lambda: (_ for _ in ()).throw(RuntimeError("repeated dirty tree"))
        self.assertEqual(self.run_cycles(reloaded, 1), [60])
        self.assertEqual(len([1 for _, subject, _ in self.posts if subject == "queue error"]), 2)

    def test_run_survives_recovery_log_failure(self):
        queue = self.queue()
        queue.cycle = lambda: (_ for _ in ()).throw(RuntimeError("git failed"))
        output = io.StringIO()
        with patch.object(mq, "log_batch", side_effect=OSError("disk full")), redirect_stderr(output):
            self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        self.assertIn("recovery failed: disk full", output.getvalue())
        self.assertEqual(len(self.posts), 1)

    def test_run_aborts_unfinished_merge_and_preserves_ignored_target(self):
        self.write(".gitignore", "/target/\n")
        self.commit("ignore build output")
        self.base = self.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        mq.git(self.repo, "fetch", "origin")
        first = self.branch("first", "shared", "first\n")
        second = self.branch("second", "shared", "second\n")
        mq.build_batch(self.repo, [first])
        mq.git(self.repo, "merge", "--no-ff", "--no-edit", second["tip"], check=False)
        self.write("target/kept", "build output\n")
        queue = self.queue()
        queue.cycle = lambda: (_ for _ in ()).throw(RuntimeError("unfinished merge"))
        self.run_cycles(queue, 1)
        self.assertEqual(self.tip(), self.base)
        self.assertNotEqual(mq.git(self.repo, "rev-parse", "--verify", "--quiet", "MERGE_HEAD",
                                   check=False).returncode, 0)
        self.assertEqual((self.repo / "target/kept").read_text(), "build output\n")

    def test_error_does_not_reject_conflicted_branch_in_same_batch(self):
        for task, text in (("a", "first\n"), ("b", "second\n")):
            self.branch(task, "shared", text, publish=True)
        queue = self.queue()
        queue.checker = lambda *args: (_ for _ in ()).throw(RuntimeError("gate guard"))
        self.run_cycles(queue, 1)
        self.assertEqual(queue.state["tried"], {})
        self.assertFalse(any("rejected" in subject for _, subject, _ in self.posts))

    def test_root_and_tree_targets_are_ignored(self):
        paths = ["target/fixture"] + [f"{tree}/target/fixture" for tree in mq.TREES]
        for prefix in ("", *(tree + "/" for tree in mq.TREES)):
            self.write(prefix + ".gitignore", (ROOT / prefix / ".gitignore").read_text())
        self.assertEqual(mq.git(self.repo, "check-ignore", *paths).stdout.splitlines(), paths)

    def test_heavy_disk_refusal_errors_without_rejection_or_bisection(self):
        for task in ("a", "b"):
            self.branch(task, publish=True)
        queue = self.queue()
        attempts = []

        def refuse(*args):
            attempts.append(args)
            return [{"name": "clippy", "green": False, "returncode": 2,
                     "error": "builder: 39 GB free, below the disk floor"}]

        queue.checker = refuse
        self.run_cycles(queue, 2)
        batches = [record for record in self.records() if "event" not in record]
        self.assertEqual([record["outcome"] for record in batches], ["error", "error"])
        self.assertEqual(len(attempts), 2)
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(len(self.posts), 1)
        self.assertIn("builder: 39 GB", self.posts[0][2])
        self.assertEqual(self.tip(), self.base)

    def test_low_disk_waits_without_starting_a_batch_and_posts_once(self):
        self.branch("a", publish=True)
        queue = self.queue(disk_floor=45)
        with patch.object(mq, "free_gb", side_effect=[44, 43]):
            self.assertEqual(self.run_cycles(queue, 2), [60, 60])
        self.assertEqual(self.checks, [])
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(len(self.posts), 1)
        self.assertIn("44.0 GB free", self.posts[0][2])
        self.assertTrue(all(record["branches"] == [] for record in self.records()))

    def test_disk_floor_allows_a_batch_at_the_boundary(self):
        self.branch("a", publish=True)
        with patch.object(mq, "free_gb", return_value=45):
            self.assertTrue(self.run_cycle(self.queue(disk_floor=45)))
        self.assertEqual(self.records()[0]["outcome"], "green")

    def test_every_batch_logs_cleanup_for_green_red_and_error(self):
        for task, outcome in (("a", "green"), ("red", "red"), ("error", "error")):
            with self.subTest(outcome=outcome):
                branch = self.branch(task)
                queue = self.queue()
                if outcome == "error":
                    queue.checker = lambda *args: (_ for _ in ()).throw(RuntimeError("gate guard"))
                with patch.object(mq, "prune_test_executables", return_value={"count": 2, "gb": 3}) as prune:
                    if outcome == "error":
                        with self.assertRaisesRegex(RuntimeError, "gate guard"):
                            queue.attempt([branch])
                    else:
                        queue.attempt([branch])
                self.assertEqual(prune.call_count, 1)
                self.assertEqual(prune.call_args.args[:2], (self.repo, 0))
                self.assertGreater(prune.call_args.args[2], 0)
                record = self.records(effects=True)[-1]
                self.assertEqual(record["outcome"], outcome)
                self.assertEqual(record["target_cleanup"], {"count": 2, "gb": 3})

    def test_changed_tip_is_preserved_by_cleanup(self):
        branch = self.branch("a", publish=True)
        self.write("new", "new\n")
        self.commit("advanced ready tip")
        advanced = self.tip()
        mq.git(self.repo, "push", "origin", f"HEAD:refs/heads/{branch['name']}")
        mq.build_batch(self.repo, [branch])
        self.queue().attempt([branch])
        cleanup = self.records(effects=True)[-1]["cleanup"]
        self.assertIn("preserved", cleanup[0]["outcome"])
        self.assertEqual(mq.git(self.repo, "ls-remote", "origin", "refs/heads/" + branch["name"]).stdout.split()[0], advanced)

    def test_tip_advanced_between_observation_and_delete_survives_legacy_cleanup(self):
        branch = self.branch("a", publish=True)
        self.write("advance", "advanced ready\n")
        self.commit("advance ready")
        advanced = self.tip()
        mq.build_batch(self.repo, [branch])
        ref = "refs/heads/" + branch["name"]
        git, pushes, observed = mq.git, [], []
        def race(repo, *args, **kwargs):
            if args[0] == "push":
                pushes.append(args)
                if "--delete" in args or args[-1] == ":" + ref:
                    self.assertEqual(observed, [branch["tip"]])
                    git(self.repo, "push", "origin", advanced + ":" + ref)
            result = git(repo, *args, **kwargs)
            if args[0] == "ls-remote" and args[-1] == ref:
                observed.append(result.stdout.split()[0])
            return result
        with patch.object(mq, "git", side_effect=race):
            self.queue().attempt([branch])
            commit = self.records()[0]["commit"]
            cleanup = self.records(effects=True)[-1]["cleanup"]
        self.assertEqual(git(self.origin, "rev-parse", ref).stdout.strip(), advanced)
        self.assertEqual(git(self.origin, "rev-parse", "refs/heads/main").stdout.strip(), commit)
        self.assertNotEqual(cleanup[0]["returncode"], 0)
        main_push, delete = pushes
        self.assertFalse(any("--force" in arg or "--lease" in arg for arg in main_push))
        self.assertEqual(delete, ("push", "--force-with-lease=" + ref + ":" + branch["tip"],
                                  str(self.origin), ":" + ref))

    def test_push_failure_does_not_delete_ready_branch(self):
        branch = self.branch("a", publish=True)
        mq.build_batch(self.repo, [branch])
        mq.git(self.repo, "config", "remote.origin.pushurl", str(self.directory / "absent.git"))
        with self.assertRaises(RuntimeError):
            self.queue().attempt([branch])
        self.assertIn(branch["name"], mq.git(self.repo, "ls-remote", "origin").stdout)

    def test_periodic_hourly_and_idle_full_policy(self):
        queue = self.queue(now=lambda: 10000)
        self.assertTrue(queue.full_due())
        queue.state.update(last_full=9999, batches_since_full=3, idle_since=9500)
        self.assertFalse(queue.full_due())
        self.assertFalse(queue.idle_due())
        queue.state["batches_since_full"] = 4
        self.assertTrue(queue.full_due())
        queue.state.update(batches_since_full=0, last_full=6400, idle_since=9400)
        self.assertTrue(queue.full_due())
        self.assertTrue(queue.idle_due())
        queue.state["last_full_main"] = self.base
        self.assertFalse(queue.idle_due())

    def test_idle_main_gets_full_check_without_push(self):
        queue = self.queue(now=lambda: 10000)
        queue.state.update(idle_since=9400)
        self.assertTrue(self.run_cycle(queue))
        self.assertEqual(self.checks[0][0], "full")
        self.assertEqual(self.records()[0]["branches"], [])
        self.assertEqual(self.posts, [])

    def test_failed_full_after_selected_success_reports_gap(self):
        branch = self.branch("red")
        queue = self.queue()
        queue.state["selected_passed"] = True
        self.assertFalse(queue.attempt([branch]))
        self.assertIn(("ops@hel8", "possible queue selection gap"),
                      [(address, subject) for address, subject, _ in self.posts])

    def test_dirty_checkout_is_rejected_without_overwriting_it(self):
        branch = self.branch("a")
        self.write("untracked", "keep\n")
        with self.assertRaisesRegex(RuntimeError, "uncommitted or untracked"):
            mq.build_batch(self.repo, [branch])
        self.assertEqual((self.repo / "untracked").read_text(), "keep\n")

    def test_empty_queue_is_noop_even_in_dirty_checkout(self):
        self.write("untracked", "keep\n")
        self.assertFalse(self.run_cycle(self.queue(dry=True)))
        self.assertEqual(self.checks, [])


class PostTests(unittest.TestCase):
    def test_post_uses_queue_identity_and_treats_exit_three_as_delivered(self):
        result = subprocess.CompletedProcess([], 3, "queued, nobody will wake it")
        with patch.object(mq, "command", return_value=result) as command:
            outcome = mq.post("fixture@example.test", "a subject", "body\n")
        self.assertEqual(command.call_args.args[0],
                         ["post", "send", "--as", "queue", "fixture@example.test", "a subject"])
        self.assertEqual(command.call_args.kwargs["body"], "body\n")
        self.assertTrue(outcome["sent"])

    def test_metadata_json_is_separate_from_diagnostics(self):
        with tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".mq-test-") as directory:
            result = mq.command([sys.executable, "-I", "-B", "-c",
                                 'import sys; print("{}"); print("diagnostic", file=sys.stderr)'],
                                Path(directory), separate_stderr=True)
        self.assertEqual(json.loads(result.stdout), {})
        self.assertIn("diagnostic", result.stderr)


if __name__ == "__main__":
    unittest.main()
