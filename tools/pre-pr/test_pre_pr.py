"""Delivery selection, known reds and inherited counts (tools/pre-pr/design.md, 3, 5, 9)."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location('pre_pr', Path(__file__).with_name('pre_pr.py'))
PRE_PR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PRE_PR)


class DeliverySelection(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory(prefix='pre-pr-selection-')
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.env = os.environ.copy()
        for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR'):
            self.env.pop(name, None)
        self.command('git', 'init', '-q')
        self.command('git', 'config', 'user.name', 'Fixture')
        self.command('git', 'config', 'user.email', 'fixture@example.invalid')
        self.command('git', 'config', 'commit.gpgsign', 'false')
        self.write('Cargo.toml', '[workspace]\nmembers = ["api"]\n')
        self.write('api/Cargo.toml', '[package]\nname = "api"\n')
        self.write('api/src/lib.rs', 'pub fn value() {}\n')
        self.write('guide.md', '# Guide\n')
        self.write('.config/gate/read-by-tests.txt', 'api/docs/input.txt\n')
        self.write('.config/gate/always.txt', 'api shared/fixtures/\n')
        self.write('tools/tiers/packages.toml', '[test-binaries]\n')
        source = Path(__file__).resolve().parents[1] / 'citations'
        for name in ('citations.py', 'sized.txt'):
            self.write('tools/citations/' + name, (source / name).read_text())
        for tree in ('skein', 'smith', 'jig', 'temper', 'hearth'):
            self.write(tree + '/count.txt', '0')
            self.write(tree + '/scripts/old_forms.py', """from pathlib import Path
import sys
count = int(Path('count.txt').read_text())
if count:
    print(f'old: count {count}, baseline 0', file=sys.stderr)
