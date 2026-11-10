"""Scratch close jobs for temper/docs/design/protocol/tooling.md, 4.2."""
import json
from pathlib import Path
import random
import shutil
import subprocess
import sys
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[2]
STATUS = 'tree/docs/design/a.status.md'
SEEDS = range(64)


class Close(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='close-')
        self.addCleanup(temporary.cleanup)
        self.job = Path(temporary.name)
        self.repository = self.job / 'fabrig'
        self.repository.mkdir()
        self.env = {'PATH': '/usr/bin:/bin', 'HOME': str(self.job),
                    'LANG': 'C.UTF-8', 'TEMPER_JOB': str(self.job)}
        self.git('init', '-q', '-b', 'head')
        self.git('config', 'user.name', 'Close test')
        self.git('config', 'user.email', 'close@example.test')
        self.git('config', 'commit.gpgsign', 'false')
        for name in ('.temper/close', 'tools/close/close.py'):
            target = self.repository / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / name, target)

    def git(self, *args):
        return subprocess.run(['git', *args], cwd=self.repository, env=self.env,
                              check=True, capture_output=True).stdout

    def write(self, path, data):
        target = self.repository / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def read(self, path=STATUS):
        return (self.repository / path).read_bytes()

    def commit(self):
        self.git('add', '.')
        self.git('commit', '-qm', 'status')
        return self.git('rev-parse', 'HEAD')

    def run_close(self, ends=None, request=None):
        if request is None:
            request = {'version': 1, 'goal': 7, 'changes': [
                {'task': task, 'landed': landed} for task, landed in sorted((ends or {}).items())]}
        path = self.job / 'request.json'
        path.write_text(json.dumps(request), encoding='utf-8')
        return subprocess.run([str(self.repository / '.temper/close'), 'close', str(path)],
                              cwd=self.job, env=self.env, capture_output=True)

    def result(self):
        return json.loads((self.job / 'result.json').read_bytes())

    def staged(self):
        return self.git('diff', '--cached', '--name-only', '-z').decode().split('\0')[:-1]

    def assert_success(self, result):
        self.assertEqual(result.returncode, 0, result.stderr.decode())

    def test_a_landed_changes_entries_are_deleted(self):
        before = b'# Status\r\n\r\nDate.\r\n\r\n'
        after = b'\r\n- **Departs:** 2: kept.\r\n  More.\r\n\xff\r\n'
        self.write(STATUS, before + b'- **Not yet:** 4.2: build it\r\n'
                   b'  with these lines\r\n  too. [in flight: change 9]\r\n' + after)
        # A final entry without a newline must not consume its predecessor's newline.
        other = 'docs/end.status.md'
        self.write(other, b'Header\n- **Not yet:** 1: end [in flight: change 9]')
        head = self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.read(), before + after)
        self.assertEqual(self.read(other), b'Header\n')
        self.assertEqual(self.staged(), [other, STATUS])
        self.assertEqual(self.git('rev-parse', 'HEAD'), head)
        self.assertEqual(self.result(), {'settled': [other, STATUS]})

    def test_an_unlanded_changes_tag_is_dropped(self):
        before = b'- **Not yet:** 4.2: still needed\r\n  more  '
        self.write(STATUS, before + b' [in flight: change 12]  \r\n')
        self.commit()
        self.assert_success(self.run_close({12: False}))
        self.assertEqual(self.read(), before + b'  \r\n')
        self.assertEqual(self.staged(), [STATUS])

    def test_a_tag_naming_no_change_fails_and_stages_nothing(self):
        self.write('docs/first.status.md', b'- **Not yet:** 1: known [in flight: change 9]\n')
        self.write(STATUS, b'# Status\n- **Not yet:** 4.2: unknown [in flight: change 40]\n')
        other = 'tree/docs/design/b.status.md'
        self.write(other, b'- **Not yet:** 1: unknown [in flight: some-name]\n')
        self.commit()
        index = self.git('write-tree')
        before = {path: self.read(path) for path in ('docs/first.status.md', STATUS, other)}
        result = self.run_close({9: True})
        self.assertEqual(result.returncode, 1)
        self.assertIn(f'{STATUS}:2:'.encode(), result.stderr)
        self.assertIn(b'change 40', result.stderr)
        self.assertIn(f'{other}:1:'.encode(), result.stderr)
        self.assertIn(b'some-name', result.stderr)
        self.assertEqual(self.git('write-tree'), index)
        self.assertEqual(self.staged(), [])
        self.assertEqual({path: self.read(path) for path in before}, before)

    def test_untagged_entries_and_other_files_are_untouched(self):
        contents = {
            STATUS: (b'# Status\n\nProse [in flight: change 40]\n\n'
                     b'- **Not yet:** 1: stays.\n  continuation\n'
                     b'- **Departs:** 2: stays [in flight: change 40]\n\n'
                     b'- **Not yet:** 2: [in flight: change 9] inside text\n'
                     b'- **Not yet:** 3: [in flight: change 9]\n  after the tag\n'
                     b'  \nHeading\n'),
            'outside.status.md': b'- **Not yet:** 1: outside [in flight: change 40]\n',
            'docs/other.md': b'- **Not yet:** 1: other [in flight: change 40]\n',
        }
        for path, data in contents.items():
            self.write(path, data)
        self.commit()
        self.write('docs/untracked.status.md', b'- **Not yet:** 1: untracked [in flight: change 40]')
        self.assert_success(self.run_close({9: True}))
        self.assertEqual({path: self.read(path) for path in contents}, contents)
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': []})

    def test_nothing_to_settle_is_success_with_nothing_staged(self):
        self.write(STATUS, b'# Status\n\n- **Not yet:** 1: stays.\n')
        self.commit()
        self.assert_success(self.run_close())
        self.assertEqual(self.result(), {'settled': []})
        self.assertEqual(self.staged(), [])

    def test_a_second_run_stages_nothing(self):
        self.write(STATUS, b'- **Not yet:** 1: built [in flight: change 9]\n'
                   b'- **Not yet:** 2: gap [in flight: change 12]\n')
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        self.assertEqual(self.read(), b'- **Not yet:** 2: gap\n')
        self.assertEqual(self.staged(), [STATUS])
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': []})

    def test_a_malformed_request_fails(self):
        self.write(STATUS, b'- **Not yet:** 1: built [in flight: change 9]\n')
        self.commit()
        valid = {'version': 1, 'goal': 7, 'changes': [{'task': 9, 'landed': True}]}
        invalid = [dict(valid, version=value) for value in (2, True, '1')]
        invalid += [dict(valid, goal=value) for value in (0, -1, '7', True)]
        invalid += [dict(valid, changes=[{'task': value, 'landed': True}])
                    for value in (0, -1, '9', True)]
        invalid += [dict(valid, changes=[{'task': 9, 'landed': value}]) for value in (1, 'true', None)]
        invalid += [dict(valid, changes=valid['changes'] * 2), dict(valid, extra=1),
                    dict(valid, changes=[{'task': 9, 'landed': True, 'extra': 1}]),
                    dict(valid, changes={}), {'version': 1, 'goal': 7}, []]
        before = self.read()
        for request in invalid:
            with self.subTest(request=request):
                result = self.run_close(request=request)
                self.assertEqual(result.returncode, 1, result.stderr.decode())
                self.assertEqual(self.staged(), [])
                self.assertEqual(self.read(), before)

    def test_the_result_is_cut_to_512_bytes_with_more(self):
        paths = sorted(f'docs/{number:02}-' + 'long-path-' * 8 + '.status.md' for number in range(15))
        for path in reversed(paths):
            self.write(path, b'- **Not yet:** 1: built [in flight: change 9]\n')
        self.commit()
        self.assert_success(self.run_close({9: True}))
        data = (self.job / 'result.json').read_bytes()
        self.assertLessEqual(len(data), 512)
        result = self.result()
        kept = len(result['settled'])
        self.assertGreater(kept, 0)
        self.assertLess(kept, len(paths))
        self.assertEqual(result, {'settled': paths[:kept], 'more': len(paths) - kept})
        self.assertEqual(self.staged(), paths)
        next_result = {'settled': paths[:kept + 1], 'more': len(paths) - kept - 1}
        self.assertGreater(len((json.dumps(next_result, separators=(',', ':')) + '\n').encode()), 512)

    def test_random_status_files_settle_by_their_changes_ends(self):
        started = time.monotonic()
        for seed in SEEDS:
            print(f'close seed {seed}', flush=True)
            rng = random.Random(seed)
            ends = {task: bool(rng.randrange(2)) for task in range(1, 21)
                    if seed % 2 == 0 or rng.randrange(4)}
            originals, expected = {}, {}
            unknown = False
            for number in range(4):
                path = f'docs/random-{number}.status.md'
                original, model = [], []
                newline = rng.choice((b'\n', b'\r\n'))
                for entry in range(24):
                    kind = rng.choice(('prose', 'untagged', 'tagged', 'departs'))
                    if kind == 'prose':
                        chunk = f'Prose {rng.getrandbits(32)}.'.encode() + newline + newline
                        original.append(chunk)
                        model.append(chunk)
                        continue
                    label = b'Departs' if kind == 'departs' else b'Not yet'
                    lines = [b'- **' + label + b':** 4.2: ' + str(entry).encode()]
                    lines += [f'  continuation {rng.getrandbits(32)}'.encode()
                              for _ in range(rng.randrange(4))]
                    bare = newline.join(lines)
                    task = rng.randint(1, 20)
                    tagged = bare + f' [in flight: change {task}]'.encode()
                    suffix = newline + newline
                    original.append((tagged if kind in ('tagged', 'departs') else bare) + suffix)
                    if kind != 'tagged':
                        model.append(original[-1])
                    elif task not in ends:
                        unknown = True
                    elif ends[task]:
                        model.append(newline)
                    else:
                        model.append(bare + suffix)
                originals[path] = b''.join(original)
                expected[path] = b''.join(model)
                self.write(path, originals[path])
            self.commit()
            result = self.run_close(ends)
            with self.subTest(seed=seed):
                if unknown:
                    self.assertEqual(result.returncode, 1, result.stderr.decode())
                    self.assertEqual({path: self.read(path) for path in originals}, originals)
                    self.assertEqual(self.staged(), [])
                else:
                    self.assert_success(result)
                    self.assertEqual({path: self.read(path) for path in expected}, expected)
                    changed = sorted(path for path in expected if expected[path] != originals[path])
                    self.assertEqual(self.staged(), changed)
                    self.assertEqual(self.result(), {'settled': changed})
        print(f'close fuzzy: {len(SEEDS)} seeds in {time.monotonic() - started:.3f}s', flush=True)


if __name__ == '__main__':
    if '--seed' in sys.argv:
        index = sys.argv.index('--seed')
        SEEDS = [int(sys.argv[index + 1])]
        del sys.argv[index:index + 2]
        if len(sys.argv) == 1:
            sys.argv.append('Close.test_random_status_files_settle_by_their_changes_ends')
    unittest.main()
