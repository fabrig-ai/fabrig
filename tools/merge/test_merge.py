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
        self.home = self.job / 'home'
        self.home.mkdir()
        self.env = {'PATH': '/usr/bin:/bin', 'HOME': str(self.job),
                    'LANG': 'C.UTF-8', 'TEMPER_JOB': str(self.job)}
        self.git('init', '-q', '-b', 'head')
        self.git('config', 'user.name', 'Merge test')
        self.git('config', 'user.email', 'merge@example.test')
        self.git('config', 'commit.gpgsign', 'false')
        for name in ('.temper/merge', 'tools/merge/merge.py',
                     'tools/old_forms/old_forms.py', 'temper/scripts/old_forms.py'):
            target = self.home / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, target)
        self.write(BASELINE, counts(12, 12))
        self.write('temper/uses.rs', 'const WORDS: &str = "' +
                   ' '.join(['max_out'] * 6 + ['JournalRoom'] * 6) + '";\n')
        self.write('temper/x.rs', 'const X: u8 = 0;\n')

    def git(self, *args, check=True):
        return subprocess.run(['git', *args], cwd=self.home, env=self.env,
                              check=check, capture_output=True)

    def write(self, name, content):
        path = self.home / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding='utf-8')

    def commit(self, message):
        self.git('add', '.')
        self.git('commit', '-qm', message)

    def conflict(self, baseline=True, other=False, head_comment='# Head baseline\n',
                 base_comment='# Incoming baseline\n', trees=('temper',), delete=None,
                 other_path='temper/x.rs'):
        self.commit('common')
        self.git('branch', 'base')
        if baseline:
            for tree in trees:
                self.write(f'{tree}/scripts/old-forms.txt', counts(8, 4, head_comment))
            if delete == 'head':
                self.git('rm', '-f', BASELINE)
        if other:
            self.write(other_path, 'const X: u8 = 1;\n')
        self.commit('head')
        head = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.git('checkout', '-q', 'base')
        if baseline:
            for tree in trees:
                self.write(f'{tree}/scripts/old-forms.txt', counts(4, 8, base_comment))
            if delete == 'base':
                self.git('rm', '-f', BASELINE)
        if other:
            self.write(other_path, 'const X: u8 = 2;\n')
        self.commit('base')
        base = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.git('checkout', '-q', 'head')
        merge = self.git('merge', '--no-edit', 'base', check=False)
        self.assertEqual(merge.returncode, 1, merge.stderr)
        (self.job / 'request.json').write_text(json.dumps(
            {'version': 1, 'head': head, 'base': base}), encoding='utf-8')

    def run_merge(self, settled, left):
        head = self.git('rev-parse', 'HEAD').stdout
        base = self.git('rev-parse', 'MERGE_HEAD').stdout
        result = subprocess.run(['home/.temper/merge', 'merge', 'request.json'],
                                cwd=self.job, env=self.env, capture_output=True)
        self.assertEqual(result.returncode, 3 if left else 0, result.stderr.decode())
        expected = {'settled': settled}
        if left:
            expected['left'] = left
        self.assertEqual(json.loads((self.job / 'result.json').read_bytes()), expected)
        self.assertEqual(self.git('diff', '--name-only', '--diff-filter=U', '-z').stdout,
                         b''.join(os.fsencode(path) + b'\0' for path in left))
        self.assertEqual(self.git('rev-parse', 'HEAD').stdout, head)
        self.assertEqual(self.git('rev-parse', 'MERGE_HEAD').stdout, base)

    def snapshot(self):
        paths = set(self.git('ls-files', '-z').stdout.split(b'\0')) - {b''}
        files = {name: (self.home / os.fsdecode(name)).read_bytes()
                 for name in paths if (self.home / os.fsdecode(name)).is_file()}
        return self.git('ls-files', '--stage', '-z').stdout, files

    def assert_recounted(self, path=BASELINE, comment='# Head baseline\n'):
        expected = counts(6, 6, comment).encode()
        self.assertEqual((self.home / path).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{path}').stdout, expected)

    def test_baseline_alone_is_recounted_and_staged(self):
        self.assertTrue(os.access(ROOT / '.temper/merge', os.X_OK))
        self.conflict()
        self.run_merge([BASELINE], [])
        self.assert_recounted()

    def test_baseline_beside_other_conflict(self):
        self.conflict(other=True)
        conflict = (self.home / 'temper/x.rs').read_bytes()
        stages = self.git('ls-files', '--stage', '--', 'temper/x.rs').stdout
        self.run_merge([BASELINE], ['temper/x.rs'])
        self.assert_recounted()
        self.assertEqual((self.home / 'temper/x.rs').read_bytes(), conflict)
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
            shutil.copy2(self.home / 'temper/scripts/old_forms.py',
                         self.home / tree / 'scripts/old_forms.py')
            shutil.copy2(self.home / 'temper/uses.rs', self.home / tree / 'uses.rs')
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


if __name__ == '__main__':
    unittest.main()
