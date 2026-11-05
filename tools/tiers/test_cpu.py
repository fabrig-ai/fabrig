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

# Recorded from nextest's cancellation output (tools/tiers/design.md, 7.4).
GLOBAL_TIMEOUT_LINE = '  Cancelling due to global timeout: 1 test still running\n'


def clean_env(**fields):
    env = dict(os.environ)
    for key in ('TIERS_WRAPPER', 'TIERS_DISK_BYTES'):
        env.pop(key, None)
    env.update(fields)
    return env


def abort(signal_number, name, kind='test abort', *, message=True):
    text = f'process aborted with signal {signal_number} ({name})'
    attribute = f' message="{text}"' if message else ''
    return f'<failure type="{kind}"{attribute}>{text}</failure>'


def setUpModule():
    global WRAPPER, SETTLEMENT_WRAPPER, COLLISION_WRAPPER
    native = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-cpu-native-')
    unittest.addModuleCleanup(native.cleanup)
    WRAPPER = Path(native.name) / 'tiers-cpu-wrapper'
    SETTLEMENT_WRAPPER = Path(native.name) / 'cpu-settlement-fixture'
    COLLISION_WRAPPER = Path(native.name) / 'cpu-collision-fixture'
    compiler = ['cc', '-std=c11', '-O2', '-static', '-Wall', '-Wextra', '-Werror',
                str(HERE / 'cpu-wrapper.c')]
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    for arguments in (['-o', str(WRAPPER)],
                      [str(HERE / 'fixtures/cpu-settlement.c'),
                       '-Wl,--wrap=wait4,--wrap=clock_gettime,--wrap=open,--wrap=read,--wrap=lseek,--wrap=kill,--wrap=write',
                       '-o', str(SETTLEMENT_WRAPPER)],
                      [str(HERE / 'fixtures/cpu-collision.c'), '-Wl,--wrap=mkstemps',
                       '-o', str(COLLISION_WRAPPER)]):
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

    def test_gate_budget_validation(self):
        path = self.root / 'budgets.toml'
        for pairs in (None, {'qa-jig'}):
            for tree, profile in (('hearth', 'default'), ('new-tree', 'new-profile')):
                with self.subTest(pairs=pairs, tree=tree, profile=profile):
                    path.write_text(f'[gate.{tree}.{profile}]\ncpu_suite_s=30')
                    self.assertEqual(CPU['budgets'](path, pairs),
                                     {'gate': {tree: {profile: {'cpu_suite_s': 30}}}})
            for content in ('gate=1', '[gate]\ncpu_suite_s=1',
                            '[gate.hearth]\ncpu_suite_s=1',
                            '[gate.hearth.default]\nunknown=1',
                            '[gate.hearth.default]\ncpu_suite_s=nan',
                            '[gate.hearth.default]\nauto_reap=[]',
                            '[gate."".default]\ncpu_suite_s=30',
                            '[gate.hearth.""]\ncpu_suite_s=30'):
                with self.subTest(pairs=pairs, content=content):
                    path.write_text(content)
                    with self.assertRaises(ValueError):
                        CPU['budgets'](path, pairs)

    def test_auto_reap_budget_declaration_warns_without_refusal(self):
        path = self.root / 'budgets.toml'
        self.record()
        for names in ('[]', '["one"]'):
            with self.subTest(names=names):
                path.write_text(f'[qa-jig]\nauto_reap={names}\n')
                limits = CPU['budgets'](path, {'qa-jig'})
                self.assertEqual(limits['qa-jig']['auto_reap'], json.loads(names))
                result = self.cli('--budgets', str(path))
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual('warning:' in result.stdout, names != '[]')
        self.record(complete=False)
        self.assertEqual(self.cli('--budgets', str(path)).returncode, 2)
        for names in ('"one"', '[1]', '[""]', 'true'):
            with self.subTest(names=names):
                path.write_text(f'[qa-jig]\nauto_reap={names}\n')
                with self.assertRaises(ValueError):
                    CPU['budgets'](path, {'qa-jig'})

    def execute_driver(self, command, *, limits=None, grace=0.15):
        driver = self.root / 'driver.py'
        driver.write_text(
            'import os,runpy,sys\n'
            f"cpu=runpy.run_path({str(HERE / 'cpu.py')!r})\n"
            f"cpu['execute'].__globals__['GRACE_S']={grace!r}\n"
            'try:\n'
            f"    code=cpu['execute']({command!r},os.environ,{str(self.directory)!r},"
            f"'qa-jig',{limits or {}!r})\n"
            'except KeyboardInterrupt:\n'
            f"    open({str(self.root / 'interrupted')!r},'w').close()\n"
            '    raise\n'
            'sys.exit(code)\n')
        return [sys.executable, '-I', '-B', str(driver)]

    def command_tree(self, *, ignore=False, escape=False, fault=False):
        (self.directory / 'junit.xml').unlink(missing_ok=True)
        child_pid = self.root / 'child-pid'
        command_pid = self.root / 'command-pid'
        terminated = self.root / 'terminated'
        child = ("import os,signal,time\nfrom pathlib import Path\n"
                 f"signal.signal(signal.SIGTERM, {int(signal.SIG_IGN if ignore else signal.SIG_DFL)})\n"
                 f"Path({str(child_pid)!r}).write_text(str(os.getpid()))\n"
                 "while True: time.sleep(1)\n")
        command = self.root / 'command.py'
        item = dict(self.item, pending=True, complete=False, exit_code=None)
        command.write_text(
            'import json,os,signal,subprocess,sys,time\nfrom pathlib import Path\n'
            'def term(signum, frame):\n'
            f"    Path({str(terminated)!r}).touch()\n"
            "    print('during grace',flush=True)\n"
            + ('signal.signal(signal.SIGTERM,term)\n' if ignore else '')
            + f"child=subprocess.Popen([sys.executable,'-I','-c',{child!r}],"
            + ('start_new_session=True' if escape else f'process_group={0 if ignore else None!r}')
            + ')\n'
            f"while not Path({str(child_pid)!r}).exists(): time.sleep(0.001)\n"
            f"Path({str(command_pid)!r}).write_text(str(os.getpid()))\n"
            "print('before cancellation',flush=True)\n"
            f"item={item!r}\nitem['start_s']=time.monotonic()\n"
            f"record=Path({str(self.directory / 'records/cpu-one')!r})\n"
            "temporary=record.with_suffix('.tmp')\n"
            + ("temporary.write_text('not json')\n" if fault else "temporary.write_text(json.dumps(item)+'\\n')\n")
            + 'temporary.replace(record)\n'
            'while True: time.sleep(1)\n')
        def cleanup():
            for path in (command_pid, child_pid):
                if path.exists():
                    try:
                        os.kill(int(path.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass
        self.addCleanup(cleanup)
        return [sys.executable, '-I', '-B', str(command)]

    def assert_tree_stopped(self):
        for name in ('command-pid', 'child-pid'):
            pid = int((self.root / name).read_text())
            try:
                stat = Path(f'/proc/{pid}/stat').read_bytes().rsplit(b')', 1)[1].split()
            except FileNotFoundError:
                continue
            self.assertEqual(stat[0], b'Z', f'fixture process {pid} still alive')

    def run_driver(self, command, **fields):
        return subprocess.run(self.execute_driver(command, **fields), env=clean_env(),
                              stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=5)

    def test_hard_wall_terminates_the_group_and_stays_green(self):
        result = self.run_driver(self.command_tree(), limits={'qa-jig': {'wall_hard_s': 0.05}})
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assert_tree_stopped()
        run = json.loads((self.directory / 'run.json').read_text())
        self.assertTrue(run['starved'])
        self.assertEqual(run['returncode'], -signal.SIGTERM)
        self.assertIn('cancel_code', run)
        self.assertIsNone(run['cancel_code'])
        self.assertIn(b'before cancellation', (self.directory / 'command-output.txt').read_bytes())
        summary = CPU['summary'](self.directory)
        self.assertEqual(summary['suite']['incomplete'], 1)
        self.assertEqual(summary['suite']['starved'], 1)
        self.assertTrue(summary['suite']['starved_run'])

    def test_hard_wall_kills_a_command_and_descendants_after_the_grace(self):
        result = self.run_driver(self.command_tree(ignore=True),
                                 limits={'qa-jig': {'wall_hard_s': 0.05}})
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assert_tree_stopped()
        run = json.loads((self.directory / 'run.json').read_text())
        self.assertEqual(run['returncode'], -signal.SIGKILL)
        self.assertIsNone(run['cancel_code'])
        output = (self.directory / 'command-output.txt').read_text()
        self.assertIn('before cancellation', output)
        self.assertIn('during grace', output)

    def test_hard_wall_ends_though_a_descendant_holds_stdout(self):
        result = self.run_driver(self.command_tree(ignore=True, escape=True),
                                 limits={'qa-jig': {'wall_hard_s': 0.05}})
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        child = int((self.root / 'child-pid').read_text())
        self.assertEqual(os.getsid(child), child)
        self.assertIn('during grace', (self.directory / 'command-output.txt').read_text())

    def test_interruption_stops_the_command_and_descendants(self):
        for second in (False, True):
            with self.subTest(second=second):
                for name in ('command-pid', 'child-pid', 'terminated', 'interrupted'):
                    (self.root / name).unlink(missing_ok=True)
                process = subprocess.Popen(
                    self.execute_driver(self.command_tree(ignore=True), grace=2 if second else 0.15),
                    env=clean_env(), stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                self.addCleanup(lambda p=process: p.poll() is None and p.kill())
                self.wait_until(lambda: (self.root / 'command-pid').exists())
                process.send_signal(signal.SIGINT)
                self.wait_until(lambda: (self.root / 'terminated').exists())
                if second:
                    process.send_signal(signal.SIGINT)
                stdout, stderr = process.communicate(timeout=1.5)
                self.assertNotEqual(process.returncode, 0, stdout + stderr)
                self.assertIn(b'KeyboardInterrupt', stderr)
                self.assertTrue((self.root / 'interrupted').exists())
                self.assert_tree_stopped()

    def test_collector_fault_stops_the_command_and_descendants(self):
        result = self.run_driver(self.command_tree(ignore=True, fault=True))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('JSONDecodeError', result.stderr)
        self.assert_tree_stopped()

    def test_timeout_is_green_and_other_failures_and_budgets_remain_red(self):
        driver = self.root / 'verdict.py'
        driver.write_text('import os,runpy,sys\n'
                          f'cpu=runpy.run_path({str(HERE / "cpu.py")!r})\n'
                          f"sys.exit(cpu['execute']([sys.executable, '-c', 'import sys; sys.exit(100)'], "
                          f"os.environ, {str(self.directory)!r}, 'qa-jig', "
                          "{'qa-jig': {'cpu_suite_s': float(sys.argv[1])}} if len(sys.argv)>1 else {}))\n")
        def invoke(*args):
            return subprocess.run([sys.executable, '-I', '-B', str(driver), *args],
                                  env=clean_env(), stdin=subprocess.DEVNULL,
                                  capture_output=True, text=True, timeout=5)
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
        command = [sys.executable, '-I', '-c', 'import time; time.sleep(60)']
        result = self.run_driver(command, limits={'qa-jig': {'wall_hard_s': 0.001}})
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)

    def test_recorded_cancellation_abort_is_starved(self):
        self.xml(f'<testcase name="one">{abort(15, "SIGTERM")}</testcase>')
        for fields in ({'exit_code': None, 'signal': 15},
                       {'exit_code': None, 'signal': 9},
                       {'exit_code': None, 'pending': True, 'complete': False}):
            with self.subTest(fields=fields):
                self.starved(junit=True)
                self.record(**fields)
                result = CPU['summary'](self.directory)
                self.assertEqual(result['failures'], [])
                self.assertEqual(result['tests'][0]['starved'], 1)
                self.assert_cancelled_verdict(0)
                (self.directory / 'records/cpu-running').unlink()
                self.finished()
                if fields.get('pending'):
                    with self.assertRaisesRegex(ValueError, 'unfinished attempt'):
                        CPU['summary'](self.directory)
                else:
                    self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])
        self.starved(junit=True)
        for fields in ({'exit_code': 7}, {'exit_code': None, 'signal': 11}):
            with self.subTest(fields=fields):
                self.record(**fields)
                self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])
                self.assert_cancelled_verdict(100)
        (self.directory / 'records/cpu-running').unlink()
        self.record(exit_code=7)
        self.record('cpu-retry', attempt=2, exit_code=None, signal=15)
        self.xml('<testcase name="one"><failure type="test failure"/>'
                 f'<rerunFailure type="test abort">process aborted with signal 15 (SIGTERM)</rerunFailure>'
                 '</testcase>')
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite one'])
        self.assert_cancelled_verdict(100)

    def test_global_timeout_is_starvation(self):
        output = self.directory / 'command-output.txt'
        for line in (GLOBAL_TIMEOUT_LINE, '\x1b[1;33m' + GLOBAL_TIMEOUT_LINE.rstrip() + '\x1b[0m\n'):
            for case in ('pending', 'unobserved', 'recorded', 'empty'):
                with self.subTest(line=line, case=case):
                    for path in (self.directory / 'records').iterdir():
                        path.unlink()
                    self.finished()
                    if case == 'empty':
                        self.xml('')
                    elif case == 'pending':
                        self.record(pending=True, complete=False, exit_code=None)
                        self.xml('<testcase name="one"/>')
                    else:
                        if case == 'recorded':
                            self.record(exit_code=None, signal=15)
                        self.xml(f'<testcase name="one">{abort(15, "SIGTERM")}</testcase>')
                    output.write_text(line)
                    before = (self.directory / 'run.json').read_bytes()
                    result = CPU['summary'](self.directory)
                    self.assertTrue(result['suite']['starved_run'])
                    self.assertEqual(result['verdict'], 'starved')
                    self.assertEqual(result['failures'], [])
                    self.assertEqual(result['suite']['unobserved'], int(case == 'unobserved'))
                    self.assertEqual(result['suite']['starved'], int(case in ('pending', 'recorded')))
                    cli = self.cli()
                    self.assertEqual(cli.returncode, 0, cli.stderr)
                    self.assertEqual((self.directory / 'run.json').read_bytes(), before)
                    command = [sys.executable, '-I', '-c', f'import sys; print({line!r},end=""); sys.exit(100)']
                    execution = self.run_driver(command)
                    self.assertEqual(execution.returncode, 0, execution.stdout + execution.stderr)
                    self.assertFalse(json.loads((self.directory / 'run.json').read_text())['starved'])
                    self.assertIsNone(json.loads((self.directory / 'run.json').read_text())['cancel_code'])
                    if case in ('pending', 'unobserved'):
                        for other in (None, '  Cancelling due to test failure\n',
                                      'prefix Cancelling due to global timeout\n',
                                      '  Cancelling due to global timeouts\n'):
                            if other is None:
                                output.unlink(missing_ok=True)
                            else:
                                output.write_text(other)
                            invalid = self.cli()
                            self.assertEqual(invalid.returncode, 2, invalid.stdout + invalid.stderr)

    def test_cancel_with_junit_preserves_extra_failed_record(self):
        self.starved(junit=True)
        self.record()
        self.record('cpu-extra', test_name='extra', exit_code=7)
        self.assertEqual(CPU['summary'](self.directory)['failures'], ['fixture::suite extra'])
        self.assert_cancelled_verdict(100)
        self.record('cpu-extra-pass', test_name='extra', attempt=2)
        self.assertEqual(CPU['summary'](self.directory)['failures'], [])
        self.assert_cancelled_verdict(0)

    def test_starved_cancellation_abort_without_record_is_unobserved(self):
        self.starved(junit=True)
        self.record()
        warning = ('qa-jig fixture::suite two: attempt 1 unobserved, '
                   'cancelled before its record')
        for element in (abort(15, 'SIGTERM'), abort(9, 'SIGKILL'),
                        abort(15, 'SIGTERM', 'test abort (leaked handles)'),
                        abort(15, 'SIGTERM', message=False),
                        '<failure type="TEST ABORT" '
                        'message="process aborted with signal 15 (SIGTERM)"/>',
                        '<failure type="test abort" message="2 errors occurred executing test">'
                        '* process aborted with signal 15 (SIGTERM)\n* leaked handles</failure>'):
            with self.subTest(element=element):
                self.xml(f'<testcase name="one"/><testcase name="two">{element}</testcase>')
                result = CPU['summary'](self.directory)
                self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite',
                                                            test_name='two', attempt=1)])
                self.assertEqual(result['suite']['unobserved'], 1)
                self.assertEqual(result['failures'], [])
                self.assertEqual([test['test_name'] for test in result['tests']], ['one'])
                self.assertEqual((result['suite']['attempts'], result['suite']['tests']), (1, 1))
                self.assertEqual(result['verdict'], 'starved')
                self.assertEqual(result['warnings'], [warning])
                command = self.cli()
                self.assertEqual(command.returncode, 0, command.stderr)
                self.assertIn('starved_run True unobserved 1 leaked', command.stdout)
                self.assertIn(f'warning: {warning}\n', command.stdout)
        result = CPU['summary'](self.directory, {'qa-jig': {'cpu_suite_s': 1}})
        self.assertEqual(result['verdict'], 'over budget')
        self.assertEqual(result['suite']['unobserved'], 1)
        element = abort(15, 'SIGTERM')
        self.xml(f'<testcase name="one"/><testcase name="zero">{element}</testcase>'
                 f'<testcase name="two">{element}</testcase>')
        result = CPU['summary'](self.directory)
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite',
                                                    test_name=name, attempt=1)
                                                for name in ('two', 'zero')])
        self.assertEqual(result['suite']['unobserved'], 2)
        self.assertEqual(result['warnings'], [warning, warning.replace(' two:', ' zero:')])
        self.xml(f'<testcase name="one"/><testcase name="two">{element}</testcase>')
        self.assert_cancelled_verdict(0)

    def test_cancellation_abort_without_record_fails_coverage_when_not_starved(self):
        self.record()
        result = CPU['summary'](self.directory)
        self.assertEqual(result['unobserved'], [])
        self.assertEqual(result['suite']['unobserved'], 0)
        self.xml(f'<testcase name="one"/><testcase name="two">{abort(15, "SIGTERM")}</testcase>')
        result = self.cli()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("record coverage: missing [('qa-jig', 'fixture::suite', 'two', 1)]",
                      result.stderr)

    def test_starved_run_missing_other_records_fails_coverage(self):
        self.starved(junit=True)
        self.record()
        for case in ('<testcase name="two"/>',
                     '<testcase name="two"><failure type="test failure with exit code 101" '
                     'message="thread \'two\' panicked"/></testcase>',
                     '<testcase name="two"><failure type="test failure with exit code 7"/></testcase>',
                     f'<testcase name="two">{abort(11, "SIGSEGV")}</testcase>',
                     f'<testcase name="two">{abort(90, "OTHER")}</testcase>',
                     f'<testcase name="two">{abort(150, "OTHER")}</testcase>',
                     f'<testcase name="two">{abort(15, "SIGTERM", "test failure")}</testcase>',
                     '<testcase name="two"><failure type="test abort"/></testcase>',
                     '<testcase name="two"><failure type="test timeout"/></testcase>'):
            with self.subTest(case=case):
                self.xml('<testcase name="one"/>' + case)
                result = self.cli()
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertIn('record coverage: missing', result.stderr)

    def test_unobserved_retry_keeps_the_observed_failure(self):
        self.starved(junit=True)
        self.record(exit_code=7)
        self.xml('<testcase name="one"><failure type="test failure with exit code 7"/>'
                 '<rerunFailure type="test abort" '
                 'message="process aborted with signal 15 (SIGTERM)"/></testcase>')
        result = CPU['summary'](self.directory)
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite',
                                                    test_name='one', attempt=2)])
        self.assertEqual(result['failures'], ['fixture::suite one'])
        self.assertEqual((result['suite']['attempts'], result['suite']['tests']), (1, 1))
        self.assert_cancelled_verdict(100)
        (self.directory / 'records/cpu-one').unlink()
        self.record('cpu-pass', attempt=2)
        self.xml('<testcase name="one"><flakyFailure type="test abort" '
                 'message="process aborted with signal 9 (SIGKILL)"/></testcase>')
        result = CPU['summary'](self.directory)
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite',
                                                    test_name='one', attempt=1)])
        self.assertEqual(result['failures'], [])

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
        env = clean_env(NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
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
        env = clean_env(NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                   NEXTEST_TEST_NAME='one', NEXTEST_ATTEMPT='1', TIERS_CLEANUP_MARKER=str(marker))
        result = subprocess.run([str(SETTLEMENT_WRAPPER), str(self.directory / 'records'), '/bin/true'],
                                env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        item = CPU['records'](self.directory)[0]
        self.assertFalse(item['complete'], 'continuous successful reaps bypassed the deadline')
        self.assertLess(item['wall_s'], 1.1)
        self.assertTrue(marker.exists(), 'continuous successful reaps bypassed cleanup')

    def wrapped(self, code, *, name='one', input_text=None):
        env = clean_env(NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
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

    def test_wrapper_preserves_published_record_on_basename_collision(self):
        self.record('cpu-AAAAAA')
        original = (self.directory / 'records/cpu-AAAAAA').read_bytes()
        env = clean_env(NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
                        NEXTEST_TEST_NAME='two', NEXTEST_ATTEMPT='1')
        result = subprocess.run([str(COLLISION_WRAPPER), str(self.directory / 'records'), '/bin/true'],
                                env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.directory / 'records/cpu-AAAAAA').read_bytes(), original)
        self.xml('<testcase name="one"/><testcase name="two"/>')
        summary = CPU['summary'](self.directory)
        self.assertEqual(summary['suite']['attempts'], 2)
        self.assertEqual(summary['failures'], [])
        self.assertEqual(list((self.directory / 'records').glob('*.tmp')), [])

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
            env = clean_env(NEXTEST_PROFILE='qa-jig', NEXTEST_BINARY_ID='fixture::suite',
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
        self.env = clean_env(PATH=f"{self.bin}:{os.environ['PATH']}",
                        FIXTURE_TARGET=str(WRAPPER.parent), FIXTURE_WRAPPER=str(WRAPPER),
                        FIXTURE_COMMAND_LOG=str(self.root / 'commands'),
                        NEXTEST_PROFILE='unchanged')
        self.fake('cargo', "import json,os,sys\n"
                  "assert sys.argv[1:]==['metadata','--format-version','1','--no-deps']\n"
                  "print(json.dumps({'target_directory':os.environ['FIXTURE_TARGET']}))\n")
        for key in ('TIERS_WRAPPER', 'TIERS_DISK_BYTES'):
            self.env.pop(key, None)
        self.env.update(TIERS_WRAPPER='wrap', TIERS_DISK_BYTES='80000000000')
        build = ("import json,os,shutil,sys\nfrom pathlib import Path\n"
                  "Path(os.environ['FIXTURE_COMMAND_LOG']).write_text(json.dumps(sys.argv[1:]))\n"
                  "if os.environ.get('FIXTURE_FAIL_BUILD'): sys.exit(9)\n"
                  "shutil.copyfile(os.environ['FIXTURE_WRAPPER'],sys.argv[-1])\n"
                  "Path(sys.argv[-1]).chmod(0o700)\n")
        self.fake('wrap', build)
        self.fake('cc', build)
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

    def test_collector_write_failure_cleans_payload_and_escaped_child(self):
        ready = self.root / 'owned-pids'
        errors = self.root / 'collector-errors'
        self.directory.mkdir()
        output = errors.open('wb')
        self.addCleanup(output.close)
        process = subprocess.Popen([sys.executable, '-I', '-B', str(HERE / 'cpu-run'),
                                    '--out', str(self.directory), '--suite', 'write-failed',
                                    '--wall-hard', '0.25', '--', sys.executable, '-I', '-c',
                                    f'''import os,signal,time
from pathlib import Path
signal.signal(signal.SIGTERM,signal.SIG_IGN)
parent=os.getpid()
child=os.fork()
if child==0:
    os.setsid()
    Path({str(ready)!r}).write_text(str(parent)+' '+str(os.getpid()))
    Path({str(self.directory)!r}).chmod(0o500)
time.sleep(30)
'''], env=self.env, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=output, start_new_session=True)
        def cleanup():
            self.directory.chmod(0o700)
            if ready.exists():
                for pid in map(int, ready.read_text().split()):
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=5)
        self.addCleanup(cleanup)
        self.assertEqual(process.wait(timeout=5), 125)
        self.assertIn('write record', errors.read_text())
        pids = [int(value) for value in ready.read_text().split()]
        self.assertEqual(len(pids), 2)
        for pid in pids:
            with self.assertRaises(ProcessLookupError, msg=f'owned process {pid} survived collector failure'):
                os.kill(pid, 0)
        _, item = self.record()
        self.assertFalse(item['complete'])
        self.assertEqual(self.summary('--expect', 'write-failed').returncode, 2)

    def test_native_writer_retains_default_sigpipe(self):
        reader, writer = os.pipe()
        os.close(reader)
        try:
            result = subprocess.run([sys.executable, '-I', '-B', str(HERE / 'cpu-run'),
                                     '--out', str(self.directory), '--suite', 'pipe', '--', 'yes'],
                                    env=self.env, stdin=subprocess.DEVNULL, stdout=writer,
                                    stderr=subprocess.PIPE, timeout=5)
        finally:
            os.close(writer)
        self.assertEqual(result.returncode, -signal.SIGPIPE, result.stderr)
        _, item = self.record()
        self.assertEqual(item['signal'], signal.SIGPIPE)
        self.assertIsNone(item['exit_code'])
        self.assertTrue(item['complete'])

    def test_descriptor_exhaustion_cleanup_reaps_escaped_child(self):
        ready = self.root / 'exhausted-pids'
        errors = self.root / 'exhausted-errors'
        output = errors.open('wb')
        self.addCleanup(output.close)
        process = subprocess.Popen([sys.executable, '-I', '-B', str(HERE / 'cpu-run'),
                                    '--out', str(self.directory), '--suite', 'exhausted',
                                    '--wall-hard', '1', '--', sys.executable, '-I', '-c',
                                    f'''import os,signal,time
from pathlib import Path
signal.signal(signal.SIGTERM,signal.SIG_IGN)
parent=os.getpid()
if os.fork()==0:
    os.setsid()
    Path({str(ready)!r}).write_text(str(parent)+' '+str(os.getpid()))
time.sleep(30)
'''], env=self.env, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=output, start_new_session=True)
        def cleanup():
            if ready.exists():
                for pid in map(int, ready.read_text().split()):
                    try:
                        os.kill(pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=5)
        self.addCleanup(cleanup)
        deadline = time.monotonic() + 5
        while not ready.exists():
            if time.monotonic() > deadline or process.poll() is not None:
                self.fail('owned payload did not become ready')
            time.sleep(0.001)
        _, hard = resource.prlimit(process.pid, resource.RLIMIT_NOFILE)
        resource.prlimit(process.pid, resource.RLIMIT_NOFILE, (0, hard))
        self.assertEqual(process.wait(timeout=8), 125)
        self.assertIn('Too many open files', errors.read_text())
        pids = [int(value) for value in ready.read_text().split()]
        self.assertEqual(len(pids), 2)
        for pid in pids:
            with self.assertRaises(ProcessLookupError, msg=f'owned process {pid} survived descriptor exhaustion'):
                os.kill(pid, 0)
        _, item = self.record()
        self.assertFalse(item['complete'])
        self.assertEqual(self.summary('--expect', 'exhausted').returncode, 2)

    def test_deadline_stays_armed_during_settlement_with_known_exit(self):
        self.directory.mkdir()
        record = self.directory / 'settlement.json'
        marker = self.root / 'cleanup'
        trace = self.root / 'records-trace'
        result = subprocess.run([str(SETTLEMENT_WRAPPER), '--suite', str(record), 'settlement',
                                 '--wall-hard', '0.5', '--', '/bin/sh', '-c', 'exit 7'],
                                env=dict(self.env, TIERS_CLEANUP_MARKER=str(marker),
                                         TIERS_RECORD_TRACE=str(trace)),
                                stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 7, result.stderr)
        item = json.loads(record.read_text())
        self.assertTrue(item['starved'], 'settlement crossed the deadline without starvation')
        self.assertEqual(item['exit_code'], 7)
        self.assertIsNone(item['signal'])
        self.assertFalse(item['pending'])
        self.assertFalse(item['settling'])
        self.assertTrue(item['complete'])
        checkpoints = [json.loads(line) for line in trace.read_text().splitlines()]
        checkpoint = next(r for r in checkpoints if r['starved'] and not r['complete'])
        self.assertEqual(checkpoint['exit_code'], 7)
        self.assertIsNone(checkpoint['signal'])
        self.assertFalse(checkpoint['pending'])
        self.assertTrue(checkpoint['settling'])

    def test_unproven_cleanup_names_surviving_pid(self):
        self.directory.mkdir()
        ready = self.root / 'refused-pid'
        errors = self.root / 'refused-errors'
        output = errors.open('wb')
        self.addCleanup(output.close)
        process = subprocess.Popen([str(SETTLEMENT_WRAPPER), '--suite',
                                    str(self.directory / 'refused.json'), 'refused',
                                    '--wall-hard', '0.25', '--', sys.executable, '-I', '-c',
                                    'import os,signal,time; from pathlib import Path; '
                                    'signal.signal(signal.SIGTERM,signal.SIG_IGN); '
                                    f'Path({str(ready)!r}).write_text(str(os.getpid())); time.sleep(30)'],
                                   env=dict(self.env, TIERS_REFUSE_KILL='1'),
                                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=output, start_new_session=True)
        def cleanup():
            if ready.exists():
                try:
                    os.kill(int(ready.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=5)
        self.addCleanup(cleanup)
        self.assertEqual(process.wait(timeout=8), 125)
        pid = int(ready.read_text())
        os.kill(pid, 0)
        self.assertIn(f'cpu-wrapper: surviving pids: {pid}\n', errors.read_text())
        _, item = self.record()
        self.assertFalse(item['complete'])
        self.assertEqual(self.summary('--expect', 'refused').returncode, 2)

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

    def test_command_build_settings_are_optional_and_disk_values_validated(self):
        for index, (prefix, minimum) in enumerate(((None, None), ("", ""),
                                                  ("wrap", None), (None, "0"))):
            with self.subTest(prefix=prefix, minimum=minimum):
                for key, value in (("TIERS_WRAPPER", prefix), ("TIERS_DISK_BYTES", minimum)):
                    self.env.pop(key, None)
                    if value is not None:
                        self.env[key] = value
                target = self.root / f"target {index}"
                self.env['FIXTURE_TARGET'] = str(target)
                (self.root / 'commands.disk').unlink(missing_ok=True)
                result = self.invoke('pass', suite=str(index))
                self.assertEqual(result.returncode, 0, result.stderr)
                command = json.loads((self.root / 'commands').read_text())
                self.assertEqual(command[:1], ['cc'] if prefix else ['-std=c11'])
                self.assertEqual((self.root / 'commands.disk').exists(), minimum == "0")
        for minimum in ('plenty', '-1'):
            self.env['TIERS_DISK_BYTES'] = minimum
            self.env['FIXTURE_TARGET'] = str(self.root / 'invalid disk target')
            (self.root / 'commands').unlink(missing_ok=True)
            (self.root / 'commands.disk').unlink(missing_ok=True)
            result = self.invoke('raise Exception("must not run")', suite=minimum)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertIn('TIERS_DISK_BYTES', result.stderr)
            self.assertFalse((self.root / 'commands').exists())
            self.assertFalse((self.root / 'commands.disk').exists())

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
        report = json.loads(result.stdout)
        self.assertEqual(report['suite']['tests'], 2)
        self.assertEqual(report['unobserved'], [])
        self.assertEqual(report['suite']['unobserved'], 0)
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
