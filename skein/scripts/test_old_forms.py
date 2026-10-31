"""Ratchet the forms of skein/docs/design/draft/component-kit.md, 4."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name('old_forms.py').resolve()
FORMS = (
    'max_out', 'JournalRoom', 'takes', 'worst_case',
    'io_resume', 'earliest', 'Lifecycle', 'GROWTH_FLOOR',
    'pop_loop', 'hex_decimal', 'forwarded_host',
)
SOURCES = (
    ('io_resume', 'skein_io::resume(host, now);'),
    ('earliest', 'fn earliest(a: Time, b: Time) -> Time { a }'),
    ('Lifecycle', 'enum Lifecycle { Active, Closing, Closed }'),
    ('GROWTH_FLOOR', 'const GROWTH_FLOOR: usize = 4;'),
    ('pop_loop', 'for _ in 0..n { let _ = self.queue.pop(); }'),
    ('hex_decimal', 'fn hex(byte: u8) -> u8 { byte }'),
    ('forwarded_host', 'impl io::Host for Process {}'),
)
EXCLUDED = (
    ('io_resume', 'crates/skein-io/', 'io::resume(host, now);'),
    ('earliest', 'crates/skein-lib/', 'fn earlier(a: Time) -> Time { a }'),
    ('Lifecycle', 'crates/skein-lib/', 'enum Lifecycle { Active }'),
    ('hex_decimal', 'crates/skein-lib/', 'fn hex(byte: u8) -> u8 { byte }'),
)


class OldForms(unittest.TestCase):
    def baseline(self, counts):
        return ''.join(f'{form} {counts.get(form, 0)}\n' for form in FORMS)

    def repository(self, files, counts=None, untracked=None):
        temporary = tempfile.TemporaryDirectory(
            prefix='.test-old-forms-', dir=SCRIPT.parents[2])
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        subprocess.run(['git', 'init', '--quiet', str(root)], check=True,
                       capture_output=True, timeout=10)
        for name, source in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding='utf-8')
        subprocess.run(['git', 'add', '--', *files], cwd=root, check=True,
                       capture_output=True, timeout=10)
        for name, source in (untracked or {}).items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding='utf-8')
        (root / 'scripts').mkdir(exist_ok=True)
        (root / 'scripts/old-forms.txt').write_text(
            self.baseline(counts or {}), encoding='utf-8')
        return root

    def run_script(self, root, *args, exit=0):
        result = subprocess.run([sys.executable, '-I', '-B', str(SCRIPT), *args],
                                cwd=root, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, exit, result.stderr)
        self.assertEqual(result.stdout, '')
        if exit == 0:
            self.assertEqual(result.stderr, '')
        return result

    def test_each_new_form_counts(self):
        for form, source in SOURCES:
            with self.subTest(form=form):
                root = self.repository({'src/lib.rs': source}, {form: 1})
                self.run_script(root)

    def test_comments_do_not_count_for_any_new_form(self):
        for form, source in SOURCES:
            for comment in ('// {}\n', '/// {}\n', '//! {}\n', '/* /* nested */ {} */'):
                with self.subTest(form=form, comment=comment):
                    root = self.repository({'src/lib.rs': comment.format(source)})
                    self.run_script(root)

    def test_nearby_syntax_does_not_count(self):
        cases = (
            ('io_resume', 'other_io::resume(host, now);'),
            ('earliest', 'fn earliest(a: Duration) -> Time { a }'),
            ('Lifecycle', 'enum LifecycleState { Active }'),
            ('GROWTH_FLOOR', 'const GROWTH_FLOOR_BYTES: usize = 4;'),
            ('pop_loop', 'for _ in 0..n { let item = queue.pop(); }'),
            ('hex_decimal', 'fn hexadecimal(byte: u8) -> u8 { byte }'),
            ('forwarded_host', 'impl io::Host for ProcessState {}'),
        )
        for form, source in cases:
            with self.subTest(form=form):
                root = self.repository({'src/lib.rs': source})
                self.run_script(root)

    def test_owned_paths_do_not_count_their_excluded_forms(self):
        for form, prefix, source in EXCLUDED:
            with self.subTest(form=form):
                root = self.repository({prefix + 'src/lib.rs': source})
                self.run_script(root)

    def test_similar_path_prefixes_still_count(self):
        for form, prefix, source in EXCLUDED:
            with self.subTest(form=form):
                root = self.repository({prefix.rstrip('/') + '-other/src/lib.rs': source},
                                       {form: 1})
                self.run_script(root)

    def test_path_exclusions_apply_only_to_the_named_forms(self):
        cases = (
            ('crates/skein-lib/src/lib.rs', 'io::resume(host, now);', 'io_resume'),
            ('crates/skein-lib/src/lib.rs', 'const GROWTH_FLOOR: usize = 4;', 'GROWTH_FLOOR'),
            ('crates/skein-io/src/lib.rs', 'enum Lifecycle { Active }', 'Lifecycle'),
        )
        for path, source, form in cases:
            with self.subTest(path=path, form=form):
                root = self.repository({path: source}, {form: 1})
                self.run_script(root)

    def test_alternate_form_spellings_count(self):
        cases = (
            ('io_resume', 'io::resume (host, now);'),
            ('earliest', 'fn earlier(a: Time) -> Time { a }'),
            ('hex_decimal', 'fn decimal(byte: u8) -> u8 { byte }'),
            ('forwarded_host', 'impl Host for Process {}'),
            ('forwarded_host', 'impl skein_io::stage::Host for Process {}'),
        )
        for form, source in cases:
            with self.subTest(form=form, source=source):
                root = self.repository({'src/lib.rs': source}, {form: 1})
                self.run_script(root)

    def test_pop_loops_count_each_discard_spelling(self):
        for body in ('queue.pop();', 'drop(self.queue.pop());', 'let _ = queue.pop();'):
            with self.subTest(body=body):
                source = 'for _ in 0..n {\n' + body + '\n}'
                root = self.repository({'src/lib.rs': source}, {'pop_loop': 1})
                self.run_script(root)

    def test_pop_loops_with_other_work_do_not_count(self):
        for body in (
            'let _ = queue.pop(); count += 1;',
            'record(); drop(queue.pop());',
            'if ready { let _ = queue.pop(); }',
        ):
            with self.subTest(body=body):
                root = self.repository({'src/lib.rs': 'for _ in 0..n { ' + body + ' }'})
                self.run_script(root)

    def test_original_forms_keep_their_identifier_and_exemption_rules(self):
        root = self.repository({'src/lib.rs': '''
let max_out_items = MAX_OUT_DOWNWARD;
let room: JournalRoom = journal.room(); journal.takes(&room);
let bound = payload.worst_case();
let exempt = (MAX_OUT_DOWN, MAX_OUT_UP, MAX_OUT_RESUME, MAX_OUT_FIRE, max_output_tokens,
              worst_case_bytes, worst_case_heap, worst_case_bytes_within, worst_case_heap_within);
// max_out JournalRoom journal.takes(&room) worst_case
'''}, {'max_out': 2, 'JournalRoom': 1, 'takes': 1, 'worst_case': 1})
        self.run_script(root)

    def test_generated_and_untracked_sources_do_not_count(self):
        source = '\n'.join(source for _, source in SOURCES)
        root = self.repository({
            'src/lib.rs': 'fn ordinary() {}',
            'generated/top.rs': source,
            'src/generated/codec.rs': source,
        }, untracked={'src/untracked.rs': source})
        self.run_script(root)

    def test_missing_baseline_initializes_all_eleven_forms(self):
        root = self.repository({'src/lib.rs': '\n'.join(source for _, source in SOURCES)})
        baseline = root / 'scripts/old-forms.txt'
        baseline.unlink()
        self.run_script(root, '--write')
        entries = [line for line in baseline.read_text(encoding='utf-8').splitlines()
                   if not line.startswith('#')]
        self.assertEqual(entries, [
            'max_out 0', 'JournalRoom 0', 'takes 0', 'worst_case 0',
            'io_resume 1', 'earliest 1', 'Lifecycle 1', 'GROWTH_FLOOR 1',
            'pop_loop 1', 'hex_decimal 1', 'forwarded_host 1',
        ])

    def test_write_lowers_new_forms_and_preserves_comments(self):
        root = self.repository({'src/lib.rs': 'io::resume(host, now);'}, {'io_resume': 2})
        baseline = root / 'scripts/old-forms.txt'
        comment = '# Raised: io_resume +1, recorded exception\n'
        baseline.write_text(comment + self.baseline({'io_resume': 2}), encoding='utf-8')
        self.run_script(root, '--write')
        self.assertEqual(baseline.read_text(encoding='utf-8'),
                         comment + self.baseline({'io_resume': 1}))

    def test_write_refuses_to_raise_each_new_form(self):
        for form, source in SOURCES:
            with self.subTest(form=form):
                root = self.repository({'src/lib.rs': source})
                baseline = root / 'scripts/old-forms.txt'
                before = baseline.read_bytes()
                result = self.run_script(root, '--write', exit=1)
                self.assertEqual(result.stderr.splitlines()[0], f'{form}: count 1, baseline 0')
                self.assertIn('skein/docs/design/draft/component-kit.md, 4', result.stderr)
                self.assertEqual(baseline.read_bytes(), before)


if __name__ == '__main__':
    unittest.main()
