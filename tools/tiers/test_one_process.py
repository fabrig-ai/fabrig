#!/usr/bin/env python3
"""Offline one-process runner proof (tools/tiers/design.md, 11.10)."""
import contextlib
import io
import json
import math
import os
from pathlib import Path
import resource
import re
import runpy
import shutil
import signal
import subprocess
import tempfile
import time
import unittest
from unittest import mock
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
FIXTURES = HERE / 'fixtures/one-process'


def setUpModule():
    global NATIVE
    native = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-one-native-')
    unittest.addModuleCleanup(native.cleanup)
    NATIVE = Path(native.name)
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    for name, source, flags in [('tests', 'tests.rs', []), ('before', 'tests.rs', ['--cfg', 'before_start']),
                                ('state', 'state.rs', []), ('again', 'again.rs', [])]:
        try:
            result = subprocess.run(['rustc', '--edition', '2024', '--test',
                                     str(FIXTURES / source), *flags, '-o', str(NATIVE / name)],
                                    stdin=subprocess.DEVNULL, capture_output=True, text=True)
        except FileNotFoundError as error:
            raise RuntimeError('one-process tests require rustc on PATH for native fixtures') from error
        if result.returncode:
            raise RuntimeError(f'native fixture build failed: {result.stdout}{result.stderr}')
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    user, system = after.ru_utime - before.ru_utime, after.ru_stime - before.ru_stime
    print(f'Native fixture build CPU: user {user:.6f}s, system {system:.6f}s, '
          f'total {user + system:.6f}s', flush=True)


class ProtocolTests(unittest.TestCase):
    def test_cancellation_keeps_result_read_before_mark(self):
        module = runpy.run_path(str(HERE / 'one_process.py'))
        process = module['BinaryProcess'].__new__(module['BinaryProcess'])
        process.job = dict(binary=dict(binary_id='fixture::suite'), kind='tests')
        process.index = 0
        process.output, process.log = bytearray(b'test a ... '), bytearray()
        process.announcement = re.compile(rb'(?m)^test (a) \.\.\. ')
        process.announcements = [('a', 0, len(process.output), 1)]
        process.logs, process.done = {}, {}
        process.reaped_at, process.cancelled_at, process.log_writer = None, None, None
        with mock.patch('os.read', return_value=b'ok a\n'), mock.patch('time.monotonic', return_value=2):
            process.read(0, 'log', None)
        process.cancelled_at = 3
        with mock.patch('os.read', return_value=b'ok\n\ntest result: ok\n'), \
                mock.patch('time.monotonic', return_value=4), contextlib.redirect_stdout(io.StringIO()):
            process.read(0, 'output', None)
        self.assertEqual([(t['test_name'], t['outcome'], t['wall_s']) for t in process.done.values()],
                         [('a', 'passed', 1)])

    def test_libtest_protocol(self):
        for scenario in ('pass', 'first-failure', 'starved', 'expected-panic'):
            logfile = NATIVE / 'protocol.log'
            name = 'expected_panic' if scenario == 'expected-panic' else 'a'
            result = subprocess.run([str(NATIVE / 'tests'), '--exact', '--nocapture',
                                     '--test-threads=1', '--logfile', str(logfile), name],
                                    env=dict(os.environ, FIXTURE_CASE=scenario),
                                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT, text=True, timeout=5)
            print(f'libtest {scenario}: {result.stdout!r}; log: {logfile.read_text()!r}', flush=True)
            self.assertIn(f'test {name}' + (' - should panic' if scenario == 'expected-panic' else '') + ' ... ', result.stdout)
            if scenario == 'starved':
                self.assertEqual(result.returncode, 0)
                self.assertEqual(logfile.read_text(), '')
                self.assertTrue(result.stdout.rstrip().endswith(
                    'skein-world: starved: waited 1 ms runnable, fixture patience'))
            else:
                self.assertIn(('ok ' if scenario in ('pass', 'expected-panic') else 'failed ') + name, logfile.read_text())


class RunnerFixture(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-one-test-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.directory = self.root / 'run with spaces'
        (self.root / '.config').mkdir()
        self.config = self.root / '.config/nextest.toml'
        self.config.write_text('[profile.default]\n')
        for name in ('cargo', 'prefix', 'hold'):
            shutil.copyfile(FIXTURES / f'{name}.py', self.root / name)
            (self.root / name).chmod(0o700)
        self.env = dict(os.environ, FIXTURE_CASE='pass', NEXTEST='1')
        self.env.pop('TIERS_HOLD', None)
        self.command = [str(self.root / 'prefix'), str(self.root / 'cargo'), 'nextest', 'run',
                        '-p', 'fixture', '--profile', 'default', '--no-fail-fast']
        self.module = runpy.run_path(str(HERE / 'one_process.py'))
        self.run = self.module['run']
        self.run.__globals__.update(LANES=1, TERM_GRACE_S=0.1, WALL_GRACE_S=0.5, LEAK_S=0.05)
        package = dict(id='fixture-id', name='fixture', manifest_path=str(self.root / 'Cargo.toml'),
                       version='1.2.3', authors=['A', 'B'])
        self.metadata = {'packages': [package], 'workspace_root': str(self.root)}
        self.env['CARGO_HOME'] = str(self.root / 'cargo-home')
        self.listing = {'rust-suites': {}, 'rust-build-meta': {
            'target-directory': str(self.root / 'target'), 'base-output-directories': ['debug'],
            'non-test-binaries': {'fixture-id': [{'name': 'helper', 'path': 'debug/helper', 'kind': 'bin-exe'},
                                               {'name': 'library', 'path': 'debug/library.so', 'kind': 'dylib'}]},
            'build-script-out-dirs': {'fixture-id': str(self.root / 'out')}, 'linked-paths': {}}}
        self.binary()

    def binary(self, binary_id='fixture::suite', names=('a', 'b', 'c'), native='tests'):
        self.listing['rust-suites'][binary_id] = dict(
            status='listed', **{'binary-id': binary_id, 'binary-path': str(NATIVE / native),
            'cwd': str(self.root), 'package-id': 'fixture-id', 'kind': 'test', 'binary-name': 'suite',
            'testcases': {name: {'ignored': name == 'ignored', 'filter-match': {'status': 'matches'}}
                          for name in names}})

    def prepare(self):
        (self.root / 'listing.json').write_text(json.dumps(self.listing) + '\n')
        (self.root / 'metadata.json').write_text(json.dumps(self.metadata) + '\n')

    def invoke(self, limits=None):
        self.prepare()
        with contextlib.redirect_stdout(io.StringIO()):
            self.run(self.command, self.env, self.directory, 'default', limits or {}, self.config)
        self.results = json.loads((self.directory / 'results.json').read_text())
        self.record = json.loads((self.directory / 'run.json').read_text())
        return self.results

    def invoke_bounded(self, limits, seconds=3):
        def expired(sig, frame):
            raise TimeoutError('runner did not finish cancellation')
        previous = signal.signal(signal.SIGALRM, expired)
        signal.setitimer(signal.ITIMER_REAL, seconds)
        try:
            return self.invoke(limits)
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, previous)


