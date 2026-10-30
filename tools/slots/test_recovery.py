"""Whole-target consequences with real files, processes, locks and pipe barriers."""

from contextlib import redirect_stdout
import io
import json
import os
from pathlib import Path
import runpy
import shutil
import signal
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
DIRECTORY = Path(__file__).resolve().parent
FIXTURE = runpy.run_path(str(DIRECTORY / "test_guard.py"))
guard = runpy.run_path(str(DIRECTORY / "guard"))
R = guard["RECOVERY"]
L = R["L"]


class RecoveryTests(unittest.TestCase):
    executable, cli, calls, state = (FIXTURE["GuardTests"].executable, FIXTURE["GuardTests"].cli,
                                               FIXTURE["GuardTests"].calls, FIXTURE["GuardTests"].state)

    def setUp(self):
        FIXTURE["GuardTests"].setUp(self)
        # cli() uses its fixture module's guard; both share standard library objects.
        self.runs_root = self.home / "run-root"
        self.runs = self.runs_root / "tmp/codex-run"
        self.runs.mkdir(parents=True)
        environment = patch.dict(os.environ, DEV_ROOT=str(self.runs_root))
        environment.start()
        self.addCleanup(environment.stop)
        host = patch.object(R["socket"], "gethostname", return_value="hetzner7")
        host.start()
        self.addCleanup(host.stop)
        self.proc = self.home / "proc"
        self.proc.mkdir()
        real_processes = R["processes"]
        for module in (R, FIXTURE["guard"].RECOVERY):
            mocked = patch.dict(module["snapshot"].__globals__,
                                processes=lambda slot, proc=None: real_processes(slot, self.proc))
            mocked.start()
            self.addCleanup(mocked.stop)

    def slot(self, number="1", *, completed=1_700_000_000, report="Done\n", ended=True):
        slot = self.slots / number
        (slot / "target").mkdir(parents=True, exist_ok=True)
        (slot / ".git/refs/heads").mkdir(parents=True, exist_ok=True)
        (slot / ".git/refs/heads/task").write_text("keep ref\n")
        (slot / ".git/HEAD").write_text("ref: refs/heads/task\n")
        (slot / "source").write_text("keep source\n")
        (slot / "target/cache").write_bytes(b"x" * 8192)
        self.add_run(slot, "run-" + number, completed, report, ended)
        return slot

    def add_run(self, slot, name, completed, report="Done\n", ended=True):
        run = self.runs / name
        run.mkdir()
        report_file = self.home / (name + "-report.md")
        report_file.write_text(report)
        report_file.chmod(0o600)
        (run / "meta").write_text(f"dir={slot}\nreport={report_file}\nstarted=2020-01-01T00:00:00Z\nto=lead/fixture\n")
        (run / "meta").chmod(0o600)
        if ended:
            (run / "ended").touch(mode=0o600)
            os.utime(run / "ended", ns=(completed * 10**9, completed * 10**9))
        return run

    def ack(self, slot):
        state = L["state_root"](self.home)
        with L["locked"](state / "sweep.lock"):
            return R["acknowledge"](slot.name, self.home, "lead/fixture", "report checked; capacity resume resolved")

    def recover(self, slots, *, dry=False, free=lambda: 60):
        details = []
        R["recover"](slots, self.home, 80, dry, free, details.append)
        return "\n".join(details)

    def barrier(self, slot, *, mode="run", cwd=False):
        release = self.home / ("release-" + slot.name)
        os.mkfifo(release, 0o600)
        fd = os.open(release, os.O_RDWR)
        self.addCleanup(os.close, fd)
        script = ("import os,sys; " + (f"os.chdir({str(slot)!r}); " if cwd else "") +
                  f"f=open({str(release)!r},'rb',buffering=0); print('ready',flush=True); f.read(1)")
        child = subprocess.Popen([sys.executable, "-I", "-B", str(DIRECTORY / "lifetime.py"), mode,
                                  str(slot), "--", sys.executable, "-I", "-B", "-c", script],
                                 stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        def cleanup():
            if child.poll() is None:
                os.write(fd, b"x")
            child.communicate(timeout=10)
        self.addCleanup(cleanup)
        self.assertEqual(child.stdout.readline(), b"ready\n")
        return child, fd

    def test_actual_builder_hostnames_can_acknowledge(self):
        slot = self.slot()
        for number in (3, 6, 7):
            with self.subTest(number=number), patch.object(
                    R["socket"], "gethostname", return_value=f"debian-16gb-hel1-{number}"):
                self.assertIn("acknowledged", self.ack(slot))
                state = L["state_root"](self.home) / slot.name / "ack.json"
                self.assertEqual(L["read"](state)["host"], f"hetzner{number}")

    def test_actual_gate_hostnames_cannot_acknowledge(self):
        slot = self.slot()
        for number in (2, 8):
            with self.subTest(number=number), patch.object(
                    R["socket"], "gethostname", return_value=f"debian-16gb-hel1-{number}"):
                with self.assertRaisesRegex(RuntimeError, "not a builder"):
                    self.ack(slot)
                self.assertTrue((slot / "target/cache").exists())

    def test_report_and_post_delivery_never_grant_eligibility(self):
        slot = self.slot()
        (self.runs / "run-1/post-output").write_text("delivered\n")
        result = self.recover([slot])
        self.assertIn("ack.json", result)
        self.assertTrue((slot / "target/cache").exists())

    def test_legacy_alias_retry_outside_slot_vetoes_recovery(self):
        slot = self.slot()
        self.ack(slot)
        alias = self.slots / ".." / "slots" / "1"
        self.add_run(alias, "retry", 1_700_000_010, ended=False)
        self.state(units={"codex-run-retry.service": {"ActiveState": "active", "WorkingDirectory": ""}})
        self.assertNotIn(": reclaimed ", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def launch_directory(self, directory, report):
        task = self.home / "task.md"
        task.write_text("fixture\n")
        return subprocess.run(["bash", str(DIRECTORY.parent / "codex-run/codex-run"), "task",
                               "--to", "lead/fixture", str(task), str(directory), str(report)],
                              text=True, capture_output=True, stdin=subprocess.DEVNULL)

    def test_new_alias_launch_invalidates_ack_and_records_canonical_directory(self):
        slot = self.slot()
        self.ack(slot)
        alias = self.slots / ".." / "slots" / "1"
        child = self.launch_directory(alias, self.home / "alias-report.md")
        self.assertEqual(child.returncode, 0, child.stderr)
        self.assertFalse((self.home / "src/fabrig/slot-maintenance/1/ack.json").exists())
        _, state = child.stdout.strip().split(" ", 1)
        meta = dict(line.split("=", 1) for line in (Path(state) / "meta").read_text().splitlines())
        self.assertEqual(meta["dir"], str(slot))
        self.assertIn("durable uncertain/live lifetime", self.recover([slot]))

    def move_report(self, run_name, destination):
        meta = self.runs / run_name / "meta"
        old = self.home / (run_name + "-report.md")
        old.rename(destination)
        meta.write_text(meta.read_text().replace(f"report={old}\n", f"report={destination}\n"))

    def test_own_report_inside_target_cannot_be_acknowledged_or_deleted(self):
        slot = self.slot()
        report = slot / "target/report.md"
        self.move_report("run-1", report)
        with self.assertRaisesRegex(RuntimeError, "protected.*target"):
            self.ack(slot)
        self.assertTrue(report.exists())

    def test_other_slot_report_inside_candidate_target_vetoes_recovery(self):
        slot, other = self.slot("1"), self.slot("2")
        self.ack(slot)
        report = slot / "target/other-report.md"
        self.move_report("run-2", report)
        self.assertNotIn(": reclaimed ", self.recover([slot]))
        self.assertTrue(report.exists())

    def test_history_root_inside_target_cannot_be_acknowledged(self):
        slot = self.slot()
        destination = slot / "target/run-root"
        self.runs_root.rename(destination)
        with patch.dict(os.environ, DEV_ROOT=str(destination)):
            with self.assertRaisesRegex(RuntimeError, "protected.*target"):
                self.ack(slot)
        self.assertTrue((destination / "tmp/codex-run/run-1/meta").exists())

    def test_recent_manual_completion_moves_old_codex_slot_after_idle_slot(self):
        recent = self.slot("1", completed=1_700_000_000)
        idle = self.slot("2", completed=1_700_000_010)
        with L["Lease"](recent, self.home) as lease:
            lease.clean = True
        for slot in (recent, idle):
            self.ack(slot)
        result = self.recover([recent, idle], free=lambda: 60 if (idle / "target").exists() else 81)
        self.assertIn("slot 2: reclaimed", result)
        self.assertNotIn("slot 1: reclaimed", result)
        self.assertTrue((recent / "target/cache").exists())

    def test_alias_service_and_finished_legacy_run_still_veto_recovery(self):
        slot = self.slot()
        alias = self.slots / ".." / "slots" / "1"
        self.add_run(alias, "retry", 1_700_000_010)
        self.ack(slot)
        self.state(units={"codex-run-retry.service": {"ActiveState": "active", "WorkingDirectory": ""}})
        self.assertIn("live Codex run: retry", self.recover([slot]))
        self.state(units={"job-heavy-wrapper.service": {"ActiveState": "active", "WorkingDirectory": str(alias)}})
        self.assertIn("live service", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_symlink_alias_cancelled_by_dot_dot_is_refused_before_launch(self):
        slot = self.slot()
        self.ack(slot)
        link = self.home / "alias"
        link.symlink_to(self.slots)
        child = self.launch_directory(link / ".." / "slots" / "1", self.home / "alias-report.md")
        self.assertNotEqual(child.returncode, 0)
        self.assertIn("symlink path", child.stderr)
        self.assertFalse((self.home / "alias-report.md").exists())
        self.assertTrue((self.home / "src/fabrig/slot-maintenance/1/ack.json").exists())

    def test_legacy_symlink_alias_and_other_report_alias_preserve_evidence(self):
        slot, other = self.slot("1"), self.slot("2")
        self.ack(slot)
        link = self.home / "alias"
        link.symlink_to(self.slots)
        meta = self.runs / "run-2/meta"
        original = meta.read_text()
        meta.write_text(original.replace(f"dir={other}\n", f"dir={link}/../slots/2\n"))
        self.assertIn("symlink path", self.recover([slot]))
        meta.write_text(original)
        report = slot / "target/other-report.md"
        self.move_report("run-2", report)
        report_link = self.home / "report-alias"
        report_link.symlink_to(slot / "target")
        meta.write_text(meta.read_text().replace(f"report={report}\n", f"report={report_link}/other-report.md\n"))
        self.assertIn("symlink path", self.recover([slot]))
        self.assertTrue(report.exists())

    def test_existing_ack_cannot_delete_its_report_moved_under_target_alias(self):
        slot = self.slot()
        self.ack(slot)
        report = slot / "target/report.md"
        self.move_report("run-1", report)
        meta = self.runs / "run-1/meta"
        alias = slot / "target/.." / "target/report.md"
        meta.write_text(meta.read_text().replace(f"report={report}\n", f"report={alias}\n"))
        self.assertIn("protected evidence overlaps target", self.recover([slot]))
        self.assertTrue(report.exists())

    def test_history_alias_and_symlink_inside_target_preserve_all_history(self):
        slot = self.slot()
        self.ack(slot)
        destination = slot / "target/run-root"
        self.runs_root.rename(destination)
        alias = slot / "target/.." / "target/run-root"
        with patch.dict(os.environ, DEV_ROOT=str(alias)):
            self.assertIn("protected evidence overlaps target", self.recover([slot]))
        link = self.home / "history-link"
        link.symlink_to(destination)
        with patch.dict(os.environ, DEV_ROOT=str(link)):
            self.assertIn("symlink path", self.recover([slot]))
        self.assertTrue((destination / "tmp/codex-run/run-1/meta").exists())

    def test_launcher_refuses_report_and_history_outputs_inside_targets(self):
        slot = self.slot()
        report = slot / "target/new-report.md"
        child = self.launch_directory(slot, report)
        self.assertNotEqual(child.returncode, 0)
        self.assertIn("protected evidence overlaps target", child.stderr)
        self.assertFalse(report.exists())
        with patch.dict(os.environ, DEV_ROOT=str(slot / "target/run-root")):
            child = self.launch_directory(slot, self.home / "new-report.md")
        self.assertNotEqual(child.returncode, 0)
        self.assertFalse((slot / "target/run-root").exists())

    def test_completed_lifetime_digest_and_identity_are_bound_to_acknowledgement(self):
        slot = self.slot()
        with L["Lease"](slot, self.home) as lease:
            lease.clean = True
        self.ack(slot)
        completion = self.home / "src/fabrig/slot-maintenance/1/completed.json"
        record = L["read"](completion)
        L["write"](completion, {**record, "purpose": "changed completion evidence"})
        self.assertIn("acknowledgement stale", self.recover([slot]))
        L["write"](completion, {**record, "identity": [0, 0]})
        self.assertIn("unknown completed lifetime", self.recover([slot]))

    def test_new_lifetime_does_not_overwrite_unknown_completion_evidence(self):
        slot = self.slot()
        with L["Lease"](slot, self.home) as lease:
            lease.clean = True
        completion = self.home / "src/fabrig/slot-maintenance/1/completed.json"
        L["write"](completion, {**L["read"](completion), "identity": [0, 0]})
        original = completion.read_bytes()
        with self.assertRaisesRegex(RuntimeError, "unknown completed lifetime"):
            with L["Lease"](slot, self.home) as lease:
                lease.clean = True
        self.assertEqual(completion.read_bytes(), original)
        self.assertEqual(len(list(completion.parent.glob("*.hold"))), 1)
        self.assertTrue((slot / "target/cache").exists())

    def test_completed_precheck_refreshes_idle_order_after_restoration(self):
        recent = self.slot("1", completed=1_700_000_000)
        idle = self.slot("2", completed=1_700_000_010)
        shutil.rmtree(recent / ".git")
        (recent / ".gitignore").write_text("target/\n")
        env = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1")
        for args in (("init", "-b", "original"), ("config", "user.name", "Fixture"),
                     ("config", "user.email", "fixture@example.invalid"), ("add", "."),
                     ("commit", "-m", "fixture")):
            subprocess.run(["git", "-C", str(recent), *args], env=env, check=True,
                           stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        child = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY.parent / "queue/precheck"),
                                "--base", "HEAD", str(recent), "HEAD"], env=env,
                               text=True, capture_output=True, stdin=subprocess.DEVNULL)
        self.assertEqual(child.returncode, 0, child.stdout + child.stderr)
        for slot in (recent, idle):
            self.ack(slot)
        result = self.recover([recent, idle], free=lambda: 60 if (idle / "target").exists() else 81)
        self.assertIn("slot 2: reclaimed", result)
        self.assertNotIn("slot 1: reclaimed", result)
        self.assertTrue((recent / "target/cache").exists())

    def test_killed_guard_fixture_leaves_scratch_outside_checked_source(self):
        root = FIXTURE["ROOT"]
        before = {path.name for path in root.iterdir()}
        code = ("import runpy,sys; C=runpy.run_path(sys.argv[1])['GuardTests']; "
                "t=C('test_above_threshold_does_not_inspect_units_or_trim'); t.setUp(); "
                "print(t.home,flush=True); sys.stdin.read(1)")
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", code, str(DIRECTORY / "test_guard.py")],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            scratch = Path(child.stdout.readline().decode().strip())
            self.assertTrue(scratch.is_absolute())
            self.assertEqual(scratch.parent, root.parent)
            self.assertTrue(scratch.name.startswith(".guard-test-"))
            self.addCleanup(lambda: shutil.rmtree(scratch) if scratch.is_dir() else None)
        finally:
            if child.poll() is None:
                child.kill()
            child.communicate(timeout=10)
        self.assertEqual(child.returncode, -signal.SIGKILL)
        self.assertTrue(scratch.is_dir())
        self.assertEqual(scratch.parent, root.parent)
        self.assertFalse(scratch.is_relative_to(root))
        self.assertEqual({path.name for path in root.iterdir()}, before)

    def test_explicit_ack_handles_nonzero_done_and_capacity_report(self):
        for number, report in (("1", "Done\nexit 1: capacity; pending resume\n"), ("2", "Done\nexit 7\n")):
            slot = self.slot(number, report=report)
            self.assertNotIn("reclaimed ", self.recover([slot]))
            self.ack(slot)
            self.assertIn("reclaimed ", self.recover([slot]))
            self.assertEqual((slot / "source").read_text(), "keep source\n")
            self.assertEqual((slot / ".git/refs/heads/task").read_text(), "keep ref\n")
            self.assertTrue((self.home / ("run-" + number + "-report.md")).exists())

    def test_new_run_and_new_report_invalidate_ack(self):
        for number, mutation in (("1", "run"), ("2", "report"), ("3", "ended")):
            slot = self.slot(number)
            self.ack(slot)
            if mutation == "run":
                self.add_run(slot, "new", 1_700_000_010, ended=False)
            elif mutation == "report":
                (self.home / "run-2-report.md").write_text("Done, changed\n")
            else:
                os.utime(self.runs / "run-3/ended", None)
            self.assertNotIn("reclaimed ", self.recover([slot]))
            self.assertTrue((slot / "target/cache").exists())

    def test_oldest_completion_first_stops_immediately_above_threshold(self):
        recent = self.slot("1", completed=1_700_000_020)
        oldest = self.slot("2", completed=1_700_000_000)
        middle = self.slot("3", completed=1_700_000_010)
        for slot in (recent, oldest, middle):
            self.ack(slot)
        free = lambda: 60 if (oldest / "target").exists() else 81
        result = self.recover([recent, middle, oldest], free=free)
        self.assertIn("slot 2: reclaimed", result)
        self.assertNotIn("slot 3: reclaimed", result)
        self.assertTrue((recent / "target").exists())
        self.assertTrue((middle / "target").exists())

    def test_exact_threshold_continues_and_stable_tie_break_is_numeric_text(self):
        first, second = self.slot("1"), self.slot("2")
        for slot in (second, first):
            self.ack(slot)
        result = self.recover([second, first], free=lambda: 80)
        self.assertLess(result.index("slot 1: reclaimed"), result.index("slot 2: reclaimed"))

    def test_dry_run_checks_every_eligible_target_without_promised_bytes(self):
        slots = [self.slot("1"), self.slot("2")]
        for slot in slots:
            self.ack(slot)
        result = self.recover(slots, dry=True)
        self.assertEqual(result.count("would-reclaim"), 2)
        self.assertNotIn("reclaim-start", result)
        self.assertTrue(all((slot / "target/cache").exists() for slot in slots))

    def test_remeasures_space_instead_of_summing_target_size(self):
        slots = [self.slot("1"), self.slot("2")]
        for slot in slots:
            self.ack(slot)
        self.assertEqual(self.recover(slots).count(": reclaimed "), 2)

    def test_slot_target_and_report_replacements_invalidate(self):
        for number, path in (("1", "target"), ("2", "slot"), ("3", "report")):
            slot = self.slot(number)
            self.ack(slot)
            if path == "target":
                (slot / "target").rename(slot / "saved-target")
                (slot / "target").mkdir()
            elif path == "slot":
                slot.rename(slot.with_name("saved-slot"))
                slot.mkdir()
                (slot / ".git").mkdir()
                (slot / "target").mkdir()
            else:
                report = self.home / "run-3-report.md"
                report.rename(report.with_suffix(".saved"))
                report.symlink_to(report.with_suffix(".saved"))
            self.assertNotIn(": reclaimed ", self.recover([slot]))
            self.assertTrue((slot / "target").exists())

    def test_symlink_ack_and_private_state_refuse(self):
        slot = self.slot()
        self.ack(slot)
        directory = self.home / "src/fabrig/slot-maintenance/1"
        (directory / "ack.json").chmod(0o644)
        self.assertIn("0600", self.recover([slot]))
        saved = self.home / "saved-ack.json"
        (directory / "ack.json").rename(saved)
        (directory / "ack.json").symlink_to(saved)
        self.assertIn("symlink", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_real_lifetime_lock_blocks_reclaim_and_invalidates_ack(self):
        slot = self.slot()
        self.ack(slot)
        child, release = self.barrier(slot)
        self.assertIn("lease busy", self.recover([slot]))
        os.write(release, b"x")
        self.assertEqual(child.wait(timeout=10), 0)
        self.assertIn("ack.json", self.recover([slot]))
        self.ack(slot)
        self.assertIn(": reclaimed ", self.recover([slot]))

    def test_inherited_lock_survives_supervisor_crash_then_hold_remains(self):
        slot = self.slot()
        child, release = self.barrier(slot)
        child.kill()
        child.wait(timeout=10)
        self.assertIn("lease busy", self.recover([slot]))
        os.write(release, b"x")
        # EOF is the deterministic child completion barrier; no timing sleep.
        child.communicate(timeout=10)
        self.assertIn("durable uncertain/live lifetime", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_launch_hold_bridges_detached_startup(self):
        slot = self.slot()
        child, release = self.barrier(slot, mode="launch")
        os.write(release, b"x")
        self.assertEqual(child.wait(timeout=10), 0)
        self.assertIn("durable uncertain/live lifetime", self.recover([slot]))

    def test_real_cwd_process_blocks_even_without_a_lease(self):
        slot = self.slot()
        self.ack(slot)
        code = "import os,sys; print('ready',flush=True); sys.stdin.read(1)"
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", code], cwd=slot,
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.addCleanup(lambda: child.communicate(b"x", timeout=10))
        self.assertEqual(child.stdout.readline(), b"ready\n")
        (self.proc / str(child.pid)).symlink_to(Path("/proc") / str(child.pid))
        self.assertIn("own-user process", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_legacy_codex_retry_and_queued_heavy_wrapper_veto(self):
        slot = self.slot()
        self.ack(slot)
        for name, props in (("codex-run-run-1.service", {"WorkingDirectory": ""}),
                            ("job-heavy-queued-wrapper.service", {"WorkingDirectory": str(slot)})):
            self.state(units={name: {"ActiveState": "active", **props}})
            self.assertNotIn(": reclaimed ", self.recover([slot]))
            self.assertTrue((slot / "target/cache").exists())

    def test_actual_worker_and_precheck_unresolved_markers_refuse(self):
        slot = self.slot()
        self.ack(slot)
        for name in ("mq-worker-unresolved.json", "precheck.unresolved"):
            marker = slot / ".git" / name
            marker.symlink_to(slot / "missing")
            self.assertIn("symlink", self.recover([slot]))
            marker.unlink()

    def test_interrupted_deletion_is_never_retried(self):
        slot = self.slot()
        self.ack(slot)
        with patch.object(shutil, "rmtree", side_effect=OSError("interrupted")) as remove:
            remove.avoids_symlink_attacks = True
            self.assertIn("interrupted", self.recover([slot]))
        self.assertIn("interrupted reclaim", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_gate_hosts_cannot_ack_or_reclaim(self):
        slot = self.slot()
        for host in ("hetzner2", "hetzner8"):
            with patch.object(R["socket"], "gethostname", return_value=host):
                with self.assertRaisesRegex(RuntimeError, "not a builder"):
                    self.ack(slot)
                self.assertIn("not a builder", self.recover([slot]))

    def test_executable_first_then_whole_target_recovery(self):
        slot = self.slot()
        executable = self.executable("1")
        self.ack(slot)
        free = lambda: 60 if executable.exists() else 81
        code, output = self.cli(free=free)
        self.assertEqual(code, 0)
        self.assertTrue((slot / "target/cache").exists())
        self.assertNotIn("reclaimed ", output)
        self.assertIn("trimmed 1 executables", output)

    def test_exhausted_trim_falls_back_and_preserves_claims_and_report(self):
        slot = self.slot()
        metadata = {str(p.relative_to(slot)): p.read_bytes() for p in (slot / ".git").rglob("*") if p.is_file()}
        self.ack(slot)
        claims = self.home / "src/fabrig/claims"
        claims.mkdir()
        (claims / "fixture").write_text("owner\n")
        free = lambda: 60 if (slot / "target").exists() else 81
        code, output = self.cli(free=free)
        self.assertEqual(code, 0)
        self.assertIn(": reclaimed ", output)
        self.assertEqual((claims / "fixture").read_text(), "owner\n")
        self.assertEqual((self.home / "run-1-report.md").read_text(), "Done\n")
        self.assertEqual((slot / "source").read_text(), "keep source\n")
        self.assertEqual({str(p.relative_to(slot)): p.read_bytes() for p in (slot / ".git").rglob("*") if p.is_file()}, metadata)

    def test_exclusive_maintenance_blocks_real_launcher_before_report_reservation(self):
        slot = self.slot()
        task, report = self.home / "task.md", self.home / "new-report.md"
        task.write_text("fixture\n")
        with R["maintenance"](slot, self.home):
            child = subprocess.run(["bash", str(DIRECTORY.parent / "codex-run/codex-run"), "task",
                                    "--to", "lead/fixture", str(task), str(slot), str(report)],
                                   text=True, capture_output=True, stdin=subprocess.DEVNULL)
        self.assertNotEqual(child.returncode, 0)
        self.assertIn("lease busy", child.stderr)
        self.assertFalse(report.exists())
        self.assertEqual(sorted(p.name for p in self.runs.iterdir()), ["run-1"])

    def test_explicit_hold_reconciliation_requires_quiescence_and_fresh_ack(self):
        slot = self.slot()
        child, release = self.barrier(slot, mode="launch")
        os.write(release, b"x")
        child.wait(timeout=10)
        state = self.home / "src/fabrig/slot-maintenance/1"
        token = next(state.glob("*.hold")).stem
        self.state(units={"job-heavy-wrapper.service": {"WorkingDirectory": str(slot), "ActiveState": "active"}})
        with self.assertRaisesRegex(RuntimeError, "live service"):
            R["release_hold"]("1", token, self.home, "ops", "known completed")
        self.state()
        R["release_hold"]("1", token, self.home, "ops", "known completed")
        self.assertIn("ack.json", self.recover([slot]))
        self.ack(slot)
        self.assertIn(": reclaimed ", self.recover([slot]))

    def test_unknown_process_observation_and_unavailable_systemd_preserve_target(self):
        slot = self.slot()
        self.ack(slot)
        proc = self.proc / "100"
        proc.mkdir()
        (proc / "comm").write_text("python3\n")
        (proc / "cgroup").write_text("0::/user.slice/user-1000.slice/session-123.scope\n")
        real_readlink = os.readlink
        def unreadable(path, *args, **kwargs):
            if Path(path) == proc / "cwd":
                raise PermissionError("fixture unknown cwd")
            return real_readlink(path, *args, **kwargs)
        with patch.object(os, "readlink", side_effect=unreadable):
            self.assertIn("unreadable own-user cwd", self.recover([slot]))
        shutil.rmtree(proc)
        self.state(fail=True)
        self.assertIn("fixture systemd unavailable", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_timer_reconciler_and_vanished_units_are_classified_defensively(self):
        slot = self.slot()
        self.ack(slot)
        self.state(units={
            "job-heavy-reconcile.timer": {"ActiveState": "active"},
            "job-heavy-maintenance-reconcile.service": {"ActiveState": "active"},
            "job-heavy-gone.service": {"LoadState": "not-found", "ActiveState": "inactive"}})
        self.assertIn(": reclaimed ", self.recover([slot]))

    def test_missing_active_service_properties_refuse_whole_target(self):
        slot = self.slot()
        self.ack(slot)
        self.state(units={"job-heavy-wrapper.service": {"ActiveState": "active"}})
        self.assertIn("missing heavy unit properties", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_replacement_at_mutation_boundary_cannot_redirect_deletion(self):
        slot = self.slot()
        self.ack(slot)
        outside = self.home / "keep-target"
        outside.mkdir()
        (outside / "source").write_text("keep\n")
        def record(line):
            if ": reclaim-start " in line:
                (slot / "target").rename(slot / "saved-target")
                (slot / "target").symlink_to(outside)
        R["recover"]([slot], self.home, 80, False, lambda: 60, record)
        self.assertEqual((outside / "source").read_text(), "keep\n")
        self.assertTrue((slot / "saved-target/cache").exists())

    def test_real_filesystem_free_space_is_observed_after_deletion(self):
        slot = self.slot()
        self.ack(slot)
        # An external hard link keeps these blocks allocated after target deletion.
        os.link(slot / "target/cache", self.home / "keep-hardlink")
        samples = []
        def free():
            value = shutil.disk_usage(self.slots).free / 1e9
            samples.append(value)
            return value
        details = []
        R["recover"]([slot], self.home, float("inf"), False, free, details.append)
        self.assertFalse((slot / "target").exists())
        self.assertEqual((self.home / "keep-hardlink").stat().st_size, 8192)
        self.assertGreaterEqual(len(samples), 3)
        self.assertIn(f"after={samples[-1]:.3f}", details[-1])

    def test_ack_cli_requires_all_assertions_and_exposes_addressed_notice(self):
        slot = self.slot()
        code, _ = self.cli("--ack", "1", "--by", "lead/fixture", "--reason", "checked",
                           "--report-handled", "--no-pending-resume", "--exclusive-lease-use")
        self.assertEqual(code, 0)
        self.assertIn(": reclaimed ", self.recover([slot]))
        code, output = self.cli("--notices")
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(output)["owner"], "lead/fixture")
        with redirect_stdout(io.StringIO()), patch.object(sys, "stderr", io.StringIO()):
            with self.assertRaises(SystemExit) as rejected:
                self.cli("--ack", "1", "--by", "ops", "--reason", "Done")
        self.assertEqual(rejected.exception.code, 2)

    def test_nested_launcher_records_a_distinct_durable_launch_hold(self):
        slot = self.slot()
        task, report = self.home / "task.md", self.home / "nested-report.md"
        task.write_text("fixture\n")
        with L["Lease"](slot, self.home) as lease:
            env = dict(os.environ, FABRIG_SLOT_LEASE_FD=str(lease.fd), FABRIG_SLOT_HOLD=lease.token,
                       FABRIG_SLOT_LEASE_MODE="run")
            child = subprocess.run(["bash", str(DIRECTORY.parent / "codex-run/codex-run"), "task",
                                    "--to", "lead/fixture", str(task), str(slot), str(report)], env=env,
                                   pass_fds=(lease.fd,), text=True, capture_output=True, stdin=subprocess.DEVNULL)
            self.assertEqual(child.returncode, 0, child.stderr)
            _, run = child.stdout.strip().split(" ", 1)
            meta = dict(line.split("=", 1) for line in (Path(run) / "meta").read_text().splitlines())
            self.assertNotEqual(meta["slot_hold"], lease.token)
            self.assertTrue((lease.directory / (meta["slot_hold"] + ".hold")).exists())
            lease.clean = True

    def test_corrupt_and_unknown_maintenance_state_fail_closed(self):
        slot = self.slot()
        self.ack(slot)
        state = self.home / "src/fabrig/slot-maintenance/1"
        ack = state / "ack.json"
        original = ack.read_text()
        ack.write_text("[]\n")
        self.assertIn("JSON object", self.recover([slot]))
        ack.write_text(original)
        (state / "unknown").touch()
        self.assertIn("unknown maintenance state", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_internal_symlink_is_unlinked_without_traversing_its_source(self):
        slot = self.slot()
        outside = self.home / "keep-source"
        outside.mkdir()
        (outside / "keep").write_text("source\n")
        (slot / "target/link").symlink_to(outside)
        self.ack(slot)
        self.assertIn(": reclaimed ", self.recover([slot]))
        self.assertEqual((outside / "keep").read_text(), "source\n")

    def test_mount_beneath_target_refuses_before_generated_files_are_deleted(self):
        slot = self.slot()
        self.ack(slot)
        real_read = Path.read_text
        def mountinfo(path, *args, **kwargs):
            if path == Path("/proc/self/mountinfo"):
                return f"1 0 0:1 / {slot}/target/cache rw - ext4 /dev/fixture rw\n"
            return real_read(path, *args, **kwargs)
        with patch.object(Path, "read_text", mountinfo):
            self.assertIn("mount beneath target", self.recover([slot]))
        self.assertTrue((slot / "target/cache").exists())

    def test_independent_sweep_lock_excludes_acknowledgement_and_ordinary_trim(self):
        slot = self.slot()
        executable = self.executable("1")
        state = L["state_root"](self.home)
        code = ("import fcntl,sys; f=open(sys.argv[1],'a+'); "
                "fcntl.flock(f,fcntl.LOCK_EX); print('ready',flush=True); sys.stdin.read(1)")
        child = subprocess.Popen([sys.executable, "-I", "-B", "-c", code, str(state / "sweep.lock")],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.addCleanup(lambda: child.communicate(b"x", timeout=10))
        self.assertEqual(child.stdout.readline(), b"ready\n")
        result, output = self.cli()
        self.assertEqual(result, 1)
        self.assertIn("lease busy", output)
        with patch.object(sys, "stderr", io.StringIO()):
            result, _ = self.cli("--ack", "1", "--by", "ops", "--reason", "checked",
                                 "--report-handled", "--no-pending-resume", "--exclusive-lease-use")
        self.assertEqual(result, 1)
        self.assertTrue(executable.exists())


class AutomaticObservationTests(unittest.TestCase):
    setUp = RecoveryTests.setUp
    slot, add_run, ack = RecoveryTests.slot, RecoveryTests.add_run, RecoveryTests.ack
    state, cli, calls = RecoveryTests.state, RecoveryTests.cli, RecoveryTests.calls

    def observe(self, slot, *, available=60_000_000_000, now=None):
        details = []
        # Advance only the fixture clock; file change times remain kernel metadata.
        now = now or R["time"].time_ns() + 2 * 86400 * 10**9
        with patch.object(R["time"], "time_ns", return_value=now):
            R["observe_targets"]([slot], self.home, available, details.append)
        return "\n".join(details)

    def test_old_metadata_is_observed_without_granting_eligibility(self):
        slot = self.slot()
        output = self.observe(slot)
        self.assertIn("observed-idle-over24h=YES", output)
        for field in ("claim", "claim-age-seconds", "central-exclusion", "terminal-no-resume",
                      "live-process", "source", "borrower"):
            self.assertIn(field + "=UNKNOWN", output)
        self.assertIn("deletion=PROTECTED", output)
        self.assertFalse((self.home / "src/fabrig/slot-maintenance").exists())
        self.assertTrue((slot / "target/cache").exists())

    def test_idle_boundary_includes_every_target_entry_change_time(self):
        slot = self.slot()
        cache = slot / "target/cache"
        # An old root directory and old mtime do not hide a newer file ctime.
        os.utime(slot / "target", ns=(1, 1))
        os.utime(cache, ns=(1, 1))
        newest = max(max(path.stat().st_mtime_ns, path.stat().st_ctime_ns)
                     for path in (cache, slot / "target"))
        day = 86400 * 10**9
        for age, expected in ((day - 1, "NO"), (day, "NO"), (day + 1, "YES")):
            with self.subTest(age=age):
                self.assertIn("observed-idle-over24h=" + expected,
                              self.observe(slot, now=newest + age))

    def test_exact_decimal_disk_boundary_never_changes_deletion_status(self):
        slot = self.slot()
        for available, expected in ((79_999_999_999, "YES"), (80_000_000_000, "NO"),
                                    (80_000_000_001, "NO")):
            with self.subTest(available=available):
                output = self.observe(slot, available=available)
                self.assertIn("pressure-below80GB=" + expected, output)
                self.assertIn("deletion=PROTECTED", output)

    def test_missing_or_unfinished_history_is_unknown(self):
        slot = self.slot(ended=False)
        self.assertIn("completion-age-seconds=UNKNOWN", self.observe(slot))
        shutil.rmtree(self.runs)
        self.assertIn("no trustworthy Codex run history", self.observe(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_completed_lifetime_ended_and_question_reports_never_prove_finality(self):
        for number, report in (("1", "Done\n"), ("2", "Question: resume after capacity?\n"),
                               ("3", "codex-run: the run wrote no report\n")):
            with self.subTest(report=report):
                slot = self.slot(number, report=report)
                with L["Lease"](slot, self.home) as lease:
                    lease.clean = True
                output = self.observe(slot)
                self.assertIn("completion-age-seconds=", output)
                self.assertIn("terminal-no-resume=UNKNOWN", output)
                self.assertIn("deletion=PROTECTED", output)

    def test_unknown_or_changed_claim_records_and_ack_cannot_authorize_observation(self):
        slot = self.slot()
        self.ack(slot)
        calls_before = self.calls()
        claims = self.home / "src/fabrig/claims"
        claims.mkdir()
        claim = claims / "hetzner7-1"
        for content in (None, 'invalid record\n', '{"owner":"new owner"}\n'):
            with self.subTest(content=content):
                if content is not None:
                    claim.write_text(content)
                code, output = self.cli("--automatic-targets")
                self.assertEqual(code, 2)
                self.assertIn("claim=UNKNOWN", output)
                self.assertIn("claim snapshots grant no deletion exclusion", output)
                self.assertTrue((slot / "target/cache").exists())
                self.assertTrue((self.home / "src/fabrig/slot-maintenance/1/ack.json").exists())
                if content is not None:
                    self.assertEqual(claim.read_text(), content)
        self.assertEqual(self.calls(), calls_before)

    def test_automatic_live_refuses_before_reading_or_mutating_files(self):
        slot = self.slot()
        self.ack(slot)
        paths = [path for path in self.home.rglob("*") if path.is_file()]
        before = {path: path.read_bytes() for path in paths}
        with patch.object(sys, "stderr", io.StringIO()) as error, \
                patch.dict(FIXTURE["guard"].main.__globals__,
                           sweep=unittest.mock.Mock(side_effect=AssertionError("destructive sweep")),
                           automatic_observation=unittest.mock.Mock(side_effect=AssertionError("observation"))):
            for args in (("--automatic-targets", "live"), ("--automatic-targets", "live", "--dry-run")):
                code, _ = self.cli(*args)
                self.assertEqual(code, 1)
                self.assertIn("central exclusion and finality protocol unavailable", error.getvalue())
        self.assertEqual({path: path.read_bytes() for path in paths}, before)
        self.assertFalse(self.log.exists())

    def test_automatic_observation_isolated_from_manual_recovery_and_trim(self):
        slot = self.slot()
        self.ack(slot)
        calls_before = self.calls()
        binary = FIXTURE["GuardTests"].executable(self, "1")
        protected = [slot / "source", slot / ".git/HEAD", slot / ".git/refs/heads/task",
                     self.home / "run-1-report.md", slot / "target/cache", binary]
        before = {path: path.read_bytes() for path in protected}
        with patch.dict(FIXTURE["guard"].main.__globals__,
                        sweep=unittest.mock.Mock(side_effect=AssertionError("destructive sweep"))):
            code, output = self.cli("--automatic-targets", "observe", "--dry-run")
        self.assertEqual(code, 2)
        self.assertIn("automatic-target PROTECTED", output)
        self.assertEqual({path: path.read_bytes() for path in protected}, before)
        self.assertEqual(self.calls(), calls_before)
        self.assertFalse((self.home / "src/fabrig/slot-maintenance/1/reclaim.json").exists())

    def test_symlink_target_and_unrecognized_builder_stay_protected(self):
        slot = self.slot()
        (slot / "target").rename(slot / "saved-target")
        (slot / "target").symlink_to(slot / "saved-target")
        self.assertIn("target-write-age-seconds=UNKNOWN", self.observe(slot))
        for host in ("hetzner2", "hetzner8", "unknown"):
            with patch.object(R["socket"], "gethostname", return_value=host):
                self.assertIn("not a builder", self.observe(slot))
        self.assertTrue((slot / "saved-target/cache").exists())

    def test_automatic_actions_cannot_mix_with_manual_or_timer_options(self):
        for option in (("--timer",), ("--stop",), ("--ack", "1"), ("--notices",),
                       ("--release-hold", "1"), ("--below", "100")):
            with self.subTest(option=option), patch.object(sys, "stderr", io.StringIO()):
                with self.assertRaises(SystemExit) as rejected:
                    self.cli("--automatic-targets", "observe", *option)
                self.assertEqual(rejected.exception.code, 2)
        self.assertEqual(self.calls(), [])


FAKE_CENTRAL_SSH = '''#!/usr/bin/python3
import json, os, runpy, shlex, socket, sys
from pathlib import Path
assert sys.argv[1:6] == ["-o", "BatchMode=yes", "hel8", "bash", "-lc"], sys.argv
args = shlex.split(shlex.split(sys.argv[-1])[0])
assert args[0] == "claim"
fixture = Path(os.environ["GUARD_FIXTURE"])
behavior = json.loads((fixture / "central-behavior.json").read_text())
with (fixture / "central-calls").open("a") as output:
    output.write(json.dumps(args) + "\\n")
if args[1] == "release":
    task = json.loads((fixture / "src/fabrig/slot-maintenance/4/automatic.json").read_text())
    assert task["status"] == "completed" and task["exit"] == 0
    assert task["preservation"]["target_absent"] and task["end"]
    assert args[-2:] == ["--what", task["claim"]["what"]]
    archives = list((fixture / "src/fabrig/slot-maintenance/4").glob("automatic-*.json"))
    assert any(json.loads(path.read_text()) == task for path in archives)
    (fixture / "evidence-before-release").write_text("verified\\n")
    if behavior.get("fail_release"):
        sys.exit("fixture release unavailable")
if args[1] == "list" and behavior.get("fail_list"):
    sys.exit("fixture list unavailable")
os.environ["HOME"] = str(fixture / "central-home")
socket.gethostname = lambda: "hel8"
sys.argv = args
try:
    runpy.run_path(os.environ["GUARD_CLAIM_SOURCE"], run_name="__main__")
except SystemExit as done:
    code = done.code or 0
if args[1] == "claim" and code == 0 and behavior.get("ambiguous"):
    print("unexpected extra response")
sys.exit(code)
'''


class AutomaticClaimTests(unittest.TestCase):
    slot, add_run, ack = RecoveryTests.slot, RecoveryTests.add_run, RecoveryTests.ack
    state, cli, calls = RecoveryTests.state, RecoveryTests.cli, RecoveryTests.calls

    def setUp(self):
        RecoveryTests.setUp(self)
        ssh = self.bin / "ssh"
        ssh.write_text(FAKE_CENTRAL_SSH)
        ssh.chmod(0o700)
        environment = patch.dict(os.environ, GUARD_CLAIM_SOURCE=str(DIRECTORY / "claim"))
        environment.start()
        self.addCleanup(environment.stop)
        self.behavior()
        for module in (R, FIXTURE["guard"].RECOVERY):
            real = module["automatic_processes"]
            mocked = patch.dict(module["automatic_snapshot"].__globals__,
                                automatic_processes=lambda slot, inodes, real=real: real(slot, inodes, self.proc))
            mocked.start()
            self.addCleanup(mocked.stop)

    def behavior(self, **values):
        (self.home / "central-behavior.json").write_text(json.dumps(values))

    def ready(self):
        slot = self.slot("4")  # An actual inventory pool share; no invented lend.
        with L["Lease"](slot, self.home) as lease:
            lease.clean = True
        self.final("run-4")
        return slot

    def final(self, name, **changes):
        value = {"state": "withdrawn", "owner": "lead/fixture", "reason": "fixture completed",
                 "time": R["datetime"].now(R["timezone"].utc).isoformat()}
        value.update(changes)
        path = self.runs / name / "final.json"
        path.write_text(json.dumps(value))
        path.chmod(0o600)

    def claim_path(self, number="4"):
        return self.home / "central-home/src/fabrig/claims" / ("hetzner7-" + number)

    def journal(self, number="4"):
        return self.home / "src/fabrig/slot-maintenance" / number / "automatic.json"

    def run_auto(self, slot, *, record=None, free=None):
        details = []
        future = R["time"].time_ns() + 2 * 86400 * 10**9
        free = free or (lambda: 60_000_000_000 if (slot / "target").exists() else 81_000_000_000)
        with patch.dict(R["require_automatic_activation"].__globals__, FIRSTDAYDRYRUN=False), \
                patch.object(R["time"], "time_ns", return_value=future):
            R["automatic_recover"]([slot], self.home, free, record or details.append)
        return "\n".join(details)

    def test_gate_refuses_without_any_claim_or_local_maintenance_state(self):
        slot = self.slot("4")
        with self.assertRaisesRegex(RuntimeError, "FIRSTDAYDRYRUN"):
            R["automatic_recover"]([slot], self.home, lambda: 60, lambda line: None)
        self.assertFalse(self.journal().exists())
        self.assertFalse((self.home / "central-calls").exists())
        code, _ = self.cli("--automatic-targets")
        self.assertEqual(code, 2)
        self.assertFalse((self.home / "central-calls").exists())

    def test_completed_evidence_precedes_exact_release_and_preserves_source(self):
        slot = self.ready()
        before = R["source_preservation"](slot)
        result = self.run_auto(slot)
        self.assertIn("automatic reclaimed", result)
        self.assertFalse((slot / "target").exists())
        self.assertEqual(before, R["source_preservation"](slot))
        task = json.loads(self.journal().read_text())
        self.assertEqual((task["status"], task["release"], task["exit"]), ("completed", "released", 0))
        self.assertEqual((task["files_removed"], task["bytes_freed"]), (1, 21_000_000_000))
        self.assertTrue(task["preservation"]["target_absent"])
        self.assertTrue((self.home / "evidence-before-release").exists())
        self.assertFalse(self.claim_path().exists())
        self.assertEqual(task["claim"]["owner"], "ops")
        self.assertTrue(task["claim"]["what"].startswith("guard reclaim "))
        self.assertEqual(self.journal().stat().st_mode & 0o777, 0o600)

    def test_concurrent_claimant_refused_while_guard_owns_custody(self):
        slot = self.ready()
        attempts = []
        def competing(line):
            if "reclaim-start" in line:
                with self.assertRaisesRegex(RuntimeError, "held by"):
                    R["central_claim"]("claim", "hetzner7", "4", "competing lifetime")
                attempts.append(line)
        self.run_auto(slot, record=competing)
        self.assertEqual(len(attempts), 1)
        self.assertFalse(self.claim_path().exists())

    def test_ambiguous_success_retains_actual_claim_and_never_deletes(self):
        slot = self.ready()
        self.behavior(ambiguous=True)
        self.assertIn("ambiguous", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())
        claim = json.loads(self.claim_path().read_text())
        task = json.loads(self.journal().read_text())
        self.assertEqual(claim["what"], task["claim"]["what"])
        self.assertEqual(task["status"], "uncertain")

    def test_nonpool_share_refusal_does_not_invent_lending_permission(self):
        slot = self.slot("1")
        self.assertIn("lent-by", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())
        self.assertFalse(self.claim_path("1").exists())

    def test_failed_release_keeps_completed_reconciliation_evidence(self):
        slot = self.ready()
        self.behavior(fail_release=True)
        self.assertIn("release unavailable", self.run_auto(slot))
        task = json.loads(self.journal().read_text())
        self.assertEqual((task["status"], task["release"]), ("completed", "retained"))
        self.assertTrue(self.claim_path().exists())
        self.assertFalse((slot / "target").exists())
        self.assertTrue((self.home / "evidence-before-release").exists())
        self.assertIn("prior automatic claim/task", self.run_auto(slot, free=lambda: 60_000_000_000))

    def test_death_after_deletion_before_evidence_retains_claim(self):
        slot = self.ready()
        child = os.fork()
        if child == 0:
            def die(line):
                if "deletion returned" in line:
                    os._exit(73)
            self.run_auto(slot, record=die)
            os._exit(74)
        _, status = os.waitpid(child, 0)
        self.assertEqual(os.waitstatus_to_exitcode(status), 73)
        self.assertFalse((slot / "target").exists())
        self.assertTrue(self.claim_path().exists())
        self.assertNotEqual(json.loads(self.journal().read_text())["status"], "completed")
        self.assertFalse((self.home / "evidence-before-release").exists())

    def test_missing_invalid_or_wrong_owner_finality_stays_protected(self):
        for mutation in ("missing", "owner", "state", "time", "sha", "reason"):
            with self.subTest(mutation=mutation):
                slot = self.ready()
                final = self.runs / "run-4/final.json"
                if mutation == "missing":
                    final.unlink()
                else:
                    changes = {mutation: "unknown"}
                    if mutation == "sha":
                        changes["state"] = "landed"
                    if mutation == "reason":
                        changes["reason"] = ""
                    self.final("run-4", **changes)
                self.assertIn("PROTECTED/reconcile", self.run_auto(slot))
                self.assertTrue((slot / "target/cache").exists())
                self.assertTrue(self.claim_path().exists())
                # Explicit fixture teardown between independent histories/claims.
                shutil.rmtree(self.home / "src/fabrig/slot-maintenance")
                shutil.rmtree(self.home / "central-home")
                shutil.rmtree(slot)
                shutil.rmtree(self.runs / "run-4")

    def test_every_matching_run_needs_finality_even_if_latest_final_is_valid(self):
        slot = self.ready()
        self.add_run(slot, "earlier-unresolved", 1_700_000_001, "Question: resume?\n")
        self.assertIn("PROTECTED/reconcile", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_busy_service_protects_after_acquisition(self):
        slot = self.ready()
        self.state(units={"job-heavy-wrapper.service": {"WorkingDirectory": str(slot), "ActiveState": "active"}})
        self.assertIn("live service", self.run_auto(slot))
        self.assertTrue(self.claim_path().exists())
        self.assertTrue((slot / "target/cache").exists())

    def test_new_process_at_deletion_boundary_blocks(self):
        slot = self.ready()
        def arriving(line):
            if "reclaim-start" in line:
                process = self.proc / "123"
                process.mkdir()
                (process / "cwd").symlink_to(slot)
        self.assertNotIn("automatic reclaimed", self.run_auto(slot, record=arriving))
        self.assertTrue((slot / "target/cache").exists())
        self.assertTrue(self.claim_path().exists())

    def test_live_fd_and_maps_protect_without_a_slot_cwd(self):
        slot = self.ready()
        process = self.proc / "123"
        process.mkdir()
        (process / "cwd").symlink_to(self.home)
        (process / "fd").mkdir()
        (process / "maps").write_text("")
        (process / "fd/1").symlink_to(slot / "target/cache")
        self.assertIn("live source/target fd", self.run_auto(slot))
        (process / "fd/1").unlink()
        info = (slot / "target/cache").stat()
        (process / "maps").write_text(f"100-200 r--p 0 {os.major(info.st_dev):x}:{os.minor(info.st_dev):x} {info.st_ino} {slot}/target/cache\n")
        shutil.rmtree(self.home / "src/fabrig/slot-maintenance")
        shutil.rmtree(self.home / "central-home")
        with L["Lease"](slot, self.home) as lease:
            lease.clean = True
        self.assertIn("live source/target map", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_host_alternate_borrower_protects_target(self):
        slot = self.ready()
        alternate = self.home / "src/fabrig/foreign/.git/objects/info/alternates"
        alternate.parent.mkdir(parents=True)
        alternate.write_text(str(slot / "target") + "\n")
        self.assertIn("object-store borrower", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_transitive_alternate_borrower_protects_target(self):
        slot = self.ready()
        store = self.home / "src/fabrig/object-store"
        (store / "info").mkdir(parents=True)
        (store / "info/alternates").write_text(str(slot / "target") + "\n")
        alternate = self.home / "src/fabrig/foreign/.git/objects/info/alternates"
        alternate.parent.mkdir(parents=True)
        alternate.write_text(str(store) + "\n")
        self.assertIn("object-store borrower", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_dormant_submodule_object_store_protects_target(self):
        slot = self.ready()
        stored = self.home / "src/fabrig/foreign/.git/modules/dormant"
        (stored / "objects/info").mkdir(parents=True)
        (stored / "refs").mkdir()
        (stored / "HEAD").write_text("ref: refs/heads/fixture\n")
        (stored / "objects/info/alternates").write_text(str(slot / "target") + "\n")
        self.assertIn("object-store borrower", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_orphaned_object_store_without_refs_is_still_a_borrower(self):
        slot = self.ready()
        alternate = self.home / "src/fabrig/orphaned/objects/info/alternates"
        alternate.parent.mkdir(parents=True)
        alternate.write_text(str(slot / "target") + "\n")
        self.assertIn("object-store borrower", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_source_file_device_boundary_is_unknown_preservation(self):
        slot = self.ready()
        real = L["fingerprint"]
        def other_device(path):
            value = real(path)
            if Path(path) == slot / "source":
                value["identity"][0] += 1
            return value
        with patch.dict(L, fingerprint=other_device):
            self.assertIn("cross-device source file", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_source_change_before_mutation_blocks_and_after_mutation_retains_claim(self):
        slot = self.ready()
        def edit(line):
            if "reclaim-start" in line:
                (slot / "source").write_text("changed before deletion\n")
        self.run_auto(slot, record=edit)
        self.assertTrue((slot / "target/cache").exists())
        self.assertTrue(self.claim_path().exists())
        shutil.rmtree(self.home / "src/fabrig/slot-maintenance")
        shutil.rmtree(self.home / "central-home")
        with L["Lease"](slot, self.home) as lease:
            lease.clean = True
        def edit_after(line):
            if "deletion returned" in line:
                (slot / "source").write_text("changed after deletion\n")
        self.run_auto(slot, record=edit_after)
        self.assertFalse((slot / "target").exists())
        self.assertTrue(self.claim_path().exists())
        self.assertNotEqual(json.loads(self.journal().read_text())["status"], "completed")

    def test_missing_claim_recheck_response_retains_claim_and_target(self):
        slot = self.ready()
        self.behavior(fail_list=True)
        self.assertIn("list unavailable", self.run_auto(slot))
        self.assertTrue(self.claim_path().exists())
        self.assertTrue((slot / "target/cache").exists())

    def test_unknown_maps_proof_protects_target(self):
        slot = self.ready()
        process = self.proc / "123"
        process.mkdir()
        (process / "cwd").symlink_to(self.home)
        (process / "fd").mkdir()
        (process / "maps").write_text("unknown maps record\n")
        self.assertIn("unknown maps", self.run_auto(slot))
        self.assertTrue((slot / "target/cache").exists())

    def test_fresh_target_write_after_observation_blocks(self):
        slot = self.ready()
        def write_target(line):
            if "reclaim-start" in line:
                (slot / "target/cache").write_text("new build input\n")
        self.run_auto(slot, record=write_target)
        self.assertTrue((slot / "target/cache").exists())
        self.assertTrue(self.claim_path().exists())

    def test_exact_pressure_boundary_never_requests_claim(self):
        slot = self.ready()
        self.run_auto(slot, free=lambda: 80_000_000_000)
        self.assertFalse((self.home / "central-calls").exists())
        self.assertTrue((slot / "target/cache").exists())

    def test_landed_finality_accepts_only_a_full_sha(self):
        slot = self.ready()
        self.final("run-4", state="landed", sha="a" * 40)
        self.assertIn("automatic reclaimed", self.run_auto(slot))

    def test_activated_fixture_cli_honors_dry_run_before_exercising_live(self):
        slot = self.ready()
        future = R["time"].time_ns() + 2 * 86400 * 10**9
        with patch.dict(R["require_automatic_activation"].__globals__, FIRSTDAYDRYRUN=False), \
                patch.dict(FIXTURE["guard"].RECOVERY["require_automatic_activation"].__globals__, FIRSTDAYDRYRUN=False), \
                patch.object(R["time"], "time_ns", return_value=future):
            code, output = self.cli("--automatic-targets", "live", "--dry-run")
            self.assertEqual(code, 2)
            self.assertIn("automatic-targets dry-run", output)
            self.assertFalse((self.home / "central-calls").exists())
            self.assertTrue((slot / "target/cache").exists())
            code, output = self.cli("--automatic-targets", "live",
                                    free=lambda: 60 if (slot / "target").exists() else 81)
            self.assertEqual(code, 0, output)
        self.assertFalse((slot / "target").exists())
        self.assertFalse(self.claim_path().exists())


class ProcessClassificationTests(unittest.TestCase):
    def test_only_exact_protected_parent_comm_and_cgroup_pairs_qualify(self):
        prefix = "0::/user.slice/user-1000.slice/"
        for comm, group, expected in (
            ("systemd", prefix + "user@1000.service/init.scope", "user manager"),
            ("(sd-pam)", prefix + "user@1000.service/init.scope", "user manager"),
            ("sshd-session", prefix + "session-3730.scope", "SSH session parent"),
            ("bash", prefix + "session-3730.scope", None),
            ("python3", prefix + "user@1000.service/app.slice/job-heavy-build.service", None),
            ("systemd", prefix + "user@1000.service/app.slice/task.service", None),
            ("ssh-agent", prefix + "user@1000.service/app.slice/ssh-agent.service", None)):
            with self.subTest(comm=comm, group=group):
                self.assertEqual(R["process_class"](comm, group, 1000), expected)


if __name__ == "__main__":
    unittest.main()
