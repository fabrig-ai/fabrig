#!/usr/bin/env python3
"""Candidate construction, evidence and full-check policy."""

import sys
sys.dont_write_bytecode = True

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent
SPEC = importlib.util.spec_from_file_location("train_fixture", DIRECTORY / "train_fixture.py")
fixture = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixture)
train, worker, mq = fixture.train, fixture.worker, fixture.mq


class PreparationTests(fixture.TrainFixture, unittest.TestCase):
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



    def test_preparation_metadata_error_after_merge_conflict_rejects_neither_branch(self):
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



    def test_shared_preparation_or_gate_clone_is_refused(self):
        self.workers["gate-one"] = self.prep
        with self.assertRaises(ValueError):
            self.reload()
        self.workers["gate-one"] = self.workers["gate-two"]
        with self.assertRaises(ValueError):
            self.reload()



    def test_maximum_generation_token_still_has_valid_unique_attempt_identity(self):
        alternate = self.root / "alternate state"
        alternate.mkdir(mode=0o700)
        self.branch("a")
        self.controller = train.Controller(
            self.preparer, train.StateStore(alternate), self.transport, self.authority,
            generation="g" * 128, gates=list(self.workers), completion_adapter=self.transport, runtime=worker.runtime_identity(0),
            now=lambda: self.clock, batch=1)
        a, record = self.admit()
        self.assertLessEqual(len(a), 128)
        self.assertEqual(record["request"]["generation"], "g" * 128)
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])



if __name__ == "__main__":
    unittest.main()
