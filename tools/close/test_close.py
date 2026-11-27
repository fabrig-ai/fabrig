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
OPENING = b'# x: status\n\n2026-11-20. Where the code lags.\n'
BUILT = b'- **Not yet:** 3: built [in flight: change 9].\n'
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

    def test_tagged_entries_outside_docs_settle(self):
        paths = ['outside.status.md', 'tools/x/design.status.md']
        stays = b'- **Not yet:** 4: stays.\n'
        gap = b'- **Not yet:** 5: gap'
        for path in paths:
            self.write(path, BUILT + stays + gap + b' [in flight: change 12].\n')
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        for path in paths:
            self.assertEqual(self.read(path), stays + gap + b'.\n', path)
        self.assertEqual(self.staged(), paths)
        self.assertEqual(self.result(), {'settled': paths})

    def test_a_file_whose_last_entry_settles_reads_no_gap(self):
        multiline = b'\n \t\n' + OPENING + b'This is still the dated paragraph.  \n'
        nested = (b'- **Not yet:**\n'
                  b'  - 3: built [in flight: change 9].\n'
                  b'  - 4: built too [in flight: change 10].\n')
        cases = (
            (STATUS, OPENING, BUILT, b'\n'),
            ('docs/windows.status.md', OPENING, BUILT + b'\n \t\n', b'\r\n'),
            ('tools/x/nested.status.md', multiline, nested, b'\n'),
        )
        for path, opening, entries, newline in cases:
            self.write(path, (opening + b'\n' + entries).replace(b'\n', newline))
        self.commit()
        self.assert_success(self.run_close({9: True, 10: True}))
        for path, opening, _, newline in cases:
            with self.subTest(path=path):
                self.assertEqual(self.read(path), opening.replace(b'\n', newline)
                                 + newline + b'No gap.' + newline)
        paths = sorted(path for path, _, _, _ in cases)
        self.assertEqual(self.staged(), paths)
        self.assertEqual(self.result(), {'settled': paths})
        self.commit()
        self.assert_success(self.run_close({9: True, 10: True}))
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': []})

    def test_text_left_after_the_opening_is_preserved(self):
        remnants = {
            'docs/untagged.status.md': b'- **Not yet:** 4: stays.\n',
            'tools/x/departs.status.md': b'- **Departs:** 5: stays.\n',
            'tools/x/prose.status.md': b'Prose after the opening.\n',
        }
        foreign = ['docs/foreign.status.md', 'foreign.status.md']
        foreign_text = OPENING + b'\n- **Not yet:** 6: other [in flight: change 40].\n'
        for path, remnant in remnants.items():
            self.write(path, OPENING + b'\n' + BUILT + remnant)
        for path in foreign:
            self.write(path, foreign_text)
        self.commit()
        self.assert_success(self.run_close({9: True}))
        for path, remnant in remnants.items():
            self.assertEqual(self.read(path), OPENING + b'\n' + remnant, path)
        for path in foreign:
            self.assertEqual(self.read(path), foreign_text, path)
        self.assertEqual(self.staged(), sorted(remnants))
        self.assertEqual(self.result(), {'settled': sorted(remnants), 'kept': [
            f'{path}:5: [in flight: change 40]' for path in foreign]})

    def test_an_existing_no_gap_is_kept(self):
        before = OPENING + b'\n \t\nNo gap.\n\n\n'
        path = 'tools/x/done.status.md'
        self.write(path, before + BUILT)
        self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.read(path), before)
        self.assertEqual(self.staged(), [path])
        self.assertEqual(self.result(), {'settled': [path]})

    def test_missing_openings_and_unsettled_files_gain_nothing(self):
        without_opening = {
            'no-heading.status.md': b'Not a heading.\n\n2026-11-20. Where the code lags.\n\n',
            'tools/x/no-date.status.md': b'# x: status\n\nWhere the code lags.\n\n',
        }
        for path, before in without_opening.items():
            self.write(path, before + BUILT)
        blank = OPENING + b'\n \t\n'
        self.write(STATUS, blank)
        self.commit()
        self.assert_success(self.run_close({9: True}))
        for path, before in without_opening.items():
            self.assertEqual(self.read(path), before, path)
        self.assertEqual(self.read(), blank)
        self.assertEqual(self.staged(), sorted(without_opening))
        self.assertEqual(self.result(), {'settled': sorted(without_opening)})

    def test_a_wrapped_tag_settles_as_a_one_line_one(self):
        wraps = (
            (b'{nl}  [in flight: change {task}]', b'\n'),
            (b' [in{nl}  flight: change {task}]', b'\n'),
            (b' [in flight: \t{nl}  change {task}]', b'\n'),
            (b' [in flight: change{nl}    {task}]', b'\n'),
            (b' [in flight: change \t{nl}    {task}]', b'\r\n'),
        )
        expected = {}
        for index, (wrap, newline) in enumerate(wraps):
            path = f'docs/wrap-{index}.status.md'
            before = b'# Status' + newline + newline
            after = newline + b'- **Departs:** 3: stays.' + newline
            kept_prefix = b'- **Not yet:** 2: gap (`x.rs`)'
            landed = (b'- **Not yet:** 1: built' + wrap.replace(b'{nl}', newline)
                      .replace(b'{task}', b'9') + b'.' + newline)
            unlanded = (kept_prefix + wrap.replace(b'{nl}', newline).replace(b'{task}', b'12')
                        + b'.!? \t' + newline)
            self.write(path, before + landed + unlanded + after)
            expected[path] = before + kept_prefix + b'.!? \t' + newline + after
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        for path, content in expected.items():
            with self.subTest(path=path):
                self.assertEqual(self.read(path), content)
        self.assertEqual(self.staged(), list(expected))
        self.assertEqual(self.result(), {'settled': list(expected)})

    def test_wrapped_tags_naming_no_change_are_kept(self):
        before = (b'# Status\r\n\r\n- **Not yet:** 1: other\r\n'
                  b'  Its continuation.\r\n'
                  b'  [in flight: \t\r\n  change \t\r\n    99].\r\n'
                  b'- **Not yet:** 2: outside [in\r\n'
                  b'  flight: some\r\n  name];\r\n')
        self.write(STATUS, before)
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        self.assertEqual(self.read(), before)
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': [], 'kept': [
            f'{STATUS}:5: [in flight: change 99]',
            f'{STATUS}:8: [in flight: some name]']})

    def test_wrapped_nested_tags_settle_only_their_items(self):
        label = b'- **Not yet:**\n'
        built = b'  - 1: built. [in\n    flight: change 9]\n'
        sibling = b'  - 2: an untagged gap.\n    Its continuation.\n'
        self.write(STATUS, label + built + sibling)
        empty = 'docs/empty.status.md'
        self.write(empty, b'# Empty\n\n' + label + built
                   + b'  - 2: built too.\n    [in flight: change\n    9].')
        self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.read(), label + sibling)
        self.assertEqual(self.read(empty), b'# Empty\n\n')
        self.assertEqual(self.staged(), [empty, STATUS])
        self.assertEqual(self.result(), {'settled': [empty, STATUS]})
        self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': []})

    def test_a_bare_not_yet_label_settles_each_nested_item(self):
        before = (b'# Status\r\n\r\n- **Not yet:**\r\n'
                  b'  - 1: built\r\n    over two lines. [in flight: change 9]\r\n'
                  b'  - 2: still needed [in flight: change 12]\r\n'
                  b'  - 3: built too. [in flight: change 9]\r\n')
        self.write(STATUS, before)
        empty = 'docs/empty.status.md'
        self.write(empty, b'# Empty\n\n- **Not yet:**\n  - 1: built. [in flight: change 9]\n')
        self.commit()
        self.assert_success(self.run_close({9: True, 12: False}))
        self.assertEqual(self.read(), b'# Status\r\n\r\n- **Not yet:**\r\n'
                         b'  - 2: still needed\r\n')
        self.assertEqual(self.read(empty), b'# Empty\n\n')
        self.assertEqual(self.staged(), [empty, STATUS])

    def test_a_landed_nested_item_never_deletes_an_untagged_sibling(self):
        kept = (b'# Status\n\n- **Not yet:**\n'
                b'  - 1: an untagged gap.\n    Its continuation.\n')
        self.write(STATUS, kept + b'  - 2: built. [in flight: change 9]\n')
        self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.read(), kept)
        self.assertEqual(self.staged(), [STATUS])

    def test_a_tag_naming_no_change_is_kept(self):
        first = 'docs/first.status.md'
        self.write(first, b'- **Not yet:** 1: known [in flight: change 9]\n')
        self.write(STATUS, b'# Status\n- **Not yet:** 4.2: other [in flight: change 40]\n')
        other = 'tree/docs/design/b.status.md'
        self.write(other, b'- **Not yet:** 1: named [in flight: some-name]\n')
        self.commit()
        before = {path: self.read(path) for path in (STATUS, other)}
        self.assert_success(self.run_close({9: True}))
        self.assertEqual({path: self.read(path) for path in before}, before)
        self.assertEqual(self.read(first), b'')
        self.assertEqual(self.staged(), [first])
        self.assertEqual(self.result(), {'settled': [first], 'kept': [
            f'{STATUS}:2: [in flight: change 40]',
            f'{other}:1: [in flight: some-name]']})
        self.assertEqual((self.job / 'result.json').read_bytes(),
                         b'{"settled":["docs/first.status.md"],"kept":['
                         b'"tree/docs/design/a.status.md:2: [in flight: change 40]",'
                         b'"tree/docs/design/b.status.md:1: [in flight: some-name]"]}\n')

    def test_sentence_punctuation_after_a_tag_is_kept(self):
        for punctuation, spaces in ((b'.', b''), (b';', b'  '), (b',', b'\t'),
                                    (b'?!', b'  '), (b'...', b' \t'), (b':', b' ')):
            with self.subTest(punctuation=punctuation):
                suffix = punctuation + spaces
                other = b'- **Not yet:** 3: other [in flight: change 40].\n'
                self.write(STATUS, b'- **Not yet:** 1: built [in flight: change 9]'
                           + suffix + b'\n- **Not yet:** 2: gap [in flight: change 12]'
                           + suffix + b'\r\n' + other)
                self.commit()
                self.assert_success(self.run_close({9: True, 12: False}))
                self.assertEqual(self.read(), b'- **Not yet:** 2: gap' + suffix + b'\r\n' + other)
                self.assertEqual(self.staged(), [STATUS])
                self.assertEqual(self.result(), {'settled': [STATUS], 'kept': [
                    f'{STATUS}:3: [in flight: change 40]']})
                self.commit()
                self.assert_success(self.run_close({9: True, 12: False}))
                self.assertEqual(self.staged(), [])
                self.assertEqual(self.result(), {'settled': [], 'kept': [
                    f'{STATUS}:2: [in flight: change 40]']})

    def test_text_after_a_tag_is_not_a_tag(self):
        before = (b'- **Not yet:** 1: gap [in flight: change 9] .\n'
                  b'- **Not yet:** 2: gap [in flight: change 9]).\n'
                  b'- **Not yet:** 3: gap [in flight: change 9] more\n'
                  b'- **Not yet:** 4: gap [in flight: change 9].\n  continuation\n')
        self.write(STATUS, before)
        self.commit()
        self.assert_success(self.run_close({9: True}))
        self.assertEqual(self.read(), before)
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': []})

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
        untracked = 'docs/untracked.status.md'
        self.write(untracked, BUILT)
        self.assert_success(self.run_close({9: True}))
        self.assertEqual({path: self.read(path) for path in contents}, contents)
        self.assertEqual(self.read(untracked), BUILT)
        self.assertEqual(self.staged(), [])
        self.assertEqual(self.result(), {'settled': [], 'kept': [
            'outside.status.md:1: [in flight: change 40]']})

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

    def assert_bounded_result(self, settled, kept):
        data = (self.job / 'result.json').read_bytes()
        self.assertLessEqual(len(data), 512)
        result = self.result()
        path_count = len(result['settled'])
        tag_count = len(result.get('kept', []))
        self.assertLessEqual(path_count, len(settled))
        self.assertLessEqual(tag_count, len(kept))
        if path_count < len(settled):
            self.assertEqual(tag_count, 0)

        def model(paths, tags):
            value = {'settled': settled[:paths]}
            if kept:
                value['kept'] = kept[:tags]
            if paths < len(settled):
                value['more'] = len(settled) - paths
            if tags < len(kept):
                value['kept_more'] = len(kept) - tags
            return value

        self.assertEqual(result, model(path_count, tag_count))
        if path_count < len(settled):
            next_result = model(path_count + 1, 0)
        elif tag_count < len(kept):
            next_result = model(path_count, tag_count + 1)
        else:
            return
        self.assertGreater(len((json.dumps(next_result, ensure_ascii=True,
                                           separators=(',', ':')) + '\n').encode()), 512)

    def test_kept_tags_are_cut_after_all_settled_paths(self):
        settled = ['docs/first.status.md', 'docs/second.status.md']
        paths = sorted(f'docs/{number:02}-' + 'long-path-' * 8 + '.status.md'
                       for number in range(15))
        for path in reversed(paths):
            self.write(path, b'- **Not yet:** 1: other [in flight: change 40]\n')
        for path in settled:
            self.write(path, b'- **Not yet:** 1: built [in flight: change 9]\n')
        self.commit()
        self.assert_success(self.run_close({9: True}))
        kept = [f'{path}:1: [in flight: change 40]' for path in paths]
        self.assert_bounded_result(settled, kept)
        result = self.result()
        self.assertEqual(result['settled'], settled)
        self.assertNotIn('more', result)
        self.assertGreater(len(result['kept']), 0)
        self.assertGreater(result['kept_more'], 0)
        self.assertEqual(self.staged(), settled)
        for path in paths:
            self.assertEqual(self.read(path), b'- **Not yet:** 1: other [in flight: change 40]\n')

    def test_settled_paths_overflow_before_any_kept_tag(self):
        paths = sorted(f'docs/{number:02}-' + 'long-path-' * 8 + '.status.md'
                       for number in range(15))
        for path in reversed(paths):
            self.write(path, b'- **Not yet:** 1: built [in flight: change 9]\n')
        other = b'- **Not yet:** 2: other [in flight: change 40]\n'
        self.write(STATUS, other * 3)
        self.commit()
        self.assert_success(self.run_close({9: True}))
        kept = [f'{STATUS}:{line}: [in flight: change 40]' for line in range(1, 4)]
        self.assert_bounded_result(paths, kept)
        result = self.result()
        self.assertGreater(len(result['settled']), 0)
        self.assertGreater(result['more'], 0)
        self.assertEqual(result['kept'], [])
        self.assertEqual(result['kept_more'], len(kept))
        self.assertEqual(self.read(), other * 3)
        self.assertEqual(self.staged(), paths)

    def test_random_status_files_settle_by_their_changes_ends(self):
        started = time.monotonic()
        paths = ('docs/random-0.status.md', 'docs/random-1.status.md', 'random-2.status.md',
                 'tools/x/random-3.status.md', 'tools/x/random-4.status.md')
        for seed in SEEDS:
            print(f'close seed {seed}', flush=True)
            rng = random.Random(seed)
            ends = {task: bool(rng.randrange(2)) for task in range(1, 21)
                    if seed % 2 == 0 or rng.randrange(4)}
            originals, expected = {}, {}
            kept = []
            landed = [task for task, did_land in ends.items() if did_land]
            for number, path in enumerate(paths):
                original, model = [], []
                newline = rng.choice((b'\n', b'\r\n'))
                all_tagged = number == len(paths) - 1
                prefix = b''
                if all_tagged or (seed + number) % 2:
                    prefix = OPENING.replace(b'\n', newline) + newline
                for entry in range(24):
                    kind = ('tagged' if all_tagged else
                            rng.choice(('prose', 'untagged', 'tagged', 'departs')))
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
                    task = rng.choice(landed) if all_tagged and landed else rng.randint(1, 20)
                    punctuation = bytes(rng.choice(b'.,;:!?') for _ in range(rng.randrange(1, 5))) \
                        if rng.randrange(2) else b''
                    spaces = rng.choice((b'', b' ', b'  ', b'\t'))
                    tag = []
                    for word in (b'[in', b'flight:', b'change', f'{task}]'.encode()):
                        separator = b' '
                        if rng.randrange(2):
                            separator = (rng.choice((b'', b' ', b' \t')) + newline
                                         + rng.choice((b'  ', b'    ', b'  \t')))
                        tag.append(separator + word)
                    tagged = bare + b''.join(tag) + punctuation + spaces
                    suffix = newline + newline
                    original.append((tagged if kind in ('tagged', 'departs') else bare) + suffix)
                    if kind != 'tagged':
                        model.append(original[-1])
                    elif task not in ends:
                        line = (prefix.count(b'\n')
                                + sum(chunk.count(b'\n') for chunk in original[:-1])
                                + len(lines) + tag[0].count(b'\n'))
                        kept.append(f'{path}:{line}: [in flight: change {task}]')
                        model.append(original[-1])
                    elif ends[task]:
                        model.append(newline)
                    else:
                        model.append(bare + punctuation + spaces + suffix)
                body, settled_body = b''.join(original), b''.join(model)
                originals[path] = prefix + body
                expected[path] = prefix + settled_body
                if prefix and body != settled_body and not settled_body.strip():
                    expected[path] = prefix + b'No gap.' + newline
                self.write(path, originals[path])
            self.commit()
            result = self.run_close(ends)
            with self.subTest(seed=seed):
                self.assert_success(result)
                self.assertEqual({path: self.read(path) for path in expected}, expected)
                changed = sorted(path for path in expected if expected[path] != originals[path])
                self.assertEqual(self.staged(), changed)
                self.assert_bounded_result(changed, kept)
        print(f'close fuzzy: {len(SEEDS)} seeds in {time.monotonic() - started:.3f}s', flush=True)


if __name__ == '__main__':
    if '--seed' in sys.argv:
        index = sys.argv.index('--seed')
        SEEDS = [int(sys.argv[index + 1])]
        del sys.argv[index:index + 2]
        if len(sys.argv) == 1:
            sys.argv.append('Close.test_random_status_files_settle_by_their_changes_ends')
    unittest.main()
