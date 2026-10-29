#!/usr/bin/env python3
"""Corruption, explicit evidence and process publication boundaries; no services."""

import copy
import importlib.util
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("recovery_fixture", DIRECTORY / "test_train.py")
fixtures = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixtures)
train, worker, mq = fixtures.train, fixtures.worker, fixtures.mq
recovery, durable = train.recovery, train.durable


class RecoveryTests(fixtures.TrainTests):
    def assert_refuses_before_mutation(self):
        calls = []
        git = mq.git
        def observed(repo, *args, **kwargs):
            calls.append(args)
            return git(repo, *args, **kwargs)
        with patch.object(mq, "git", side_effect=observed), \
                patch.object(self.authority, "promote") as promote, \
                patch.object(self.authority, "post") as post, \
                self.assertRaises((ValueError, OSError, RuntimeError)):
            self.reload()
        self.assertFalse(any(args[0] in {"fetch", "switch", "reset", "push", "update-ref", "merge"} for args in calls), calls)
        promote.assert_not_called()
        post.assert_not_called()

    def test_unknown_ill_typed_and_inconsistent_state_never_reaches_effects(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        original = self.store.load()
        changes = [
            lambda s: s.update(version=2), lambda s: s.update(version=True),
            lambda s: s.update(active=["missing"]), lambda s: s.update(active=[a, a]),
            lambda s: s.update(gates={r["gate"]: a}), lambda s: s.update(records=[]),
            lambda s: s.update(serial=True), lambda s: s.update(last_full="yesterday"),
            lambda s: s.update(pending=[[{"name": "ready/ops/a", "tip": self.base}]]),
            lambda s: s.update(tried={"ready/ops/forged": self.base}),
            lambda s: s["config"].update(runtime=[]), lambda s: s["config"].update(gates="two"),
            lambda s: s["config"]["runtime"].update(version=True),
            lambda s: s["config"].update(preparation="relative"),
            lambda s: s["records"][a].update(parent="HEAD"),
            lambda s: s["records"][a].update(candidate=self.base),
            lambda s: s["records"][a].update(phase="unknown-phase"),
            lambda s: s["records"][a].update(applied="true"),
            lambda s: s["records"][a].update(outcome="red"),
            lambda s: s["records"][a].update(pins=[]),
            lambda s: s["records"][a]["request"].update(candidate=self.base),
            lambda s: s["records"][a].update(promotion={"parent": self.base}),
            lambda s: s["records"][a].update(phase="running"),
            lambda s: s["records"][a].update(result=[]),
        ]
        for index, change in enumerate(changes):
            with self.subTest(case=index):
                state = copy.deepcopy(original)
                change(state)
                self.store.save(state)
                self.assert_refuses_before_mutation()
        for payload in (b"[]", b"null", b'{"version":1,"version":1}', b'{"version":1,', b'{"version":NaN}'):
            with self.subTest(payload=payload):
                self.store.path.write_bytes(payload)
                self.assert_refuses_before_mutation()
        self.store.save(original)

    def test_corrupt_decision_counters_and_outbox_cannot_replay_effects(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        original = self.store.load()
        event = next(iter(original["outbox"]))
        changes = [lambda s: s.update(batches_since_full=999), lambda s: s.update(tried={}),
                   lambda s: s["records"][a].update(decision="red"),
                   lambda s: s["records"][a].pop("accounting"),
                   lambda s: s["outbox"].pop(event),
                   lambda s: s["outbox"][event].update(delivered=1),
                   lambda s: s["outbox"][event].update(body="no stable identity"),
                   lambda s: s["outbox"][event].update(attempt="elsewhere"),
                   lambda s: s["outbox"][event].update(kind="rejected")]
        for index, change in enumerate(changes):
            with self.subTest(case=index):
                state = copy.deepcopy(original)
                change(state)
                self.store.save(state)
                self.assert_refuses_before_mutation()
        self.store.save(original)

    def test_corrupt_missing_public_and_symlink_artifacts_refuse_before_effects(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        job = self.artifacts / a
        for name in ("request.json", "candidate.bundle", "prepare.log", "returned.json", "returned.log"):
            path = job / name
            saved = path.read_bytes()
            for corruption in ("truncate", "missing", "public", "symlink"):
                with self.subTest(name=name, corruption=corruption):
                    if corruption == "truncate":
                        path.write_bytes(b"truncated")
                    elif corruption == "missing":
                        path.unlink()
                    elif corruption == "public":
                        path.chmod(0o644)
                    else:
                        path.unlink()
                        path.symlink_to(self.store.path)
                    self.assert_refuses_before_mutation()
                    if path.is_symlink():
                        path.unlink()
                    path.write_bytes(saved)
                    path.chmod(0o600)

    def test_pending_bisection_cannot_change_inputs_or_drop_inherited_full(self):
        self.set_batch(2)
        self.branch("a")
        self.branch("b")
        self.controller.state["batches_since_full"] = 4
        a, _ = self.admit()
        self.finish(a, code=1)
        self.controller.adjudicate_head()
        original = self.store.load()
        for field, value in (("pending", []), ("force_full", False)):
            changed = copy.deepcopy(original)
            changed[field] = value
            self.store.save(changed)
            self.assert_refuses_before_mutation()
        self.store.save(original)
        self.controller = self.reload()
        _, record = self.admit()
        self.assertTrue(record["full"])

    def test_missing_snapshot_with_retained_artifacts_and_pins_is_not_a_new_generation(self):
        self.branch("a")
        self.admit()
        self.store.path.unlink()
        self.assert_refuses_before_mutation()
        self.assertFalse(self.store.path.exists())

    def test_missing_snapshot_with_orphan_publication_refuses_initialization(self):
        self.store.path.unlink()
        (self.state_dir / ".publish-interrupted").write_bytes(b"partial")
        self.assert_refuses_before_mutation()

    def test_dangling_snapshot_symlink_is_not_absent_state(self):
        self.store.path.unlink()
        self.store.path.symlink_to(self.root / "missing")
        self.assert_refuses_before_mutation()
        self.assertTrue(self.store.path.is_symlink())

    def test_changed_pin_is_refused_without_reconstructing_candidate(self):
        self.branch("a")
        _, r = self.admit()
        mq.git(self.prep, "update-ref", r["pins"][-1], self.base)
        self.assert_refuses_before_mutation()
        self.assertEqual(mq.git(self.prep, "rev-parse", r["pins"][-1]).stdout.strip(), self.base)

    def test_corruption_after_load_poison_prevents_push_or_new_work(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        (self.artifacts / a / "returned.log").write_bytes(b"corrupt")
        with self.assertRaises(ValueError):
            self.controller.adjudicate_head()
        self.assertEqual(self.authority.pushes, [])
        self.assertEqual(r["outcome"], "green")  # Corruption is never a test red.
        with self.assertRaisesRegex(RuntimeError, "reload/reconcile"):
            self.controller.admit()

    def test_returned_publication_failure_poison_keeps_gate_and_raw_bytes(self):
        self.branch("a")
        a, r = self.admit()
        self.transport.finish(a)
        publish = train.write_bytes
        def fail(path, data):
            if path.name == "returned.json":
                raise OSError("publication failed")
            publish(path, data)
        with patch.object(train, "write_bytes", side_effect=fail), self.assertRaises(OSError):
            self.controller.poll(a)
        self.assertEqual(self.controller.state["gates"][r["gate"]], a)
        with self.assertRaises(RuntimeError):
            self.controller.adjudicate_head()
        self.controller = self.reload()
        inventory = self.controller.recovery_inventory()["artifacts"][a]
        self.assertEqual(inventory["unrecorded_returned"], ["returned.log"])
        self.controller.poll(a)
        self.assertTrue(self.controller.adjudicate_head())

    def test_failed_decision_snapshot_cannot_claim_drained_from_mutated_memory(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        save = self.store.save
        def fail(state):
            if state["records"][a]["applied"]:
                raise OSError("decision publication failed")
            save(state)
        with patch.object(self.store, "save", side_effect=fail), self.assertRaises(OSError):
            self.controller.adjudicate_head()
        self.assertFalse(self.controller.drained())
        self.controller = self.reload()
        self.assertFalse(self.controller.drained())
        self.assertTrue(self.controller.adjudicate_head())
        self.assertTrue(self.controller.drained())

    def test_incomplete_preparation_and_inventory_block_without_reset(self):
        self.branch("a")
        with patch.object(self.preparer, "prepare_candidate", side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
            self.controller.admit()
        self.controller = self.reload()
        a = self.controller.state["active"][0]
        before = self.tip()
        with self.assertRaisesRegex(RuntimeError, "supervisor"):
            self.controller.reconcile_preparation(a)
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.tip(), before)
        self.assertEqual(self.controller.recovery_inventory()["preparing"], [a])

    def test_prepared_snapshot_allows_only_its_matching_completion_marker_to_clear(self):
        self.branch("a")
        a, r = self.admit()
        marker = self.prep / ".git/mq-preparation-unresolved.json"
        worker.atomic_json(marker, {"attempt": "wrong", "generation": "generation-one", "state": str(self.store.path)})
        self.controller = self.reload()
        with self.assertRaises(ValueError):
            self.controller.reconcile_preparation(a)
        worker.atomic_json(marker, {"attempt": a, "generation": "generation-one", "state": str(self.store.path)})
        self.controller.reconcile_preparation(a)
        self.assertFalse(marker.exists())
        self.assertEqual(r["candidate"], self.controller.state["records"][a]["candidate"])

    def test_default_completion_adapter_refuses_boolean_strings_remote_and_heavy(self):
        self.branch("a")
        a, r = self.admit()
        self.transport.finish(a)
        endpoint = self.transport.identity()[r["gate"]]
        result_path = Path(endpoint["artifacts"]) / a / "result.json"
        proof = {"kind": "local-empty-plan-v1", "result": str(result_path),
                 "sha256": worker.digest(result_path.read_bytes())}
        for evidence in (True, "process absent", {"kind": "pid", "pid": 1}, dict(proof, sha256="wrong")):
            with self.subTest(evidence=evidence), self.assertRaises(ValueError):
                recovery.local_completion(r["request"], evidence, endpoint, mq.authority)
        self.assertEqual(recovery.local_completion(r["request"], proof, endpoint, mq.authority), proof)
        with self.assertRaises(ValueError):
            recovery.local_completion(r["request"], proof, dict(endpoint, host="remote"), mq.authority)
        # A complete synchronous fake result carrying a heavy plan still cannot
        # prove a real heavy lifetime through the production local adapter.
        request = copy.deepcopy(r["request"])
        request["plan"] = [{"name": "default", "command": ["heavy", "--gate", "cargo", "nextest", "run"], "cwd": "."}]
        result = train.read_json(result_path.read_bytes())
        result["requested"] = request
        result_path.write_bytes(worker.encoded(result))
        proof["sha256"] = worker.digest(result_path.read_bytes())
        with self.assertRaisesRegex(ValueError, "part-5"):
            recovery.local_completion(request, proof, endpoint, mq.authority)

    def test_no_boolean_adapter_can_release_gate(self):
        self.branch("a")
        a, r = self.admit()
        self.transport.finish(a)
        with patch.object(self.transport, "verify_completion", return_value=True):
            self.controller.poll(a)
        self.assertEqual(self.controller.state["gates"][r["gate"]], a)
        self.assertEqual(r["outcome"], "INFRA")

    def test_runtime_identity_freezes_all_imported_effect_and_recovery_modules(self):
        read_bytes = Path.read_bytes
        for name in ("authority.py", "recovery.py", "durable.py", "train.py", "restart"):
            with self.subTest(module=name):
                def changed(path):
                    return b"changed" if path == DIRECTORY / name else read_bytes(path)
                with patch.object(Path, "read_bytes", changed), self.assertRaises(RuntimeError):
                    self.controller.admit()
        self.assertEqual(self.authority.pushes, [])

    def test_terminal_evidence_conflict_poison_does_not_replace_old_bytes(self):
        self.branch("a")
        a, r = self.admit()
        self.transport.finish(a)
        job = self.artifacts / a
        train.write_bytes(job / "returned.json", b"existing immutable bytes")
        with self.assertRaises(ValueError):
            self.controller.poll(a)
        self.assertEqual((job / "returned.json").read_bytes(), b"existing immutable bytes")
        self.assertEqual(self.controller.state["gates"][r["gate"]], a)
        self.assertEqual(self.controller.state["tried"], {})

    def test_loaded_running_gates_require_identity_status_before_speculation(self):
        self.branch("a")
        self.branch("b")
        a, r = self.admit()
        self.controller = self.reload()
        self.assertEqual(self.controller.startup_inventory["unreconciled_gates"], [a])
        self.assertIsNone(self.controller.admit())
        self.controller.poll(a)
        self.assertEqual(self.controller.recovery_inventory()["unreconciled_gates"], [])
        b, s = self.admit()
        self.assertEqual(s["parent"], r["candidate"])

    def test_loaded_unexpected_main_blocks_preparation_before_discovery(self):
        self.branch("a")
        a, r = self.admit()
        other = self.branch("other")
        mq.git(self.origin, "update-ref", "refs/heads/main", other["tip"])
        self.controller = self.reload()
        self.controller.poll(a)
        with patch.object(self.preparer, "discover") as discover:
            self.assertIsNone(self.controller.admit())
        discover.assert_not_called()

    def test_uncertain_dispatch_inventory_preserves_active_and_draining_owners(self):
        self.branch("a")
        self.branch("b")
        a, r = self.admit()
        b, s = self.admit()
        self.transport.fail_status = True
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.controller = self.reload()
        inv = self.controller.recovery_inventory()
        self.assertEqual(set(inv["gates"].values()), {a, b})
        self.assertEqual(set(inv["dispatch"]), {a, b})
        self.assertFalse(self.controller.drained())
        with self.assertRaises(ValueError):
            self.controller.reconcile_job(b, quiescent=True, evidence={"process": "absent"})


class PublicationTests(unittest.TestCase):
    def setUp(self):
        os.umask(0o077)
        self.tmp = tempfile.TemporaryDirectory(dir=DIRECTORY.parent.parent, prefix=".publication-test-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def test_immutable_publication_never_replaces_different_bytes(self):
        path = self.root / "evidence"
        train.write_bytes(path, b"original")
        train.write_bytes(path, b"original")
        with self.assertRaises(ValueError):
            train.write_bytes(path, b"different")
        self.assertEqual(path.read_bytes(), b"original")
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_process_death_before_and_after_snapshot_rename_reloads_one_complete_snapshot(self):
        code = '''import importlib.util, os, pathlib, sys
spec=importlib.util.spec_from_file_location('d',sys.argv[1]); d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
path=pathlib.Path(sys.argv[2]); real=os.replace
boundary=sys.argv[3]
def replace(source, target):
 if boundary == 'before': os._exit(31)
 real(source,target)
 os._exit(32)
os.replace=replace
d.atomic_json(path, {'decision':'one','outbox':['stable-id'],'counter':1})
'''
        path = self.root / "state.json"
        for boundary, code_value in (("before", 31), ("after", 32)):
            durable.atomic_json(path, {"decision": None, "outbox": [], "counter": 0})
            child = subprocess.run([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "durable.py"),
                                    str(path), boundary], stdin=subprocess.DEVNULL, capture_output=True)
            self.assertEqual(child.returncode, code_value, child.stderr)
            state = durable.read_json(durable.read_bytes(path))
            self.assertEqual(state, {"decision": None, "outbox": [], "counter": 0} if boundary == "before" else
                             {"decision": "one", "outbox": ["stable-id"], "counter": 1})
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_immutable_rename_publication_crash_exposes_only_complete_or_absent_evidence(self):
        code = '''import importlib.util, os, pathlib, sys
spec=importlib.util.spec_from_file_location('d',sys.argv[1]);d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
real=os.replace
def replace(source,target,**kwargs):
 if sys.argv[3]=='before': os._exit(41)
 real(source,target,**kwargs);os._exit(42)
os.replace=replace
d.publish(pathlib.Path(sys.argv[2]), b'complete evidence', immutable=True)
'''
        for boundary, value in (("before", 41), ("after", 42)):
            path = self.root / boundary
            child = subprocess.run([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "durable.py"),
                                    str(path), boundary], stdin=subprocess.DEVNULL, capture_output=True)
            self.assertEqual(child.returncode, value, child.stderr)
            self.assertEqual(path.read_bytes() if path.exists() else None, b"complete evidence" if boundary == "after" else None)

    def test_file_and_directory_fsync_failures_never_permit_partial_snapshot(self):
        path = self.root / "snapshot.json"
        durable.atomic_json(path, {"old": True})
        for boundary in (1, 2):
            fsync, count = os.fsync, [0]
            def fail(fd):
                count[0] += 1
                if count[0] == boundary:
                    raise OSError("injected fsync")
                return fsync(fd)
            with patch.object(os, "fsync", side_effect=fail), self.assertRaises(OSError):
                durable.atomic_json(path, {"new": True})
            self.assertIn(durable.read_json(durable.read_bytes(path)), ({"old": True}, {"new": True}))


def suite():
    return unittest.TestSuite(cls(name) for cls in (RecoveryTests, PublicationTests)
                              for name in cls.__dict__ if name.startswith("test_"))


if __name__ == "__main__":
    unittest.main(defaultTest="suite")
