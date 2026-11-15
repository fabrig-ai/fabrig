"""Delivery selection, goal reds and inherited counts (tools/pre-pr/design.md, 3, 5, 9)."""
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
        for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR', 'TIERS_HOLD'):
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
                build_answer=(0, ''), events=None, dependencies=None, formatting_answer=(), hold=None):
        calls = []
        events = [] if events is None else events
        dependencies = {} if dependencies is None else dependencies
        data = {'packages': [{'name': name, 'id': name,
                              'dependencies': [dict(dependency, path=str(self.root / dependency['name'].lower()))
                                               for dependency in dependencies.get(name, [])],
                              'manifest_path': str(self.root / name.lower() / 'Cargo.toml')} for name in packages],
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

        # Judgement annotates errors; each delivery gets fresh program answers.
        def clippy_check(*args):
            events.append('clippy')
            return clippy_answer[0], [error.copy() for error in clippy_answer[1]]

        run_tests = PRE_PR.one_process

        def test_check(*args):
            events.append('one-process')
            if one_process_answer is None:
                return run_tests(*args)
            code, errors, starved = one_process_answer
            return code, [error.copy() for error in errors], starved

        captured = io.BytesIO()
        log = io.TextIOWrapper(captured, encoding="utf-8")
        holder = (mock.patch.object(PRE_PR.runpy, 'run_path', return_value={'hold': hold})
                  if hold is not None else contextlib.nullcontext())
        with contextlib.chdir(self.root), mock.patch.dict(os.environ, self.env, clear=True), holder, \
                mock.patch.object(PRE_PR, 'metadata', return_value=data), \
                mock.patch.object(PRE_PR, 'formatting', return_value=list(formatting_answer)) as formatting, \
                mock.patch.object(PRE_PR, 'clippy', side_effect=clippy_check) as clippy, \
                mock.patch.object(PRE_PR, 'one_process', side_effect=test_check) as one_process, \
                mock.patch.object(PRE_PR, 'run', side_effect=run), \
                contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
            result = PRE_PR.main()
        self.clippy_mock = clippy
        self.one_process_mock = one_process
        log.flush()
        if expected_bundle is not None:
            self.assertTrue(clippy.called)
            self.assertTrue(one_process.called)
            self.assertEqual(clippy.call_args.args[3]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
            self.assertEqual(one_process.call_args.args[2]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
        return result, captured.getvalue().decode(), calls, (formatting.called, clippy.called, one_process.called)

    def test_base_table(self):
        for row in ('no pin', 'unrelated', 'behind'):
            with self.subTest(row=row):
                self.setUp()
                ancestor = self.command('git', 'rev-parse', 'HEAD').strip()
                if row == 'no pin':
                    self.command('git', 'update-ref', '-d', 'refs/temper/landing')
                elif row == 'unrelated':
                    other = self.command('git', 'commit-tree', 'HEAD^{tree}', '-m', 'pin').strip()
                    self.command('git', 'update-ref', 'refs/temper/landing', other)
                else:
                    self.command('git', 'commit', '--allow-empty', '-qm', 'head')
                self.write('temper/docs/addition.md', '# Delivery\n')
                with mock.patch.object(PRE_PR, 'git', wraps=PRE_PR.git) as git:
                    result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertFalse(self.clippy_mock.called)
                self.assertFalse(self.one_process_mock.called)
                counts = [cwd.name for args, cwd in calls if 'scripts/old_forms.py' in args]
                if row == 'behind':
                    self.assertIn(mock.call(['merge-base', 'refs/temper/landing', 'HEAD'], self.root, mock.ANY),
                                  git.call_args_list)
                    self.assertIn(f'base {ancestor}, merge base of refs/temper/landing and HEAD', log)
                    self.assertIn('docs only: citations', log)
                    self.assertEqual(code, (False, False, False))
                    self.assertEqual(counts, [])
                    self.assertTrue(any('--base' in args and '--head' in args for args, _ in calls))
                else:
                    reason = ('no refs/temper/landing' if row == 'no pin' else
                              'no merge base of refs/temper/landing and HEAD')
                    self.assertIn(f'pre-pr: no base ({reason}): the format check and the counted baselines alone', log)
                    self.assertEqual(code, (True, False, False))
                    self.assertEqual(counts, ['skein', 'smith', 'jig', 'temper', 'hearth'])
                    self.assertFalse(any('tools/citations/citations.py' in args for args, _ in calls))

    def test_each_trees_count_refuses_without_a_base(self):
        self.command('git', 'update-ref', '-d', 'refs/temper/landing')
        for tree in ('skein', 'smith', 'jig', 'temper', 'hearth'):
            with self.subTest(tree=tree):
                self.write(tree + '/count.txt', '1')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 1, log)
                self.assertIn(f'old-forms {tree}', log)
                self.assertIn(f'pre-pr: old-forms {tree}: failed,', log)
                self.assertFalse((self.root / '.git/pre-pr-passed').exists())
                self.assertEqual(code, (True, False, False))
                self.assertEqual(sum('scripts/old_forms.py' in args for args, _ in calls), 5)
                self.write(tree + '/count.txt', '0')

    def test_selection_table(self):
        rows = (
            ('dev', 'a/src/lib.rs', 'B\n', '', ['A'], ['A', 'B']),
            ('untracked', 'a/src/new.rs', 'B\n', '', ['A'], ['A']),
            ('deleted', 'a/src/lib.rs', 'B\n', '', ['A'], ['A']),
            ('root manifest', 'Cargo.toml', 'B\n', '', [], []),
            ('docs', 'temper/docs/selection.md', 'B\n', '', [], []),
            ('reader', 'a/src/lib.rs', 'B a/\n', '', ['A'], ['A', 'B']),
            ('reader only', 'temper/docs/selection.md', 'B temper/docs/\n',
             'temper/docs/selection.md\n', [], ['B']),
            ('prefix-less', 'temper/docs/selection.md', 'B\n', '', [], []),
            ('component boundary', 'a-sibling/new.rs', 'B\n', '', [], []),
            ('unknown reader', 'shared/input.txt', 'missing shared/\n', '', [], []),
        )
        for row, path, always, inputs, lints, tests in rows:
            with self.subTest(row=row):
                self.setUp()
                for name in ('a', 'b', 'p2'):
                    self.write(name + '/Cargo.toml', '[package]\n')
                    self.write(name + '/src/lib.rs', 'fn source() {}\n')
                self.write('.config/gate/always.txt', always)
                self.write('.config/gate/read-by-tests.txt', inputs)
                self.pin()
                if row == 'deleted':
                    (self.root / path).unlink()
                else:
                    self.write(path, '# changed\n')
                dependencies = {'B': [{'name': 'A', 'kind': 'dev'}],
                                'P2': [{'name': 'B', 'kind': None}]} if row == 'dev' else {}
                result, log, _, (_, linted, tested) = self.deliver(('A', 'B', 'P2'), dependencies=dependencies)
                self.assertEqual(result, 0, log)
                self.assertEqual(self.clippy_mock.call_args.args[0] if linted else [], lints)
                self.assertEqual(self.one_process_mock.call_args.args[0] if tested else [], tests)
                if not lints and not tests and 'docs only:' not in log:
                    self.assertIn('pre-pr: no package changed', log)
                if row == 'unknown reader':
                    self.assertIn('pre-pr: path reader missing: not a workspace member, skipped', log)

    def test_moved_pin_misses_the_cache(self):
        for row in ('first pin', 'moved pin'):
            with self.subTest(row=row):
                self.setUp()
                if row == 'first pin':
                    self.command('git', 'update-ref', '-d', 'refs/temper/landing')
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 0, log)
                self.command('git', 'commit', '--allow-empty', '-qm', row)
                self.command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
                result, log, calls, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertTrue(code[0])
                self.assertEqual(sum('scripts/old_forms.py' in args for args, _ in calls), 5)
                self.assertNotIn('passed before', log)

    def test_unset_or_empty_hold_starts_none(self):
        real_hold = PRE_PR.runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))['hold']
        for value in (None, ''):
            with self.subTest(value=value):
                self.setUp()
                self.env.pop('TIERS_HOLD', None)
                if value is not None:
                    self.env['TIERS_HOLD'] = value
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                process = mock.Mock(spec=subprocess)
                process.Popen.side_effect = AssertionError('unexpected hold start')
                with mock.patch.dict(real_hold.__wrapped__.__globals__, subprocess=process):
                    result, log, _, code = self.deliver(hold=real_hold)
                    self.assertEqual(result, 0, log)
                    self.assertTrue(all(code))
                    process.Popen.assert_not_called()
                    result, log, _, code = self.deliver(hold=real_hold)
                    self.assertEqual(result, 0, log)
                    self.assertIn('passed before', log)
                    self.assertFalse(any(code))
                    process.Popen.assert_not_called()

    def test_early_hold_exit_keeps_a_refusal(self):
        self.env['TIERS_HOLD'] = "python3 -c __import__('sys').exit(3)"
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        fault = PRE_PR.failure('format', 'format fault')
        real_hold = PRE_PR.runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))['hold']
        starts = []

        @contextlib.contextmanager
        def hold(prefix, *args, **kwargs):
            starts.append(prefix)
            with real_hold(prefix, *args, **kwargs):
                # Cargo's metadata and the formatter start after the failed hold.
                self.assertFalse(PRE_PR.metadata.called)
                yield
                self.assertTrue(PRE_PR.metadata.called)

        result, log, _, code = self.deliver(formatting_answer=[fault], hold=hold)
        self.assertEqual(starts, [['python3', '-c', "__import__('sys').exit(3)"]])
        self.assertEqual(result, 1, log)
        self.assertEqual(log.count('pre-pr: hold:'), 1, log)
        self.assertIn('; going on unheld', log)
        self.assertLess(log.index('pre-pr: hold:'), log.index('pre-pr: format …'))
        self.assertEqual(code, (True, True, True))
        self.assertIn('format fault', log)
        self.assertFalse((self.root / '.git/pre-pr-passed').exists())

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
        self.goal_file('reds', b'temper-reds 1\n- test smith::end_to_end runs_fake\n'
                       b'- lint smith-fake-claude E0001 src/main.rs\n')
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

    def test_tree_list_counts_nothing(self):
        test = PRE_PR.failure('test api::rules listed', 'test output', ('api::rules', 'listed'), 'api')
        lint = PRE_PR.failure('clippy api clippy::rule api/src/lib.rs', 'lint output',
                              ('api', 'clippy::rule', 'api/src/lib.rs', 'message'), 'api')
        for entry, check, error in (
                ('- test api::rules listed', 'tests', test),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', lint)):
            for extra in ('', '\n- test api::rules other' * 65, '\n' + 'x' * 8193):
                for goal in (False, True):
                    with self.subTest(entry=entry, size=len(extra), goal=goal):
                        self.setUp()
                        self.write('.temper/known-reds.md', entry + extra + '\n')
                        self.pin()
                        path = self.goal_file('reds', ('temper-reds 1\n' + entry + '\n').encode())
                        if not goal:
                            path.unlink()
                        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                        answer = (101 if check == 'clippy' else 100, [error.copy()])
                        result, log, calls, _ = self.deliver(**{
                            'clippy_answer' if check == 'clippy' else 'one_process_answer':
                            answer if check == 'clippy' else (*answer, None)})
                        self.assertEqual(result, int(not goal), log)
                        self.assertEqual((self.root / '.git/pre-pr-passed').exists(), goal)
                        self.assertTrue(self.one_process_mock.called)
                        self.assertFalse(any('temper-pre-pr/base' in str(cwd) for _, cwd in calls))
                        self.assertFalse((self.root / '.git/temper-pre-pr/base').exists())
                        self.assertNotIn('inherited: build', log)
                        summary = log.rsplit('pre-pr: passed (' if goal else 'pre-pr: refused (', 1)[1]
                        if goal:
                            self.assertIn('inherited: ' + error['name'] + ' (red, temper/reds)', summary)
                        else:
                            self.assertIn('\n' + error['name'] + '\n' + error['log'], summary)
                            self.assertNotIn('inherited:', summary)
                        self.assertNotIn('pre-pr: known reds', log)
                        self.assertNotIn('\nknown-reds\n', summary)
                        (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def goal_file(self, name, content):
        path = self.root / self.command('git', 'rev-parse', '--git-path', 'temper/' + name).strip()
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_goal_reds_judge_tests_and_coded_lints(self):
        test = PRE_PR.failure('test api::rules rule', 'test output', ('api::rules', 'rule'), 'api')
        lint = PRE_PR.failure('clippy api clippy::rule api/src/lib.rs', 'message',
                              ('api', 'clippy::rule', 'api/src/lib.rs', 'message'), 'api')
        uncoded = PRE_PR.failure('clippy api  api/src/lib.rs', 'compiler error',
                                 ('api', '', 'api/src/lib.rs', 'compiler error'), 'api')
        for entry, check, error, accepted in (
                ('- test api::rules rule', 'tests', test, True),
                ('- test api::rules other', 'tests', test, False),
                ('- test api::other rule', 'tests', test, False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', lint, True),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy', uncoded, False),
                ('- lint api clippy::other api/src/lib.rs', 'clippy', lint, False),
                ('- lint api clippy::rule api/src/other.rs', 'clippy', lint, False),
                ('- lint other clippy::rule api/src/lib.rs', 'clippy', lint, False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('build', 'no tests ran'), False),
                ('- test api::rules rule', 'tests', PRE_PR.failure('check one-process', 'tool error'), False),
                ('- lint api clippy::rule api/src/lib.rs', 'clippy',
                 PRE_PR.failure('check clippy', 'tool error'), False)):
            with self.subTest(entry=entry, failure=error['name']):
                self.goal_file('reds', ('temper-reds 1\n' + entry + '\n').encode())
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                answer = (101 if check == 'clippy' else 100, [error.copy()])
                result, log, calls, _ = self.deliver(**{
                    'clippy_answer' if check == 'clippy' else 'one_process_answer':
                    answer if check == 'clippy' else (*answer, None)})
                self.assertEqual(result, int(not accepted), log)
                self.assertEqual((self.root / '.git/pre-pr-passed').exists(), accepted)
                self.assertTrue(self.one_process_mock.called, 'tests run even after clippy fails')
                self.assertNotIn('inherited: build', log)
                self.assertNotIn('pre-pr: known reds', log)
                self.assertIn('pre-pr: reds: 1 entries (temper/reds)', log)
                summary = log.rsplit('pre-pr: passed (' if accepted else 'pre-pr: refused (', 1)[1]
                self.assertIn(error['name'], summary)
                if accepted:
                    self.assertIn('inherited: ' + error['name'] + ' (red, temper/reds)', log)
                    self.assertIn(f'pre-pr: {check}: inherited,', log)
                else:
                    self.assertNotIn('inherited: ' + error['name'], log)
                    self.assertIn(error['log'], summary)
                self.assertFalse(any('temper-pre-pr/base' in str(cwd) for _, cwd in calls))
                self.assertFalse((self.root / '.git/temper-pre-pr/base').exists())
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
        error = PRE_PR.failure('test api::rules rule', 'goal red output', ('api::rules', 'rule'), 'api')
        answer = (100, [error], None)
        result, log, _, code = self.deliver(one_process_answer=answer)
        self.assertEqual(result, 0, log)
        self.assertTrue(all(code))
        cache = self.root / '.git/pre-pr-passed'
        tree, base, digest = cache.read_text().split()
        self.assertEqual(digest, hashlib.sha256(content).hexdigest())
        hit = f'pre-pr: passed before (tree {tree}, base {base}, reds {digest})'
        fault = [PRE_PR.failure('format', 'format fault')]
        result, log, _, code = self.deliver(one_process_answer=answer, formatting_answer=fault)
        self.assertEqual(result, 0, log)
        self.assertFalse(any(code))
        self.assertIn(hit, log)
        self.assertLess(log.index('pre-pr: reds:'), log.index(hit))
        record = cache.read_text()
        cache.write_text(f'{tree} {base}\n')
        result, log, _, code = self.deliver(formatting_answer=fault)
        self.assertEqual(result, 1, log)
        self.assertTrue(all(code), 'two-field records must not hit')
        self.assertNotIn('passed before', log)
        cache.write_text(record)
        for changed in (b'temper-reds 1\n', b'bad header\n', b'bad header changed\n', None):
            with self.subTest(changed=changed):
                previous = cache.read_text()
                if changed is None:
                    path.unlink()
                else:
                    path.write_bytes(changed)
                result, log, _, code = self.deliver(one_process_answer=answer)
                self.assertEqual(result, 1, log)
                self.assertTrue(all(code), log)
                self.assertNotIn('passed before', log)
                self.assertIn('test api::rules rule', log)
                self.assertNotIn('inherited: test', log)
                self.assertEqual(cache.read_text(), previous, 'refusal keeps the prior pass')
                result, log, _, code = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertTrue(all(code), log)
                self.assertNotIn('passed before', log)
                self.assertEqual(cache.read_text().split()[2],
                                 '-' if changed is None else hashlib.sha256(changed).hexdigest())

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
            dict(case='no file', code=100, tests=[test], reds=None, error='test api::rules rule', tail='stderr tail'),
            dict(case='error node', code=100, tests=[test], error='test api::rules rule',
                 files={'junit.xml': junit.replace('failure>', 'error>')}),
            dict(case='alone passed', code=100, tests=[dict(test, alone='passed', before=['first', 'second'])],
                 error='test api::rules rule', tail='may depend on an earlier test: first, second'),
            dict(case='alone failed', code=100, tests=[dict(test, alone='failed', before=['first'])],
                 error='test api::rules rule', tail='stderr tail'),
            dict(case='goal red', code=100, tests=[test], reds='- test api::rules rule', inherited=True),
            dict(case='different binary', code=100, tests=[test], reds='- test api::other rule',
                 error='test api::rules rule'),
            dict(case='empty 100', code=100, error='check one-process', tail='exit 100 without JUnit failures',
                 files={'junit.xml': '<testsuites/>'}),
            dict(case='build', code=2, output='compiler output\n' + build + '\n', error='build', tail='no tests ran'),
            dict(case='lint test and build failures', code=2, tests=[test],
                 clippy=True, output=build + '\n', error='build', tail='no tests ran'),
            dict(case='build line not last', code=2, output=build + '\nmore output\n',
                 error='check one-process', tail='exit 2'),
            dict(case='refusal', code=2, error='check one-process', tail='exit 2'),
            dict(case='other exit', code=102, error='check one-process', tail='exit 102'),
            dict(case='starved run', code=0, starved=True, warning=warning, reds='- test api::rules rule',
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

                entries = row.get('reds', '- test api::rules rule' if
                                  row.get('error') in ('build', 'check one-process') else '')
                if row.get('error') == 'build':
                    entries += '\n- lint api clippy::rule api/src/lib.rs'
                path = self.goal_file('reds', ('temper-reds 1\n' + (entries + '\n' if entries else '')).encode())
                if entries is None:
                    path.unlink()
                lint = PRE_PR.failure('clippy api clippy::rule api/src/lib.rs', 'lint fault',
                                      ('api', 'clippy::rule', 'api/src/lib.rs', 'lint fault'), 'api')
                result, log, calls, _ = self.deliver(('api', 'client'), one_process_answer=None, process_answer=fake,
                                                    clippy_answer=(101, [lint]) if row.get('clippy') else (0, []))
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
                        self.assertIn('inherited: test api::rules rule (red, temper/reds)', summary)
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
                self.assertNotIn('inherited: build', log)
                self.assertNotIn('pre-pr: known reds', log)
                self.assertFalse(any('temper-pre-pr/base' in str(cwd) for _, cwd in calls))
                self.assertFalse((self.root / '.git/temper-pre-pr/base').exists())
                self.assertFalse(any('cargo' in args and 'nextest' in args for args, _ in calls))
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)

    def ledger_run(self, tests, again=None, failure_output='under load\n\nfirst log tail', **kwargs):
        """Fake only the one-process boundary, including fresh again directories."""
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.again_calls = []
        self.initial_run = None
        again = {} if again is None else again

        def fake(arguments, cwd, env, output, stdout):
            self.assertEqual(cwd, self.root)
            self.assertTrue(stdout)
            self.assertIsNone(output)
            self.assertNotIn('GIT_INDEX_FILE', env)
            directory = Path(arguments[arguments.index('--out') + 1])
            self.assertFalse(directory.exists())
            if '--again' in arguments:
                self.again_calls.append(arguments)
                self.assertEqual(arguments[:8], ['python3', '-I', '-B', 'tools/tiers/one-process',
                                                 '--again', str(self.initial_run), '--out', str(directory)])
                self.assertTrue((self.initial_run / 'results.json').exists())
                self.assertEqual(directory, self.initial_run.parent / f'again-{len(self.again_calls)}')
                binary, *names = arguments[8:]
                original = next(t for t in tests if t['binary_id'] == binary and t['test_name'] == names[-1])
                self.assertEqual(names, original.get('before', []) + [original['test_name']])
                answer = again.get((binary, names[-1]), {})
                if answer.get('missing'):
                    return 0, 'no directory\n'
                directory.mkdir()
                content = answer.get('content', json.dumps({'tests': [dict(original, outcome=answer.get('outcome', 'passed'))]}))
                (directory / 'results.json').write_text(content)
                return answer.get('code', 0), 'again output, not the failure log\n'
            self.assertIsNone(self.initial_run)
            self.initial_run = directory
            directory.mkdir()
            (directory / 'run.json').write_text('{"starved": false}')
            (directory / 'results.json').write_text(json.dumps({'tests': tests, 'unobserved': []}))
            document = PRE_PR.ET.Element('testsuites')
            for test in tests:
                suite = PRE_PR.ET.SubElement(document, 'testsuite', name=test['binary_id'])
                case = PRE_PR.ET.SubElement(suite, 'testcase', name=test['test_name'])
                if test['outcome'] in ('failed', 'crashed', 'cancelled'):
                    PRE_PR.ET.SubElement(case, 'failure').text = failure_output
            (directory / 'junit.xml').write_bytes(PRE_PR.ET.tostring(document))
            return (100 if any(t['outcome'] in ('failed', 'crashed') for t in tests) else 0), 'first run\n'

        return self.deliver(one_process_answer=None, process_answer=fake, **kwargs)

    def test_summary_fits_the_kept_tail(self):
        output = ('x' * 63 + '\n') * 64
        self.assertEqual(len(output.encode()), 4096)
        tests = [dict(binary_id='api::rules', test_name=f'rule_{number:02}', outcome='failed')
                 for number in range(40)]
        result, log, _, _ = self.ledger_run(tests, failure_output=output)
        self.assertEqual(result, 1, log)
        summary = 'pre-pr: refused' + log.rsplit('pre-pr: refused', 1)[1]
        self.assertLessEqual(len(summary.encode()), 8192)
        self.assertIn('test api::rules rule_00', summary)
        self.assertIn(' bytes cut]', summary)
        entries = summary.split('test api::rules ')[1:]
        self.assertIn(f'[… {40 - len(entries)} more failures]', summary)
        for entry in entries:
            with self.subTest(test=entry.splitlines()[0]):
                lines = entry.split('\n', 1)[1].splitlines()
                tail = '\n'.join(line for line in lines if ' more failures]' not in line)
                self.assertLess(len(tail.encode()), 1536)
                marker, content = tail.split('\n', 1)
                self.assertRegex(marker, r'^\[… [0-9]+ bytes cut\]$')
                self.assertTrue(all(len(line) == 63 for line in content.splitlines()), content)
                self.assertTrue(content.endswith('x' * 63))

    def test_process_again_marks_only_a_passed_target_as_a_flake(self):
        self.goal_file('rerun', b'temper-rerun 1 4\n')
        test = dict(binary_id='api::rules', test_name='rule', outcome='failed',
                    before=['first', 'second'], alone='passed')
        key = ('api::rules', 'rule')
        for answer, flake in (({}, True), ({'code': 100}, True), ({'outcome': 'failed'}, False),
                              ({'outcome': 'cancelled'}, False), ({'outcome': 'crashed'}, False),
                              ({'code': 2}, False), ({'missing': True}, False),
                              ({'content': 'bad json'}, False), ({'content': '{"tests": []}'}, False)):
            with self.subTest(answer=answer):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                result, log, _, _ = self.ledger_run([test], {key: answer})
                self.assertEqual(result, int(not flake), log)
                self.assertEqual(len(self.again_calls), 1)
                self.assertFalse(self.initial_run.exists(), 'temporary run must be removed')
                summary = log.rsplit('pre-pr: passed (' if flake else 'pre-pr: refused (', 1)[1]
                if flake:
                    self.assertRegex(summary, r'flake: test api::rules rule \(failed under load; passed when '
                                     r'its process ran again: 3 tests, [0-9.]+ s\)')
                    self.assertIn('pre-pr: tests: inherited,', log)
                    self.assertNotIn('inherited: test', summary)
                else:
                    self.assertIn('\ntest api::rules rule\n', summary)
                    self.assertIn('pre-pr: tests: failed,', log)
                self.assertIn('under load\n\nfirst log tail\nmay depend on an earlier test: first, second', summary)
                self.assertNotIn('again output', summary)

    def test_process_again_excludes_reds_and_runs_in_key_order(self):
        self.goal_file('rerun', b'temper-rerun 1 2\n')
        self.goal_file('reds', b'temper-reds 1\n- test api::rules goal\n')
        tests = [dict(binary_id='api::rules', test_name=name, outcome='crashed')
                 for name in ('z', 'goal', 'a')]
        tests += [dict(binary_id='api::rules', test_name=outcome, outcome=outcome)
                  for outcome in ('cancelled', 'starved', 'passed')]
        result, log, _, _ = self.ledger_run(tests)
        self.assertEqual(result, 0, log)
        self.assertEqual([args[8:] for args in self.again_calls], [['api::rules', 'a'], ['api::rules', 'z']])
        self.assertIn('pre-pr: tests again: api::rules a: no before recorded; running the test alone', log)
        summary = log.rsplit('pre-pr: passed (', 1)[1]
        self.assertLess(summary.index('flake: test api::rules a'), summary.index('flake: test api::rules z'))
        self.assertLess(summary.index('flake: test api::rules z'), summary.index('inherited: test api::rules'))
        self.assertIn('inherited: test api::rules goal (red, temper/reds)', summary)

    def test_rerun_bound_is_strict_and_all_or_none(self):
        test = dict(binary_id='api::rules', test_name='rule', outcome='failed')
        rows = [(b'temper-rerun 1 4\n', 5, False), (b'temper-rerun 1 0\n', 1, False),
                (None, 1, False), (b'temper-rerun 1 16', 1, True),
                (b'temper-rerun 1 17\n', 1, False), (b'temper-rerun 2 4\n', 1, False),
                (b'temper-rerun 1 -1\n', 1, False), (b'temper-rerun 1 4 \n', 1, False),
                (b'temper-rerun 1 4\n\n', 1, False), (b'\xff', 1, False),
                (b'temper-rerun 1 ' + b'9' * 5000, 1, False)]
        for content, count, runs in rows:
            with self.subTest(content=content, count=count):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                path = self.goal_file('rerun', content or b'')
                if content is None:
                    path.unlink()
                tests = [dict(test, test_name=f'rule{i}') for i in range(count)]
                result, log, _, _ = self.ledger_run(tests)
                self.assertEqual(result, int(not runs), log)
                self.assertEqual(len(self.again_calls), count if runs else 0)
                malformed = content is not None and content not in (rows[0][0], rows[1][0], rows[3][0])
                self.assertEqual('pre-pr: rerun: ignored temper/rerun (malformed)' in log, malformed)
                if not runs:
                    bound = 4 if count == 5 else 0
                    self.assertIn(f'pre-pr: tests again: none ({count} failures, at most {bound} run again)', log)

    def test_check_results_reports_observed_goal_reds_and_flakes_atomically(self):
        self.goal_file('reds-wait', b'temper-reds-wait 1 1\n')
        self.goal_file('rerun', b'temper-rerun 1 4\n')
        names = ('passed', 'failed', 'crashed', 'starved', 'cancelled', 'ignored', 'unobserved')
        test_keys = ['test api::rules ' + name for name in names]
        lint_keys = ['lint api clippy::bad api/src/lib.rs', 'lint api clippy::good api/src/lib.rs',
                     'lint other clippy::good other/src/lib.rs']
        self.goal_file('reds', ('temper-reds 1\n' + ''.join('- ' + key + '\n' for key in test_keys + lint_keys)).encode())
        tests = [dict(binary_id='api::rules', test_name=name, outcome=name) for name in names[:-1]]
        tests.append(dict(binary_id='api::rules', test_name='flake', outcome='failed'))
        lint = PRE_PR.failure('clippy api clippy::bad api/src/lib.rs', 'lint output',
                              ('api', 'clippy::bad', 'api/src/lib.rs', 'message'), 'api')
        replacements = []
        replace = Path.replace

        def replacing(source, target):
            if source.name == 'check-results.tmp':
                replacements.append((source.name, target.name))
                self.assertTrue(source.read_text().startswith('temper-check 1\ncheck '))
            return replace(source, target)

        with mock.patch.object(Path, 'replace', replacing):
            result, log, _, _ = self.ledger_run(tests, clippy_answer=(101, [lint]))
        self.assertEqual(result, 0, log)
        path = self.root / '.git/temper/check-results'
        lines = path.read_text().splitlines()
        tree, base, _ = (self.root / '.git/pre-pr-passed').read_text().split()
        self.assertEqual(lines[:2], ['temper-check 1', f'check {tree} {base} passed'])
        self.assertEqual(lines[2:7], ['failed ' + lint_keys[0], 'passed ' + lint_keys[1],
                                     'failed test api::rules crashed', 'failed test api::rules failed',
                                     'passed test api::rules passed'])
        self.assertEqual(lines[7:], ['flake test api::rules flake', '| under load', '|', '| first log tail'])
        self.assertEqual(replacements, [('check-results.tmp', 'check-results')])
        self.assertFalse(path.with_suffix('.tmp').exists())
        # A cache hit leaves the prior result alone, rather than claiming new observations.
        path.write_text('previous result\n')
        self.goal_file('rerun', b'malformed\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertIn('passed before', log)
        self.assertLess(log.index('pre-pr: rerun: ignored'), log.index('passed before'))
        self.assertEqual(path.read_text(), 'previous result\n')

    def test_check_results_marker_refusal_and_uncoded_lints(self):
        self.goal_file('reds', b'temper-reds 1\n- lint api clippy::rule api/src/lib.rs\n')
        path = self.root / '.git/temper/check-results'
        for marker, error, expected in (
                (None, None, None), (b'temper-reds-wait 1 0\n', None, None),
                (b'temper-reds-wait 1 1\n', None, 'passed lint api clippy::rule api/src/lib.rs\n'),
                (b'temper-reds-wait 1 1\n', PRE_PR.failure('check clippy', 'tool error'), ''),
                (b'temper-reds-wait 1 1\n', PRE_PR.failure('clippy api  api/src/lib.rs', 'compiler error',
                 ('api', '', 'api/src/lib.rs', 'compiler error'), 'api'), '')):
            with self.subTest(marker=marker, error=error):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                path.unlink(missing_ok=True)
                mark = self.goal_file('reds-wait', marker or b'')
                if marker is None:
                    mark.unlink()
                result, log, _, _ = self.ledger_run([], clippy_answer=(101, [error]) if error else (0, []))
                self.assertEqual(result, int(error is not None), log)
                if expected is None:
                    self.assertFalse(path.exists())
                else:
                    lines = path.read_text().splitlines(keepends=True)
                    self.assertTrue(lines[1].endswith(' refused\n' if error else ' passed\n'))
                    self.assertEqual(''.join(lines[2:]), expected)
        # A check without a pin still reports its tree, but no test or lint selections.
        self.command('git', 'update-ref', '-d', 'refs/temper/landing')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertRegex(path.read_text(), r'\Atemper-check 1\ncheck [0-9a-f]{40} - passed\n\Z')

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
