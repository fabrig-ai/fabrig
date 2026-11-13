"""Focused fixtures and fake tools for tools/crates/design.md, 4."""
import contextlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


TOOL = Path(__file__).with_name('crates.py').resolve()
SPEC = importlib.util.spec_from_file_location('crates', TOOL)
CRATES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CRATES)


def target(name, kind):
    return {'name': name, 'kind': [kind], 'src_path': f'{name}/src/lib.rs'}


def metadata(root):
    packages = [{'id': name, 'name': name, 'version': '0.1.0',
                 'manifest_path': str(root / name / 'Cargo.toml'),
                 'targets': [target(name, 'lib')]}
                for name in ('a', 'b', 'c', 'd', 'external')]
    packages[1]['targets'] += [target('integration', 'test'), target('speed', 'bench'),
                               target('demo', 'example'), target('command', 'bin')]
    packages[2]['targets'].append(target('integration', 'test'))
    dependencies = {'a': [('external', None)], 'b': [('a', None)],
                    'c': [('b', None), ('d', 'dev')], 'd': [], 'external': []}
    return {'workspace_root': str(root), 'packages': packages,
            'workspace_members': ['a', 'b', 'c', 'd'],
            'resolve': {'nodes': [{'id': name, 'deps': [
                {'pkg': dep, 'dep_kinds': [{'kind': kind, 'target': None}]}
                for dep, kind in deps]} for name, deps in dependencies.items()]}}


def unit(name, duration, rmeta=None, start=0, description='', mode='todo'):
    return {'i': 0, 'name': name, 'version': '0.1.0', 'target': description, 'mode': mode,
            'start': start, 'duration': duration, 'rmeta_time': rmeta,
            'unlocked_units': [], 'unlocked_rmeta_units': [], 'sections': []}


def report(units):
    return ('<!DOCTYPE html><script>const DURATION = 100;\nconst UNIT_DATA = '
            + json.dumps(units, indent=2) + ';\nconst CONCURRENCY_DATA = [];\n'
            'const CPU_USAGE = [];\n</script>')


def libraries():
    return [unit('external', 1, .5), unit('a', 10, 4, .5),
            unit('b', 6, 2, 4.5), unit('c', 3, 1, 6.5), unit('d', 2, 1)]


def tests():
    return libraries() + [unit('a', 4, start=1, description=' lib (test)'),
                          unit('b', 5, start=10.5, description=' test "integration" (test)'),
                          unit('c', 7, start=10.5, description=' lib (test)'),
                          unit('c', 8, start=10.5, description=' test "integration" (test)'),
                          unit('b', 2, start=10.5, description=' bench "speed" (test)'),
                          unit('b', 1, start=10.5, description=' example "demo"'),
                          unit('b', 3, start=10.5, description=' bin "command"')]


