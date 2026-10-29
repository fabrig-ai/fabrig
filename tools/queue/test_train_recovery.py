#!/usr/bin/env python3
"""Uncertain ownership, publication replay and drain boundaries."""

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


class RecoveryTests(fixture.TrainFixture, unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
