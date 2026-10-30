"""Confirmed publication retires exact Ready/feature tips; design.md, 3.11."""

import importlib.util
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("lifecycle_fixtures", DIRECTORY / "test_publication.py")
fixtures = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixtures)
mq = fixtures.mq


class LifecycleTests(unittest.TestCase):
    def setUp(self):
        # Compose the fixture so discovery does not inherit its publication cases.
        self.p = fixtures.PublicationTests()
        self.p.setUp()
        self.addCleanup(self.p.doCleanups)
        self.f = self.p.f
        self.repo, self.origin = self.p.repo, self.p.origin

    def branch(self, task="docs"):
        branch = self.f.branch(task, publish=True)
        mq.git(self.repo, "push", "origin", branch["tip"] + ":refs/heads/" + branch["name"][6:])
        return branch

    def heads(self):
        return dict(line.split()[::-1] for line in mq.git(self.origin, "show-ref", "--heads").stdout.splitlines())

    def refs(self, branch):
        return ["refs/heads/" + branch["name"], "refs/heads/" + branch["name"][6:]]

    def confirmed(self, branch):
        intent = self.p.prepare([branch])
        self.p.store.reconcile()
        return intent

    def finish(self, intent):
        # Startup reconciles confirmation and finishes effects under both locks.
        queue = self.f.queue()
        queue.finish_publication(intent)
        return queue

    def test_both_exact_heads_deleted_after_confirmation_once(self):
        branch = self.branch()
        intent = self.confirmed(branch)
        self.assertTrue(all(ref in self.heads() for ref in self.refs(branch)))
        self.finish(intent)
        self.assertTrue(all(ref not in self.heads() for ref in self.refs(branch)))
        self.assertEqual(self.heads()["refs/heads/main"], intent["candidate"])
        self.finish(intent)
        self.assertEqual(len(self.f.posts), 1)
        self.assertEqual([row for row in self.p.rows() if "event" not in row], [intent["journal_row"]])

    def test_no_cleanup_before_publication_and_durable_journal_confirmation(self):
        queue = self.f.queue()
        branch = self.branch()
        intent = self.p.prepare([branch])
        with patch.object(mq, "cleanup_ready") as cleanup:
            with self.assertRaisesRegex(RuntimeError, "journal confirmation"):
                queue.finish_publication(intent)
            # Publishing C alone does not authorize effects.
            mq.git(self.repo, "push", "origin", intent["candidate"] + ":refs/heads/main")
            with self.assertRaisesRegex(RuntimeError, "journal confirmation"):
                queue.finish_publication(intent)
            self.p.store.journal(intent, append=True)
            with self.assertRaisesRegex(RuntimeError, "journal confirmation"):
                queue.finish_publication(intent)
            cleanup.assert_not_called()
        self.assertTrue(all(ref in self.heads() for ref in self.refs(branch)))
        self.p.store.reconcile()
        queue.finish_publication(intent)
        self.assertTrue(all(ref not in self.heads() for ref in self.refs(branch)))

    def test_failed_and_dry_work_preserve_both_heads(self):
        for task, dry in (("red", False), ("docs", True)):
            with self.subTest(task=task):
                branch = self.branch(task)
                queue = self.f.queue(dry=dry)
                with patch.object(mq, "cleanup_ready") as cleanup:
                    self.assertEqual(queue.attempt([branch]), dry)
                    cleanup.assert_not_called()
                self.assertTrue(all(self.heads().get(ref) == branch["tip"] for ref in self.refs(branch)))
        self.assertEqual(self.p.remote(), self.p.base)

    def advance(self, ref):
        self.f.write("advance", "later work\n")
        self.f.commit("Later work")
        advanced = self.f.tip()
        mq.git(self.repo, "push", "origin", advanced + ":" + ref)
        return advanced

    def assert_independent(self, changed):
        branch = self.branch()
        intent = self.confirmed(branch)
        refs = self.refs(branch)
        advanced = self.advance(refs[changed])
        self.finish(intent)
        self.assertEqual(self.heads()[refs[changed]], advanced)
        self.assertNotIn(refs[1 - changed], self.heads())
        self.assertEqual(self.heads()["refs/heads/main"], intent["candidate"])

    def test_changed_feature_preserved_ready_deleted(self):
        self.assert_independent(1)

    def test_changed_ready_preserved_feature_deleted(self):
        self.assert_independent(0)

    def test_absent_feature_is_idempotent_and_ready_deleted(self):
        branch = self.branch()
        refs = self.refs(branch)
        mq.git(self.repo, "push", "origin", ":" + refs[1])
        intent = self.confirmed(branch)
        self.finish(intent)
        self.assertTrue(all(ref not in self.heads() for ref in refs))
        self.assertTrue(all("absent" in row["outcome"] for row in
                            mq.cleanup_ready(self.repo, [branch], str(self.origin))))

    def test_lost_main_ack_reconciles_and_cleans_once_without_republication(self):
        branch = self.branch()
        queue = self.f.queue()
        git, calls = mq.git, []
        def lost_ack(repo, *args, **kwargs):
            result = git(repo, *args, **kwargs)
            if args[0] in ("push", "ls-remote"):
                calls.append(args)
            if args[0] == "push" and args[-1].endswith(":refs/heads/main"):
                self.assertEqual(result.returncode, 0)
                return subprocess.CompletedProcess(args, 255, "lost acknowledgement")
            return result
        with patch.object(mq, "git", side_effect=lost_ack):
            self.assertTrue(queue.attempt([branch]))
            queue.reconcile_publications()
            self.f.queue()
        self.assertEqual(len([call for call in calls if call[0] == "push"]), 2)
        self.assertEqual(len([call for call in calls if call[0] == "ls-remote"]), 1)
        self.assertEqual(len([row for row in self.p.rows() if "event" not in row]), 1)
        self.assertEqual(len(self.f.posts), 1)
        self.assertTrue(all(ref not in self.heads() for ref in self.refs(branch)))

    def test_concurrent_feature_change_atomically_preserves_all_eligible_heads(self):
        branch = self.branch()
        intent = self.confirmed(branch)
        ready, feature = self.refs(branch)
        self.f.write("advance", "concurrent feature\n")
        self.f.commit("Concurrent feature")
        advanced = self.f.tip()
        git = mq.git
        def race(repo, *args, **kwargs):
            if args[0] == "push" and "--atomic" in args:
                git(self.repo, "push", "origin", advanced + ":" + feature)
            return git(repo, *args, **kwargs)
        with patch.object(mq, "git", side_effect=race):
            self.finish(intent)
        self.assertEqual(self.heads()[ready], branch["tip"])
        self.assertEqual(self.heads()[feature], advanced)
        self.assertEqual(self.p.remote(), intent["candidate"])
        self.assertTrue(all(row["returncode"] != 0 for row in self.p.rows()[-1]["cleanup"]))
        self.assertEqual(len(self.f.posts), 1)

    def test_atomic_remote_refusal_preserves_all_heads_and_publication(self):
        branch = self.branch()
        intent = self.confirmed(branch)
        hook = self.origin / "hooks/update"
        hook.write_text('#!/bin/sh\n[ "$1" != "refs/heads/coord/docs" ]\n')
        hook.chmod(0o700)
        self.finish(intent)
        self.assertTrue(all(self.heads().get(ref) == branch["tip"] for ref in self.refs(branch)))
        self.assertTrue(all(row["returncode"] != 0 for row in self.p.rows()[-1]["cleanup"]))
        self.assertEqual(self.p.rows()[0], intent["journal_row"])
        self.assertEqual(len(self.f.posts), 1)

    def assert_cleanup_failure(self, stage, exception=False):
        branch = self.branch()
        queue = self.f.queue()
        git, failures, pushes = mq.git, [], []
        retire, in_cleanup = mq.cleanup_ready, []
        def cleanup(*args, **kwargs):
            in_cleanup.append(True)
            try:
                return retire(*args, **kwargs)
            finally:
                in_cleanup.pop()
        def fail(repo, *args, **kwargs):
            if args[0] == "push":
                pushes.append(args)
            if (stage == "observation" and args[0] == "ls-remote" or
                    stage == "deletion" and args[0] == "push" and "--atomic" in args or
                    stage == "validation" and in_cleanup and args[0] == "check-ref-format"):
                failures.append(args)
                if exception:
                    raise OSError("Git client unavailable")
                return subprocess.CompletedProcess(args, 255, "authentication refused")
            return git(repo, *args, **kwargs)
        with patch.object(mq, "git", side_effect=fail), patch.object(mq, "cleanup_ready", side_effect=cleanup):
            self.assertTrue(queue.attempt([branch]))
            queue.reconcile_publications()
            self.f.queue()
        self.assertEqual(len(failures), 1)
        self.assertEqual(len(pushes), 1 + (stage == "deletion"))
        rows = self.p.rows()
        self.assertEqual(len([row for row in rows if "event" not in row]), 1)
        self.assertEqual(rows[0]["outcome"], "green")
        self.assertEqual(rows[-1]["outcome"], "green")
        self.assertTrue(all(row.get("error") or row.get("returncode") == 255 for row in rows[-1]["cleanup"]))
        self.assertEqual(queue.state["tried"][branch["name"]], branch["tip"])
        self.assertEqual(len(self.f.posts), 1)
        self.assertTrue(all(self.heads().get(ref) == branch["tip"] for ref in self.refs(branch)))

    def test_observation_auth_failure_no_red_no_repeat_publication_or_notice(self):
        self.assert_cleanup_failure("observation")

    def test_deletion_auth_failure_no_red_no_repeat_publication_or_notice(self):
        self.assert_cleanup_failure("deletion")

    def test_ssh_launch_failure_is_operational_and_not_retried(self):
        self.assert_cleanup_failure("deletion", exception=True)

    def test_validation_launch_failure_is_operational_and_not_retried(self):
        self.assert_cleanup_failure("validation", exception=True)

    def assert_invalid_default(self, kind):
        branch = self.branch()
        intent = self.confirmed(branch)
        ready, feature = self.refs(branch)
        targets = f"{branch['tip']} {ready}\n{branch['tip']} {feature}\n"
        symref = f"ref: {feature} HEAD\n"
        head = f"{branch['tip']} HEAD\n"
        observations = {
            "missing": targets,
            "malformed": "ref: invalid HEAD\n" + head + targets,
            "inconsistent": symref + f"{self.p.base} HEAD\n" + targets,
            "missing_oid": symref + targets,
            "detached": head + targets,
            "unobserved_default": "ref: refs/heads/other HEAD\n" + head + targets,
        }
        git, calls = mq.git, []
        def observe(repo, *args, **kwargs):
            if args[0] in ("ls-remote", "push"):
                calls.append(args)
            if args[0] == "ls-remote":
                return subprocess.CompletedProcess(args, 0, observations[kind])
            return git(repo, *args, **kwargs)
        with patch.object(mq, "git", side_effect=observe):
            self.finish(intent)
            self.finish(intent)
        self.assertEqual(self.heads().get(ready), branch["tip"])
        self.assertEqual(self.heads().get(feature), branch["tip"])
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][0], "ls-remote")
        self.assertEqual(len(self.f.posts), 1)
        rows = self.p.rows()
        self.assertEqual([row for row in rows if "event" not in row], [intent["journal_row"]])
        self.assertTrue(all("default" in row["error"] for row in rows[-1]["cleanup"]))

    def test_missing_head_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("missing")

    def test_malformed_default_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("malformed")

    def test_inconsistent_head_oid_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("inconsistent")

    def test_missing_head_oid_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("missing_oid")

    def test_detached_head_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("detached")

    def test_unobserved_default_oid_preserves_all_refs_without_replay(self):
        self.assert_invalid_default("unobserved_default")

    def test_batch_has_one_observation_and_one_atomic_exact_lease_push(self):
        branches = [self.branch(f"docs-{index}") for index in range(16)]
        intent = self.p.prepare(branches)
        self.p.store.reconcile()
        with patch.object(mq, "git", wraps=mq.git) as git:
            self.finish(intent)
        observations = [call.args[1:] for call in git.call_args_list if call.args[1] == "ls-remote"]
        pushes = [call.args[1:] for call in git.call_args_list if call.args[1] == "push"]
        refs = [ref for branch in branches for ref in self.refs(branch)]
        self.assertEqual(observations, [("ls-remote", "--symref", str(self.origin), "HEAD", "refs/heads/main", *refs)])
        self.assertEqual(pushes, [("push", "--atomic",
            *("--force-with-lease=" + ref + ":" + branch["tip"] for branch in branches for ref in self.refs(branch)),
            str(self.origin), *(":" + ref for ref in refs))])
        self.assertEqual(self.heads(), {"refs/heads/main": intent["candidate"]})

    def test_advertised_default_is_preserved_even_at_published_tip(self):
        branch = self.branch()
        intent = self.confirmed(branch)
        ready, feature = self.refs(branch)
        mq.git(self.origin, "symbolic-ref", "HEAD", feature)
        self.finish(intent)
        self.assertNotIn(ready, self.heads())
        self.assertEqual(self.heads()[feature], branch["tip"])

    def test_unknown_observation_preserves_every_head(self):
        branch = self.branch()
        intent = self.confirmed(branch)
        git = mq.git
        def unknown(repo, *args, **kwargs):
            if args[0] == "ls-remote":
                return subprocess.CompletedProcess(args, 0, branch["tip"] + " refs/heads/unknown\n")
            return git(repo, *args, **kwargs)
        with patch.object(mq, "git", side_effect=unknown):
            self.finish(intent)
        self.assertTrue(all(self.heads().get(ref) == branch["tip"] for ref in self.refs(branch)))
        self.assertTrue(all("unknown" in row["error"] for row in self.p.rows()[-1]["cleanup"]))

    def test_main_default_and_unknown_inputs_never_become_cleanup_targets(self):
        branches = [{"name": name, "tip": self.p.base} for name in
                    ("main", "master", "coord/docs", "ready/main", "ready/coord/../main")]
        branches.append({"name": "ready/coord/docs", "tip": "unknown"})
        with patch.object(mq, "git", wraps=mq.git) as git:
            outcomes = mq.cleanup_ready(self.repo, branches, str(self.origin))
        self.assertTrue(all("preserved" in row["outcome"] for row in outcomes))
        self.assertFalse(any(call.args[1] in ("ls-remote", "push") for call in git.call_args_list))
        self.assertEqual(self.heads(), {"refs/heads/main": self.p.base})


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