class Scratch(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory(prefix='crates-fixture-')
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)
        self.root = self.scratch / 'repository'
        self.root.mkdir()
        self.env = os.environ.copy()
        for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR'):
            self.env.pop(name, None)
        self.command('git', 'init', '-q', '-b', 'main')
        self.command('git', 'config', 'user.name', 'Fixture')
        self.command('git', 'config', 'user.email', 'fixture@example.invalid')
        self.command('git', 'config', 'commit.gpgsign', 'false')
        for name in ('a', 'b', 'c', 'd', 'external'):
            self.write(f'{name}/src/lib.rs', '// library\n')
            self.write(f'{name}/Cargo.toml', f'[package]\nname = "{name}"\n')
        self.data = metadata(self.root)
        self.workspace = CRATES.Workspace(self.data)
        self.out = self.scratch / 'reports'
        self.out.mkdir()
        self.write_reports()
        self.commit('root', '2026-10-20T12:00:00Z')

    def command(self, *arguments, env=None):
        return subprocess.run(arguments, cwd=self.root, env=env or self.env,
                              capture_output=True, text=True, check=True).stdout

    def write(self, path, content):
        destination = self.root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(content)

    def commit(self, subject, date):
        self.command('git', 'add', '.')
        self.command('git', 'commit', '-qm', subject,
                     env=dict(self.env, GIT_COMMITTER_DATE=date, GIT_AUTHOR_DATE=date))
        return self.command('git', 'rev-parse', 'HEAD').strip()

    def write_reports(self):
        (self.out / 'metadata.json').write_text(json.dumps(self.data))
        (self.out / 'libraries.html').write_text(report(libraries()))
        (self.out / 'tests.html').write_text(report(tests()))

    def read_shape(self, shape):
        units = CRATES.read_units(self.out / f'{shape}.html', self.workspace)
        return units, CRATES.waits_for(units, self.workspace)

    def cli(self, *arguments, env=None):
        return subprocess.run([sys.executable, '-I', '-B', str(TOOL), *map(str, arguments)],
                              cwd=self.root, env=env or self.env, capture_output=True, text=True)

    @contextlib.contextmanager
    def inside(self):
        with contextlib.chdir(self.root), mock.patch.dict(os.environ, self.env, clear=True):
            yield

    def history(self):
        self.write('c/src/lib.rs', '// old\n')
        self.old = self.commit('too old', '2026-11-01T12:00:00Z')
        self.write('guide.md', 'boundary\n')
        boundary = self.commit('seven day boundary', '2026-11-05T12:00:00Z')
        self.write('a/src/lib.rs', '// a\n')
        a = self.commit('change a', '2026-11-06T12:00:00Z')
        self.write('b/src/lib.rs', '// b\n')
        b = self.commit('change b', '2026-11-07T12:00:00Z')
        self.command('git', 'checkout', '-qb', 'side')
        self.write('c/src/lib.rs', '// side\n')
        self.side = self.commit('second parent only', '2026-11-08T12:00:00Z')
        self.command('git', 'checkout', '-q', 'main')
        self.write('b/tests/integration.rs', '// test\n')
        integration = self.commit('integration only', '2026-11-09T12:00:00Z')
        self.command('git', 'merge', '--no-ff', '-qm', 'merge side', 'side', env=dict(
            self.env, GIT_COMMITTER_DATE='2026-11-10T12:00:00Z', GIT_AUTHOR_DATE='2026-11-10T12:00:00Z'))
        merge = self.command('git', 'rev-parse', 'HEAD').strip()
        self.write('guide.md', 'outside\n')
        outside = self.commit('no member', '2026-11-11T12:00:00Z')
        self.write('b/examples/demo.rs', '// example\n')
        head = self.commit('example only', '2026-11-12T12:00:00Z')
        self.head = head
        self.expected_history = [boundary, a, b, integration, merge, outside, head]
        record = {'commit': head, 'branch': 'main',
                  'toolchain': {'first_line': 'rustc 1.95.0 (fixture)', 'release': 'release: 1.95.0'},
                  'cpu_count': 8, 'environment': CRATES.ENVIRONMENT, 'builds': {}}
        (self.out / 'build.json').write_text(json.dumps(record))
        with self.inside():
            return CRATES.landings(head)


