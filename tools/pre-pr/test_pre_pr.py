"""Delivery selection and base judgments (tools/pre-pr/design.md, 3, 5, 9)."""
import contextlib
import importlib.util
import io
import os
from pathlib import Path
import subprocess
import tempfile
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

    def deliver(self, packages=('api',), expected_bundle=None):
        calls = []
        data = {'packages': [{'name': name, 'id': name, 'dependencies': [],
                             'manifest_path': str(self.root / name / 'Cargo.toml')} for name in packages],
                'workspace_members': list(packages), 'target_directory': str(self.root / 'target')}

        def run(arguments, cwd, env, output=None, stdout=False):
            calls.append((arguments, cwd))
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
                mock.patch.object(PRE_PR, 'clippy', return_value=(0, [])) as clippy, \
                mock.patch.object(PRE_PR, 'nextest', return_value=(0, [])) as nextest, \
                mock.patch.object(PRE_PR, 'run', side_effect=run), \
                contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
            result = PRE_PR.main()
        log.flush()
        if expected_bundle is not None:
            self.assertTrue(clippy.called)
            self.assertTrue(nextest.called)
            self.assertEqual(clippy.call_args.args[3]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
            self.assertEqual(nextest.call_args.args[2]['TEMPER_INTERNAL_BUNDLE'], expected_bundle)
        return result, captured.getvalue().decode(), calls, (formatting.called, clippy.called, nextest.called)

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
        for path in ('Cargo.lock', 'rustfmt.toml', '.config/nextest.toml',
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


class SharedTarget(unittest.TestCase):
    def test_base_and_head_builds_use_their_own_sources(self):
        self.check_builds(generated=False)

    def test_build_script_regenerates_each_sides_code(self):
        for missing in (False, True):
            with self.subTest(missing_input=missing):
                self.check_builds(generated=True, missing=missing)

    def test_out_of_package_include_uses_each_sides_page(self):
        self.check_builds(generated=False, embedded=True)

    def check_builds(self, generated, missing=False, embedded=False):
        with tempfile.TemporaryDirectory(prefix='pre-pr-target-') as directory:
            root = Path(directory)
            env = os.environ.copy()
            for name in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'CARGO_TARGET_DIR'):
                env.pop(name, None)
            env.update(TEMPER_INTERNAL_BUNDLE='none', CARGO_TARGET_DIR=str(root / 'target'))

            def command(*args, cwd=root, command_env=env):
                return subprocess.run(args, cwd=cwd, env=command_env, check=True,
                                      capture_output=True, text=True).stdout

            def write(name, content):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content)

            command('git', 'init', '-q')
            command('git', 'config', 'user.name', 'Pre-pr test')
            command('git', 'config', 'user.email', 'pre-pr@example.invalid')
            command('git', 'config', 'commit.gpgsign', 'false')
            write('.gitignore', '/target\n')
            api = 'crates/x' if embedded else 'api'
            write('Cargo.toml', f'[workspace]\nmembers = ["{api}", "consumer"]\nresolver = "3"\n')
            write(f'{api}/Cargo.toml', '[package]\nname = "api"\nversion = "0.1.0"\nedition = "2024"\n')
            write('consumer/Cargo.toml', '[package]\nname = "consumer"\nversion = "0.1.0"\n'
                  f'edition = "2024"\n[dependencies]\napi = {{ path = "../{api}" }}\n')
            source = 'api/value.txt' if generated else 'api/src/lib.rs'
            if embedded:
                source = 'crates/docs/page.md'
                write(f'{api}/src/lib.rs',
                      'pub fn value() -> &\'static str { include_str!("../../docs/page.md").trim() }\n')
                write('.config/gate/always.txt', 'api crates/docs/\n')
                write('.config/gate/read-by-tests.txt', 'crates/docs/page.md\n')
            if generated:
                write('api/src/lib.rs', 'include!(concat!(env!("OUT_DIR"), "/value.rs"));\n')
                write('api/build.rs', '''fn main() {
    println!("cargo:rerun-if-changed=value.txt");
    let value = std::fs::read_to_string("value.txt").unwrap_or_else(|_| "head".to_owned());
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("value.rs");
    std::fs::write(output, format!("pub fn value() -> &'static str {{ {:?} }}", value.trim())).unwrap();
}
''')
            write(source, 'base\n' if generated or embedded else 'pub fn value() -> &\'static str { "base" }\n')
            write('consumer/src/main.rs', 'fn main() { println!("{}", api::value()); }\n')
            command('cargo', 'generate-lockfile', '--offline')
            command('git', 'add', '.')
            command('git', 'commit', '-qm', 'base')
            command('git', 'update-ref', 'refs/temper/landing', 'HEAD')
            # A kept base checkout can predate the head's shared artifacts too.
            worktree = root / '.git/temper-pre-pr/base'
            if not embedded:
                command('git', 'worktree', 'add', '--detach', str(worktree), 'HEAD')
            if missing:
                (root / source).unlink()
            else:
                write(source, 'head\n' if generated or embedded else 'pub fn value() -> &\'static str { "head" }\n')
            # Older head sources reproduce Cargo accepting the base's newer rlib.
            older = time.time_ns() - 10_000_000_000
            if not missing:
                os.utime(root / source, ns=(older, older))
            self.assertEqual(command('cargo', 'run', '--offline', '-q', '-p', 'consumer'), 'head\n')
            index = (root / '.git/index').read_bytes()
            unchanged = (root / 'consumer/src/main.rs').stat().st_mtime_ns
            real_run = PRE_PR.run
            base_builds = []

            def run(arguments, cwd, run_env, *args, **kwargs):
                if arguments[0] == 'python3':
                    return 0, ''
                return real_run(arguments, cwd, run_env, *args, **kwargs)

            def build_base(cwd, build_env):
                self.assertEqual(build_env['CARGO_TARGET_DIR'], str(root / 'target'))
                command('cargo', 'build', '--offline', '-q', '-p', 'consumer',
                        cwd=cwd, command_env=build_env)
                self.assertEqual(command(str(root / 'target/debug/consumer')), 'base\n')
                base_builds.append(cwd)
                return 0, []

            def clippy(packages, data, cwd, build_env):
                if cwd == root:
                    return 101, [PRE_PR.failure('clippy api lint api/src/lib.rs', 'head lint',
                                                ('api', 'lint', 'api/src/lib.rs', 'head lint'), 'api')]
                return build_base(cwd, build_env)

            def nextest(packages, cwd, build_env, expression=None):
                if not embedded:
                    return 0, []
                if cwd == root:
                    return 100, [PRE_PR.failure('test api::pages page', 'head failure',
                                                ('api::pages', 'page'), 'api')]
                return build_base(cwd, build_env)

            def judge():
                with contextlib.chdir(root), mock.patch.dict(os.environ, env, clear=True), \
                        mock.patch.object(PRE_PR, 'formatting', return_value=[]), \
                        mock.patch.object(PRE_PR, 'run', side_effect=run), \
                        mock.patch.object(PRE_PR, 'clippy', side_effect=clippy), \
                        mock.patch.object(PRE_PR, 'nextest', side_effect=nextest):
                    self.assertEqual(PRE_PR.main(), 1)

            # The second page judgment uses a kept base checkout after a head build.
            for judgment in range(2 if embedded else 1):
                judge()
                self.assertEqual(len(base_builds), judgment + 1)
                dep_info = '\n'.join(repr(path.read_bytes()) for path in
                                     (root / 'target/debug/.fingerprint').glob('api-*/dep-lib-api'))
                self.assertEqual(command('cargo', 'run', '--offline', '-q', '-p', 'consumer'), 'head\n', dep_info)
            self.assertEqual((root / '.git/index').read_bytes(), index)
            self.assertEqual((root / 'consumer/src/main.rs').stat().st_mtime_ns, unchanged)


if __name__ == '__main__':
    unittest.main()