class EndToEndTests(RunnerFixture):
    def test_execute_outcomes(self):
        cpu = runpy.run_path(str(HERE / 'cpu.py'))
        execute = cpu['execute_one_process']
        # Keep the native fixture's lane count deterministic through the public call.
        execute.__globals__['runpy'] = mock.Mock(run_path=lambda path: {'run': self.run})
        rows = [('crash', 100), ('hang', 0), ('starved', 0),
                ('state', 100), ('split', 0), ('empty', 4)]
        for case, status in rows:
            with self.subTest(case=case):
                self.directory = self.root / case
                self.env['FIXTURE_CASE'] = case
                self.run.__globals__['LANES'] = 2 if case == 'split' else 1
                self.config.write_text('[profile.default]\n' +
                                       ('global-timeout="1s"\n' if case == 'hang' else ''))
                if case == 'state':
                    self.binary(names=('a_sets', 'b_reads'), native='state')
                else:
                    self.binary(names=() if case == 'empty' else ('a', 'b', 'c'))
                self.prepare()
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    code = execute(self.command, self.env, self.directory, 'default', {}, self.config)
                self.assertEqual(code, status)
                summary = cpu['summary'](self.directory)
                if case == 'hang':
                    self.assertEqual(summary['unobserved'], [dict(
                        binary_id='fixture::suite', test_name='c', attempt=1)])
                    self.assertIn('c: unobserved', output.getvalue())
                elif case == 'starved':
                    self.assertEqual(summary['verdict'], 'starved')
                elif case == 'state':
                    self.assertIn('may depend on an earlier test', output.getvalue())
                    self.assertEqual(next(t for t in summary['tests']
                                          if t['test_name'] == 'b_reads')['alone'], 'passed')
                elif case == 'split':
                    self.assertEqual([p['binary_id'] for p in summary['processes']],
                                     ['fixture::suite', 'fixture::suite'])
                self.assertIn(output.getvalue().splitlines()[-1],
                              (self.directory / 'command-output.txt').read_text())

    def test_command_empty_selection(self):
        self.binary(names=())
        self.prepare()
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'))
        result = subprocess.run([str(HERE / 'one-process'), '--out', str(self.directory),
                                 'default', '-p', 'fixture'], cwd=self.root, env=env,
                                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True, timeout=10)
        self.assertEqual(result.returncode, 4, result.stdout)
        self.assertEqual(json.loads((self.directory / 'results.json').read_text())['selected'], 0)


