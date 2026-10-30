#!/usr/bin/env python3
"""Offline collector checks with private native fixtures (design.md, 7.4)."""

import json
import os
from pathlib import Path
import resource
import runpy
import signal
import statistics
import subprocess
import sys
import tempfile
import time
import unittest

HERE = Path(__file__).resolve().parent
CPU = runpy.run_path(str(HERE / 'cpu.py'))


def setUpModule():
    global WRAPPER, SETTLEMENT_WRAPPER
    native = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-cpu-native-')
    unittest.addModuleCleanup(native.cleanup)
    WRAPPER = Path(native.name) / 'tiers-cpu-wrapper'
    SETTLEMENT_WRAPPER = Path(native.name) / 'cpu-settlement-fixture'
    compiler = ['cc', '-std=c11', '-O2', '-static', '-Wall', '-Wextra', '-Werror',
                str(HERE / 'cpu-wrapper.c')]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    for arguments in (['-o', str(WRAPPER)],
                      [str(HERE / 'fixtures/cpu-settlement.c'),
                       '-Wl,--wrap=wait4,--wrap=clock_gettime,--wrap=fopen,--wrap=kill',
                       '-o', str(SETTLEMENT_WRAPPER)]):
        try:
            subprocess.run([*compiler, *arguments], check=True, stdin=subprocess.DEVNULL,
                           capture_output=True, text=True)
        except FileNotFoundError as error:
            raise RuntimeError('CPU collector tests require cc on PATH to build native fixtures') from error
        except subprocess.CalledProcessError as error:
            raise RuntimeError(f'CPU collector native fixture build failed: {error.stdout}{error.stderr}') from error
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    user_s = after.ru_utime - before.ru_utime
    system_s = after.ru_stime - before.ru_stime
    print(f'Native fixture build CPU: user {user_s:.6f}s, system {system_s:.6f}s, '
          f'total {user_s + system_s:.6f}s', flush=True)


class CollectorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-cpu-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.directory = self.root / 'records with spaces'
        self.config = self.root / 'original.toml'
        self.config.write_text('[profile.default]\nslow-timeout={period="6s"}\n')
        self.overlay = CPU['prepare'](self.directory, 'qa-jig', WRAPPER, self.config)
        self.item = dict(profile='qa-jig', binary_id='fixture::suite', test_name='one', attempt=1,
                         user_s=1.0, system_s=0.5, wall_s=2.0, start_s=100.0,
                         wrapper_cpu_s=0.001, complete=True, leaked=0, pending=False,
                         interrupted=0, exit_code=0, signal=None)
        self.finished()
        self.xml('<testcase name="one"/>')

    def finished(self, **fields):
        CPU['save'](self.directory / 'run.json', dict(profile='qa-jig', finished=True,
                                                     starved=False, wall_s=3.0, returncode=0, **fields))

    def xml(self, cases):
        (self.directory / 'junit.xml').write_text(
            '<testsuites><testsuite name="fixture::suite">' + cases + '</testsuite></testsuites>')

    def record(self, name='cpu-one', **fields):
        CPU['save'](self.directory / 'records' / name, dict(self.item, **fields))

    def cli(self, *arguments):
        return subprocess.run([sys.executable, '-I', '-B', str(HERE / 'cpu-summary'),
                               str(self.directory), *arguments], text=True, capture_output=True,
                              stdin=subprocess.DEVNULL, timeout=10)

    def test_overlay_arguments_and_every_profile(self):
        self.assertEqual(self.overlay, ['--config-file', str(self.directory / 'repository.toml'),
                                        '--tool-config-file', f'tiers-cpu:{self.directory}/nextest.toml'])
        overlay = CPU['tomllib'].loads((self.directory / 'nextest.toml').read_text())
        repository = CPU['tomllib'].loads((self.directory / 'repository.toml').read_text())
        self.assertEqual(repository['experimental'], ['wrapper-scripts'])
        self.assertEqual(repository['profile']['default']['slow-timeout'], {'period': '6s'})
        self.assertEqual(overlay['scripts']['wrapper']['@tool:tiers-cpu:account']['command'],
                         [str(WRAPPER), str(self.directory / 'records')])
        self.assertEqual(overlay['profile']['qa-jig']['scripts'],
                         [{'filter': 'all()', 'run-wrapper': '@tool:tiers-cpu:account'}])
        self.assertEqual(overlay['profile']['qa-jig']['junit']['path'], str(self.directory / 'junit.xml'))
        with self.assertRaisesRegex(ValueError, 'must be empty'):
            CPU['prepare'](self.directory, 'default', WRAPPER)
        for profile in ('default', 'fuzzy', 'live-temper', 'exhaustive-skein'):
            path = self.root / profile
            CPU['prepare'](path, profile, WRAPPER, self.config)
            self.assertIn(profile, CPU['tomllib'].loads((path / 'nextest.toml').read_text())['profile'])

    def test_summary_totals_json_and_verdict_exits(self):
        self.record()
        result = self.cli('--json')
        self.assertEqual(result.returncode, 0, result.stderr)
        suite = json.loads(result.stdout)['suite']
        self.assertEqual((suite['cpu_s'], suite['user_s'], suite['system_s'], suite['wall_s']), (1.5, 1, 0.5, 3))
        self.assertEqual((suite['tests'], suite['attempts']), (1, 1))
        budget = self.root / 'budgets.toml'
        budget.write_text('[qa-jig]\ncpu_suite_s = 1.5\ncpu_test_s = 1.5\n')
        self.assertEqual(self.cli('--budgets', str(budget), '--pair', 'qa-jig').returncode, 0)
        budget.write_text('[qa-jig]\ncpu_suite_s = 1.0\ncpu_test_s = 1.0\n')
        result = self.cli('--budgets', str(budget))
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn('test fixture::suite one', result.stdout)
        self.assertIn('suite qa-jig', result.stdout)
        self.assertEqual(self.cli('--pair', 'qa-skein').returncode, 2)

    def test_missing_extra_duplicate_malformed_and_unfinished_records(self):
        self.assertIn('missing', self.cli().stderr)
        self.record()
        self.record('cpu-two')
        self.assertEqual(self.cli().returncode, 2)
        (self.directory / 'records/cpu-two').unlink()
        for fields in ({'user_s': -1}, {'user_s': '1'}, {'attempt': 0}, {'attempt': True},
                       {'complete': False}, {'interrupted': '15'}, {'signal': 9}, {'pending': True, 'exit_code': None, 'complete': False}):
            with self.subTest(fields=fields):
                self.record(**fields)
                self.assertEqual(self.cli().returncode, 2)
        self.record(test_name='unreported')
        self.assertIn('unexpected', self.cli().stderr)
        self.record()
        (self.directory / 'junit.xml').unlink()
        self.assertIn('missing nextest JUnit', self.cli().stderr)
        self.finished(finished_unused=True)
        data = json.loads((self.directory / 'run.json').read_text())
        data['finished'] = False
        CPU['save'](self.directory / 'run.json', data)
        self.assertIn('not finished', self.cli().stderr)

    def test_retries_skips_timeouts_and_real_failures(self):
        self.xml('<testcase name="one"><flakyFailure type="test timeout"/></testcase>'
                 '<testcase name="skip"><skipped/></testcase>')
        self.record(exit_code=None, signal=15)
        self.record('cpu-two', attempt=2)
        result = CPU['summary'](self.directory)
        self.assertEqual(result['suite']['attempts'], 2)
        self.assertEqual(result['suite']['cpu_s'], 3)
        self.assertEqual(result['suite']['starved'], 1)
        self.assertEqual(result['failures'], [])
        self.xml('<testcase name="one"><failure type="test failure"/><rerunFailure type="test timeout"/></testcase>')
        result = CPU['summary'](self.directory)
        self.assertEqual(result['suite']['starved'], 1)
        self.assertEqual(result['failures'], ['fixture::suite one'])
        (self.directory / 'records/cpu-two').unlink()
        self.assertIn('missing', self.cli().stderr)

    def test_starved_run_retains_incomplete_and_existing_failures(self):
        data = dict(profile='qa-jig', finished=True, starved=True, wall_s=3, returncode=130)
        CPU['save'](self.directory / 'run.json', data)
        (self.directory / 'junit.xml').unlink()
        self.record(pending=True, complete=False, exit_code=None)
        result = CPU['summary'](self.directory)
        self.assertEqual(result['verdict'], 'starved')
        self.assertEqual(result['suite']['incomplete'], 1)
        self.assertEqual(result['suite']['starved'], 1)
        self.record('cpu-two', test_name='failed', exit_code=1)
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite failed'])

    def test_finished_run_past_hard_wall_is_still_starved(self):
        self.record()
        result = CPU['summary'](self.directory, {'qa-jig': {'wall_hard_s': 1}})
        self.assertEqual(result['verdict'], 'starved')
        self.assertTrue(result['suite']['starved_run'])
        self.assertEqual(result['failures'], [])

    def test_timeout_pass_and_failure_after_timeout(self):
        self.record(exit_code=None, signal=15, interrupted=15)
        result = CPU['summary'](self.directory)
        self.assertEqual(result['suite']['starved'], 1)
        self.assertEqual(result['failures'], [])
        self.xml('<testcase name="one"><failure type="test timeout"/><rerunFailure type="test failure"/></testcase>')
        self.record('cpu-two', attempt=2, exit_code=1)
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])

    def test_timeout_pass_after_group_signal_uses_junit_success(self):
        self.record(exit_code=None, signal=15, interrupted=0)
        result = CPU['summary'](self.directory)
        self.assertEqual(result['suite']['starved'], 1)
        self.assertEqual(result['failures'], [])

    def test_budget_validation(self):
        path = self.root / 'budgets.toml'
        for content in ('[unknown]\ncpu_test_s=1', '[qa-jig]\nwall_hard_s=0',
                        '[qa-jig]\ncpu_suite_s=-1', '[qa-jig]\ncpu_test_s=nan',
                        '[qa-jig]\ncpu_test_s=inf', '[qa-jig]\ncpu_test_s=true',
                        '[qa-jig]\nunknown=1', 'qa-jig=1', '[qa-jig'):
            with self.subTest(content=content):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    CPU['budgets'](path, {'qa-jig'})
        path.write_text('')
        self.assertEqual(CPU['budgets'](path, {'qa-jig'}), {})

    def test_hard_wall_uses_heavy_cancel_and_stays_green(self):
        fake = self.root / 'heavy'
        fake.write_text('''#!/usr/bin/env python3
import json, os, signal, subprocess, sys
from pathlib import Path
state = Path(os.environ['FIXTURE_STATE'])
if sys.argv[1] == '--cancel':
    (state / 'cancelled').write_text(sys.argv[2])
    os.killpg(int((state / 'pid').read_text()), signal.SIGKILL)
    sys.exit(0)
print('heavy: queued task 1 in the build lane token abcdef', flush=True)
env = dict(os.environ, NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
           NEXTEST_TEST_NAME='one', NEXTEST_ATTEMPT='1')
child = subprocess.Popen([env['FIXTURE_WRAPPER'], env['FIXTURE_RECORDS'], '/bin/sleep', '5'],
                         env=env, start_new_session=True)
(state / 'pid').write_text(str(child.pid))
child.wait()
sys.exit(130)
''')
        fake.chmod(0o700)
        (self.directory / 'junit.xml').unlink()
        driver = self.root / 'driver.py'
        driver.write_text("import os,runpy,sys\n"
                          f"cpu=runpy.run_path({str(HERE / 'cpu.py')!r})\n"
                          f"sys.exit(cpu['execute'](['heavy','fixture'], os.environ, {str(self.directory)!r}, "
                          "'qa-jig', {'qa-jig': {'wall_hard_s': 0.1}}))\n")
        env = dict(os.environ, PATH=f"{self.root}:{os.environ['PATH']}",
                   FIXTURE_STATE=str(self.root), FIXTURE_WRAPPER=str(WRAPPER),
                   FIXTURE_RECORDS=str(self.directory / 'records'))
        result = subprocess.run([sys.executable, '-I', '-B', str(driver)], env=env,
                                stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertEqual((self.root / 'cancelled').read_text(), 'abcdef')
        self.assertIn('starved', result.stdout)
        summary = CPU['summary'](self.directory)
        self.assertEqual(summary['suite']['incomplete'], 1)
        self.assertEqual(summary['suite']['starved'], 1)
        self.assertTrue(summary['suite']['starved_run'])

    def test_timeout_is_green_and_other_failures_and_budgets_remain_red(self):
        driver = self.root / 'verdict.py'
        driver.write_text('import os,runpy,sys\n'
                          f'cpu=runpy.run_path({str(HERE / "cpu.py")!r})\n'
                          f"sys.exit(cpu['execute']([sys.executable, '-c', 'import sys; sys.exit(100)'], "
                          f"os.environ, {str(self.directory)!r}, 'qa-jig', "
                          "{'qa-jig': {'cpu_suite_s': float(sys.argv[1])}} if len(sys.argv)>1 else {}))\n")
        def invoke(*args):
            return subprocess.run([sys.executable, '-I', '-B', str(driver), *args],
                                  stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=5)
        self.record(exit_code=None, signal=15)
        self.xml('<testcase name="one"><failure type="test timeout"/></testcase>')
        result = invoke()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('starved', result.stdout)
        result = invoke('0.5')
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.record(exit_code=None, signal=signal.SIGSEGV)
        result = invoke()
        self.assertEqual(result.returncode, 100, result.stdout + result.stderr)
        self.record(exit_code=None, signal=15)
        self.record('cpu-two', test_name='assertion', exit_code=1)
        self.xml('<testcase name="one"><failure type="test timeout"/></testcase>'
                 '<testcase name="assertion"><failure type="test failure"/></testcase>')
        result = invoke()
        self.assertEqual(result.returncode, 100, result.stdout + result.stderr)

    def starved(self, *, junit=False):
        CPU['save'](self.directory / 'run.json', dict(profile='qa-jig', finished=True,
                                                     starved=True, wall_s=3, returncode=130))
        if not junit:
            (self.directory / 'junit.xml').unlink()

    def assert_cancelled_verdict(self, expected):
        self.record('cpu-running', test_name='running', pending=True, complete=False, exit_code=None)
        marker = self.root / 'cancelled'
        marker.unlink(missing_ok=True)
        heavy = self.root / 'heavy'
        heavy.write_text(f'''#!/usr/bin/env python3
from pathlib import Path
import sys,time
marker=Path({str(marker)!r})
if sys.argv[1] == '--cancel':
    marker.touch()
    sys.exit(0)
print('heavy: queued task 1 in the build lane token abcdef',flush=True)
while not marker.exists():
    time.sleep(0.001)
sys.exit(130)
''')
        heavy.chmod(0o700)
        driver = self.root / 'cancel-verdict.py'
        driver.write_text('import os,runpy,sys\n'
                          f'cpu=runpy.run_path({str(HERE / "cpu.py")!r})\n'
                          f"sys.exit(cpu['execute'](['heavy','fixture'],os.environ,{str(self.directory)!r},"
                          "'qa-jig',{'qa-jig':{'wall_hard_s':0.001}}))\n")
        result = subprocess.run([sys.executable,'-I','-B',str(driver)],
                                env=dict(os.environ,PATH=f"{self.root}:{os.environ['PATH']}"),
                                stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=5)
        self.assertEqual(result.returncode,expected,result.stdout+result.stderr)

    def test_cancel_with_junit_preserves_extra_failed_record(self):
        self.starved(junit=True)
        self.record()
        self.record('cpu-extra', test_name='extra', exit_code=7)
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite extra'])
        self.assert_cancelled_verdict(100)
        self.record('cpu-extra-pass', test_name='extra', attempt=2)
        self.assertEqual(CPU['summary'](self.directory)['failures'], [])
        self.assert_cancelled_verdict(0)

    def test_cancel_pending_retry_preserves_completed_failure(self):
        self.starved()
        self.record(exit_code=7)
        self.record('cpu-retry', attempt=2, pending=True, complete=False, exit_code=None)
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])
        self.assert_cancelled_verdict(100)
        self.record('cpu-pass', attempt=3)
        self.assertEqual(CPU['summary'](self.directory)['failures'], [])
        self.assert_cancelled_verdict(0)

    def live_wrapper(self, code, *, wrapper=None, extra_env=None):
        if wrapper is None:
            wrapper = WRAPPER
        env = dict(os.environ, NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                   NEXTEST_TEST_NAME='one', NEXTEST_ATTEMPT='1', **(extra_env or {}))
        process = subprocess.Popen([str(wrapper), str(self.directory / 'records'),
                                    sys.executable, '-I', '-c', code], env=env,
                                   stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
        def cleanup():
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.communicate(timeout=5)
        self.addCleanup(cleanup)
        return process

    def wait_until(self, predicate):
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            if predicate():
                return
            time.sleep(0.001)
        self.fail('fixture did not reach its expected state')

    def test_group_signal_is_delivered_once(self):
        ready, events, done = (self.root / name for name in ('ready', 'events', 'done'))
        process = self.live_wrapper(f'''import os,signal,time
from pathlib import Path
ready,events,done = map(Path, {[str(ready), str(events), str(done)]!r})
def handle(sig, frame):
    with events.open('a') as output:
        output.write(str(sig)+'\\n')
for sig in (signal.SIGTERM,signal.SIGINT,signal.SIGHUP,signal.SIGQUIT):
    signal.signal(sig,handle)
ready.touch()
while not done.exists():
    time.sleep(0.001)
''')
        self.wait_until(ready.exists)
        # Hold only the supervisor so the child handles the group signal before
        # any supervisor-side forwarding could run and coalesce with it.
        expected = []
        for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP, signal.SIGQUIT):
            os.kill(process.pid, signal.SIGSTOP)
            os.killpg(process.pid, sig)
            expected.append(str(sig))
            self.wait_until(lambda: events.exists() and len(events.read_text().splitlines()) >= len(expected))
            os.kill(process.pid, signal.SIGCONT)
            time.sleep(0.03)
        done.touch()
        output, error = process.communicate(timeout=5)
        self.assertEqual(process.returncode, 0, (output, error))
        self.assertEqual(events.read_text().splitlines(), expected)

    def test_repeated_group_signals_need_no_forwarding_or_wakeup(self):
        ready, events, done = (self.root / name for name in ('ready', 'events', 'done'))
        process = self.live_wrapper(f'''import signal,time
from pathlib import Path
ready,events,done = map(Path, {[str(ready), str(events), str(done)]!r})
def handle(sig, frame):
    with events.open('a') as output:
        output.write(str(sig)+'\\n')
signal.signal(signal.SIGTERM,handle)
ready.touch()
while not done.exists():
    time.sleep(0.001)
''')
        self.wait_until(ready.exists)
        # PID-only signals must never be forwarded. They can arrive while the
        # supervisor is already blocked in wait4; group signals reach the test.
        os.kill(process.pid, signal.SIGTERM)
        time.sleep(0.03)
        self.assertFalse(events.exists(), 'supervisor forwarded a PID-only signal')
        for count in range(1, 4):
            os.killpg(process.pid, signal.SIGTERM)
            self.wait_until(lambda: events.exists() and len(events.read_text().splitlines()) >= count)
        done.touch()
        output, error = process.communicate(timeout=5)
        self.assertEqual(process.returncode, 0, (output, error))
        self.assertEqual(events.read_text().splitlines(), [str(signal.SIGTERM)] * 3)

    def test_failed_test_is_checkpointed_before_descendant_settlement(self):
        orphan = self.root / 'orphan'
        process = self.live_wrapper(f'''import os,time
from pathlib import Path
if os.fork() == 0:
    Path({str(orphan)!r}).touch()
    time.sleep(30)
    os._exit(0)
while not Path({str(orphan)!r}).exists():
    time.sleep(0.001)
os._exit(7)
''')
        self.wait_until(orphan.exists)
        def checkpointed():
            items = CPU['records'](self.directory)
            return bool(items and items[0]['exit_code'] == 7)
        self.wait_until(checkpointed)
        item = CPU['records'](self.directory)[0]
        self.assertIsNone(process.poll(), 'test status was only published after settlement')
        self.assertFalse(item['complete'])
        os.killpg(process.pid, signal.SIGKILL)
        process.communicate(timeout=5)
        self.starved()
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])

    def test_auto_reaped_cpu_has_explicit_accounting_scope(self):
        measured = self.root / 'auto-reaped-cpu'
        result, items = self.wrapped(f'''import os,signal,time
from pathlib import Path
signal.signal(signal.SIGCHLD,signal.SIG_IGN)
if os.fork() == 0:
    start=time.process_time()
    while time.process_time()-start < 0.04:
        pass
    Path({str(measured)!r}).write_text(str(time.process_time()-start))
    os._exit(0)
while not Path({str(measured)!r}).exists():
    time.sleep(0.001)
while True:
    try:
        os.waitpid(-1,os.WNOHANG)
    except ChildProcessError:
        break
    time.sleep(0.001)
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertGreater(float(measured.read_text()), 0.03)
        self.assertEqual(items[0].get('complete_scope'), 'waited-and-reaped-descendants')

    def test_settlement_deadline_and_cleanup_apply_during_reap_stream(self):
        marker = self.root / 'cleanup'
        env = dict(os.environ, NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                   NEXTEST_TEST_NAME='one', NEXTEST_ATTEMPT='1', TIERS_CLEANUP_MARKER=str(marker))
        result = subprocess.run([str(SETTLEMENT_WRAPPER), str(self.directory / 'records'), '/bin/true'],
                                env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        item = CPU['records'](self.directory)[0]
        self.assertFalse(item['complete'], 'continuous successful reaps bypassed the deadline')
        self.assertLess(item['wall_s'], 1.1)
        self.assertTrue(marker.exists(), 'continuous successful reaps bypassed cleanup')

    def wrapped(self, code, *, name='one', input_text=None):
        env = dict(os.environ, NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                   NEXTEST_TEST_NAME=name, NEXTEST_ATTEMPT='1')
        result = subprocess.run([str(WRAPPER), str(self.directory / 'records'),
                                 sys.executable, '-I', '-c', code], env=env,
                                input=input_text, text=True, capture_output=True, timeout=5)
        return result, CPU['records'](self.directory)

    def test_wrapper_stdio_and_exit_pass_through(self):
        name = ('quoted"\\\n\t\r\b\f\x01é😺name' * 512)
        result, items = self.wrapped('import sys; print(sys.stdin.read()); print("err", file=sys.stderr); sys.exit(7)',
                                    input_text='input', name=name)
        self.assertEqual((result.returncode, result.stdout, result.stderr), (7, 'input\n', 'err\n'))
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]['test_name'], name)
        self.assertTrue(items[0]['complete'])
        self.assertEqual(items[0]['exit_code'], 7)
        self.assertGreater(items[0]['user_s'] + items[0]['system_s'], 0)

    def test_wrapper_signal_pass_through(self):
        result, items = self.wrapped('import os,signal; os.kill(os.getpid(), signal.SIGTERM)')
        self.assertEqual(result.returncode, -signal.SIGTERM)
        self.assertEqual(items[0]['signal'], signal.SIGTERM)
        self.assertIsNone(items[0]['exit_code'])
        self.assertTrue(items[0]['complete'])

    def test_wrapper_reaps_daemon_and_counts_descendant_cpu(self):
        result, items = self.wrapped('''import os, time
child = os.fork()
if child == 0:
    os.setsid()
    child = os.fork()
    if child:
        os._exit(0)
    print(os.getpid(), flush=True)
    end = time.process_time() + 0.03
    while time.process_time() < end:
        pass
    time.sleep(30)
os.waitpid(child, 0)
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        daemon = int(result.stdout.strip())
        with self.assertRaises(ProcessLookupError):
            os.kill(daemon, 0)
        self.assertEqual(items[0]['leaked'], 1)
        self.assertTrue(items[0]['complete'])
        self.assertGreater(items[0]['user_s'] + items[0]['system_s'], 0.03)
        self.assertLess(items[0]['wall_s'], 2)

    def test_wrapper_waited_descendant_is_accounted_once(self):
        result, items = self.wrapped('''import os,time
child = os.fork()
if child == 0:
    end = time.process_time() + 0.03
    while time.process_time() < end:
        pass
    os._exit(0)
os.waitpid(child, 0)
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        cpu = items[0]['user_s'] + items[0]['system_s']
        self.assertGreater(cpu, 0.03)
        self.assertLess(cpu, 0.09)
        self.assertEqual(items[0]['leaked'], 0)

    def test_wrapper_overhead_on_100_trivial_attempts(self):
        overheads = []
        snapshots = []
        for attempt in range(100):
            env = dict(os.environ, NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                       NEXTEST_TEST_NAME=f'trivial_{attempt}', NEXTEST_ATTEMPT='1')
            child = os.posix_spawn(str(WRAPPER), [str(WRAPPER), str(self.directory / 'records'), '/bin/true'], env)
            _, status, usage = os.wait4(child, 0)
            self.assertEqual(status, 0)
            item = next(r for r in CPU['records'](self.directory) if r['test_name'] == f'trivial_{attempt}')
            overheads.append(usage.ru_utime + usage.ru_stime
                             - item['user_s'] - item['system_s'])
            snapshots.append(item['wrapper_cpu_s'])
        print(f'Wrapper CPU overhead, 100 trivial tests: total {sum(overheads):.6f}s, '
              f'mean {sum(overheads) / 100 * 1000:.3f}ms, '
              f'median {statistics.median(overheads) * 1000:.3f}ms, '
              f'max {max(overheads) * 1000:.3f}ms', flush=True)
        slowest = max(range(100), key=overheads.__getitem__)
        print(f'Slowest attempt: wrapper CPU before final publication {snapshots[slowest] * 1000:.3f}ms, '
              f'whole-lifetime overhead {overheads[slowest] * 1000:.3f}ms', flush=True)
        self.assertLess(statistics.median(overheads), 0.002)
        self.assertLess(max(overheads), 0.010)


class CommandTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-cpu-command-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.directory = self.root / 'suite records'
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.env = dict(os.environ, PATH=f"{self.bin}:{os.environ['PATH']}",
                        FIXTURE_TARGET=str(WRAPPER.parent), FIXTURE_WRAPPER=str(WRAPPER),
                        FIXTURE_COMMAND_LOG=str(self.root / 'commands'),
                        NEXTEST_PROFILE='unchanged')
        self.fake('cargo', "import json,os,sys\n"
                  "assert sys.argv[1:]==['metadata','--format-version','1','--no-deps']\n"
                  "print(json.dumps({'target_directory':os.environ['FIXTURE_TARGET']}))\n")
        self.fake('heavy', "import json,os,shutil,sys\nfrom pathlib import Path\n"
                  "Path(os.environ['FIXTURE_COMMAND_LOG']).write_text(json.dumps(sys.argv[1:]))\n"
                  "if os.environ.get('FIXTURE_FAIL_BUILD'): sys.exit(9)\n"
                  "shutil.copyfile(os.environ['FIXTURE_WRAPPER'],sys.argv[-1])\n"
                  "Path(sys.argv[-1]).chmod(0o700)\n")
        self.fake('df', "import os\nfrom pathlib import Path\n"
                  "Path(os.environ['FIXTURE_COMMAND_LOG']+'.disk').touch()\n"
                  "print('Avail\\n80000000000')\n")

    def fake(self, name, source):
        path = self.bin / name
        path.write_text(f'#!{sys.executable}\n' + source)
        path.chmod(0o700)

    def invoke(self, code, *, suite='tools/fixture.py', input_text=None, wall_hard=None):
        timeout = [] if wall_hard is None else ['--wall-hard', str(wall_hard)]
        return subprocess.run([sys.executable, '-I', '-B', str(HERE / 'cpu-run'),
                               '--out', str(self.directory), '--suite', suite, *timeout, '--',
                               sys.executable, '-I', '-c', code], env=self.env,
                              input=input_text, text=True, capture_output=True, timeout=10)

    def summary(self, *arguments):
        return subprocess.run([sys.executable, '-I', '-B', str(HERE / 'cpu-summary'),
                               str(self.directory), '--pair', 'tools', *arguments],
                              stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=5)

    def record(self):
        paths = list(self.directory.glob('*.json'))
        self.assertEqual(len(paths), 1)
        return paths[0], json.loads(paths[0].read_text())

    def test_command_stdio_environment_status_and_named_atomic_record(self):
        suite = '../tools/quoted" é%_.py'
        result = self.invoke('import os,sys; print(sys.stdin.read()); '
                             'print(os.environ["NEXTEST_PROFILE"],file=sys.stderr); sys.exit(7)',
                             suite=suite, input_text='input')
        self.assertEqual((result.returncode, result.stdout, result.stderr), (7, 'input\n', 'unchanged\n'))
        path, item = self.record()
        self.assertEqual(path.name, '..%2Ftools%2Fquoted%22%20%C3%A9%25_.py.json')
        self.assertEqual((item['profile'], item['binary_id'], item['test_name'], item['attempt']),
                         ('tools', suite, 'suite', 1))
        self.assertEqual(item['exit_code'], 7)
        self.assertTrue(item['complete'])
        self.assertFalse(item['pending'])
        self.assertFalse(item['settling'])
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(list(self.directory.glob('*.tmp')), [])
        self.assertEqual(self.summary('--expect', suite).returncode, 0)
        self.assertEqual(self.invoke('raise Exception("must not run")', suite=suite).returncode, 2)
        self.assertEqual(json.loads(path.read_text()), item)
        # The native mode also refuses concurrent attempts to publish one suite.
        duplicate = subprocess.run([str(WRAPPER), '--suite', str(path), suite, '--', '/bin/false'],
                                   stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(duplicate.returncode, 125)
        self.assertEqual(json.loads(path.read_text()), item)

    def test_command_signal_passes_through(self):
        result = self.invoke('import os,signal; os.kill(os.getpid(),signal.SIGTERM)')
        self.assertEqual(result.returncode, -signal.SIGTERM, result.stderr)
        _, item = self.record()
        self.assertEqual(item['signal'], signal.SIGTERM)
        self.assertIsNone(item['exit_code'])
        self.assertTrue(item['complete'])

    def test_wall_starvation_reaps_payload_group_and_escaped_child(self):
        for mode in ('term', 'kill', 'escaped'):
            with self.subTest(mode=mode):
                result = self.invoke(f'''import os,signal,time
mode={mode!r}
if mode=='kill': signal.signal(signal.SIGTERM,signal.SIG_IGN)
print(os.getpid(),flush=True)
if os.fork()==0:
    if mode=='escaped': os.setsid()
    print(os.getpid(),flush=True)
time.sleep(30)
''', suite=mode, wall_hard=0.25)
                self.assertEqual(result.returncode,
                                 -signal.SIGKILL if mode == 'kill' else -signal.SIGTERM,
                                 result.stderr)
                pids = [int(line) for line in result.stdout.splitlines()]
                self.assertEqual(len(pids), 2)
                for pid in pids:
                    with self.assertRaises(ProcessLookupError):
                        os.kill(pid, 0)
                item = next(r for r in CPU['records'](self.directory, tools=True)
                            if r['binary_id'] == mode)
                self.assertTrue(item['starved'])
                self.assertTrue(item['complete'])
                self.assertFalse(item['pending'])
                self.assertFalse(item['settling'])
                if mode != 'term':
                    self.assertGreaterEqual(item['wall_s'], 1.25)
                    self.assertEqual(item['leaked'], 1)
        result = self.summary('--expect', 'term', '--expect', 'kill', '--expect', 'escaped', '--json')
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report['verdict'], 'starved')
        self.assertEqual(report['suite']['starved'], 3)
        self.assertEqual(report['suite']['incomplete'], 0)
        self.assertEqual(report['failures'], [])

    def test_killed_supervisor_starved_checkpoint_is_invalid(self):
        ready = self.root / 'payload'
        process = subprocess.Popen([sys.executable, '-I', '-B', str(HERE / 'cpu-run'),
                                    '--out', str(self.directory), '--suite', 'killed',
                                    '--wall-hard', '0.25', '--', sys.executable, '-I', '-c',
                                    'import os,signal,time; from pathlib import Path; '
                                    'signal.signal(signal.SIGTERM,signal.SIG_IGN); '
                                    f'Path({str(ready)!r}).write_text(str(os.getpid())); time.sleep(30)'],
                                   env=self.env, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                   start_new_session=True)
        def cleanup():
            if ready.exists():
                try:
                    os.killpg(int(ready.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=5)
        self.addCleanup(cleanup)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            paths = list(self.directory.glob('*.json'))
            if paths and json.loads(paths[0].read_text())['starved']:
                break
            time.sleep(0.001)
        else:
            self.fail('supervisor did not checkpoint starvation')
        process.kill()
        process.communicate(timeout=5)
        self.assertEqual(process.returncode, -signal.SIGKILL)
        _, item = self.record()
        self.assertTrue(item['starved'])
        self.assertTrue(item['pending'])
        self.assertFalse(item['complete'])
        result = self.summary('--expect', 'killed')
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('incomplete', result.stderr)

    def test_wall_hard_validation_and_timely_completion(self):
        for value in ('0', '-1', 'nan', 'inf', 'bad'):
            with self.subTest(value=value):
                result = self.invoke('raise Exception("must not run")', wall_hard=value)
                self.assertEqual(result.returncode, 2, result.stderr)
        result = self.invoke('pass', wall_hard=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        _, item = self.record()
        self.assertFalse(item['starved'])
        self.assertTrue(item['complete'])
        self.assertEqual(self.summary('--expect', 'tools/fixture.py').returncode, 0)

    def test_command_daemon_is_reaped_and_accounted(self):
        result = self.invoke('''import os,time
child=os.fork()
if child==0:
    os.setsid()
    if os.fork(): os._exit(0)
    print(os.getpid(),flush=True)
    end=time.process_time()+0.03
    while time.process_time()<end: pass
    time.sleep(30)
os.waitpid(child,0)
''')
        self.assertEqual(result.returncode, 0, result.stderr)
        with self.assertRaises(ProcessLookupError):
            os.kill(int(result.stdout), 0)
        _, item = self.record()
        self.assertTrue(item['complete'])
        self.assertEqual(item['leaked'], 1)
        self.assertGreater(item['user_s'] + item['system_s'], 0.03)

    def test_command_builds_missing_or_stale_wrapper_with_disk_admission(self):
        target = self.root / 'target with spaces'
        self.env['FIXTURE_TARGET'] = str(target)
        for stale in (False, True):
            with self.subTest(stale=stale):
                if stale:
                    os.utime(target / 'tiers-cpu-wrapper', (1, 1))
                result = self.invoke('print("suite output")', suite=str(stale))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, 'suite output\n')
                command = json.loads((self.root / 'commands').read_text())
                self.assertEqual(command, ['cc', '-std=c11', '-O2', '-static', '-Wall', '-Wextra',
                                          '-Werror', str(HERE / 'cpu-wrapper.c'),
                                          '-o', str(target / 'tiers-cpu-wrapper')])
                self.assertTrue((self.root / 'commands.disk').exists())
        self.env['FIXTURE_TARGET'] = str(self.root / 'failed target')
        self.env['FIXTURE_FAIL_BUILD'] = '1'
        result = self.invoke('raise Exception("must not run")', suite='failed')
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn('CPU wrapper build: exit 9', result.stderr)

    def test_command_summary_requires_complete_expected_inventory(self):
        self.directory.mkdir()
        self.assertEqual(self.summary('--expect', 'missing').returncode, 2)
        self.assertIn('missing', self.summary('--expect', 'missing').stderr)
        self.assertEqual(self.summary().returncode, 2)
        result = self.invoke('pass')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.summary('--expect', 'tools/fixture.py', '--expect', 'missing').returncode, 2)
        self.assertEqual(self.summary('--expect', 'other').returncode, 2)
        path, item = self.record()
        for changes in ({'complete': False}, {'user_s': -1}, {'attempt': 2},
                        {'profile': 'default'}, {'test_name': 'other'}, {'settling': True},
                        {'starved': 1}, {'starved': None}):
            with self.subTest(changes=changes):
                CPU['save'](path, dict(item, **changes))
                self.assertEqual(self.summary('--expect', 'tools/fixture.py').returncode, 2)
        CPU['save'](path, item)
        CPU['save'](self.directory / 'duplicate.json', item)
        self.assertEqual(self.summary('--expect', 'tools/fixture.py').returncode, 2)

    def test_command_summary_tools_budgets_and_multiple_suites(self):
        self.assertEqual(self.invoke('pass').returncode, 0)
        path, item = self.record()
        CPU['save'](path, dict(item, user_s=1.0, system_s=0.5))
        self.assertEqual(self.invoke('pass', suite='tools/second.py').returncode, 0)
        budget = self.root / 'budgets.toml'
        arguments = ['--expect', 'tools/fixture.py', '--expect', 'tools/second.py',
                     '--budgets', str(budget), '--json']
        budget.write_text('[tools."tools/fixture.py"]\ncpu_suite_s=1.5\ncpu_test_s=1.5\n')
        self.assertIn('tools', CPU['budgets'](budget, {'qa-jig'}))
        result = self.summary(*arguments)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)['suite']['tests'], 2)
        budget.write_text('[tools."tools/fixture.py"]\ncpu_suite_s=1.0\ncpu_test_s=1.0\n')
        result = self.summary(*arguments)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(len(json.loads(result.stdout)['over_budget']), 2)
        for content in ('tools=1', '[tools]\ncpu_suite_s=1',
                        '[tools."tools/fixture.py"]\ncpu_suite_s=nan',
                        '[tools."tools/fixture.py"]\nunknown=1'):
            with self.subTest(content=content):
                budget.write_text(content)
                self.assertEqual(self.summary(*arguments).returncode, 2)


if __name__ == '__main__':
    os.umask(0o077)
    unittest.main()
