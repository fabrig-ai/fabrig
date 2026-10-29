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

    def completion_identity(self):
        return {"kind": "fixture-completion-v1"}

    def verify_completion(self, request, evidence, endpoint):
        # The fake checker returns synchronously and never launches heavy.
        # An explicit finish event, the exact request and durable result are
        # required. A boolean, missing process or released lock proves nothing.
        job = self.jobs[request["attempt"]]
        if (job["state"] != "terminal" or job["request"] != request or
                not isinstance(evidence, dict) or evidence != job["evidence"]):
            raise ValueError("fixture completion event absent")
        result = train.read_json(job["result"])
        if result["completion_known"] is not True or result["reconciliation_required"] is not False:
            raise ValueError("fixture commands unresolved")
        with mq.authority.checkout_lock(Path(endpoint["repo"])):
            stored = train.durable.read_bytes(Path(endpoint["artifacts"]) / request["attempt"] / "result.json")
            if worker.digest(stored) != evidence["sha256"]:
                raise ValueError("fixture evidence changed")
        return evidence

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
                   result=worker.encoded(result), log=Path(result["log"]["path"]).read_bytes(),
                   evidence={"kind": "fixture-completion-v1", "attempt": attempt,
                             "sha256": worker.digest(worker.encoded(result))})
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



class TrainFixture:
    def setUp(self):
        os.umask(0o077)
        self.temporary = tempfile.TemporaryDirectory(dir=ROOT.parent, prefix=".train-test-")
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
                                runtime=worker.runtime_identity(0), now=lambda: self.clock, batch=self.batch,
                                completion_adapter=self.transport)


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


    def partial_conflict_inputs(self):
        self.set_batch(3)
        left = self.branch("a", {"README.md": "left"})
        conflict = self.branch("b", {"README.md": "right"})
        source = self.branch("c", {"core/src/lib.rs": "unrelated source"})
        return [left, source], conflict


    def partial_conflict_attempt(self):
        merged, conflict = self.partial_conflict_inputs()
        attempt, record = self.admit()
        self.assertEqual(record["request"]["branches"], merged)
        self.assertEqual(record["parent"], self.base)
        self.assertEqual(record["phase"], "running")
        self.assertEqual([b["name"] for b in record["conflicts"]], [conflict["name"]])
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertEqual(self.authority.posts, [])
        self.assertFalse(self.controller.adjudicate_head())
        self.assertIsNone(self.controller.admit())
        return attempt, record, merged, conflict


    def assert_partial_conflict_infra(self, attempt, record):
        self.assertEqual(record["outcome"], "INFRA")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(record["decision"], "INFRA")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])
        self.assertFalse(self.controller.state["bisecting"])
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.assertEqual(self.authority.main(), self.base)
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(self.controller.state["gates"][record["gate"]], attempt)
        self.assertFalse(self.controller.drained())
        self.assertIsNone(self.controller.admit())
        self.controller.flush_outbox()
        self.assertEqual([event["kind"] for event in self.authority.posts], ["infrastructure"])


