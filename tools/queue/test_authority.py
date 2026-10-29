#!/usr/bin/env python3
"""Sole authority and concrete effects; disposable Git, fake post, pipe barriers."""

from contextlib import redirect_stderr, redirect_stdout
import copy
import importlib.util
import io
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
SPEC = importlib.util.spec_from_file_location("authority_fixture", DIRECTORY / "train_fixture.py")
fixtures = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(fixtures)
train, mq, worker = fixtures.train, fixtures.mq, fixtures.worker
authority = mq.authority
RESTART = importlib.util.spec_from_file_location("restart_fixture", DIRECTORY / "test_mq.py")
restart_fixture = importlib.util.module_from_spec(RESTART)
RESTART.loader.exec_module(restart_fixture)
restart = restart_fixture.restart

# No stdin control channel or synchronization sleeps: explicit inherited pipes.
HOLDER = '''import importlib.util, os, pathlib, sys
spec = importlib.util.spec_from_file_location('authority', sys.argv[1])
a = importlib.util.module_from_spec(spec); spec.loader.exec_module(a)
layout = a.Layout(pathlib.Path(sys.argv[2]))
with (layout.restart_lock() if sys.argv[3] == 'restart' else layout.acquire()) as lock:
    os.write(int(sys.argv[4]), b'held')
    os.read(int(sys.argv[5]), 1)
'''