class Figure(Scratch):
    def test_pattern_chains_and_landing_full_times(self):
        (self.out / 'libraries.html').write_text(report([unit('a', 10, 4), unit('b', 6, 2, 4)]))
        units, waits = self.read_shape('libraries')
        self.assertEqual(CRATES.chain(units, waits, {0, 1}), (16, [0, 1]))
        self.assertEqual(CRATES.chain(units, waits, {0, 1}, True), (10, [0, 1]))
        for path, expected, recompile, chain in [('b/src/lib.rs', {1}, 6, 6),
                                                  ('a/src/lib.rs', {0, 1}, 16, 16)]:
            with self.subTest(path=path):
                chosen, _ = CRATES.rebuilt(units, waits, self.workspace, [path])
                self.assertEqual(chosen, expected)
                self.assertEqual(sum(units[i]['duration'] for i in chosen), recompile)
                self.assertEqual(CRATES.chain(units, waits, chosen)[0], chain)

    def test_waits_normal_dev_build_own_library_and_later_host_build(self):
        self.data['packages'][3]['targets'][0]['kind'] = ['proc-macro']
        self.data['resolve']['nodes'][2]['deps'].append(
            {'pkg': 'a', 'dep_kinds': [{'kind': 'build', 'target': None}]})
        self.workspace = CRATES.Workspace(self.data)
        rows = tests() + [unit('c', 2, description=' build script'),
                          unit('c', 1, description=' build script (run)', mode='run-custom-build'),
                          unit('b', 9, 3, 20)]
        (self.out / 'tests.html').write_text(report(rows))
        units, waits = self.read_shape('tests')
        self.assertEqual(waits[2], {1})  # b normal dependency, no dev dependencies
        self.assertEqual(waits[3], {13, 14})  # c library: latest b and script run
        self.assertEqual(waits[7], {4, 13, 14})  # c lib test: dev d, not own library
        self.assertEqual(waits[8], {3, 4, 13, 14})  # integration also needs own library
        self.assertEqual(waits[12], {1})  # build compile: a, not normal b or dev d
        self.assertEqual(waits[13], {12})
        for index in (6, 9, 10, 11):  # b's integration, bench, example and binary
            self.assertEqual(waits[index], {1, 14})
        self.assertTrue(units[4]['proc_macro'])

    def test_pipeline_links_proc_macros_and_script_runs_wait_for_end(self):
        rows = [unit('a', 10, 2), unit('b', 6, 1), unit('c', 4, 1)]
        (self.out / 'libraries.html').write_text(report(rows))
        units, _ = self.read_shape('libraries')
        waits = [set(), {0}, {1}]
        self.assertEqual(CRATES.chain(units, waits, range(3), True), (10, [0]))
        units[0]['proc_macro'] = True
        self.assertEqual(CRATES.chain(units, waits, range(3), True), (16, [0, 1]))
        units[0]['proc_macro'] = False
        units[1]['kind'] = 'run'
        self.assertEqual(CRATES.chain(units, waits, range(3), True), (20, [0, 1, 2]))
        units[1]['kind'] = 'test'
        self.assertEqual(CRATES.chain(units, waits, range(3), True)[0], 20)
        self.assertEqual(CRATES.chain(units, waits, set()), (0, []))
        with self.assertRaisesRegex(ValueError, 'cycle'):
            CRATES.chain(units, [{1}, {0}, set()], range(3))

    def test_path_ownership_and_target_only_changes(self):
        for shape in ('libraries', 'tests'):
            units, waits = self.read_shape(shape)
            for path, expected in [('b/tests/test.rs', {6}), ('b/benches/speed.rs', {9}),
                                   ('b/examples/demo.rs', {10}), ('guide.md', set()),
                                   ('bigger/src/lib.rs', set())]:
                with self.subTest(shape=shape, path=path):
                    rebuilt, _ = CRATES.rebuilt(units, waits, self.workspace, [path])
                    self.assertEqual(rebuilt, expected if shape == 'tests' else set())
        self.workspace.directories['d'] = Path('b/nested')
        self.assertEqual(self.workspace.owner('b/nested/src/lib.rs'), ('d', Path('src/lib.rs')))
        self.assertEqual(self.workspace.owner('b/nested-other/a'), ('b', Path('nested-other/a')))

    def test_window_first_parent_and_each_landing_score(self):
        history = self.history()
        self.assertEqual([row['commit'] for row in history], self.expected_history)
        self.assertNotIn(self.old, [row['commit'] for row in history])
        self.assertNotIn(self.side, [row['commit'] for row in history])
        expected = {
            'libraries': [(set(), 0, 0), ({1, 2, 3}, 19, 19), ({2, 3}, 9, 9),
                          (set(), 0, 0), ({3}, 3, 3), (set(), 0, 0), (set(), 0, 0)],
            'tests': [(set(), 0, 0), ({1, 2, 3, 5, 6, 7, 8, 9, 10, 11}, 49, 27),
                      ({2, 3, 6, 7, 8, 9, 10, 11}, 35, 17), ({6}, 5, 5),
                      ({3, 7, 8}, 18, 11), (set(), 0, 0), ({10}, 1, 1)]}
        for shape in ('libraries', 'tests'):
            units, waits = self.read_shape(shape)
            scores = CRATES.score(units, waits, self.workspace, history)
            self.assertEqual([(score['units'], score['recompile'], score['chain']) for score in scores],
                             expected[shape])

    def test_from_scratch_both_shapes(self):
        for shape, total, full, pipelined, path, pipeline_path in [
                ('libraries', 22, 20, 10.5, [0, 1, 2, 3], [0, 1, 2]),
                ('tests', 52, 28, 18.5, [0, 1, 2, 3, 8], [0, 1, 2, 8])]:
            with self.subTest(shape=shape):
                units, waits = self.read_shape(shape)
                self.assertEqual(sum(row['duration'] for row in units), total)
                self.assertEqual(CRATES.chain(units, waits, range(len(units))), (full, path))
                self.assertEqual(CRATES.chain(units, waits, range(len(units)), True), (pipelined, pipeline_path))

    def test_statistics_tenth_is_ceiling_from_top_and_empty_is_zero(self):
        self.assertEqual(CRATES.summary(range(1, 12)), (6, 6, 10))
        self.assertEqual(CRATES.summary([1, 9]), (5, 5, 9))
        self.assertEqual(CRATES.summary([]), (0, 0, 0))

    def test_costliest_ranking_share_counts_and_above(self):
        history = self.history()
        expected = {'libraries': [('b', 6, 3, 2, 12 / 31, 1), ('a', 10, 1, 1, 10 / 31, 2),
                                  ('c', 3, 1, 3, 9 / 31, 0), ('d', 2, 0, 0, 0, 0)],
                    'tests': [('c', 18, 1, 3, .5, 0), ('b', 17, 3, 4, 40 / 108, 1),
                              ('a', 14, 1, 1, 14 / 108, 2), ('d', 2, 0, 0, 0, 1)]}
        for shape in ('libraries', 'tests'):
            units, waits = self.read_shape(shape)
            scores = CRATES.score(units, waits, self.workspace, history)
            rows = CRATES.costliest(units, waits, self.workspace, scores)
            self.assertEqual([tuple(row[key] for key in ('package', 'time', 'changed', 'rebuilt', 'share', 'above'))
                              for row in rows], expected[shape])
        # More members than the table can hold; rank before truncating.
        for index in range(12):
            identity = f'empty-{index}'
            self.workspace.members.add(identity)
            self.workspace.packages[identity] = {'name': identity}
        self.assertEqual(len(CRATES.costliest(units, waits, self.workspace, scores)), 10)

    def test_markdown_sections_statistics_columns_and_landing_order(self):
        self.history()
        self.write('guide.md', 'after the recorded build\n')
        self.commit('not the build', '2026-11-20T12:00:00Z')
        result = self.cli('figure', self.out)
        self.assertEqual(result.returncode, 0, result.stderr)
        text = result.stdout
        self.assertEqual([line for line in text.splitlines() if line.startswith('#')],
                         ['# Build', '## Libraries', '### From scratch', '### Landings', '### Costliest members',
                          '## Tests', '### From scratch', '### Landings', '### Costliest members', '## Each landing'])
        self.assertIn(f'Commit: `{self.head}`; branch: main.', text)
        self.assertIn('rustc 1.95.0 (fixture); release: 1.95.0.', text)
        self.assertIn('CPUs: 8.', text)
        self.assertIn('7 landings.', text)
        self.assertIn('5 units; total compile: 22 s; wall: 10 s.', text)
        self.assertIn('12 units; total compile: 52 s; wall: 18 s.', text)
        self.assertIn('Full chain: 20 s; `external 0.1.0` (1 s) → `a 0.1.0` (10 s) → '
                      '`b 0.1.0` (6 s) → `c 0.1.0` (3 s).', text)
        self.assertIn('Pipelined chain: 10 s; `external 0.1.0` (1 s) → '
                      '`a 0.1.0` (10 s) → `b 0.1.0` (6 s).', text)
        self.assertIn('Recompile: mean 4 s; tenth slowest 19 s.', text)
        self.assertIn('Chain: mean 10 s; median 9 s; tenth slowest 19 s.', text)
        self.assertIn('Recompile: mean 15 s; tenth slowest 49 s.', text)
        self.assertIn('Chain: mean 12 s; median 11 s; tenth slowest 27 s.', text)
        self.assertEqual(text.count('| Crate | Time | Changed | Rebuilt | Share | Above it |'), 2)
        self.assertIn('| `b` | 6 s | 3 | 2 | 39% | 1 |', text)
        self.assertIn('| `c` | 18 s | 1 | 3 | 50% | 0 |', text)
        landing_table = text.split('## Each landing')[1]
        self.assertEqual([line.split('`')[1] for line in landing_table.splitlines() if line.startswith('| `')],
                         [identity[:10] for identity in self.expected_history])
        self.assertIn('| integration only | 0 s | 0 s | 5 s | 5 s |', landing_table)
        self.assertIn('| merge side | 3 s | 3 s | 18 s | 11 s |', landing_table)
        self.assertNotIn('second parent only', text)
        self.assertNotIn('not the build', text)

    def test_unreadable_inputs_exit_one_with_reason(self):
        self.history()
        cases = [('missing', None, 'libraries.html'),
                 ('no data', '<html></html>', 'UNIT_DATA'),
                 ('not array', 'const UNIT_DATA = {}', 'UNIT_DATA'),
                 ('bad json', 'const UNIT_DATA = [oops]', 'Expecting value'),
                 ('unknown package', report([unit('absent', 1)]), 'package not in metadata.json'),
                 ('bad target', report([unit('a', 1, description=' lib (check)')]), 'unreadable target/mode'),
                 ('bad kind', report([unit('a', 1, description=' example "unknown"')]), 'unreadable target/mode')]
        for field in ('start', 'duration'):
            row = unit('a', 1)
            del row[field]
            cases.append((f'missing {field}', report([row]), field))
        for name, content, reason in cases:
            with self.subTest(name=name):
                path = self.out / 'libraries.html'
                if content is None:
                    path.unlink()
                else:
                    path.write_text(content)
                result = self.cli('figure', self.out)
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertIn(reason, result.stderr)
                self.assertEqual(result.stdout, '')
        self.write_reports()
        (self.out / 'metadata.json').write_text('not json')
        self.assertIn('metadata.json', self.cli('figure', self.out).stderr)

    def test_report_target_forms_and_metadata_fallback(self):
        variants = [(' lib (test)', 'lib-test'), (' bin "command"', 'bin'),
                    (' bin "command" (test)', 'bin'), (' test "integration" (test)', 'test'),
                    (' bench "speed" (test)', 'bench'), (' bench "speed" (bench)', 'bench'),
                    (' example "demo"', 'example'), (' example "demo" (test)', 'example'),
                    (' build script', 'build'), (' build script (run)', 'run')]
        rows = [unit('b', 3, description=description,
                     mode='run-custom-build' if kind == 'run' else 'todo') for description, kind in variants]
        (self.out / 'tests.html').write_text(report(rows))
        units, _ = self.read_shape('tests')
        self.assertEqual([row['kind'] for row in units], [kind for _, kind in variants])
        self.assertEqual([row['rmeta_time'] for row in units], [3] * len(rows))
        for description in (' lib (check)', ' lib (check-test)', ' lib (doc)',
                            ' lib (doc test)', ' lib (doc scrape)', ' ???'):
            with self.subTest(description=description):
                (self.out / 'tests.html').write_text(report([unit('b', 1, description=description)]))
                with self.assertRaisesRegex(ValueError, 'unit 0.*unreadable target/mode'):
                    self.read_shape('tests')
        for field, value in [('start', -1), ('duration', 'slow'), ('rmeta_time', float('nan')),
                             ('duration', True), ('rmeta_time', 2)]:
            with self.subTest(field=field, value=value):
                row = unit('a', 1)
                row[field] = value
                (self.out / 'tests.html').write_text(report([row]))
                with self.assertRaisesRegex(ValueError, field):
                    self.read_shape('tests')

    def test_usage_errors_exit_two(self):
        for arguments in ((), ('unknown',), ('build',), ('figure',), ('figure', 'a', 'b')):
            with self.subTest(arguments=arguments):
                result = self.cli(*arguments)
                self.assertEqual(result.returncode, 2)
                self.assertIn('usage:', result.stderr)


