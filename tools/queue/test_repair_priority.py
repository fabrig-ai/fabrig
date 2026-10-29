"""Pinned repair policy against local Git repositories and simulated restarts."""

import sys
sys.dont_write_bytecode = True

from contextlib import redirect_stderr, redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch


DIRECTORY = Path(__file__).resolve().parent


def load(name, path):
    loader = importlib.machinery.SourceFileLoader(name, str(path))
    spec = importlib.util.spec_from_loader(name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


mq = load("repair_mq", DIRECTORY / "mq")
restart = load("repair_restart", DIRECTORY / "restart")
REASON = "Repair the recurring gate failure before ordinary inputs"


class RepairTests(unittest.TestCase):
    def setUp(self):
        # All mutable fixtures, including runtime copies, are outside checked sources.
        self.temporary = tempfile.TemporaryDirectory(dir=DIRECTORY.parents[2], prefix=".repair-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.home = self.root / "home"
        self.preparer = self.root / "preparer"
        self.preparer.mkdir()
        self.origin = self.root / "origin.git"
        self.repo = self.root / "queue"
        self.audit = self.root / "queue.jsonl"
        self.state = self.root / "state.json"
        self.commit_count = 0
        mq.git(self.preparer, "init", "-b", "main")
        self.identity(self.preparer)
        (self.preparer / "shared.md").write_text("base\n")
        self.commit("base")
        self.base = self.tip(self.preparer)
        mq.git(self.preparer, "init", "--bare", str(self.origin))
        mq.git(self.preparer, "remote", "add", "origin", str(self.origin))
        mq.git(self.preparer, "push", "origin", "HEAD:main")
        mq.git(self.root, "clone", "-b", "main", str(self.origin), str(self.repo))
        self.identity(self.repo)
        self.checks = []
        self.posts = []
        self.result = "green"

    def identity(self, repo):
        mq.git(repo, "config", "user.name", "Repair fixture")
        mq.git(repo, "config", "user.email", "repair@example.test")
        mq.git(repo, "config", "commit.gpgsign", "false")

    def tip(self, repo):
        return mq.git(repo, "rev-parse", "HEAD").stdout.strip()

    def commit(self, subject):
        mq.git(self.preparer, "add", ".")
        self.commit_count += 1
        with patch.dict(os.environ, {"GIT_COMMITTER_DATE": f"@{1700000000 + self.commit_count} +0000"}):
            mq.git(self.preparer, "commit", "-m", subject)

    def branch(self, task, path=None, text="change\n", *, parent=None):
        mq.git(self.preparer, "switch", "--detach", parent or self.base)
        (self.preparer / (path or task + ".md")).write_text(text)
        self.commit(task)
        branch = {"name": "ready/coord/" + task, "tip": self.tip(self.preparer)}
        mq.git(self.preparer, "push", "origin", "HEAD:refs/heads/" + branch["name"])
        return branch

    def repair(self, **kwargs):
        return {**self.branch("zz-repair", **kwargs), "reason": REASON}

    def checker(self, repo, selection, paths, log):
        self.checks.append((selection, paths, self.tip(repo)))
        if self.result == "error":
            raise RuntimeError("fixture infrastructure failure")
        if self.result == "capacity":
            return [{"name": "build", "green": False, "error": "fixture below the disk floor"}]
        if self.result == "launch":
            return [{"name": "tools", "green": False, "suites": [
                {"name": "suite", "green": False, "launch_error": "fixture missing interpreter"}]}]
        return [{"name": "fixture", "green": self.result == "green", "duration": 0}]

    def queue(self, repair=None, **kwargs):
        if not hasattr(self, "lease"):
            self.layout = mq.authority.Layout(self.root / "authority")
            self.lease = self.enterContext(self.layout.acquire())
            self.checkout = self.enterContext(mq.authority.checkout_lock(self.repo))
            self.enterContext(mq.writer_authority(self.lease, self.checkout))
        def guard():
            self.lease.guard()
            self.checkout.guard()
            self.layout.legacy_guard(self.repo)
        return mq.Queue(self.repo, batch=16, repair=repair, state_path=self.state, log_path=self.audit,
                        disk_floor=0, checker=self.checker, poster=self.poster, now=lambda: 100,
                        writer_guard=guard, **kwargs)

    def poster(self, address, subject, body):
        self.posts.append((address, subject, body))
        return {"sent": True}

    def cycle(self, queue):
        with redirect_stdout(io.StringIO()):
            return queue.cycle()

    def records(self):
        return [json.loads(line) for line in self.audit.read_text().splitlines()]

    def assert_repair_record(self, repair, outcome):
        record = self.records()[-1]
        self.assertEqual(record["repair"], repair)
        self.assertEqual(record["outcome"], outcome)
        return record

    def test_full_repair_alone_then_sixteen_ordinary_then_remaining(self):
        ordinary = [self.branch(f"ordinary-{index:02}") for index in range(17)]
        repair = self.repair()
        mq.git(self.repo, "fetch", "origin")
        candidates = mq.list_candidates(self.repo, {})
        self.assertLess(candidates[0]["timestamp"], candidates[-1]["timestamp"])
        self.assertEqual(candidates[-1]["name"], repair["name"])
        queue = self.queue(repair)
        queue.state.update(last_full=100, batches_since_full=2)
        self.cycle(queue)
        record = self.assert_repair_record(repair, "green")
        self.assertEqual([b["name"] for b in record["branches"]], [repair["name"]])
        self.assertEqual(record["selection"], "full")
        self.assertEqual(self.checks[0][1], ["zz-repair.md"])
        self.assertEqual(queue.state["batches_since_full"], 0)
        self.assertEqual(queue.state["last_full_main"], record["commit"])
        self.assertEqual(queue.state["tried"], {repair["name"]: repair["tip"]})
        # The same configuration survives restart after ready cleanup.
        queue = self.queue(repair)
        self.cycle(queue)
        self.assertEqual(len(self.records()[-1]["branches"]), 16)
        self.assertEqual(self.records()[-1]["selection"], "docs")
        self.assertNotIn("repair", self.records()[-1])
        self.cycle(queue)
        self.assertEqual(len(self.records()[-1]["branches"]), 1)
        landed = [b["name"] for record in self.records() for b in record["branches"]]
        self.assertCountEqual(landed, [b["name"] for b in ordinary] + [repair["name"]])
        self.assertEqual(len(self.posts), 18)

    def test_red_rejects_only_repair_and_resumes_normal_without_repeating(self):
        ordinary = self.branch("ordinary")
        repair = self.repair()
        self.result = "red"
        queue = self.queue(repair)
        self.cycle(queue)
        self.assert_repair_record(repair, "red")
        self.assertEqual(queue.state["tried"], {repair["name"]: repair["tip"]})
        self.assertEqual(len(self.records()), 1)
        self.assertEqual(mq.git(self.repo, "rev-parse", "origin/main").stdout.strip(), self.base)
        self.result = "green"
        self.cycle(self.queue(repair))
        self.assertEqual([b["name"] for b in self.records()[-1]["branches"]], [ordinary["name"]])

    def test_infrastructure_errors_preserve_unrelated_tried_and_retry_only_repair(self):
        ordinary = self.branch("ordinary")
        repair = self.repair()
        queue = self.queue(repair)
        unrelated = {"ready/coord/completed": self.base}
        queue.state["tried"].update(unrelated)
        queue.save()
        for failure in ("error", "error", "capacity", "launch"):
            self.result = failure
            with self.subTest(failure=failure), self.assertRaises(RuntimeError):
                self.cycle(queue)
            record = self.assert_repair_record(repair, "error")
            self.assertEqual([b["name"] for b in record["branches"]], [repair["name"]])
            self.assertEqual(queue.state["tried"], unrelated)
            self.assertEqual(json.loads(self.state.read_text())["tried"], unrelated)
            queue.recover(RuntimeError("fixture recovery"))
            queue = self.queue(repair)
        self.assertEqual(len(self.checks), 4)
        self.assertFalse(any(subject.startswith("rejected") for _, subject, _ in self.posts))
        self.result = "green"
        self.cycle(queue)
        self.cycle(queue)
        self.assertEqual([b["name"] for b in self.records()[-1]["branches"]], [ordinary["name"]])

    def test_disk_floor_before_attempt_retains_priority_audit(self):
        repair = self.repair()
        queue = self.queue(repair)
        with patch.object(mq, "free_gb", return_value=-1), self.assertRaises(mq.DiskFloorError):
            self.cycle(queue)
        self.assert_repair_record(repair, "error")
        self.assertEqual(queue.state["tried"], {})
        self.assertEqual(self.checks, [])

    def test_checkout_refusal_before_attempt_retains_priority_audit(self):
        repair = self.repair()
        (self.repo / "untracked").write_text("pending\n")
        queue = self.queue(repair)
        with self.assertRaisesRegex(RuntimeError, "uncommitted or untracked"):
            self.cycle(queue)
        self.assert_repair_record(repair, "error")
        self.assertEqual(self.checks, [])
        self.assertEqual(queue.state["tried"], {})

    def test_moved_missing_or_other_ready_pin_refuses_without_normal_fallback(self):
        ordinary = self.branch("ordinary")
        repair = self.repair()
        cases = [dict(repair, tip=ordinary["tip"]), dict(repair, name="ready/coord/missing")]
        self.branch("zz-repair", text="advanced\n", parent=repair["tip"])
        cases.append(repair)
        for configured in cases:
            with self.subTest(configured=configured), self.assertRaisesRegex(ValueError, "pinned tip"):
                self.cycle(self.queue(configured))
            record = self.assert_repair_record(configured, "error")
            self.assertEqual(record["event"], "repair-refusal")
        self.assertEqual(self.checks, [])
        self.assertFalse(self.state.exists())
        self.assertEqual(self.tip(self.repo), self.base)

    def test_pin_advanced_after_selection_preserves_new_ready_ref(self):
        repair = self.repair()
        queue = self.queue(repair)
        original = queue.checker
        advanced = []

        def checker(*args):
            advanced.append(self.branch("zz-repair", text="successor\n", parent=repair["tip"]))
            return original(*args)

        queue.checker = checker
        self.cycle(queue)
        record = self.assert_repair_record(repair, "green")
        self.assertEqual(record["cleanup"][0]["outcome"], "tip changed or branch absent; preserved")
        self.assertEqual(mq.git(self.preparer, "ls-remote", "--heads", "origin",
                                "refs/heads/" + repair["name"]).stdout.split()[0], advanced[0]["tip"])
        self.assertEqual(mq.git(self.repo, "show", "origin/main:zz-repair.md").stdout, "change\n")

    def test_conflict_rejects_repair_with_reason_and_no_checks(self):
        repair = self.repair(path="shared.md", text="repair\n")
        mq.git(self.preparer, "switch", "--detach", self.base)
        (self.preparer / "shared.md").write_text("main\n")
        self.commit("main advances")
        mq.git(self.preparer, "push", "origin", "HEAD:main")
        queue = self.queue(repair)
        self.cycle(queue)
        self.assert_repair_record(repair, "conflict")
        self.assertEqual(queue.state["tried"], {repair["name"]: repair["tip"]})
        self.assertEqual(self.checks, [])
        self.assertTrue(self.posts[0][1].startswith("rejected"))

    def test_plan_previews_full_singleton_and_default_queue_is_unchanged(self):
        ordinary = self.branch("ordinary")
        repair = self.repair()
        queue = self.queue(repair)
        with redirect_stdout(io.StringIO()) as output:
            queue.plan()
        plan = json.loads(output.getvalue())
        self.assertEqual(plan["repair"], repair)
        self.assertEqual(plan["selection"], "full")
        self.assertEqual([b["name"] for b in plan["branches"]], [repair["name"]])
        queue = self.queue()
        self.cycle(queue)
        self.assertCountEqual([b["name"] for b in self.records()[-1]["branches"]],
                              [ordinary["name"], repair["name"]])
        self.assertEqual(mq.Queue(self.repo, log_path=self.audit).batch, 8)

    def test_full_plan_keeps_ratchets_tools_and_all_cargo_checks(self):
        suite = self.repo / "tools/test_fixture.py"
        suite.parent.mkdir()
        suite.write_text("raise SystemExit(0)\n")
        mq.git(self.repo, "add", ".")
        mq.git(self.repo, "commit", "-m", "offline suite")
        commands = []

        def runner(argv, **kwargs):
            commands.append(argv)
            return subprocess.CompletedProcess(argv, 0)

        steps = mq.run_checks(self.repo, "full", ["temper/src/lib.rs"], self.root / "plan.log", runner=runner)
        self.assertEqual([step["name"] for step in steps],
                         ["ratchet:temper", "tools", "fmt", "clippy", "build", "default", "fuzzy"])
        self.assertEqual(commands, [
            ["python3", "-I", "-B", str(self.repo / "temper/scripts/old_forms.py"), "--write"],
            ["python3", "-I", "-B", "tools/test_fixture.py"],
            ["cargo", "fmt", "--all", "--check"],
            ["heavy", "--gate", "cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
            ["heavy", "--gate", "cargo", "nextest", "run", "--workspace", "--no-run"],
            ["heavy", "--gate", "cargo", "nextest", "run", "--workspace", "--no-fail-fast"],
            ["heavy", "--gate", "cargo", "nextest", "run", "--workspace", "--no-fail-fast", "--profile", "fuzzy"]])

    def test_malformed_cli_triplets_refuse_before_lock_or_queue_and_are_audited(self):
        ready = "ready/coord/repair"
        tip = self.base
        rows = [(ready, None, REASON), (None, tip, REASON), (ready, tip, None),
                ("other/coord/repair", tip, REASON), ("ready/coord/a..b", tip, REASON),
                ("ready/coord/a/b", tip, REASON), (ready, tip[:12], REASON),
                (ready, "G" * 40, REASON), (ready, tip, " "), (ready, tip, "reason\nmore")]
        for name, oid, reason in rows:
            argv = ["mq", "once", "--repo", str(self.repo)]
            for flag, value in (("--repair-ready", name), ("--repair-tip", oid), ("--repair-reason", reason)):
                if value is not None:
                    argv += [flag, value]
            with self.subTest(argv=argv), patch.object(sys, "argv", argv), \
                    patch.object(Path, "home", return_value=self.home), patch.object(mq, "Queue") as queue, \
                    redirect_stderr(io.StringIO()):
                self.assertEqual(mq.main(), 1)
                queue.assert_not_called()
                self.assertFalse((self.home / "src/fabrig/queue-authority").exists())
        records = (self.home / "src/fabrig/notes/queue.jsonl").read_text().splitlines()
        self.assertEqual(len(records), len(rows))
        self.assertTrue(all(json.loads(record)["event"] == "repair-refusal" and
                            json.loads(record)["outcome"] == "error" for record in records))

    def test_cli_forwards_validated_triplet_and_explicit_batch(self):
        repair = self.repair()
        for action, method in (("once", "cycle"), ("run", "run"), ("plan", "plan")):
            argv = ["mq", action, "--repo", str(self.repo), "--batch", "16",
                    "--authority-dir", str(self.root / "cli-authority"),
                    "--repair-ready", repair["name"], "--repair-tip", repair["tip"], "--repair-reason", REASON]
            with self.subTest(action=action), patch.object(sys, "argv", argv), \
                    patch.object(mq.shutil, "which", return_value="/fixture/tool"), \
                    patch.object(mq, "Queue") as queue:
                self.assertEqual(mq.main(), 0)
            self.assertEqual(queue.call_args.kwargs["repair"], repair)
            self.assertEqual(queue.call_args.kwargs["batch"], 16)
            getattr(queue.return_value, method).assert_called_once()

    def test_restart_cli_forwards_explicit_triplet_runtime_and_batch(self):
        repair = self.repair()
        runtime = self.runtime()
        argv = ["restart", "--repo", str(self.repo), "--runtime", str(runtime), "--batch", "16",
                "--repair-ready", repair["name"], "--repair-tip", repair["tip"], "--repair-reason", REASON]
        with patch.object(sys, "argv", argv), patch.object(restart.shutil, "which", return_value="/fixture/tool"), \
                patch.object(Path, "home", return_value=self.home), patch.object(restart, "restart") as launch:
            self.assertEqual(restart.main(), 0)
        launch.assert_called_once_with(self.repo, 1800, self.home, batch=16, runtime=runtime, authority_dir=None,
                                       repair_ready=repair["name"], repair_tip=repair["tip"], repair_reason=REASON)

    def runtime(self, name="runtime"):
        directory = self.root / name
        directory.mkdir()
        for name in restart.RUNTIME_FILES:
            shutil.copyfile(DIRECTORY / name, directory / name)
        return directory

    def launch_restart(self, repair, runtime, *, result=None, launch_error=None, mutate=None):
        stop = self.home / "src/fabrig/queue.stop"
        stop.parent.mkdir(parents=True, exist_ok=True)
        calls = []
        self.restart_calls = calls
        real_popen = subprocess.Popen
        real_select = restart.select.select
        adopted = []
        process_fds = []
        layout = mq.authority.Layout(self.home / "src/fabrig/queue-authority")

        def command(argv, repo, *, pass_fds=()):
            calls.append(argv)
            if argv[1] in ("fetch", "switch"):
                self.assertTrue(stop.exists())
                self.assertEqual(len(pass_fds), 2)
                for fd in pass_fds:
                    os.fstat(fd)
            return str(repo) if "--show-toplevel" in argv else self.base

        def pidfd(pid):
            read_fd, write_fd = os.pipe()
            os.write(write_fd, b"D")
            os.close(write_fd)
            process_fds.append(read_fd)
            return read_fd

        def select(fds, *args):
            if fds == process_fds and mutate:
                self.assertTrue(stop.exists())
                mutate(runtime)
            return real_select(fds, *args)

        def launch(argv, **kwargs):
            if argv[0] == "git":
                return real_popen(argv, **kwargs)
            self.assertFalse(stop.exists())
            authority_fd, startup_fd = kwargs["pass_fds"]
            self.assertEqual(argv, ["python3", "-B", "-u", str(runtime / "mq"), "run", "--repo", str(self.repo),
                                    "--authority-dir", str(layout.root), "--authority-fd", str(authority_fd),
                                    "--startup-fd", str(startup_fd),
                                    "--batch", "16", "--repair-ready", repair["name"],
                                    "--repair-tip", repair["tip"], "--repair-reason", REASON])
            self.assertEqual(kwargs["cwd"], self.repo)
            self.assertEqual(kwargs["stdin"], subprocess.DEVNULL)
            self.assertTrue(kwargs["start_new_session"])
            if launch_error:
                raise launch_error
            held = layout.acquire(inherited=authority_fd)
            adopted.append(held)
            with mq.authority.checkout_lock(self.repo):
                held.guard()
                layout.legacy_guard(self.repo)
                os.write(startup_fd, b"READY\n")
            return Mock(pid=123, returncode=result, poll=lambda: result)

        try:
            with patch.object(restart, "command", side_effect=command), \
                    patch.object(restart, "queue_pid", return_value=42), \
                    patch.object(os, "pidfd_open", side_effect=pidfd), \
                    patch.object(restart.select, "select", side_effect=select), \
                    patch.object(subprocess, "Popen", side_effect=launch), \
                    patch.object(restart.time, "sleep") as sleep, redirect_stdout(io.StringIO()):
                restart.restart(self.repo, 1800, self.home, batch=16, runtime=runtime,
                                repair_ready=repair["name"], repair_tip=repair["tip"], repair_reason=REASON)
            sleep.assert_not_called()
            with self.assertRaises(RuntimeError):
                layout.acquire()
        finally:
            for held in adopted:
                held.close()
        self.assertEqual(calls[1:3], [["git", "fetch", "origin"], ["git", "switch", "--detach", "origin/main"]])

    def test_external_restart_forwards_triplet_batch_and_uses_external_lists(self):
        repair = self.repair()
        runtime = self.runtime()
        self.launch_restart(repair, runtime)
        self.assertFalse((self.home / "src/fabrig/queue.stop").exists())
        (runtime / "always.txt").write_text("external-package\n")
        module, _ = restart.runtime_module(runtime, self.repo, external=True)
        self.assertEqual(module.CONFIG, runtime)
        self.assertEqual(Path(module.authority.__file__), runtime / "authority.py")
        self.assertEqual(Path(module.authority.durable.__file__), runtime / "durable.py")
        worker = load("repair_worker", runtime / "worker.py")
        train = load("repair_train", runtime / "train.py")
        self.assertEqual(set(restart.RUNTIME_FILES), set(worker.RUNTIME_FILES))
        self.assertEqual(Path(train.recovery.__file__), runtime / "recovery.py")
        self.assertEqual(Path(train.recovery.durable.__file__), runtime / "durable.py")
        self.assertEqual(module.lines(module.CONFIG / "always.txt"), ["external-package"])
        self.assertEqual(list(runtime.rglob("__pycache__")), [])
        self.assertEqual(list(self.repo.rglob("__pycache__")), [])

    def test_every_launch_failure_restores_stop(self):
        repair = self.repair()
        runtime = self.runtime()
        for result, error in ((2, None), (None, OSError("fixture launch failed"))):
            with self.subTest(result=result), self.assertRaises((RuntimeError, OSError)):
                self.launch_restart(repair, runtime, result=result, launch_error=error)
            self.assertTrue((self.home / "src/fabrig/queue.stop").exists())

    def test_runtime_change_during_drain_retains_stop_and_refuses_launch(self):
        repair = self.repair()
        runtime = self.runtime()
        with self.assertRaisesRegex(RuntimeError, "runtime changed"):
            self.launch_restart(repair, runtime, mutate=lambda root: (root / "always.txt").write_text("changed\n"))
        self.assertEqual(self.restart_calls, [["git", "rev-parse", "--show-toplevel"]])
        self.assertTrue((self.home / "src/fabrig/queue.stop").exists())
        record = json.loads((self.home / "src/fabrig/notes/queue.jsonl").read_text())
        self.assertEqual(record["repair"], repair)
        self.assertEqual(record["outcome"], "error")
        self.assertEqual(record["event"], "repair-refusal")

    def test_restart_invalid_runtime_or_pin_never_requests_stop(self):
        repair = self.repair()
        runtime = self.runtime()
        missing = self.root / "missing"
        missing.mkdir()
        cases = [(self.repo, repair["tip"]), (missing, repair["tip"]), (runtime, self.base)]
        for directory, tip in cases:
            with self.subTest(directory=directory), patch.object(restart, "queue_pid") as pid, \
                    self.assertRaises((ValueError, RuntimeError)):
                restart.restart(self.repo, 1800, self.home, runtime=directory,
                                repair_ready=repair["name"], repair_tip=tip, repair_reason=REASON)
            pid.assert_not_called()
            self.assertFalse((self.home / "src/fabrig/queue.stop").exists())

    def test_external_runtime_requires_complete_valid_code_and_lists(self):
        rows = [
            ("missing", lambda root: (root / "worker.py").unlink(), ValueError),
            ("syntax", lambda root: (root / "mq").write_text("invalid Python!\n"), SyntaxError),
            ("old-code", lambda root: (root / "mq").write_text("pass\n"), ValueError),
            ("empty-always", lambda root: (root / "always.txt").write_text(""), ValueError),
            ("empty-leads", lambda root: (root / "leads.txt").write_text(""), ValueError),
        ]
        for name, alter, error in rows:
            directory = self.runtime(name)
            alter(directory)
            with self.subTest(name=name), self.assertRaises(error):
                restart.runtime_module(directory, self.repo, external=True)

    def test_all_external_dependencies_missing_or_changed_during_drain_refuse_before_update(self):
        repair = self.repair()
        for index, name in enumerate(restart.RUNTIME_FILES):
            for action, alter in (("missing", lambda path: path.unlink()),
                                  ("changed", lambda path: path.write_bytes(path.read_bytes() + b"\n"))):
                runtime = self.runtime(f"dependency-{index}-{action}")
                with self.subTest(name=name, action=action), self.assertRaisesRegex(RuntimeError, "runtime"):
                    self.launch_restart(repair, runtime, mutate=lambda root: alter(root / name))
                self.assertTrue((self.home / "src/fabrig/queue.stop").exists())
                self.assertEqual(self.restart_calls, [["git", "rev-parse", "--show-toplevel"]])

    def test_external_priority_startup_and_restart_respect_persistent_ownership(self):
        repair = self.repair()
        runtime = self.runtime()
        module, _ = restart.runtime_module(runtime, self.repo, external=True)
        (self.home / "src/fabrig").mkdir(parents=True)
        layout = mq.authority.Layout(self.home / "src/fabrig/queue-authority")
        markers = [self.repo / ".git" / name for name in
                   ("mq-worker-unresolved.json", "precheck.unresolved", "mq-preparation-unresolved.json")]
        markers += [layout.root / "train-owner.json", layout.train / "state.json"]
        argv = ["mq", "once", "--repo", str(self.repo), "--authority-dir", str(layout.root),
                "--repair-ready", repair["name"], "--repair-tip", repair["tip"], "--repair-reason", REASON]
        for marker in markers:
            marker.write_bytes(b"unknown ownership")
            try:
                with self.subTest(marker=marker.name), patch.object(sys, "argv", argv), \
                        patch.object(module, "Queue") as queue, redirect_stderr(io.StringIO()):
                    self.assertEqual(module.main(), 1)
                    queue.assert_not_called()
                with self.assertRaisesRegex(RuntimeError, "reconciliation"):
                    self.launch_restart(repair, runtime)
                self.assertEqual(self.restart_calls, [["git", "rev-parse", "--show-toplevel"]])
                self.assertTrue((self.home / "src/fabrig/queue.stop").exists())
            finally:
                marker.unlink()

    def test_external_priority_cannot_enter_a_shared_authority_held_for_another_clone(self):
        repair = self.repair()
        runtime = self.runtime()
        module, _ = restart.runtime_module(runtime, self.repo, external=True)
        layout = mq.authority.Layout(self.root / "shared")
        other = self.root / "other"
        mq.git(self.root, "clone", "-b", "main", str(self.origin), str(other))
        argv = ["mq", "once", "--repo", str(other), "--authority-dir", str(layout.root),
                "--repair-ready", repair["name"], "--repair-tip", repair["tip"], "--repair-reason", REASON]
        with layout.acquire(), patch.object(sys, "argv", argv), patch.object(module, "Queue") as queue, \
                redirect_stderr(io.StringIO()):
            self.assertEqual(module.main(), 1)
            queue.assert_not_called()

    def test_external_real_child_adopts_authority_and_checkout_before_acknowledgement(self):
        repair = self.repair()
        runtime = self.runtime()
        ready_r, ready_w = os.pipe()
        release_r, release_w = os.pipe()
        child_queue = f'''class FixtureQueue(Queue):
    def run(self, stop):
        self.writer_guard()
        os.write({ready_w}, b'R')
        os.read({release_r}, 1)
Queue = FixtureQueue

'''
        script = runtime / "mq"
        script.write_text(script.read_text().replace('if __name__ == "__main__":',
                                                    child_queue + 'if __name__ == "__main__":'))
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        for name in ("cargo", "heavy", "post"):
            path = fake_bin / name
            path.write_text("#!/bin/sh\nexit 99\n")
            path.chmod(0o700)
        (self.home / "src/fabrig").mkdir(parents=True)
        layout = mq.authority.Layout(self.home / "src/fabrig/queue-authority")
        real_popen = subprocess.Popen
        children = []

        def launch(argv, **kwargs):
            if argv[0] != "git":
                kwargs["pass_fds"] += (ready_w, release_r)
            child = real_popen(argv, **kwargs)
            if argv[0] != "git":
                children.append(child)
            return child

        def command(argv, repo, **kwargs):
            return str(repo) if "--show-toplevel" in argv else self.base

        try:
            with patch.object(restart, "command", side_effect=command), \
                    patch.object(restart, "queue_pid", return_value=None), \
                    patch.object(subprocess, "Popen", side_effect=launch), \
                    patch.dict(os.environ, {"PATH": str(fake_bin) + os.pathsep + os.environ["PATH"]}), \
                    redirect_stdout(io.StringIO()):
                restart.restart(self.repo, 1800, self.home, batch=16, runtime=runtime,
                                repair_ready=repair["name"], repair_tip=repair["tip"], repair_reason=REASON)
            self.assertTrue(restart.select.select([ready_r], [], [], 10)[0])
            self.assertEqual(os.read(ready_r, 1), b"R")
            with self.assertRaises(RuntimeError):
                layout.acquire()
            with self.assertRaises(RuntimeError), mq.authority.checkout_lock(self.repo):
                pass
        finally:
            os.close(release_w)
            os.close(release_r)
            os.close(ready_r)
            os.close(ready_w)
            for child in children:
                child.wait(timeout=10)
        self.assertEqual(children[0].returncode, 0)
        with layout.acquire(), mq.authority.checkout_lock(self.repo):
            pass

    def test_restart_malformed_triplets_refuse_before_requesting_stop(self):
        repair = self.repair()
        for changes in ({"repair_tip": None}, {"repair_ready": "ready/coord/bad..ref"},
                        {"repair_tip": "0"}, {"repair_reason": "\n"}):
            options = dict(repair_ready=repair["name"], repair_tip=repair["tip"], repair_reason=REASON)
            options.update(changes)
            with self.subTest(changes=changes), patch.object(restart, "queue_pid") as pid, \
                    self.assertRaises(ValueError):
                restart.restart(self.repo, 1800, self.home, batch=16, **options)
            pid.assert_not_called()
            self.assertFalse((self.home / "src/fabrig/queue.stop").exists())


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
