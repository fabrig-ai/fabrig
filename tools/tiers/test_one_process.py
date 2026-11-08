#!/usr/bin/env python3
"""Offline one-process runner proof (tools/tiers/design.md, 11.10)."""
import contextlib
import io
import json
import math
import os
from pathlib import Path
import resource
import runpy
import shutil
import signal
import subprocess
import tempfile
import time
import unittest
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
                                ('state', 'state.rs', [])]:
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
            self.assertIn(('ok ' if scenario in ('pass', 'expected-panic') else 'failed ') + name, logfile.read_text())


class RunnerTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-one-test-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.directory = self.root / 'run with spaces'
        (self.root / '.config').mkdir()
        self.config = self.root / '.config/nextest.toml'
        self.config.write_text('[profile.default]\n')
        for name in ('cargo', 'prefix'):
            shutil.copyfile(FIXTURES / f'{name}.py', self.root / name)
            (self.root / name).chmod(0o700)
        self.env = dict(os.environ, FIXTURE_CASE='pass', NEXTEST='1')
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
            'non-test-binaries': {'fixture-id': [{'name': 'helper', 'path': str(self.root / 'helper'), 'kind': 'bin'}]},
            'build-script-out-dirs': {'fixture-id': str(self.root / 'out')}, 'linked-paths': {}}}
        self.binary()

    def binary(self, binary_id='fixture::suite', names=('a', 'b', 'c'), native='tests'):
        self.listing['rust-suites'][binary_id] = dict(
            status='listed', **{'binary-id': binary_id, 'binary-path': str(NATIVE / native),
            'cwd': str(self.root), 'package-id': 'fixture-id', 'kind': 'test', 'binary-name': 'suite',
            'testcases': {name: {'ignored': name == 'ignored', 'filter-match': {'status': 'matches'}}
                          for name in names}})

    def invoke(self, limits=None):
        (self.root / 'listing.json').write_text(json.dumps(self.listing) + '\n')
        (self.root / 'metadata.json').write_text(json.dumps(self.metadata) + '\n')
        with contextlib.redirect_stdout(io.StringIO()):
            self.run(self.command, self.env, self.directory, 'default', limits or {}, self.config)
        self.results = json.loads((self.directory / 'results.json').read_text())
        self.record = json.loads((self.directory / 'run.json').read_text())
        return self.results

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
                                       'default', '-p', 'fixture'])
        self.assertEqual(calls[1][1:3], ['-I', '-B'])
        self.assertIn('lanes', calls[1])
        output = (self.directory / 'command-output.txt').read_text()
        self.assertIn('fixture listing suffix', output)
        self.assertIn('PASS', output)
        self.assertNotIn('tiers-one-process: ', output)

    def test_unterminated_output(self):
        self.env['FIXTURE_CASE'] = 'unterminated'
        self.assertEqual([t['outcome'] for t in self.invoke()['tests']], ['passed'] * 3)

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
        self.directory = self.root / 'before-run'
        self.binary(native='before')
        result = self.invoke()
        self.assertEqual([t['outcome'] for t in result['tests']], ['crashed'] * 3)
        self.assertEqual(len(result['processes']), 1)

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

    def test_starved_and_later_panic(self):
        for case, outcome in [('starved', 'starved'), ('starved-panic', 'failed')]:
            with self.subTest(case=case):
                self.env['FIXTURE_CASE'] = case
                self.directory = self.root / case
                result = self.invoke()
                self.assertEqual(result['tests'][0]['outcome'], outcome)
                self.assertIn('skein-world: starved:', result['tests'][0]['output'])
                self.assertEqual([t['outcome'] for t in result['tests'][1:]], ['passed', 'passed'])
                self.assertEqual(len(result['processes']), 1)

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
        self.command.extend(['-E', 'all()'])
        result = self.invoke()
        self.assertEqual(result['selected'], 3)


if __name__ == '__main__':
    os.umask(0o077)
    unittest.main()
