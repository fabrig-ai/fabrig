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
        path.write_text(content, encoding='utf-8')

    def commit(self, message):
        self.git('add', '.')
        self.git('commit', '-qm', message)

    def conflict(self, baseline=True, other=False, head_comment='# Head baseline\n',
                 base_comment='# Incoming baseline\n', trees=('temper',), delete=None,
                 other_path='temper/x.rs', other_paths=()):
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
        result = subprocess.run(['fabrig/.temper/merge', 'merge', 'request.json'],
                                cwd=self.job, env=self.env, capture_output=True)
        self.assertEqual(result.returncode, 3 if left else 0, result.stderr.decode())
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

    def snapshot(self):
        paths = set(self.git('ls-files', '-z').stdout.split(b'\0')) - {b''}
        files = {name: (self.repository / os.fsdecode(name)).read_bytes()
                 for name in paths if (self.repository / os.fsdecode(name)).is_file()}
        return self.git('ls-files', '--stage', '-z').stdout, files

    def assert_recounted(self, path=BASELINE, comment='# Head baseline\n'):
        expected = counts(6, 6, comment).encode()
        self.assertEqual((self.repository / path).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{path}').stdout, expected)

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
