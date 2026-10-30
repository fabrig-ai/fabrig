"""Publication faults against private local origins; tools/queue/design.md, 3.11."""

from contextlib import ExitStack
import copy
import hashlib
import importlib.util
import json
import os
import select
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("publication_fixtures", DIRECTORY / "test_mq.py")
fixtures = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixtures)
mq = fixtures.mq
pub = mq.publication
ROSTER = b"coord s/owner@fixture.example\nbeta -\n"
CREATED = "2026-10-30T23:59:59Z"


class PublicationTests(unittest.TestCase):
    def setUp(self):
        self.f = fixtures.RepositoryTests()
        self.f.setUp()
        self.addCleanup(self.f.doCleanups)
        self.repo, self.origin, self.base = self.f.repo, self.f.origin, self.f.base
        self.journal = self.f.directory / "queue.jsonl"
        stack = self.enterContext(ExitStack())
        self.layout = mq.authority.Layout(self.f.directory / "authority")
        self.lease = stack.enter_context(self.layout.acquire())
        self.checkout = stack.enter_context(mq.authority.checkout_lock(self.repo))
        stack.enter_context(mq.writer_authority(self.lease, self.checkout))
        self.store = pub.Store(self.layout.root / "publications", self.repo, mq.git,
                               lambda: (self.lease.guard(), self.checkout.guard()))

    def candidate_inputs(self, branches=None, *, baseline=False):
        branches = branches or [self.f.branch("docs", publish=True)]
        merged, conflicts = mq.build_batch(self.repo, branches)
        if baseline:
            self.f.write("baseline", "lower\n")
            self.f.commit("Lower baseline")
        row = {"start": "2026-10-30T23:58:00Z", "branches": branches, "selection": "docs",
               "steps": [], "posts": [], "conflicts": conflicts, "dry": False,
               "parent": self.base, "commit": self.f.tip(), "journal_path": str(self.journal)}
        return row, merged

    def prepare(self, branches=None, *, baseline=False):
        row, merged = self.candidate_inputs(branches, baseline=baseline)
        return self.store.prepare(str(self.origin), row, merged, ROSTER, CREATED)

    def rows(self):
        return [json.loads(line) for line in self.journal.read_bytes().splitlines()] if self.journal.exists() else []

    def remote(self):
        return mq.git(self.origin, "rev-parse", "refs/heads/main").stdout.strip()

    def test_death_before_intent_commit_cannot_push(self):
        row, merged = self.candidate_inputs()
        payload = self.f.directory / "pre-intent.json"
        payload.write_bytes(pub.durable.encoded({"row": row, "merges": merged, "origin": str(self.origin),
                                               "roster": ROSTER.decode(), "created": CREATED}))
        script = self.child_loader() + """
payload=m.publication.durable.read_json(pathlib.Path(sys.argv[4]).read_bytes())
publish=m.publication.durable.publish
def crash(path,data,**kwargs):
 if pathlib.Path(path).name=="intent.json": os._exit(73)
 return publish(path,data,**kwargs)
m.publication.durable.publish=crash
store.prepare(payload["origin"],payload["row"],payload["merges"],payload["roster"].encode(),payload["created"])
"""
        result = self.child(script, str(payload))
        self.assertEqual(result.returncode, 73, result.stderr)
        self.assertEqual(self.store.reconcile(), [])
        self.assertEqual(self.remote(), self.base)
        self.assertEqual(self.rows(), [])

    def test_after_intent_before_push_reuses_saved_candidate(self):
        intent = self.prepare()
        self.f.write("unrelated", "later HEAD\n")
        self.f.commit("unrelated")
        self.store.reconcile()
        self.assertEqual(self.remote(), intent["candidate"])
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def child_loader(self):
        return """import importlib.machinery, importlib.util, os, pathlib, sys
p=pathlib.Path(sys.argv[1]); loader=importlib.machinery.SourceFileLoader("m",str(p/"mq"))
s=importlib.util.spec_from_loader("m",loader); m=importlib.util.module_from_spec(s); loader.exec_module(m)
store=m.publication.Store(pathlib.Path(sys.argv[3])/"publications",pathlib.Path(sys.argv[2]),m.git,lambda:None)
"""

    def child(self, script, *args):
        return subprocess.run([sys.executable, "-I", "-B", "-c", script, str(DIRECTORY),
                               str(self.repo), str(self.layout.root), *args],
                              stdin=subprocess.DEVNULL, capture_output=True, text=True,
                              pass_fds=mq.writer_fds(), check=False)

    def test_publication_lost_ack_roster_changed(self):
        intent = self.prepare()
        # A separate queue process pushes the real C and dies at the journal boundary.
        script = self.child_loader() + """
intent=store.entries()[0][1]
store.journal=lambda *a,**kw: os._exit(73)
store.reconcile_one(intent)
"""
        result = self.child(script)
        self.assertEqual(result.returncode, 73, result.stderr)
        self.assertEqual(self.remote(), intent["candidate"])
        changed = self.f.directory / "replacement"
        changed.mkdir()
        (changed / "coordinators.txt").write_bytes(b"coord s/replacement@fixture.example\n")
        (changed / "leads.txt").write_bytes((DIRECTORY / "leads.txt").read_bytes())
        with patch.object(mq, "CONFIG", changed):
            queue = self.f.queue()
        self.assertEqual(self.rows()[0], intent["journal_row"])
        self.assertEqual(self.rows()[0]["merges"][0]["coordinator"], "s/owner@fixture.example")
        self.assertEqual(queue.coordinators_bytes, (changed / "coordinators.txt").read_bytes())
        self.assertEqual((self.store.root / intent["publication_id"] / "coordinators.txt").read_bytes(), ROSTER)

    def test_reconcile_main_advanced_past_candidate(self):
        intent = self.prepare()
        self.f.write("later", "main advanced\n")
        self.f.commit("main advanced")
        advanced = self.f.tip()
        mq.git(self.repo, "push", "origin", "HEAD:main")
        self.store.reconcile()
        self.assertEqual(self.remote(), advanced)
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def test_surviving_push_child_retains_both_locks(self):
        intent = self.prepare()
        # A Git pre-push child waits on a pipe, retaining inherited authority descriptors.
        read_fd, write_fd = os.pipe()
        self.addCleanup(os.close, write_fd)
        hook = self.repo / ".git/hooks/pre-push"
        ready_read, ready_write = os.pipe()
        hook.write_text(f"#!{sys.executable}\nimport os\nos.write({ready_write}, b'ready')\nos.read({read_fd}, 1)\n")
        hook.chmod(0o700)
        process = subprocess.Popen(["git", "push", str(self.origin), intent["candidate"] + ":refs/heads/main"],
            cwd=self.repo, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            pass_fds=(*mq.writer_fds(), read_fd, ready_write))
        def reap():
            if process.poll() is None:
                process.kill()
            process.communicate()
        self.addCleanup(reap)
        os.close(read_fd)
        os.close(ready_write)
        self.addCleanup(os.close, ready_read)
        self.assertTrue(select.select([ready_read], [], [], 5)[0])
        self.assertEqual(os.read(ready_read, 5), b"ready")
        # Releasing the parent descriptors must not release the inherited open descriptions.
        self.lease.close()
        self.checkout.close()
        probe = '''import importlib.util,pathlib,sys
p=pathlib.Path(sys.argv[1]); s=importlib.util.spec_from_file_location("a",p/"authority.py")
a=importlib.util.module_from_spec(s); s.loader.exec_module(a)
failures=0
for path in (pathlib.Path(sys.argv[3])/"authority.lock",pathlib.Path(sys.argv[2]+".queue.lock")):
 try:
  with a.Lock(path): pass
 except RuntimeError: failures+=1
sys.exit(0 if failures==2 else 1)
'''
        result = subprocess.run([sys.executable, "-I", "-B", "-c", probe, str(DIRECTORY),
                                 str(self.repo), str(self.layout.root)], stdin=subprocess.DEVNULL,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        os.write(write_fd, b"x")
        output, errors = process.communicate(timeout=5)
        self.assertEqual(process.returncode, 0, errors)

    def test_unavailable_remote_retains_intent(self):
        intent = self.prepare()
        self.origin.rename(self.origin.with_suffix(".unavailable"))
        with self.assertRaises(RuntimeError):
            self.store.reconcile()
        self.assertTrue((self.store.root / intent["publication_id"] / "intent.json").exists())
        self.assertEqual(self.rows(), [])

    def test_divergent_remote_retains_intent_without_push(self):
        intent = self.prepare()
        mq.git(self.repo, "switch", "--detach", self.base)
        self.f.write("other", "other\n")
        self.f.commit("other main")
        mq.git(self.repo, "push", "origin", "HEAD:main")
        divergent = self.remote()
        with self.assertRaisesRegex(RuntimeError, "diverged"):
            self.store.reconcile()
        self.assertEqual(self.remote(), divergent)
        self.assertEqual(self.rows(), [])
        self.assertFalse((self.store.root / intent["publication_id"] / "journal-confirmation.json").exists())

    def test_after_journal_before_confirmation_is_idempotent(self):
        intent = self.prepare()
        script = self.child_loader() + """
publish=m.publication.durable.publish
def crash(path,data,**kwargs):
 if pathlib.Path(path).name=="journal-confirmation.json": os._exit(73)
 return publish(path,data,**kwargs)
m.publication.durable.publish=crash
store.reconcile()
"""
        result = self.child(script)
        self.assertEqual(result.returncode, 73, result.stderr)
        self.store.reconcile()
        self.store.reconcile()
        self.store.reconcile()
        self.assertEqual(self.rows(), [intent["journal_row"]])
        confirmation = json.loads((self.store.root / intent["publication_id"] / "journal-confirmation.json").read_bytes())
        self.assertEqual(confirmation["row_sha256"], hashlib.sha256(pub.durable.encoded(intent["journal_row"])).hexdigest())

    def test_journal_fsync_failure_prevents_cleanup(self):
        intent = self.prepare()
        sync = pub.durable.sync_directory
        def fail(path):
            if Path(path) == self.journal.parent:
                raise OSError("journal fsync")
            return sync(path)
        with patch.object(pub.durable, "sync_directory", side_effect=fail), self.assertRaisesRegex(OSError, "fsync"):
            self.store.reconcile()
        self.assertFalse((self.store.root / intent["publication_id"] / "journal-confirmation.json").exists())
        self.assertIn(intent["merges"][0]["name"], mq.git(self.repo, "ls-remote", "origin").stdout)
        self.store.reconcile()
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def test_retry_across_midnight_keeps_exact_time(self):
        intent = self.prepare()
        with patch.object(mq, "utc_now", return_value="2026-10-31T00:01:00Z"):
            self.store.reconcile()
        self.assertEqual(self.rows(), [intent["journal_row"]])
        self.assertEqual(self.rows()[0]["publication_time"], CREATED)

    def test_changed_ready_ref_survives_cleanup(self):
        intent = self.prepare()
        branch = intent["merges"][0]
        self.f.write("advanced", "new ready\n")
        self.f.commit("new ready")
        advanced = self.f.tip()
        mq.git(self.repo, "push", "origin", "HEAD:refs/heads/" + branch["name"])
        self.store.reconcile()
        queue = self.f.queue()
        queue.finish_publication(intent)
        self.assertEqual(mq.git(self.origin, "rev-parse", "refs/heads/" + branch["name"]).stdout.strip(), advanced)
        self.assertEqual(self.rows()[0], intent["journal_row"])

    def test_baseline_above_merge_is_not_a_merge_entry(self):
        intent = self.prepare(baseline=True)
        self.store.reconcile()
        self.assertNotEqual(intent["candidate"], intent["merges"][-1]["commit"])
        self.assertEqual(len(self.rows()[0]["merges"]), 1)

    def test_nested_reachable_and_deleted_ready_inputs(self):
        first = self.f.branch("first", publish=True)
        second = self.f.branch("second", publish=True)
        mq.git(self.repo, "merge", "--no-ff", first["tip"], "-m", "nested")
        second["tip"] = self.f.tip()
        mq.git(self.repo, "push", "origin", "HEAD:refs/heads/" + second["name"])
        intent = self.prepare([second, first])
        mq.git(self.repo, "push", "origin", ":refs/heads/" + second["name"])
        self.store.reconcile()
        self.assertEqual([m["name"] for m in intent["merges"]], [second["name"]])
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def test_graph_parent_order_and_bool_fields_are_refused(self):
        intent = self.prepare()
        for key, value in (("version", True), ("candidate", self.base), ("merges", [])):
            changed = {**intent, key: value}
            with self.subTest(key=key), self.assertRaises(ValueError):
                pub.validate(changed, mq.git, self.repo)
        changed = copy.deepcopy(intent)
        changed["merges"][0]["parent"] = changed["merges"][0]["tip"]
        with self.assertRaisesRegex(ValueError, "parents/order"):
            pub.graph(changed, mq.git, self.repo)

    def test_partial_conflicting_and_duplicate_journal_are_refused(self):
        intent = self.prepare()
        rows = [b'{"partial":', b'[]\n', b'{"duplicate":1,"duplicate":2}\n',
                b'{"publication_id":"00000000000000000000000000000000"}\n',
                pub.durable.encoded({**intent["journal_row"], "event": "recovery"}),
                pub.durable.encoded({**intent["journal_row"], "end": "2026-10-31T00:00:00Z"}),
                pub.durable.encoded(intent["journal_row"]) * 2]
        for data in rows:
            self.journal.write_bytes(data)
            self.journal.chmod(0o600)
            with self.subTest(data=data), self.assertRaises(ValueError):
                self.store.reconcile()
            self.assertEqual(self.journal.read_bytes(), data)
            self.assertFalse((self.store.root / intent["publication_id"] / "journal-confirmation.json").exists())
        self.journal.write_bytes(b'{"event":"existing","outcome":"red"}\n')
        self.store.reconcile()
        self.assertEqual(self.rows(), [{"event": "existing", "outcome": "red"}, intent["journal_row"]])

    def test_pending_recovery_precedes_invalid_replacement_roster(self):
        intent = self.prepare()
        changed = self.f.directory / "invalid-runtime"
        changed.mkdir()
        (changed / "coordinators.txt").write_bytes(b"invalid\n")
        with patch.object(mq, "CONFIG", changed), self.assertRaises(ValueError):
            self.f.queue()
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def test_two_merges_and_amendment_preserve_exact_parents(self):
        self.f.ratchet_base()
        self.base = self.f.base
        first = self.f.ratchet_branch("a", "a", 1)
        second = self.f.ratchet_branch("b", "b", 2)
        intent = self.prepare([first, second])
        self.store.reconcile()
        merges = self.rows()[0]["merges"]
        self.assertEqual([m["name"] for m in merges], [first["name"], second["name"]])
        self.assertEqual(merges[1]["commit"], intent["candidate"])
        self.assertEqual(merges[1]["parent"], merges[0]["commit"])
        self.assertEqual(mq.git(self.repo, "show", intent["candidate"] + ":skein/scripts/old-forms.txt").stdout, "0\n")
        pub.graph(intent, mq.git, self.repo)

    def test_nonpublication_and_recovery_rows_have_no_merge_fields(self):
        queue = self.f.queue()
        red = self.f.branch("red")
        queue.attempt([red])
        green = self.f.branch("dry")
        dry = self.f.queue(dry=True)
        dry.attempt([green])
        queue.attempt([], idle=True)
        first = self.f.branch("first", "shared", "one\n")
        conflict = self.f.branch("conflict", "shared", "two\n")
        queue.attempt([first, conflict])
        queue.checker = lambda *args: (_ for _ in ()).throw(RuntimeError("fixture gate error"))
        with self.assertRaisesRegex(RuntimeError, "gate error"):
            queue.attempt([green])
        queue.recover(RuntimeError("fixture gate error"))
        rows = self.rows()
        self.assertEqual([r["outcome"] for r in rows], ["red", "green", "green", "conflict", "error", "error"])
        for row in rows:
            self.assertFalse(pub.LANDING_FIELDS & row.keys(), row)

    def runtime(self, directory=None):
        directory = directory or self.f.directory / "runtime"
        directory.mkdir(parents=True, exist_ok=True)
        for name in fixtures.restart.RUNTIME_FILES:
            (directory / name).write_bytes((DIRECTORY / name).read_bytes())
        return directory

    def test_runtime_manifests_freeze_coordinators_and_parser(self):
        runtime = self.runtime()
        (runtime / "coordinators.txt").write_bytes(ROSTER)
        module, _ = fixtures.restart.runtime_module(runtime, self.repo, external=True)
        spec = importlib.util.spec_from_file_location("publication_worker", runtime / "worker.py")
        worker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(worker)
        digest = hashlib.sha256(ROSTER).hexdigest()
        self.assertEqual(module.restart_identity["coordinators_sha256"], digest)
        self.assertEqual(worker.runtime_identity()["coordinators_sha256"], digest)
        self.assertEqual(set(module.restart_identity["files"]), set(worker.runtime_identity()["files"]))
        self.assertIn("publication.py", module.restart_identity["files"])
        (runtime / "coordinators.txt").write_bytes(b"coord -\n")
        with self.assertRaisesRegex(RuntimeError, "runtime/config"):
            worker.runtime_unchanged()
        with self.assertRaisesRegex(RuntimeError, "runtime changed"):
            fixtures.restart.runtime_unchanged(runtime, module.restart_frozen)

    def test_restart_rejects_missing_or_invalid_routes_before_stop(self):
        runtime = self.runtime()
        home = self.f.directory / "home"
        for data in (None, b"coord local\n", b"coord -\ncoord -\n"):
            path = runtime / "coordinators.txt"
            if data is None:
                path.unlink()
            else:
                path.write_bytes(data)
            with patch.object(fixtures.restart, "queue_pid") as pid, self.assertRaises((OSError, ValueError)):
                fixtures.restart.restart(self.repo, 0, home, runtime=runtime, authority_dir=self.layout.root)
            pid.assert_not_called()
            self.assertFalse((home / "src/fabrig/queue.stop").exists())

    def test_restart_recovers_before_checkout_reset_with_saved_routes(self):
        intent = self.prepare()
        mq.git(self.repo, "push", str(self.origin), intent["candidate"] + ":refs/heads/main")
        runtime = self.runtime()
        (runtime / "coordinators.txt").write_bytes(b"coord s/replacement@fixture.example\n")
        home = self.f.directory / "home"
        (home / "src/fabrig").mkdir(parents=True)
        self.lease.close()
        self.checkout.close()
        command = fixtures.restart.command
        observed = []
        def inspect(argv, *args, **kwargs):
            if argv[:2] == ["git", "switch"]:
                observed.append(self.rows()[0])
            return command(argv, *args, **kwargs)
        original = subprocess.Popen
        with patch.object(fixtures.restart, "queue_pid", return_value=None), \
                patch.object(fixtures.restart, "command", side_effect=inspect), \
                patch.object(fixtures.restart.subprocess, "Popen", wraps=subprocess.Popen) as launch:
            # Block only the replacement service, allowing all fixture Git children.
            def no_service(argv, **kwargs):
                if argv[0] != "git":
                    raise OSError("fixture replacement not launched")
                return original(argv, **kwargs)
            launch.side_effect = no_service
            with self.assertRaisesRegex(OSError, "not launched"):
                fixtures.restart.restart(self.repo, 0, home, runtime=runtime, authority_dir=self.layout.root)
        self.assertEqual(observed, [intent["journal_row"]])
        self.assertTrue((home / "src/fabrig/queue.stop").exists())

    def test_intent_directory_fsync_failure_is_completed_before_push(self):
        sync = pub.durable.sync_directory
        def fail(path):
            if (Path(path) / "intent.json").exists():
                raise OSError("intent directory fsync")
            return sync(path)
        with patch.object(pub.durable, "sync_directory", side_effect=fail), self.assertRaisesRegex(OSError, "intent directory"):
            self.prepare()
        self.assertEqual(self.remote(), self.base)
        self.store.reconcile()
        self.assertEqual(len(self.rows()), 1)

    def test_startup_block_alerts_ops_and_preserves_checkout(self):
        intent = self.prepare()
        self.origin.rename(self.origin.with_suffix(".unavailable"))
        before = self.f.tip()
        with self.assertRaises(RuntimeError):
            self.f.queue()
        self.assertEqual(self.f.tip(), before)
        self.assertEqual(self.f.posts[0][1], "queue publication recovery blocked")
        self.assertTrue((self.store.root / intent["publication_id"] / "intent.json").exists())

    def test_already_reachable_ready_tip_is_consumed_without_publication(self):
        branch = {"name": "ready/coord/already", "tip": self.base, "timestamp": 1}
        mq.git(self.repo, "push", "origin", self.base + ":refs/heads/" + branch["name"])
        queue = self.f.queue()
        self.assertTrue(queue.attempt([branch]))
        self.assertEqual(self.rows()[0]["event"], "already-reachable")
        self.assertFalse(pub.LANDING_FIELDS & self.rows()[0].keys())
        self.assertEqual(queue.state["tried"], {branch["name"]: self.base})
        self.assertEqual(self.store.entries(), [])
        self.assertEqual(self.f.checks, [])

    def test_confirmation_directory_fsync_failure_blocks_cleanup_until_durable(self):
        intent = self.prepare()
        directory = self.store.root / intent["publication_id"]
        sync = pub.durable.sync_directory
        def fail(path):
            if Path(path) == directory and (directory / "journal-confirmation.json").exists():
                raise OSError("confirmation directory fsync")
            return sync(path)
        with patch.object(pub.durable, "sync_directory", side_effect=fail):
            with self.assertRaisesRegex(OSError, "confirmation directory"):
                self.store.reconcile()
            with self.assertRaisesRegex(OSError, "confirmation directory"):
                self.f.queue()
        self.assertIn(intent["merges"][0]["name"], mq.git(self.repo, "ls-remote", "origin").stdout)
        self.store.reconcile()
        self.assertEqual(self.rows(), [intent["journal_row"]])

    def test_diagnostic_writer_cannot_race_publication_replacement(self):
        intent = self.prepare()
        publish = pub.durable.publish
        observations = []
        def inspect(path, data, **kwargs):
            if Path(path) == self.journal:
                with self.assertRaisesRegex(RuntimeError, "holds this checkout or queue authority"):
                    mq.log_batch(self.journal, {"event": "refusal"})
                observations.append("excluded")
            return publish(path, data, **kwargs)
        with patch.object(pub.durable, "publish", side_effect=inspect):
            self.store.reconcile()
        mq.log_batch(self.journal, {"event": "refusal"})
        self.assertEqual(observations, ["excluded"])
        self.assertEqual(self.rows(), [intent["journal_row"], {"event": "refusal"}])

    def test_plan_and_dry_block_pending_publication_without_push(self):
        intent = self.prepare()
        for options in ({"dry": True}, {"read_only": True}):
            with self.subTest(options=options), self.assertRaisesRegex(RuntimeError, "nondry reconciliation"):
                self.f.queue(**options)
        self.assertEqual(self.remote(), self.base)
        self.assertEqual(self.rows(), [])
        self.assertTrue((self.store.root / intent["publication_id"] / "intent.json").exists())

    def test_default_restart_validates_target_before_requesting_stop(self):
        target = self.repo / "tools/queue"
        home = self.f.directory / "home"
        cases = [("coordinators.txt", None), ("coordinators.txt", b"coord local\n"),
                 ("coordinators.txt", b"coord -\ncoord -\n"), ("mq", b"invalid Python!\n")]
        for name, data in cases:
            self.runtime(target)
            path = target / name
            if data is None:
                path.unlink()
            else:
                path.write_bytes(data)
            with self.subTest(name=name, data=data), \
                    patch.object(fixtures.restart, "queue_pid", side_effect=AssertionError("invalid target reached drain")) as pid:
                with self.assertRaises((OSError, ValueError, SyntaxError)):
                    fixtures.restart.restart(self.repo, 0, home, authority_dir=self.layout.root)
                pid.assert_not_called()
                self.assertFalse((home / "src/fabrig/queue.stop").exists())

    def unrelated_publication_rows(self, intent):
        row = copy.deepcopy(intent["journal_row"])
        row["publication_id"] = "a" * 32
        return [("duplicate", row, copy.deepcopy(row)),
                ("conflict", row, {**row, "selection": "full"})]

    def test_publication_rejects_duplicates_of_unrelated_identity(self):
        intent = self.prepare()
        for label, first, second in self.unrelated_publication_rows(intent):
            with self.subTest(kind=label):
                data = pub.durable.encoded(first) + pub.durable.encoded(second)
                self.journal.write_bytes(data)
                self.journal.chmod(0o600)
                with self.assertRaisesRegex(ValueError, "duplicate|conflict"):
                    self.store.journal(intent, append=True)
                self.assertEqual(self.journal.read_bytes(), data)
                self.assertFalse((self.store.root / intent["publication_id"] / "journal-confirmation.json").exists())

    def test_audit_rejects_duplicates_of_any_publication_identity(self):
        intent = self.prepare()
        for label, first, second in self.unrelated_publication_rows(intent):
            with self.subTest(kind=label):
                data = pub.durable.encoded(first) + pub.durable.encoded(second)
                self.journal.write_bytes(data)
                self.journal.chmod(0o600)
                with self.assertRaisesRegex(ValueError, "duplicate|conflict"):
                    mq.log_batch(self.journal, {"event": "fixture-refusal"})
                self.assertEqual(self.journal.read_bytes(), data)
                prefix = pub.durable.encoded(first)
                self.journal.write_bytes(prefix)
                with self.assertRaisesRegex(ValueError, "duplicate|conflict"):
                    mq.log_batch(self.journal, second)
                self.assertEqual(self.journal.read_bytes(), prefix)

    def test_unique_unrelated_publication_and_current_replay_preserve_rows(self):
        intent = self.prepare()
        first = self.unrelated_publication_rows(intent)[0][1]
        prefix = pub.durable.encoded(first)
        self.journal.write_bytes(prefix)
        self.journal.chmod(0o600)
        self.store.journal(intent, append=True)
        self.store.journal(intent, append=True)
        mq.log_batch(self.journal, {"event": "fixture-refusal"})
        self.assertEqual(self.journal.read_bytes(), prefix + pub.durable.encoded(intent["journal_row"]) +
                         pub.durable.encoded({"event": "fixture-refusal"}))

    def test_default_restart_rechecks_target_bytes_after_drain(self):
        target = self.repo / "tools/queue"
        home = self.f.directory / "home"
        (home / "src/fabrig").mkdir(parents=True)
        self.lease.close()
        self.checkout.close()
        select = fixtures.restart.select.select
        for name in ("coordinators.txt", "mq"):
            self.runtime(target)
            ready_read, ready_write = os.pipe()
            os.close(ready_write)
            calls = []
            def changed(fds, *args):
                if fds == [ready_read]:
                    path = target / name
                    path.write_bytes(path.read_bytes() + b"\n")
                return select(fds, *args)
            def command(argv, repo, **kwargs):
                calls.append(argv)
                return str(repo)
            with self.subTest(name=name), \
                    patch.object(fixtures.restart, "queue_pid", return_value=42), \
                    patch.object(fixtures.restart.os, "pidfd_open", return_value=ready_read), \
                    patch.object(fixtures.restart.select, "select", side_effect=changed), \
                    patch.object(fixtures.restart, "command", side_effect=command), \
                    patch.object(fixtures.restart.subprocess, "Popen") as spawn:
                with self.assertRaisesRegex(RuntimeError, "runtime changed"):
                    fixtures.restart.restart(self.repo, 0, home, authority_dir=self.layout.root)
                self.assertEqual(calls, [["git", "rev-parse", "--show-toplevel"]])
                self.assertTrue((home / "src/fabrig/queue.stop").exists())
                spawn.assert_not_called()

    def test_default_restart_rechecks_target_bytes_before_launch(self):
        target = self.runtime(self.repo / "tools/queue")
        home = self.f.directory / "home"
        (home / "src/fabrig").mkdir(parents=True)
        self.lease.close()
        self.checkout.close()
        def command(argv, repo, **kwargs):
            if argv[:2] == ["git", "switch"]:
                path = target / "mq"
                path.write_bytes(path.read_bytes() + b"\n")
            return str(repo)
        with patch.object(fixtures.restart, "queue_pid", return_value=None), \
                patch.object(fixtures.restart, "command", side_effect=command), \
                patch.object(fixtures.restart.subprocess, "Popen") as spawn:
            with self.assertRaisesRegex(RuntimeError, "runtime changed"):
                fixtures.restart.restart(self.repo, 0, home, authority_dir=self.layout.root)
        self.assertTrue((home / "src/fabrig/queue.stop").exists())
        spawn.assert_not_called()


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
