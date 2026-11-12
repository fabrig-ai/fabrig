"""Regression checks for the shared code/comment ratchet and its tree entries."""
from pathlib import Path
import contextlib
import io
import runpy
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
ENGINE = runpy.run_path(str(Path(__file__).with_name('old_forms.py')))
CONFIG = {tree: runpy.run_path(str(ROOT / tree / 'scripts/old_forms.py'))
          for tree in ('skein', 'smith', 'jig', 'temper')}


class SharedForms(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='old-forms-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True, capture_output=True)

    def count(self, source, name='crates/smith-domain/src/lib.rs', tree='smith'):
        code, comments, syntax = ENGINE['split_source'](source)
        config = CONFIG[tree]
        syntax_names = config.get('SYNTAX_FORMS', ())
        totals = {}
        for forms, text, comment in (
            ({k: v for k, v in config['FORMS'].items() if k not in syntax_names}, code, False),
            ({k: v for k, v in config['FORMS'].items() if k in syntax_names}, syntax, False),
            (config['COMMENT_FORMS'], comments, True),
        ):
            totals.update(ENGINE['count_text'](config, forms, text, name,
                                               self.root / tree, self.root, comments=comment))
        return totals

    def test_comments_and_literals_have_independent_counts(self):
        totals = self.count('''
// actual actually genuine genuinely originally previously no longer used to formerly
/// Contract: smith/docs/design/domain/run.md
//! Lower test of something
/* outer actual /* nested genuinely */ end */
let actual_count = "genuine actually";
let myActualValue = r#"https://example.test/actual /* genuine */"#;
let actualResult = 'x';
''')
        self.assertEqual({k: totals[k] for k in ('actual', 'genuine', 'comment:actual', 'comment:genuine')},
                         {'actual': 5, 'genuine': 2, 'comment:actual': 3, 'comment:genuine': 3})
        for word in ('originally', 'previously', 'no_longer', 'used_to', 'formerly',
                     'contract_stamp', 'lower_test_of'):
            self.assertEqual(totals['comment:' + word], 1, word)

    def test_raw_strings_chars_lifetimes_and_unterminated_comments(self):
        source = '''fn f<'a>(x: &'a str) { let c = '/'; let b = b'"';
let s = br##"// actual /* genuine */"##; // genuinely
/* actual /* nested */'''
        code, comments, syntax = ENGINE['split_source'](source)
        self.assertIn("fn f<'a>(x: &'a str)", code)
        self.assertIn('// actual /* genuine */', code)
        self.assertNotIn('genuinely', code)
        self.assertNotIn('actual', syntax)
        self.assertIn('/* actual /* nested */', comments)
        self.assertEqual(ENGINE['without_comments'](r'"\nactual"'), '" actual"')

    def test_allow_expect_and_path_ignore_comment_and_string_examples(self):
        totals = self.count('''
#[allow(dead_code)] #![allow(unused)] #[path = "file.rs"]
#[expect(clippy::manual_let_else, reason = "x")]
#[expect(clippy::single_match, reason = "x")]
// #[allow(dead_code)] #[path = "file.rs"]
let s = "#[allow(dead_code)] expect(clippy::single_match) #[path";
''')
        self.assertEqual(totals['allow'], 2)
        self.assertEqual(totals['path_attribute'], 1)
        self.assertEqual(totals['expect_manual_let_else'], 1)
        self.assertEqual(totals['expect_single_match'], 1)

    def test_limits_are_literals_of_any_qualified_limits_type(self):
        totals = self.count('''
pub struct Limits { x: usize }
impl Limits { fn f() -> Limits { Limits { x: 1 } } }
impl Trait for Limits { }
fn other() -> module::Limits { module::Limits { x: 2 } }
let a = Limits { x: { other::Value { y: 3 } }, z: "}" };
let b = Limits { x: 3, ..DEFAULT };
let c = Limits { x: Other { ..DEFAULT } };
let range = Limits { x: 0..4 };
let generic = other::Limits::<u8> { x: 3 };
let Limits { x } = a;
match a { Limits { x } => x }
let s = "Limits { x: 1 }"; // Limits { x: 1 }
''')
        self.assertEqual(totals['full_limits'], 6)

    def test_limits_of_the_file_layer_are_the_new_form(self):
        totals = self.count('''
let a = file_layer::Limits { files: 2 };
let b = skein_io::file_layer::Limits { files: 2 };
let c = other_layer::Limits { files: 2 };
''')
        self.assertEqual(totals['full_limits'], 1)

    def test_smith_limits_exempt_fixture_definitions_but_count_readers_and_step_copies(self):
        source = 'const SMALL: Limits = Limits { calls: 2 }; fn worst_case() {}'
        fixture = self.count(source, 'testing/smith-protocol-llm-fixtures/src/lib.rs')
        self.assertEqual(fixture['full_limits'], 0)
        self.assertEqual(fixture['worst_case'], 1)
        for name in ('tests/protocol-llm/src/fixture.rs',
                     'crates/smith-protocol-llm/src/fixture.rs', 'testing_more/src/lib.rs'):
            with self.subTest(name=name):
                self.assertEqual(self.count(source, name)['full_limits'], 1)
        inline = '#[cfg(test)] mod tests { ' + source + ' }'
        self.assertEqual(self.count(inline, 'crates/smith-protocol-llm/src/lib.rs')['full_limits'], 1)

    def test_test_windows_in_files_and_inline_items(self):
        source = '''fn production() { bytes.windows(2); }
#[cfg(test)] mod checks { #[test] fn check() { bytes.windows(2); } }
#[test] fn other() { bytes.windows(3); }
// .windows(4)
let s = ".windows(5)";
'''
        self.assertEqual(self.count(source)['test_windows'], 2)
        for name in ('tests/world/src/lib.rs', 'crates/smith-domain/src/tests.rs',
                     'crates/smith-domain/src/route_tests.rs', 'benchmarks/world.rs'):
            self.assertEqual(self.count(source, name)['test_windows'], 3, name)

    def test_domain_loops_exclude_test_harnesses_and_other_crates(self):
        source = '''fn production() { while condition {} loop {} }
#[cfg(test)] mod checks { fn check() { while condition {} loop {} } }
'''
        domains = ('smith-domain', 'smith-domain-run', 'smith-domain-session', 'smith-domain-tools',
                   'smith-host-domain', 'smith-local-domain', 'smith-inline-agent')
        for domain in domains:
            totals = self.count(source, f'crates/{domain}/src/lib.rs')
            self.assertEqual((totals['domain_while'], totals['domain_loop']), (1, 1), domain)
            totals = self.count(source, f'crates/{domain}/src/tests.rs')
            self.assertEqual((totals['domain_while'], totals['domain_loop']), (0, 0), domain)
        totals = self.count(source, 'crates/smith-agent-shell/src/lib.rs')
        self.assertEqual((totals['domain_while'], totals['domain_loop']), (0, 0))

    def test_external_test_module_does_not_hide_following_production_code(self):
        source = '''#[cfg(test)] mod checks;
fn production() { while condition {} loop {} bytes.windows(2); }
'''
        totals = self.count(source)
        self.assertEqual((totals['domain_while'], totals['domain_loop']), (1, 1))
        self.assertEqual(totals['test_windows'], 0)

    def test_wire_kind_pattern_counts_one_byte_fields_and_writer_arguments(self):
        totals = self.count('''
frame_writer(0x01, body); wire::push_record(0xFF, body);
Kind { kind: 0x02 }; Kind { kind: 0x0100 };
frame_writer(0x0100, body); let bytes = [0xff]; let unrelated = 0x01;
// frame_writer(0x02, body)
let s = "kind: 0x03";
''')
        self.assertEqual(totals['bare_wire_kind'], 3)

    def test_temp_dir_and_io_resume_ignore_literals_and_comments(self):
        totals = self.count('''
std::env::temp_dir(); temp_dir (); io::resume(); skein_io::resume();
// temp_dir() io::resume
let s = "temp_dir() io::resume";
''')
        self.assertEqual(totals['temp_dir'], 2)
        self.assertEqual(totals['io_resume'], 2)

    def test_dead_citations_resolve_paths_in_the_repository(self):
        for name in ('smith/docs/design/domain/run.md', 'skein/docs/foundation/testing-strategy.md',
                     'smith/docs/design/limits.md', 'smith/crates/smith-domain/src/nearby.md',
                     'skein/docs/foundation/programming-model.md'):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.touch()
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True, capture_output=True)
        subprocess.run(['git', 'add', '.'], cwd=self.root, check=True, capture_output=True)
        source = '''
//! smith/docs/design/domain/run.md, 3; skein/docs/foundation/testing-strategy.md, 1.
/// docs/design/domain/run.md; limits.md; nearby.md; programming-model.md
// scratch/client.md, missing.md, missing.md
// https://example.test/remote.md
let s = "missing.md";
'''
        self.assertEqual(self.count(source)['comment:dead_citation'], 3)
        (self.root / 'smith/scratch').mkdir()
        (self.root / 'smith/scratch/client.md').touch()
        self.assertEqual(self.count(source)['comment:dead_citation'], 2)
        (self.root / 'skein/docs/foundation/programming-model.md').unlink()
        self.assertEqual(self.count(source)['comment:dead_citation'], 3)

    def test_markers_words_and_both_exemption_styles(self):
        self.assertEqual(self.count('worst_case_bytes worst_case_bytes_extra', tree='skein')['worst_case'], 1)
        for tree in ('jig', 'temper'):
            totals = self.count('io::MAX_OUT_DOWN other::io::MAX_OUT_DOWN MAX_OUT_DOWN', tree=tree)
            self.assertEqual(totals['max_out'], 2, tree)
        totals = self.count('io::resume();', 'crates/skein-io/src/lib.rs', tree='skein')
        self.assertEqual(totals['io_resume'], 0)
        totals = self.count('io::resume();', 'crates/skein-other/src/lib.rs', tree='skein')
        self.assertEqual(totals['io_resume'], 1)
        config = dict(CONFIG['skein'], EXCLUDE={'comment:actual': ('excluded/',)})
        self.assertEqual(ENGINE['count_text'](config, config['COMMENT_FORMS'], '// actual',
                                            'excluded/a.rs', self.root, self.root,
                                            comments=True)['comment:actual'], 0)

    def test_each_tree_entry_initializes_and_ratchets_comments_in_isolated_mode(self):
        for tree in CONFIG:
            with self.subTest(tree=tree):
                root = self.root / tree
                root.mkdir()
                (root / 'scripts').mkdir()
                subprocess.run(['git', 'init', '-q', str(root)], check=True, capture_output=True)
                (root / 'lib.rs').write_text('// actual\n')
                subprocess.run(['git', 'add', 'lib.rs'], cwd=root, check=True, capture_output=True)
                command = [sys.executable, '-I', '-B', str(ROOT / tree / 'scripts/old_forms.py')]
                result = subprocess.run([*command, '--write'], cwd=root, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                baseline = root / 'scripts/old-forms.txt'
                self.assertIn('comment:actual 1\n', baseline.read_text())
                before = baseline.read_bytes()
                (root / 'lib.rs').write_text('// actual actually\n')
                for args in ([], ['--write']):
                    result = subprocess.run([*command, *args], cwd=root, capture_output=True, text=True)
                    self.assertEqual(result.returncode, 1, result.stderr)
                    self.assertIn('comment:actual: count 2, baseline 1', result.stderr)
                    self.assertEqual(baseline.read_bytes(), before)

    def test_baseline_append_validation_and_raise_comments(self):
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True, capture_output=True)
        config = dict(CONFIG['smith'], BASELINE=self.root / 'old-forms.txt')
        # A prefix baseline can append new forms, preserving existing comments.
        config['BASELINE'].write_text('# Raised: retained\nmax_out 0\nJournalRoom 0\ntakes 0\nworst_case 0')
        before = config['BASELINE'].read_bytes()
        with contextlib.redirect_stderr(io.StringIO()), contextlib.chdir(self.root):
            with self.assertRaises(SystemExit) as error:
                ENGINE['main'](config, [])
            self.assertEqual(error.exception.code, 2)
        self.assertEqual(config['BASELINE'].read_bytes(), before)
        with contextlib.chdir(self.root):
            self.assertEqual(ENGINE['main'](config, ['--write']), 0)
        self.assertTrue(config['BASELINE'].read_text().startswith('# Raised: retained\n'))
        self.assertIn('comment:dead_citation 0\n', config['BASELINE'].read_text())
        for malformed in ('max_out -1\n', 'max_out 0\nmax_out 0\n', 'takes 0\n', 'max_out nope\n'):
            config['BASELINE'].write_text(malformed)
            with contextlib.redirect_stderr(io.StringIO()), contextlib.chdir(self.root):
                with self.assertRaises(SystemExit) as error:
                    ENGINE['main'](config, ['--write'])
                self.assertEqual(error.exception.code, 2)
            self.assertEqual(config['BASELINE'].read_text(), malformed)


if __name__ == '__main__':
    unittest.main()