FAKE = '''#!INTERPRETER
import json
import os
from pathlib import Path
import sys
name = Path(sys.argv[0]).name
environment = {key: os.environ[key] for key in (
    'TEMPER_INTERNAL_BUNDLE', 'CARGO_INCREMENTAL', 'RUSTC_WRAPPER',
    'CARGO_BUILD_RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_TARGET_DIR',
    'FIXTURE_CALLER') if key in os.environ}
target = Path(os.environ['CARGO_TARGET_DIR']) if 'CARGO_TARGET_DIR' in os.environ else None
entry = {'tool': name, 'args': sys.argv[1:], 'environment': environment,
         'fresh': target.is_dir() and not any(target.iterdir()) if target else None}
with open(os.environ['FIXTURE_LOG'], 'a') as log:
    log.write(json.dumps(entry) + '\\n')
if name == 'rustc':
    print('rustc 1.95.0 (fixture)\\nbinary: rustc\\nrelease: 1.95.0')
elif os.environ.get('FAIL_CARGO') == sys.argv[1]:
    print('fixture cargo refused the build', file=sys.stderr)
    sys.exit(42)
elif sys.argv[1] == 'metadata':
    print(Path(os.environ['FIXTURE_METADATA']).read_text())
else:
    assert entry['fresh']
    destination = target / 'cargo-timings/cargo-timing.html'
    destination.parent.mkdir(parents=True)
    shape = 'libraries' if '--lib' in sys.argv else 'tests'
    destination.write_text((Path(os.environ['FIXTURE_REPORTS']) / (shape + '.html')).read_text())
'''