class EffectsTests(fixtures.TrainFixture, unittest.TestCase):
    def setUp(self):
        super().setUp()
        self.batch = getattr(self, "batch_size", self.batch)
        self.layout = authority.Layout(self.root / "shared-authority")
        self.lease = self.layout.acquire()
        self.addCleanup(self.lease.close)
        self.calls = []
        self.post_code = 0
        self.lose_ack = False
        self.before_cleanup = None
        def runner(argv, **kwargs):
            self.calls.append((list(argv), dict(kwargs)))
            if argv[0] == "post":
                return subprocess.CompletedProcess(argv, self.post_code, "", "")
            if self.before_cleanup and argv[1] == "push" and argv[-1].startswith(":refs/heads/ready/"):
                callback, self.before_cleanup = self.before_cleanup, None
                callback()
            result = subprocess.run(argv, **kwargs)
            if self.lose_ack and argv[1] == "push" and argv[-1].endswith(":refs/heads/main"):
                raise OSError("lost main acknowledgement")
            return result
        self.controller_repo = self.root / "controller"
        mq.git(self.root, "clone", str(self.origin), str(self.controller_repo))
        self.effect = authority.GitAuthority(self.layout, self.lease, self.controller_repo, str(self.origin),
                                            {"ops": "ops@fixture", "skein": "lead/skein@fixture"}, runner=runner)
        self.store = train.StateStore(self.layout.train)
        self.authority = self.effect
        self.controller = self.reload()
        self.controller.state["last_full"] = self.clock
        self.controller.save()

    def test_concrete_push_uses_durable_p_c_intent_and_explicit_oid(self):
        self.branch("literal")
        a, r = self.admit()
        self.finish(a)
        # HEAD is deliberately moved after certification. Push must still use C.
        mq.git(self.prep, "switch", "--detach", self.base)
        self.assertTrue(self.controller.adjudicate_head())
        pushes = [argv for argv, _ in self.calls if argv[:2] == ["git", "push"]]
        self.assertEqual(pushes, [["git", "push", "--", str(self.origin), r["candidate"] + ":refs/heads/main"]])
        self.assertEqual(self.effect.main(), r["candidate"])
        self.assertEqual(mq.git(self.controller_repo, "rev-parse", "HEAD").stdout.strip(), self.base)
        self.assertTrue(any(argv[:3] == ["git", "bundle", "unbundle"] for argv, _ in self.calls))
        self.assertFalse(any("--force" in arg or "HEAD" in arg for argv in pushes for arg in argv))

    def test_direct_promotion_without_intent_has_no_git_effect(self):
        self.calls.clear()
        with self.assertRaises(ValueError):
            self.effect.promote(self.base, self.base, "invented")
        self.assertEqual(self.calls, [])

    def test_changed_main_refuses_certified_push(self):
        self.branch("a")
        a, _ = self.admit()
        self.finish(a)
        other = self.branch("other")
        mq.git(self.prep, "push", "origin", other["tip"] + ":refs/heads/main")
        self.assertFalse(self.controller.adjudicate_head())
        self.assertFalse(any(argv[:2] == ["git", "push"] for argv, _ in self.calls))
        self.assertEqual(self.controller.state["tried"], {})

    def test_lost_main_ack_reloads_one_decision_and_counters(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        self.lose_ack = True
        self.assertFalse(self.controller.adjudicate_head())
        self.assertEqual(self.effect.main(), r["candidate"])
        self.controller = self.reload()
        self.assertEqual(self.controller.recovery_inventory()["promotions"][0]["disposition"], "candidate")
        self.assertTrue(self.controller.adjudicate_head())
        self.controller = self.reload()
        self.assertEqual(self.controller.state["batches_since_full"], 1)
        self.assertEqual(len(self.controller.state["outbox"]), 2)
        self.assertEqual(len([argv for argv, _ in self.calls if argv[:2] == ["git", "push"]]), 1)

    def test_parent_after_push_failure_retries_same_intent(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        push = self.effect.promote
        with patch.object(self.effect, "promote", side_effect=OSError("before push")):
            self.assertFalse(self.controller.adjudicate_head())
        saved = copy.deepcopy(r["promotion"])
        self.controller = self.reload()
        self.assertEqual(self.controller.recovery_inventory()["promotions"][0]["disposition"], "parent")
        self.assertTrue(self.controller.adjudicate_head())
        self.assertEqual(self.controller.state["records"][a]["promotion"], saved)

    def test_cleanup_advancement_between_lease_and_delete_preserves_new_tip(self):
        selected = self.branch("selected")
        replacement = self.branch("replacement", base=selected["tip"])
        self.before_cleanup = lambda: mq.git(self.prep, "push", "origin",
            replacement["tip"] + ":refs/heads/" + selected["name"])
        self.assertTrue(self.effect.cleanup(selected["name"], selected["tip"], "cleanup-event"))
        self.assertEqual(self.effect.remote("refs/heads/" + selected["name"]), replacement["tip"])
        delete = [argv for argv, _ in self.calls if argv[:2] == ["git", "push"]][0]
        ref = "refs/heads/" + selected["name"]
        self.assertEqual(delete, ["git", "push", "--force-with-lease=" + ref + ":" + selected["tip"],
                                  "--", str(self.origin), ":" + ref])

    def test_deleted_recreated_different_ready_tip_survives(self):
        selected = self.branch("selected")
        replacement = self.branch("replacement")
        ref = "refs/heads/" + selected["name"]
        mq.git(self.prep, "push", "origin", ":" + ref)
        self.assertTrue(self.effect.cleanup(selected["name"], selected["tip"], "cleanup-event"))
        mq.git(self.prep, "push", "origin", replacement["tip"] + ":" + ref)
        self.assertTrue(self.effect.cleanup(selected["name"], selected["tip"], "cleanup-event"))
        self.assertEqual(self.effect.remote(ref), replacement["tip"])

    def test_cleanup_failure_stays_pending_without_changing_landing(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        def failed(*args, **kwargs):
            if args[0] == "push":
                return subprocess.CompletedProcess([], 1, "", "unreachable")
            raise OSError("read also unreachable")
        with patch.object(self.effect, "git", side_effect=failed):
            self.controller.flush_outbox()
        self.assertTrue(r["applied"])
        self.assertEqual(r["decision"], "green")
        self.assertTrue(any(e["kind"] == "cleanup" and not e["delivered"] for e in self.controller.state["outbox"].values()))
        self.controller.flush_outbox()
        self.assertTrue(all(e["delivered"] for e in self.controller.state["outbox"].values()))

    def test_posts_route_frozen_leads_and_replay_stable_ids_for_exits_zero_three(self):
        self.branch("a")
        a, r = self.admit()
        self.finish(a)
        self.controller.adjudicate_head()
        self.post_code = 2
        self.controller.flush_outbox()
        first = [argv for argv, _ in self.calls if argv[0] == "post"]
        self.assertEqual(len(first), 1)
        self.controller = self.reload()
        self.post_code = 3
        self.controller.flush_outbox()
        posts = [(argv, kw) for argv, kw in self.calls if argv[0] == "post"]
        self.assertEqual(sorted(argv for argv, _ in posts[1:]), sorted(first))
        for argv, kwargs in posts:
            self.assertEqual(argv[:5], ["post", "send", "--as", "queue", "ops@fixture"])
            self.assertIn("Event: ", kwargs["input"])
            self.assertIn("Evidence: " + str(self.artifacts / a), kwargs["input"])
            self.assertNotIn("shell", kwargs)
        for event in self.controller.state["outbox"].values():
            if event["kind"] != "cleanup":
                self.post_code = 0
                self.assertEqual(self.effect.post(event), 0)
        original = copy.deepcopy(next(e for e in self.controller.state["outbox"].values() if e["kind"] == "landed"))
        for kind in ("infrastructure", "selection-gap", "rejected"):
            event = copy.deepcopy(original)
            event["kind"] = kind
            event["body"] += "Literal $(touch ignored); `text`\n"
            self.assertEqual(self.effect.post(event), 0)
            self.assertEqual(self.calls[-1][0][4], "ops@fixture")
        area = copy.deepcopy(next(e for e in self.controller.state["outbox"].values() if e["kind"] == "landed"))
        area["branch"]["name"] = "ready/skein/a"
        self.effect.post(area)
        self.assertEqual(self.calls[-1][0][4], "lead/skein@fixture")

    def test_origin_refs_oids_and_configuration_are_validated_before_push(self):
        for origin in ("-option", "ext::evil", "relative", "ssh://host/repo\nother"):
            with self.subTest(origin=origin), self.assertRaises(ValueError):
                authority.GitAuthority(self.layout, self.lease, self.prep, origin, {"ops": "ops@fixture"})
        for name, tip in (("main", self.base), ("ready/ops/bad..ref", self.base), ("ready/ops/a", "HEAD")):
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.effect.cleanup(name, tip, "event")
        self.effect.leads["ops"] = "changed@fixture"
        with self.assertRaises(RuntimeError):
            self.effect.main()

    def test_production_binding_requires_shared_state_location(self):
        with self.assertRaises(ValueError):
            self.effect.bind(train.StateStore(self.state_dir), self.preparer, config=self.controller.config)

    def test_controller_hard_exit_at_dispatch_keeps_real_snapshot_and_legacy_refusal(self):
        self.branch("a")
        self.lease.close()
        code = """import importlib.util, os, pathlib, sys
s=importlib.util.spec_from_file_location('t',sys.argv[1]);t=importlib.util.module_from_spec(s);s.loader.exec_module(t)
a=t.authority_module; root=pathlib.Path(sys.argv[2]); layout=a.Layout(root/'shared-authority')
with layout.acquire() as held:
 store=t.StateStore(layout.train); config=store.load()['config']
 class Transport:
  def identity(self): return config['transport']
  def completion_identity(self): return config['completion']
  def start(self,request,bundle): os._exit(23)
 def no_metadata(repo): raise RuntimeError('unexpected metadata command')
 prep=t.Preparer(pathlib.Path(config['preparation']),pathlib.Path(config['artifacts']),metadata_loader=no_metadata,disk_floor=0)
 effect=a.GitAuthority(layout,held,pathlib.Path(config['authority']['repo']),config['authority']['origin'],config['authority']['leads'])
 transport=Transport()
 controller=t.Controller(prep,store,transport,effect,generation=config['generation'],gates=config['gates'],runtime=t.worker.runtime_identity(0),now=lambda:10000,batch=config['batch'],completion_adapter=transport)
 controller.admit()
 raise RuntimeError('dispatch boundary not reached')
"""
        child = subprocess.run([sys.executable, "-I", "-B", "-c", code,
            str(DIRECTORY / "train.py"), str(self.root)], stdin=subprocess.DEVNULL, capture_output=True)
        self.assertEqual(child.returncode, 23, child.stderr)
        state = self.store.load()
        a = state["active"][0]
        self.assertEqual(state["records"][a]["phase"], "dispatch-intent")
        self.assertIn(a, state["gates"].values())
        with self.layout.acquire():
            with self.assertRaisesRegex(RuntimeError, "train recovery"):
                self.layout.legacy_guard(self.prep)
        with patch.object(sys, "argv", ["mq", "once", "--repo", str(self.controller_repo),
                     "--authority-dir", str(self.layout.root)]), patch.object(mq, "Queue") as queue, \
                redirect_stderr(io.StringIO()):
            self.assertEqual(mq.main(), 1)
        queue.assert_not_called()

    def test_train_owner_receipt_survives_missing_state_and_blocks_legacy(self):
        self.store.path.unlink()
        with self.assertRaisesRegex(RuntimeError, "train recovery"):
            self.layout.legacy_guard(self.prep)
        self.assertTrue((self.layout.root / "train-owner.json").exists())

    def test_authority_loss_and_replaced_inode_block_effects(self):
        self.lease.path.unlink()
        self.lease.path.touch(mode=0o600)
        with self.assertRaises(RuntimeError):
            self.effect.main()


class BisectionEffectsTests(EffectsTests):
    batch_size = 2

    def red_pair(self):
        return self.red_inputs(("a", "b"))

    def red_inputs(self, names):
        inputs = [self.branch(name, {f"core/src/{name}.rs": name}) for name in names]
        self.controller.state["batches_since_full"] = 4
        attempt, record = self.admit()
        self.assertEqual(record["request"]["selection"], "full")
        self.finish(attempt, code=1)
        self.assertTrue(self.controller.adjudicate_head())
        self.controller = self.reload()
        middle = len(inputs) // 2
        self.assertEqual(self.controller.state["pending"], [inputs[:middle], inputs[middle:]])
        return inputs

    def complete_group(self, attempt, *, code=0):
        self.controller = self.reload()
        self.finish(attempt, code=code)
        self.controller = self.reload()
        self.assertTrue(self.controller.adjudicate_head())
        self.controller = self.reload()

    def test_admitted_left_reloads_and_concrete_promotions_preserve_right_parent(self):
        inputs = self.red_pair()
        a, left = self.admit()
        self.assertEqual(self.controller.state["pending"], [[inputs[1]]])
        self.complete_group(a)
        self.assertEqual(self.effect.main(), left["candidate"])
        b, right = self.admit()
        self.assertEqual(right["parent"], left["candidate"])
        self.complete_group(b)
        self.assertEqual(self.effect.main(), right["candidate"])
        self.assertTrue(self.controller.drained())
        self.assertIsNone(self.controller.admit())
        self.controller = self.reload()
        self.assertFalse(self.controller.state["bisecting"])
        self.assertEqual(self.controller.state["pending"], [])

    def test_admitted_singleton_red_reloads_then_right_lands_on_unchanged_main(self):
        inputs = self.red_pair()
        a, _ = self.admit()
        self.complete_group(a, code=1)
        self.assertEqual(self.controller.state["tried"], {inputs[0]["name"]: inputs[0]["tip"]})
        b, right = self.admit()
        self.assertEqual(right["parent"], self.base)
        self.assertEqual(right["request"]["selection"], "full")
        self.complete_group(b)
        self.assertEqual(self.effect.main(), right["candidate"])

    def test_admitted_infra_requeues_once_after_verified_release_and_reload(self):
        inputs = self.red_pair()
        a, left = self.admit()
        self.controller = self.reload()
        self.transport.fail_status = True
        self.controller.poll(a)
        self.controller.adjudicate_head()
        self.controller = self.reload()
        self.assertEqual(self.controller.state["pending"], [[inputs[0]], [inputs[1]]])
        self.assertEqual(self.controller.state["gates"][left["gate"]], a)
        self.assertFalse(self.controller.drained())
        self.transport.fail_status = False
        self.transport.finish(a)
        self.controller.reconcile_job(a, quiescent=True, evidence=self.transport.jobs[a]["evidence"])
        self.controller = self.reload()
        self.assertEqual(self.controller.state["tried"], {})
        retry, accepted = self.admit()
        self.assertEqual(accepted["inputs"], [inputs[0]])
        self.complete_group(retry)
        b, right = self.admit()
        self.assertEqual(right["parent"], accepted["candidate"])
        self.complete_group(b)
        self.assertEqual(self.effect.main(), right["candidate"])


class NestedBisectionEffectsTests(BisectionEffectsTests):
    batch_size = 4

    def test_nested_red_infra_retry_and_singleton_red_reload_each_frontier(self):
        inputs = self.red_inputs(("a", "b", "c", "d"))
        left, _ = self.admit()
        self.complete_group(left, code=1)
        self.assertEqual(self.controller.state["pending"], [[inputs[0]], [inputs[1]], inputs[2:]])
        for branch in inputs[:2]:
            attempt, record = self.admit()
            self.assertEqual(record["inputs"], [branch])
            self.complete_group(attempt)
        parent = self.effect.main()
        right, record = self.admit()
        self.assertEqual(record["inputs"], inputs[2:])
        self.transport.fail_status = True
        self.controller.poll(right)
        self.controller.adjudicate_head()
        self.controller = self.reload()
        self.assertEqual(self.controller.state["pending"], [inputs[2:]])
        self.transport.fail_status = False
        self.transport.finish(right)
        self.controller.reconcile_job(right, quiescent=True, evidence=self.transport.jobs[right]["evidence"])
        self.controller = self.reload()
        retry, _ = self.admit()
        self.complete_group(retry, code=1)
        self.assertEqual(self.controller.state["pending"], [[inputs[2]], [inputs[3]]])
        rejected, _ = self.admit()
        self.complete_group(rejected, code=1)
        self.assertEqual(self.effect.main(), parent)
        final, record = self.admit()
        self.assertEqual(record["parent"], parent)
        self.complete_group(final)
        self.assertEqual(self.effect.main(), record["candidate"])
        self.assertEqual(self.controller.state["tried"], {b["name"]: b["tip"] for b in inputs})
        self.assertTrue(self.controller.drained())


class GuardTests(unittest.TestCase):
    def legacy_ratchet_survives_parent(self, launch):
        ready_r, ready_w = os.pipe()
        release_r, release_w = os.pipe()
        script = self.repo / "jig/scripts/old_forms.py"
        script.parent.mkdir(parents=True)
        script.write_text('''import os, pathlib, sys
os.write(int(sys.argv[2]), str(os.getpid()).encode() + bytes([10]))
os.read(int(sys.argv[3]), 1)
pathlib.Path(sys.argv[4]).write_text('ratchet completed')
''')
        output = self.repo / "ratchet-finished"
        code = '''import importlib.machinery, importlib.util, pathlib, subprocess, sys, time
settings=sys.argv[:]
loader=importlib.machinery.SourceFileLoader('mq',settings[1]);spec=importlib.util.spec_from_loader(loader.name,loader)
mq=importlib.util.module_from_spec(spec);loader.exec_module(mq)
real_run=subprocess.run
def run(argv, **kwargs):
 kwargs['pass_fds']=(*kwargs.get('pass_fds',()),int(settings[5]),int(settings[6]))
 return real_run(argv,**kwargs)
mq.subprocess.run=run
class RatchetQueue:
 def __init__(self,repo,writer_guard,**kwargs): self.repo,self.guard=repo,writer_guard
 def cycle(self):
  self.guard()
  argv=[sys.executable,'-I','-B',settings[4],'--write',settings[5],settings[6],settings[7]]
  if settings[8]=='command': mq.command(argv,self.repo)
  else:
   with (pathlib.Path(settings[7]).parent/'ratchet.log').open('x+') as log:
    mq.run_step('ratchet:jig',argv,self.repo,log,runner=mq.subprocess.run,clock=time.monotonic)
mq.Queue=RatchetQueue
sys.argv=['mq','once','--repo',settings[2],'--authority-dir',settings[3]]
sys.exit(mq.main())
'''
        parent = subprocess.Popen([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "mq"),
            str(self.repo), str(self.layout.root), str(script), str(ready_w), str(release_r), str(output), launch],
            pass_fds=(ready_w, release_r), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        os.close(ready_w)
        os.close(release_r)
        worker_code = '''import importlib.util, pathlib, sys
s=importlib.util.spec_from_file_location('w',sys.argv[1]);w=importlib.util.module_from_spec(s);s.loader.exec_module(w)
try:
 with w.checkout_ownership(pathlib.Path(sys.argv[2])): print('admitted')
except RuntimeError as error:
 print(str(error),file=sys.stderr);sys.exit(2)
'''
        def worker_admission():
            return subprocess.run([sys.executable, "-I", "-B", "-c", worker_code,
                str(DIRECTORY / "worker.py"), str(self.repo)], stdin=subprocess.DEVNULL, capture_output=True)
        pid_fd = None
        try:
            self.assertTrue(select.select([ready_r], [], [], 10)[0])
            ready = os.read(ready_r, 32)
            if not ready:
                out, err = parent.communicate(timeout=10)
                self.fail((out, err))
            pid_fd = os.pidfd_open(int(ready))
            parent.kill()
            parent.communicate(timeout=10)
            self.assertFalse(output.exists())
            with self.assertRaises(RuntimeError):
                self.layout.acquire()
            competitor = worker_admission()
            self.assertEqual(competitor.returncode, 2, competitor.stdout + competitor.stderr)
            self.assertIn(b"holds this checkout", competitor.stderr)
        finally:
            os.close(ready_r)
            os.close(release_w)
            if parent.poll() is None:
                parent.kill()
                parent.communicate(timeout=10)
            if pid_fd is not None:
                self.assertTrue(select.select([pid_fd], [], [], 10)[0])
                os.close(pid_fd)
        self.assertEqual(output.read_text(), "ratchet completed")
        self.assertEqual(worker_admission().returncode, 0)

    def test_legacy_command_child_retains_checkout_after_parent_death(self):
        self.legacy_ratchet_survives_parent("command")

    def test_legacy_step_child_retains_checkout_after_parent_death(self):
        self.legacy_ratchet_survives_parent("step")

    def setUp(self):
        os.umask(0o077)
        self.tmp = tempfile.TemporaryDirectory(dir=DIRECTORY.parent.parent.parent, prefix=".authority-test-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.home = self.root / "home"
        (self.home / "src/fabrig").mkdir(parents=True, mode=0o700)
        self.repo = self.home / "src/fabrig/alternate-preparation"
        self.repo.mkdir(mode=0o700)
        mq.git(self.repo, "init", "-b", "main")
        self.layout = authority.Layout(home=self.home)

    def test_fixture_root_is_outside_checked_source(self):
        self.assertNotIn(DIRECTORY.parent.parent, self.root.parents)

    def holder(self, kind="writer"):
        ready_r, ready_w = os.pipe()
        release_r, release_w = os.pipe()
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", HOLDER,
            str(DIRECTORY / "authority.py"), str(self.layout.root), kind, str(ready_w), str(release_r)],
            pass_fds=(ready_w, release_r), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        os.close(ready_w)
        os.close(release_r)
        self.assertTrue(select.select([ready_r], [], [], 10)[0])
        self.assertEqual(os.read(ready_r, 4), b"held")
        os.close(ready_r)
        def release():
            os.close(release_w)
            out, err = child.communicate(timeout=10)
            self.assertEqual(child.returncode, 0, (out, err))
        self.addCleanup(release)
        return child

    def test_one_default_layout_ignores_alternate_clone_names(self):
        self.assertEqual(authority.Layout(home=self.home).root, self.layout.root)
        self.holder()
        with self.assertRaises(RuntimeError):
            authority.Layout(home=self.home).acquire()
        with self.assertRaises(RuntimeError):
            authority.GitAuthority(self.layout, self.layout.acquire(), self.repo, str(self.root), {"ops": "ops@fixture"})

    def test_legacy_once_and_run_refuse_other_authority_before_fetch(self):
        self.holder()
        for mode in ("once", "run"):
            with patch.object(sys, "argv", ["mq", mode, "--repo", str(self.repo),
                         "--authority-dir", str(self.layout.root)]), patch.object(mq.shutil, "which", return_value="fake"), \
                    patch.object(mq, "Queue") as queue, redirect_stderr(io.StringIO()):
                self.assertEqual(mq.main(), 1)
                queue.assert_not_called()

    def test_restart_can_request_stop_without_writer_authority_then_refuses_update(self):
        self.holder()
        calls = []
        with patch.object(restart, "queue_pid", return_value=None), \
                patch.object(restart, "command", side_effect=lambda argv, repo: calls.append(argv) or str(repo)), \
                patch.object(subprocess, "Popen") as spawn, redirect_stdout(io.StringIO()), self.assertRaises(RuntimeError):
            restart.restart(self.repo, 0, self.home)
        self.assertEqual(calls, [["git", "rev-parse", "--show-toplevel"]])
        self.assertTrue((self.repo.parent / "queue.stop").exists())
        spawn.assert_not_called()

    def test_concurrent_restarts_serialize_separately(self):
        self.holder("restart")
        with patch.object(restart, "command") as mutate, self.assertRaises(RuntimeError):
            restart.restart(self.repo, 0, self.home)
        mutate.assert_not_called()
        # A restart waiting for the old queue must not consume its authority.
        with self.layout.acquire():
            pass

    def test_library_queue_without_authority_refuses_before_fetch(self):
        queue = mq.Queue(self.repo)
        with patch.object(mq, "git") as git, self.assertRaisesRegex(RuntimeError, "sole authority"):
            queue.cycle()
        git.assert_not_called()

    def test_legacy_recovery_refuses_marker_before_abort_switch_or_cleanup(self):
        queue = mq.Queue(self.repo, log_path=self.root / "audit.jsonl", writer_guard=lambda: None)
        (self.repo / ".git/precheck.unresolved").write_bytes(b"unresolved")
        with patch.object(mq, "git") as git, self.assertRaisesRegex(RuntimeError, "reconciliation"):
            queue.recover(RuntimeError("earlier failure"))
        git.assert_not_called()

    def test_worker_precheck_and_preparation_markers_block_legacy_before_queue(self):
        for name in ("mq-worker-unresolved.json", "precheck.unresolved", "mq-preparation-unresolved.json"):
            marker = self.repo / ".git" / name
            marker.write_bytes(b"even corrupt ownership blocks")
            with patch.object(sys, "argv", ["mq", "once", "--repo", str(self.repo),
                          "--authority-dir", str(self.layout.root)]), patch.object(mq, "Queue") as queue, \
                    redirect_stderr(io.StringIO()):
                self.assertEqual(mq.main(), 1)
                queue.assert_not_called()
            marker.unlink()

    def test_durable_train_state_blocks_alternate_clone_even_after_controller_exit(self):
        code = '''import importlib.util, pathlib, os, sys
spec=importlib.util.spec_from_file_location('a',sys.argv[1]); a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
l=a.Layout(pathlib.Path(sys.argv[2])); held=l.acquire()
a.durable.atomic_json(l.train/'state.json', {'version': 1, 'gates': {'other': 'draining'}})
os._exit(23)
'''
        child = subprocess.run([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "authority.py"),
                                str(self.layout.root)], stdin=subprocess.DEVNULL, capture_output=True)
        self.assertEqual(child.returncode, 23)
        with self.layout.acquire():
            with self.assertRaisesRegex(RuntimeError, "train recovery"):
                self.layout.legacy_guard(self.repo)
        with patch.object(restart, "queue_pid", return_value=None), \
                patch.object(restart, "command", return_value=str(self.repo)) as command, \
                patch.object(subprocess, "Popen") as spawn, redirect_stdout(io.StringIO()), \
                self.assertRaisesRegex(RuntimeError, "train recovery"):
            restart.restart(self.repo, 0, self.home)
        self.assertEqual(command.call_count, 1)
        spawn.assert_not_called()

    def test_actual_checkout_and_precheck_locks_are_respected(self):
        for path in (self.repo.with_name(self.repo.name + ".queue.lock"), self.repo / ".git/precheck.lock",
                     self.repo.with_name(self.repo.name + ".precheck.lock")):
            with authority.Lock(path):
                with self.assertRaises(RuntimeError):
                    with authority.checkout_lock(self.repo):
                        pass

    def test_inherited_authority_handoff_never_unlocks_live_child(self):
        ready_r, ready_w = os.pipe()
        release_r, release_w = os.pipe()
        lease = self.layout.acquire()
        code = '''import importlib.util, pathlib, os, sys
spec=importlib.util.spec_from_file_location('a',sys.argv[1]);a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
l=a.Layout(pathlib.Path(sys.argv[2]))
with l.acquire(inherited=int(sys.argv[3])) as held:
 os.close(int(sys.argv[3]));held.guard();os.write(int(sys.argv[4]),b'R');os.read(int(sys.argv[5]),1)
'''
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "authority.py"),
            str(self.layout.root), str(lease.fd), str(ready_w), str(release_r)],
            pass_fds=(lease.fd, ready_w, release_r), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        os.close(ready_w)
        os.close(release_r)
        try:
            self.assertEqual(os.read(ready_r, 1), b"R")
            lease.close()
            with self.assertRaises(RuntimeError):
                self.layout.acquire()
        finally:
            lease.close()
            os.close(ready_r)
            os.close(release_w)
            out, err = child.communicate(timeout=10)
        self.assertEqual(child.returncode, 0, (out, err))
        with self.layout.acquire():
            pass

    def test_restart_real_child_adopts_authority_before_acknowledging_start(self):
        queue_dir = self.repo / "tools/queue"
        queue_dir.mkdir(parents=True)
        release_r, release_w = os.pipe()
        script = f"""import argparse, importlib.util, os, pathlib
p=argparse.ArgumentParser();p.add_argument('action');p.add_argument('--repo');p.add_argument('--authority-dir');p.add_argument('--authority-fd',type=int);p.add_argument('--startup-fd',type=int);p.add_argument('--batch');v=p.parse_args()
s=importlib.util.spec_from_file_location('a', {str(DIRECTORY / 'authority.py')!r});a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
l=a.Layout(pathlib.Path(v.authority_dir))
with l.acquire(inherited=v.authority_fd) as held, a.checkout_lock(pathlib.Path(v.repo)):
 os.close(v.authority_fd);held.guard();l.legacy_guard(pathlib.Path(v.repo));os.write(v.startup_fd,b'READY\\n');os.close(v.startup_fd);os.read({release_r},1)
"""
        (queue_dir / "mq").write_text(script)
        popen = subprocess.Popen
        children = []
        def launch(argv, **kwargs):
            kwargs["pass_fds"] += (release_r,)
            child = popen(argv, **kwargs)
            children.append(child)
            return child
        def command(argv, repo, **ignored):
            return str(repo) if "--show-toplevel" in argv else "fixture-main"
        try:
            with patch.object(restart, "command", side_effect=command), \
                    patch.object(restart, "queue_pid", return_value=None), \
                    patch.object(subprocess, "Popen", side_effect=launch), redirect_stdout(io.StringIO()):
                restart.restart(self.repo, 0, self.home, batch=16)
            with self.assertRaises(RuntimeError):
                self.layout.acquire()
        finally:
            os.close(release_w)
            os.close(release_r)
            for child in children:
                child.wait(timeout=10)
        self.assertEqual(children[0].returncode, 0)
        with self.layout.acquire():
            pass

    def test_effect_child_keeps_authority_after_parent_hard_exit(self):
        ready_r, ready_w = os.pipe()
        release_r, release_w = os.pipe()
        code = """import importlib.util, os, pathlib, subprocess, sys
s=importlib.util.spec_from_file_location('a',sys.argv[1]);a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
l=a.Layout(pathlib.Path(sys.argv[2]));held=l.acquire()
def run(argv, **kwargs):
 child_code='import os, sys;os.write(int(sys.argv[1]), str(os.getpid()).encode()+bytes([10]));os.read(int(sys.argv[2]),1)'
 subprocess.Popen([sys.executable,'-I','-B','-c',child_code,sys.argv[4],sys.argv[5]],
  pass_fds=(*kwargs['pass_fds'],int(sys.argv[4]),int(sys.argv[5])),stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 os._exit(23)
effect=a.GitAuthority(l,held,pathlib.Path(sys.argv[3]),sys.argv[3],{'ops':'ops@fixture'},runner=run)
effect.main()
"""
        parent = subprocess.Popen([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "authority.py"),
            str(self.layout.root), str(self.repo), str(ready_w), str(release_r)],
            pass_fds=(ready_w, release_r), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        os.close(ready_w)
        os.close(release_r)
        pid_fd = None
        try:
            self.assertTrue(select.select([ready_r], [], [], 10)[0])
            pid = int(os.read(ready_r, 32))
            pid_fd = os.pidfd_open(pid)
            out, err = parent.communicate(timeout=10)
            self.assertEqual(parent.returncode, 23, (out, err))
            with self.assertRaises(RuntimeError):
                self.layout.acquire()
        finally:
            os.close(ready_r)
            os.close(release_w)
            if pid_fd is not None:
                self.assertTrue(select.select([pid_fd], [], [], 10)[0])
                os.close(pid_fd)
        with self.layout.acquire():
            pass

    def test_layout_refuses_symlinks_or_public_permissions(self):
        public = self.root / "public"
        public.mkdir(mode=0o755)
        public.chmod(0o755)
        with self.assertRaises(ValueError):
            authority.Layout(public)
        link = self.root / "alias"
        link.symlink_to(self.layout.root, target_is_directory=True)
        with self.assertRaises(ValueError):
            authority.Layout(link)


def suite():
    # Reuse the disposable Git world without rerunning its inherited suite.
    return unittest.TestSuite(cls(name) for cls in (
        EffectsTests, BisectionEffectsTests, NestedBisectionEffectsTests, GuardTests)
                              for name in cls.__dict__ if name.startswith("test_"))


if __name__ == "__main__":
    unittest.main(defaultTest="suite")