sys.exit(int(count > 0))
""")
        self.pin()

    def command(self, *args):
        return subprocess.run(args, cwd=self.root, env=self.env, check=True,
                              capture_output=True, text=True).stdout

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def pin(self):
        self.command('git', 'add', '.')
        self.command('git', 'commit', '-qm', 'base')
        self.command('git', 'update-ref', 'refs/temper/landing', 'HEAD')

    def deliver(self, packages=('api',), expected_bundle=None,
                clippy_answer=(0, []), one_process_answer=(0, [], None), process_answer=None,
                build_answer=(0, ''), events=None):
        calls = []
        events = [] if events is None else events
        data = {'packages': [{'name': name, 'id': name, 'dependencies': [],
                             'manifest_path': str(self.root / name / 'Cargo.toml')} for name in packages],
                'workspace_members': list(packages), 'target_directory': str(self.root / 'target')}

        def run(arguments, cwd, env, output=None, stdout=False):
            calls.append((arguments, cwd))
            if arguments[:2] == ['cargo', 'build']:
                events.append(arguments)
                self.assertEqual(env['TEMPER_INTERNAL_BUNDLE'], 'none')
                self.assertNotIn('TIERS_HOLD', env)
                return build_answer
            if process_answer is not None and arguments[:4] == ['python3', '-I', '-B', 'tools/tiers/one-process']:
                return process_answer(arguments, cwd, env, output, stdout)
            result = subprocess.run(arguments, cwd=cwd, env=env, capture_output=True)
            log = PRE_PR.Tail()
            for line in (result.stdout + result.stderr).splitlines(keepends=True):
                if output:
                    output(line, log)
                else:
                    log.add(line)
                    print(line.decode(), end='')
            return result.returncode, log.text()

        def clippy_check(*args):
            events.append('clippy')
            return clippy_answer

        run_tests = PRE_PR.one_process

        def test_check(*args):
            events.append('one-process')
            return run_tests(*args) if one_process_answer is None else one_process_answer

        captured = io.BytesIO()
        log = io.TextIOWrapper(captured, encoding="utf-8")
        with contextlib.chdir(self.root), mock.patch.dict(os.environ, self.env, clear=True), \
                mock.patch.object(PRE_PR, 'metadata', return_value=data), \
                mock.patch.object(PRE_PR, 'formatting', return_value=[]) as formatting, \
                mock.patch.object(PRE_PR, 'clippy', side_effect=clippy_check) as clippy, \
                mock.patch.object(PRE_PR, 'one_process', side_effect=test_check) as one_process, \
                mock.patch.object(PRE_PR, 'run', side_effect=run), \
                contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
            result = PRE_PR.main()
        log.flush()
        if expected_bundle is not None:
            self.assertTrue(clippy.called)
            self.assertTrue(one_process.called)
            self.assertEqual(clippy.call_args.args[3]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
            self.assertEqual(one_process.call_args.args[2]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
        return result, captured.getvalue().decode(), calls, (formatting.called, clippy.called, one_process.called)

    def test_selected_test_binaries_build_between_lints_and_tests(self):
        inventory = Path(__file__).resolve().parents[1] / 'tiers/packages.toml'
        self.write('tools/tiers/packages.toml', inventory.read_text())
        self.pin()
        self.write('smith/src/lib.rs', 'pub fn changed() {}\n')
        events = []
        result, log, calls, _ = self.deliver(('api', 'smith', 'smith-fake-claude'), events=events)
        build = ['cargo', 'build', '-p', 'smith-fake-claude', '--bin', 'fake-claude']
        self.assertEqual(result, 0, log)
        self.assertEqual(events, ['clippy', build, 'one-process'])
        self.assertIn((build, self.root), calls)
        self.assertIn('pre-pr: test binaries …', log)
        self.assertIn('pre-pr: test binaries: passed,', log)

    def test_unlisted_test_package_builds_no_prerequisites(self):
        self.write('tools/tiers/packages.toml', '[test-binaries]\nsmith = '
                   '[{package="smith-fake-claude", bin="fake-claude"}]\n')
        self.pin()
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        events = []
        result, log, calls, _ = self.deliver(('api', 'smith'), events=events)
        self.assertEqual(result, 0, log)
        self.assertEqual(events, ['clippy', 'one-process'])
        self.assertFalse(any(args[:2] == ['cargo', 'build'] for args, _ in calls))
        self.assertNotIn('pre-pr: test binaries', log)

    def test_test_binary_build_failure_is_not_inherited(self):
        self.write('tools/tiers/packages.toml', '[test-binaries]\nsmith = '
                   '[{package="smith-fake-claude", bin="fake-claude"}, '
                   '{package="z-helper", bin="helper"}]\n')
        self.write('.temper/known-reds.md', '- test smith::end_to_end runs_fake\n'
                   '- lint smith-fake-claude E0001 src/main.rs\n')
        self.pin()
        self.write('smith/src/lib.rs', 'pub fn changed() {}\n')
        events = []
        result, log, calls, code = self.deliver(('api', 'smith'), events=events,
                                               build_answer=(101, 'compiler output\n'))
        build = ['cargo', 'build', '-p', 'smith-fake-claude', '--bin', 'fake-claude']
        self.assertEqual(result, 1, log)
        self.assertEqual(events, ['clippy', build])
        self.assertEqual(code, (True, True, False))
        self.assertIn('pre-pr: test binaries: failed,', log)
        self.assertIn('\nbuild\nsmith-fake-claude/fake-claude: exit 101\ncompiler output', log)
        self.assertNotIn('inherited: build', log)
        self.assertFalse(any('temper-pre-pr/base' in str(cwd) for _, cwd in calls))

    def test_invalid_test_binary_inventory_starts_no_checks(self):
        valid = '[{package="smith-fake-claude", bin="fake-claude"}]'
        for key, value in (
                ('smith', '[{package="smith-fake-claude"}]'),
                ('smith', '[{package="p", bin="../bad"}]'),
                ('smith', '[{package="p", bin="b"}, {package="p", bin="b"}]'),
                ('missing', valid),
                ('smith', '[{package="z", bin="b"}, {package="a", bin="a"}]'),
                ('smith', '[{package="a", bin="b"}, {package="z", bin="b"}]'),
                ('smith', '[{package="p", bin="b", env="P_TEST_BIN"}]'),
                ('smith', '[{package=1, bin="b"}]'),
                ('smith', '"not a list"'),
                ('smith', '["not a record"]')):
            with self.subTest(key=key, value=value):
                self.write('tools/tiers/packages.toml', f'[test-binaries]\n{key} = {value}\n')
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                result, log, calls, code = self.deliver(('api', 'smith'))
                self.assertEqual(result, 1, log)
                self.assertIn('refused', log)
                self.assertIn(key, log)
                self.assertIn('tools/tiers/packages.toml', log)
                self.assertEqual(code, (False, False, False))
                self.assertEqual(calls, [])

    def test_test_binaries_build_each_record_in_order(self):
        self.write('tools/tiers/packages.toml', '[test-binaries]\napi = '
                   '[{package="a-helper", bin="first"}, {package="z-helper", bin="last"}]\n')
        self.pin()
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        events = []
        result, log, _, _ = self.deliver(events=events)
        self.assertEqual(result, 0, log)
        self.assertEqual(events, ['clippy',
                                 ['cargo', 'build', '-p', 'a-helper', '--bin', 'first'],
                                 ['cargo', 'build', '-p', 'z-helper', '--bin', 'last'],
                                 'one-process'])

    def test_known_reds_judge_only_listed_tests_and_lints(self):
        test = PRE_PR.failure('test api::rules rule', 'test output', ('api::rules', 'rule'), 'api')
        lint = PRE_PR.failure('clippy api clippy::rule api/src/lib.rs', 'changed message',
                              ('api', 'clippy::rule', 'api/src/lib.rs', 'changed message'), 'api')
        for entry, check, error, accepted in (
                ('- test api::rules rule', 'tests', test, True),
                ('- test api::rules rule\n' * 65, 'tests', test, False),
                ('- test api::rules rule\n' + 'x' * 8193, 'tests', test, False),
                ('- test api::rules other', 'tests', test, False),
                ('- test api::other rule', 'tests', test, False),
                (None, 'tests', test, False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', lint, True),
                ('- lint other clippy::rule api/src/lib.rs', 'clippy', lint, False),
                ('- lint api clippy::other api/src/lib.rs', 'clippy', lint, False),
                ('- lint api clippy::rule api/src/other.rs', 'clippy', lint, False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('build', 'no tests ran'), False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('check one-process', 'tool error'), False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy',
                 PRE_PR.failure('check clippy', 'tool error'), False)):
            with self.subTest(entry=entry, failure=error['name']):
                path = self.root / '.temper/known-reds.md'
                path.unlink(missing_ok=True)
                if entry is not None:
                    self.write('.temper/known-reds.md', '# Known reds\n\nSome prose.\n' + entry + '  \n')
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                answer = (101 if error['name'] == 'build' or check == 'clippy' else 100, [error.copy()])
                result, log, calls, _ = self.deliver(**{
                    'clippy_answer' if check == 'clippy' else 'one_process_answer':
                    answer if check == 'clippy' else (*answer, None)})
                self.assertEqual(result, int(not accepted), log)
                summary = log.rsplit('pre-pr: passed (' if accepted else 'pre-pr: refused (', 1)[1]
                self.assertIn(error['name'], summary)
                if accepted:
                    self.assertIn('inherited: ' + error['name'] +
                                  ' (known red, .temper/known-reds.md)', summary)
                    self.assertIn(f'pre-pr: {check}: inherited,', log)
                else:
                    self.assertNotIn('inherited: ' + error['name'], summary)
                if entry is None:
                    self.assertNotIn('pre-pr: known reds:', log)
                else:
                    self.assertIn('pre-pr: known reds:', log)
                self.assertFalse(any('temper-pre-pr/base' in str(cwd) for _, cwd in calls))
                self.assertFalse((self.root / '.git/temper-pre-pr/base').exists())
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def goal_file(self, name, content):
        path = self.root / self.command('git', 'rev-parse', '--git-path', 'temper/' + name).strip()
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_goal_reds_judge_tests_and_coded_lints_beside_the_list(self):
        test = PRE_PR.failure('test api::rules rule', 'test output', ('api::rules', 'rule'), 'api')
        lint = PRE_PR.failure('clippy api clippy::rule api/src/lib.rs', 'message',
                              ('api', 'clippy::rule', 'api/src/lib.rs', 'message'), 'api')
        uncoded = PRE_PR.failure('clippy api  api/src/lib.rs', 'compiler error',
                                 ('api', '', 'api/src/lib.rs', 'compiler error'), 'api')
        for entry, check, error, listed, accepted in (
                ('- test api::rules rule', 'tests', test, '', True),
                ('- test api::rules rule', 'tests', test, '- test api::rules rule\n', True),
                ('- test api::rules rule', 'tests', test, '- test api::rules list_only\n' * 65, False),
                ('- test api::rules rule', 'tests', test, '- test api::rules list_only\n' + 'x' * 8193, False),
                ('- test api::rules other', 'tests', test, '', False),
                ('- test api::other rule', 'tests', test, '', False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', lint, '', True),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', lint,
                 '- lint api clippy::rule api/src/lib.rs\n', True),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', uncoded, '', False),
                ('- lint api clippy::other api/src/lib.rs', 'clippy', lint, '', False),
                ('- lint api clippy::rule api/src/other.rs', 'clippy', lint, '', False),
                ('- lint other clippy::rule api/src/lib.rs', 'clippy', lint, '', False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('build', 'no tests ran'), '', False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('check one-process', 'tool error'), '', False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy',
                 PRE_PR.failure('check clippy', 'tool error'), '', False)):
            with self.subTest(entry=entry, failure=error['name'], listed=listed[:80]):
                self.goal_file('reds', ('temper-reds 1\n' + entry + '\n').encode())
                self.write('.temper/known-reds.md', listed)
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                over_cap = len(listed.encode()) > 8192 or listed.count('\n') > 64
                errors = [error.copy()]
                if over_cap:
                    errors.append(PRE_PR.failure('test api::rules list_only', 'list-only failure',
                                                 ('api::rules', 'list_only'), 'api'))
                answer = (101 if check == 'clippy' else 100, errors)
                result, log, _, _ = self.deliver(**{
                    'clippy_answer' if check == 'clippy' else 'one_process_answer':
                    answer if check == 'clippy' else (*answer, None)})
                self.assertEqual(result, int(not accepted), log)
                self.assertIn('pre-pr: reds: 1 entries (temper/reds)', log)
                inherits = error in (test, lint) and entry in (
                    '- test api::rules rule', '- lint api clippy::rule api/src/lib.rs')
                if inherits:
                    self.assertIn('inherited: ' + error['name'] + ' (red, temper/reds)', log)
                    self.assertNotIn(' (known red, .temper/known-reds.md)', log)
                else:
                    self.assertNotIn('inherited: ' + error['name'], log)
                if over_cap:
                    self.assertIn('\nknown-reds\n', log)
                    self.assertIn('\ntest api::rules list_only\n', log)
                    self.assertNotIn('inherited: test api::rules list_only', log)
                self.assertLess(log.index('pre-pr: reds:'), log.index('pre-pr: known reds:'))
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def test_goal_reds_are_strict_whole_bounded_files(self):
        entry = b'- test api::rules rule\n'
        header = b'temper-reds 1\n'
        # Both sides of the byte bound, with a valid entry whose name fills it.
        prefix = header + b'- test api::rules '
        at_cap = prefix + b'x' * (2097152 - len(prefix) - 1) + b'\n'
        self.assertEqual(len(at_cap), 2097152)
        for content, expected in (
                (None, None),
                (b'temper-reds 1', '0 entries (temper/reds)'),
                (header, '0 entries (temper/reds)'),
                (header + entry.rstrip(b'\n'), '1 entries (temper/reds)'),
                (header + entry, '1 entries (temper/reds)'),
                (b'', 'none (temper/reds malformed at line 1)'),
                (b'temper-reds 2\n' + entry, 'none (temper/reds malformed at line 1)'),
                (b'temper-reds 1 \n' + entry, 'none (temper/reds malformed at line 1)'),
                (header + entry + b'prose\n', 'none (temper/reds malformed at line 3)'),
                (header + entry.rstrip(b'\n') + b' \n', 'none (temper/reds malformed at line 2)'),
                (header + entry.rstrip(b'\n') + b'\t\n', 'none (temper/reds malformed at line 2)'),
                (header + b'\n', 'none (temper/reds malformed at line 2)'),
                (header + b' - test api::rules rule\n', 'none (temper/reds malformed at line 2)'),
                (header + b'- test api::rules \xff\n', 'none (temper/reds malformed at line 2)'),
                (at_cap, '1 entries (temper/reds)'),
                (at_cap + b'x', 'none (temper/reds oversized: 2097153 bytes)')):
            with self.subTest(expected=expected, size=None if content is None else len(content)):
                path = self.goal_file('reds', content or b'')
                if content is None:
                    path.unlink()
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                error = PRE_PR.failure('test api::rules rule', 'test output', ('api::rules', 'rule'), 'api')
                result, log, _, _ = self.deliver(one_process_answer=(100, [error], None))
                accepted = content in (header + entry, header + entry.rstrip(b'\n'))
                self.assertEqual(result, int(not accepted), log)
                if expected is None:
                    self.assertNotIn('pre-pr: reds:', log)
                else:
                    self.assertIn('pre-pr: reds: ' + expected, log)
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def test_goal_reds_wait_for_the_marker_only(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        for marker, expected, waits in (
                (b'temper-reds-wait 1 100\n', 'none (temper/reds not written within 100 ms)', True),
                (b'temper-reds-wait 1 0\n', 'ignored temper/reds-wait (malformed)', False),
                (b'temper-reds-wait 1 600001\n', 'ignored temper/reds-wait (malformed)', False),
                (b'temper-reds-wait 2 100\n', 'ignored temper/reds-wait (malformed)', False),
                (b'temper-reds-wait 1 100 \n', 'ignored temper/reds-wait (malformed)', False),
                (b'temper-reds-wait 1 100\n\n', 'ignored temper/reds-wait (malformed)', False),
                (b'', 'ignored temper/reds-wait (malformed)', False)):
            with self.subTest(marker=marker):
                self.goal_file('reds-wait', marker)
                with mock.patch.object(PRE_PR, 'time', wraps=time) as clock:
                    start = time.monotonic()
                    result, log, _, _ = self.deliver()
                    elapsed = time.monotonic() - start
                self.assertEqual(result, 0, log)
                self.assertEqual(clock.sleep.called, waits)
                if waits:
                    self.assertGreaterEqual(elapsed, 0.1)
                self.assertIn('pre-pr: reds: ' + expected, log)
                (self.root / '.git/pre-pr-passed').unlink()

        self.goal_file('reds-wait', b'temper-reds-wait 1 200\n')
        path = self.goal_file('reds', b'')
        path.unlink()
        sleep = time.sleep
        timers = []

        def worker(seconds):
            if not timers:
                timer = threading.Timer(0.01, lambda: path.write_bytes(b'temper-reds 1\n- test api::rules rule\n'))
                timers.append(timer)
                timer.start()
            sleep(seconds)

        error = PRE_PR.failure('test api::rules rule', 'output', ('api::rules', 'rule'), 'api')
        with mock.patch.object(PRE_PR, 'time', wraps=time) as clock:
            clock.sleep.side_effect = worker
            result, log, _, _ = self.deliver(one_process_answer=(100, [error], None))
        for timer in timers:
            timer.join()
        self.assertTrue(timers, 'file must be written during the wait')
        self.assertEqual(result, 0, log)
        self.assertIn('pre-pr: reds: 1 entries (temper/reds)', log)
        self.assertNotIn('not written within', log)

    def test_goal_reds_bytes_key_the_cache(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        content = b'temper-reds 1\n- test api::rules rule\n'
        path = self.goal_file('reds', content)
        result, log, _, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertTrue(all(code))
        cache = self.root / '.git/pre-pr-passed'
        tree, base, digest = cache.read_text().split()
        self.assertEqual(digest, hashlib.sha256(content).hexdigest())
        hit = f'pre-pr: passed before (tree {tree}, base {base}, reds {digest})'
        result, log, _, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertFalse(any(code))
        self.assertIn(hit, log)
        self.assertLess(log.index('pre-pr: reds:'), log.index(hit))
        for changed in (b'temper-reds 1\n', b'bad header\n', b'bad header changed\n', None):
            with self.subTest(changed=changed):
                if changed is None:
                    path.unlink()
                else:
                    path.write_bytes(changed)
                result, log, _, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertTrue(all(code), log)
                self.assertNotIn('passed before', log)
                self.assertEqual(cache.read_text().split()[2],
                                 '-' if changed is None else hashlib.sha256(changed).hexdigest())
        cache.write_text(f'{tree} {base}\n')
        result, log, _, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertTrue(all(code), 'two-field records must not hit')

    def test_one_process_reads_the_run_directory(self):
        self.write('client/Cargo.toml', '[package]\nname = "client"\n')
        self.write('client/src/lib.rs', 'pub fn value() {}\n')
        self.pin()
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.write('client/src/lib.rs', 'pub fn changed() {}\n')
        junit = ('<testsuites><testsuite name="api::rules"><testcase name="rule">'
                 '<failure>fault</failure><system-out>' + 'old output\n' * 200 +
                 'junit tail</system-out><system-err>stderr tail</system-err>'
                 '</testcase></testsuite></testsuites>')
        test = dict(binary_id='api::rules', test_name='rule', outcome='failed')
        warning = ('pre-pr: tests starved: 1 starved or cancelled, 2 unobserved, '
                   'a warning (skein/docs/foundation/testing-strategy.md, 8.1)')
        build = 'one-process: nextest listing exited with 101'
        rows = [
            dict(case='argv and pass', code=0),
            dict(case='failure', code=100, tests=[test], error='test api::rules rule', tail='stderr tail'),
            dict(case='error node', code=100, tests=[test], error='test api::rules rule',
                 files={'junit.xml': junit.replace('failure>', 'error>')}),
            dict(case='alone passed', code=100, tests=[dict(test, alone='passed', before=['first', 'second'])],
                 error='test api::rules rule', tail='may depend on an earlier test: first, second'),
            dict(case='alone failed', code=100, tests=[dict(test, alone='failed', before=['first'])],
                 error='test api::rules rule', tail='stderr tail'),
            dict(case='known red', code=100, tests=[test], known='- test api::rules rule', inherited=True),
            dict(case='different binary', code=100, tests=[test], known='- test api::other rule',
                 error='test api::rules rule'),
            dict(case='empty 100', code=100, error='check one-process', tail='exit 100 without JUnit failures',
                 files={'junit.xml': '<testsuites/>'}),
            dict(case='build', code=2, output='compiler output\n' + build + '\n', error='build'),
            dict(case='build line not last', code=2, output=build + '\nmore output\n',
                 error='check one-process', tail='exit 2'),
            dict(case='refusal', code=2, error='check one-process', tail='exit 2'),
            dict(case='other exit', code=102, error='check one-process', tail='exit 102'),
            dict(case='starved run', code=0, starved=True, warning=warning,
                 tests=[dict(test, outcome='cancelled')], unobserved=[{}, {}]),
            dict(case='run flag alone', code=0, starved=True,
                 warning=warning.replace('1 starved', '0 starved').replace('2 unobserved', '0 unobserved')),
            dict(case='starved test', code=0, tests=[dict(test, outcome='starved')],
                 unobserved=[{}, {}], warning=warning),
            dict(case='cancelled test', code=0, tests=[dict(test, outcome='cancelled')],
                 unobserved=[{}, {}], warning=warning),
            dict(case='cancelled 100', code=100, tests=[dict(test, outcome='cancelled')],
                 error='check one-process', warning=warning.replace('2 unobserved', '0 unobserved')),
            dict(case='starved with failure', code=100, tests=[test, dict(test, test_name='stop', outcome='cancelled')],
                 error='test api::rules rule', warning=warning.replace('2 unobserved', '0 unobserved')),
            dict(case='cancelled beside failure', code=100,
                 tests=[dict(test, outcome='cancelled'), dict(test, test_name='other')],
                 files={'junit.xml': junit.replace('</testsuite>',
                        '<testcase name="other"><error>other fault</error></testcase></testsuite>')},
                 error='test api::rules other', warning=warning.replace('2 unobserved', '0 unobserved')),
        ]
        for name in ('run.json', 'results.json', 'junit.xml'):
            for contents in (None, 'malformed', 'directory'):
                rows.append(dict(case=f'{name}: {contents}', code=100, error='check one-process',
                                 tail=name + ':', files={name: contents}))
        rows += [dict(case='malformed run shape', code=0, error='check one-process',
                      files={'run.json': '{"starved": "false"}'}),
                 dict(case='malformed results shape', code=0, error='check one-process',
                      files={'results.json': '{"tests": null, "unobserved": []}'})]
        for row in rows:
            with self.subTest(case=row['case']):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

                def fake(arguments, cwd, env, output, stdout):
                    self.assertEqual(cwd, self.root)
                    self.assertTrue(stdout)
                    self.assertIsNone(output)
                    directory = Path(arguments[arguments.index('--out') + 1])
                    self.assertEqual(arguments, ['python3', '-I', '-B', 'tools/tiers/one-process',
                                                '--out', str(directory), 'default', '-p', 'api',
                                                '-p', 'client', '--no-tests=warn'])
                    self.assertEqual(directory.name, 'tests')
                    self.assertFalse(directory.exists())
                    self.assertEqual(directory.parent.stat().st_mode & 0o777, 0o700)
                    directory.mkdir()
                    files = {'junit.xml': junit, 'run.json': json.dumps({'starved': row.get('starved', False)}),
                             'results.json': json.dumps({'tests': row.get('tests', []),
                                                         'unobserved': row.get('unobserved', [])})}
                    files.update(row.get('files', {}))
                    for name, contents in files.items():
                        if contents == 'directory':
                            (directory / name).mkdir()
                        elif contents is not None:
                            (directory / name).write_text(contents)
                    text = row.get('output', 'streamed output\n')
                    print(text, end='')
                    return row['code'], PRE_PR.tail(text)

                self.write('.temper/known-reds.md', row.get('known', '- test api::rules rule' if
                           row.get('error') in ('build', 'check one-process') else '') + '\n')
                result, log, calls, _ = self.deliver(('api', 'client'), one_process_answer=None, process_answer=fake)
                self.assertEqual(result, int('error' in row), log)
                self.assertEqual((self.root / '.git/pre-pr-passed').exists(), result == 0)
                summary = log.rsplit('pre-pr: refused (' if result else 'pre-pr: passed (', 1)[1]
                if 'error' in row:
                    self.assertIn('\n' + row['error'] + '\n', summary)
                    self.assertIn(row.get('tail', ''), summary)
                    self.assertIn('pre-pr: tests: failed,', log)
                else:
                    status = 'inherited' if row.get('inherited') else 'passed'
                    self.assertIn(f'pre-pr: tests: {status},', log)
                    if row.get('inherited'):
                        self.assertIn('inherited: test api::rules rule (known red, .temper/known-reds.md)', summary)
                    else:
                        self.assertNotIn('\ntest api::', summary)
                if row.get('tail', '').startswith('may depend'):
                    self.assertTrue(summary.split('\ninherited:', 1)[0].rstrip().endswith(row['tail']), summary)
                    self.assertIn('bytes cut]', summary)
                elif row['case'] == 'alone failed':
                    self.assertNotIn('may depend', log)
                if 'warning' in row:
                    self.assertEqual(log.count(row['warning']), 2, log)
                    self.assertTrue(summary.split('tests: api client\n', 1)[1].startswith(row['warning']))
                else:
                    self.assertNotIn('pre-pr: tests starved:', log)
                if row['case'] in ('cancelled 100', 'cancelled beside failure'):
                    self.assertNotIn('\ntest api::rules rule\n', summary)
                self.assertFalse(any('cargo' in args and 'nextest' in args for args, _ in calls))
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def test_known_reds_are_bounded_snapshot_entries(self):
        for with_base in (True, False):
            if not with_base:
                self.command('git', 'update-ref', '-d', 'refs/temper/landing')
            for content, count, accepted in (
                    ('- test api::rules rule\n' * 64, 64, True),
                    ('- test api::rules rule\n' * 65, 65, False),
                    ('x' * 8192, 0, True),
                    ('x' * 8193, 0, False),
                    ('é' * 4096, 0, True),
                    ('é' * 4096 + 'x', 0, False),
                    ('- test api::rules rule\n' + 'x' * 8193, 1, False),
                    ('# Prose\n - test api::rules rule\n- test api::rules\n'
                     '- lint api clippy::rule\n', 0, True)):
                with self.subTest(entries=count, size=len(content.encode()), base=with_base):
                    # Only this unstaged Markdown changes: reading is independent of code selection.
                    self.write('.temper/known-reds.md', content)
                    result, log, _, code = self.deliver()
                    self.assertEqual(result, int(not accepted), log)
                    self.assertEqual(code, (not with_base, False, False))
                    self.assertIn(f'pre-pr: known reds: {count} entries (.temper/known-reds.md)', log)
                    if not accepted:
                        self.assertIn('known-reds\n.temper/known-reds.md: '
                                      f'{count} entries, {len(content.encode()):,} bytes; '
                                      'the cap is 64 entries and 8,192 bytes', log)
                    (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def test_saved_edits_are_checked_with_new_edits(self):
        title = 'Unchecked: work in progress, saved before a question'
        for name in ('a', 'b'):
            self.write(name + '/Cargo.toml', '[package]\nname = "' + name + '"\n')
            self.write(name + '/src/lib.rs', 'pub fn initial() {}\n')
        for count, subject in ((1, title), (2, title), (1, title + ' extra')):
            with self.subTest(saves=count, subject=subject):
                self.command('git', 'add', '.')
                self.command('git', 'commit', '--allow-empty', '-qm', 'checked head')
                head = self.command('git', 'rev-parse', 'HEAD').strip()
                for number in range(count):
                    self.write('a/src/lib.rs', f'pub fn saved_{count}_{number}() {{}}\n')
                    self.command('git', 'add', 'a')
                    self.command('git', 'commit', '--allow-empty', '-qm', subject)
                saved = self.command('git', 'rev-parse', 'HEAD').strip()
                self.command('git', 'update-ref', 'refs/temper/landing', saved)
                self.write('b/src/lib.rs', f'pub fn delivered_{count}() {{}}\n')
                expected = head if subject == title else saved
                result, log, _, _ = self.deliver(('a', 'b'))
                self.assertEqual(result, 0, log)
                self.assertIn(f'base {expected}, merge base', log)
                self.assertIn('tests: a b\n' if subject == title else 'tests: b\n', log)
                self.assertEqual(self.command('git', 'rev-parse', 'refs/temper/landing').strip(), saved)

    def test_docs_only(self):
        # Markdown inside a package still selects docs unless registered as an input.
        for path in ('guide.md', 'api/README.md', 'api/docs/diagram.svg'):
            with self.subTest(path=path):
                self.write(path, '# Changed\n')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertEqual(code, (False, False, False))
                self.assertFalse(any('scripts/old_forms.py' in args for args, _ in calls))
                self.assertTrue(any('--base' in args and '--head' in args for args, _ in calls))
                self.assertIn('docs only: citations', log)

    def test_code_only(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        result, log, calls, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertEqual(code, (True, True, True))
        self.assertEqual(sum('scripts/old_forms.py' in args for args, _ in calls), 5)
        self.assertFalse(any('tools/citations/citations.py' in args for args, _ in calls))
        self.assertIn('code only:', log)

    def test_bundle_defaults_to_none_and_keeps_the_callers_value(self):
        for value in (None, 'embed'):
            with self.subTest(value=value):
                self.env.pop('TEMPER_INTERNAL_BUNDLE', None)
                if value is not None:
                    self.env['TEMPER_INTERNAL_BUNDLE'] = value
                expected = 'none' if value is None else value
                self.write('api/src/lib.rs', f'pub fn {expected}() {{}}\n')
                result, log, _, _ = self.deliver(expected_bundle=expected)
                self.assertEqual(result, 0, log)

    def test_both(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.write('guide.md', '# Changed\n')
        result, log, calls, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertEqual(code, (True, True, True))
        self.assertTrue(any('tools/citations/citations.py' in args for args, _ in calls))
        self.assertIn('code and docs: format', log)
        self.assertIn('code and docs: citations', log)

    def test_docs_only_on_red_baseline(self):
        self.write('skein/count.txt', '2')
        self.pin()
        self.write('guide.md', '# Changed\n')
        result, log, calls, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertEqual(code, (False, False, False))
        self.assertFalse(any('scripts/old_forms.py' in args for args, _ in calls))

    def test_new_broken_citation(self):
        self.write('skein/docs/guide.md', '# Guide\nSee `missing.md`, section 1.\n')
        result, log, _, code = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertIn('missing.md', log)
        self.assertIn('1 new broken', log)
        self.assertEqual(code, (False, False, False))

    def test_inherited_broken_citation(self):
        self.write('skein/docs/guide.md', '# Guide\nSee `missing.md`, section 1.\n')
        self.pin()
        self.write('skein/docs/guide.md', '# Guide\nExtra paragraph.\nSee `missing.md`, section 1.\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertIn('0 new broken', log)
        self.assertIn('inherited: citations', log)

    def test_inherited_baseline_and_regression(self):
        self.write('skein/count.txt', '2')
        self.pin()
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertIn('inherited: old-forms skein old', log)
        self.write('skein/count.txt', '3')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertIn('old-forms skein old', log)

    def test_test_inputs_select_code(self):
        for path in ('api/fixtures/value.golden', 'api/docs/input.txt', 'shared/fixtures/value.bin'):
            with self.subTest(path=path):
                self.write(path, 'changed\n')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertTrue(code[0])
                self.assertTrue(code[2])
                (self.root / path).unlink()

    def test_code_configuration_and_ratchet_inputs(self):
        for path in ('Cargo.lock', 'rustfmt.toml', '.config/one_process.toml',
                     'skein/scripts/old-forms.txt', 'tools/old_forms/old_forms.py',
                     'skein/scripts/check.sh'):
            with self.subTest(path=path):
                self.write(path, '# changed\n')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertEqual(code, (True, False, False))
                self.assertFalse(any('tools/citations/citations.py' in args for args, _ in calls))
                (self.root / path).unlink()

    def test_rust_and_cargo_under_docs_select_both(self):
        for path in ('api/docs/example.rs', 'api/docs/Cargo.toml'):
            with self.subTest(path=path):
                self.write(path, '# changed\n')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertEqual(code, (True, True, True))
                self.assertIn('code and docs: citations', log)
                (self.root / path).unlink()

    def test_deleting_a_cited_document_fails(self):
        self.write('skein/docs/target.md', '# Target\n')
        self.write('skein/docs/reader.md', '# Reader\nSee `target.md`.\n')
        self.pin()
        (self.root / 'skein/docs/target.md').unlink()
        result, log, _, code = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertIn('reader.md', log)
        self.assertEqual(code, (False, False, False))

    def test_new_citation_is_not_hidden_by_inherited_findings(self):
        self.write('skein/docs/guide.md', '# Guide\nSee `missing.md`.\n')
        self.pin()
        self.write('skein/docs/guide.md', '# Guide\nSee `missing.md` and `new-missing.md`.\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertIn('1 new broken', log)
        self.assertIn('inherited: citations', log)

    def test_tightening_a_red_baseline_is_new(self):
        self.write('skein/scripts/old_forms.py', (self.root / 'skein/scripts/old_forms.py').read_text()
                   .replace('baseline 0', 'baseline 1'))
        self.write('skein/count.txt', '2')
        self.pin()
        self.write('skein/scripts/old_forms.py', (self.root / 'skein/scripts/old_forms.py').read_text()
                   .replace('baseline 1', 'baseline 0'))
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertNotIn('inherited: old-forms', log)

    def test_neither(self):
        self.write('notes.txt', 'Private notes\n')
        result, log, calls, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertFalse(calls)
        self.assertEqual(code, (False, False, False))
        self.assertIn('no code or docs inputs changed', log)

    def test_size_limit(self):
        self.write('skein/docs/design/large.md', '# Large\n' + 'x' * 65536)
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        self.assertIn('size limit', log)


if __name__ == '__main__':
    unittest.main()
