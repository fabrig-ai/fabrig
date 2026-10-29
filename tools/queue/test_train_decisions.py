#!/usr/bin/env python3
"""Ordered promotion, conflicts and sequential bisection."""

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


class DecisionTests(fixture.TrainFixture, unittest.TestCase):
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



    def test_partial_conflict_checks_original_candidate_before_green_decision(self):
        self.set_batch(2)
        merged = self.branch("a", {"README.md": "left", "core/src/lib.rs": "source"})
        conflict = self.branch("b", {"README.md": "right"})
        self.branch("c")
        a, record = self.admit()
        candidate = record["candidate"]
        self.assertIsNone(record["outcome"])
        self.assertEqual(record["phase"], "running")
        self.assertEqual(record["parent"], self.base)
        self.assertEqual(self.transport.jobs[a]["request"]["branches"], [merged])
        self.assertEqual(record["conflicts"][0]["name"], conflict["name"])
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertEqual(self.authority.posts, [])
        self.assertFalse(self.controller.adjudicate_head())
        self.assertIsNone(self.controller.admit())
        self.finish(a)
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), candidate)
        self.assertEqual(self.authority.pushes, [(self.base, candidate, a + ":promotion")])
        self.assertEqual(self.controller.state["tried"],
                         {merged["name"]: merged["tip"], conflict["name"]: conflict["tip"]})
        self.assertEqual(self.controller.state["pending"], [])
        self.assertFalse(self.controller.state["bisecting"])
        self.assertEqual(self.transport.starts, [a])
        self.controller = self.reload()
        self.controller.flush_outbox()
        rejected = [event for event in self.authority.posts if event["kind"] == "rejected"]
        self.assertEqual([event["branch"]["name"] for event in rejected], [conflict["name"]])
        self.controller = self.reload()
        self.controller.flush_outbox()
        self.assertEqual(len(self.authority.posts), 2)
        self.assertEqual(self.controller.state["batches_since_full"], 1)



    def test_partial_conflict_red_bisects_only_checked_inputs_and_preserves_full(self):
        self.controller.state["batches_since_full"] = 4
        a, record, merged, conflict = self.partial_conflict_attempt()
        self.assertEqual(record["request"]["selection"], "full")
        self.finish(a, code=1)
        self.assertEqual(record["outcome"], "red")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertIsNone(self.controller.admit())
        self.controller.adjudicate_head()
        self.assertEqual(self.controller.state["tried"], {conflict["name"]: conflict["tip"]})
        self.assertEqual(self.controller.state["pending"], [[merged[0]], [merged[1]]])
        self.assertEqual(self.authority.pushes, [])
        self.controller = self.reload()
        left, left_record = self.admit()
        self.assertEqual(left_record["request"]["selection"], "full")
        self.assertEqual(left_record["request"]["branches"], [merged[0]])
        self.assertEqual(left_record["parent"], self.base)
        self.finish(left)
        self.controller.adjudicate_head()
        right, right_record = self.admit()
        self.assertEqual(right_record["parent"], left_record["candidate"])
        self.assertEqual(right_record["request"]["selection"], "full")
        self.assertEqual(right_record["request"]["branches"], [merged[1]])
        self.finish(right, code=1)
        self.controller.adjudicate_head()
        rejected = [event["branch"]["name"] for event in self.controller.state["outbox"].values()
                    if event["kind"] == "rejected"]
        self.assertEqual(rejected, [conflict["name"], merged[1]["name"]])
        self.assertEqual(self.authority.main(), left_record["candidate"])
        self.assertEqual(self.controller.state["pending"], [])



    def test_partial_conflict_cannot_be_applied_as_an_unchecked_conflict_decision(self):
        a, record, _, _ = self.partial_conflict_attempt()
        record["outcome"] = "conflict"
        with self.assertRaisesRegex(ValueError, "require a checked outcome"):
            self.controller.apply_decision_once(record, "conflict")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertEqual(self.controller.state["gates"][record["gate"]], a)



    def test_all_conflict_inputs_are_decided_without_dispatch_or_check_accounting(self):
        self.set_batch(2)
        conflicts = [self.branch("a", {"README.md": "left"}),
                     self.branch("b", {"README.md": "right"})]
        mq.git(self.prep, "switch", "--detach", self.base)
        self.write("README.md", "main changed")
        self.commit("advance fixture main")
        parent = self.tip()
        mq.git(self.prep, "push", "origin", parent + ":refs/heads/main")
        a, record = self.admit()
        self.assertEqual(record["outcome"], "conflict")
        self.assertEqual(record["chain"], [])
        self.assertEqual(record["parent"], parent)
        self.assertEqual(record["candidate"], parent)
        self.assertEqual(self.transport.starts, [])
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["tried"], {b["name"]: b["tip"] for b in conflicts})
        self.assertEqual(self.controller.state["pending"], [])
        self.assertEqual(self.controller.state["batches_since_full"], 0)
        self.assertEqual(self.authority.pushes, [])
        self.assertTrue(self.controller.drained())
        self.controller.flush_outbox()
        self.assertEqual([event["branch"]["name"] for event in self.authority.posts],
                         [b["name"] for b in conflicts])



    def test_checked_partial_conflict_successor_is_discarded_when_ancestor_is_infra(self):
        self.set_batch(2)
        self.branch("a", {"README.md": "left"})
        self.branch("b", {"core/src/b.rs": "first source"})
        self.branch("c", {"README.md": "right"})
        unrelated = self.branch("d", {"core/src/lib.rs": "unrelated source"})
        a, head = self.admit()
        b, successor = self.admit()
        self.assertEqual(successor["request"]["branches"], [unrelated])
        self.assertEqual(successor["parent"], head["candidate"])
        self.finish(b)
        self.assertEqual(successor["outcome"], "green")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["outbox"], {})
        self.transport.fail_status = True
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.assertTrue(successor["invalidated"])
        self.assertEqual(successor["phase"], "discarded")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.authority.pushes, [])
        self.assertFalse(self.controller.adjudicate_head())



    def test_stop_partial_conflict_green_drains_original_attempt_without_replacement(self):
        a, record, merged, conflict = self.partial_conflict_attempt()
        self.controller.stop()
        self.finish(a)
        self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])
        self.assertEqual(self.controller.state["tried"],
                         {b["name"]: b["tip"] for b in [*merged, conflict]})
        self.assertTrue(self.controller.drained())
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.transport.starts, [a])



    def test_stop_partial_conflict_red_retains_only_merged_bisection_groups(self):
        a, _, merged, conflict = self.partial_conflict_attempt()
        self.controller.stop()
        self.finish(a, code=1)
        self.controller.adjudicate_head()
        self.controller = self.reload()
        self.assertEqual(self.controller.state["tried"], {conflict["name"]: conflict["tip"]})
        self.assertEqual(self.controller.state["pending"], [[merged[0]], [merged[1]]])
        self.assertTrue(self.controller.drained())
        self.assertIsNone(self.controller.admit())
        self.assertEqual(self.transport.starts, [a])



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



    def test_red_against_unexpected_main_never_rejects(self):
        self.branch("a", {"core/src/lib.rs": "bad"})
        a, record = self.admit()
        self.finish(a, code=1)
        other = self.branch("other")
        mq.git(self.origin, "update-ref", "refs/heads/main", other["tip"])
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.controller.state["pending"], [])



    def test_partial_conflict_heavy_capacity_refusal_rejects_no_branch(self):
        a, record, _, _ = self.partial_conflict_attempt()
        def capacity_runner(argv, **kwargs):
            refused = argv[0] == "heavy"
            kwargs["stdout"].write("fixture below the disk floor\n" if refused else "fixture passed\n")
            return subprocess.CompletedProcess(argv, 2 if refused else 0)
        def checker(*args, **kwargs):
            return mq.run_checks(*args, runner=capacity_runner, **kwargs)
        result = self.finish(a, checker=checker)
        self.assertEqual(result["outcome"], "INFRA")
        self.assertIn("below the disk floor", result["error"])
        self.assertTrue(result["completion_known"])
        self.assert_partial_conflict_infra(a, record)



    def test_partial_conflict_worker_exception_preserves_unresolved_ownership(self):
        a, record, _, _ = self.partial_conflict_attempt()
        def checker(*args, **kwargs):
            raise RuntimeError("fixture checker interrupted")
        result = self.finish(a, checker=checker)
        self.assertFalse(result["completion_known"])
        self.assertTrue((self.workers[record["gate"]] / ".git/mq-worker-unresolved.json").exists())
        self.assert_partial_conflict_infra(a, record)
        self.controller.stop()
        self.controller = self.reload()
        self.assertFalse(self.controller.drained())
        self.assertIsNone(self.controller.admit())



    def test_partial_conflict_corrupt_result_rejects_no_branch(self):
        a, record, _, _ = self.partial_conflict_attempt()
        result = self.transport.finish(a)
        self.assertEqual(result["outcome"], "green")
        result["observed"]["candidate_tree"] = "0" * 40
        self.transport.jobs[a]["result"] = worker.encoded(result)
        self.controller.poll(a)
        self.assertIn("observed identities mismatch", record["error"])
        self.assert_partial_conflict_infra(a, record)



    def test_partial_conflict_status_loss_and_late_green_cannot_reject_or_land(self):
        a, record, _, _ = self.partial_conflict_attempt()
        self.transport.fail_status = True
        self.controller.poll(a)
        self.assert_partial_conflict_infra(a, record)
        self.transport.fail_status = False
        self.finish(a)
        self.assertEqual(record["phase"], "discarded")
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.authority.pushes, [])
        self.assertTrue(self.controller.drained())
        self.assertFalse(self.controller.adjudicate_head())



    def test_partial_conflict_lost_start_acknowledgement_rejects_no_branch(self):
        merged, _ = self.partial_conflict_inputs()
        self.transport.fail_start = "after"
        a = self.controller.admit()
        record = self.controller.state["records"][a]
        self.assertEqual(record["request"]["branches"], merged)
        self.assertEqual(self.transport.starts, [a])
        self.assertEqual(record["phase"], "unknown")
        self.assert_partial_conflict_infra(a, record)
        self.transport.fail_start = None
        self.finish(a)
        self.assertEqual(self.controller.state["tried"], {})
        self.assertEqual(self.authority.pushes, [])



    def test_partial_conflict_combined_decision_and_outbox_replay_after_publication_failures(self):
        a, record, merged, conflict = self.partial_conflict_attempt()
        self.controller = self.reload()
        self.assertIsNone(self.controller.admit())
        self.finish(a)
        original = self.store.save
        def decision_save(state):
            if state["records"][a]["applied"]:
                raise OSError("combined decision publication failed")
            original(state)
        with patch.object(self.store, "save", side_effect=decision_save):
            with self.assertRaises(OSError):
                self.controller.adjudicate_head()
        self.assertEqual(self.authority.main(), record["candidate"])
        durable = self.store.load()
        self.assertEqual(durable["tried"], {})
        self.assertEqual(durable["outbox"], {})
        self.controller = self.reload()
        self.controller.adjudicate_head()
        expected = {b["name"]: b["tip"] for b in [*merged, conflict]}
        self.assertEqual(self.controller.state["tried"], expected)
        self.assertEqual(self.controller.state["batches_since_full"], 1)
        self.assertEqual(len(self.authority.pushes), 1)
        self.assertEqual(len(self.controller.state["outbox"]), 5)
        def post_save(state):
            if any(event["delivered"] for event in state["outbox"].values()):
                raise OSError("post acknowledgement publication failed")
            original(state)
        with patch.object(self.store, "save", side_effect=post_save):
            with self.assertRaises(OSError):
                self.controller.flush_outbox()
        self.controller = self.reload()
        self.controller.flush_outbox()
        self.assertEqual(self.authority.posts[0]["id"], self.authority.posts[1]["id"])
        rejected = [event for event in self.authority.posts if event["kind"] == "rejected"]
        self.assertEqual([event["branch"]["name"] for event in rejected], [conflict["name"]])
        self.assertEqual(self.controller.state["tried"], expected)
        self.assertEqual(self.controller.state["batches_since_full"], 1)
        self.assertFalse(self.controller.adjudicate_head())



if __name__ == "__main__":
    unittest.main()
