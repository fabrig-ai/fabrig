#!/usr/bin/env python3
"""Offline one-process runner proof (tools/tiers/design.md, 11.10)."""
import contextlib
import errno
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
import struct
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
                                ('state', 'state.rs', []), ('again', 'again.rs', []),
                                ('count', 'count.rs', [])]:
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
        for scenario in ('pass', 'first-failure', 'starved'):
            logfile = NATIVE / 'protocol.log'
            result = subprocess.run([str(NATIVE / 'tests'), '--exact', '--nocapture',
                                     '--test-threads=1', '--logfile', str(logfile), 'a'],
                                    env=dict(os.environ, FIXTURE_CASE=scenario),
                                    stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT, text=True, timeout=5)
            print(f'libtest {scenario}: {result.stdout!r}; log: {logfile.read_text()!r}', flush=True)
            self.assertIn('test a ... ', result.stdout)
            if scenario == 'starved':
                self.assertEqual(result.returncode, 0)
                self.assertEqual(logfile.read_text(), '')
                self.assertTrue(result.stdout.rstrip().endswith(
                    'skein-world: starved: waited 1 ms runnable, fixture patience'))
            else:
                self.assertIn('ok a' if scenario == 'pass' else 'failed a', logfile.read_text())


class ProfileTargetTests(unittest.TestCase):
    def test_cargo_targets_and_named_test_exceptions(self):
        one = runpy.run_path(str(HERE / 'one_process.py'))
        with tempfile.TemporaryDirectory(dir=HERE.parents[1]) as scratch, contextlib.chdir(scratch):
            root = Path.cwd()
            (root / 'Cargo.toml').write_text('[workspace]\nmembers=["app", "other"]\nresolver="2"\n')
            for name in ('app', 'other'):
                package = root / name
                (package / 'tests').mkdir(parents=True)
                (package / 'Cargo.toml').write_text(f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2024"\n')
                for target in ('live', 'suite'):
                    (package / 'tests' / (target + '.rs')).write_text(
                        '#[test] fn kept() {}\n#[test] fn slow() {}\nmod fuzzy { #[test] fn sample() {} }\n')
            config = root / '.config/nextest.toml'
            config.parent.mkdir()
            config.write_text("""[profile.default]
default-filter = "not test(/^fuzzy::/) & (not binary(=live) | (package(=other) & test(=kept)))"
[profile.fuzzy]
default-filter = "test(/^fuzzy::/) - binary(=live)"
""")
            subprocess.run(['cargo', 'generate-lockfile', '--offline'], check=True, capture_output=True)
            command = ['cargo', 'nextest', 'run', '-p', 'app', '-p', 'other', '--no-tests=warn']
            build_target = root / 'landing'
            landing_env = dict(os.environ, CARGO_TARGET_DIR=str(build_target), CARGO_NET_OFFLINE='true',
                               TIERS_PROFILE_TARGETS='1')
            built = subprocess.run([HERE / 'build-profile-targets', '--workspace'], env=landing_env,
                                   capture_output=True, text=True)
            self.assertEqual(built.returncode, 0, built.stderr)
            self.assertIn('skip linking app::live', built.stdout)
            dep_info = '\n'.join(path.read_text() for path in build_target.rglob('*.d'))
            self.assertNotIn('app/tests/live.rs', dep_info)
            self.assertIn('other/tests/live.rs', dep_info)
            for profile in ('default', 'fuzzy'):
                selections = []
                for mode in ('broad', 'pruned', 'landing'):
                    if mode == 'landing':
                        target, env = build_target, landing_env
                        args = ['cargo', 'nextest', 'run', '--workspace', '--no-tests=warn']
                    else:
                        target = root / mode
                        env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE='true',
                                   TIERS_PRE_PR_TARGETS='1' if mode == 'pruned' else '0')
                        args = command
                    with contextlib.redirect_stdout(io.StringIO()):
                        plan, *_ = one['listed_plan'](args, env, profile, {}, config, None, root / 'layers.toml')
                    selections.append({(binary['binary_id'], case['name'])
                                       for binary in plan['binaries'] for case in binary['tests']})
                    linked = any('app/tests/live.rs' in path.read_text() for path in target.rglob('*.d'))
                    self.assertEqual(linked, mode == 'broad')
                self.assertEqual(selections[0], selections[1])
                self.assertEqual(selections[0], selections[2])
                self.assertIn(('app::suite', 'kept' if profile == 'default' else 'fuzzy::sample'), selections[2])
                self.assertEqual(('other::live', 'kept') in selections[2], profile == 'default')

            config.write_text('[profile.default]\ndefault-filter="none()"\n')
            with contextlib.redirect_stdout(io.StringIO()):
                plan, _, no_tests, _, _ = one['listed_plan'](command, env, 'default', {}, config, None, root / 'layers.toml')
            self.assertEqual((plan['binaries'], no_tests), ([], 'warn'))

            # Splitting the two roots would disable a feature of a shared dependency for app.
            (root / '.cargo').mkdir()
            (root / '.cargo/config.toml').write_text('[resolver]\nfeature-unification="selected"\n')
            (root / 'shared/src').mkdir(parents=True)
            (root / 'shared/Cargo.toml').write_text('[package]\nname="shared"\nversion="0.1.0"\n[features]\nextra=[]\n')
            (root / 'shared/src/lib.rs').touch()
            for name in ('app', 'other'):
                with (root / name / 'Cargo.toml').open('a') as manifest:
                    features = ', features=["extra"]' if name == 'other' else ''
                    manifest.write(f'[dependencies]\nshared={{path="../shared"{features}}}\n')
            subprocess.run(['cargo', 'generate-lockfile', '--offline'], check=True, capture_output=True)
            metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1']))
            args = ['-p', 'app', '-p', 'other']
            filters = [one['filterset']('not (package(=app) & binary(=live))')]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(one['profile_target_groups'](args, metadata, filters, ['cargo'], os.environ), [args])


class ConfigurationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(dir=HERE.parents[1], prefix='tiers-one-config-')
        self.addCleanup(temporary.cleanup)
        self.config = Path(temporary.name) / 'nextest.toml'
        self.configuration = runpy.run_path(str(HERE / 'one_process.py'))['configuration']

    def test_platform_overrides_are_skipped_without_compiling_filters(self):
        rows = [('cfg(target_os = "macos")', 'slow-timeout="12s"'),
                ('cfg(unix)', 'filter="all()"'),
                ('cfg(unix)', 'filter="rdeps(x)"')]
        for platform, settings in rows:
            with self.subTest(platform=platform, settings=settings):
                self.config.write_text('[profile.default]\n[[profile.default.overrides]]\n'
                                       f"platform='{platform}'\n" + settings + '\n')
                _, overrides, _, _ = self.configuration(self.config, 'default', {}, {}, False)
                self.assertEqual(overrides, [])

    def test_platform_overrides_with_honoured_keys_are_refused(self):
        for settings in ('priority=5', 'test-group="serial"'):
            with self.subTest(settings=settings):
                self.config.write_text('[profile.default]\n[[profile.default.overrides]]\n'
                                       'platform="cfg(unix)"\n' + settings + '\n')
                with self.assertRaisesRegex(ValueError, 'unsupported override platform'):
                    self.configuration(self.config, 'default', {}, {}, False)

    def test_platform_overrides_keep_other_refusals(self):
        for settings, message in (('retries=1', 'retries'),
                                  ('slow-timeout={period="12s",terminate-after=2}', 'terminate-after'),
                                  ('threads-required=2', 'threads-required'),
                                  ('run-extra-args=[]', 'run-extra-args')):
            with self.subTest(settings=settings):
                self.config.write_text('[profile.default]\n[[profile.default.overrides]]\n'
                                       'platform="cfg(unix)"\n' + settings + '\n')
                with self.assertRaisesRegex(ValueError, message):
                    self.configuration(self.config, 'default', {}, {}, False)

    def test_repository_profiles_load(self):
        for profile in ('default', 'fuzzy'):
            with self.subTest(profile=profile):
                _, overrides, _, _ = self.configuration(HERE.parents[1] / '.config/nextest.toml',
                                                        profile, {}, {}, False)
                self.assertTrue(overrides)
                self.assertTrue(all('platform' not in settings for _, settings in overrides))


class CountingDecisionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.paranoid = Path(temporary.name) / 'paranoid'
        self.module = runpy.run_path(str(HERE / 'one_process.py'))
        self.counting = self.module['counting']

    def test_past_two(self):
        self.paranoid.write_text('3\n')
        self.assertEqual(self.counting(self.paranoid),
                         dict(counted=False, why='kernel.perf_event_paranoid is 3, past 2'))

    def test_unreadable(self):
        decision = self.counting(self.paranoid)
        self.assertFalse(decision['counted'])
        self.assertTrue(decision['why'].startswith(f'no counter: cannot read {self.paranoid}: '))

    def test_no_integer(self):
        self.paranoid.write_text('x\n')
        self.assertEqual(self.counting(self.paranoid),
                         dict(counted=False, why=f'no counter: {self.paranoid} holds no integer'))

    def test_permitted_levels_try_machine_counter(self):
        for level in (2, -1):
            with self.subTest(level=level):
                self.paranoid.write_text(f'{level}\n')
                decision = self.counting(self.paranoid)
                if decision['counted']:
                    self.assertEqual(decision, dict(counted=True, why=None))
                else:
                    self.assertTrue(decision['why'].startswith(('perf_event_open refused: ', 'no counter: ')))

    def test_counter_decision_errors_and_attribute(self):
        self.paranoid.write_text('2\n')
        globals_ = self.counting.__globals__
        with mock.patch.dict(globals_, PERF_EVENT_OPEN=None):
            self.assertEqual(self.counting(self.paranoid),
                             dict(counted=False, why=f"no counter: no perf_event_open on {self.module['platform'].machine()}"))
        for code, prefix in ((errno.EACCES, 'perf_event_open refused'), (errno.EPERM, 'perf_event_open refused'),
                             (errno.ENOSYS, 'no counter: perf_event_open')):
            with self.subTest(code=code):
                error = OSError(code, os.strerror(code))
                with mock.patch.dict(globals_, PERF_EVENT_OPEN=298,
                                     perf_event_open=mock.Mock(side_effect=error)):
                    self.assertEqual(self.counting(self.paranoid), dict(counted=False, why=f'{prefix}: {error}'))
        fd = os.open(self.paranoid, os.O_RDONLY)
        opened = mock.Mock(return_value=fd)
        with mock.patch.dict(globals_, PERF_EVENT_OPEN=298, perf_event_open=opened):
            self.assertEqual(self.counting(self.paranoid), dict(counted=True, why=None))
        opened.assert_called_once_with(struct.pack('=IIQQQQQIIQ', 0, 64, 1, 0, 0, 3, 99, 0, 0, 0), 0, -1, -1, 8)
        with self.assertRaises(OSError):
            os.fstat(fd)

    def test_counter_reads_only_complete_unscaled_values_and_closes(self):
        process_type = self.module['BinaryProcess']
        process = process_type.__new__(process_type)
        for data, expected in ((struct.pack('=QQQ', 123, 7, 7), 123),
                               (struct.pack('=QQQ', 123, 7, 6), None), (b'short', None)):
            with self.subTest(data=data):
                read, write = os.pipe()
                os.write(write, data)
                os.close(write)
                process.counter = read
                self.assertEqual(process.instructions(), expected)
                self.assertIsNone(process.counter)
                with self.assertRaises(OSError):
                    os.fstat(read)
        # A write-only descriptor makes read fail, and still must close.
        read, write = os.pipe()
        os.close(read)
        process.counter = write
        self.assertIsNone(process.instructions())
        self.assertIsNone(process.counter)
        with self.assertRaises(OSError):
            os.fstat(write)
        self.assertIsNone(process.instructions())


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
        self.env = dict(os.environ, FIXTURE_CASE='pass', NEXTEST='1',
                        GIT_CEILING_DIRECTORIES=str(self.root.parent))
        self.env.pop('TIERS_HOLD', None)
        self.command = [str(self.root / 'prefix'), str(self.root / 'cargo'), 'nextest', 'run',
                        '-p', 'fixture', '--profile', 'default', '--no-fail-fast']
        self.module = runpy.run_path(str(HERE / 'one_process.py'))
        self.run = self.module['run']
        self.run.__globals__.update(LANES=1, TERM_GRACE_S=0.1, WALL_GRACE_S=0.5, LEAK_S=0.05)
        package = dict(id='fixture-id', name='fixture', manifest_path=str(self.root / 'Cargo.toml'),
                       version='1.2.3', authors=['A', 'B'],
                       targets=[dict(name='fixture', kind=['lib'], test=True),
                                dict(name='suite', kind=['test'], test=True)])
        self.metadata = {'packages': [package], 'workspace_members': ['fixture-id'],
                         'workspace_root': str(self.root)}
        self.env['CARGO_HOME'] = str(self.root / 'cargo-home')
        self.listing = {'rust-suites': {}, 'rust-build-meta': {
            'target-directory': str(self.root / 'target'), 'base-output-directories': ['debug'],
            'non-test-binaries': {'fixture-id': [{'name': 'helper', 'path': 'debug/helper', 'kind': 'bin-exe'},
                                               {'name': 'library', 'path': 'debug/library.so', 'kind': 'dylib'}]},
            'build-script-out-dirs': {'fixture-id': str(self.root / 'out')}, 'linked-paths': []}}
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

    def invoke(self, limits=None, **kwargs):
        self.prepare()
        with contextlib.chdir(self.root), contextlib.redirect_stdout(io.StringIO()):
            self.run(self.command, self.env, self.directory, 'default', limits or {}, self.config, **kwargs)
        self.results = json.loads((self.directory / 'results.json').read_text())
        self.record = json.loads((self.directory / 'run.json').read_text())
        return self.results

    def assert_counting(self, results):
        decision = self.module['counting']()
        self.assertEqual(results['counting'], decision)
        output = (self.directory / 'command-output.txt').read_text()
        line = 'tiers: not counting instructions: '
        if decision['counted']:
            self.assertNotIn(line, output)
            for process in results['processes']:
                if process['kind'] in ('tests', 'fresh'):
                    self.assertIs(type(process['instructions']), int)
                    self.assertGreater(process['instructions'], 0)
                    if process['binary_id'] == 'fixture::count' and 'spin' in process['tests']:
                        self.assertGreaterEqual(process['instructions'], 10_000_000)
                else:
                    self.assertIsNone(process['instructions'])
        else:
            self.assertEqual(output.count(line), 1)
            self.assertEqual(output.count(f"{line}{decision['why']}\n"), 1)
            status = re.search(r'(?m)^(PASS|FAIL|CRASH|STARVED|CANCEL) ', output)
            self.assertIsNotNone(status)
            self.assertLess(output.index(line), status.start())
            for process in results['processes']:
                self.assertIsNone(process['instructions'])

    def assert_uncounted(self, results, directory):
        self.assertIsNone(results['counting'])
        self.assertIsNone(results['commit'])
        for process in results['processes']:
            self.assertIsNone(process['instructions'])
        self.assertNotIn('tiers: not counting', (directory / 'command-output.txt').read_text())
        self.assertFalse(json.loads((directory / 'plan.json').read_text())['count'])

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


class CountingRunTests(RunnerFixture):
    def checks_fixture(self, contents):
        runner = self.root / 'write-checks'
        runner.write_text(rf"""#!{shutil.which('python3')}
import os
from pathlib import Path
import sys
name = sys.argv[-1]
checks = Path(os.environ['TIERS_CHECKS_FILE'])
assert checks.is_absolute() and not checks.exists()
if name == 'a':
    with checks.open('ab') as output:
        output.write({contents!r})
print(f'test {{name}} ... ok')
with open(sys.argv[sys.argv.index('--logfile') + 1], 'w') as log:
    log.write(f'ok {{name}}\n')
print('test result: ok')
""")
        runner.chmod(0o700)
        self.binary(names=('a', 'b'))
        self.listing['rust-suites']['fixture::suite']['binary-path'] = str(runner)
        self.run.__globals__['LANES'] = 2

    def test_checks_are_read_once_per_process_and_missing_is_empty(self):
        self.checks_fixture(b'store-check 2\nseed 4\nstore-check 3\n')
        result = self.invoke()
        self.assertEqual({tuple(p['tests']): p['checks'] for p in result['processes']},
                         {('a',): {'store-check': 5, 'seed': 4}, ('b',): {}})
        checks, = self.directory.glob('checks-*.txt')
        self.assertEqual(checks.read_bytes(), b'store-check 2\nseed 4\nstore-check 3\n')

    def test_malformed_checks_refuse_the_run_with_process_file_and_line(self):
        self.checks_fixture(b'seed 1\nstore-check 0\n')
        with self.assertRaisesRegex(ValueError, r'process \d+ \(fixture::suite\): .*checks-\d+\.txt, '
                                    r"malformed check line 2: b'store-check 0\\n'"):
            self.invoke()

    def test_suite_counts_spawned_thread_and_idle_process(self):
        self.listing['rust-suites'].clear()
        self.binary('fixture::count', names=('idle', 'spin'), native='count')
        self.listing['rust-suites']['fixture::count']['binary-name'] = 'count'
        self.metadata['packages'][0]['targets'].append(dict(name='count', kind=['test'], test=True))
        self.run.__globals__['LANES'] = 2
        result = self.invoke()
        self.assertEqual([t['outcome'] for t in result['tests']], ['passed', 'passed'])
        self.assertEqual(sorted(p['tests'] for p in result['processes']), [['idle'], ['spin']])
        self.assertTrue(json.loads((self.directory / 'plan.json').read_text())['count'])
        self.assert_counting(result)

    def test_commit_from_metadata_workspace_root(self):
        # Invoke from elsewhere: only metadata's root is the run's checkout.
        for args in (['init', '--quiet'], ['-c', 'user.name=t', '-c', 'user.email=t@t.invalid',
                                          'commit', '--quiet', '--allow-empty', '-m', 'fixture']):
            subprocess.run(['git', *args], cwd=self.root, env=self.env, stdin=subprocess.DEVNULL,
                           check=True, capture_output=True)
        expected = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=self.root, env=self.env,
                                           stdin=subprocess.DEVNULL, text=True).strip()
        self.prepare()
        with contextlib.redirect_stdout(io.StringIO()):
            self.run(self.command, self.env, self.directory, 'default', {}, self.config)
        self.assertEqual(json.loads((self.directory / 'results.json').read_text())['commit'], expected)

    def test_commit_without_repository(self):
        self.assertIsNone(self.invoke()['commit'])

    def test_counted_descriptor_handoff_and_refusal(self):
        # A readable stand-in exercises the handoff even without kernel permission.
        counter = self.root / 'counter'
        counter.write_bytes(struct.pack('=QQQ', 123, 7, 7))
        self.env.update(FIXTURE_CASE='descriptors', FIXTURE_DESCRIPTOR_CHECK=str(FIXTURES / 'descriptors.py'))
        self.binary(names=('a',))
        self.prepare()
        with contextlib.chdir(self.root), contextlib.redirect_stdout(io.StringIO()):
            plan, _, _, _, _ = self.module['listed_plan'](
                self.command, self.env, 'default', {}, self.config, None, HERE / 'layers.toml')
        plan['count'] = True
        globals_ = self.module['lanes'].__globals__

        def open_counter(attr, pid, cpu, group, flags):
            assert (attr, pid, cpu, group, flags) == (
                struct.pack('=IIQQQQQIIQ', 0, 64, 1, 0, 0, 3, 4195, 0, 0, 0), 0, -1, -1, 8)
            return os.open(counter, os.O_RDONLY | os.O_CLOEXEC)

        for opener, expected in ((open_counter, 123), (mock.Mock(side_effect=OSError(errno.EPERM, 'refused')), None)):
            with self.subTest(expected=expected):
                events = []
                def emit(kind, **fields):
                    events.append(dict(event=kind, **fields))
                with mock.patch.dict(globals_, counting=lambda: dict(counted=True, why=None),
                                     perf_event_open=opener, emit=emit):
                    self.module['lanes'](plan, self.root)
                self.assertEqual([e['event'] for e in events[:2]], ['lanes', 'counting'])
                process, = [e['item'] for e in events if e['event'] == 'process']
                test, = [e['item'] for e in events if e['event'] == 'test']
                self.assertEqual(test['outcome'], 'passed', test['output'])
                self.assertEqual(process['instructions'], expected)


class LaneTests(RunnerFixture):
    cpu_counts = [(None, 3, 3), ({0, 1}, 8, 2), (None, None, 1)]

    def check_lane_counts(self, fixed):
        self.binary(names=())
        self.run.__globals__['LANES'] = fixed
        for index, (affinity, cpus, default) in enumerate(self.cpu_counts):
            with self.subTest(affinity=affinity, cpus=cpus), contextlib.ExitStack() as patches:
                if affinity is None:
                    if hasattr(os, 'sched_getaffinity'):
                        patches.enter_context(mock.patch.object(os, 'sched_getaffinity'))
                        del os.sched_getaffinity
                else:
                    patches.enter_context(mock.patch.object(os, 'sched_getaffinity',
                                                           return_value=affinity, create=True))
                patches.enter_context(mock.patch.object(os, 'cpu_count', return_value=cpus))
                self.assertEqual(self.module['default_lanes'](), default)
                self.directory = self.root / f'run-{index}'
                self.invoke()
                expected = fixed or default
                self.assertEqual(self.record['lanes'], expected)
                plan = json.loads((self.directory / 'plan.json').read_text())
                self.assertEqual(plan['lanes'], fixed)
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    self.module['lanes'](plan, self.root)
                events = [json.loads(line.removeprefix(self.module['EVENT']))
                          for line in output.getvalue().splitlines()]
                self.assertEqual([event['lanes'] for event in events if event['event'] == 'lanes'],
                                 [expected])

    def test_default_lanes_follow_available_cpu_counts(self):
        self.check_lane_counts(None)

    def test_fixed_lanes_ignore_cpu_counts(self):
        self.check_lane_counts(1)


