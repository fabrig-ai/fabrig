"""Scratch merges for temper/docs/design/protocol/tooling.md, 4.1."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
BASELINE = 'temper/scripts/old-forms.txt'
KNOWN_REDS = '.temper/known-reds.md'
RECORDING = 'temper/tests/playground/tests/fixtures/story-a.log'
GENERATOR = 'temper/tests/playground/tests/suite/walk.rs'
WALK = 'walk::story_a_walks_from_the_welcome_to_the_csv_change_landed'


def counts(first, second, comment='# Head baseline\n'):
    return (f'{comment}max_out {first}\nJournalRoom {second}\ntakes 0\n'
            'worst_case 0\ncomment:actual 0\ncomment:genuine 0\n')


class Merge(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='merge-')
        self.addCleanup(temporary.cleanup)
        self.job = Path(temporary.name)
        self.repository = self.job / 'fabrig'
        self.repository.mkdir()
        self.env = {'PATH': '/usr/bin:/bin', 'HOME': str(self.job),
                    'LANG': 'C.UTF-8', 'TEMPER_JOB': str(self.job)}
        self.git('init', '-q', '-b', 'head')
        self.git('config', 'user.name', 'Merge test')
        self.git('config', 'user.email', 'merge@example.test')
        self.git('config', 'commit.gpgsign', 'false')
        for name in ('.temper/merge', 'tools/merge/merge.py',
                     'tools/old_forms/old_forms.py', 'temper/scripts/old_forms.py'):
            target = self.repository / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, target)
        self.write(BASELINE, counts(12, 12))
        self.write('temper/uses.rs', 'const WORDS: &str = "' +
                   ' '.join(['max_out'] * 6 + ['JournalRoom'] * 6) + '";\n')
        self.write('temper/x.rs', 'const X: u8 = 0;\n')

    def git(self, *args, check=True):
        return subprocess.run(['git', *args], cwd=self.repository, env=self.env,
                              check=check, capture_output=True)

    def write(self, name, content):
        path = self.repository / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            path.write_bytes(content)
        else:
            path.write_text(content, encoding='utf-8')

    def commit(self, message):
        self.git('add', '.')
        self.git('commit', '-qm', message)

    def conflict(self, baseline=True, other=False, head_comment='# Head baseline\n',
                 base_comment='# Incoming baseline\n', trees=('temper',), delete=None,
                 other_path='temper/x.rs', other_paths=(), head_files=(), base_files=(),
                 known_reds=None):
        if known_reds is not None and known_reds[0] is not None:
            self.write(KNOWN_REDS, known_reds[0])
        self.commit('common')
        self.git('branch', 'base')
        if baseline:
            for tree in trees:
                self.write(f'{tree}/scripts/old-forms.txt', counts(8, 4, head_comment))
            if delete == 'head':
                self.git('rm', '-f', BASELINE)
        if other:
            for path in (other_path, *other_paths):
                self.write(path, 'const X: u8 = 1;\n')
        for path, content in head_files:
            self.write(path, content)
        if known_reds is not None:
            if known_reds[1] is None:
                self.git('rm', '-f', KNOWN_REDS)
            else:
                self.write(KNOWN_REDS, known_reds[1])
        self.commit('head')
        head = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.git('checkout', '-q', 'base')
        if baseline:
            for tree in trees:
                self.write(f'{tree}/scripts/old-forms.txt', counts(4, 8, base_comment))
            if delete == 'base':
                self.git('rm', '-f', BASELINE)
        if other:
            for path in (other_path, *other_paths):
                self.write(path, 'const X: u8 = 2;\n')
        for path, content in base_files:
            self.write(path, content)
        if known_reds is not None:
            if known_reds[2] is None:
                self.git('rm', '-f', KNOWN_REDS)
            else:
                self.write(KNOWN_REDS, known_reds[2])
        self.commit('base')
        base = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.git('checkout', '-q', 'head')
        merge = self.git('merge', '--no-edit', 'base', check=False)
        self.assertEqual(merge.returncode, 1, merge.stderr)
        (self.job / 'request.json').write_text(json.dumps(
            {'version': 1, 'head': head, 'base': base}), encoding='utf-8')

    def run_merge(self, settled, left, truncated=False):
        head = self.git('rev-parse', 'HEAD').stdout
        base = self.git('rev-parse', 'MERGE_HEAD').stdout
        execution = subprocess.run(['fabrig/.temper/merge', 'merge', 'request.json'],
                                   cwd=self.job, env=self.env, capture_output=True)
        self.assertEqual(execution.returncode, 3 if left else 0, execution.stderr.decode())
        encoded = (self.job / 'result.json').read_bytes()
        self.assertLessEqual(len(encoded), 512)
        result = json.loads(encoded)
        if truncated:
            listed = sorted(result['settled'] + result['left'])
            paths = sorted(settled + left)
            self.assertTrue(listed)
            self.assertEqual(listed, paths[:len(listed)])
            self.assertEqual(result['settled'], [path for path in listed if path in settled])
            self.assertEqual(result['left'], [path for path in listed if path in left])
            self.assertEqual(result['more'], len(paths) - len(listed))
            self.assertGreater(result['more'], 0)
            next_path = paths[len(listed)]
            result['settled' if next_path in settled else 'left'].append(next_path)
            result['more'] -= 1
            self.assertGreater(len((json.dumps(result, separators=(',', ':')) + '\n').encode()), 512)
        else:
            expected = {'settled': settled}
            if left:
                expected['left'] = left
            self.assertEqual(result, expected)
        self.assertEqual(self.git('diff', '--name-only', '--diff-filter=U', '-z').stdout,
                         b''.join(os.fsencode(path) + b'\0' for path in left))
        self.assertEqual(self.git('rev-parse', 'HEAD').stdout, head)
        self.assertEqual(self.git('rev-parse', 'MERGE_HEAD').stdout, base)
        return execution

    def snapshot(self):
        paths = set(self.git('ls-files', '-z').stdout.split(b'\0')) - {b''}
        files = {name: (self.repository / os.fsdecode(name)).read_bytes()
                 for name in paths if (self.repository / os.fsdecode(name)).is_file()}
        return self.git('ls-files', '--stage', '-z').stdout, files

    def assert_recounted(self, path=BASELINE, comment='# Head baseline\n'):
        expected = counts(6, 6, comment).encode()
        self.assertEqual((self.repository / path).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{path}').stdout, expected)

    def recording_generator(self, outcome='write'):
        self.write(RECORDING, 'common recording\n')
        self.write(GENERATOR, 'merged generator\n')
        self.write('left-input', 'common left\n')
        self.write('right-input', 'common right\n')
        executable = self.job / 'bin/cargo'
        executable.parent.mkdir()
        executable.write_text(
            '#!/usr/bin/python3\n'
            'import json, os, sys\nfrom pathlib import Path\n'
            'root = Path.cwd()\n'
            'job = Path(os.environ["TEMPER_JOB"])\n'
            '(job / "generator-call.json").write_text(json.dumps({\n'
            '    "args": sys.argv[1:], "cwd": str(root),\n'
            '    "write": os.environ.get("SKEIN_CODEGEN_WRITE"),\n'
            '    "bundle": os.environ.get("TEMPER_INTERNAL_BUNDLE")}))\n'
            f'target = root / {RECORDING!r}\n'
            'assert not target.exists(), "a fresh output is required"\n'
            f'outcome = {outcome!r}\n'
            'if outcome == "write":\n'
            f'    target.write_text((root / {GENERATOR!r}).read_text() +\n'
            '                      (root / "left-input").read_text() +\n'
            '                      (root / "right-input").read_text())\n'
            'elif outcome == "fail":\n'
            '    target.write_text("partial recording")\n    sys.exit(1)\n'
            'elif outcome == "markers":\n'
            '    target.write_text("<<<<<<< head\\n=======\\n>>>>>>> base\\n")\n',
            encoding='utf-8')
        executable.chmod(0o755)
        self.env['PATH'] = str(executable.parent) + ':' + self.env['PATH']

    def assert_generator_called(self):
        call_path = self.job / 'generator-call.json'
        self.assertTrue(call_path.is_file(), 'the recording generator was invoked')
        call = json.loads(call_path.read_bytes())
        self.assertEqual(call, {
            'args': ['nextest', 'run', '-p', 'temper-playground-world', '--test', 'suite',
                     '-E', f'test(={WALK})'],
            'cwd': str(self.repository), 'write': '1', 'bundle': 'none'})

    def test_recording_is_generated_on_the_merged_tree_and_only_known_paths_are_staged(self):
        self.recording_generator()
        unknown = ['temper/tests/playground/tests/fixtures/other.log',
                   'temper/tests/real-loop/tests/fixtures/b0/llm.json',
                   'temper/tests/real-loop/tests/fixtures/b1/llm.json']
        for path in unknown:
            self.write(path, 'common fixture\n')
        self.conflict(other=True, other_path=RECORDING, other_paths=unknown,
                      head_files=[('left-input', 'head input\n')],
                      base_files=[('right-input', 'base input\n'),
                                  (GENERATOR, 'incoming generator\n')])
        previous = {path: ((self.repository / path).read_bytes(),
                          self.git('ls-files', '--stage', '--', path).stdout) for path in unknown}
        self.run_merge([BASELINE, RECORDING], sorted(unknown))
        expected = b'incoming generator\nhead input\nbase input\n'
        self.assertEqual((self.repository / RECORDING).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{RECORDING}').stdout, expected)
        self.assert_recounted()
        self.assert_generator_called()
        for path in unknown:
            self.assertEqual(((self.repository / path).read_bytes(),
                              self.git('ls-files', '--stage', '--', path).stdout), previous[path])

    def test_failed_recording_generator_restores_conflict_and_index(self):
        self.recording_generator('fail')
        self.conflict(baseline=False, other=True, other_path=RECORDING)
        previous = self.snapshot()
        self.run_merge([], [RECORDING])
        self.assert_generator_called()
        self.assertEqual(self.snapshot(), previous)

    def test_recording_generator_must_produce_a_fresh_file(self):
        self.recording_generator('no-output')
        self.conflict(baseline=False, other=True, other_path=RECORDING)
        previous = self.snapshot()
        self.run_merge([], [RECORDING])
        self.assert_generator_called()
        self.assertEqual(self.snapshot(), previous)

    def test_recording_generator_cannot_leave_conflict_markers(self):
        self.recording_generator('markers')
        self.conflict(baseline=False, other=True, other_path=RECORDING)
        previous = self.snapshot()
        self.run_merge([], [RECORDING])
        self.assert_generator_called()
        self.assertEqual(self.snapshot(), previous)

    def test_unavailable_recording_generator_leaves_conflict(self):
        self.recording_generator()
        (self.job / 'bin/cargo').unlink()
        self.conflict(baseline=False, other=True, other_path=RECORDING)
        previous = self.snapshot()
        execution = self.run_merge([], [RECORDING])
        self.assertIn(b'No such file or directory', execution.stderr)
        self.assertEqual(self.snapshot(), previous)

    def test_conflicted_recording_generator_is_not_run(self):
        self.recording_generator()
        self.conflict(baseline=False, other=True, other_path=RECORDING, other_paths=[GENERATOR])
        previous = self.snapshot()
        execution = self.run_merge([], sorted([RECORDING, GENERATOR]))
        self.assertIn(b'generator source is missing or unmerged', execution.stderr)
        self.assertFalse((self.job / 'generator-call.json').exists())
        self.assertEqual(self.snapshot(), previous)

    def test_missing_recording_generator_is_not_run(self):
        self.recording_generator()
        (self.repository / GENERATOR).unlink()
        self.conflict(baseline=False, other=True, other_path=RECORDING)
        previous = self.snapshot()
        execution = self.run_merge([], [RECORDING])
        self.assertIn(b'generator source is missing or unmerged', execution.stderr)
        self.assertFalse((self.job / 'generator-call.json').exists())
        self.assertEqual(self.snapshot(), previous)

    def test_baseline_alone_is_recounted_and_staged(self):
        self.assertTrue(os.access(ROOT / '.temper/merge', os.X_OK))
        self.conflict()
        self.run_merge([BASELINE], [])
        self.assert_recounted()

    def test_baseline_beside_other_conflict(self):
        self.conflict(other=True)
        conflict = (self.repository / 'temper/x.rs').read_bytes()
        stages = self.git('ls-files', '--stage', '--', 'temper/x.rs').stdout
        self.run_merge([BASELINE], ['temper/x.rs'])
        self.assert_recounted()
        self.assertEqual((self.repository / 'temper/x.rs').read_bytes(), conflict)
        self.assertEqual(self.git('ls-files', '--stage', '--', 'temper/x.rs').stdout, stages)

    def test_known_reds_are_merged_by_entries(self):
        common = ''.join(f'- test p t{number}\n' for number in range(60))
        head_entries = '- test p h1\n- test p h2\n'
        incoming_entries = '- test p i1\n- test p i2\n'
        byte_head = '- lint p c ' + 'é' * 2037 + '\n'
        byte_incoming = '- test p ' + 'é' * 2048 + '\n'
        self.assertEqual(len((byte_incoming + byte_head).encode()), 8192)
        rows = (
            ('additions and removals on both sides',
             '# Known reds\n- test a x\n- test b y\n',
             '# Head prose\n- test a x\n- test c z\n- lint p c f.rs\n',
             '# Known reds\nIncoming prose\n- test b y\n- test d w\n- test c z\n',
             '# Known reds\nIncoming prose\n- test d w\n- test c z\n- lint p c f.rs\n'),
            ('incoming prose alone', '# Known reds\n', '# Head prose\n',
             '# Incoming prose\n', '# Incoming prose\n'),
            ('shared addition once', '# Known reds\n', '# Head\n- test c z\n',
             '# Incoming\n- test c z\n', '# Incoming\n- test c z\n'),
            ('additions before trailing prose', 'P\n- test b y\nQ\n',
             'P\n- test b y\n- test e v\nQ\n',
             'P\n- test b y\n- test d w\nQ\n',
             'P\n- test b y\n- test d w\n- test e v\nQ\n'),
            ('additions replace removed last entry', 'P\n- test b y\n- test d w\nQ\n',
             'P\n- test b y\n- test e v\nQ\n',
             'P\n- test b y\n- test d w\nIncoming Q\n',
             'P\n- test b y\n- test e v\nIncoming Q\n'),
            ('no incoming entry and unterminated prose', '# Known reds\n',
             '# Head\n- test e v\n', '# Incoming', '# Incoming\n- test e v\n'),
            ('unterminated incoming entry', '# Known reds\n',
             '# Head\n- test e v\n', '# Incoming\n- test d w',
             '# Incoming\n- test d w\n- test e v\n'),
            ('trailing whitespace keys', '# Known reds\n- test a x \t\n',
             '# Head\n- test c z \t\n', '# Incoming\n- test a x\n- test c z\t\n',
             '# Incoming\n- test c z\t\n'),
            ('head additions deduplicated in order', '# Known reds\n',
             '# Head\n- lint p c f.rs \t\n- test c z\n- lint p c f.rs\n',
             '# Incoming\n', '# Incoming\n- lint p c f.rs\n- test c z\n'),
            ('both sides added the file', None, '# Head\n- test c z\n',
             '# Incoming\n- test d w\n', '# Incoming\n- test d w\n- test c z\n'),
            ('exactly 64 entry lines', common, common + head_entries,
             common + incoming_entries, common + incoming_entries + head_entries),
            ('exactly 8192 UTF-8 bytes', '', byte_head, byte_incoming,
             byte_incoming + byte_head),
            ('incoming lines kept byte for byte', '# Known reds\n',
             '# Head\n- test c z\n',
             '# Incoming\r\n - test a x\r\n- test missing\r\n'
             '- lint p code file with spaces.rs\r\nTrailing prose\r\n',
             '# Incoming\r\n - test a x\r\n- test missing\r\n'
             '- lint p code file with spaces.rs\r\n- test c z\nTrailing prose\r\n'),
        )
        for name, merge_base, head, incoming, expected in rows:
            with self.subTest(name=name):
                self.setUp()
                self.conflict(baseline=False, known_reds=(merge_base, head, incoming))
                self.run_merge([KNOWN_REDS], [])
                self.assertEqual((self.repository / KNOWN_REDS).read_bytes(), expected.encode())
                self.assertEqual(self.git('show', f':0:{KNOWN_REDS}').stdout, expected.encode())

    def test_known_reds_are_left_with_reason_and_unchanged(self):
        common = ''.join(f'- test p t{number}\n' for number in range(60))
        head_entries = '- test p h1\n- test p h2\n- test p h3\n'
        incoming_entries = '- test p i1\n- test p i2\n'
        entries_size = len((common + incoming_entries + head_entries).encode())
        # Two individually bounded sides produce a file over the UTF-8 byte cap.
        long_head = '- test p ' + 'é' * 2050 + '\n'
        long_incoming = '- lint p c ' + 'é' * 2050 + '\n'
        bytes_size = len((long_incoming + long_head).encode())
        duplicate_incoming = '- test p i\n' * 64
        duplicate_size = len((duplicate_incoming + '- test p h\n').encode())
        rows = (
            ('entry cap', common, common + head_entries, common + incoming_entries,
             f'the merged file holds 65 entries, {entries_size:,} bytes; '
             'the cap is 64 entries and 8,192 bytes', None),
            ('duplicate entry lines count toward cap', '', '- test p h\n',
             duplicate_incoming,
             f'the merged file holds 65 entries, {duplicate_size:,} bytes; '
             'the cap is 64 entries and 8,192 bytes', None),
            ('byte cap', '', long_head, long_incoming,
             f'the merged file holds 2 entries, {bytes_size:,} bytes; '
             'the cap is 64 entries and 8,192 bytes', None),
            ('head deletion', '# Known reds\n', None, '# Incoming\n',
             "missing on the head's side", None),
            ('incoming deletion', '# Known reds\n', '# Head\n', None,
             'missing on the incoming side', None),
            ('non-UTF-8 side', '# Known reds\n', b'# Head\n\xff\n', '# Incoming\n',
             "'utf-8' codec can't decode byte 0xff in position 7: invalid start byte", None),
            ('failed staging restores conflicted bytes', '# Known reds\n',
             '# Head\n- test c z\n', '# Incoming\n- test d w\n',
             "Command '['git', 'add', '--', '.temper/known-reds.md']' returned non-zero exit status 128.",
             'index lock'),
        )
        for name, merge_base, head, incoming, reason, setup in rows:
            with self.subTest(name=name):
                self.setUp()
                self.conflict(baseline=False, known_reds=(merge_base, head, incoming))
                if setup == 'index lock':
                    (self.repository / '.git/index.lock').touch()
                previous = self.snapshot()
                execution = self.run_merge([], [KNOWN_REDS])
                self.assertEqual(self.snapshot(), previous)
                self.assertEqual(execution.stderr.decode().splitlines(),
                                 [f'{KNOWN_REDS}: left: {reason}'])

    def test_known_reds_beside_other_conflict(self):
        self.conflict(baseline=False, other=True,
                      known_reds=('# Known reds\n', '# Head\n- test c z\n',
                                  '# Incoming\n- test d w\n'))
        conflict = (self.repository / 'temper/x.rs').read_bytes()
        stages = self.git('ls-files', '--stage', '--', 'temper/x.rs').stdout
        self.run_merge([KNOWN_REDS], ['temper/x.rs'])
        expected = b'# Incoming\n- test d w\n- test c z\n'
        self.assertEqual((self.repository / KNOWN_REDS).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{KNOWN_REDS}').stdout, expected)
        self.assertEqual((self.repository / 'temper/x.rs').read_bytes(), conflict)
        self.assertEqual(self.git('ls-files', '--stage', '--', 'temper/x.rs').stdout, stages)

    def test_other_conflict_leaves_work_tree_and_index_unchanged(self):
        self.conflict(baseline=False, other=True)
        previous = self.snapshot()
        self.run_merge([], ['temper/x.rs'])
        self.assertEqual(self.snapshot(), previous)

    def test_incoming_raise_is_left_unchanged(self):
        self.conflict(base_comment='# Incoming baseline\n# Raised: approved increase\n')
        previous = self.snapshot()
        self.run_merge([], [BASELINE])
        self.assertEqual(self.snapshot(), previous)

    def test_shared_raise_keeps_head_comments(self):
        head_comment = '# 2026\n# Head baseline\n# Raised: approved increase\n'
        self.conflict(head_comment=head_comment,
                      base_comment='# Incoming baseline\n# Raised: approved increase\n')
        self.run_merge([BASELINE], [])
        self.assert_recounted(comment=head_comment)

    def test_failed_recount_restores_conflict_and_leaves_index(self):
        self.write('temper/scripts/old_forms.py',
                   "from pathlib import Path\nimport sys\n"
                   "Path('scripts/old-forms.txt').write_text('partial recount')\n"
                   "sys.exit(1)\n")
        self.conflict()
        previous = self.snapshot()
        self.run_merge([], [BASELINE])
        self.assertEqual(self.snapshot(), previous)

    def test_recount_refuses_counts_above_both_sides(self):
        self.write('temper/uses.rs', 'const WORDS: &str = "' + 'max_out ' * 9 + '";\n')
        self.conflict()
        previous = self.snapshot()
        self.run_merge([], [BASELINE])
        self.assertEqual(self.snapshot(), previous)

    def test_all_four_tree_baselines_are_recounted(self):
        trees = ('skein', 'smith', 'jig', 'temper')
        for tree in trees[:-1]:
            # The same real counter and forms exercise each allowed path.
            self.write(f'{tree}/scripts/old-forms.txt', counts(12, 12))
            shutil.copy2(self.repository / 'temper/scripts/old_forms.py',
                         self.repository / tree / 'scripts/old_forms.py')
            shutil.copy2(self.repository / 'temper/uses.rs', self.repository / tree / 'uses.rs')
        self.conflict(trees=trees)
        paths = sorted(f'{tree}/scripts/old-forms.txt' for tree in trees)
        self.run_merge(paths, [])
        for path in paths:
            self.assert_recounted(path)

    def test_modify_delete_conflicts_are_left(self):
        self.conflict(delete='head')
        previous = self.snapshot()
        self.run_merge([], [BASELINE])
        self.assertEqual(self.snapshot(), previous)

    def test_delete_modify_conflicts_are_left(self):
        self.conflict(delete='base')
        previous = self.snapshot()
        self.run_merge([], [BASELINE])
        self.assertEqual(self.snapshot(), previous)

    def test_path_with_newline_is_left(self):
        path = 'temper/other\nfile.rs'
        self.write(path, 'const X: u8 = 0;\n')
        self.conflict(baseline=False, other=True, other_path=path)
        previous = self.snapshot()
        self.run_merge([], [path])
        self.assertEqual(self.snapshot(), previous)

    def test_many_conflicts_list_the_paths_that_fit_and_count_the_rest(self):
        paths = [f'temper/z{number:02d}-' + 'x' * 40 + '.rs' for number in range(20)]
        for path in paths:
            self.write(path, 'const X: u8 = 0;\n')
        self.conflict(other=True, other_path=paths[0], other_paths=paths[1:])
        self.run_merge([BASELINE], paths, truncated=True)
        self.assert_recounted()

    def test_one_path_larger_than_the_result_is_counted(self):
        path = 'temper/' + ('x' * 200 + '/') * 3 + 'file.rs'
        self.write(path, 'const X: u8 = 0;\n')
        self.conflict(baseline=False, other=True, other_path=path)
        previous = self.snapshot()
        result = subprocess.run(['fabrig/.temper/merge', 'merge', 'request.json'],
                                cwd=self.job, env=self.env, capture_output=True)
        self.assertEqual(result.returncode, 3, result.stderr.decode())
        self.assertEqual(json.loads((self.job / 'result.json').read_bytes()),
                         {'settled': [], 'left': [], 'more': 1})
        self.assertEqual(self.snapshot(), previous)


if __name__ == '__main__':
    unittest.main()
