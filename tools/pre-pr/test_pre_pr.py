"""Delivery selection, known reds and inherited counts (tools/pre-pr/design.md, 3, 5, 9)."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
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
                clippy_answer=(0, []), one_process_answer=(0, [], None), process_answer=None):
        calls = []
        data = {'packages': [{'name': name, 'id': name, 'dependencies': [],
                             'manifest_path': str(self.root / name / 'Cargo.toml')} for name in packages],
                'workspace_members': list(packages), 'target_directory': str(self.root / 'target')}

        def run(arguments, cwd, env, output=None, stdout=False):
            calls.append((arguments, cwd))
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

        captured = io.BytesIO()
        log = io.TextIOWrapper(captured, encoding="utf-8")
        with contextlib.chdir(self.root), mock.patch.dict(os.environ, self.env, clear=True), \
                mock.patch.object(PRE_PR, 'metadata', return_value=data), \
                mock.patch.object(PRE_PR, 'formatting', return_value=[]) as formatting, \
                mock.patch.object(PRE_PR, 'clippy', return_value=clippy_answer) as clippy, \
                mock.patch.object(PRE_PR, 'one_process', side_effect=(
                    PRE_PR.one_process if one_process_answer is None else lambda *args: one_process_answer)) as one_process, \
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
