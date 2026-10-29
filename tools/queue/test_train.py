#!/usr/bin/env python3
"""Disposable Git/controller worlds; no Cargo, real services, SSH or sleeps."""

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
SPEC = importlib.util.spec_from_file_location("queue_train", DIRECTORY / "train.py")
train = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(train)
worker, mq = train.worker, train.mq


class Transport:
    """Starts and completions are explicit events, independent of controller order."""

    def __init__(self, world):
        self.world = world
        self.jobs = {}
        self.starts = []
        self.fail_start = None
        self.fail_status = False
        self.fail_transfer = False

    def start(self, request, bundle):
        self.starts.append(request["attempt"])
        if self.fail_start == "before":
            raise OSError("start unreachable")
        data = Path(bundle["path"]).read_bytes()
        if self.fail_transfer or len(data) != bundle["bytes"] or worker.digest(data) != bundle["sha256"]:
            raise OSError("bundle transfer corrupt")
        identity = request["attempt"]
        if identity in self.jobs:
            if self.jobs[identity]["request"] != request:
                raise ValueError("attempt identity reused")
        else:
            repo = self.world.workers[request["gate"]]
            train.import_bundle(repo, bundle, request)
            self.jobs[identity] = {"request": copy.deepcopy(request), "state": "running"}
        if self.fail_start == "after":
            raise OSError("start acknowledgement lost")

    def identity(self):
        return {gate: {"repo": str(repo), "artifacts": str(self.world.worker_artifacts[gate])}
                for gate, repo in self.world.workers.items()}

    def status(self, request):
        if self.fail_status:
            raise OSError("status unreachable")
        return self.jobs[request["attempt"]]

    def finish(self, attempt, *, code=0, checker=None):
        job = self.jobs[attempt]
        request = job["request"]
        def runner(argv, **kwargs):
            self.world.commands.append(argv)
            kwargs["stdout"].write("fixture command\n")
            return subprocess.CompletedProcess(argv, code(argv) if callable(code) else code)
        def checks(*args, **kwargs):
            return mq.run_checks(*args, runner=runner, **kwargs)
        result = worker.run_job(self.world.workers[request["gate"]],
                                self.world.worker_artifacts[request["gate"]], request,
                                gate=request["gate"], disk_floor=0, checker=checker or checks,
                                metadata_loader=self.world.metadata)
        job.update(state="terminal", quiescent=not result["reconciliation_required"],
                   result=worker.encoded(result), log=Path(result["log"]["path"]).read_bytes())
        return result


class Authority:
    """Only this disposable adapter can push; every push targets the fixture."""

    def __init__(self, world):
        self.world = world
        self.held = True
        self.pushes = []
        self.posts = []
        self.post_code = 0
        self.lose_push_ack = False
        self.before_cleanup = None

    def guard(self):
        if not self.held:
            raise RuntimeError("authority/startup guard absent")

    def main(self):
        self.guard()
        return mq.git(self.world.origin, "rev-parse", "refs/heads/main").stdout.strip()

    def promote(self, parent, candidate, event_id):
        self.guard()
        if self.main() != parent:
            raise RuntimeError("fresh main mismatch")
        self.pushes.append((parent, candidate, event_id))
        mq.git(self.world.prep, "push", "origin", candidate + ":refs/heads/main")
        if self.lose_push_ack:
            raise OSError("push acknowledgement lost")

    def cleanup(self, name, tip, event_id):
        self.guard()
        if self.before_cleanup:
            callback, self.before_cleanup = self.before_cleanup, None
            callback()
        ref = "refs/heads/" + name
        result = mq.git(self.world.prep, "push", "--force-with-lease=" + ref + ":" + tip,
                        "origin", ":" + ref, check=False)
        # A changed/deleted ref is a completed, harmless cleanup disposition.
        current = mq.git(self.world.origin, "rev-parse", "--verify", ref, check=False).stdout.strip()
        return result.returncode == 0 or current != tip

    def post(self, event):
        self.guard()
        self.posts.append(copy.deepcopy(event))
        return self.post_code