class Build(Scratch):
    def setUp(self):
        super().setUp()
        tools = self.scratch / 'bin'
        tools.mkdir()
        for name in ('cargo', 'rustc'):
            path = tools / name
            path.write_text(FAKE.replace('INTERPRETER', sys.executable))
            path.chmod(0o700)
        self.log = self.scratch / 'calls.jsonl'
        self.caller_target = self.scratch / 'caller-target'
        self.caller_target.mkdir()
        (self.caller_target / 'cached').write_text('caller cache')
        self.env.update(CARGO_TARGET_DIR=str(self.caller_target), PATH=str(tools) + os.pathsep + self.env['PATH'],
                        FIXTURE_LOG=str(self.log), FIXTURE_METADATA=str(self.out / 'metadata.json'),
                        FIXTURE_REPORTS=str(self.out), FIXTURE_CALLER='kept',
                        TEMPER_INTERNAL_BUNDLE='caller', CARGO_INCREMENTAL='1')
        self.env.update({name: 'must-not-run' for name in CRATES.WRAPPERS})
        self.destination = self.root / 'measurements'

    def test_two_fresh_builds_flags_environment_record_and_cleanup(self):
        result = self.cli('build', '--out', self.destination)
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual([(call['tool'], call['args']) for call in calls], [
            ('rustc', ['-vV']), ('cargo', ['metadata', '--format-version', '1', '--locked']),
            ('cargo', ['build', '--workspace', '--lib', '--locked', '--timings']),
            ('cargo', ['build', '--workspace', '--all-targets', '--locked', '--timings'])])
        targets = []
        for call in calls[2:]:
            env = call['environment']
            self.assertEqual(env['TEMPER_INTERNAL_BUNDLE'], 'none')
            self.assertEqual(env['CARGO_INCREMENTAL'], '0')
            self.assertEqual(env['FIXTURE_CALLER'], 'kept')
            for name in CRATES.WRAPPERS:
                self.assertNotIn(name, env)
            self.assertTrue(call['fresh'])
            target_dir = Path(env['CARGO_TARGET_DIR'])
            self.assertEqual(target_dir.parent, self.destination)
            self.assertFalse(target_dir.exists())
            targets.append(target_dir)
        self.assertNotEqual(*targets)
        self.assertEqual((self.caller_target / 'cached').read_text(), 'caller cache')
        self.assertEqual({path.name for path in self.destination.iterdir()},
                         {'metadata.json', 'build.json', 'libraries.html', 'tests.html'})
        for name in ('libraries.html', 'tests.html', 'metadata.json'):
            self.assertEqual((self.destination / name).read_text(),
                             (self.out / name).read_text() + ('\n' if name == 'metadata.json' else ''))
        record = json.loads((self.destination / 'build.json').read_text())
        self.assertEqual(record['commit'], self.command('git', 'rev-parse', 'HEAD').strip())
        self.assertEqual(record['branch'], 'main')
        self.assertEqual(record['toolchain'], {'first_line': 'rustc 1.95.0 (fixture)', 'release': 'release: 1.95.0'})
        self.assertIsInstance(record['cpu_count'], int)
        self.assertGreater(record['cpu_count'], 0)
        self.assertEqual(record['environment'], {'TEMPER_INTERNAL_BUNDLE': 'none', 'CARGO_INCREMENTAL': '0'})
        self.assertEqual(set(record['removed_environment']), set(CRATES.WRAPPERS))
        for index, name in enumerate(('libraries', 'tests'), 2):
            entry = record['builds'][name]
            self.assertEqual(entry['command'], ['cargo', *calls[index]['args']])
            self.assertRegex(entry['start'], r'^\d{4}-\d\d-\d\dT.*\+00:00$')
            self.assertEqual(entry['environment']['CARGO_TARGET_DIR'], str(targets[index - 2]))

    def test_detached_head_omits_branch_and_accepts_existing_empty_directory(self):
        self.command('git', 'checkout', '--detach', '-q')
        self.destination.mkdir()
        result = self.cli('build', '--out', self.destination)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('branch', json.loads((self.destination / 'build.json').read_text()))

    def test_refuses_tracked_untracked_or_nonempty_before_running_tools(self):
        for case in ('tracked', 'untracked'):
            with self.subTest(case=case):
                if case == 'tracked':
                    self.write('a/src/lib.rs', '// changed\n')
                elif case == 'untracked':
                    self.write('surprise', 'untracked\n')
                result = self.cli('build', '--out', self.destination)
                self.assertEqual(result.returncode, 1)
                self.assertIn('working tree', result.stderr)
                self.assertFalse(self.log.exists())
                if case == 'tracked':
                    self.command('git', 'restore', 'a/src/lib.rs')
                elif case == 'untracked':
                    (self.root / 'surprise').unlink()
        # An outside, nonempty output is refused even when git sees a clean tree.
        result = self.cli('build', '--out', self.out)
        self.assertEqual(result.returncode, 1)
        self.assertIn('not empty', result.stderr)
        self.assertFalse(self.log.exists())
        self.assertEqual((self.out / 'libraries.html').read_text(), report(libraries()))

    def test_cargo_failure_has_reason_exit_one_and_removes_target(self):
        result = self.cli('build', '--out', self.destination, env=dict(self.env, FAIL_CARGO='build'))
        self.assertEqual(result.returncode, 1)
        self.assertIn('fixture cargo refused the build', result.stderr)
        self.assertNotIn('Traceback', result.stderr)
        calls = [json.loads(line) for line in self.log.read_text().splitlines()]
        self.assertEqual(len(calls), 3)
        self.assertFalse(Path(calls[-1]['environment']['CARGO_TARGET_DIR']).exists())
        self.assertFalse((self.destination / 'build.json').exists())


if __name__ == '__main__':
    unittest.main()
