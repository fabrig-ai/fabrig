"""Delivery selection, goal reds and inherited counts (tools/pre-pr/design.md, 3, 5, 9)."""
import contextlib
import fnmatch
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
        for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR', 'TIERS_HOLD', 'TIERS_LEDGER'):
            self.env.pop(name, None)
        self.command('git', 'init', '-q')
        self.command('git', 'config', 'user.name', 'Fixture')
        self.command('git', 'config', 'user.email', 'fixture@example.invalid')
        self.command('git', 'config', 'commit.gpgsign', 'false')
        self.write('.gitignore', '/target/\n')
        self.write('Cargo.toml', '[workspace]\nmembers = ["api"]\n')
        self.write('api/Cargo.toml', '[package]\nname = "api"\n')
        self.write('api/src/lib.rs', 'pub fn value() {}\n')
        self.write('guide.md', '# Guide\n')
        self.write('.config/gate/read-by-tests.txt', 'api/docs/input.txt\n')
        self.write('.config/gate/always.txt', 'api shared/fixtures/\n')
        self.write('tools/tiers/packages.toml', '[test-binaries]\n')
        self.write('tools/tiers/figures.py', 'raise SystemExit(0)\n')
        source = Path(__file__).resolve().parents[1] / 'citations'
        for name in ('citations.py', 'sized.txt'):
            self.write('tools/citations/' + name, (source / name).read_text())
        source = Path(__file__).resolve().parents[1] / 'outward'
        for name in ('outward.py', 'names.txt'):
            self.write('tools/outward/' + name, (source / name).read_text())
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

    def metadata(self, packages, directories=None, dependencies=None):
        dependencies = {} if dependencies is None else dependencies
        return {'packages': [{'name': name, 'id': name,
                              'dependencies': [dict(dependency, path=str(self.root / dependency['name'].lower()))
                                               for dependency in dependencies.get(name, [])],
                              'manifest_path': str(self.root / (directories or {}).get(name, name.lower()) / 'Cargo.toml')} for name in packages],
                'workspace_members': list(packages), 'target_directory': str(self.root / 'target')}

    def deliver(self, packages=('api',), expected_bundle=None,
                clippy_answer=(0, []), one_process_answer=(0, [], None), process_answer=None,
                build_answer=(0, ''), events=None, directories=None, dependencies=None, formatting_answer=(), hold=None,
                metadata_forbidden=False, stderr=None):
        calls = []
        events = [] if events is None else events
        data = self.metadata(packages, directories, dependencies)

        def run(arguments, cwd, env, output=None, stdout=False):
            calls.append((arguments, cwd))
            self.assertNotIn('TIERS_LEDGER', env)
            if arguments[:2] == ['cargo', 'build']:
                events.append(arguments)
                self.assertEqual(env['TEMPER_INTERNAL_BUNDLE'], 'none')
                self.assertNotIn('TIERS_HOLD', env)
                return build_answer
            if process_answer is not None and arguments[3:4] in (['tools/tiers/one-process'], ['tools/tiers/cpu-summary']):
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
            if args[6] != 'default':
                return 0, [], None
            code, errors, starved = one_process_answer
            return code, [error.copy() for error in errors], starved

        captured = io.BytesIO()
        log = io.TextIOWrapper(captured, encoding="utf-8")
        holder = (mock.patch.object(PRE_PR.runpy, 'run_path', return_value={'hold': hold})
                  if hold is not None else contextlib.nullcontext())
        with contextlib.chdir(self.root), mock.patch.dict(os.environ, self.env, clear=True), holder, \
                mock.patch.object(PRE_PR, 'metadata', return_value=data,
                                  side_effect=AssertionError('cargo metadata ran') if metadata_forbidden else None), \
                mock.patch.object(PRE_PR, 'formatting', return_value=list(formatting_answer)) as formatting, \
                mock.patch.object(PRE_PR, 'clippy', side_effect=clippy_check) as clippy, \
                mock.patch.object(PRE_PR, 'one_process', side_effect=test_check) as one_process, \
                mock.patch.object(PRE_PR, 'run', side_effect=run), \
                contextlib.redirect_stdout(log), contextlib.redirect_stderr(stderr if stderr is not None else log):
            result = PRE_PR.main()
        self.clippy_mock = clippy
        self.one_process_mock = one_process
        log.flush()
        if expected_bundle is not None:
            self.assertTrue(clippy.called, captured.getvalue().decode())
            self.assertTrue(one_process.called, captured.getvalue().decode())
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
                self.write('temper/addition.md', '# Delivery\n')
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
            ('untracked', 'a/src/new.rs', 'B\n', '', ['A'], ['A', 'B']),
            ('deleted', 'a/src/lib.rs', 'B\n', '', ['A'], ['A', 'B']),
            ('root manifest', 'Cargo.toml', 'B\n', '', [], ['B']),
            ('docs', 'temper/selection.md', 'B\n', '', [], ['B']),
            ('reader', 'a/src/lib.rs', 'B a/\n', '', ['A'], ['A', 'B']),
            ('reader only', 'temper/docs/selection.md', 'B temper/docs/\n',
             'temper/docs/selection.md\n', [], ['B']),
            ('component boundary', 'a-sibling/new.rs', 'B\n', '', [], ['B']),
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

    def test_selection_excludes_merged_inputs(self):
        for pending in (True, False):
            with self.subTest(pending=pending):
                self.setUp()
                self.write('beta/Cargo.toml', '[package]\nname = "beta"\n')
                self.write('beta/src/lib.rs', 'pub fn base() {}\n')
                self.write('.config/gate/always.txt', 'reader beta/\n')
                self.pin()
                start = self.command('git', 'rev-parse', 'HEAD').strip()
                self.write('beta/src/lib.rs', 'pub fn merged() {}\n')
                self.write('guide.md', '# Merged guide\n')
                self.command('git', 'commit', '-qam', 'merged inputs')
                merged = self.command('git', 'rev-parse', 'HEAD').strip()
                self.command('git', 'checkout', '-q', '--detach', start)
                self.command('git', 'commit', '--allow-empty', '-qm', 'branch')
                self.command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
                self.command('git', 'merge', '--no-ff', '--no-commit', merged)
                if not pending:
                    self.command('git', 'commit', '-qm', 'merge inputs')
                result, log, calls, code = self.deliver(
                    ('beta', 'dependent', 'reader', 'fabrig-boundary'),
                    dependencies={'dependent': [{'name': 'beta', 'kind': None}]})
                self.assertEqual(result, 0, log)
                self.assertEqual(self.clippy_mock.call_args.args[0] if self.clippy_mock.called else [], [], log)
                self.assertEqual(self.one_process_mock.call_args.args[0] if self.one_process_mock.called else [], [], log)
                self.assertEqual(code, (False, False, False), log)
                self.assertFalse(calls, log)
                line = f"pre-pr: merged tips: {merged}: what they hold is not the delivery's\n"
                selection = 'pre-pr: no code or docs inputs changed\n'
                self.assertEqual(log.count(line), 1, log)
                self.assertIn(selection, log)
                self.assertLess(log.index(line), log.index(selection))

    def test_selection_keeps_merge_resolutions_and_later_edits(self):
        for row in ('resolution', 'later edit'):
            with self.subTest(row=row):
                self.setUp()
                self.write('beta/Cargo.toml', '[package]\nname = "beta"\n')
                self.write('beta/src/lib.rs', 'pub fn base() {}\n')
                self.pin()
                start = self.command('git', 'rev-parse', 'HEAD').strip()
                self.write('beta/src/lib.rs', 'pub fn merged() {}\n')
                self.command('git', 'commit', '-qam', 'merged source')
                merged = self.command('git', 'rev-parse', 'HEAD').strip()
                self.command('git', 'checkout', '-q', '--detach', start)
                if row == 'resolution':
                    self.write('beta/src/lib.rs', 'pub fn branch() {}\n')
                self.command('git', 'commit', '--allow-empty', '-qam', 'branch')
                self.command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
                if row == 'resolution':
                    with self.assertRaises(subprocess.CalledProcessError) as conflict:
                        self.command('git', 'merge', '--no-ff', '--no-commit', merged)
                    self.assertIn('CONFLICT (content)', conflict.exception.stdout)
                    self.write('beta/src/lib.rs', 'pub fn resolution() {}\n')
                    self.command('git', 'add', 'beta/src/lib.rs')
                else:
                    self.command('git', 'merge', '--no-ff', '--no-commit', merged)
                self.command('git', 'commit', '-qm', 'merge source')
                if row == 'later edit':
                    self.write('beta/src/lib.rs', 'pub fn edited() {}\n')
                result, log, _, _ = self.deliver(('beta',))
                self.assertEqual(result, 0, log)
                self.assertEqual(self.clippy_mock.call_args.args[0] if self.clippy_mock.called else [], ['beta'], log)
                self.assertEqual(self.one_process_mock.call_args.args[0] if self.one_process_mock.called else [], ['beta'], log)
                line = f"pre-pr: merged tips: {merged}: what they hold is not the delivery's\n"
                self.assertEqual(log.count(line), 1, log)
                self.assertLess(log.index(line), log.index('pre-pr: code only:'))

    def test_selection_path_failure_is_not_empty_change(self):
        with mock.patch.object(PRE_PR, 'delivery_paths', side_effect=RuntimeError('cannot read paths')):
            result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertIn(dict(name='check selection', key=None, **{'class': 'new'}), record['failures'])
        self.assertIn('cannot read paths', log)

    def test_nested_package_ownership(self):
        data = self.metadata(('A', 'B'), directories={'A': 'a', 'B': 'a/b'})
        for path, package in (('a/b/c.rs', 'B'), ('a/bc/d.rs', 'A'), ('a/x.rs', 'A'),
                              ('a/b/input.bin', 'B'), ('a/../input.bin', 'A'), ('a', 'A')):
            with self.subTest(path=path):
                self.assertEqual(PRE_PR.code_inputs([path], data, self.root), [path])
                self.assertEqual(PRE_PR.select(data, {self.root / path: path}, ''),
                                 ([package], [package]))

    def test_selection_large_change(self):
        # 400 packages, with a child package under each of the 200 parents.
        directories = {f'{kind}-{i}': f'packages/p{i}' + suffix
                       for i in range(200) for kind, suffix in (('parent', ''), ('child', '/nested'))}
        directories.update(dependent='dependent', transitive='transitive', reader='reader')
        data = self.metadata(directories, directories=directories, dependencies={
            'dependent': [{'name': 'parent-0', 'kind': 'dev'}],
            'transitive': [{'name': 'dependent', 'kind': None}],
        })
        patterns = ('packages/p0/docs/read-*.md', 'packages/*/nested/docs/read-*.md',
                    'packages/p1/docs/note-*.md')
        self.write('.config/gate/read-by-tests.txt', patterns[0] + '\n')
        self.write('.config/fixture-readers.toml',
                   f'[[fixture]]\npaths = ["{patterns[1]}"]\nregenerate = ["{patterns[2]}"]\n'
                   '[[fixture.reader]]\npair = "default"\npackage = "reader"\nfilter = "kind(lib)"\n')
        always = 'reader packages/p0-sibling/\n'
        self.write('.config/gate/always.txt', always)
        paths = []
        for i in range(200):
            for j in range(50):
                for suffix in ('', '/nested', '/nested-sibling', '-sibling', '/docs', '/nested/docs'):
                    name = (f'read-{j}.md' if j % 2 else f'note-{j}.md') if 'docs' in suffix else f'value-{j}.bin'
                    paths.append(f'packages/p{i}{suffix}/{name}')

        # Reference: the longest package directory prefix, by whole components.
        # It enumerates the rule, independently of the driver's parent walk.
        package_parts = {Path(directory).parts: name for name, directory in directories.items()}
        owners = {}
        for path in paths:
            parts = Path(path).parts
            directory = max((directory for directory in package_parts
                             if parts[:len(directory)] == directory), key=len, default=None)
            owners[path] = package_parts.get(directory)
        expected_code = [path for path in paths
                         if (not path.endswith('.md') and owners[path] is not None)
                         or any(fnmatch.fnmatchcase(path, pattern) for pattern in patterns)
                         or path.startswith('packages/p0-sibling/')]
        expected_lints = {owners[path] for path in expected_code} - {None}
        expected_tests = expected_lints | {'dependent', 'reader'}

        selected = PRE_PR.code_inputs(paths, data, self.root)
        self.assertEqual(selected, expected_code)
        lints, tests = PRE_PR.select(data, {self.root / path: path for path in selected}, always)
        self.assertEqual(lints, sorted(expected_lints))
        self.assertEqual(tests, sorted(expected_tests))

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

    def test_waiting_hold_delays_metadata_until_held(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.write('hold.py', """import os, sys, time
from pathlib import Path
assert 'TIERS_HOLD' not in os.environ
with Path('hold-events').open('a') as events:
    events.write(f'waiting {time.monotonic()}\\n')
print('waiting for a build place', flush=True)
time.sleep(0.3)
print('waiting for a build place', flush=True)
time.sleep(0.8)
with Path('hold-events').open('a') as events:
    events.write(f'held {time.monotonic()}\\n')
print('held', flush=True)
sys.stdin.read()
""")
        self.env['TIERS_HOLD'] = 'python3 -I -B hold.py'
        real_hold = PRE_PR.runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))['hold']
        stderr = io.StringIO()

        @contextlib.contextmanager
        def hold(*args, **kwargs):
            def metadata(*args):
                with (self.root / 'hold-events').open('a') as events:
                    events.write(f'metadata {time.monotonic()}\n')
                return self.metadata(('api',))

            PRE_PR.metadata.side_effect = metadata
            with mock.patch.dict(real_hold.__wrapped__.__globals__, HOLD_READY_S=1):
                with real_hold(*args, **kwargs):
                    yield

        result, log, _, code = self.deliver(hold=hold, stderr=stderr)
        events = [line.split() for line in (self.root / 'hold-events').read_text().splitlines()]
        self.assertEqual([event[0] for event in events], ['waiting', 'held', 'metadata'], events)
        self.assertLessEqual(float(events[1][1]), float(events[2][1]), events)
        self.assertEqual(result, 0, log)
        self.assertTrue(all(code))
        self.assertEqual(stderr.getvalue(), 'smith-check: waiting\nsmith-check: admitted\n')
        self.assertEqual(log.count('pre-pr: hold: waiting for a build place\n'), 1, log)
        self.assertRegex(log, r'(?m)^pre-pr: hold: held after [0-9]+ s$')
        self.assertNotIn('going on unheld', log)

    def test_hold_reads_whole_lines_and_stops_at_held(self):
        real_hold = PRE_PR.runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))['hold']
        for chunks, expected in (([b'held\nwaiting ignored\n'], ''),
                                 ([b'wait', b'ing for a build place\nwaiting again\nheld\nwaiting ignored\n'],
                                  'pre-pr: hold: waiting for a build place\npre-pr: hold: held after 0 s\n')):
            with self.subTest(chunks=chunks):
                output, stderr = io.StringIO(), io.StringIO()
                process = mock.Mock()
                process.poll.return_value = None
                environment = dict(self.env, TIERS_HOLD='hold')
                with mock.patch.dict(real_hold.__wrapped__.__globals__,
                                     subprocess=mock.Mock(Popen=mock.Mock(return_value=process)),
                                     select=mock.Mock(select=mock.Mock(return_value=([process.stdout], [], []))),
                                     os=mock.Mock(read=mock.Mock(side_effect=chunks)),
                                     time=mock.Mock(monotonic=mock.Mock(return_value=0))), \
                        contextlib.redirect_stderr(stderr):
                    with real_hold(['hold'], 'pre-pr', self.root, environment, output=output):
                        self.assertFalse(process.stdin.close.called)
                self.assertEqual(output.getvalue(), expected)
                self.assertEqual(stderr.getvalue(), 'smith-check: waiting\nsmith-check: admitted\n' if expected else '')
                process.kill.assert_not_called()
                process.stdin.close.assert_called_once_with()
                process.stdout.close.assert_called_once_with()

    def test_waiting_hold_failure_resumes_before_warning(self):
        for mode in ('silent', 'exit', 'closed'):
            with self.subTest(mode=mode):
                self.setUp()
                self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                self.write('hold.py', """import os, sys, time
print('waiting for a build place', flush=True)
if sys.argv[1] == 'silent':
    time.sleep(5)
elif sys.argv[1] == 'closed':
    os.close(1)
    sys.stdin.read()
else:
    sys.exit(3)
""")
                self.env['TIERS_HOLD'] = 'python3 -I -B hold.py ' + mode
                real_hold = PRE_PR.runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))['hold']
                fault = PRE_PR.failure('format', 'format fault')
                with mock.patch.dict(real_hold.__wrapped__.__globals__, HOLD_READY_S=0.5):
                    result, log, _, code = self.deliver(formatting_answer=[fault], hold=real_hold)
                self.assertEqual(result, 1, log)
                self.assertEqual(code, (True, True, True))
                self.assertEqual(log.count('smith-check: waiting\n'), 1, log)
                self.assertEqual(log.count('smith-check: admitted\n'), 1, log)
                warning = log.index('; going on unheld')
                self.assertLess(log.index('smith-check: admitted\n'), log.rfind('pre-pr: hold:', 0, warning))
                self.assertLess(warning, log.index('pre-pr: format …'))
                self.assertIn('format fault', log)
                self.assertFalse((self.root / '.git/pre-pr-passed').exists())

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

    def test_failure_file_uses_root_target_without_touching_the_git_directory(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.env['CARGO_TARGET_DIR'] = str(self.root / 'other-target')
        old = self.root / '.git/temper/pre-pr-failures.json'
        self.write('.git/temper/pre-pr-failures.json', 'previous check\n')
        errors = [PRE_PR.failure('test api::rules ' + name, 'fault', ('api::rules', name))
                  for name in ('first', 'second')]
        result, log, _, _ = self.deliver(one_process_answer=(100, errors, None))
        self.assertEqual(result, 1, log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(record['failures'], [
            {'class': 'new', 'name': error['name'],
             'key': {'kind': 'test', 'binary': 'api::rules', 'test': name}}
            for error, name in zip(errors, ('first', 'second'))])
        self.assertIn('failures: 2 new, 0 flakes, 0 inherited; every key in target/temper/pre-pr-failures.json\n', log)
        self.assertEqual(old.read_text(), 'previous check\n')
        self.assertFalse((self.root / 'other-target/temper/pre-pr-failures.json').exists())

    def test_failure_file_keeps_mixed_keys_beyond_the_summary_cap(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        path = self.root / 'target/temper/pre-pr-failures.json'
        for count in (26, 194, 349):
            with self.subTest(count=count):
                errors, expected = [], []
                for index in range(count):
                    name = f'rule_{index:03}'
                    cases = [
                        (f'test api::rules {name}', ('api::rules', name),
                         {'kind': 'test', 'binary': 'api::rules', 'test': name}),
                        (f'clippy api E0001 {name}.rs', ('api', 'E0001', name + '.rs', name),
                         {'kind': 'lint', 'package': 'api', 'code': 'E0001', 'file': name + '.rs', 'message': name}),
                        (f'old-forms skein {name}', ('skein', name, index + 1, index),
                         {'kind': 'old-forms', 'tree': 'skein', 'form': name, 'count': index + 1, 'baseline': index}),
                        ('check ' + name, None, None),
                    ]
                    display, key, structured = cases[index % len(cases)]
                    error = PRE_PR.failure(display, 'failure log\n' * 150, key)
                    kind = 'new'
                    if index < 3:
                        error['inherited'] = True
                        if index == 0:
                            error['flake'] = (2, 0.125)
                        kind = 'inherited'
                    elif index % 4 == 0:
                        error['flake'] = (2, 0.125)
                        kind = 'flake'
                    errors.append(error)
                    expected.append({'class': kind, 'name': display, 'key': structured})
                expected = ([entry for entry in expected if entry['class'] == 'new'] +
                            sorted((entry for entry in expected if entry['class'] == 'flake'),
                                   key=lambda entry: entry['key']['test']) +
                            [entry for entry in expected if entry['class'] == 'inherited'])
                counts = {kind: sum(entry['class'] == kind for entry in expected)
                          for kind in ('new', 'flake', 'inherited')}
                # Large sets exercise the header's cut after its two kept lines.
                with mock.patch.object(PRE_PR, 'judge_base', return_value=''), \
                        mock.patch.object(PRE_PR, 'select', return_value=(['é' * 3000], ['api'])):
                    result, log, _, _ = self.deliver(one_process_answer=(100, errors, None))
                self.assertEqual(result, 1, log)
                self.assertFalse((self.root / '.git/temper/pre-pr-failures.json').exists())
                text = path.read_text(encoding='utf-8')
                record = json.loads(text)
                self.assertEqual(record['failures'], expected)
                self.assertEqual(record['counts'], counts)
                self.assertEqual(record['form'], 'pre-pr-failures 1')
                self.assertEqual(record['base'], self.command('git', 'rev-parse', 'HEAD').strip())
                self.assertEqual(record['verdict'], 'refused')
                self.assertFalse(record['cached'])
                lines = text.splitlines()
                self.assertTrue(lines[0].endswith(', "failures": ['))
                self.assertEqual([json.loads(line.rstrip(',')) for line in lines[1:-1]], expected)
                self.assertEqual(lines[-1], ']}')
                summary = 'pre-pr: refused (' + log.rsplit('pre-pr: refused (', 1)[1]
                header = summary.splitlines()
                self.assertTrue(header[0].startswith('pre-pr: refused (base '))
                self.assertEqual(header[1], f'failures: {counts["new"]} new, {counts["flake"]} flakes, '
                                 f'{counts["inherited"]} inherited; every key in target/temper/pre-pr-failures.json')
                self.assertLessEqual(len(summary.encode()), 8192)
                before_failure, _ = summary.split('\n' + expected[0]['name'] + '\n', 1)
                self.assertLessEqual(len(before_failure.encode()) + 1, 4096)
                self.assertIn(' more failures]', summary)

    def test_failure_file_keeps_all_450_test_keys_in_class_order(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        errors = []
        for index in range(450):
            name = f'rule_{index:03}'
            error = PRE_PR.failure('test api::rules ' + name, 'fault\n' * 300, ('api::rules', name), 'api')
            if index < 10:
                error['inherited'] = True
            elif index < 15:
                error['flake'] = (2, 0.125)
            errors.append(error)
        errors.reverse()
        result, log, _, _ = self.deliver(one_process_answer=(100, errors, None))
        self.assertEqual(result, 1, log)
        path = self.root / 'target/temper/pre-pr-failures.json'
        record = json.loads(path.read_text())
        self.assertEqual(record['counts'], {'new': 435, 'flake': 5, 'inherited': 10})
        indices = list(range(449, 14, -1)) + list(range(10, 15)) + list(range(9, -1, -1))
        self.assertEqual(record['failures'], [
            {'class': 'new' if index >= 15 else 'flake' if index >= 10 else 'inherited',
             'name': f'test api::rules rule_{index:03}',
             'key': {'kind': 'test', 'binary': 'api::rules', 'test': f'rule_{index:03}'}}
            for index in indices])
        summary = 'pre-pr: refused (' + log.rsplit('pre-pr: refused (', 1)[1]
        self.assertEqual(summary.splitlines()[1],
                         'failures: 435 new, 5 flakes, 10 inherited; every key in target/temper/pre-pr-failures.json')
        self.assertLessEqual(len(summary.encode()), 8192)

    def test_failure_file_preserves_long_unicode_and_surrogate_keys(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        name = '規則_é' * 2000
        message = '不正な値 é ' + '\ud800'
        errors = [PRE_PR.failure('test api::rules ' + name, 'fault', ('api::rules', name)),
                  PRE_PR.failure('clippy api  ', 'fault', ('api', '', '', message))]
        result, log, _, _ = self.deliver(one_process_answer=(100, errors, None))
        self.assertEqual(result, 1, log)
        raw = (self.root / 'target/temper/pre-pr-failures.json').read_bytes()
        record = json.loads(raw)
        self.assertEqual(record['failures'], [
            {'class': 'new', 'name': errors[0]['name'],
             'key': {'kind': 'test', 'binary': 'api::rules', 'test': name}},
            {'class': 'new', 'name': errors[1]['name'],
             'key': {'kind': 'lint', 'package': 'api', 'code': '', 'file': '', 'message': message}}])
        self.assertIn(name.encode(), raw)
        self.assertIn(b'\\ud800', raw)

    def test_failure_file_outlives_the_one_process_directory(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        names = [f'rule_{index:03}' for index in range(26)]
        directories = []

        def fake(arguments, cwd, env, output, stdout):
            directory = Path(arguments[arguments.index('--out') + 1])
            directories.append(directory)
            directory.mkdir()
            if directory.name == 'fuzzy':
                (directory / 'run.json').write_text('{"starved": false}')
                (directory / 'results.json').write_text('{"tests": [], "unobserved": []}')
                return 0, 'fuzzy run\n'
            (directory / 'junit.xml').write_text('<testsuites><testsuite name="api::rules">' + ''.join(
                f'<testcase name="{name}"><failure>fault</failure></testcase>' for name in names) +
                '</testsuite></testsuites>')
            (directory / 'results.json').write_text(json.dumps({'tests': [
                dict(binary_id='api::rules', test_name=name, outcome='failed') for name in names], 'unobserved': []}))
            (directory / 'run.json').write_text('{"starved": false}')
            return 100, 'fake run\n'

        summarize = PRE_PR.summary
        path = self.root / 'target/temper/pre-pr-failures.json'

        def summary(*args):
            self.assertEqual(len(directories), 2)
            self.assertFalse(directories[0].parent.exists())
            self.assertEqual(len(json.loads(path.read_text())['failures']), len(names))
            return summarize(*args)

        with mock.patch.object(PRE_PR, 'summary', side_effect=summary):
            result, log, _, _ = self.deliver(one_process_answer=None, process_answer=fake)
        self.assertEqual(result, 1, log)
        self.assertEqual(len(directories), 2)
        self.assertFalse(directories[0].parent.exists())
        record = json.loads(path.read_text())
        self.assertEqual([entry['key'] for entry in record['failures']],
                         [{'kind': 'test', 'binary': 'api::rules', 'test': name} for name in names])

    def test_failure_file_uses_the_linked_worktrees_root(self):
        main = self.root
        linked = main / 'linked'
        self.command('git', 'worktree', 'add', '--detach', str(linked), 'HEAD')
        self.root = linked
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        error = PRE_PR.failure('format', 'fault')
        result, log, _, _ = self.deliver(one_process_answer=(100, [error], None))
        self.assertEqual(result, 1, log)
        path = linked / 'target/temper/pre-pr-failures.json'
        self.assertEqual(json.loads(path.read_text())['failures'],
                         [{'class': 'new', 'name': 'format', 'key': None}])
        self.assertIn('failures: 1 new, 0 flakes, 0 inherited; every key in target/temper/pre-pr-failures.json\n', log)
        self.assertFalse((main / 'target/temper/pre-pr-failures.json').exists())
        self.assertFalse((main / '.git/temper/pre-pr-failures.json').exists())
        self.assertFalse((main / '.git/worktrees/linked/temper/pre-pr-failures.json').exists())

    def test_failure_file_write_error_preserves_verdict_and_other_output(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.goal_file('reds-wait', b'temper-reds-wait 1 1\n')
        self.goal_file('reds', b'temper-reds 1\n')
        error = PRE_PR.failure('test api::rules rule', 'fault', ('api::rules', 'rule'))
        for errors in ([error], []):
            with self.subTest(refused=bool(errors)):
                path = self.root / 'target/temper/pre-pr-failures.json'
                path.unlink(missing_ok=True)
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                good, good_log, _, _ = self.deliver(one_process_answer=(100, errors, None))
                results = (self.root / '.git/temper/check-results').read_bytes()
                path.unlink()
                path.mkdir()
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                bad, bad_log, _, _ = self.deliver(one_process_answer=(100, errors, None))
                self.assertEqual(bad, good)
                self.assertEqual(bad, int(bool(errors)))
                self.assertEqual((self.root / '.git/temper/check-results').read_bytes(), results)
                verdict = 'refused' if errors else 'passed'
                before = good_log.rsplit('pre-pr: ' + verdict + ' (', 1)[1].splitlines()
                after = bad_log.rsplit('pre-pr: ' + verdict + ' (', 1)[1].splitlines()
                self.assertEqual(after[:1] + after[2:], before[:1] + before[2:])
                self.assertTrue(after[1].startswith(f'failures: {len(errors)} new, 0 flakes, 0 inherited; '
                                                   'not written to target/temper/pre-pr-failures.json: '), after[1])
                self.assertIn('directory', after[1].lower())
                path.rmdir()

    def test_failure_file_write_exceptions_are_bounded(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        error = PRE_PR.failure('format', 'fault')
        message = 'x' + 'é' * 300 + '界' + '\nnot the first line'
        for cause in (OSError(message), ValueError(message), UnicodeError(message)):
            with self.subTest(write=cause), mock.patch.object(Path, 'write_bytes', side_effect=cause):
                result, log, _, _ = self.deliver(one_process_answer=(100, [error], None))
                self.assertEqual(result, 1, log)
                summary = log.rsplit('pre-pr: refused (', 1)[1].splitlines()
                self.assertEqual(summary[1], 'failures: 1 new, 0 flakes, 0 inherited; '
                                 'not written to target/temper/pre-pr-failures.json: x' + 'é' * 255)
                self.assertIn('format\nfault\n', '\n'.join(summary) + '\n')

    def test_failure_file_is_replaced_by_a_pass_and_a_cache_hit(self):
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        error = PRE_PR.failure('format', 'fault')
        result, log, _, _ = self.deliver(one_process_answer=(100, [error], None))
        self.assertEqual(result, 1, log)
        path = self.root / 'target/temper/pre-pr-failures.json'
        refused = json.loads(path.read_text())
        self.assertEqual(refused['verdict'], 'refused')
        replacements = []
        replace = Path.replace

        def replacing(source, target):
            if source.name == 'pre-pr-failures.json.tmp':
                self.assertEqual(json.loads(path.read_text()), refused)
                replacements.append(json.loads(source.read_text()))
            return replace(source, target)

        with mock.patch.object(Path, 'replace', replacing):
            result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        passed = json.loads(path.read_text())
        tree, base, _, _, _ = (self.root / '.git/pre-pr-passed').read_text().split()
        self.assertEqual(passed, {'form': 'pre-pr-failures 1', 'tree': tree, 'base': base,
                                 'cached': False, 'verdict': 'passed',
                                 'counts': {'new': 0, 'flake': 0, 'inherited': 0}, 'failures': []})
        self.assertEqual(replacements, [passed])
        self.assertFalse(path.with_name('pre-pr-failures.json.tmp').exists())
        # Removing the prior file proves a hit writes, rather than just reusing it.
        path.unlink()
        result, log, _, code = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertFalse(any(code))
        self.assertEqual(json.loads(path.read_text()), dict(passed, cached=True))
        lines = log.splitlines()
        hit = next(index for index, line in enumerate(lines) if line.startswith('pre-pr: passed before ('))
        self.assertEqual(lines[hit + 1], 'failures: a cached pass, none observed; in target/temper/pre-pr-failures.json')
        path.unlink()
        path.mkdir()
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        self.assertIn('failures: a cached pass, none observed; not written to target/temper/pre-pr-failures.json: ', log)

    def test_selected_test_binaries_build_between_lints_and_tests(self):
        inventory = Path(__file__).resolve().parents[1] / 'tiers/packages.toml'
        self.write('tools/tiers/packages.toml', inventory.read_text())
        self.write('.config/gate/always.txt', 'smith\n')
        for path in ('smith/src/lib.rs', 'guide.md'):
            with self.subTest(path=path):
                self.pin()
                self.write(path, '# changed\n')
                events = []
                result, log, calls, _ = self.deliver(('api', 'smith', 'smith-fake-claude'), events=events)
                build = ['cargo', 'build', '-p', 'smith-fake-claude', '--bin', 'fake-claude']
                self.assertEqual(result, 0, log)
                self.assertEqual(events, (['clippy'] if path.endswith('.rs') else []) +
                                 [build, 'one-process', 'one-process'])
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
        self.assertEqual(events, ['clippy', 'one-process', 'one-process'])
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
                                 'one-process', 'one-process'])

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
        tree, base, digest, ledgers, _ = cache.read_text().split()
        self.assertEqual(ledgers, '-')
        self.assertEqual(digest, hashlib.sha256(content).hexdigest())
        hit = f'pre-pr: passed before (tree {tree}, base {base}, reds {digest}, ledgers -)'
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

    def main_ledger(self, profile):
        content = json.dumps(dict(ledger=2, profile=profile, binaries={}, budgets={}, moves=[])).encode()
        path = self.root / '.git/main-ledgers' / (profile + '.json')
        path.parent.mkdir(exist_ok=True)
        path.write_bytes(content)
        self.env['TIERS_LEDGER'] = '.git/main-ledgers'
        return path, content

    def budget_run(self, taken, exits=None, summary=(1, None), tests=(), starved=False):
        """Run main in its repository, faking only the one-process and summary commands."""
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.write('client/src/lib.rs', 'pub fn changed() {}\n')
        exits = {} if exits is None else exits
        directories, copies = {}, {}
        self.budget_commands, self.budget_again = [], []

        def fake(arguments, cwd, env, output, stdout):
            self.assertEqual(cwd, self.root)
            self.assertNotIn('TIERS_LEDGER', env)
            self.assertNotIn('GIT_INDEX_FILE', env)
            self.budget_commands.append(arguments)
            if arguments[3] == 'tools/tiers/cpu-summary':
                profile = 'default' if Path(arguments[4]).name == 'tests' else 'fuzzy'
                expected = ['python3', '-I', '-B', 'tools/tiers/cpu-summary', str(directories[profile])]
                if (self.root / 'tools/tiers/budgets.toml').exists():
                    expected += ['--budgets', 'tools/tiers/budgets.toml']
                self.assertEqual(arguments, expected + ['--ledger', str(copies[profile]), '--gate', '--json'])
                self.assertEqual(copies[profile].read_bytes(), taken[profile])
                self.assertFalse(stdout)
                code, text = summary
                output(text.encode(), PRE_PR.Tail())
                return code, PRE_PR.tail(text)
            self.assertTrue(stdout)
            self.assertIsNone(output)
            directory = Path(arguments[arguments.index('--out') + 1])
            self.assertFalse(directory.exists())
            directory.mkdir()
            if '--again' in arguments:
                self.assertNotIn('--ledger', arguments)
                self.assertNotIn('--gate', arguments)
                self.budget_again.append(arguments)
                self.assertEqual(arguments[-2:], ['api::rules', 'replay'])
                (directory / 'results.json').write_text(json.dumps({'tests': [
                    dict(binary_id='api::rules', test_name='replay', outcome='passed')]}))
                return 0, 'passed again\n'
            profile = 'default' if directory.name == 'tests' else 'fuzzy'
            directories[profile] = directory
            expected = ['python3', '-I', '-B', 'tools/tiers/one-process', '--out', str(directory)]
            if profile in taken:
                self.assertIn('--ledger', arguments)
                copy = Path(arguments[arguments.index('--ledger') + 1])
                copies[profile] = copy
                self.assertEqual(copy.name, profile + '-ledger.json')
                self.assertNotEqual(copy, self.root / '.git/main-ledgers' / (profile + '.json'))
                self.assertEqual(copy.parent.stat().st_mode & 0o777, 0o700)
                self.assertEqual(copy.read_bytes(), taken[profile])
                expected += ['--ledger', str(copy), '--gate']
            self.assertEqual(arguments, expected + [profile, '-p', 'api', '-p', 'client', '--no-tests=warn'])
            selected = tests if profile in exits else []
            (directory / 'run.json').write_text(json.dumps({'starved': starved}))
            (directory / 'results.json').write_text(json.dumps({'tests': selected, 'unobserved': []}))
            document = PRE_PR.ET.Element('testsuites')
            for test in selected:
                suite = PRE_PR.ET.SubElement(document, 'testsuite', name=test['binary_id'])
                case = PRE_PR.ET.SubElement(suite, 'testcase', name=test['test_name'])
                if test['outcome'] in ('failed', 'cancelled'):
                    PRE_PR.ET.SubElement(case, 'failure').text = 'test failure output'
            (directory / 'junit.xml').write_bytes(PRE_PR.ET.tostring(document))
            return exits.get(profile, 0), 'one-process output\n'

        return self.deliver(('api', 'client'), one_process_answer=None, process_answer=fake)

    def test_unset_or_empty_ledgers_warn_and_keep_commands(self):
        for setting in (None, ''):
            with self.subTest(setting=setting):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                if setting is not None:
                    self.env['TIERS_LEDGER'] = setting
                result, log, _, _ = self.budget_run({})
                self.assertEqual(result, 0, log)
                self.assertEqual(log.count('pre-pr: budgets:'), 1)
                self.assertIn('pre-pr: budgets: warnings only (TIERS_LEDGER unset)\n', log)
                self.assertEqual(len(self.budget_commands), 2)
                self.assertEqual((self.root / '.git/pre-pr-passed').read_text().split()[3], '-')

    def test_ledgers_read_each_profile_from_root(self):
        for bad_profile in ('default', 'fuzzy'):
            for state in ('missing', 'directory', 'refused', 'other', 'taken'):
                with self.subTest(profile=bad_profile, state=state):
                    self.setUp()
                    paths, taken = {}, {}
                    for profile in ('default', 'fuzzy'):
                        paths[profile], taken[profile] = self.main_ledger(profile)
                    path = paths[bad_profile]
                    if state == 'missing':
                        path.unlink()
                        why = f'no {bad_profile}.json in TIERS_LEDGER'
                    elif state == 'directory':
                        path.unlink()
                        path.mkdir()
                        why = f'{bad_profile}.json unreadable: '
                    elif state == 'refused':
                        path.write_text('{}')
                        why = f'{bad_profile}.json refused: invalid ledger keys for ledger'
                    elif state == 'other':
                        other = 'fuzzy' if bad_profile == 'default' else 'default'
                        path.write_bytes(taken[other])
                        why = f'{bad_profile}.json holds the {other} ledger'
                    if state != 'taken':
                        taken.pop(bad_profile)
                    result, log, _, _ = self.budget_run(taken)
                    self.assertEqual(result, 0, log)
                    self.assertEqual(log.count('pre-pr: budgets:'), 2)
                    self.assertLess(log.index('pre-pr: budgets: default'), log.index('pre-pr: budgets: fuzzy'))
                    if state != 'taken':
                        self.assertIn(f'pre-pr: budgets: {bad_profile} warnings only ({why}', log)
                    for profile in taken:
                        self.assertIn(f"pre-pr: budgets: {profile} judged with main's ledger "
                                      f'(TIERS_LEDGER/{profile}.json)\n', log)
                    self.assertEqual(len(self.budget_commands), 2)
                    self.assertFalse(any('--again' in args for args in self.budget_commands))

    def test_ledger_loading_and_copy_errors_are_bounded_warnings(self):
        for cause in ('loader', 'copy', 'copy missing'):
            with self.subTest(cause=cause):
                self.setUp()
                self.main_ledger('default')
                self.main_ledger('fuzzy')
                message = 'a' + 'é' * 400 + '\nsecond line'
                load = PRE_PR.runpy.run_path
                write = Path.write_bytes

                def load_cpu(path):
                    if path.endswith('/cpu.py'):
                        raise RuntimeError(message)
                    return load(path)

                def write_copy(path, content):
                    if path.name.endswith('-ledger.json'):
                        raise (FileNotFoundError if cause == 'copy missing' else OSError)(message)
                    return write(path, content)

                patch = (mock.patch.object(PRE_PR.runpy, 'run_path', side_effect=load_cpu) if cause == 'loader'
                         else mock.patch.object(Path, 'write_bytes', write_copy))
                with patch:
                    result, log, _, _ = self.budget_run({})
                self.assertEqual(result, 0, log)
                kind = 'refused' if cause == 'loader' else 'unreadable'
                for profile in ('default', 'fuzzy'):
                    line = f'pre-pr: budgets: {profile} warnings only ({profile}.json {kind}: a' + 'é' * 255 + ')\n'
                    self.assertIn(line, log)
                self.assertNotIn('second line', log)

    def test_gate_verdict_names_and_invalid_summaries(self):
        layer = 'layer over budget: temper default domain: 12.00 CPU s, budget 10.0 s, at main 9.00 CPU s'
        reference = 'test over its reference: temper-engine::domain forge::opens: 0.300 CPU s alone, limit 0.200 s'
        entries = [layer + '\nbinary growth details', reference, 'another over line',
                   'layer over budget: no separator', 'test over its reference: no separator']
        names = ['budget temper default domain', 'reference temper-engine::domain forge::opens', 'budget PROFILE',
                 'budget no separator', 'reference no separator']
        rows = [(1, json.dumps({'over_budget': entries}), names)]
        rows += [(code, json.dumps({'over_budget': [layer]}), None) for code in (0, 2)]
        rows += [(1, text, None) for text in ('{"over_budget": []}', '{}', '[]', 'null',
                                            '{"over_budget": [1]}', '{"over_budget": "line"}',
                                            '{"over_budget": ["x"]}\n{}', 'not JSON')]
        for profile in ('default', 'fuzzy'):
            for code, text, expected in rows:
                with self.subTest(profile=profile, code=code, text=text):
                    self.setUp()
                    _, content = self.main_ledger(profile)
                    self.goal_file('reds-wait', b'temper-reds-wait 1 1\n')
                    self.goal_file('reds', b'temper-reds 1\n- test temper-engine::domain forge::opens\n')
                    self.goal_file('rerun', b'temper-rerun 1 4\n')
                    if profile == 'default':
                        self.write('tools/tiers/budgets.toml', '# present\n')
                    result, log, _, _ = self.budget_run({profile: content}, {profile: 1}, (code, text))
                    self.assertEqual(result, 1, log)
                    self.assertFalse((self.root / '.git/pre-pr-passed').exists())
                    self.assertFalse(self.budget_again)
                    failures = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                    wanted = ([name.replace('PROFILE', profile) for name in expected] if expected else
                              ['check one-process' + ('' if profile == 'default' else ' fuzzy')])
                    self.assertEqual(failures['failures'], [dict(name=name, key=None, **{'class': 'new'}) for name in wanted])
                    for name in wanted:
                        self.assertIn('\n' + name + '\n', log)
                    for entry in entries if expected else [text]:
                        self.assertIn(entry, log)
                    self.assertIn(' refused\n', (self.root / '.git/temper/check-results').read_text())
                    self.assertEqual(len(self.budget_commands), 3)
        self.setUp()
        result, log, _, _ = self.budget_run({}, {'default': 1, 'fuzzy': 1})
        self.assertEqual(result, 1, log)
        self.assertIn('\ncheck one-process\nexit 1\n', log)
        self.assertIn('\ncheck one-process fuzzy\nexit 1\n', log)
        self.assertEqual(len(self.budget_commands), 2, 'no summary without --gate')

    def test_gate_failures_keep_junit_judgement_and_replay(self):
        _, content = self.main_ledger('default')
        self.goal_file('reds-wait', b'temper-reds-wait 1 1\n')
        self.goal_file('reds', b'temper-reds 1\n- test api::rules red\n- test api::rules passed\n')
        self.goal_file('rerun', b'temper-rerun 1 4\n')
        tests = [dict(binary_id='api::rules', test_name=name, outcome=outcome) for name, outcome in
                 (('red', 'failed'), ('replay', 'failed'), ('passed', 'passed'), ('stop', 'cancelled'))]
        # Longer than run()'s tail: entries and the first entry's details must be read whole.
        entries = ['layer over budget: temper default domain: over\n' + 'growth\n' * 300,
                   'test over its reference: api::rules red: over']
        result, log, _, _ = self.budget_run({'default': content}, {'default': 1},
                                          (1, json.dumps({'over_budget': entries})), tests, starved=True)
        self.assertEqual(result, 1, log)
        self.assertEqual(len(self.budget_again), 1)
        self.assertIn('inherited: test api::rules red (red, temper/reds)', log)
        self.assertIn('flake: test api::rules replay', log)
        self.assertIn('pre-pr: tests starved: 1 starved or cancelled', log)
        failures = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(failures['counts'], dict(new=2, flake=1, inherited=1))
        self.assertEqual(failures['failures'][:2], [dict(name=name, key=None, **{'class': 'new'}) for name in
                                                 ('budget temper default domain', 'reference api::rules red')])
        check = (self.root / '.git/temper/check-results').read_text()
        self.assertIn(' refused\n', check)
        self.assertIn('passed test api::rules passed\n', check)
        self.assertIn('failed test api::rules red\n', check)
        self.assertIn('flake test api::rules replay\n', check)

    def test_ledger_bytes_key_the_cache(self):
        paths, taken = {}, {}
        for profile in ('default', 'fuzzy'):
            paths[profile], taken[profile] = self.main_ledger(profile)
        cache = self.root / '.git/pre-pr-passed'
        for action in ('initial', 'change default', 'change fuzzy', 'remove default', 'remove fuzzy',
                       'add default', 'add fuzzy'):
            with self.subTest(action=action):
                if action != 'initial':
                    operation, profile = action.split()
                    if operation == 'remove':
                        paths[profile].unlink()
                        taken.pop(profile)
                    else:
                        taken[profile] = taken.get(profile, json.dumps(dict(
                            ledger=2, profile=profile, binaries={}, budgets={}, moves=[])).encode()) + b'\n'
                        paths[profile].write_bytes(taken[profile])
                result, log, _, code = self.budget_run(taken)
                self.assertEqual(result, 0, log)
                self.assertTrue(all(code), 'changed ledgers must judge again')
                digests = ''.join(f'{profile} {hashlib.sha256(taken[profile]).hexdigest() if profile in taken else "-"}\n'
                                  for profile in ('default', 'fuzzy'))
                expected = hashlib.sha256(digests.encode()).hexdigest() if taken else '-'
                tree, base, reds, digest, _ = cache.read_text().split()
                self.assertEqual(digest, expected)
                self.assertEqual(reds, '-')
                result, log, _, code = self.budget_run(taken)
                self.assertEqual(result, 0, log)
                self.assertFalse(any(code))
                self.assertIn(f'pre-pr: passed before (tree {tree}, base {base}, reds -, ledgers {digest})', log)
                self.assertFalse(self.budget_commands)
        for fields in (2, 3, 4):
            with self.subTest(old_fields=fields):
                cache.write_text(' '.join(cache.read_text().split()[:fields]) + '\n')
                result, log, _, code = self.budget_run(taken)
                self.assertEqual(result, 0, log)
                self.assertTrue(all(code), 'old records match nothing')

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
                    if directory.name == 'fuzzy':
                        directory.mkdir()
                        (directory / 'run.json').write_text('{"starved": false}')
                        (directory / 'results.json').write_text('{"tests": [], "unobserved": []}')
                        return 0, 'fuzzy run\n'
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

    def ledger_run(self, tests, again=None, runs=None, changed=True,
                   failure_output='under load\n\nfirst log tail', **kwargs):
        """Fake only the one-process boundary, including fresh again directories."""
        if changed:
            self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.again_calls = []
        self.process_calls = []
        self.run_directories = {}
        self.initial_run = None
        again = {} if again is None else again
        runs = {} if runs is None else runs

        def fake(arguments, cwd, env, output, stdout):
            self.assertEqual(cwd, self.root)
            self.assertTrue(stdout)
            self.assertIsNone(output)
            self.assertNotIn('GIT_INDEX_FILE', env)
            directory = Path(arguments[arguments.index('--out') + 1])
            self.assertFalse(directory.exists())
            self.process_calls.append(arguments)
            if '--again' in arguments:
                self.again_calls.append(arguments)
                original_run = Path(arguments[5])
                self.assertIn(original_run, self.run_directories.values())
                self.assertEqual(arguments[:8], ['python3', '-I', '-B', 'tools/tiers/one-process',
                                                 '--again', str(original_run), '--out', str(directory)])
                self.assertTrue((original_run / 'results.json').exists())
                self.assertEqual(directory.parent, original_run.parent)
                profile = 'default' if original_run.name == 'tests' else 'fuzzy'
                binary, *names = arguments[8:]
                original = next(t for t in tests if t['binary_id'] == binary and t['test_name'] == names[-1]
                                and t.get('profile', 'default') == profile)
                self.assertEqual(names, original.get('before', []) + [original['test_name']])
                answer = again.get((binary, names[-1]), {})
                if answer.get('missing'):
                    return 0, 'no directory\n'
                directory.mkdir()
                content = answer.get('content', json.dumps({'tests': [dict(original, outcome=answer.get('outcome', 'passed'))]}))
                (directory / 'results.json').write_text(content)
                return answer.get('code', 0), 'again output, not the failure log\n'
            profile = arguments[6]
            self.assertIn(profile, ('default', 'fuzzy'))
            self.assertEqual(directory.name, 'tests' if profile == 'default' else 'fuzzy')
            self.assertNotIn(profile, self.run_directories)
            self.run_directories[profile] = directory
            if profile == 'default':
                self.initial_run = directory
            selected = [test for test in tests if test.get('profile', 'default') == profile]
            answer = runs.get(profile, {})
            directory.mkdir()
            (directory / 'run.json').write_text(json.dumps({'starved': answer.get('starved', False)}))
            (directory / 'results.json').write_text(json.dumps({'tests': selected,
                                                              'unobserved': answer.get('unobserved', [])}))
            document = PRE_PR.ET.Element('testsuites')
            for test in selected:
                suite = PRE_PR.ET.SubElement(document, 'testsuite', name=test['binary_id'])
                case = PRE_PR.ET.SubElement(suite, 'testcase', name=test['test_name'])
                if test['outcome'] in ('failed', 'crashed', 'cancelled'):
                    PRE_PR.ET.SubElement(case, 'failure').text = failure_output
            (directory / 'junit.xml').write_bytes(PRE_PR.ET.tostring(document))
            for name, content in answer.get('files', {}).items():
                (directory / name).write_text(content)
            return answer.get('code', 100 if any(t['outcome'] in ('failed', 'crashed') for t in selected) else 0), \
                answer.get('output', 'first run\n')

        return self.deliver(one_process_answer=None, process_answer=fake, **kwargs)

    def test_boundary_selection(self):
        rows = [
            ('docs', 'notes.md', 'B\n', ('A', 'B'), [], ['B']),
            ('code', 'a/src/lib.rs', 'B\n', ('A', 'B'), ['A'], ['A', 'B']),
            ('implicit', 'notes.md', '', ('A', 'fabrig-boundary'), [], ['fabrig-boundary']),
            ('missing', 'notes.md', 'missing\nB\n', ('A', 'B'), [], ['B']),
            ('prefixed', 'a/src/lib.rs', 'B a/\n', ('A', 'B'), ['A'], ['A', 'B']),
            ('neither', None, 'B\n', ('A', 'B'), [], []),
            ('no base', 'notes.md', 'B\n', ('A', 'B'), [], []),
            ('empty', 'notes.md', '', ('A',), [], []),
        ]
        for row, path, always, packages, lints, tests in rows:
            with self.subTest(row=row):
                self.write('.config/gate/always.txt', always)
                self.write('row.txt', row)
                self.pin()
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                if path:
                    self.write(path, f'# {row}\n')
                if row == 'no base':
                    self.command('git', 'update-ref', '-d', 'refs/temper/landing')
                result, log, calls, checks = self.ledger_run([], changed=False, packages=packages,
                                                           directories={'A': 'a', 'B': 'b'})
                self.assertEqual(result, 0, log)
                self.assertEqual(checks, (row in ('code', 'prefixed', 'no base'), bool(lints), bool(tests)))
                self.assertIn('lints: ' + ' '.join(lints) + '\n', log)
                self.assertIn('tests: ' + ' '.join(tests) + '\n', log)
                expected = []
                for profile in ('default', 'fuzzy') if tests else ():
                    directory = self.run_directories[profile]
                    expected.append(['python3', '-I', '-B', 'tools/tiers/one-process', '--out', str(directory),
                                     profile] + PRE_PR.package_args(tests) + ['--no-tests=warn'])
                self.assertEqual(self.process_calls, expected)
                selected = [args for args, _ in calls if 'tools/tiers/one-process' in args
                            or 'tools/citations/citations.py' in args]
                if path and path.endswith('.md') and row != 'no base':
                    self.assertEqual(selected[:-1], expected)
                    self.assertIn('tools/citations/citations.py', selected[-1])
                    self.assertIn('pre-pr: docs only: citations and design size limits', log)
                    self.assertFalse(any('scripts/old_forms.py' in args for args, _ in calls))
                else:
                    self.assertEqual(selected, expected)
                if row not in ('no base', 'neither'):
                    boundary = ['B'] if always in ('B\n', 'missing\nB\n') else \
                        ['fabrig-boundary'] if row == 'implicit' else []
                    self.assertIn('pre-pr: boundary tests: ' + (' '.join(boundary) or 'none'), log)
                else:
                    self.assertNotIn('pre-pr: boundary tests:', log)
                if row == 'missing':
                    self.assertIn('pre-pr: boundary package missing: not a workspace member, skipped', log)
                if lints:
                    self.assertIn('clippy, focused and fuzzy tests (code input', log)

    def test_fuzzy_follows_every_focused_exit_and_judges_its_own_failures(self):
        test = dict(binary_id='api::rules', test_name='rule', outcome='failed')
        fuzzy = dict(test, test_name='fuzzy::rule', profile='fuzzy')
        build = 'one-process: nextest listing exited with 101\n'
        for row, focused, tests, reds, statuses in (
                ('passed', {}, [], '', ('passed', 'passed')),
                ('failed', {}, [test], '', ('failed', 'passed')),
                ('build', {'code': 2, 'output': build}, [], '', ('failed', 'passed')),
                ('check', {'code': 2}, [], '', ('failed', 'passed')),
                ('focused inherited', {}, [test, fuzzy], '- test api::rules rule\n', ('inherited', 'failed')),
                ('fuzzy inherited', {}, [test, fuzzy], '- test api::rules fuzzy::rule\n', ('failed', 'inherited'))):
            with self.subTest(row=row):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                self.goal_file('reds', ('temper-reds 1\n' + reds).encode())
                result, log, _, _ = self.ledger_run(tests, runs={'default': focused})
                self.assertEqual(result, int('failed' in statuses), log)
                self.assertEqual([args[6:] for args in self.process_calls],
                                 [[profile, '-p', 'api', '--no-tests=warn'] for profile in ('default', 'fuzzy')])
                self.assertLess(log.index('pre-pr: tests …'), log.index('pre-pr: fuzzy tests …'))
                for name, status in zip(('tests', 'fuzzy tests'), statuses):
                    self.assertIn(f'pre-pr: {name}: {status},', log)
                    self.assertEqual(log.count(f'pre-pr: {name}: '), 1, log)

    def test_fuzzy_reads_failures_starvation_and_goal_results(self):
        test = dict(binary_id='api::rules', test_name='fuzzy::rule', outcome='failed', profile='fuzzy')
        warning = ('pre-pr: fuzzy tests starved: 1 starved or cancelled, 2 unobserved, '
                   'a warning (skein/docs/foundation/testing-strategy.md, 8.1)')
        rows = [
            ('failure', [test], {}, '', 1, 'test api::rules fuzzy::rule'),
            ('inherited', [test], {}, '- test api::rules fuzzy::rule\n', 0,
             'inherited: test api::rules fuzzy::rule (red, temper/reds)'),
            ('different binary', [test], {}, '- test api::other fuzzy::rule\n', 1, 'test api::rules fuzzy::rule'),
            ('build', [], {'code': 2, 'output': 'one-process: nextest listing exited with 101\n'}, '', 1, 'build fuzzy'),
            ('check', [], {'code': 2}, '', 1, 'check one-process fuzzy'),
            ('empty 100', [], {'code': 100}, '', 1, 'check one-process fuzzy'),
            ('malformed', [], {'files': {'run.json': 'bad json'}}, '', 1, 'check one-process fuzzy'),
            ('starved', [dict(test, outcome='cancelled')], {'starved': True, 'unobserved': [{}, {}]}, '', 0, warning),
            ('passed', [dict(test, outcome='passed')], {}, '- test api::rules fuzzy::rule\n', 0, 'failures: 0 new'),
        ]
        self.goal_file('reds-wait', b'temper-reds-wait 1 1\n')
        for row, tests, answer, reds, exit_code, named in rows:
            with self.subTest(row=row):
                (self.root / '.git/pre-pr-passed').unlink(missing_ok=True)
                self.goal_file('reds', ('temper-reds 1\n' + reds).encode())
                focused = dict(test, profile='default', test_name='stop', outcome='cancelled')
                result, log, _, _ = self.ledger_run([focused] + tests, runs={'fuzzy': answer})
                self.assertEqual(result, exit_code, log)
                summary = log.rsplit('pre-pr: refused (' if result else 'pre-pr: passed (', 1)[1]
                self.assertIn(named, summary)
                status = 'inherited' if row == 'inherited' else 'failed' if exit_code else 'passed'
                self.assertIn(f'pre-pr: fuzzy tests: {status},', log)
                if row == 'starved':
                    self.assertEqual(log.count(warning), 2, log)
                    self.assertLess(summary.index('pre-pr: tests starved:'), summary.index(warning))
                results = (self.root / '.git/temper/check-results').read_text().splitlines()[2:]
                expected = ['failed test api::rules fuzzy::rule'] if row == 'inherited' else \
                    ['passed test api::rules fuzzy::rule'] if row == 'passed' else []
                self.assertEqual(results, expected)

    def test_each_profiles_failures_run_again_under_its_own_bound(self):
        self.goal_file('rerun', b'temper-rerun 1 1\n')
        focused = dict(binary_id='api::rules', test_name='rule', outcome='failed', before=['setup'])
        fuzzy = dict(focused, test_name='fuzzy::rule', profile='fuzzy')
        result, log, _, _ = self.ledger_run([focused, fuzzy])
        self.assertEqual(result, 0, log)
        self.assertEqual([('--again' in args, Path(args[5]).name) for args in self.process_calls],
                         [(False, 'tests'), (True, 'tests'), (False, 'fuzzy'), (True, 'fuzzy')])
        self.assertEqual([args[8:] for args in self.again_calls],
                         [['api::rules', 'setup', 'rule'], ['api::rules', 'setup', 'fuzzy::rule']])
        self.assertEqual(len({args[7] for args in self.again_calls}), 2)
        for name in ('tests', 'fuzzy tests'):
            self.assertIn(f'pre-pr: {name}: inherited,', log)

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
        tree, base, _, _, _ = (self.root / '.git/pre-pr-passed').read_text().split()
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
                result, log, calls, code = self.deliver(('api', 'fabrig-boundary'))
                self.assertEqual(result, 0, log)
                self.assertEqual(code, (False, False, True))
                self.assertFalse(any('scripts/old_forms.py' in args for args, _ in calls))
                self.assertTrue(any('--base' in args and '--head' in args for args, _ in calls))
                self.assertIn('docs only: citations', log)

    def test_prose_only_status_uses_text_checks_without_a_hold(self):
        path = 'temper/docs/design/example.status.md'
        self.write(path, '# Status\n')
        self.pin()
        self.write(path, '# Status\nImplementation complete.\n')
        self.env['TIERS_HOLD'] = 'must-not-run'

        def hold(*args, **kwargs):
            self.fail('a prose-only delivery took a hold')

        with mock.patch.object(PRE_PR, 'outward', wraps=PRE_PR.outward) as outward, \
                mock.patch.object(PRE_PR, 'baselines', side_effect=AssertionError('counted baseline ran')):
            result, log, calls, checks = self.deliver(('api', 'fabrig-boundary'), hold=hold,
                                                    metadata_forbidden=True)
        self.assertEqual(result, 0, log)
        self.assertEqual(checks, (False, False, False))
        self.assertEqual([args for args, _ in calls], [
            ['python3', '-I', '-B', 'tools/citations/citations.py', '--base', mock.ANY, '--head', mock.ANY],
            ['python3', '-I', '-B', 'tools/outward/outward.py']])
        self.assertEqual(outward.call_count, 1)
        self.assertNotEqual(outward.call_args.args[1]['GIT_INDEX_FILE'], str(self.root / '.git/index'))
        self.assertIn(f'pre-pr: prose only: outward names, citations and design size limits (docs input {path!r})', log)
        self.assertIn('pre-pr: boundary tests: none (prose only)', log)
        self.assertIn('lints: \ntests: \n', log)

    def test_prose_only_outward_finding_is_a_test_and_inherits(self):
        entries = (self.root / 'tools/outward/names.txt').read_text().splitlines()
        name = next(line[4:] for line in entries if line and not line.startswith('#'))
        path = 'temper/docs/example.md'
        self.write(path, f'# Example\n{name}\n')
        for red in (False, True):
            with self.subTest(red=red):
                if red:
                    self.write('.git/temper/reds',
                               'temper-reds 1\n- test fabrig-boundary::outward no_file_names_the_lab\n')
                with mock.patch.object(PRE_PR, 'failure', wraps=PRE_PR.failure) as failure:
                    result, log, calls, checks = self.deliver(metadata_forbidden=True)
                self.assertEqual(result, 0 if red else 1, log)
                self.assertEqual(checks, (False, False, False))
                failure.assert_any_call(
                    'test fabrig-boundary::outward no_file_names_the_lab', mock.ANY,
                    ('fabrig-boundary::outward', 'no_file_names_the_lab'))
                if red:
                    self.assertIn('inherited: test fabrig-boundary::outward no_file_names_the_lab', log)
                    self.assertIn('pre-pr: outward: inherited,', log)
                else:
                    self.assertIn(f'{path}:2: {name}', log)
                    self.assertIn('pre-pr: outward: failed,', log)
                self.assertEqual(sum('tools/outward/outward.py' in args for args, _ in calls), 1)

    def test_prose_only_outward_error_is_a_check(self):
        self.write('tools/outward/names.txt', 'w x entry\n')
        self.pin()
        self.write('temper/docs/example.md', '# Example\n')
        self.write('.git/temper/reds',
                   'temper-reds 1\n- test fabrig-boundary::outward no_file_names_the_lab\n')
        result, log, _, _ = self.deliver(metadata_forbidden=True)
        self.assertEqual(result, 1, log)
        self.assertIn('check outward\n', log)
        self.assertIn('outward: names.txt:1: malformed entry', log)

    def test_registered_markdown_and_other_docs_keep_cargo(self):
        for row in ('glob', 'fixture', 'outside', 'mixed'):
            with self.subTest(row=row):
                self.setUp()
                path = 'temper/docs/example.md'
                if row == 'glob':
                    self.write('.config/gate/read-by-tests.txt', '*/docs/*.md\n')
                elif row == 'fixture':
                    self.write('.config/fixture-readers.toml',
                               '[[fixture]]\npaths = ["temper/docs/*.md"]\nreader = ["api"]\n')
                elif row == 'outside':
                    path = 'README.md'
                if row in ('glob', 'fixture'):
                    self.pin()
                self.write(path, '# Example\n')
                if row == 'mixed':
                    self.write('temper/docs/drawing.svg', '<svg/>\n')
                result, log, calls, checks = self.deliver(('api', 'fabrig-boundary'))
                self.assertEqual(result, 0, log)
                self.assertEqual(checks, (row in ('glob', 'fixture'), False, True))
                self.assertIn('pre-pr: code and docs: format' if row in ('glob', 'fixture')
                              else 'pre-pr: docs only: citations', log)
                self.assertIn('pre-pr: boundary tests: fabrig-boundary', log)
                self.assertFalse(any('tools/outward/outward.py' in args for args, _ in calls))

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
        result, log, calls, code = self.deliver(('api', 'fabrig-boundary'))
        self.assertEqual(result, 0, log)
        self.assertEqual(code, (False, False, True))
        self.assertFalse(any('scripts/old_forms.py' in args for args, _ in calls))

    def fake_figures(self, code, output):
        """Fake check 7 at its process boundary (tools/pre-pr/design.md, 4)."""
        self.write('tools/tiers/figures.py', f'''import json
import os
from pathlib import Path
import sys
assert 'GIT_INDEX_FILE' not in os.environ
assert Path('tools/tiers/budgets.toml').is_file()
Path('target').mkdir(exist_ok=True)
Path('target/figures-argv.json').write_text(json.dumps(sys.argv[1:]))
print({output!r})
raise SystemExit({code})
''')
        self.write('tools/tiers/budgets.toml', '# base figures\n')
        self.write('tools/citations/citations.py', 'print("citations: passed")\n')
        self.pin()

    def test_figures_passes_last_on_delivery_snapshot(self):
        """Check 7 follows every selected check (tools/pre-pr/design.md, 4)."""
        for row in ('budgets alone', 'mixed', 'earlier failure'):
            with self.subTest(row=row):
                self.setUp()
                output = 'figures: accepted snapshot'
                self.fake_figures(0, output)
                base = self.command('git', 'rev-parse', 'HEAD').strip()
                self.write('tools/tiers/budgets.toml', '# delivered figures\n')
                if row != 'budgets alone':
                    self.write('api/src/lib.rs', 'pub fn changed() {}\n')
                    self.write('guide.md', '# Changed\n')
                errors = [PRE_PR.failure('format', 'bad format\n')] if row == 'earlier failure' else []
                result, log, calls, _ = self.deliver(formatting_answer=errors)
                record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                self.assertEqual(result, int(bool(errors)), log)
                self.assertEqual(record['failures'], [dict(name='format', key=None, **{'class': 'new'})]
                                 if errors else [])
                self.assertEqual(calls[-1], (['python3', '-I', '-B', 'tools/tiers/figures.py',
                                             '--base', base, '--head', record['tree']], self.root))
                self.assertEqual(sum('tools/tiers/figures.py' in args for args, _ in calls), 1)
                self.assertNotEqual(record['tree'], self.command('git', 'rev-parse', 'HEAD^{tree}').strip())
                self.assertEqual(self.command('git', 'show', record['tree'] + ':tools/tiers/budgets.toml'),
                                 '# delivered figures\n')
                self.assertIn('pre-pr: figures …\n' + output + '\n', log)
                self.assertRegex(log, r'pre-pr: figures: passed, \d+\.\d{3} s\n')
                self.assertLess(log.index('pre-pr: old-forms hearth:'), log.index('pre-pr: figures …'))
                if row != 'budgets alone':
                    for name in ('clippy', 'tests', 'fuzzy tests', 'citations'):
                        self.assertLess(log.index(f'pre-pr: {name}:'), log.index('pre-pr: figures …'))

    def test_figures_passes_pending_merge_tip(self):
        self.fake_figures(0, 'figures: merged snapshot')
        base = self.command('git', 'rev-parse', 'HEAD').strip()
        self.write('tools/tiers/budgets.toml', '# main figures\n')
        self.command('git', 'commit', '-qam', 'main figures alone')
        merged = self.command('git', 'rev-parse', 'HEAD').strip()
        self.command('git', 'checkout', '-q', '--detach', base)
        self.write('guide.md', '# Branch\n')
        self.command('git', 'commit', '-qam', 'branch')
        self.command('git', 'merge', '--no-ff', '--no-commit', merged)
        result, log, calls, _ = self.deliver()
        self.assertEqual(result, 0, log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        argv = ['--base', base, '--head', record['tree'], '--merged', merged]
        self.assertEqual(json.loads((self.root / 'target/figures-argv.json').read_text()), argv)
        self.assertEqual(calls[-1], (['python3', '-I', '-B', 'tools/tiers/figures.py', *argv], self.root))

    def test_figures_passes_merge_tip_before_head_after_stepped_save_base(self):
        self.fake_figures(0, 'figures: saved merge snapshot')
        start = self.command('git', 'rev-parse', 'HEAD').strip()
        self.write('tools/tiers/budgets.toml', '# main figures\n')
        self.command('git', 'commit', '-qam', 'main figures alone')
        merged = self.command('git', 'rev-parse', 'HEAD').strip()
        self.command('git', 'checkout', '-q', '--detach', start)
        self.write('guide.md', '# Branch\n')
        self.command('git', 'commit', '-qam', 'branch')
        base = self.command('git', 'rev-parse', 'HEAD').strip()
        self.command('git', 'merge', '--no-ff', '-qm', 'Unchecked: work in progress, saved before a question', merged)
        self.command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
        self.command('git', 'commit', '--allow-empty', '-qm', 'after saved merge')
        result, log, calls, _ = self.deliver()
        self.assertEqual(result, 0, log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(record['base'], base)
        argv = ['--base', base, '--head', record['tree'], '--merged', merged]
        self.assertEqual(json.loads((self.root / 'target/figures-argv.json').read_text()), argv)
        self.assertEqual(calls[-1], (['python3', '-I', '-B', 'tools/tiers/figures.py', *argv], self.root))

    def test_figures_merged_tips_are_pending_then_first_parent_order_deduplicated(self):
        self.fake_figures(0, 'figures: several merged tips')
        base = self.command('git', 'rev-parse', 'HEAD').strip()
        tree = self.command('git', 'rev-parse', 'HEAD^{tree}').strip()
        left = self.command('git', 'commit-tree', tree, '-p', base, '-m', 'left').strip()
        right = self.command('git', 'commit-tree', tree, '-p', base, '-m', 'right').strip()
        inner = self.command('git', 'commit-tree', tree, '-p', base, '-m', 'inner').strip()
        nested = self.command('git', 'commit-tree', tree, '-p', left, '-p', inner, '-m', 'nested').strip()
        first = self.command('git', 'commit-tree', tree, '-p', base, '-p', nested, '-m', 'first').strip()
        # Both non-first parents count, but nested's own parents do not.
        head = self.command('git', 'commit-tree', tree, '-p', first, '-p', right, '-p', nested,
                            '-m', 'several parents').strip()
        self.command('git', 'checkout', '-q', '--detach', head)
        self.write('.git/MERGE_HEAD', nested + '\n')
        self.write('tools/tiers/budgets.toml', '# delivered figures\n')
        result, log, calls, _ = self.deliver()
        self.assertEqual(result, 0, log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        argv = ['--base', base, '--head', record['tree'], '--merged', nested, '--merged', right]
        self.assertEqual(json.loads((self.root / 'target/figures-argv.json').read_text()), argv)
        self.assertEqual(calls[-1], (['python3', '-I', '-B', 'tools/tiers/figures.py', *argv], self.root))

    def test_figures_landing_repair_base_and_parent_order(self):
        """A repair keeps its base, selects its own paths and checks figures (design.md, 2–4)."""
        budgets = 'tools/tiers/budgets.toml'
        reasons = ('landed_first', 'landed_second')
        repair = ['api/src/repair.rs']
        change = ['api/src/lib.rs', *repair]
        landed = [*repair, 'client/src/landed.rs', budgets]
        rows = (('B', 'first', 'skipped', repair, 'api'),
                ('B', 'second', 'skipped', change, 'api'),
                ('H', 'second', 'passed', repair, 'api'),
                ('H', 'first', 'failed', landed, 'api client'))
        for pin, base_parent, verdict, own_paths, packages in rows:
            with self.subTest(pin=pin, base_parent=base_parent):
                self.setUp()
                self.command('git', 'branch', '-M', 'main')
                source = Path(__file__).resolve().parents[1] / 'tiers'
                for name in ('figures.py', 'cpu.py'):
                    self.write('tools/tiers/' + name, (source / name).read_text())
                self.write('tools/tiers/layers.toml', 'layers = ["step", "domain", "simulated"]\n')
                self.write(budgets, '[gate.default]\n')
                self.write('Cargo.toml', '[workspace]\nmembers = ["api", "client"]\n')
                self.write('client/Cargo.toml', '[package]\nname = "client"\n')
                self.write('client/src/lib.rs', 'pub fn client() {}\n')
                self.pin()
                fork = self.command('git', 'rev-parse', 'HEAD').strip()

                self.command('git', 'checkout', '-q', '--detach', fork)
                self.write('api/src/lib.rs', 'pub fn change() {}\n')
                self.command('git', 'commit', '-qam', 'change head H')
                old_head = self.command('git', 'rev-parse', 'HEAD').strip()

                self.command('git', 'checkout', '-q', 'main')
                self.write(budgets, '[gate.default]\n' + ''.join(
                    f'\n[[gate.default.reasons]]\nbinary = "client::suite"\n'
                    f'test = "{name}"\ncpu_s = 3\nwhy = "landed test"\n'
                    for name in reasons))
                self.write('client/src/landed.rs', 'pub fn landed() {}\n')
                self.command('git', 'add', '.')
                self.command('git', 'commit', '-qm', 'landing base B')
                landing_base = self.command('git', 'rev-parse', 'HEAD').strip()

                parents = ((landing_base, old_head) if base_parent == 'first' else
                           (old_head, landing_base))
                self.command('git', 'checkout', '-q', '--detach', parents[0])
                self.command('git', 'merge', '--no-ff', '-qm', 'repair merge', parents[1])
                self.write('api/src/repair.rs', 'pub fn repair() {}\n')
                expected_base = landing_base if pin == 'B' else old_head
                self.command('git', 'update-ref', 'refs/temper/landing', expected_base)

                with mock.patch.object(PRE_PR, 'code_inputs', wraps=PRE_PR.code_inputs) as inputs:
                    result, log, calls, _ = self.deliver(packages=('api', 'client'))
                record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                self.assertEqual(record['base'], expected_base, log)
                self.assertEqual(result, int(verdict == 'failed'), log)
                delivery_verdict = 'refused' if verdict == 'failed' else 'passed'
                self.assertIn(f'pre-pr: {delivery_verdict} (base {expected_base}, '
                              'merge base of refs/temper/landing and HEAD)\n', log)
                paths = change if pin == 'B' else landed
                raw_paths = self.command('git', 'diff-tree', '-r', '--no-renames', '--name-only',
                                         expected_base, record['tree']).splitlines()
                self.assertEqual(raw_paths, paths)
                self.assertTrue(inputs.called)
                for call in inputs.call_args_list:
                    self.assertEqual(call.args[0], own_paths)
                self.assertIn(f'\nlints: {packages}\ntests: {packages}\n', log)
                figure_calls = [args for args, _ in calls if 'tools/tiers/figures.py' in args]
                if verdict == 'skipped':
                    self.assertEqual(figure_calls, [])
                    self.assertNotIn('pre-pr: figures', log)
                else:
                    merged = landing_base if base_parent == 'second' else old_head
                    self.assertEqual(figure_calls, [
                        ['python3', '-I', '-B', 'tools/tiers/figures.py', '--base', expected_base,
                         '--head', record['tree'], '--merged', merged]])
                    self.assertRegex(log, rf'pre-pr: figures: {verdict}, \d+\.\d{{3}} s\n')
                    count = len(reasons) if verdict == 'failed' else 0
                    self.assertIn(f'figures: {count} raised or added since {old_head} and {merged}\n', log)
                    if verdict == 'passed':
                        self.assertIn(f'figures: {budgets} unchanged since {landing_base}\n', log)
                if verdict == 'failed':
                    for name in reasons:
                        self.assertIn(f'figure added: default reason client::suite {name}: 3\n', log)
                    self.assertEqual(record['failures'], [dict(name='figures', key=None, **{'class': 'new'})])
                else:
                    self.assertEqual(record['failures'], [])
                    self.assertNotIn('figure added:', log)
                self.assertEqual(self.command('git', 'rev-parse', 'refs/temper/landing').strip(), expected_base)

    def test_figures_refusal_is_not_inherited(self):
        """A figure refusal is the delivery's (tools/pre-pr/design.md, 4)."""
        output = 'figure raised: default temper domain ceiling: 137.5 to 141.5'
        self.fake_figures(1, output)
        self.write('tools/tiers/budgets.toml', '# delivered figures\n')
        self.write('api/src/lib.rs', 'pub fn changed() {}\n')
        self.write('.git/temper/reds', 'temper-reds 1\n- test figures figures\n'
                   '- lint figures figures tools/tiers/budgets.toml\n')
        with mock.patch.object(PRE_PR, 'failure', wraps=PRE_PR.failure) as failure:
            result, log, _, _ = self.deliver()
        failure.assert_called_once_with('figures', output + '\n')
        self.assertEqual(result, 1, log)
        self.assertIn('pre-pr: refused (base ', log)
        self.assertIn('pre-pr: figures …\n' + output + '\n', log)
        self.assertRegex(log, r'pre-pr: figures: failed, \d+\.\d{3} s\n')
        self.assertIn('\nfigures\n' + output + '\n', log)
        record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(record['verdict'], 'refused')
        self.assertEqual(record['counts'], dict(new=1, flake=0, inherited=0))
        self.assertEqual(record['failures'], [dict(name='figures', key=None, **{'class': 'new'})])

    def test_figures_tool_error_is_a_check(self):
        """Other exits fail the check itself (tools/pre-pr/design.md, 4)."""
        for code in (2, 7):
            with self.subTest(code=code):
                self.setUp()
                output = 'figures: cannot read tree'
                self.fake_figures(code, output)
                self.write('tools/tiers/budgets.toml', '# delivered figures\n')
                with mock.patch.object(PRE_PR, 'failure', wraps=PRE_PR.failure) as failure:
                    result, log, _, _ = self.deliver()
                failure.assert_called_once_with('check figures', output + '\n')
                self.assertEqual(result, 1, log)
                self.assertRegex(log, r'pre-pr: figures: failed, \d+\.\d{3} s\n')
                self.assertIn('\ncheck figures\n' + output + '\n', log)
                record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                self.assertEqual(record['failures'], [dict(name='check figures', key=None, **{'class': 'new'})])

    def test_figures_skips_unchanged_budgets_and_no_base(self):
        """Selection needs the changed path and a base (tools/pre-pr/design.md, 4)."""
        for row in ('prose', 'other tools', 'no base'):
            with self.subTest(row=row):
                self.setUp()
                self.fake_figures(1, 'figures must not run')
                if row == 'prose':
                    self.write('temper/docs/example.md', '# Example\n')
                elif row == 'other tools':
                    self.write('tools/other.py', '# Changed\n')
                else:
                    self.write('tools/tiers/budgets.toml', '# delivered figures\n')
                    self.command('git', 'update-ref', '-d', 'refs/temper/landing')
                result, log, calls, _ = self.deliver(metadata_forbidden=row == 'prose')
                self.assertEqual(result, 0, log)
                self.assertFalse(any('tools/tiers/figures.py' in args for args, _ in calls))
                self.assertNotIn('pre-pr: figures', log)

    def journey_checks(self):
        # These cases exercise git and the driver, not the citation checker's inputs.
        self.write('tools/citations/citations.py', 'raise SystemExit(0)\n')
        self.pin()

    def test_journeys_page_and_source_refused_with_and_without_base(self):
        page = 'temper/docs/design/client/web/journeys/x/01.html'
        source = 'temper/x.rs'
        for pinned in (True, False):
            with self.subTest(pinned=pinned):
                self.setUp()
                self.journey_checks()
                self.write(page, 'base page\n')
                self.write(source, 'fn base() {}\n')
                self.pin()
                base = self.command('git', 'rev-parse', 'HEAD').strip()
                self.write(page, 'delivered page\n')
                self.write(source, 'fn delivered() {}\n')
                if pinned:
                    self.command('git', 'commit', '-qam', 'delivery')
                else:
                    self.command('git', 'update-ref', '-d', 'refs/temper/landing')
                    self.command('git', 'add', source)
                    self.write('temper/docs/design/client/web/journeys/x/02.html', 'untracked\n')
                index = (self.root / '.git/index').read_bytes()
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 1, log)
                self.assertEqual((self.root / '.git/index').read_bytes(), index)
                self.assertIn('pre-pr: journeys …\n', log)
                self.assertRegex(log, r'pre-pr: journeys: failed, \d+\.\d{3} s\n')
                self.assertIn('written journey file: ' + page, log)
                self.assertIn('other path: ' + source, log)
                repair_base = base if pinned else 'HEAD'
                self.assertIn("a journey's recording, pages and the files beside them are written on main by "
                              "the journeys' writer alone (AGENTS.md, 4): restore them with git "
                              f"checkout {repair_base} -- PATH..., and compare pages by cargo run -p "
                              "temper-client-journeys in your own tree, or, for a journey whose walk is of version 2, "
                              "by cargo run -p temper-client-walk -- print --out DIR\n", log)
                record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                self.assertEqual(record['counts'], dict(new=1, flake=0, inherited=0), log)
                self.assertEqual(record['failures'], [dict(name='journeys', key=None, **{'class': 'new'})])

    def test_journeys_written_files_alone_pass(self):
        for kind, page, style, script in (('web', '01.html', 'skein-ui.css', 'journey.js'),
                                          ('tui', '123.txt', 'theme.css', 'tui-journey.js')):
            with self.subTest(kind=kind):
                self.setUp()
                self.journey_checks()
                root = f'temper/docs/design/client/{kind}/journeys/'
                paths = [root + 'x/recording', root + 'x/' + page, root + style, root + script]
                for path in paths:
                    self.write(path, 'base\n')
                self.pin()
                for path in paths:
                    self.write(path, 'written\n')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertRegex(log, r'pre-pr: journeys: passed, \d+\.\d{3} s\n')
                # Each kind must be classified as written, not merely ignored.
                self.write('temper/x.rs', 'fn source() {}\n')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 1, log)
                for path in paths:
                    self.assertIn('written journey file: ' + path, log)

    def test_journeys_sources_beside_code_pass(self):
        for kind in ('web', 'tui'):
            with self.subTest(kind=kind):
                self.setUp()
                self.journey_checks()
                root = f'temper/docs/design/client/{kind}/journeys/'
                for path in ('README.md', 'x/README.md', 'x/setup', 'x/walk', 'x/1.html',
                             'x/skein-ui.css', 'x/nested/01.html', 'overlay.css'):
                    self.write(root + path, '' if path == 'overlay.css' else 'source\n')
                self.write('temper/x.rs', 'fn source() {}\n')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertRegex(log, r'pre-pr: journeys: passed, \d+\.\d{3} s\n')

    def test_journeys_merged_pages_pass_until_edited_here(self):
        for pending in (False, True):
            with self.subTest(pending=pending):
                self.setUp()
                self.journey_checks()
                page = 'temper/docs/design/client/web/journeys/x/01.html'
                deleted = 'temper/docs/design/client/web/journeys/x/02.html'
                self.write(page, 'base page\n')
                self.write(deleted, 'deleted by writer\n')
                self.write('temper/x.rs', 'fn base() {}\n')
                self.pin()
                base = self.command('git', 'rev-parse', 'HEAD').strip()
                self.write(page, 'writer page\n')
                (self.root / deleted).unlink()
                self.command('git', 'commit', '-qam', 'writer')
                tip = self.command('git', 'rev-parse', 'HEAD').strip()
                self.command('git', 'checkout', '-q', '--detach', base)
                self.write('temper/x.rs', 'fn delivery() {}\n')
                self.command('git', 'commit', '-qam', 'source')
                self.command('git', 'merge', '--no-ff', '--no-commit', tip)
                if not pending:
                    self.command('git', 'commit', '-qm', 'merge writer')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 0, log)
                self.assertIn('pre-pr: journeys: passed,', log)
                self.write(page, 'edited after merge\n')
                result, log, _, _ = self.deliver()
                self.assertEqual(result, 1, log)
                self.assertIn('written journey file: ' + page, log)
                self.assertNotIn('written journey file: ' + deleted, log)

    def test_journeys_nonempty_overlays_refuse_alone_and_mixed(self):
        for kind in ('web', 'tui'):
            for mixed in (False, True):
                with self.subTest(kind=kind, mixed=mixed):
                    self.setUp()
                    self.journey_checks()
                    overlay = f'temper/docs/design/client/{kind}/journeys/overlay.css'
                    self.write(overlay, '')
                    self.pin()
                    self.write(overlay, ' ')
                    if mixed:
                        self.write('temper/x.rs', 'fn delivery() {}\n')
                    result, log, _, _ = self.deliver()
                    self.assertEqual(result, 1, log)
                    self.assertIn('nonempty overlay: ' + overlay, log)
                    self.assertIn('pre-pr: journeys: failed,', log)
                    record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                    self.assertEqual(record['failures'], [dict(name='journeys', key=None, **{'class': 'new'})])

    def test_journeys_prose_only_runs_last_and_never_inherits(self):
        for nonempty in (False, True):
            with self.subTest(nonempty=nonempty):
                self.setUp()
                self.journey_checks()
                overlay = 'temper/docs/design/client/tui/journeys/overlay.css'
                self.write(overlay, 'x' if nonempty else '')
                self.pin()
                self.write('temper/docs/example.md', '# Prose\n')
                self.write('.git/temper/reds', 'temper-reds 1\n- test journeys journeys\n'
                           '- lint journeys journeys ' + overlay + '\n')
                result, log, calls, code = self.deliver(metadata_forbidden=True)
                self.assertEqual(result, int(nonempty), log)
                self.assertEqual(code, (False, False, False))
                self.assertIn('pre-pr: prose only:', log)
                self.assertIn('pre-pr: journeys …', log)
                self.assertLess(log.index('pre-pr: outward:'), log.index('pre-pr: journeys …'))
                self.assertFalse(any(args[0] == 'cargo' for args, _ in calls))
                record = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
                self.assertEqual(record['counts'], dict(new=int(nonempty), flake=0, inherited=0))
                if nonempty:
                    self.assertIn('nonempty overlay: ' + overlay, log)
                    self.assertEqual(record['failures'], [dict(name='journeys', key=None, **{'class': 'new'})])

    def test_journeys_cache_accounts_for_head_without_base(self):
        self.journey_checks()
        page = 'temper/docs/design/client/web/journeys/x/01.html'
        self.write(page, 'base page\n')
        self.write('temper/x.rs', 'fn base() {}\n')
        self.pin()
        base = self.command('git', 'rev-parse', 'HEAD').strip()
        self.write(page, 'other page\n')
        self.command('git', 'commit', '-qam', 'other head')
        other = self.command('git', 'rev-parse', 'HEAD').strip()
        self.command('git', 'checkout', '-q', '--detach', base)
        self.command('git', 'update-ref', '-d', 'refs/temper/landing')
        self.write('temper/x.rs', 'fn delivered() {}\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        passed = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.command('git', 'checkout', '-q', '--detach', other)
        self.write(page, 'base page\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        refused = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(passed['tree'], refused['tree'])
        self.assertFalse(refused['cached'])
        self.assertIn('written journey file: ' + page, log)

    def test_journeys_cache_accounts_for_pending_merge(self):
        self.journey_checks()
        page = 'temper/docs/design/client/web/journeys/x/01.html'
        self.write(page, 'base page\n')
        self.write('temper/x.rs', 'fn base() {}\n')
        self.pin()
        base = self.command('git', 'rev-parse', 'HEAD').strip()
        self.write(page, 'writer page\n')
        self.command('git', 'commit', '-qam', 'writer')
        tip = self.command('git', 'rev-parse', 'HEAD').strip()
        self.command('git', 'checkout', '-q', '--detach', base)
        self.write('temper/x.rs', 'fn delivered() {}\n')
        self.command('git', 'commit', '-qam', 'source')
        self.command('git', 'merge', '--no-ff', '--no-commit', tip)
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 0, log)
        passed = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.command('git', 'merge', '--quit')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        refused = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(passed['tree'], refused['tree'])
        self.assertEqual(passed['base'], refused['base'])
        self.assertFalse(refused['cached'])
        self.assertIn('written journey file: ' + page, log)

    def test_journeys_written_log_is_bounded_and_counts_remaining(self):
        self.journey_checks()
        root = 'temper/docs/design/client/web/journeys/x/'
        for number in range(21):
            self.write(root + f'{number:02}.html', 'page\n')
        self.write('temper/x.rs', 'fn delivery() {}\n')
        result, log, _, _ = self.deliver()
        self.assertEqual(result, 1, log)
        for number in range(20):
            self.assertIn('written journey file: ' + root + f'{number:02}.html', log)
        self.assertNotIn('written journey file: ' + root + '20.html', log)
        self.assertIn('… 1 more written journey files', log)
        self.assertIn('other path: temper/x.rs', log)

    def test_new_broken_citation(self):
        self.write('skein/docs/guide.md', '# Guide\nSee `missing.md`, section 1.\n')
        result, log, _, code = self.deliver(('api', 'fabrig-boundary'))
        self.assertEqual(result, 1, log)
        self.assertIn('missing.md', log)
        self.assertIn('1 new broken', log)
        self.assertIn('pre-pr: prose only:', log)
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

    def check_base_target(self):
        with tempfile.TemporaryDirectory(prefix='pre-pr-count-env-') as directory:
            records = Path(directory) / 'counts.jsonl'
            script = self.root / 'skein/scripts/old_forms.py'
            self.write('skein/scripts/old_forms.py', f"""import json
import os
from pathlib import Path
with Path({str(records)!r}).open('a') as output:
    output.write(json.dumps([str(Path.cwd()), os.environ.get('CARGO_TARGET_DIR')]) + '\\n')
""" + script.read_text())
            self.write('skein/count.txt', '2')
            self.pin()
            self.write('api/src/lib.rs', 'pub fn changed() {}\n')
            git_dir = Path(self.command('git', 'rev-parse', '--absolute-git-dir').strip())
            target = git_dir / 'temper-pre-pr/target'
            expected = [[str(self.root / 'skein'), self.env.get('CARGO_TARGET_DIR')],
                        [str(git_dir / 'temper-pre-pr/base/skein'), str(target)]]
            result, log, _, _ = self.deliver()
            self.assertEqual(result, 0, log)
            self.assertIn('inherited: old-forms skein old', log)
            self.assertEqual([json.loads(line) for line in records.read_text().splitlines()], expected)
            self.assertTrue(target.is_dir())
            kept = target / 'kept'
            kept.write_text('base artifact\n')

            self.write('skein/count.txt', '3')
            result, log, _, _ = self.deliver()
            self.assertEqual(result, 1, log)
            self.assertIn('old-forms skein old', log)
            self.assertEqual([json.loads(line) for line in records.read_text().splitlines()], expected * 2)
            self.assertEqual(kept.read_text(), 'base artifact\n')

    def test_base_target_overrides_head_target(self):
        self.env['CARGO_TARGET_DIR'] = str(self.root / 'target-head')
        self.check_base_target()

    def test_base_target_when_head_target_unset(self):
        self.assertNotIn('CARGO_TARGET_DIR', self.env)
        self.check_base_target()

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
        result, log, _, code = self.deliver(('api', 'fabrig-boundary'))
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