class TrainTests(unittest.TestCase):
    def setUp(self):
        os.umask(0o077)
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT, prefix=".train-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.origin, self.prep = self.root / "origin.git", self.root / "preparation"
        self.artifacts, self.state_dir = self.root / "artifacts", self.root / "state"
        for directory in (self.artifacts, self.state_dir):
            directory.mkdir(mode=0o700)
        mq.git(self.root, "init", "--bare", str(self.origin))
        mq.git(self.root, "clone", str(self.origin), str(self.prep))
        mq.git(self.prep, "config", "user.name", "Fixture")
        mq.git(self.prep, "config", "user.email", "fixture@example.invalid")
        mq.git(self.prep, "switch", "-c", "main")
        self.names = ["core", "consumer", *worker.frozen_lines("always.txt")]
        for name in self.names:
            self.write(name + "/Cargo.toml", f'[package]\nname="{name}"\nversion="1.0.0"\n')
            self.write(name + "/src/lib.rs", "")
        self.write("core/src/deps.json", "[]")
        self.write("Cargo.lock", 'version=4\n[[package]]\nname="external"\nversion="1.0.0"\n')
        self.write("README.md", "base\n")
        self.commit("base")
        self.base = self.tip()
        mq.git(self.prep, "push", "origin", "main")
        mq.git(self.origin, "symbolic-ref", "HEAD", "refs/heads/main")
        self.workers, self.worker_artifacts = {}, {}
        for gate in ("gate-one", "gate-two"):
            repo = self.root / (gate + " checkout ; literal")
            mq.git(self.root, "clone", str(self.origin), str(repo))
            self.workers[gate] = repo
            path = self.root / (gate + " artifacts")
            path.mkdir(mode=0o700)
            self.worker_artifacts[gate] = path
        self.commands = []
        self.metadata_heads = []
        self.clock = 10000
        self.batch = 1
        self.preparer = train.Preparer(self.prep, self.artifacts, metadata_loader=self.metadata, disk_floor=0)
        self.store = train.StateStore(self.state_dir)
        self.transport, self.authority = Transport(self), Authority(self)
        self.controller = self.reload()
        self.controller.state["last_full"] = self.clock
        self.controller.save()

    def reload(self):
        return train.Controller(self.preparer, self.store, self.transport, self.authority,
                                generation="generation-one", gates=list(self.workers),
                                runtime=worker.runtime_identity(0), now=lambda: self.clock, batch=self.batch)

    def set_batch(self, batch):
        # Seed the world's policy before its first admission.
        self.batch = batch
        self.controller.config["batch"] = self.controller.state["config"]["batch"] = batch

    def write(self, name, text):
        path = self.prep / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, message):
        mq.git(self.prep, "add", ".")
        mq.git(self.prep, "commit", "-m", message)

    def tip(self):
        return mq.git(self.prep, "rev-parse", "HEAD").stdout.strip()

    def metadata(self, repo):
        self.metadata_heads.append(mq.git(repo, "rev-parse", "HEAD").stdout.strip())
        packages = [{"id": name, "name": name, "version": "1.0.0", "source": None,
                     "manifest_path": str(repo / name / "Cargo.toml")} for name in self.names]
        deps = json.loads((repo / "core/src/deps.json").read_text())
        return {"workspace_root": str(repo), "workspace_members": self.names, "packages": packages,
                "resolve": {"nodes": [{"id": name, "dependencies": deps if name == "consumer" else []}
                                       for name in self.names]}}

    def branch(self, task, changes=None, *, base=None):
        mq.git(self.prep, "switch", "--detach", base or self.base)
        for name, value in (changes or {task + ".md": task}).items():
            self.write(name, value)
        self.commit(task)
        tip = self.tip()
        name = "ready/ops/" + task
        mq.git(self.prep, "push", "origin", tip + ":refs/heads/" + name)
        return {"name": name, "tip": tip}

    def admit(self):
        attempt = self.controller.admit()
        self.assertIsNotNone(attempt)
        record = self.controller.state["records"][attempt]
        self.assertNotEqual(record["outcome"], "INFRA", record)
        return attempt, record

    def finish(self, attempt, **kwargs):
        result = self.transport.finish(attempt, **kwargs)
        self.controller.poll(attempt)
        return result

    def test_successor_finishes_first_lands_original_candidates_in_order(self):
        first = self.branch("a")
        second = self.branch("b")
        a, r1 = self.admit()
        b, r2 = self.admit()
        c1, c2 = r1["candidate"], r2["candidate"]
        self.assertEqual(r2["parent"], c1)
        self.assertEqual(self.authority.main(), self.base)
        self.assertEqual(mq.git(self.prep, "rev-parse", "origin/main").stdout.strip(), self.base)
        self.assertIsNone(self.controller.admit())
        self.finish(b)
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.authority.posts, [])
        self.finish(a)
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(self.authority.main(), c1)
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(self.authority.main(), c2)
        self.assertEqual([(p, c) for p, c, _ in self.authority.pushes], [(self.base, c1), (c1, c2)])
        self.assertEqual(self.controller.state["tried"], {first["name"]: first["tip"], second["name"]: second["tip"]})
        self.assertTrue(self.controller.drained())
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["batches_since_full"], 2)
        self.controller.flush_outbox()
        self.assertEqual(len(self.authority.posts), 2)

    def test_parent_relative_selection_and_candidate_dependency_graph(self):
        self.branch("a", {"core/src/deps.json": '["core"]'})
        self.branch("b", {"core/src/lib.rs": "new source"})
        a, r1 = self.admit()
        b, r2 = self.admit()
        self.assertEqual(r2["expected"]["changed_paths"], ["core/src/lib.rs"])
        self.assertIn("consumer", r2["request"]["selection"])
        self.assertEqual(r2["parent"], r1["candidate"])
        self.assertEqual(self.finish(b)["metadata"], r2["expected"]["metadata"])
        self.finish(a)
        self.controller.adjudicate_head()
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), r2["candidate"])

    def test_parent_relative_lock_ignores_predecessor_lock_edits(self):
        original = (self.prep / "Cargo.lock").read_text()
        self.branch("a", {"Cargo.lock": original.replace('1.0.0', '2.0.0')})
        self.branch("b", {"later.md": "second"})
        self.admit()
        b, r2 = self.admit()
        self.assertEqual(r2["expected"]["changed_paths"], ["later.md"])
        self.assertEqual(r2["request"]["selection"], "docs")
        self.assertEqual(self.finish(b)["outcome"], "green")

    def test_name_reservations_advance_and_delete_selected_tips(self):
        original = self.branch("a")
        a, record = self.admit()
        advanced = self.branch("a", {"a.md": "advanced"}, base=original["tip"])
        self.branch("b")
        b, successor = self.admit()
        self.assertEqual([i["name"] for i in successor["inputs"]], ["ready/ops/b"])
        mq.git(self.origin, "update-ref", "-d", "refs/heads/ready/ops/a")
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])
        self.assertEqual(self.controller.state["tried"][original["name"]], original["tip"])
        mq.git(self.origin, "update-ref", "refs/heads/ready/ops/a", advanced["tip"])
        self.controller.flush_outbox()
        self.assertEqual(mq.git(self.origin, "rev-parse", "refs/heads/ready/ops/a").stdout.strip(), advanced["tip"])

    def test_compare_delete_preserves_tip_changed_at_cleanup(self):
        old = self.branch("a")
        a, _ = self.admit()
        newer = self.branch("a", {"a.md": "newer"}, base=old["tip"])
        mq.git(self.origin, "update-ref", "refs/heads/ready/ops/a", old["tip"])
        self.finish(a)
        self.controller.adjudicate_head()
        self.authority.before_cleanup = lambda: mq.git(
            self.origin, "update-ref", "refs/heads/ready/ops/a", newer["tip"])
        self.controller.flush_outbox()
        self.assertEqual(mq.git(self.origin, "rev-parse", "refs/heads/ready/ops/a").stdout.strip(), newer["tip"])

    def test_red_head_invalidates_successor_and_bisects_sequentially(self):
        self.set_batch(3)
        inputs = [self.branch(name, {"core/src/" + name + ".rs": name}) for name in ("a", "b", "c")]
        self.branch("d")
        a, red = self.admit()
        b, successor = self.admit()
        self.finish(a, code=1)
        self.assertTrue(successor["invalidated"])
        self.controller.adjudicate_head()
        self.assertIsNone(self.controller.admit())
        self.finish(b)
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(self.controller.state["tried"], {})
        landed = []
        while self.controller.state["pending"]:
            attempt, record = self.admit()
            self.assertEqual(record["parent"], self.authority.main())
            if any(i["name"].endswith("/b") for i in record["inputs"]):
                self.finish(attempt, code=1)
            else:
                self.finish(attempt)
                landed.append(record["candidate"])
            self.controller.adjudicate_head()
        self.assertEqual(len(landed), 2)
        self.assertEqual(self.authority.main(), landed[-1])
        rejected = [e["branch"]["name"] for e in self.controller.state["outbox"].values() if e["kind"] == "rejected"]
        self.assertEqual(rejected, [inputs[1]["name"]])
        self.assertNotIn("ready/ops/d", self.controller.state["tried"])
        self.assertIsNone(self.controller.admit())  # Ends bisection without speculating in that call.

    def test_full_failure_keeps_all_bisection_groups_full(self):
        self.set_batch(2)
        self.controller.state["batches_since_full"] = 4
        self.branch("a", {"core/src/a.rs": "a"})
        self.branch("b", {"core/src/b.rs": "b"})
        a, record = self.admit()
        self.assertEqual(record["request"]["selection"], "full")
        self.finish(a, code=1)
        self.controller.adjudicate_head()
        self.controller = self.reload()
        for _ in range(2):
            attempt, group = self.admit()
            self.assertEqual(group["request"]["selection"], "full")
            self.finish(attempt)
            self.controller.adjudicate_head()

    def test_unknown_start_repeats_same_identity_without_duplicate_job(self):
        self.branch("a")
        self.transport.fail_start = "after"
        a = self.controller.admit()
        record = self.controller.state["records"][a]
        self.assertEqual(record["outcome"], "INFRA")
        self.assertEqual(record["phase"], "unknown")
        self.assertEqual(self.store.load()["gates"][record["gate"]], a)
        self.transport.fail_start = None
        self.controller = self.reload()
        self.controller.dispatch(a)
        self.assertEqual(self.transport.starts, [a, a])
        self.assertEqual(len(self.transport.jobs), 1)
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(len(self.authority.pushes), 1)

    def test_status_failure_retains_ownership_invalidates_successor_and_rejects_nothing(self):
        self.branch("a")
        self.branch("b")
        a, r1 = self.admit()
        b, r2 = self.admit()
        self.transport.fail_status = True
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["gates"][r1["gate"]], a)
        self.assertTrue(r2["invalidated"])
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])
        self.transport.fail_status = False
        self.finish(b)
        self.assertEqual(self.authority.pushes, [])
        with self.assertRaises(ValueError):
            self.controller.reconcile_job(a, quiescent=False, evidence="process absent")
        self.controller.reconcile_job(a, quiescent=True, evidence="fixture supervisor and checkout reconciled")
        self.assertTrue(self.controller.drained())

    def test_transfer_error_and_missing_objects_are_infrastructure(self):
        self.branch("a")
        self.transport.fail_transfer = True
        a = self.controller.admit()
        record = self.controller.state["records"][a]
        self.assertEqual(record["phase"], "unknown")
        self.assertEqual(self.controller.state["gates"][record["gate"]], a)
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])
        missing = copy.deepcopy(record["request"])
        missing["candidate"] = "0" * 40
        with self.assertRaises(RuntimeError):
            worker.verify_objects(self.workers[record["gate"]], missing)

    def test_malformed_result_holds_gate_and_invalidates_descendants(self):
        self.branch("a")
        self.branch("b")
        a, record = self.admit()
        b, successor = self.admit()
        self.transport.finish(a)
        self.transport.jobs[a]["result"] = b'{"version":1,"version":1}'
        self.controller.poll(a)
        self.assertEqual(record["outcome"], "INFRA")
        self.assertTrue(successor["invalidated"])
        self.assertEqual(self.controller.state["gates"][record["gate"]], a)
        self.controller.adjudicate_head()
        self.finish(b)
        self.assertFalse(self.authority.pushes)
        self.assertEqual(self.controller.state["tried"], {})

    def test_all_certificate_identity_plan_snapshots_steps_events_and_log_corruption(self):
        self.branch("a", {"core/src/lib.rs": "code"})
        a, record = self.admit()
        good = self.transport.finish(a)
        log = self.transport.jobs[a]["log"]
        self.assertEqual(train.validate_result(self.preparer, record, good, log), "green")
        corruptions = [
            ("version", 2), ("requested.generation", "other"), ("requested.attempt", "other"),
            ("requested.gate", "other"), ("requested.parent", "0" * 40),
            ("requested.candidate", "0" * 40), ("requested.branches", []),
            ("requested.runtime", {}), ("requested.branches.0.tip", "0" * 40),
            ("observed.gate", "other"), ("observed.runtime", {}),
            ("observed.parent", "0" * 40), ("observed.candidate", "0" * 40),
            ("observed.candidate_tree", "0" * 40), ("observed.branches", []),
            ("changed_paths", []), ("selection", "docs"), ("metadata.sha256", "x"),
            ("plan", []), ("steps", []), ("events", []), ("before.clean", False),
            ("before.clean", 1), ("after.head", self.base), ("initial.clean", False),
            ("events.0.checked.tree", "0" * 40), ("completion_known", False),
            ("reconciliation_required", True), ("log.bytes", 0), ("log.sha256", "x"),
            ("log.complete", False), ("error", "failure"), ("outcome", "red"),
        ]
        for path, value in corruptions:
            with self.subTest(path=path):
                broken = copy.deepcopy(good)
                target = broken
                parts = path.split(".")
                for key in parts[:-1]:
                    target = target[int(key) if isinstance(target, list) else key]
                target[parts[-1]] = value
                with self.assertRaises((ValueError, RuntimeError)):
                    train.validate_result(self.preparer, record, broken, log)
        for field, value in [("command", ["true"]), ("cwd", "core"), ("returncode", 1),
                             ("duration", -1), ("green", False)]:
            with self.subTest(step_field=field):
                broken = copy.deepcopy(good)
                broken["steps"][0][field] = value
                with self.assertRaises((ValueError, RuntimeError)):
                    train.validate_result(self.preparer, record, broken, log)
        for bad_log in (log[:-1], log + b"extra", b""):
            with self.assertRaises(ValueError):
                train.validate_result(self.preparer, record, good, bad_log)

    def test_tools_evidence_all_suites_and_red_prefix_nextest_four(self):
        self.branch("a", {"tools/test_a.py": "", "tools/test_b.py": "", "core/src/lib.rs": "code"})
        a, record = self.admit()
        result = self.transport.finish(a, code=lambda argv: 4 if "nextest" in argv else 0)
        self.assertEqual(train.validate_result(self.preparer, record, result, self.transport.jobs[a]["log"]), "green")
        tools = next(s for s in result["steps"] if s["name"] == "tools")
        self.assertEqual(len(tools["suites"]), 2)
        bad = copy.deepcopy(result)
        bad["steps"][0]["suites"].pop()
        with self.assertRaises(ValueError):
            train.validate_result(self.preparer, record, bad, self.transport.jobs[a]["log"])
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.branch("b", {"tools/test_c.py": "", "tools/test_d.py": ""})
        b, record = self.admit()
        red = self.finish(b, code=4)
        self.assertEqual(red["outcome"], "red")
        self.assertEqual(len(red["steps"]), 1)
        self.assertEqual(len(red["steps"][0]["suites"]), 4)
        self.assertEqual(record["outcome"], "red")

    def test_fresh_main_mismatch_blocks_promotion(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        unexpected = self.branch("other")
        mq.git(self.origin, "update-ref", "refs/heads/main", unexpected["tip"])
        self.assertFalse(self.controller.adjudicate_head())
        self.assertNotIn("promotion", record)
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(self.controller.state["tried"], {})

    def test_lost_push_ack_reconciles_exact_candidate_once_after_reload(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        self.authority.lose_push_ack = True
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(record["phase"], "promotion-intent")
        self.assertEqual(self.authority.main(), record["candidate"])
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.controller = self.reload()
        self.assertEqual(len(self.controller.recovery_inventory()["intents"]), 1)
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(len(self.authority.pushes), 1)
        self.assertEqual(self.controller.state["batches_since_full"], 1)
        self.controller = self.reload()
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["batches_since_full"], 1)

    def test_outbox_failed_delivery_and_replay_stable_id_exit_three(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        self.authority.post_code = 2
        self.controller.flush_outbox()
        event = self.authority.posts[-1]
        self.assertIn(event["id"], event["body"])
        self.assertFalse(self.controller.state["outbox"][event["id"]]["delivered"])
        self.assertEqual(self.authority.main(), record["candidate"])
        self.controller = self.reload()
        self.authority.post_code = 3
        self.controller.flush_outbox()
        self.assertEqual(self.authority.posts[-1]["id"], event["id"])
        self.assertTrue(self.controller.state["outbox"][event["id"]]["delivered"])

    def test_conservative_fifth_hourly_idle_and_unresolved_full_reservations(self):
        self.controller.state["batches_since_full"] = 3
        self.branch("a")
        self.branch("b")
        a, first = self.admit()
        b, second = self.admit()
        self.assertEqual(first["request"]["selection"], "docs")
        self.assertEqual(second["request"]["selection"], "full")
        self.finish(b)
        self.assertEqual(self.controller.state["batches_since_full"], 3)
        self.controller.state["batches_since_full"] = 4
        self.assertTrue(self.controller.full_due())
        self.controller.state["batches_since_full"] = 0
        self.clock += 3600
        self.assertTrue(self.controller.full_due())

    def test_unresolved_full_does_not_reset_successor_cadence(self):
        self.controller.state["batches_since_full"] = 4
        self.branch("a")
        self.branch("b")
        self.admit()
        _, successor = self.admit()
        self.assertEqual(successor["request"]["selection"], "full")

    def test_idle_full_has_equal_parent_candidate_and_no_push(self):
        self.clock += 600
        a, record = self.admit()
        self.assertEqual(record["parent"], record["candidate"])
        self.assertEqual(record["request"]["selection"], "full")
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(self.controller.state["last_full_main"], self.base)
        self.clock += 600
        self.assertIsNone(self.controller.admit())

    def test_stop_closes_admission_and_drains_without_replacements(self):
        self.branch("a")
        self.branch("b")
        self.branch("c")
        a, _ = self.admit()
        b, _ = self.admit()
        self.controller.stop()
        self.controller = self.reload()
        self.assertFalse(self.controller.drained())
        self.finish(b)
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertIsNone(self.controller.admit())
        self.controller.adjudicate_head()
        self.assertTrue(self.controller.drained())
        self.assertIsNone(self.controller.admit())
        self.assertEqual(len(self.authority.pushes), 2)

    def test_stop_after_red_retains_pending_bisection(self):
        self.set_batch(2)
        self.branch("a", {"core/src/a.rs": "a"})
        self.branch("b", {"core/src/b.rs": "b"})
        a, _ = self.admit()
        self.controller.stop()
        self.finish(a, code=1)
        self.controller.adjudicate_head()
        self.assertTrue(self.controller.drained())
        self.assertEqual(len(self.controller.state["pending"]), 2)
        self.assertIsNone(self.controller.admit())

    def test_state_version_modes_generation_and_authority_guard(self):
        self.assertEqual(self.store.path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(self.state_dir.stat().st_mode & 0o777, 0o700)
        self.authority.held = False
        with self.assertRaises(RuntimeError):
            self.controller.admit()
        self.authority.held = True
        state = self.store.load()
        state["version"] = 99
        self.store.save(state)
        with self.assertRaises(ValueError):
            self.reload()
        state["version"] = 1
        state["config"]["runtime"] = {}
        self.store.save(state)
        with self.assertRaises(ValueError):
            self.reload()

    def test_state_failure_before_start_prevents_transport_and_requires_reload(self):
        self.branch("a")
        original = self.store.save
        def save(state):
            if any(r["phase"] == "dispatch-intent" for r in state["records"].values()):
                raise OSError("injected fsync failure")
            original(state)
        with patch.object(self.store, "save", side_effect=save):
            with self.assertRaises(OSError):
                self.controller.admit()
        self.assertEqual(self.transport.starts, [])
        with self.assertRaises(RuntimeError):
            self.controller.admit()
        self.controller = self.reload()
        attempt = self.controller.state["active"][0]
        self.assertEqual(self.controller.state["records"][attempt]["phase"], "prepared")
        self.controller.dispatch(attempt)
        self.finish(attempt)
        self.controller.adjudicate_head()

    def test_merge_chain_and_contained_input_are_verified(self):
        branch = self.branch("a")
        a, record = self.admit()
        train.verify_chain(self.prep, record)
        broken = copy.deepcopy(record)
        broken["chain"][0]["before"] = "0" * 40
        with self.assertRaises(ValueError):
            train.verify_chain(self.prep, broken)
        self.finish(a)
        self.controller.adjudicate_head()
        # Another ready name for the already-contained exact input is explicit.
        mq.git(self.origin, "update-ref", "refs/heads/ready/ops/b", branch["tip"])
        b, contained = self.admit()
        self.assertTrue(contained["chain"][0]["contained"])
        self.assertEqual(contained["parent"], contained["candidate"])
        self.finish(b)
        self.controller.adjudicate_head()
        self.assertEqual(len(self.authority.pushes), 1)

    def ratchet_base(self):
        mq.git(self.prep, "switch", "--detach", self.base)
        self.write("skein/scripts/old-forms.txt", "3\n")
        self.write("skein/src/a.rs", "old old\n")
        self.write("skein/src/b.rs", "old\n")
        self.write("skein/scripts/old_forms.py", '''from pathlib import Path
import sys
baseline = Path('scripts/old-forms.txt')
count = sum(p.read_text().count('old') for p in Path('src').glob('*.rs'))
if count > int(baseline.read_text()):
    sys.exit(1)
if '--write' in sys.argv:
    baseline.write_text(str(count) + '\\n')
''')
        self.commit("ratchet fixture")
        self.base = self.tip()
        mq.git(self.prep, "push", "origin", self.base + ":main")

    def test_real_ratchet_lowers_before_freeze_and_worker_cannot_substitute(self):
        self.ratchet_base()
        self.branch("a", {"skein/src/a.rs": ""})
        a, record = self.admit()
        self.assertEqual(len(record["baseline_commits"]), 1)
        frozen = record["candidate"]
        self.assertEqual(mq.git(self.prep, "show", frozen + ":skein/scripts/old-forms.txt").stdout, "1\n")
        self.assertEqual(self.commands, [])  # Preparation ran only the fixture ratchet.
        train.verify_chain(self.prep, record)
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), frozen)
        self.assertEqual(mq.git(self.workers[record["gate"]], "rev-parse", "HEAD").stdout.strip(), frozen)

    def test_ratchet_raise_is_red_and_execution_error_is_infra(self):
        self.ratchet_base()
        branch = self.branch("a", {"skein/src/a.rs": "old old old old\n"})
        a, record = self.admit()
        self.assertEqual(record["outcome"], "red")
        self.assertEqual(self.transport.starts, [])
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {branch["name"]: branch["tip"]})
        self.controller.admit()  # Finish singleton bisection.
        self.branch("c", {"skein/scripts/old_forms.py": "import sys; sys.exit(2)\n"})
        attempt = self.controller.admit()
        infra = self.controller.state["records"][attempt]
        self.assertEqual(infra["outcome"], "INFRA")
        self.controller.adjudicate_head()
        self.assertNotIn("ready/ops/c", self.controller.state["tried"])

    def test_conflict_blocks_speculation_and_later_error_rolls_back_rejections(self):
        self.set_batch(2)
        self.branch("a", {"README.md": "left"})
        conflict = self.branch("b", {"README.md": "right"})
        self.branch("c")
        a, record = self.admit()
        self.assertEqual(record["outcome"], "conflict")
        self.assertIsNone(self.controller.admit())
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {conflict["name"]: conflict["tip"]})
        a, retry = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), retry["candidate"])

    def test_error_after_merge_conflict_rejects_neither_branch(self):
        self.set_batch(2)
        self.branch("a", {"README.md": "left", "core/src/lib.rs": "source"})
        self.branch("b", {"README.md": "right"})
        with patch.object(self.preparer, "metadata_loader", side_effect=RuntimeError("metadata unavailable")):
            a = self.controller.admit()
        record = self.controller.state["records"][a]
        self.assertEqual(record["outcome"], "INFRA")
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])

    def test_successor_lock_change_compares_its_parent_not_lagging_main(self):
        original = (self.prep / "Cargo.lock").read_text()
        first = self.branch("a", {"Cargo.lock": original.replace("1.0.0", "2.0.0")})
        self.branch("b", {"Cargo.lock": original, "second.md": "second"}, base=first["tip"])
        ordinary_metadata = self.metadata
        def metadata(repo):
            data = ordinary_metadata(repo)
            version = mq.tomllib.loads((repo / "Cargo.lock").read_text())["package"][0]["version"]
            data["packages"].append({"id": "external", "name": "external", "version": version,
                                     "source": None})
            data["resolve"]["nodes"].append({"id": "external", "dependencies": []})
            next(n for n in data["resolve"]["nodes"] if n["id"] == "core")["dependencies"] = ["external"]
            return data
        self.metadata = self.preparer.metadata_loader = metadata
        self.admit()
        b, record = self.admit()
        self.assertEqual(record["expected"]["changed_paths"], ["Cargo.lock", "second.md"])
        self.assertIn("core", record["request"]["selection"])
        self.assertEqual(mq.git(self.prep, "show", "origin/main:Cargo.lock").stdout,
                         mq.git(self.prep, "show", record["candidate"] + ":Cargo.lock").stdout)
        self.assertEqual(self.finish(b)["outcome"], "green")

    def test_successor_red_has_no_rejection_until_predecessor_lands(self):
        self.branch("a")
        bad = self.branch("b", {"core/src/lib.rs": "bad"})
        a, _ = self.admit()
        b, successor = self.admit()
        self.finish(b, code=1)
        self.assertEqual(self.controller.state["tried"], {})
        self.assertFalse(self.controller.adjudicate_head())
        self.assertIsNone(self.controller.admit())
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), successor["parent"])
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"][bad["name"]], bad["tip"])
        self.assertEqual(len(self.authority.pushes), 1)

    def test_successor_conflict_is_discarded_after_ancestor_infrastructure_failure(self):
        self.branch("a", {"README.md": "left"})
        self.branch("b", {"README.md": "right"})
        a, _ = self.admit()
        _, successor = self.admit()
        self.assertEqual(successor["outcome"], "conflict")
        self.assertIsNone(self.controller.admit())
        self.transport.fail_status = True
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.assertTrue(successor["invalidated"])
        self.assertEqual(successor["phase"], "discarded")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["batches_since_full"], 0)

    def test_heavy_completion_unknown_never_frees_checkout_or_admits_replacement(self):
        self.branch("a", {"core/src/lib.rs": "code"})
        self.branch("b")
        a, head = self.admit()
        b, _ = self.admit()
        result = self.finish(a, code=lambda argv: 137 if argv[0] == "heavy" else 0)
        self.assertFalse(result["completion_known"])
        self.assertTrue((self.workers[head["gate"]] / ".git/mq-worker-unresolved.json").exists())
        self.controller.adjudicate_head()
        self.finish(b)
        self.assertEqual(self.controller.state["gates"][head["gate"]], a)
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.assertFalse(self.controller.drained())

    def test_quiescence_cannot_be_asserted_by_result_without_supervisor_boundary(self):
        self.branch("a")
        a, record = self.admit()
        self.transport.finish(a)
        job = self.transport.jobs[a]
        with self.assertRaises(ValueError):
            self.controller.ingest(a, job["result"], job["log"])
        job["quiescent"] = False
        self.controller.poll(a)
        self.assertEqual(record["outcome"], "INFRA")
        self.assertEqual(self.controller.state["gates"][record["gate"]], a)

    def test_result_publication_before_state_failure_can_replay_same_evidence(self):
        self.branch("a")
        a, _ = self.admit()
        self.transport.finish(a)
        original = self.store.save
        def save(state):
            if state["records"][a]["phase"] == "terminal":
                raise OSError("result-state publication failure")
            original(state)
        with patch.object(self.store, "save", side_effect=save):
            with self.assertRaises(OSError):
                self.controller.poll(a)
        self.controller = self.reload()
        self.controller.poll(a)
        self.assertEqual(self.controller.state["records"][a]["outcome"], "green")
        self.controller.adjudicate_head()
        self.assertEqual(len(self.authority.pushes), 1)

    def test_dispatch_ack_state_failure_replays_same_job(self):
        self.branch("a")
        original = self.store.save
        def save(state):
            if any(r["phase"] == "running" for r in state["records"].values()):
                raise OSError("dispatch acknowledgement state failure")
            original(state)
        with patch.object(self.store, "save", side_effect=save):
            with self.assertRaises(OSError):
                self.controller.admit()
        self.controller = self.reload()
        a = self.controller.state["active"][0]
        self.assertEqual(self.controller.state["records"][a]["phase"], "dispatch-intent")
        self.controller.dispatch(a)
        self.assertEqual(len(self.transport.jobs), 1)
        self.finish(a)
        self.controller.adjudicate_head()

    def test_decision_state_failure_after_push_replays_counters_and_outbox_once(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        original = self.store.save
        def save(state):
            if state["records"][a]["applied"]:
                raise OSError("decision publication failure")
            original(state)
        with patch.object(self.store, "save", side_effect=save):
            with self.assertRaises(OSError):
                self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])
        self.controller = self.reload()
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["batches_since_full"], 1)
        self.assertEqual(len(self.controller.state["outbox"]), 2)
        self.assertEqual(len(self.authority.pushes), 1)

    def test_outbox_ack_state_failure_repeats_message_with_same_id(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        original = self.store.save
        def save(state):
            if any(e["delivered"] for e in state["outbox"].values()):
                raise OSError("post acknowledgement publication failure")
            original(state)
        with patch.object(self.store, "save", side_effect=save):
            with self.assertRaises(OSError):
                self.controller.flush_outbox()
        self.assertEqual(len(self.authority.posts), 1)
        self.controller = self.reload()
        self.controller.flush_outbox()
        self.assertEqual(len(self.authority.posts), 2)
        self.assertEqual(self.authority.posts[0]["id"], self.authority.posts[1]["id"])
        self.assertEqual(self.controller.state["batches_since_full"], 1)

    def test_preparation_interruption_stays_reserved_and_blocks_admission(self):
        self.branch("a")
        with patch.object(self.preparer, "prepare_candidate", side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.controller.admit()
        self.controller = self.reload()
        inventory = self.controller.recovery_inventory()
        self.assertEqual(len(inventory["preparing"]), 1)
        self.assertIn("ready/ops/a", self.controller.reservations())
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.transport.starts, [])

    def test_pins_bundles_and_evidence_remain_private_without_origin_speculation(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        refs = mq.git(self.prep, "for-each-ref", "--format=%(refname)", "refs/mq/jobs/").stdout.splitlines()
        self.assertEqual(set(refs), set(record["pins"]))
        self.assertEqual(mq.git(self.origin, "for-each-ref", "refs/mq/").stdout, "")
        for repo in self.workers.values():
            self.assertEqual(mq.git(repo, "for-each-ref", "refs/mq/").stdout, "")
        job = self.artifacts / a
        self.assertEqual(job.stat().st_mode & 0o777, 0o700)
        for path in job.iterdir():
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertTrue((job / "returned.log").exists())
        self.assertTrue((job / "returned.json").exists())

    def test_bundle_corruption_refs_and_missing_prerequisites_are_refused(self):
        self.branch("a")
        self.branch("b")
        _, first = self.admit()
        _, second = self.admit()
        repo = self.root / "fresh import"
        mq.git(self.root, "clone", str(self.origin), str(repo))
        corrupt = copy.deepcopy(second["bundle"])
        corrupt["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            train.import_bundle(repo, corrupt, second["request"])
        corrupt = copy.deepcopy(second["bundle"])
        corrupt["refs"] = {}
        with self.assertRaises(ValueError):
            train.import_bundle(repo, corrupt, second["request"])
        path = self.root / "incremental.bundle"
        ref = next(r for r in second["pins"] if r.endswith("/candidate"))
        mq.git(self.prep, "bundle", "create", str(path), ref, "^" + first["candidate"])
        data = path.read_bytes()
        incremental = {"path": str(path), "bytes": len(data), "sha256": worker.digest(data),
                       "refs": {ref: second["candidate"]}}
        with self.assertRaises(RuntimeError):
            train.import_bundle(repo, incremental, second["request"])
        self.assertEqual(self.controller.state["tried"], {})

    def test_ratchet_conflict_amendment_preserves_exact_merge_messages(self):
        self.ratchet_base()
        self.set_batch(2)
        self.branch("a", {"skein/src/a.rs": "", "skein/scripts/old-forms.txt": "1\n"})
        self.branch("b", {"skein/src/b.rs": "", "skein/scripts/old-forms.txt": "2\n"})
        a, record = self.admit()
        self.assertEqual(record["conflicts"], [])
        self.assertEqual(mq.git(self.prep, "show", record["candidate"] + ":skein/scripts/old-forms.txt").stdout, "0\n")
        train.verify_chain(self.prep, record)
        for branch in record["chain"]:
            self.assertEqual(mq.git(self.prep, "show", "-s", "--format=%s", branch["commit"]).stdout.strip(),
                             "Merge " + branch["name"])
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])

    def test_full_selection_gap_and_infra_accounting(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        self.controller.state["batches_since_full"] = 4
        self.branch("b")
        b, _ = self.admit()
        self.finish(b, code=1)
        self.controller.adjudicate_head()
        self.assertTrue(any(e["kind"] == "selection-gap" for e in self.controller.state["outbox"].values()))
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.controller.admit()  # Finish singleton bisection.
        self.branch("c")
        c, _ = self.admit()
        self.transport.fail_status = True
        self.controller.poll(c)
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["batches_since_full"], 0)

    def test_generation_gate_configuration_cannot_change_in_place(self):
        self.workers["gate-one"] = self.root / "replacement"
        with self.assertRaises(ValueError):
            self.controller.admit()
        with self.assertRaises(ValueError):
            self.reload()

    def test_red_against_unexpected_main_never_rejects(self):
        self.branch("a", {"core/src/lib.rs": "bad"})
        a, record = self.admit()
        self.finish(a, code=1)
        other = self.branch("other")
        mq.git(self.origin, "update-ref", "refs/heads/main", other["tip"])
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])

    def test_failed_promotion_retries_same_explicit_intent_on_parent(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        with patch.object(self.authority, "promote", side_effect=OSError("push unavailable")):
            self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.authority.main(), record["parent"])
        intent = copy.deepcopy(record["promotion"])
        self.controller = self.reload()
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(self.authority.pushes, [(intent["parent"], intent["candidate"], intent["id"])])

    def test_unexpected_main_during_intent_blocks_replay_and_admission(self):
        self.branch("a")
        a, record = self.admit()
        self.finish(a)
        with patch.object(self.authority, "promote", side_effect=OSError("push unavailable")):
            self.controller.adjudicate_head()
        other = self.branch("other")
        mq.git(self.origin, "update-ref", "refs/heads/main", other["tip"])
        self.controller = self.reload()
        self.assertFalse(self.controller.adjudicate_head())
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(self.controller.state["tried"], {})

    def test_shared_preparation_or_gate_clone_is_refused(self):
        self.workers["gate-one"] = self.prep
        with self.assertRaises(ValueError):
            self.reload()
        self.workers["gate-one"] = self.workers["gate-two"]
        with self.assertRaises(ValueError):
            self.reload()

    def test_stop_without_jobs_is_drained(self):
        self.controller.stop()
        self.assertTrue(self.controller.drained())
        self.assertIsNone(self.controller.admit())

    def test_stop_one_job_drains_and_never_starts_second(self):
        self.branch("a")
        self.branch("b")
        a, _ = self.admit()
        self.controller.stop()
        self.assertIsNone(self.controller.admit())
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertTrue(self.controller.drained())
        self.assertEqual(len(self.transport.jobs), 1)

    def test_worker_infrastructure_retains_raw_evidence_and_no_counter_decision(self):
        self.branch("a", {"core/src/lib.rs": "source"})
        a, record = self.admit()
        def checker(*args, **kwargs):
            raise RuntimeError("fixture checker unavailable")
        result = self.finish(a, checker=checker)
        self.assertEqual(result["outcome"], "INFRA")
        self.assertEqual(record["outcome"], "INFRA")
        self.assertEqual((self.artifacts / a / "returned.json").read_bytes(), worker.encoded(result))
        self.assertEqual((self.artifacts / a / "returned.log").read_bytes(), self.transport.jobs[a]["log"])
        self.assertEqual(self.controller.state["gates"][record["gate"]], a)
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["batches_since_full"], 0)

    def test_maximum_generation_token_still_has_valid_unique_attempt_identity(self):
        alternate = self.root / "alternate state"
        alternate.mkdir(mode=0o700)
        self.branch("a")
        self.controller = train.Controller(
            self.preparer, train.StateStore(alternate), self.transport, self.authority,
            generation="g" * 128, gates=list(self.workers), runtime=worker.runtime_identity(0),
            now=lambda: self.clock, batch=1)
        a, record = self.admit()
        self.assertLessEqual(len(a), 128)
        self.assertEqual(record["request"]["generation"], "g" * 128)
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])


if __name__ == "__main__":
    unittest.main()