class PlacementTests(RunnerFixture):
    def placement_binaries(self, rows):
        self.listing['rust-suites'].clear()
        original = self.metadata['packages'][0]
        packages = {}
        for binary_id, manifest, kind, _, _ in rows:
            package_id = binary_id.split('::')[0]
            package = packages.setdefault(package_id, dict(original, id=package_id, name=package_id,
                                                           manifest_path=str(self.root / manifest), targets=[]))
            target_name = binary_id.split('::', 1)[1] if '::' in binary_id else package_id
            if kind == 'bin':
                target_name = target_name.removeprefix('bin/')
            package['targets'].append(dict(name=target_name, kind=[kind], test=True))
            self.binary(binary_id, names=('a',))
            self.listing['rust-suites'][binary_id].update({'package-id': package_id, 'kind': kind})
        self.metadata['packages'] = list(packages.values())

    def assert_placements(self, result, rows):
        expected = {binary_id: (tree, layer) for binary_id, _, _, tree, layer in rows}
        plan = json.loads((self.directory / 'plan.json').read_text())
        for records in (plan['binaries'], result['processes']):
            self.assertEqual({b['binary_id']: (b['tree'], b['layer']) for b in records}, expected)
        self.assertEqual([t['outcome'] for t in result['tests']], ['passed'] * len(rows))
        self.assertEqual(result['workspace'],
                         [dict(binary_id=binary_id, tree=tree, layer=layer, listed=True)
                          for binary_id, (tree, layer) in sorted(expected.items())])

    def replay_layers(self):
        rows = [
            ('app', 'temper/crates/app/Cargo.toml', 'lib', 'temper', 'step'),
            ('app::sim', 'temper/crates/app/Cargo.toml', 'test', 'temper', 'simulated'),
            ('app::real', 'temper/crates/app/Cargo.toml', 'test', 'temper', 'real-loop'),
            ('app::end', 'temper/crates/app/Cargo.toml', 'test', 'temper', 'end-to-end'),
            ('app::unplaced', 'temper/crates/app/Cargo.toml', 'test', 'temper', None),
            ('check::suite', 'tests/check/Cargo.toml', 'test', 'repository', 'checks'),
        ]
        self.placement_binaries(rows)
        self.metadata['workspace_members'] = [p['id'] for p in self.metadata['packages']]
        self.command[5] = 'app'
        layers = self.root / 'tools/tiers/layers.toml'
        layers.parent.mkdir(parents=True)
        layers.write_text('layers = ["step", "domain", "system", "machine", "protocol", "io", '
                          '"simulated", "real-loop", "end-to-end"]\n'
                          '[temper]\n"app::sim" = "simulated"\n"app::real" = "real-loop"\n'
                          '"app::end" = "end-to-end"\n')
        return rows

    def test_default_and_fuzzy_run_only_replaying_layers(self):
        rows = self.replay_layers()
        self.config.write_text('[profile.default]\n[profile.fuzzy]\n[profile.other]\n')
        self.prepare()
        for profile in ('default', 'fuzzy', 'other'):
            with self.subTest(profile=profile):
                self.directory = self.root / profile
                command = [*self.command[:7], profile, *self.command[8:]]
                with contextlib.chdir(self.root), contextlib.redirect_stdout(io.StringIO()):
                    self.run(command, self.env, self.directory, profile, {}, self.config)
                result = json.loads((self.directory / 'results.json').read_text())
                plan = json.loads((self.directory / 'plan.json').read_text())
                all_layers = ['step', 'domain', 'system', 'machine', 'protocol', 'io',
                              'simulated', 'real-loop', 'end-to-end', 'checks']
                expected = rows if profile == 'other' else [r for r in rows if r[4] not in ('real-loop', 'end-to-end')]
                wanted = {r[0] for r in expected}
                self.assertEqual({b['binary_id'] for b in plan['binaries']}, wanted)
                self.assertEqual({p['binary_id'] for p in result['processes']}, wanted)
                self.assertEqual(result['selected_layers'], all_layers if profile == 'other' else all_layers[:7] + ['checks'])
                output = (self.directory / 'command-output.txt').read_text()
                self.assertEqual(output.count('layers: leaving out real-loop, end-to-end: 2 binaries\n'),
                                 0 if profile == 'other' else 1)
                calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
                self.assertTrue(all('-E' not in call for call in calls))

    def test_explicit_layers_and_all(self):
        rows = self.replay_layers()
        for index, selected in enumerate((['end-to-end', 'real-loop'], ['all'], ['checks'])):
            with self.subTest(selected=selected):
                self.directory = self.root / f'explicit-{index}'
                result = self.invoke(selected_layers=selected)
                wanted = {r[0] for r in rows if r[4] is None or r[4] in selected or selected == ['all']}
                self.assertEqual({p['binary_id'] for p in result['processes']}, wanted)
                expected = ['real-loop', 'end-to-end'] if index == 0 else (
                    result['layers'] + ['checks'] if index == 1 else ['checks'])
                self.assertEqual(result['selected_layers'], expected)
        (self.root / 'tools/tiers/layers.toml').unlink()
        self.directory = self.root / 'absent-all'
        result = self.invoke(selected_layers=['all'])
        self.assertEqual({p['binary_id'] for p in result['processes']}, {r[0] for r in rows})
        self.assertIsNone(result['selected_layers'])

    def test_caller_filter_selects_within_layers(self):
        self.replay_layers()
        cpu = runpy.run_path(str(HERE / 'cpu.py'))
        for binary, rule, expected in (('app::sim', 'fail', 0), ('app::real', 'fail', 4),
                                       ('app::real', 'warn', 0)):
            with self.subTest(binary=binary, rule=rule):
                self.directory = self.root / f'{binary}-{rule}'
                expression = f'binary_id(={binary}) & test(=a)'
                # Fake nextest supplies the caller's selection; the runner must only narrow it.
                for name, suite in self.listing['rust-suites'].items():
                    suite['testcases']['a']['filter-match'] = {
                        'status': 'matches' if name == binary else 'mismatch'}
                self.prepare()
                command = [*self.command, '-E', expression, f'--no-tests={rule}']
                with contextlib.chdir(self.root), contextlib.redirect_stdout(io.StringIO()):
                    code = cpu['execute_one_process'](command, self.env, self.directory, 'default', {}, self.config)
                self.assertEqual(code, expected)
                result = json.loads((self.directory / 'results.json').read_text())
                self.assertEqual({p['binary_id'] for p in result['processes']},
                                 {binary} if binary == 'app::sim' else set())
                output = (self.directory / 'command-output.txt').read_text()
                self.assertEqual('warning: no tests to run\n' in output, rule == 'warn')
                calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
                for call in calls:
                    if call[:2] == ['nextest', 'list']:
                        self.assertEqual(call.count('-E'), 1)
                        self.assertIn(call[call.index('-E') + 1],
                                      ('binary_id(=app::sim) & test(=a)', 'binary_id(=app::real) & test(=a)'))

    def test_layer_refusals_before_commands(self):
        self.replay_layers()
        for selected in ([], ['bogus'], ['checks', 'bogus'], ['']):
            with self.subTest(selected=selected), self.assertRaises(ValueError):
                self.invoke(selected_layers=selected)
        (self.root / 'tools/tiers/layers.toml').unlink()
        with self.assertRaises(ValueError):
            self.invoke(selected_layers=['checks'])
        for args in (['--layers', 'bogus', 'default'], ['--layers', '', 'default'],
                     ['--layers', 'all', '--again', 'absent', 'app', 'a'],
                     ['--layers', 'all', '--alone', 'absent']):
            with self.subTest(args=args):
                result = subprocess.run([str(HERE / 'one-process'), *args], cwd=self.root, env=self.env,
                                        stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
        self.assertFalse((self.root / 'prefix-argv.jsonl').exists())

    def test_left_out_layer_still_checks_default_filter_proof(self):
        self.replay_layers()
        self.config.write_text('[profile.default]\ndefault-filter="not binary_id(=app::real)"\n')
        with self.assertRaisesRegex(ValueError, 'default-filter disagrees.*app::real'):
            self.invoke()

    def test_narrowed_targets_leave_out_unselected_layers(self):
        self.replay_layers()
        table, selected = self.module['layer_selection'](self.root / 'tools/tiers/layers.toml', 'default', None)
        calls = []
        for binary in ('app::real', 'app::end'):
            del self.listing['rust-suites'][binary]

        def roots(cargo, selection, env):
            calls.append(selection)
            targets = self.metadata['packages'][0]['targets']
            return {('app', 'test', tuple(t['kind']), t['name'], 'same') for t in targets
                    if selection == ['-p', 'app'] or
                    (t['kind'] == ['lib'] and '--lib' in selection) or
                    any(selection[n:n + 2] == ['--test', t['name']] for n in range(len(selection) - 1))}

        with mock.patch.dict(self.module['profile_target_groups'].__globals__, cargo_test_roots=roots), \
                contextlib.redirect_stdout(io.StringIO()):
            for variable in ('TIERS_PRE_PR_TARGETS', 'TIERS_PROFILE_TARGETS'):
                self.prepare()
                with contextlib.chdir(self.root):
                    _, _, _, output, _ = self.module['listed_plan'](
                        self.command, dict(self.env, **{variable: '1'}), 'default', {},
                        self.config, None, self.root / 'tools/tiers/layers.toml', selected_layers=None)
                self.assertIn(b'layers: leaving out real-loop, end-to-end: 2 binaries\n', output)
                call = json.loads((self.root / 'cargo-argv.jsonl').read_text().splitlines()[-1])
                # Metadata can follow the listing only outside the narrowing path.
                self.assertEqual(call[:2], ['nextest', 'list'])
                self.assertIn('--lib', call)
                self.assertIn('sim', call)
                self.assertIn('unplaced', call)
                self.assertNotIn('real', call)
                self.assertNotIn('end', call)
            checks = self.module['profile_target_groups'](['-p', 'app'], self.metadata,
                    [self.module['filterset']('all()')], ['cargo'], self.env, table, ['checks'])
            self.assertEqual(checks, [['-p', 'app', '--test', 'unplaced']])
            broad = self.module['profile_target_groups'](['-p', 'app'], self.metadata,
                    [self.module['filterset']('all()')], ['cargo'], self.env, table, None)
            self.assertEqual(broad, [['-p', 'app']])
        self.assertTrue(calls)

    def test_layers_place_binaries_by_tree_and_ordered_rules(self):
        rows = [
            ('a::suite', 'temper/crates/a/Cargo.toml', 'test', 'temper', 'domain'),
            ('a', 'temper/crates/a/Cargo.toml', 'lib', 'temper', 'step'),
            ('a::bin/a', 'temper/crates/a/Cargo.toml', 'bin', 'temper', 'step'),
            ('a::unplaced', 'temper/crates/a/Cargo.toml', 'test', 'temper', None),
            ('b::suite', 'skein/tests/b/Cargo.toml', 'test', 'skein', 'protocol'),
            ('c::suite', 'smith/tests/c/Cargo.toml', 'test', 'smith', 'world'),
            ('d::suite', 'jig/tests/d/Cargo.toml', 'test', 'jig', 'machine'),
            ('e::suite', 'hearth/tests/e/Cargo.toml', 'test', 'hearth', 'domain'),
            ('x', 'tests/x/Cargo.toml', 'lib', 'repository', 'checks'),
            ('x::t', 'tests/x/Cargo.toml', 'test', 'repository', 'checks'),
            ('x::bin/x', 'tests/x/Cargo.toml', 'bin', 'repository', 'checks'),
        ]
        self.placement_binaries(rows)
        layers = self.root / 'tools/tiers/layers.toml'
        layers.parent.mkdir(parents=True)
        layers.write_text('layers = ["step", "domain", "protocol", "world", "machine"]\n'
                          '[temper]\n"a::suite" = "domain"\n'
                          '"a" = "world"\n"a::bin/a" = "world"\n'
                          '[skein]\n"b::suite" = "protocol"\n'
                          '[smith]\n"c::suite" = "world"\n'
                          '[jig]\n"d::suite" = "machine"\n'
                          '[hearth]\n"e::suite" = "domain"\n')
        result = self.invoke()
        self.assert_placements(result, rows)
        self.assertEqual(result['layers'], ['step', 'domain', 'protocol', 'world', 'machine'])

    def test_absent_layers_still_places_rule_binaries(self):
        rows = [
            ('a', 'temper/crates/a/Cargo.toml', 'lib', 'temper', 'step'),
            ('a::bin/a', 'temper/crates/a/Cargo.toml', 'bin', 'temper', 'step'),
            ('a::suite', 'temper/crates/a/Cargo.toml', 'test', 'temper', None),
            ('x', 'tests/x/Cargo.toml', 'lib', 'repository', 'checks'),
            ('x::t', 'tests/x/Cargo.toml', 'test', 'repository', 'checks'),
        ]
        self.placement_binaries(rows)
        result = self.invoke()
        self.assert_placements(result, rows)
        self.assertEqual(result['layers'], [])

    def test_invalid_layers_refuse_before_commands(self):
        layers = self.root / 'custom-layers.toml'
        for text in ('not toml', '', 'layers = "step"', 'layers = ["step", 1]', 'layers = {}'):
            with self.subTest(text=text):
                layers.write_text(text)
                with self.assertRaises(ValueError):
                    self.invoke(layers=layers)
                self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
                self.assertFalse((self.root / 'prefix-argv.jsonl').exists())


class WorkspaceTests(RunnerFixture):
    def test_workspace_includes_unlisted_packages_and_excludes_non_test_targets(self):
        package = self.metadata['packages'][0]
        package.update(name='temper-fixture', manifest_path=str(self.root / 'temper/crates/fixture/Cargo.toml'),
                       targets=[dict(name='temper_fixture', kind=['lib'], test=True),
                                dict(name='suite', kind=['test'], test=True),
                                dict(name='helper', kind=['bin'], test=False)])
        self.metadata['packages'].append(dict(
            package, id='other-id', name='skein-other',
            manifest_path=str(self.root / 'skein/crates/other/Cargo.toml'),
            targets=[dict(name='skein_other', kind=['lib'], test=True)]))
        self.listing['rust-suites'].clear()
        self.binary('temper-fixture', names=('a',))
        self.binary('temper-fixture::suite', names=('b',))
        self.command[5] = 'temper-fixture'
        layers = self.root / 'layers.toml'
        layers.write_text('layers = ["step", "domain"]\n[temper]\n"temper-fixture::suite" = "domain"\n')
        for table, layer in ((layers, 'domain'), (self.root / 'absent.toml', None)):
            with self.subTest(layer=layer):
                self.directory = self.root / f'run-{layer}'
                result = self.invoke(layers=table)
                self.assertEqual(result['workspace'], [
                    dict(binary_id='skein-other', tree='skein', layer='step', listed=False),
                    dict(binary_id='temper-fixture', tree='temper', layer='step', listed=True),
                    dict(binary_id='temper-fixture::suite', tree='temper', layer=layer, listed=True)])
                self.assertEqual(result['selected'], 2)

    def test_workspace_ids_follow_target_kinds(self):
        rows = [
            (['lib'], 'library'), (['rlib'], 'library'), (['cdylib'], 'library'),
            (['dylib'], 'library'), (['staticlib'], 'library'), (['proc-macro'], 'library'),
            (['rlib', 'cdylib'], 'library'),
            (['bin'], 'bin/target_name'), (['test'], 'target_name'),
            (['example'], 'example/target_name'), (['bench'], 'bench/target_name'),
            (['custom-build'], None),
        ]
        original = self.metadata['packages'][0]
        self.metadata['packages'] = []
        self.listing['rust-suites'].clear()
        expected = []
        for index, (kinds, suffix) in enumerate(rows):
            name = f'package-{index}'
            self.metadata['packages'].append(dict(
                original, id=name, name=name, targets=[dict(name='target_name', kind=kinds, test=True)]))
            if suffix is not None:
                binary_id = name if suffix == 'library' else f'{name}::{suffix}'
                expected.append(dict(binary_id=binary_id, tree='repository', layer='checks', listed=False))
        self.command.extend(['--no-tests', 'pass'])
        self.assertEqual(self.invoke()['workspace'], sorted(expected, key=lambda b: b['binary_id']))

    def test_workspace_listed_does_not_require_selected_tests(self):
        self.command.extend(['-E', 'none()', '--no-tests', 'pass'])
        for case in self.listing['rust-suites']['fixture::suite']['testcases'].values():
            case['filter-match'] = dict(status='mismatch', reason='expression')
        self.listing['rust-suites']['fixture'] = dict(status='skipped')
        result = self.invoke()
        self.assertEqual(result['workspace'], [
            dict(binary_id='fixture', tree='repository', layer='checks', listed=False),
            dict(binary_id='fixture::suite', tree='repository', layer='checks', listed=True)])
        self.assertEqual(result['selected'], 0)
        self.assertEqual(result['processes'], [])


class ReferenceTests(RunnerFixture):
    """New and flagged tests measured after the suite (references.md, 3 and 4)."""

    def test_flag_rule(self):
        for wall, median, flagged in ((0.130, 0.060, True), (0.110, 0.060, False),
                                      (0.060, 0.020, False)):
            with self.subTest(wall=wall, median=median):
                tests = [dict(binary_id='fixture', test_name='test', outcome='passed', wall_s=wall)]
                ledger = dict(binaries={'fixture': dict(tests={'test': [median + 1, median, 0]})})
                self.assertEqual(self.module['pick_references'](tests, ledger),
                                 [('fixture', 'test', 'flagged')] if flagged else [])

    def test_picks_processes_and_suite_records(self):
        self.env['FIXTURE_CASE'] = 'slow'
        self.binary('fixture::failure', names=('b_fails',), native='again')
        self.binary('fixture::early', names=('d',))
        ledger = dict(binaries={'fixture::suite': dict(tests={'b': [0.0], 'c': [0.080]})})
        self.config.write_text('[test-groups.serial]\nmax-threads=1\n[profile.default]\n'
                               '[[profile.default.overrides]]\nfilter="test(=b)"\ntest-group="serial"\n')
        results = self.invoke(ledger=ledger)
        self.assert_counting(results)
        self.assertEqual([(r['binary_id'], r['test_name'], r['why']) for r in results['references']],
                         [('fixture::early', 'd', 'new'), ('fixture::suite', 'a', 'new'),
                          ('fixture::suite', 'b', 'flagged')])
        suite = [p for p in results['processes'] if p['kind'] == 'tests']
        refs = [p for p in results['processes'] if p['kind'] == 'reference']
        empties = [p for p in results['processes'] if p['kind'] == 'empty']
        self.assertEqual([p['tests'] for p in refs], [['d'], ['a'], ['b']])
        self.assertEqual(len(empties), 2)
        for process in refs + empties:
            self.assertIsNone(process['instructions'])
        for empty in empties:
            self.assertEqual((empty['tests'], empty['exit_code'], empty['leaked']), ([], 0, False))
        self.assertGreaterEqual(min(p['start_s'] for p in refs + empties),
                                max(p['start_s'] + p['wall_s'] for p in suite))
        for row in results['references']:
            reference, empty = (results['processes'][row[key]] for key in ('process', 'empty_process'))
            self.assertEqual(row['outcome'], 'passed')
            self.assertEqual(row['cpu_s'], round(max(0, reference['user_s'] + reference['system_s']
                                                   - empty['user_s'] - empty['system_s']), 3))
            self.assertIn(f"REFERENCE {row['binary_id']} {row['test_name']}:",
                          (self.directory / 'command-output.txt').read_text())
        plan = json.loads((self.directory / 'plan.json').read_text())
        self.assertNotIn('reference_picks', plan)
        phase = json.loads((self.directory / 'references-plan.json').read_text())
        self.assertFalse(phase['count'])
        jobs = self.module['split_jobs'](phase, 1)
        self.assertEqual([j['tests'][0]['group'] for j in jobs], [None, None, None, None, 'serial'])
        self.assertEqual({j['tests'][0]['name'] for j in jobs if j['kind'] == 'empty'},
                         {'tiers-one-process: no test'})
        cases = ET.parse(self.directory / 'junit.xml').findall('.//testcase')
        self.assertEqual(len(cases), 5)
        times = json.loads((self.root / 'target/tiers-one-process/times.json').read_text())['default']
        self.assertEqual(times, {binary: {t['test_name']: t['wall_s'] for t in results['tests']
                                        if t['binary_id'] == binary} for binary in times})
        self.assertAlmostEqual(self.record['wall_s'],
                               max(p['start_s'] + p['wall_s'] for p in results['processes'])
                               - min(p['start_s'] for p in suite))

    def assert_rounds(self, results, count, *, highest=False):
        row, = results['references']
        self.assertEqual(len(row['runs']), count)
        references = [p for p in results['processes'] if p['kind'] == 'reference']
        empties = [p for p in results['processes'] if p['kind'] == 'empty']
        self.assertEqual((len(references), len(empties)), (count, count))
        for run, reference, empty in zip(row['runs'], references, empties):
            self.assertEqual(set(run), {'outcome', 'process', 'empty_process', 'cpu_s'})
            self.assertEqual(run['outcome'], 'passed')
            self.assertEqual(results['processes'][run['process']], reference)
            self.assertEqual(results['processes'][run['empty_process']], empty)
            self.assertEqual(run['cpu_s'], round(max(0, reference['user_s'] + reference['system_s']
                                                    - empty['user_s'] - empty['system_s']), 3))
        figure = (max if highest else min)(row['runs'], key=lambda run: run['cpu_s'])
        self.assertEqual({key: row[key] for key in figure}, figure)
        rounds = [[results['processes'][run[key]] for key in ('process', 'empty_process')] for run in row['runs']]
        for before, after in zip(rounds, rounds[1:]):
            self.assertGreaterEqual(min(p['start_s'] for p in after), max(p['start_s'] + p['wall_s'] for p in before))
        output = (self.directory / 'command-output.txt').read_text()
        for number in range(count):
            self.assertIn(f'round {number + 1}, passed, CPU ', output)
        if count > 1:
            self.assertIn(f"REFERENCE FIGURE fixture::suite b: {'highest' if highest else 'lowest'}, CPU {row['cpu_s']} s", output)
        return row

    def test_head_repeats_only_past_first_limit_and_takes_lowest(self):
        self.binary(names=('b',))
        self.env['FIXTURE_CASE'] = 'cpu'
        for limit, count in ((0.001, 3), (100, 1)):
            with self.subTest(limit=limit):
                self.directory = self.root / str(limit)
                results = self.invoke({'gate': {'default': {'references': {'checks': limit}}}}, ledger=dict(binaries={}))
                row = self.assert_rounds(results, count)
                self.assertEqual(row['run_limit_s'], limit)

    def test_flagged_running_limit_uses_ledger_median_or_reference_or_line(self):
        self.binary(names=('b',))
        self.env['FIXTURE_CASE'] = 'cpu'
        for history, reference, line, limit, count in (([100, 10, 20], 0.001, None, 30, 1),
                ([0], 100, None, 100, 1), ([], 0.001, None, 0.001, 3),
                ([], 0.001, 100, 100, 1), ([], None, None, None, 1)):
            with self.subTest(history=history, reference=reference, line=line):
                self.directory = self.root / f'case-{reference}-{line}-{limit}'
                profile = {'references': {} if reference is None else {'checks': reference}}
                if line is not None:
                    profile['reasons'] = [dict(binary='fixture::suite', test='b', cpu_s=line)]
                ledger = dict(binaries={'fixture::suite': dict(tests={'b': [0]}, alone={'b': history})})
                results = self.invoke({'gate': {'default': profile}}, ledger=ledger)
                row = self.assert_rounds(results, count)
                self.assertEqual((row['why'], row['run_limit_s']), ('flagged', limit))

    def test_later_rounds_share_bounds_and_leave_unstarted_runs_unobserved(self):
        self.binary(names=('b',))
        self.env['FIXTURE_CASE'] = 'cpu'
        execute = self.module['execute_plan']
        cpu = execute.__globals__['CPU']
        original = cpu['reference_limit']
        for hard in (False, True):
            with self.subTest(hard=hard):
                self.directory = self.root / str(hard)
                # Let the first round settle, then spend the remainder of the bound
                # computing its limit. No later process may start after that bound.
                def delayed_limit(*args, **kwargs):
                    time.sleep(0.5)
                    return original(*args, **kwargs)
                plan_builder = execute.__globals__['make_plan']
                def bounded(*args):
                    plan = plan_builder(*args)
                    plan['global_timeout'] = None if hard else 0.5
                    return plan
                limits = {'gate': {'default': {'references': {'checks': 0.001}}}}
                if hard:
                    limits['default'] = {'wall_hard_s': 0.5}
                with mock.patch.dict(cpu, reference_limit=delayed_limit), mock.patch.dict(
                        execute.__globals__, make_plan=bounded):
                    results = self.invoke(limits, ledger=dict(binaries={}))
                row, = results['references']
                self.assertEqual(row['runs'][0]['outcome'], 'passed')
                self.assertGreater(row['runs'][0]['cpu_s'], row['run_limit_s'])
                self.assertEqual({key: row[key] for key in row['runs'][0]}, row['runs'][0])
                self.assertEqual(row['runs'][1:], [dict(outcome='unobserved', process=None,
                                 empty_process=None, cpu_s=None)] * 2)
                self.assertEqual([p['kind'] for p in results['processes']], ['tests', 'reference', 'empty'])
                self.assertEqual(results['cancelled'], 'wall-hard' if hard else 'global-timeout')
                self.assertTrue(self.record['starved'])

    def test_reference_failure_is_not_a_suite_failure(self):
        self.binary(names=('a_passes', 'c_reads'), native='again')
        results = self.invoke(ledger=dict(binaries={}))
        self.assertEqual([t['outcome'] for t in results['tests']], ['passed', 'passed'])
        self.assertEqual([(r['test_name'], r['outcome']) for r in results['references']],
                         [('a_passes', 'passed'), ('c_reads', 'failed')])
        self.assertIsNone(results['references'][1]['cpu_s'])
        self.assertIsNone(results['references'][1]['run_limit_s'])
        self.assertEqual(len(results['references'][1]['runs']), 1)
        self.assertEqual([p['kind'] for p in results['processes']],
                         ['tests', 'reference', 'empty', 'reference'])
        self.assertIsNone(ET.parse(self.directory / 'junit.xml').find('.//failure'))
        self.assertEqual(self.module['CPU']['summary'](self.directory)['failures'], [])

    def test_no_references_without_ledger_or_in_again(self):
        results = self.invoke()
        self.assertEqual(results['references'], [])
        self.assertEqual([p['kind'] for p in results['processes']], ['tests'])
        plan = self.module['again_plan'](self.directory, 'fixture::suite', ['a'])
        again = self.root / 'again'
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(self.module['again'](plan, [], self.env, again), 0)
        results = json.loads((again / 'results.json').read_text())
        self.assertEqual(results['references'], [])
        self.assertEqual([p['kind'] for p in results['processes']], ['tests'])

    def test_cancelled_suite_runs_no_references(self):
        self.env['FIXTURE_CASE'] = 'hang'
        self.config.write_text('[profile.default]\nglobal-timeout="1s"\n')
        results = self.invoke(ledger=dict(binaries={}))
        self.assertEqual(results['cancelled'], 'global-timeout')
        self.assertEqual(results['tests'][0]['outcome'], 'passed')
        self.assertEqual(results['references'], [])
        self.assertEqual([p['kind'] for p in results['processes']], ['tests'])

    def test_reference_phase_runs_without_suite_and_crashes_once(self):
        self.env['FIXTURE_CASE'] = 'crash'
        self.invoke()
        plan = json.loads((self.directory / 'plan.json').read_text())
        for native, announced in (('tests', ['b']), ('before', [])):
            with self.subTest(native=native):
                plan['binaries'][0]['path'] = str(NATIVE / native)
                phase = self.module['reference_plan'](plan, [('fixture::suite', 'b', 'base')])
                alone = self.root / f'references-{native}'
                alone.mkdir()
                with contextlib.redirect_stdout(io.StringIO()):
                    results = self.module['execute_plan'](phase, [], self.env, alone, {})
                self.assertEqual([(p['kind'], p['tests']) for p in results['processes']],
                                 [('reference', announced), ('empty', [])])
                self.assertEqual(results['references'][0]['outcome'], 'crashed')
                self.assertEqual(results['references'][0]['process'], 0)
                self.assertIsNone(results['references'][0]['cpu_s'])
                self.assertEqual((results['selected'], results['tests'], results['unobserved']), (0, [], []))

    def test_reference_bounds_start_with_suite(self):
        for hard in (False, True):
            with self.subTest(hard=hard):
                self.directory = self.root / str(hard)
                self.env['FIXTURE_CASE'] = 'slow'
                self.config.write_text('[profile.default]\nglobal-timeout="60s"\n')
                self.prepare()
                # Supply a subsecond bound directly: the config's duration syntax is integral.
                execute = self.module['execute_plan']
                original = execute.__globals__['make_plan']
                def bounded(*args):
                    plan = original(*args)
                    plan['global_timeout'] = None if hard else 0.35
                    return plan
                with mock.patch.dict(execute.__globals__, make_plan=bounded):
                    results = self.invoke({'default': {'wall_hard_s': 0.35}} if hard else {},
                                          ledger=dict(binaries={}))
                self.assertEqual(results['cancelled'], 'wall-hard' if hard else 'global-timeout')
                self.assertTrue(self.record['starved'])
                self.assertEqual([t['outcome'] for t in results['tests']], ['passed'] * 3)
                self.assertTrue(results['references'])
                self.assertTrue(any(r['cpu_s'] is None for r in results['references']))
                self.assertLess(self.record['wall_s'], 0.7)


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
                results_path = self.directory / 'results.json'
                results_text = results_path.read_text()
                results = json.loads(results_text)
                del results['workspace']
                results_path.write_text(json.dumps(results))
                self.assertEqual(cpu['summary'](self.directory), summary)
                results_path.write_text(results_text)
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

    def assert_ledger_counts(self, result, data, ledger, output, main=None):
        self.assert_counting(data)
        self.assertEqual(result['counting'], data['counting'])
        self.assertEqual([p['instructions'] for p in result['processes']],
                         [p['instructions'] for p in data['processes']])
        processes = [p for p in result['processes'] if p['kind'] in ('tests', 'fresh')]
        self.assertTrue(processes)
        for process in result['processes']:
            if process['kind'] not in ('tests', 'fresh'):
                self.assertIsNone(process['instructions'])
        if result['counting']['counted']:
            for process in processes:
                self.assertIs(type(process['instructions']), int)
                self.assertGreater(process['instructions'], 0)
            count = sum(p['instructions'] for p in processes)
            own = [dict(commit=data['commit'], instructions=count)]
        else:
            count, own = None, []
            for process in result['processes']:
                self.assertIsNone(process['instructions'])
            self.assertIn(f"tiers: not counting instructions: {data['counting']['why']}\n", output)
        previous = [] if main is None else main['binaries']['fixture::suite']['counts']
        self.assertEqual(ledger['binaries']['fixture::suite']['counts'], previous + own)
        main_count = previous[-1]['instructions'] if previous else None
        self.assertEqual([(r['tree'], r['layer']) for r in result['layers']], [('repository', 'checks')])
        self.assertEqual([r['tree'] for r in result['trees']], ['repository'])
        for row in result['layers'] + result['trees']:
            self.assertEqual(row['instructions'], count)
            self.assertEqual(row['main_instructions'], main_count)
        line = next(line for line in output.splitlines() if line.startswith('layer repository checks:'))
        ending = f", instructions {count if count is not None else 'none'}"
        if main is not None:
            ending += f", main instructions {main_count if main_count is not None else 'none'}"
        self.assertTrue(line.endswith(ending), line)
        self.assertIn(line, (self.directory / 'command-output.txt').read_text())
        return count

    def test_execute_writes_readable_ledger(self):
        cpu = runpy.run_path(str(HERE / 'cpu.py'))
        execute = cpu['execute_one_process']
        execute.__globals__['runpy'] = mock.Mock(run_path=lambda path: {'run': self.run})
        # Split one binary across processes so its ledger must sum their counts.
        self.run.__globals__['LANES'] = 2
        self.prepare()
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = execute(self.command, self.env, self.directory, 'default', {}, self.config)
        self.assertEqual(code, 0)
        ledger = cpu['ledger'](self.directory / 'ledger.json')
        result = cpu['summary'](self.directory)
        data = json.loads((self.directory / 'results.json').read_text())
        self.assertEqual(len(result['processes']), 2)
        count = self.assert_ledger_counts(result, data, ledger, output.getvalue())
        counts = [] if count is None else [dict(commit=data['commit'], instructions=count)]
        self.assertEqual(ledger, dict(ledger=6, profile='default', budgets={}, moves=[], coverage_reasons=[], binaries={
            'fixture::suite': dict(tree='repository', layer='checks',
                                  cpu=[sum(p['cpu_s'] for p in result['processes'])],
                                  tests={t['test_name']: [t['wall_s']] for t in result['tests']},
                                  alone={}, counts=counts, checks={})}))
        self.assertEqual(set(ledger['binaries']['fixture::suite']['tests']), {'a', 'b', 'c'})

        self.directory = self.root / 'second run with spaces'
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = execute(self.command, self.env, self.directory, 'default', {}, self.config, ledger=ledger)
        self.assertEqual(code, 0)
        second_ledger = cpu['ledger'](self.directory / 'ledger.json')
        second = cpu['summary'](self.directory, ledger=ledger)
        second_data = json.loads((self.directory / 'results.json').read_text())
        self.assert_ledger_counts(second, second_data, second_ledger, output.getvalue(), main=ledger)
        entry = second_ledger['binaries']['fixture::suite']
        self.assertEqual(entry['cpu'], [*ledger['binaries']['fixture::suite']['cpu'], second['suite']['cpu_s']])
        self.assertEqual(entry['tests'], {t['test_name']: [*ledger['binaries']['fixture::suite']['tests'][t['test_name']],
                                                        t['wall_s']] for t in second['tests']})

    def test_walls_replace_the_profiles_global_timeout(self):
        self.env['FIXTURE_CASE'] = 'hang'
        self.binary(names=('b',))
        self.config.write_text('[profile.default]\nglobal-timeout="60s"\n')
        walls = self.root / 'walls.toml'
        walls.write_text('[profile.default]\nglobal-timeout="1s"\n')
        self.prepare()
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'))
        result = subprocess.run([str(HERE / 'one-process'), '--out', str(self.directory),
                                 '--walls', str(walls), 'default', '-p', 'fixture'],
                                cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        plan = json.loads((self.directory / 'plan.json').read_text())
        self.assertEqual(plan['global_timeout'], 1)
        again = self.module['again_plan'](self.directory, 'fixture::suite', ['b'])
        self.assertEqual(again['global_timeout'], 1)
        run = json.loads((self.directory / 'run.json').read_text())
        self.assertTrue(run['starved'])
        self.assertLess(run['wall_s'], 4)
        results = json.loads((self.directory / 'results.json').read_text())
        self.assertEqual(results['cancelled'], 'global-timeout')
        self.assertEqual([(t['test_name'], t['outcome']) for t in results['tests']], [('b', 'cancelled')])
        failure = ET.parse(self.directory / 'junit.xml').find('.//failure')
        self.assertEqual(failure.get('type'), 'test abort')
        self.assertIn('signal 15 (SIGTERM)', failure.get('message'))
        self.assertEqual(self.module['CPU']['summary'](self.directory)['verdict'], 'starved')
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertIn(str(self.config), calls[0])
        self.assertNotIn(str(walls), calls[0])

    def test_walls_refuse_before_commands(self):
        walls = self.root / 'walls.toml'
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'))
        for label, source, arguments, message in (
                ('another key', '[profile.default]\nglobal-timeout="1s"\nslow-timeout="1s"\n',
                 ['default'], 'global-timeout alone'),
                ('another table', '[profile.default]\nglobal-timeout="1s"\n[test-groups]\n',
                 ['default'], 'only profile tables'),
                ('duration', '[profile.default]\nglobal-timeout="soon"\n',
                 ['default'], "invalid duration 'soon'"),
                ('missing profile', '[profile.fuzzy]\nglobal-timeout="1s"\n',
                 ['default'], "no wall for profile 'default'"),
                ('again', '[profile.default]\nglobal-timeout="1s"\n',
                 ['--again', str(self.root / 'no plan'), 'fixture::suite', 'b'],
                 '--walls is not supported with --again')):
            with self.subTest(label=label):
                for name in ('cargo-argv.jsonl', 'prefix-argv.jsonl'):
                    (self.root / name).unlink(missing_ok=True)
                walls.write_text(source)
                result = subprocess.run([str(HERE / 'one-process'), '--walls', str(walls), *arguments],
                                        cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                        capture_output=True, text=True, timeout=4)
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
                self.assertFalse((self.root / 'prefix-argv.jsonl').exists())
                self.assertIn(message, result.stderr)
                if label != 'again':
                    self.assertIn(str(walls), result.stderr)

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


class BaseTests(RunnerFixture):
    """The head's flagged tests measured at the base (references.md, 5)."""

    def head(self, references=(), profile='default'):
        self.directory.mkdir(exist_ok=True)
        (self.directory / 'results.json').write_text(json.dumps(dict(profile=profile, references=references)))

    def flagged(self, name='b', binary='fixture::suite', why='flagged'):
        return dict(binary_id=binary, test_name=name, why=why)

    def invoke_base(self, *, directory=None, default_out=False, extra=(), hold=None):
        self.prepare()
        env = dict(self.env, PATH=str(self.root) + os.pathsep + self.env['PATH'],
                   TIERS_WRAPPER=str(self.root / 'prefix'))
        if hold is not None:
            env['TIERS_HOLD'] = hold
        command = [str(HERE / 'one-process'), '--alone', str(self.directory)]
        if not default_out:
            command += ['--out', str(directory or self.root / 'base output')]
        return subprocess.run([*command, *extra], cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                              capture_output=True, text=True, timeout=10)

    def test_base_runs_three_rounds_and_takes_highest(self):
        self.head([self.flagged()])
        self.binary(names=('b',))
        self.env['FIXTURE_CASE'] = 'cpu'
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = self.root / 'base output'
        results = json.loads((base / 'results.json').read_text())
        self.assert_uncounted(results, base)
        row, = results['references']
        self.assertEqual(len(row['runs']), 3)
        self.assertIsNone(row['run_limit_s'])
        self.assertEqual([p['kind'] for p in results['processes']], ['reference', 'empty'] * 3)
        for number, run in enumerate(row['runs']):
            self.assertEqual((run['outcome'], run['process'], run['empty_process']), ('passed', number * 2, number * 2 + 1))
            reference, empty = (results['processes'][run[key]] for key in ('process', 'empty_process'))
            self.assertEqual(run['cpu_s'], round(max(0, reference['user_s'] + reference['system_s']
                                                    - empty['user_s'] - empty['system_s']), 3))
            if number:
                before = results['processes'][(number - 1) * 2:number * 2]
                self.assertGreaterEqual(min(reference['start_s'], empty['start_s']),
                                        max(p['start_s'] + p['wall_s'] for p in before))
        figure = max(row['runs'], key=lambda run: run['cpu_s'])
        self.assertEqual({key: row[key] for key in figure}, figure)
        output = (base / 'command-output.txt').read_text()
        for number in range(3):
            self.assertIn(f'round {number + 1}, passed, CPU ', output)
        self.assertIn(f"REFERENCE FIGURE fixture::suite b: highest, CPU {row['cpu_s']} s", output)

    def test_two_packages_list_only_their_targets(self):
        self.binary('zebra', names=('c',))
        self.listing['rust-suites']['zebra'].update({
            'package-id': 'zebra-id', 'kind': 'lib', 'binary-name': 'zebra'})
        self.metadata['packages'].insert(0, dict(
            self.metadata['packages'][0], id='zebra-id', name='zebra',
            targets=[dict(name='zebra', kind=['rlib'], test=True)]))
        self.metadata['workspace_members'].append('zebra-id')
        for package in self.metadata['packages']:
            listing = dict(self.listing, **{'rust-suites': {
                binary: suite for binary, suite in self.listing['rust-suites'].items()
                if suite['package-id'] == package['id']}})
            (self.root / f"listing-{package['name']}.json").write_text(json.dumps(listing))
        self.head([self.flagged('c', 'zebra'), self.flagged('b'), self.flagged('a')], profile='candidate')
        self.config.write_text('[profile.default]\n[profile.candidate]\nglobal-timeout="2s"\n')
        self.metadata['target_directory'] = str(self.root / 'target')
        result = self.invoke_base(default_out=True, extra=('-E', 'all()', '--no-fail-fast'),
                                  hold=str(self.root / 'hold'))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        listing_args = ['nextest', 'list', '--message-format', 'json', '--profile', 'candidate',
                        '--config-file', str(self.config)]
        self.assertEqual(calls, [['metadata', '--format-version', '1', '--no-deps'],
                                 ['metadata', '--format-version', '1', '--no-deps'],
                                 [*listing_args, '-p', 'fixture', '--test', 'suite', '-E', 'all()'],
                                 [*listing_args, '-p', 'zebra', '--lib', '-E', 'all()']])
        prefixed = [json.loads(line) for line in (self.root / 'prefix-argv.jsonl').read_text().splitlines()]
        self.assertEqual([call[1:] for call in prefixed[:2]], calls[2:])
        self.assertEqual(len(prefixed), 5)
        events = [json.loads(line) for line in (self.root / 'hold-events.jsonl').read_text().splitlines()]
        self.assertEqual(events[0][0], 'hold-start')
        self.assertEqual([event[1] for event in events if event[0] == 'cargo'], calls)
        self.assertEqual(events[-1], ['hold-end', result.stdout.splitlines()[-1]])
        base = Path(result.stdout.splitlines()[0])
        output = (base / 'command-output.txt').read_text()
        self.assertLess(output.index('fixture::suite'), output.index('"zebra"'))
        self.assertEqual(output.count('fixture listing prefix'), 2)
        self.assertEqual(output.count('fixture listing suffix'), 2)
        results = json.loads((base / 'results.json').read_text())
        self.assertEqual([(r['binary_id'], r['test_name'], r['why'], r['outcome'])
                          for r in results['references']],
                         [('fixture::suite', 'a', 'base', 'passed'),
                          ('fixture::suite', 'b', 'base', 'passed'), ('zebra', 'c', 'base', 'passed')])
        self.assertEqual(len(results['processes']), 15)
        for row in results['references']:
            reference, empty = (results['processes'][row[key]] for key in ('process', 'empty_process'))
            self.assertEqual((reference['binary_id'], reference['kind'], reference['tests']),
                             (row['binary_id'], 'reference', [row['test_name']]))
            self.assertEqual((empty['binary_id'], empty['kind'], empty['tests']),
                             (row['binary_id'], 'empty', []))
            self.assertIsInstance(row['cpu_s'], (int, float))

    def test_one_package_combines_targets_in_binary_id_order(self):
        self.binary('fixture', names=('a',))
        self.listing['rust-suites']['fixture'].update({'kind': 'lib', 'binary-name': 'fixture'})
        self.head([self.flagged(), self.flagged('a', 'fixture')])
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertEqual(calls, [['metadata', '--format-version', '1', '--no-deps'],
                                 ['nextest', 'list', '--message-format', 'json', '--profile', 'default',
                                  '--config-file', str(self.config), '-p', 'fixture', '--lib', '--test', 'suite']])
        results = json.loads((self.root / 'base output/results.json').read_text())
        self.assertEqual([(r['binary_id'], r['outcome']) for r in results['references']],
                         [('fixture', 'passed'), ('fixture::suite', 'passed')])
        self.assertEqual(len(results['processes']), 12)

    def test_binary_target_uses_bin_flag(self):
        self.metadata['packages'][0]['targets'].append(dict(name='tool', kind=['bin'], test=True))
        self.binary('fixture::bin/tool', names=('a',))
        self.listing['rust-suites']['fixture::bin/tool'].update({'kind': 'bin', 'binary-name': 'tool'})
        self.head([self.flagged('a', 'fixture::bin/tool')])
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertEqual(calls[1:], [
            ['nextest', 'list', '--message-format', 'json', '--profile', 'default',
             '--config-file', str(self.config), '-p', 'fixture', '--bin', 'tool']])
        results = json.loads((self.root / 'base output/results.json').read_text())
        row, = results['references']
        self.assertEqual((row['binary_id'], row['outcome']), ('fixture::bin/tool', 'passed'))

    def test_example_and_bench_target_flags(self):
        for kind in ('example', 'bench'):
            with self.subTest(kind=kind):
                self.assertEqual(self.module['target_flags'](f'fixture::{kind}/tool'), ['--' + kind, 'tool'])

    def test_listing_build_environments_are_unioned(self):
        first = {'rust-suites': {'fixture::suite': {}}, 'rust-build-meta': {
            'target-directory': '/target', 'base-output-directories': ['debug'],
            'non-test-binaries': {'fixture-id': [{'name': 'one'}]},
            'build-script-out-dirs': {'fixture-id': 'out/fixture'},
            'linked-paths': ['debug/build/shared/out']}}
        second = {'rust-suites': {'zebra': {}}, 'rust-build-meta': {
            'target-directory': '/target', 'base-output-directories': ['debug', 'custom'],
            'non-test-binaries': {'fixture-id': [{'name': 'one'}, {'name': 'two'}],
                                  'zebra-id': [{'name': 'zebra'}]},
            'build-script-out-dirs': {'zebra-id': 'out/zebra'},
            'linked-paths': ['debug/build/shared/out', 'debug/build/zebra/out']}}
        listing = self.module['merge_listings']([first, second])
        self.assertEqual(set(listing['rust-suites']), {'fixture::suite', 'zebra'})
        self.assertEqual(listing['rust-build-meta'], {
            'target-directory': '/target', 'base-output-directories': ['debug', 'custom'],
            'non-test-binaries': {'fixture-id': [{'name': 'one'}, {'name': 'two'}],
                                  'zebra-id': [{'name': 'zebra'}]},
            'build-script-out-dirs': {'fixture-id': 'out/fixture', 'zebra-id': 'out/zebra'},
            'linked-paths': ['debug/build/shared/out', 'debug/build/zebra/out']})

    def test_listings_with_different_target_directories_are_refused(self):
        listings = [{'rust-suites': {}, 'rust-build-meta': {'target-directory': target}}
                    for target in ('/first', '/second')]
        with self.assertRaisesRegex(ValueError, 'different target directories'):
            self.module['merge_listings'](listings)

    def test_head_base_and_judgement_meet(self):
        self.env['FIXTURE_CASE'] = 'slow'
        ledger = dict(profile='default', budgets={}, moves=[], binaries={'fixture::suite': dict(
            tree='repository', layer='checks', cpu=[1.0], counts=[],
            tests={'a': [10], 'b': [0], 'c': [10]}, alone={'b': [100]})})
        head = self.invoke(ledger=ledger)
        self.assertEqual([(r['test_name'], r['why']) for r in head['references']], [('b', 'flagged')])
        times = self.root / 'target/tiers-one-process/times.json'
        times_before = times.read_bytes()
        (self.root / 'cargo-argv.jsonl').unlink()
        (self.root / 'prefix-argv.jsonl').unlink()
        self.env['FIXTURE_CASE'] = 'pass'
        result = self.invoke_base(extra=('-E', 'test(=b)', '--no-fail-fast'))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = self.root / 'base output'
        self.assertEqual(result.stdout.splitlines()[0], str(base))
        self.assertEqual({p.name for p in base.iterdir()},
                         {'plan.json', 'run.json', 'results.json', 'command-output.txt'})
        results = json.loads((base / 'results.json').read_text())
        self.assertEqual((results['tests'], results['unobserved']), ([], []))
        self.assertEqual([p['kind'] for p in results['processes']], ['reference', 'empty'] * 3)
        self.assertEqual(results['processes'][0]['tests'], ['b'])
        row, = results['references']
        self.assertEqual((row['binary_id'], row['test_name'], row['why'], row['outcome']),
                         ('fixture::suite', 'b', 'base', 'passed'))
        self.assertIsInstance(row['cpu_s'], (int, float))
        self.assertGreaterEqual(row['cpu_s'], 0)
        self.assertEqual(times.read_bytes(), times_before)
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertEqual(calls, [['metadata', '--format-version', '1', '--no-deps'],
                                 ['nextest', 'list', '--message-format', 'json', '--profile', 'default',
                                  '--config-file', str(self.config), '-p', 'fixture', '--test', 'suite',
                                  '-E', 'test(=b)']])
        prefixed = [json.loads(line) for line in (self.root / 'prefix-argv.jsonl').read_text().splitlines()]
        self.assertEqual(len(prefixed), 4)
        self.assertEqual(prefixed[0][1:], calls[1])
        self.assertEqual(prefixed[1][-2:], ['lanes', str(base / 'plan.json')])
        summary = self.module['CPU']['summary'](
            self.directory, {'gate': {'default': {'references': {'checks': 1}}}}, ledger=ledger, base=base)
        judged, = summary['references']
        self.assertEqual((judged['main_from'], judged['main_cpu_s']), ('base', row['cpu_s']))

    def test_no_flagged_builds_nothing(self):
        self.head([self.flagged(why='new')])
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = self.root / 'base output'
        self.assertEqual({p.name for p in base.iterdir()},
                         {'plan.json', 'run.json', 'results.json', 'command-output.txt'})
        results = json.loads((base / 'results.json').read_text())
        self.assertEqual((results['references'], results['processes'], results['tests'], results['unobserved']),
                         ([], [], [], []))
        self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
        self.assertFalse((self.root / 'target').exists())

    def test_missing_and_filtered_names_at_base(self):
        self.head([self.flagged('gone'), self.flagged(binary='absent::bin/tool'), self.flagged(),
                   self.flagged(binary='fixture::removed'), self.flagged('a', why='new')])
        # Even an over-inclusive listing cannot supply a target absent from metadata.
        self.binary('fixture::removed', names=('b',))
        self.listing['rust-suites']['fixture::suite']['testcases']['b']['filter-match'] = {
            'status': 'mismatch', 'reason': 'default-filter'}
        self.config.write_text('[profile.default]\ndefault-filter="none()"\n')
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = self.root / 'base output'
        results = json.loads((base / 'results.json').read_text())
        self.assertEqual([(r['binary_id'], r['test_name'], r['why'], r['outcome'])
                          for r in results['references']],
                         [('absent::bin/tool', 'b', 'base', 'missing'),
                          ('fixture::removed', 'b', 'base', 'missing'),
                          ('fixture::suite', 'b', 'base', 'passed'),
                          ('fixture::suite', 'gone', 'base', 'missing')])
        for row in results['references']:
            if row['outcome'] == 'missing':
                self.assertEqual((row['cpu_s'], row['process'], row['empty_process']), (None, None, None))
                self.assertEqual((row['runs'], row['run_limit_s']), ([], None))
                self.assertIn(f"{row['binary_id']} {row['test_name']}: base, missing",
                              (base / 'command-output.txt').read_text())
        self.assertEqual([p['tests'] for p in results['processes']], [['b'], []] * 3)
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertEqual(calls, [['metadata', '--format-version', '1', '--no-deps'],
                                 ['nextest', 'list', '--message-format', 'json', '--profile', 'default',
                                  '--config-file', str(self.config), '-p', 'fixture', '--test', 'suite']])
        self.assertFalse((self.root / 'target/tiers-one-process/times.json').exists())

    def test_all_missing_has_no_process(self):
        self.metadata['packages'][0]['targets'].append(dict(name='disabled', kind=['test'], test=False))
        self.metadata['packages'].append(dict(self.metadata['packages'][0], id='dependency-id', name='dependency'))
        self.head([self.flagged(binary=binary) for binary in
                   ('absent', 'fixture::removed', 'fixture::disabled', 'dependency::suite')])
        result = self.invoke_base()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = self.root / 'base output'
        self.assertEqual({p.name for p in base.iterdir()},
                         {'plan.json', 'run.json', 'results.json', 'command-output.txt'})
        results = json.loads((base / 'results.json').read_text())
        self.assertEqual((results['processes'], results['tests'], results['unobserved']), ([], [], []))
        self.assertEqual([(row['binary_id'], row['outcome'], row['cpu_s']) for row in results['references']],
                         [(binary, 'missing', None) for binary in
                          ('absent', 'dependency::suite', 'fixture::disabled', 'fixture::removed')])
        calls = [json.loads(line) for line in (self.root / 'cargo-argv.jsonl').read_text().splitlines()]
        self.assertEqual(calls, [['metadata', '--format-version', '1', '--no-deps']])
        self.assertFalse((self.root / 'prefix-argv.jsonl').exists())

    def test_nonpasses_still_exit_zero_and_use_head_profile_bound(self):
        for scenario, name, native, outcome in (
                ('pass', 'b_fails', 'again', 'failed'), ('crash', 'b', 'tests', 'crashed'),
                ('starved', 'a', 'tests', 'starved'), ('hang', 'b', 'tests', 'cancelled')):
            with self.subTest(scenario=scenario):
                self.head([self.flagged(name)], profile='candidate')
                self.binary(names=(name,), native=native)
                self.env['FIXTURE_CASE'] = scenario
                bound = '100ms' if scenario == 'hang' else '2s'
                self.config.write_text('[profile.default]\nglobal-timeout="10s"\n'
                                       f'[profile.candidate]\nglobal-timeout="{bound}"\n')
                base = self.root / f'base-{scenario}'
                result = self.invoke_base(directory=base)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                results = json.loads((base / 'results.json').read_text())
                row, = results['references']
                self.assertEqual((row['outcome'], row['cpu_s']), (outcome, None))
                plan = json.loads((base / 'plan.json').read_text())
                self.assertEqual((plan['profile'], plan['global_timeout']), ('candidate', 0.1 if scenario == 'hang' else 2))
                self.assertEqual(results['cancelled'], 'global-timeout' if scenario == 'hang' else None)
                self.assertEqual([run['outcome'] for run in row['runs']],
                                 [outcome, 'unobserved', 'unobserved'] if scenario == 'hang' else [outcome] * 3)

    def test_invalid_invocations_exit_two(self):
        self.head()
        valid = (self.directory / 'results.json').read_bytes()
        fresh = self.root / 'empty directory'
        fresh.mkdir()
        occupied = self.root / 'occupied'
        occupied.write_text('not a directory')
        for label, head, out, extra, message in (
                ('missing', None, None, (), 'results.json'),
                ('no references', b'{"profile":"default"}', None, (), 'references'),
                ('existing empty', valid, fresh, (), 'fresh'),
                ('existing file', valid, occupied, (), 'fresh'),
                ('budgets', valid, None, ('--budgets', 'unused'), '--budgets'),
                ('walls', valid, None, ('--walls', 'unused'), '--walls'),
                ('ledger', valid, None, ('--ledger', 'unused'), '--ledger'),
                ('gate', valid, None, ('--gate',), '--gate'),
                ('again', valid, None, ('--again', 'unused'), '--again')):
            with self.subTest(label=label):
                path = self.directory / 'results.json'
                if head is None:
                    path.unlink()
                else:
                    path.write_bytes(head)
                result = self.invoke_base(directory=out, extra=extra)
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertIn(message, result.stderr)
                self.assertFalse((self.root / 'cargo-argv.jsonl').exists())
                self.assertFalse((self.root / 'base output').exists())

    def test_default_output_and_hold_cover_metadata_and_reference_phase(self):
        self.head([self.flagged()])
        self.metadata['target_directory'] = str(self.root / 'target')
        result = self.invoke_base(default_out=True, hold=str(self.root / 'hold'))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        base = Path(result.stdout.splitlines()[0])
        self.assertEqual(base.parent, self.root / 'target/tiers-one-process/runs')
        self.assertTrue((base / 'results.json').is_file())
        events = [json.loads(line) for line in (self.root / 'hold-events.jsonl').read_text().splitlines()]
        self.assertEqual(events[0], ['hold-start', [], os.getpgrp(), os.getsid(0)])
        self.assertEqual(events[1][0:2], ['cargo', ['metadata', '--format-version', '1', '--no-deps']])
        self.assertEqual(events[-1], ['hold-end', result.stdout.splitlines()[-1]])
        self.assertEqual(sum(e[0] == 'hold-start' for e in events), 1)


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
        plan['layers'] = ['step', 'domain']
        plan['binaries'][0].update(tree='temper', layer='domain')
        plan['groups']['process-of-its-own'] = 1
        plan['binaries'][0]['tests'][0]['group'] = 'process-of-its-own'
        path.write_text(json.dumps(plan))
        result = self.invoke_again(['c_reads', 'a_passes'])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(result.stdout.splitlines()[0], str(self.again_directory))
        results = json.loads((self.again_directory / 'results.json').read_text())
        self.assert_uncounted(results, self.again_directory)
        self.assertEqual([(t['binary_id'], t['test_name'], t['outcome']) for t in results['tests']],
                         [('fixture::suite', 'a_passes', 'passed'),
                          ('fixture::suite', 'c_reads', 'passed')])
        self.assertEqual(len(results['processes']), 1)
        self.assertEqual(results['processes'][0]['tests'], ['a_passes', 'c_reads'])
        self.assertEqual((results['processes'][0]['tree'], results['processes'][0]['layer']),
                         ('temper', 'domain'))
        self.assertEqual(results['layers'], [])
        self.assertEqual(results['workspace'], [])
        self.assertEqual(results['unobserved'], [])
        self.assertEqual({p.name for p in self.again_directory.iterdir()},
                         {'plan.json', 'run.json', 'results.json', 'junit.xml', 'command-output.txt'})
        record = json.loads((self.again_directory / 'run.json').read_text())
        self.assertEqual((record['mode'], record['lanes'], record['finished']), ('one-process', 1, True))
        again_plan = json.loads((self.again_directory / 'plan.json').read_text())
        self.assertEqual(again_plan['global_timeout'], plan['global_timeout'])
        for key in ('path', 'cwd', 'env', 'tests', 'tree', 'layer'):
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
                 ('--budgets', 'unused'), '--budgets'),
                ('gate', 'fixture::suite', ['a_passes'], None, None, ('--gate',), '--gate')):
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

    def test_waiting_hold_delays_metadata_until_held(self):
        command, env = self.prepare_command(str(self.root / 'hold') + ' waiting')
        result = subprocess.run(command, cwd=self.root, env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 4, result.stderr)
        events = self.events()
        self.assertEqual(events[0][0:2], ['hold-start', ['waiting']])
        self.assertEqual(events[1], ['held'], events)
        self.assertEqual(events[2][0:2], ['cargo', ['metadata', '--format-version', '1', '--no-deps']])
        self.assertEqual(events[-1], ['hold-end', result.stdout.splitlines()[-1]])
        self.assertEqual(sum(e[0] == 'hold-start' for e in events), 1)
        lines = result.stderr.splitlines()
        self.assertEqual(lines[0:3], ['one-process: hold: waiting for a build place',
                                     'smith-check: waiting', 'smith-check: admitted'])
        self.assertRegex(lines[3], r'^one-process: hold: held after [0-9]+ s$')
        self.assertEqual(sum(line == 'smith-check: waiting' for line in lines), 1)
        self.assertEqual(sum(line == 'smith-check: admitted' for line in lines), 1)
        self.assertEqual(sum(line == 'one-process: hold: waiting for a build place' for line in lines), 1)
        self.assertIn('default: 0 tests', result.stdout)

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
        self.binary(names=('a', 'b', 'c', 'ignored'))
        self.command.extend(['-Pdefault', '--status-level=fail', '--no-capture',
                             '--failure-output', 'immediate', '--color', 'never', '--retries=0'])
        result = self.invoke()
        self.assertEqual(result['selected'], 4)
        self.assertEqual([t['outcome'] for t in result['tests']], ['passed'] * 4)
        self.assertEqual(len(result['processes']), 1)
        process = result['processes'][0]
        self.assertEqual(process['kind'], 'tests')
        self.assertEqual(process['tests'], ['a', 'b', 'c', 'ignored'])
        for key in ('user_s', 'system_s'):
            self.assertTrue(math.isfinite(process[key]))
            self.assertGreaterEqual(process[key], 0)
        self.assertEqual((self.record['mode'], self.record['lanes']), ('one-process', 1))
        self.assertEqual({p.name for p in self.directory.iterdir()},
                         {'run.json', 'results.json', 'junit.xml', 'command-output.txt', 'plan.json'})
        self.assertEqual(self.directory.stat().st_mode & 0o777, 0o700)
        cases = ET.parse(self.directory / 'junit.xml').findall('.//testcase')
        self.assertEqual([(c.get('classname'), c.get('name')) for c in cases],
                         [('fixture::suite', n) for n in ('a', 'b', 'c', 'ignored')])
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
        self.assert_counting(result)
        self.assertEqual(result['tests'][0]['outcome'], 'passed', result['tests'][0]['output'])

    def test_crash_fresh_and_before_first(self):
        self.env['FIXTURE_CASE'] = 'crash'
        self.metadata['packages'][0]['manifest_path'] = str(self.root / 'temper/crates/fixture/Cargo.toml')
        layers = self.root / 'custom-layers.toml'
        layers.write_text('layers = ["step", "domain"]\n[temper]\n"fixture::suite" = "domain"\n')
        result = self.invoke(layers=Path('custom-layers.toml'))
        self.assert_counting(result)
        self.assertEqual(result['layers'], ['step', 'domain'])
        self.assertEqual({p['kind']: (p['tree'], p['layer']) for p in result['processes']},
                         {'tests': ('temper', 'domain'), 'fresh': ('temper', 'domain'),
                          'alone': ('temper', 'domain')})
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

    def test_repair_case_prioritizes_its_original_process_and_preserves_coverage(self):
        self.run.__globals__['LANES'] = 1
        self.binary(names=('a', 'b'))
        self.binary('fixture::second', names=('c', 'd'))
        self.env['TIERS_PRE_PR_REPAIR_CASE'] = json.dumps(['fixture::second', 'd'])
        result = self.invoke()
        self.assertEqual([(p['binary_id'], p['tests']) for p in result['processes']],
                         [('fixture::second', ['c', 'd']), ('fixture::suite', ['a', 'b'])])
        self.assertEqual(result['unobserved'], [])
        self.directory = self.root / 'missing-case'
        self.env['TIERS_PRE_PR_REPAIR_CASE'] = json.dumps(['fixture::suite', 'd'])
        with self.assertRaisesRegex(ValueError, 'repair case absent'):
            self.invoke()
        self.assertTrue((self.directory / 'repair-case-missing').is_file())
        self.assertFalse((self.directory / 'results.json').exists())

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
            ('[[profile.default.overrides]]\nfilter="all()"\nplatform="cfg(unix)"\npriority=5', [], {}, {}),
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