class AgainTests(RunnerFixture):
    def prepare_again(self):
        self.binary(names=('a_passes', 'b_fails', 'c_reads'), native='again')
        self.invoke()
        self.cargo_before = (self.root / 'cargo-argv.jsonl').read_bytes()
        self.times = self.root / 'target/tiers-one-process/times.json'
        self.times_before = self.times.read_bytes()
        self.prefix_before = (self.root / 'prefix-argv.jsonl').read_text().splitlines()
        self.again_directory = self.root / 'again output'

    def invoke_again(self, names, *, binary_id='fixture::suite', run_directory=None,
                     directory=None, default_out=False, hold=None, extra=()):
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'), NEXTEST_PROFILE='wrong',
                   CARGO_PKG_NAME='wrong', TIERS_ONE_PROCESS='wrong',
                   FIXTURE_HOLD_CWD=str(self.directory))
        if hold is not None:
            env['TIERS_HOLD'] = hold
        command = [str(HERE / 'one-process'), '--again', str(run_directory or self.directory)]
        if not default_out:
            command += ['--out', str(directory or self.again_directory)]
        return subprocess.run([*command, *extra, binary_id, *names], cwd=self.directory,
                              env=env, stdin=subprocess.DEVNULL, capture_output=True,
                              text=True, timeout=10)

    def test_again_passes_in_one_process_with_plan_environment(self):
        self.prepare_again()
        # A process-of-its-own group must not split an explicitly requested process.
        path = self.directory / 'plan.json'
        plan = json.loads(path.read_text())
        plan['groups']['process-of-its-own'] = 1
        plan['binaries'][0]['tests'][0]['group'] = 'process-of-its-own'
        path.write_text(json.dumps(plan))
        result = self.invoke_again(['c_reads', 'a_passes'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stdout.splitlines()[0], str(self.again_directory))
        results = json.loads((self.again_directory / 'results.json').read_text())
        self.assertEqual([(t['binary_id'], t['test_name'], t['outcome']) for t in results['tests']],
                         [('fixture::suite', 'a_passes', 'passed'),
                          ('fixture::suite', 'c_reads', 'passed')])
        self.assertEqual(len(results['processes']), 1)
        self.assertEqual(results['processes'][0]['tests'], ['a_passes', 'c_reads'])
        self.assertEqual(results['unobserved'], [])
        self.assertEqual({p.name for p in self.again_directory.iterdir()},
                         {'plan.json', 'run.json', 'results.json', 'junit.xml', 'command-output.txt'})
        record = json.loads((self.again_directory / 'run.json').read_text())
        self.assertEqual((record['mode'], record['lanes'], record['finished']), ('one-process', 1, True))
        again_plan = json.loads((self.again_directory / 'plan.json').read_text())
        self.assertEqual(again_plan['global_timeout'], plan['global_timeout'])
        for key in ('path', 'cwd', 'env', 'tests'):
            expected = plan['binaries'][0][key]
            if key == 'tests':
                expected = [t for t in expected if t['name'] != 'b_fails']
            self.assertEqual(again_plan['binaries'][0][key], expected)
        cases = ET.parse(self.again_directory / 'junit.xml').findall('.//testcase')
        self.assertEqual([c.get('name') for c in cases], ['a_passes', 'c_reads'])
        self.assertTrue(all(c.find('failure') is None for c in cases))
        output = (self.again_directory / 'command-output.txt').read_text()
        self.assertIn('PASS', output)
        self.assertIn(result.stdout.splitlines()[-1], output)
        self.assertEqual((self.root / 'cargo-argv.jsonl').read_bytes(), self.cargo_before)
        self.assertEqual(self.times.read_bytes(), self.times_before)
        calls = (self.root / 'prefix-argv.jsonl').read_text().splitlines()[len(self.prefix_before):]
        self.assertEqual(len(calls), 1)
        self.assertEqual(json.loads(calls[0])[-2:], ['lanes', str(self.again_directory / 'plan.json')])

    def test_again_failure_has_no_alone_diagnostic(self):
        self.prepare_again()
        result = self.invoke_again(['a_passes', 'b_fails'])
        self.assertEqual(result.returncode, 100, result.stdout + result.stderr)
        results = json.loads((self.again_directory / 'results.json').read_text())
        self.assertEqual([(t['test_name'], t['outcome'], t['alone']) for t in results['tests']],
                         [('a_passes', 'passed', None), ('b_fails', 'failed', None)])
        self.assertEqual([p['kind'] for p in results['processes']], ['tests'])
        failure = ET.parse(self.again_directory / 'junit.xml').find('.//failure')
        self.assertEqual(failure.get('type'), 'test failure')
        self.assertIn('again fixture failure', (self.again_directory / 'command-output.txt').read_text())
        self.assertEqual(self.times.read_bytes(), self.times_before)
        self.again_directory = self.root / 'pass alone'
        result = self.invoke_again(['a_passes'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_again_nonpasses_exit_100_without_fresh_process(self):
        for scenario, timeout, names, outcomes, unobserved in (
                ('starved', None, ['a'], ['starved'], []),
                ('crash', None, ['a', 'b', 'c'], ['passed', 'crashed'], ['c']),
                ('hang', 0.05, ['a', 'b', 'c'], ['passed', 'cancelled'], ['c'])):
            with self.subTest(scenario=scenario):
                self.directory = self.root / scenario
                self.env['FIXTURE_CASE'] = 'pass'
                self.invoke()
                path = self.directory / 'plan.json'
                plan = json.loads(path.read_text())
                plan['global_timeout'] = timeout
                plan['binaries'][0]['env']['FIXTURE_CASE'] = scenario
                path.write_text(json.dumps(plan))
                self.again_directory = self.root / f'again {scenario}'
                result = self.invoke_again(names)
                self.assertEqual(result.returncode, 100, result.stdout + result.stderr)
                results = json.loads((self.again_directory / 'results.json').read_text())
                self.assertEqual([t['outcome'] for t in results['tests']], outcomes)
                self.assertEqual([t['test_name'] for t in results['unobserved']], unobserved)
                self.assertEqual(len(results['processes']), 1)

    def test_again_refuses_before_starting_anything(self):
        self.prepare_again()
        for label, binary_id, names, source, out, extra, message in (
                ('no plan', 'fixture::suite', ['a_passes'], self.root, None, (), 'plan.json'),
                ('binary', 'missing', ['a_passes'], None, None, (), 'no binary'),
                ('name', 'fixture::suite', ['missing'], None, None, (), 'not selected'),
                ('duplicate', 'fixture::suite', ['a_passes', 'a_passes'], None, None, (), 'twice'),
                ('no names', 'fixture::suite', [], None, None, (), 'requires test names'),
                ('not fresh', 'fixture::suite', ['a_passes'], None, self.directory, (), 'fresh'),
                ('budgets', 'fixture::suite', ['a_passes'], None, None,
                 ('--budgets', 'unused'), '--budgets')):
            with self.subTest(label=label):
                result = self.invoke_again(names, binary_id=binary_id, run_directory=source,
                                           directory=out, extra=extra, hold=str(self.root / 'hold'))
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertIn(message, result.stderr)
        self.assertFalse(self.again_directory.exists())
        self.assertEqual((self.root / 'cargo-argv.jsonl').read_bytes(), self.cargo_before)
        self.assertEqual((self.root / 'prefix-argv.jsonl').read_text().splitlines(), self.prefix_before)
        events = [json.loads(line) for line in (self.root / 'hold-events.jsonl').read_text().splitlines()]
        self.assertTrue(all(e[0] == 'cargo' for e in events))

    def test_again_default_output_and_hold_lifetime(self):
        self.prepare_again()
        events_path = self.root / 'hold-events.jsonl'
        for setting in (None, '', str(self.root / 'hold')):
            with self.subTest(hold=setting):
                events_path.write_text('')
                result = self.invoke_again(['a_passes'], default_out=True, hold=setting)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                directory = Path(result.stdout.splitlines()[0])
                self.assertEqual(directory.parent, self.times.parent / 'runs')
                self.assertTrue((directory / 'results.json').is_file())
                events = [json.loads(line) for line in events_path.read_text().splitlines()]
                if setting:
                    self.assertEqual(events, [['hold-start', [], os.getpgrp(), os.getsid(0)],
                                              ['hold-end', result.stdout.splitlines()[-1]]])
                else:
                    self.assertEqual(events, [])
                shutil.rmtree(directory)
        self.assertEqual((self.root / 'cargo-argv.jsonl').read_bytes(), self.cargo_before)
        self.assertEqual(self.times.read_bytes(), self.times_before)


class HoldTests(RunnerFixture):
    def prepare_command(self, setting, *, out=False):
        self.metadata['target_directory'] = str(self.root / 'target')
        self.binary(names=())
        self.prepare()
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'))
        env.pop('TIERS_HOLD', None)
        if setting is not None:
            env['TIERS_HOLD'] = setting
        command = [str(HERE / 'one-process')]
        if out:
            command += ['--out', str(self.directory)]
        return command + ['default', '-p', 'fixture'], env

    def events(self):
        path = self.root / 'hold-events.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_hold_before_metadata_until_after_summary(self):
        command, env = self.prepare_command(str(self.root / 'hold'))
        self.binary(names=('a',))
        self.prepare()
        result = subprocess.run(command, cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        events = self.events()
        self.assertEqual(events[0], ['hold-start', [], os.getpgrp(), os.getsid(0)])
        self.assertEqual(events[1][0:2], ['cargo', ['metadata', '--format-version', '1', '--no-deps']])
        self.assertEqual(events[-1], ['hold-end', result.stdout.splitlines()[-1]])
        self.assertEqual(sum(e[0] == 'hold-start' for e in events), 1)
        self.assertIn('default: 1 tests', result.stdout)

    def test_unset_or_empty_hold_starts_none(self):
        for setting in (None, ''):
            with self.subTest(setting=setting):
                command, env = self.prepare_command(setting, out=True)
                if self.directory.exists():
                    shutil.rmtree(self.directory)
                result = subprocess.run(command, cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                        capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 4, result.stderr)
                self.assertTrue(self.events())
                self.assertTrue(all(e[0] == 'cargo' for e in self.events()))

    def test_early_hold_exit_warns_and_continues(self):
        command, env = self.prepare_command(str(self.root / 'hold') + ' fail')
        result = subprocess.run(command, cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 4, result.stderr)
        self.assertIn('one-process: hold:', result.stderr)
        self.assertIn('; going on unheld', result.stderr)
        self.assertEqual(self.events()[0][0:2], ['hold-start', ['fail']])
        self.assertEqual(self.events()[1][0], 'cargo')
        self.assertIn('default: 0 tests', result.stdout)

    def test_sigkill_releases_hold_by_eof(self):
        command, env = self.prepare_command(str(self.root / 'hold'))
        env['FIXTURE_HOLD_BLOCK'] = '1'
        with subprocess.Popen(command, cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) as process:
            try:
                deadline = time.monotonic() + 5
                while not any(e[0] == 'cargo' for e in self.events()) and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertTrue(any(e[0] == 'cargo' for e in self.events()), 'cargo did not start')
                self.assertFalse(any(e[0] == 'hold-end' for e in self.events()))
                process.kill()
                process.wait(timeout=5)
                deadline = time.monotonic() + 5
                while not any(e[0] == 'hold-end' for e in self.events()) and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertEqual(self.events()[-1], ['hold-end', None])
            finally:
                process.kill()
                for event in self.events():
                    if event[0] == 'cargo':
                        try:
                            os.kill(event[2], signal.SIGKILL)
                        except ProcessLookupError:
                            pass


class RunnerTests(RunnerFixture):
    def test_passing_binary_and_prefix(self):
        self.binary(names=('a', 'b', 'c', 'expected_panic', 'ignored'))
        self.command.extend(['-Pdefault', '--status-level=fail', '--no-capture',
                             '--failure-output', 'immediate', '--color', 'never', '--retries=0'])
        result = self.invoke()
        self.assertEqual(result['selected'], 5)
        self.assertEqual([t['outcome'] for t in result['tests']], ['passed'] * 5)
        self.assertEqual(len(result['processes']), 1)
        process = result['processes'][0]
        self.assertEqual(process['kind'], 'tests')
        self.assertEqual(process['tests'], ['a', 'b', 'c', 'expected_panic', 'ignored'])
        for key in ('user_s', 'system_s'):
            self.assertTrue(math.isfinite(process[key]))
            self.assertGreaterEqual(process[key], 0)
        self.assertEqual((self.record['mode'], self.record['lanes']), ('one-process', 1))
        self.assertEqual({p.name for p in self.directory.iterdir()},
                         {'run.json', 'results.json', 'junit.xml', 'command-output.txt', 'plan.json'})
        self.assertEqual(self.directory.stat().st_mode & 0o777, 0o700)
        cases = ET.parse(self.directory / 'junit.xml').findall('.//testcase')
        self.assertEqual([(c.get('classname'), c.get('name')) for c in cases],
                         [('fixture::suite', n) for n in ('a', 'b', 'c', 'expected_panic', 'ignored')])
        calls = [json.loads(line) for line in (self.root / 'prefix-argv.jsonl').read_text().splitlines()]
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[0][1:], ['nextest', 'list', '--message-format', 'json', '--profile',
                                       'default', '--config-file', str(self.config.resolve()), '-p', 'fixture'])
        self.assertEqual(calls[1][1:3], ['-I', '-B'])
        self.assertIn('lanes', calls[1])
        output = (self.directory / 'command-output.txt').read_text()
        self.assertIn('fixture listing suffix', output)
        self.assertIn('PASS', output)
        self.assertNotIn('tiers-one-process: ', output)

    def test_private_configuration_reaches_the_listing(self):
        self.config = self.root / 'private.toml'
        self.config.write_text('[profile.default]\n')
        self.invoke()
        listing = json.loads((self.root / 'prefix-argv.jsonl').read_text().splitlines()[0])
        self.assertIn('--config-file', listing)
        self.assertEqual(listing[listing.index('--config-file') + 1], str(self.config.resolve()))

    def test_unterminated_output(self):
        self.env['FIXTURE_CASE'] = 'unterminated'
        self.assertEqual([t['outcome'] for t in self.invoke()['tests']], ['passed'] * 3)

    def test_descendant_descriptors(self):
        self.env.update(FIXTURE_CASE='descriptors',
                        FIXTURE_DESCRIPTOR_CHECK=str(FIXTURES / 'descriptors.py'))
        self.binary(names=('a',))
        result = self.invoke()
        self.assertEqual(result['tests'][0]['outcome'], 'passed', result['tests'][0]['output'])

    def test_crash_fresh_and_before_first(self):
        self.env['FIXTURE_CASE'] = 'crash'
        result = self.invoke()
        tests = {t['test_name']: t for t in result['tests']}
        self.assertEqual(tests['b']['outcome'], 'crashed')
        self.assertIn('crash output', tests['b']['output'])
        self.assertEqual(tests['b']['before'], ['a'])
        self.assertEqual(tests['b']['alone'], 'failed')
        self.assertIn('c', tests, 'the remaining test must run in a fresh process')
        self.assertEqual(tests['c']['outcome'], 'passed')
        self.assertEqual(result['processes'][tests['b']['process']]['signal'], 6)
        self.assertEqual(result['processes'][tests['c']['process']]['kind'], 'fresh')
        alone = [p for p in result['processes'] if p['kind'] == 'alone']
        self.assertEqual([p['tests'] for p in alone], [['b']])
        output = (self.directory / 'command-output.txt').read_text()
        self.assertIn('fixture::suite b: failed alone too', output)
        self.assertEqual(output.count('crash output'), 2)
        self.directory = self.root / 'before-run'
        self.binary(native='before')
        result = self.invoke()
        self.assertEqual([t['outcome'] for t in result['tests']], ['crashed'] * 3)
        self.assertEqual(len(result['processes']), 1)

    def test_cancellation_drops_waiting_processes(self):
        self.env['FIXTURE_CASE'] = 'hang'
        self.binary('fixture::second', names=('d',))
        for reason, timeout, hard in [('global-timeout', 'global-timeout="1s"\n', 3),
                                      ('wall-hard', '', 0.5)]:
            with self.subTest(reason=reason):
                self.directory = self.root / reason
                self.config.write_text('[profile.default]\n' + timeout)
                start = time.monotonic()
                result = self.invoke_bounded({'default': {'wall_hard_s': hard}})
                self.assertLess(time.monotonic() - start, 3)
                self.assertEqual(result['cancelled'], reason)
                self.assertIn(dict(binary_id='fixture::second', test_name='d'), result['unobserved'])
                self.assertTrue(self.record['starved'])

    def test_cancelled_test_whatever_libtest_reports(self):
        self.env['FIXTURE_CASE'] = 'term-pass'
        self.config.write_text('[profile.default]\nglobal-timeout="1s"\n')
        self.run.__globals__['TERM_GRACE_S'] = 5
        result = self.invoke_bounded({})
        self.assertEqual([(t['test_name'], t['outcome']) for t in result['tests']],
                         [('a', 'passed'), ('b', 'cancelled')])
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite', test_name='c')])
        process = result['processes'][0]
        self.assertEqual(process['tests'], ['a', 'b', 'c'])
        self.assertEqual((process['exit_code'], process['signal']), (0, None))
        failure = ET.parse(self.directory / 'junit.xml').find(".//testcase[@name='b']/failure")
        self.assertEqual(failure.get('type'), 'test abort')
        self.assertIn('signal 15 (SIGTERM)', failure.get('message'))

    def test_forced_kill_finishes_starved(self):
        self.env['FIXTURE_CASE'] = 'term-ignore'
        self.run.__globals__.update(TERM_GRACE_S=5, WALL_GRACE_S=0.5)
        start = time.monotonic()
        result = self.invoke_bounded({'default': {'wall_hard_s': 0.5}})
        self.assertLess(time.monotonic() - start, 3)
        self.assertEqual(result['cancelled'], 'wall-hard')
        self.assertEqual(result['processes'], [])
        self.assertEqual([(t['test_name'], t['outcome'], t['process']) for t in result['tests']],
                         [('a', 'failed', None)])
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite', test_name=n)
                                               for n in ('b', 'c')])
        self.assertEqual((self.record['finished'], self.record['starved'], self.record['returncode']),
                         (True, True, -9))
        self.assertGreaterEqual(self.record['wall_s'], 1)
        cases = ET.parse(self.directory / 'junit.xml').findall('.//testcase')
        self.assertEqual([c.get('name') for c in cases], ['a'])
        self.assertEqual(cases[0].find('failure').get('type'), 'test failure')
        self.assertEqual(cases[0].find('failure').get('message'), 'test failed')
        sessions = [int(line) for line in (self.root / 'prefix-sessions').read_text().splitlines()]
        self.assertEqual(self.module['session_processes'](sessions[-1]), [])

    def test_forced_kill_remaps_recorded_processes(self):
        self.env['FIXTURE_CASE'] = 'term-ignore'
        self.run.__globals__.update(LANES=2, TERM_GRACE_S=5, WALL_GRACE_S=0.5)
        self.binary('fixture::second', names=('d',))
        result = self.invoke_bounded({'default': {'wall_hard_s': 0.5}})
        self.assertEqual([(p['binary_id'], p['tests']) for p in result['processes']],
                         [('fixture::suite', ['c']), ('fixture::second', ['d'])])
        tests = {(t['binary_id'], t['test_name']): t for t in result['tests']}
        self.assertEqual(tests['fixture::suite', 'a']['process'], None)
        self.assertEqual(tests['fixture::suite', 'c']['process'], 0)
        self.assertEqual(tests['fixture::second', 'd']['process'], 1)
        self.assertEqual(result['unobserved'], [dict(binary_id='fixture::suite', test_name='b')])

    def test_fresh_process_takes_its_place(self):
        self.env['FIXTURE_CASE'] = 'crash'
        self.binary('fixture::second', names=('d',))
        self.config.write_text('[profile.default]\n[[profile.default.overrides]]\n'
                               'filter="binary_id(=fixture::suite) & test(=a)"\npriority=10\n'
                               '[[profile.default.overrides]]\n'
                               'filter="binary_id(=fixture::second)"\npriority=5\n')
        processes = sorted(self.invoke()['processes'], key=lambda p: p['start_s'])
        self.assertEqual([(p['binary_id'], p['kind']) for p in processes],
                         [('fixture::suite', 'tests'), ('fixture::suite', 'alone'),
                          ('fixture::second', 'tests'), ('fixture::suite', 'fresh')])

    def test_global_timeout(self):
        self.env['FIXTURE_CASE'] = 'hang'
        self.config.write_text('[profile.default]\nglobal-timeout="1s"\n')
        start = time.monotonic()
        result = self.invoke()
        self.assertLess(time.monotonic() - start, 4)
        self.assertEqual(result['cancelled'], 'global-timeout')
        self.assertTrue(self.record['starved'])
        self.assertEqual([(t['test_name'], t['outcome']) for t in result['tests']],
                         [('a', 'passed'), ('b', 'cancelled')])
        self.assertEqual(result['unobserved'], [{'binary_id': 'fixture::suite', 'test_name': 'c'}])
        failure = ET.parse(self.directory / 'junit.xml').find('.//failure')
        self.assertEqual(failure.get('type'), 'test abort')
        self.assertIn('signal 15 (SIGTERM)', failure.get('message'))

    def test_starved_endings(self):
        for case, outcome, processes in [
                ('starved', 'starved', 2), ('starved-panic', 'failed', 1),
                ('starved-nonzero', 'crashed', 2), ('starved-return', 'passed', 1),
                ('starved-not-last', 'crashed', 2), ('starved-blanks', 'starved', 2)]:
            with self.subTest(case=case):
                self.env['FIXTURE_CASE'] = case
                self.directory = self.root / case
                result = self.invoke()
                tests = {t['test_name']: t for t in result['tests']}
                self.assertEqual(tests['a']['outcome'], outcome)
                if outcome != 'passed':
                    self.assertIn('skein-world: starved:', tests['a']['output'])
                self.assertEqual([tests[n]['outcome'] for n in ('b', 'c')], ['passed', 'passed'])
                self.assertEqual(len(result['processes']), processes)
                self.assertTrue(all(t['alone'] is None for t in tests.values()))
                self.assertNotIn('alone', [p['kind'] for p in result['processes']])
                if processes == 2:
                    self.assertEqual([p['tests'] for p in result['processes']], [['a'], ['b', 'c']])
                    self.assertEqual(result['processes'][1]['kind'], 'fresh')
                    self.assertEqual(tests['b']['process'], tests['c']['process'])
                    self.assertNotEqual(tests['a']['process'], tests['b']['process'])
                if case == 'starved-nonzero':
                    self.assertEqual(result['processes'][0]['exit_code'], 3)

    def test_starved_last_after_failure(self):
        self.env['FIXTURE_CASE'] = 'failure-then-starved'
        result = self.invoke()
        self.assertEqual([(t['test_name'], t['outcome']) for t in result['tests']],
                         [('a', 'failed'), ('b', 'passed'), ('c', 'starved')])
        self.assertEqual(len(result['processes']), 1)
        self.assertEqual(result['processes'][0]['exit_code'], 0)
        self.assertTrue(all(t['alone'] is None for t in result['tests']))

    def test_failure_alone_and_first(self):
        self.binary(names=('a_sets', 'b_reads'), native='state')
        result = self.invoke()
        failure = next(t for t in result['tests'] if t['test_name'] == 'b_reads')
        self.assertEqual((failure['outcome'], failure['before'], failure['alone']),
                         ('failed', ['a_sets'], 'passed'))
        output = (self.directory / 'command-output.txt').read_text()
        self.assertIn('may depend on an earlier test', output)
        self.assertIn('a_sets', output)
        self.assertIn('b_reads', output)
        self.binary()
        self.env['FIXTURE_CASE'] = 'first-failure'
        self.directory = self.root / 'first'
        result = self.invoke()
        self.assertIsNone(result['tests'][0]['alone'])
        self.assertEqual(len(result['processes']), 1)

    def test_splitting_and_kept_times(self):
        self.run.__globals__['LANES'] = 2
        self.binary(names=('a', 'b', 'c', 'd'))
        path = self.root / 'target/tiers-one-process/times.json'
        path.parent.mkdir(parents=True)
        for kept in (True, False):
            with self.subTest(kept=kept):
                if kept:
                    path.write_text(json.dumps({'default': {'fixture::suite': dict(a=2, b=2, c=2, d=2, absent=9)}}))
                else:
                    path.unlink()
                self.directory = self.root / f'split-{kept}'
                result = self.invoke()
                self.assertEqual(sorted(p['tests'] for p in result['processes']), [['a', 'b'], ['c', 'd']])
                times = json.loads(path.read_text())['default']['fixture::suite']
                self.assertTrue(all(times[name] >= 0 for name in ('a', 'b', 'c', 'd')))
                if kept:
                    self.assertEqual(times['absent'], 9)

    def test_zero_and_dominant_kept_times_do_not_drop_tests(self):
        self.run.__globals__['LANES'] = 8
        self.binary(names=('a', 'b', 'c', 'd'))
        path = self.root / 'target/tiers-one-process/times.json'
        path.parent.mkdir(parents=True)
        for values in (dict(a=0, b=0, c=0, d=0), dict(a=100, b=0, c=0, d=0)):
            with self.subTest(values=values):
                self.directory = self.root / ('zero' if values['a'] == 0 else 'dominant')
                path.write_text(json.dumps({'default': {'fixture::suite': values}}))
                result = self.invoke()
                self.assertEqual(sorted(t['test_name'] for t in result['tests']), ['a', 'b', 'c', 'd'])
                self.assertEqual(result['unobserved'], [])

    def test_groups_priority_and_own_process(self):
        self.env['FIXTURE_CASE'] = 'slow'
        self.run.__globals__['LANES'] = 2
        self.binary('fixture::second')
        self.config.write_text('[test-groups]\nserial={max-threads=1}\n[profile.default]\n'
                               '[[profile.default.overrides]]\nfilter="all()"\ntest-group="serial"\n')
        processes = sorted(self.invoke()['processes'], key=lambda p: p['start_s'])
        self.assertEqual(len(processes), 2)
        self.assertGreaterEqual(processes[1]['start_s'], processes[0]['start_s'] + processes[0]['wall_s'])
        self.run.__globals__['LANES'] = 1
        self.binary('fixture::second', names=('d',))
        self.config.write_text('[profile.default]\n[[profile.default.overrides]]\n'
                               'filter="binary_id(=fixture::second)"\npriority=10\n')
        self.directory = self.root / 'priority'
        self.assertEqual(self.invoke()['processes'][0]['binary_id'], 'fixture::second')
        self.listing['rust-suites'].pop('fixture::second')
        self.config.write_text('[test-groups]\nprocess-of-its-own={max-threads="num-cpus"}\n'
                               '[profile.default]\n[[profile.default.overrides]]\n'
                               'filter="test(=b)"\ntest-group="process-of-its-own"\n')
        self.directory = self.root / 'own'
        processes = self.invoke()['processes']
        self.assertEqual(len(processes), 2)
        self.assertIn(['b'], [p['tests'] for p in processes])

    def test_environment(self):
        self.env.update(FIXTURE_CASE='environment', CONFIGURED='caller', KEPT='caller')
        (self.root / 'cargo-home').mkdir()
        (self.root / 'cargo-home/config.toml').write_text(
            '[env]\nGLOBAL_CONFIGURED="global"\nCONFIGURED="global"\nNEXTEST={value="1",force=true}\n')
        (self.root / '.cargo').mkdir()
        (self.root / '.cargo/config.toml').write_text(
            '[env]\nCONFIGURED={value="forced",force=true}\nKEPT="config"\n'
            'RELATIVE={value="relative",relative=true}\n')
        self.assertEqual([t['outcome'] for t in self.invoke()['tests']], ['passed'] * 3)

    def test_wall_hard_and_session_cleanup(self):
        self.env['FIXTURE_CASE'] = 'hang'
        result = self.invoke({'default': {'wall_hard_s': 0.5}})
        self.assertEqual(result['cancelled'], 'wall-hard')
        self.assertTrue(self.record['starved'])
        self.assertEqual(json.loads((self.directory / 'starved.json').read_text()), {'wall_hard_s': 0.5})
        sessions = [int(line) for line in (self.root / 'prefix-sessions').read_text().splitlines()]
        self.assertEqual(self.module['session_processes'](sessions[-1]), [])

    def test_interruption_and_fault_stop_the_session(self):
        for error in (KeyboardInterrupt, RuntimeError):
            with self.subTest(error=error):
                self.directory = self.root / error.__name__
                self.env['FIXTURE_CASE'] = 'hang'
                def interrupt(text, **options):
                    if text.startswith('PASS '):
                        raise error('fixture interruption')
                    print(text, **options)
                self.run.__globals__['print'] = interrupt
                try:
                    with self.assertRaises(error):
                        self.invoke()
                finally:
                    self.run.__globals__.pop('print')
                    sessions = [int(line) for line in (self.root / 'prefix-sessions').read_text().splitlines()]
                    remaining = self.module['session_processes'](sessions[-1])
                    for pid in remaining:
                        try:
                            os.kill(pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                self.assertEqual(remaining, [])

    def test_leak(self):
        self.env['FIXTURE_CASE'] = 'leak'
        self.binary(names=('a',))
        self.binary('fixture::second', names=('d',))
        start = time.monotonic()
        result = self.invoke()
        self.assertLess(time.monotonic() - start, 2)
        self.assertTrue(result['processes'][0]['leaked'])
        self.assertEqual([t['outcome'] for t in result['tests']], ['passed', 'passed'])

    def test_empty_selection(self):
        self.binary(names=())
        for rule in ('fail', 'warn', 'pass'):
            self.directory = self.root / rule
            self.command.append(f'--no-tests={rule}')
            result = self.invoke()
            self.command.pop()
            self.assertEqual((result['selected'], result['no_tests']), (0, rule))
            self.assertIsNone(self.record['returncode'])
            self.assertEqual(self.record['wall_s'], 0.0)
            self.assertEqual(result['processes'], [])
            output = (self.directory / 'command-output.txt').read_text()
            self.assertEqual('warning: no tests to run\n' in output, rule == 'warn')
        calls = (self.root / 'prefix-argv.jsonl').read_text().splitlines()
        self.assertEqual(len(calls), 3)

    def test_refusals_before_commands(self):
        rows = [
            ('test-threads=2', [], {}, {}), ('', ['-j', '2'], {}, {}),
            ('scripts=[{filter="all()",setup="prepare"}]', [], {}, {}),
            ('retries=1', [], {}, {}), ('', ['--retries=1'], {}, {}),
            ('', [], {'NEXTEST_TEST_THREADS': '2'}, {}),
            ('', [], {'NEXTEST_RETRIES': '1'}, {}),
            ('slow-timeout={period="1s",terminate-after=2}', [], {}, {}),
            ('inherits="fuzzy"', [], {}, {}),
            ('[[profile.default.overrides]]\nfilter="all()"\nthreads-required=2', [], {}, {}),
            ('[[profile.default.overrides]]\nfilter="all()"\nrun-extra-args=[]', [], {}, {}),
            ('[[profile.default.overrides]]\nfilter="all()"\nplatform="cfg(unix)"', [], {}, {}),
            ('', ['--no-run'], {}, {}), ('', ['--config-file', 'x'], {}, {}),
            ('', ['--tool-config-file=x'], {}, {}), ('', ['--profile', 'fuzzy'], {}, {}), ('', ['-Pfuzzy'], {}, {}),
            ('', ['--stress-count=2'], {}, {}), ('', ['--stress-duration=1s'], {}, {}),
            ('', ['--debugger=x'], {}, {}), ('', [], {}, {'default': {'cpu_test_s': 1}}),
            ('[[profile.default.overrides]]\nfilter="rdeps(x)"', [], {}, {}),
        ]
        for index, (config, args, env, limits) in enumerate(rows):
            with self.subTest(index=index, config=config, args=args):
                for name in ('cargo-argv.jsonl', 'prefix-argv.jsonl'):
                    (self.root / name).unlink(missing_ok=True)
                self.config.write_text('[profile.default]\n' + config + '\n')
                with self.assertRaises(ValueError):
                    self.run([*self.command, *args], dict(self.env, **env), self.directory,
                             'default', limits, self.config)
                self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
                self.assertFalse((self.root / 'prefix-argv.jsonl').exists())
        self.config.write_text('[profile.default]\n')
        self.directory.mkdir(exist_ok=True)
        (self.directory / 'occupied').touch()
        with self.assertRaisesRegex(ValueError, 'empty'):
            self.run(self.command, self.env, self.directory, 'default', {}, self.config)
        self.assertFalse((self.root / 'prefix-argv.jsonl').exists())

    def test_filtersets(self):
        test = dict(package='my-pkg', binary='suite', binary_id='my-pkg::suite', kind='test', test='foo::bar')
        rows = [('all()', True), ('none()', False), ('package(my-*)', True),
                ('binary(sui*)', True), ('binary_id(*::suite)', True), ('kind(test)', True),
                ('test(bar)', True), ('test(=bar)', False), ('test(~bar)', True),
                ('test(/^foo::/)', True), ('test(#foo::*)', True), ('package(=my-pkg)', True),
                ('binary(~uit)', True), ('kind(t*)', False), ('test(!)', False),
                ('not none()', True), ('!all()', False), ('all() and none()', False),
                ('all() & all()', True), ('none() or all()', True), ('none() | all()', True),
                ('none() + all()', True), ('all() - all()', False),
                ('all() or none() and none()', True), ('(all() or none()) and none()', False),
                ('all() - none() & all()', True), ('not test(/bar$/)', False)]
        for expression, expected in rows:
            with self.subTest(expression=expression):
                self.assertEqual(self.module['filterset'](expression)(test), expected)
        for expression in ('rdeps(x)', 'test(/[/)', 'all() junk', 'test(a) and', 'unknown()'):
            with self.subTest(invalid=expression), self.assertRaises(ValueError):
                self.module['filterset'](expression)

    def test_default_filter_proof_and_caller_filter(self):
        self.config.write_text('[profile.default]\ndefault-filter="none()"\n')
        with self.assertRaisesRegex(ValueError, 'default-filter'):
            self.invoke()
        filters = [['-E', 'all()'], ['-Eall()'], ['-E=all()'],
                   ['--filterset', 'all()'], ['--filterset=all()'],
                   ['--filter-expr', 'all()'], ['--filter-expr=all()']]
        for index, args in enumerate(filters):
            with self.subTest(args=args):
                self.directory = self.root / f'filter-{index}'
                original = self.command
                self.command = [*original, *args]
                try:
                    self.assertEqual(self.invoke()['selected'], 3)
                finally:
                    self.command = original
        self.config.write_text('[profile.default]\ndefault-filter="rdeps(x)"\n')
        self.directory = self.root / 'unsupported-unused'
        self.command.extend(['-E', 'all()'])
        self.assertEqual(self.invoke()['selected'], 3)
        del self.command[-2:]
        (self.root / 'prefix-argv.jsonl').unlink()
        self.directory = self.root / 'unsupported-used'
        with self.assertRaisesRegex(ValueError, 'unsupported filter'):
            self.invoke()
        self.assertFalse((self.root / 'prefix-argv.jsonl').exists())


if __name__ == '__main__':
    os.umask(0o077)
    unittest.main()
