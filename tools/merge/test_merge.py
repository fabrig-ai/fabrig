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
RECORDING = 'temper/tests/playground/tests/fixtures/story-a.log'
GENERATOR = 'temper/tests/playground/tests/suite/walk.rs'
WALK = 'walk::story_a_walks_from_the_welcome_to_the_csv_change_landed'
CODEC = 'a/src/generated/note.rs'
SCHEMAS = ['b/schema/note-v1.schema', 'b/schema/note-v1-part1.schema']
CODEC_SOURCES = ['skein/testing/skein-codegen/src/lib.rs', 'skein/crates/skein-codec/src/schema.rs']
CODEC_LIST = 'tools/merge/codecs.txt'
LEAN_MARK = 'b/schema/lean'
JOURNEYS = 'temper/docs/design/client/web/journeys/'


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
        return self.git('rev-parse', 'HEAD').stdout.decode().strip()

    def conflict(self, baseline=True, other=False, head_comment='# Head baseline\n',
                 base_comment='# Incoming baseline\n', trees=('temper',), delete=None,
                 other_path='temper/x.rs', other_paths=(), head_files=(), base_files=()):
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
        self.commit('base')
        base = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.merge_conflict(head, base)

    def merge_conflict(self, head, base):
        self.git('checkout', '-q', 'head')
        merge = self.git('merge', '--no-edit', base, check=False)
        self.assertEqual(merge.returncode, 1, merge.stderr)
        (self.job / 'request.json').write_text(json.dumps(
            {'version': 1, 'head': head, 'base': base}), encoding='utf-8')

    def run_merge(self, settled, left, truncated=False):
        left = [{'path': item, 'kind': 'modified'} if isinstance(item, str) else item
                for item in left]
        paths = [item['path'] for item in left]
        previous = self.snapshot() if not settled else None
        left_before = {path: ((self.repository / path).read_bytes()
                              if (self.repository / path).is_file() else None,
                              self.git('--literal-pathspecs', 'ls-files', '--stage', '-z',
                                       '--', path).stdout) for path in paths}
        head = self.git('rev-parse', 'HEAD').stdout
        base = self.git('rev-parse', 'MERGE_HEAD').stdout
        execution = subprocess.run(['fabrig/.temper/merge', 'merge', 'request.json'],
                                   cwd=self.job, env=self.env, capture_output=True)
        self.assertEqual(execution.returncode, 3 if left else 0, execution.stderr.decode())
        encoded = (self.job / 'result.json').read_bytes()
        self.assertLessEqual(len(encoded), 512)
        result = json.loads(encoded)
        if truncated:
            listed_left = result.get('left', [])
            self.assertEqual(listed_left, left[:len(listed_left)])
            self.assertEqual(result['settled'], settled[:len(result['settled'])])
            more = {'left': len(left) - len(listed_left),
                    'settled': len(settled) - len(result['settled']),
                    'read': 'git status --short; git log --name-status '
                            '--diff-filter=DR HEAD...MERGE_HEAD'}
            self.assertEqual(result['more'], more)
            self.assertGreater(more['left'] + more['settled'], 0)
            expected = {'settled': result['settled']}
            if listed_left:
                expected['left'] = listed_left
            expected['more'] = more
            for group, entries in (('left', left), ('settled', settled)):
                kept = len(result.get(group, []))
                if kept < len(entries):
                    candidate = json.loads(encoded)
                    candidate.setdefault(group, []).append(entries[kept])
                    candidate['more'][group] -= 1
                    if not (candidate['more']['left'] or candidate['more']['settled']):
                        del candidate['more']
                    self.assertGreater(len(self.encode(candidate)), 512)
        else:
            expected = {'settled': settled}
            if left:
                expected['left'] = left
        self.assertEqual(encoded, self.encode(expected))
        self.assertEqual(self.git('diff', '--name-only', '--diff-filter=U', '-z').stdout,
                         b''.join(os.fsencode(path) + b'\0' for path in paths))
        self.assertEqual(self.git('rev-parse', 'HEAD').stdout, head)
        self.assertEqual(self.git('rev-parse', 'MERGE_HEAD').stdout, base)
        for path in paths:
            self.assertEqual(((self.repository / path).read_bytes()
                              if (self.repository / path).is_file() else None,
                              self.git('--literal-pathspecs', 'ls-files', '--stage', '-z',
                                       '--', path).stdout), left_before[path])
        if previous is not None:
            self.assertEqual(self.snapshot(), previous)
        return execution

    @staticmethod
    def encode(result):
        return (json.dumps(result, ensure_ascii=True, separators=(',', ':')) + '\n').encode('ascii')

    def assert_stages(self, expected):
        stages = {}
        for entry in self.git('ls-files', '--unmerged', '-z').stdout.split(b'\0'):
            if entry:
                metadata, path = entry.split(b'\t', 1)
                stages.setdefault(os.fsdecode(path), set()).add(int(metadata.split()[2]))
        self.assertEqual(stages, expected)

    def snapshot(self):
        paths = set(self.git('ls-files', '-z').stdout.split(b'\0')) - {b''}
        files = {name: (self.repository / os.fsdecode(name)).read_bytes()
                 for name in paths if (self.repository / os.fsdecode(name)).is_file()}
        return self.git('ls-files', '--stage', '-z').stdout, files

    def assert_recounted(self, path=BASELINE, comment='# Head baseline\n'):
        expected = counts(6, 6, comment).encode()
        self.assertEqual((self.repository / path).read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{path}').stdout, expected)

    def forbid_journey_generation(self):
        executable = self.job / 'bin/cargo'
        executable.parent.mkdir()
        executable.write_text('#!/bin/sh\ntouch "$TEMPER_JOB/unexpected-cargo"\nexit 1\n',
                              encoding='utf-8')
        executable.chmod(0o755)
        self.env['PATH'] = str(executable.parent) + ':' + self.env['PATH']
        self.addCleanup(lambda: self.assertFalse((self.job / 'unexpected-cargo').exists()))

    def assert_base_entry(self, path):
        base = self.git('rev-parse', 'MERGE_HEAD').stdout.decode().strip()
        self.assertEqual((self.repository / path).read_bytes(),
                         self.git('show', f'{base}:{path}').stdout)
        self.assertEqual(self.git('show', f':0:{path}').stdout,
                         self.git('show', f'{base}:{path}').stdout)
        entry = self.git('ls-tree', base, '--', path).stdout.split()[0]
        self.assertEqual(self.git('ls-files', '--stage', '--', path).stdout.split()[0], entry)
        self.assertEqual((self.repository / path).stat().st_mode & 0o111,
                         0o111 if entry == b'100755' else 0)

    def test_journey_written_conflicts_take_base_bytes_and_mode_without_generation(self):
        paths = (JOURNEYS + 'x/01.html',
                 JOURNEYS.replace('/web/', '/tui/') + 'x/123.txt',
                 JOURNEYS + 'x/recording', JOURNEYS + 'skein-ui.css',
                 JOURNEYS + 'journey.js',
                 JOURNEYS.replace('/web/', '/tui/') + 'theme.css',
                 JOURNEYS.replace('/web/', '/tui/') + 'tui-journey.js')
        for index, path in enumerate(paths):
            with self.subTest(path=path):
                self.setUp()
                self.forbid_journey_generation()
                self.write(path, 'common\n')
                # Alternate which side supplies an executable file.
                head_mode, base_mode = ((0o644, 0o755) if index % 2 == 0
                                        else (0o755, 0o644))
                (self.repository / path).chmod(head_mode)
                self.commit('common')
                self.git('branch', 'base')
                self.write(path, b'head\r\n\x00\xff\n')
                head = self.commit('head')
                self.git('checkout', '-q', 'base')
                self.write(path, b'base\r\n\x00\xfe\n')
                (self.repository / path).chmod(base_mode)
                base = self.commit('base')
                self.merge_conflict(head, base)
                self.run_merge([path], [])
                self.assert_base_entry(path)

    def test_journey_base_deletion_removes_tree_and_all_index_stages(self):
        path = JOURNEYS + 'x/01.html'
        self.forbid_journey_generation()
        self.write(path, 'common\n')
        self.commit('common')
        self.git('branch', 'base')
        self.write(path, 'head\n')
        head = self.commit('head')
        self.git('checkout', '-q', 'base')
        self.git('rm', path)
        base = self.commit('base deletes page')
        self.merge_conflict(head, base)
        self.assert_stages({path: {1, 2}})
        self.run_merge([path], [])
        self.assertFalse((self.repository / path).exists())
        self.assertEqual(self.git('ls-files', '--stage', '--', path).stdout, b'')

    def test_journey_head_deletion_restores_base_copy(self):
        path = JOURNEYS + 'x/01.html'
        self.forbid_journey_generation()
        self.write(path, 'common\n')
        self.commit('common')
        self.git('branch', 'base')
        self.git('rm', path)
        head = self.commit('head deletes page')
        self.git('checkout', '-q', 'base')
        self.write(path, 'base\n')
        (self.repository / path).chmod(0o755)
        base = self.commit('base')
        self.merge_conflict(head, base)
        self.assert_stages({path: {1, 3}})
        self.run_merge([path], [])
        self.assert_base_entry(path)

    def test_journey_renames_use_base_tree_at_each_conflicted_path(self):
        old, head_path, base_path = (JOURNEYS + 'x/' + name
                                     for name in ('01.html', '02.html', '03.html'))
        self.forbid_journey_generation()
        self.write(old, 'common\n' * 10)
        self.commit('common')
        self.git('branch', 'base')
        self.git('mv', old, head_path)
        head = self.commit('head renames page')
        self.git('checkout', '-q', 'base')
        self.git('mv', old, base_path)
        base = self.commit('base renames page')
        self.merge_conflict(head, base)
        self.assert_stages({old: {1}, head_path: {2}, base_path: {3}})
        self.run_merge(sorted([old, head_path, base_path]), [])
        for path in (old, head_path):
            self.assertFalse((self.repository / path).exists())
            self.assertEqual(self.git('ls-files', '--stage', '--', path).stdout, b'')
        self.assert_base_entry(base_path)

    def test_journey_sources_and_near_matches_are_left_with_their_records(self):
        for relative in ('x/README.md', 'x/setup', 'x/walk', 'overlay.css',
                         'x/1.html', 'x/01.html.bak', 'x/nested/01.html',
                         'x/skein-ui.css', 'recording'):
            with self.subTest(path=relative):
                self.setUp()
                self.forbid_journey_generation()
                path = JOURNEYS + relative
                self.write(path, 'common\n')
                self.conflict(baseline=False, head_files=[(path, 'head\n')],
                              base_files=[(path, 'base\n')])
                self.run_merge([], [path])

    def test_journey_page_preserves_other_merge_rules(self):
        self.recording_generator()
        path = JOURNEYS + 'x/01.html'
        self.write(path, 'common\n')
        self.conflict(other=True,
                      head_files=[(path, 'head\n'), (RECORDING, 'head recording\n')],
                      base_files=[(path, 'base\n'), (RECORDING, 'base recording\n')])
        self.run_merge(sorted([BASELINE, RECORDING, path]), ['temper/x.rs'])
        self.assert_base_entry(path)
        self.assert_recounted()
        self.assert_generator_called()
        self.assertEqual((self.repository / RECORDING).read_text(),
                         'merged generator\ncommon left\ncommon right\n')

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

    def codec_generator(self, outcome='write'):
        self.write(CODEC_LIST, '# Module, then ordered schema texts\n' + CODEC + ' ' + ' '.join(SCHEMAS) + '\n')
        self.write(CODEC, 'common generated module\n')
        self.write(SCHEMAS[0], self.codec_schema())
        self.write(SCHEMAS[1], '// schema fragment\n')
        for source in CODEC_SOURCES:
            self.write(source, 'common generator source\n')
        executable = self.job / 'bin/cargo'
        executable.parent.mkdir()
        executable.write_text(
            '#!/usr/bin/python3\n'
            'import json, os, sys\nfrom pathlib import Path\n'
            'root = Path.cwd()\n'
            'job = Path(os.environ["TEMPER_JOB"])\n'
            'log = job / "codec-call.json"\n'
            'calls = json.loads(log.read_bytes()) if log.exists() else []\n'
            'calls.append({"args": sys.argv[1:], "cwd": str(root),\n'
            '    "write": os.environ.get("SKEIN_CODEGEN_WRITE"),\n'
            '    "bundle": os.environ.get("TEMPER_INTERNAL_BUNDLE")})\n'
            'log.write_text(json.dumps(calls))\n'
            'assert sys.argv[1:8] == ["run", "-p", "skein-codegen", "--bin", "skein-codegen", "--", '
            f'{SCHEMAS[0]!r}]\n'
            f'outcome = {outcome!r}\n'
            'if outcome == "build-fail":\n    sys.exit(1)\n'
            'directory = Path(sys.argv[-1])\n'
            'assert directory.is_dir()\n'
            'target = directory / "note.rs"\n'
            'assert not target.exists(), "a fresh output is required"\n'
            'if outcome == "write":\n'
            '    mark = Path(sys.argv[7]).parent / "lean"\n'
            '    lean = mark.is_file() and "note" in [line.split("#", 1)[0].strip() '
            'for line in mark.read_text().splitlines()]\n'
            '    prefix = "// stub lean output\\n" if lean else "// stub output\\n"\n'
            '    target.write_text(prefix + "".join((root / path).read_text() '
            'for path in sys.argv[7:-1]))\n'
            'elif outcome == "fail":\n'
            '    target.write_text("partial codec")\n    sys.exit(1)\n'
            'elif outcome == "empty":\n    target.write_bytes(b"")\n'
            'elif outcome == "markers":\n'
            '    target.write_text("<<<<<<< head\\n=======\\n>>>>>>> base\\n")\n',
            encoding='utf-8')
        executable.chmod(0o755)
        self.env['PATH'] = str(executable.parent) + ':' + self.env['PATH']

    @staticmethod
    def codec_schema(left='u8', right='u8'):
        return ('family note version 1\n'
                f'record Left = 1 {{\n    value: {left} = 1\n}}\n'
                '// Separate records change on separate branches.\n\n\n\n'
                f'record Right = 2 {{\n    value: {right} = 1\n}}\n')

    def codec_conflict(self, **kwargs):
        self.conflict(baseline=False, other=True, other_path=CODEC,
                      head_files=[(SCHEMAS[0], self.codec_schema(left='u16'))],
                      base_files=[(SCHEMAS[0], self.codec_schema(right='u32'))], **kwargs)

    def assert_codec_called(self):
        calls = json.loads((self.job / 'codec-call.json').read_bytes())
        self.assertEqual(len(calls), 1, 'only the listed module runs')
        call = calls[0]
        temporary = call['args'][-1]
        self.assertEqual(call, {
            'args': ['run', '-p', 'skein-codegen', '--bin', 'skein-codegen', '--', *SCHEMAS, temporary],
            'cwd': str(self.repository), 'write': '1', 'bundle': 'none'})
        self.assertEqual(Path(temporary).parent, (self.repository / CODEC).parent)
        self.assertFalse(Path(temporary).exists(), 'scratch output removed')

    def test_codec_is_regenerated_from_clean_merged_texts_and_staged(self):
        self.codec_generator()
        self.codec_conflict()
        target = self.repository / CODEC
        target.chmod(0o755)
        self.run_merge([CODEC], [])
        self.assert_codec_called()
        expected = ('// stub output\n' + self.codec_schema(left='u16', right='u32') +
                    '// schema fragment\n').encode()
        self.assertEqual(target.read_bytes(), expected)
        self.assertEqual(self.git('show', f':0:{CODEC}').stdout, expected)
        self.assertEqual(target.stat().st_mode & 0o777, 0o755)
        self.assertTrue(self.git('ls-files', '--stage', '--', CODEC).stdout.startswith(b'100755 '))

    def test_conflicted_lean_mark_leaves_codec_unbuilt(self):
        self.codec_generator()
        self.write(LEAN_MARK, 'note\n')
        self.conflict(baseline=False, other=True, other_path=CODEC,
                      other_paths=[LEAN_MARK])
        self.run_merge([], [CODEC, LEAN_MARK])
        self.assertFalse((self.job / 'codec-call.json').exists(), 'a conflicted mark prevents building')

    def test_clean_lean_mark_regenerates_codec_in_lean_form(self):
        for side in ('head', 'base', 'both'):
            with self.subTest(side=side):
                self.setUp()
                self.codec_generator()
                common = '# choices\n\n\n\n\n# other choices\n'
                if side == 'both':
                    self.write(LEAN_MARK, common)
                self.conflict(baseline=False, other=True, other_path=CODEC,
                              head_files=[(LEAN_MARK, 'note\n' + common)]
                              if side in ('head', 'both') else [],
                              base_files=[(LEAN_MARK, common + 'other\n' if side == 'both' else 'note\n')]
                              if side in ('base', 'both') else [])
                self.run_merge([CODEC], [])
                self.assert_codec_called()
                expected = ('// stub lean output\n' + self.codec_schema() + '// schema fragment\n').encode()
                self.assertEqual((self.repository / CODEC).read_bytes(), expected)
                self.assertEqual(self.git('show', f':0:{CODEC}').stdout, expected)
                self.assertEqual((self.repository / LEAN_MARK).read_text(),
                                 'note\n' + common + 'other\n' if side == 'both'
                                 else 'note\n' + common if side == 'head' else 'note\n')

    def test_conflicted_codec_sources_leave_modules_unbuilt(self):
        for source in [SCHEMAS[0], *CODEC_SOURCES]:
            with self.subTest(source=source):
                self.setUp()
                self.codec_generator()
                other_codec = 'a/src/generated/second.rs'
                modules = [CODEC]
                if source != SCHEMAS[0]:
                    self.write(other_codec, 'common second module\n')
                    self.write(CODEC_LIST, CODEC + ' ' + ' '.join(SCHEMAS) + '\n' +
                               other_codec + ' ' + ' '.join(SCHEMAS) + '\n')
                    modules.append(other_codec)
                self.conflict(baseline=False, other=True, other_path=CODEC,
                              other_paths=[source, *modules[1:]])
                self.run_merge([], sorted([*modules, source]))
                self.assertFalse((self.job / 'codec-call.json').exists(), 'nothing is built')

    def test_failed_codec_generation_preserves_conflicted_bytes_mode_and_index(self):
        for outcome in ('build-fail', 'fail', 'no-output', 'empty', 'markers', 'unavailable'):
            with self.subTest(outcome=outcome):
                self.setUp()
                self.codec_generator(outcome)
                if outcome == 'unavailable':
                    (self.job / 'bin/cargo').unlink()
                self.codec_conflict()
                target = self.repository / CODEC
                target.chmod(0o755)
                mode = target.stat().st_mode
                self.run_merge([], [CODEC])
                self.assertEqual(target.stat().st_mode, mode)
                self.assertEqual(list(target.parent.glob('.merge-codec-*')), [])
                if outcome != 'unavailable':
                    self.assert_codec_called()

    def test_only_listed_codecs_run_and_conflicted_contents_supply_no_arguments(self):
        self.codec_generator()
        unknown = 'a/src/generated/unlisted.rs'
        self.write(unknown, '// Generated by skein-codegen from unlisted version 1.\n')
        self.conflict(baseline=False, other=True, other_path=CODEC, other_paths=[unknown],
                      head_files=[(CODEC, '// Generated by skein-codegen from evil/schema-v1.schema\n')],
                      base_files=[(CODEC, '// Generated by skein-codegen from other/schema-v2.schema\n')])
        self.run_merge([CODEC], [unknown])
        self.assert_codec_called()

    def test_conflicted_codec_list_is_not_used(self):
        self.codec_generator()
        row = CODEC + ' ' + ' '.join(SCHEMAS) + '\n'
        self.conflict(baseline=False, other=True, other_path=CODEC,
                      head_files=[(CODEC_LIST, '# Head list\n' + row)],
                      base_files=[(CODEC_LIST, '# Incoming list\n' + row)])
        self.run_merge([], [CODEC, CODEC_LIST])
        self.assertFalse((self.job / 'codec-call.json').exists(), 'a conflicted list supplies no command')

    def test_baseline_alone_is_recounted_and_staged(self):
        self.assertTrue(os.access(ROOT / '.temper/merge', os.X_OK))
        self.conflict()
        self.run_merge([BASELINE], [])
        self.assert_recounted()

    def test_baseline_beside_other_conflict(self):
        for path in ('temper/x.rs', '.temper/known-reds.md'):
            with self.subTest(path=path):
                self.setUp()
                self.conflict(other=True, other_path=path,
                              head_files=((path, '- test p head\n'),),
                              base_files=((path, '- test p incoming\n'),))
                conflict = (self.repository / path).read_bytes()
                stages = self.git('ls-files', '--stage', '--', path).stdout
                execution = self.run_merge([BASELINE], [{'path': path, 'kind':
                                                       'added' if path == '.temper/known-reds.md'
                                                       else 'modified'}])
                self.assert_recounted()
                self.assertEqual((self.repository / path).read_bytes(), conflict)
                self.assertEqual(self.git('ls-files', '--stage', '--', path).stdout, stages)
                self.assertNotIn(f'{path}: left:', execution.stderr.decode())

    def test_other_conflict_leaves_work_tree_and_index_unchanged(self):
        for path in ('temper/x.rs', '.temper/known-reds.md'):
            with self.subTest(path=path):
                self.setUp()
                self.conflict(baseline=False, other=True, other_path=path,
                              head_files=((path, '- test p head\n'),),
                              base_files=((path, '- test p incoming\n'),))
                previous = self.snapshot()
                execution = self.run_merge([], [{'path': path, 'kind':
                                                 'added' if path == '.temper/known-reds.md'
                                                 else 'modified'}])
                self.assertEqual(self.snapshot(), previous)
                self.assertNotIn(f'{path}: left:', execution.stderr.decode())

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
        self.assert_stages({BASELINE: {1, 3}})
        commit = self.git('rev-parse', 'HEAD').stdout.decode().strip()
        self.run_merge([], [{'path': BASELINE, 'kind': 'deleted', 'by': 'head',
                             'commit': commit[:12]}])
        self.assertEqual(self.snapshot(), previous)

    def test_delete_modify_conflicts_are_left(self):
        self.conflict(delete='base')
        previous = self.snapshot()
        self.assert_stages({BASELINE: {1, 2}})
        commit = self.git('rev-parse', 'MERGE_HEAD').stdout.decode().strip()
        self.run_merge([], [{'path': BASELINE, 'kind': 'deleted', 'by': 'base',
                             'commit': commit[:12]}])
        self.assertEqual(self.snapshot(), previous)

    def test_added_on_both_sides_is_named(self):
        path = 'added.txt'
        self.conflict(baseline=False, head_files=[(path, 'head\n')],
                      base_files=[(path, 'base\n')])
        self.assert_stages({path: {2, 3}})
        self.run_merge([], [{'path': path, 'kind': 'added'}])

    def test_deleting_commit_precedes_the_base_tip(self):
        path = '.temper/known-reds.md'
        self.write(path, '- shared red\n')
        self.commit('common')
        self.git('branch', 'base')
        self.write(path, '- shared red\n- head red\n')
        head = self.commit('append red')
        self.git('checkout', '-q', 'base')
        self.git('rm', path)
        deletion = self.commit('delete reds')
        self.write('unrelated', 'later\n')
        base = self.commit('later base tip')
        self.merge_conflict(head, base)
        self.assert_stages({path: {1, 2}})
        self.run_merge([], [{'path': path, 'kind': 'deleted', 'by': 'base',
                             'commit': deletion[:12]}])

    def test_rename_before_rewrite_is_not_reported_as_deletion(self):
        for by in ('base', 'head'):
            with self.subTest(by=by):
                self.setUp()
                source = 'old\n"\\é[*].txt'
                target = 'new\n"\\é[*].txt'
                # A later deletion that a nonliteral pathspec could also match.
                decoy = 'old\n"é*.txt'
                self.write(source, 'common file\n' * 10)
                self.write(decoy, 'decoy\n')
                self.commit('common')
                self.git('branch', 'base')
                if by == 'base':
                    self.write(source, 'common file\n' * 10 + 'head addition\n')
                    head = self.commit('head modifies source')
                    self.git('checkout', '-q', 'base')
                self.git('mv', '--', source, target)
                rename = self.commit('pure rename')
                self.write(target, 'rewritten destination\n' * 10)
                self.git('rm', '--', decoy)
                renamed_tip = self.commit('rewrite destination and remove decoy')
                if by == 'base':
                    base = renamed_tip
                else:
                    head = renamed_tip
                    self.git('checkout', '-q', 'base')
                    self.write(source, 'common file\n' * 10 + 'base addition\n')
                    base = self.commit('base modifies source')
                self.merge_conflict(head, base)
                self.assert_stages({source: {1, 2} if by == 'base' else {1, 3}})
                self.run_merge([], [{'path': source, 'kind': 'renamed', 'by': by,
                                     'to': target, 'commit': rename[:12]}])

    def test_rename_into_an_added_path_names_its_source(self):
        source = 'old\n"\\é[*].txt'
        target = 'new\n"\\é[*].txt'
        self.write(source, 'common file\n')
        self.commit('common')
        self.git('branch', 'base')
        self.git('mv', '--', source, target)
        head = self.commit('rename on head')
        self.git('checkout', '-q', 'base')
        self.write(target, 'added on base\n')
        base = self.commit('base adds destination')
        self.merge_conflict(head, base)
        self.assert_stages({target: {2, 3}})
        self.run_merge([], [{'path': target, 'kind': 'renamed', 'by': 'head',
                             'from': source, 'commit': head[:12]}])

    def test_both_sides_rename_into_one_path(self):
        target = 'destination'
        self.write('first', 'first file\n')
        self.write('second', 'second file\n')
        self.commit('common')
        self.git('branch', 'base')
        self.git('mv', 'first', target)
        head = self.commit('head rename')
        self.git('checkout', '-q', 'base')
        self.git('mv', 'second', target)
        base = self.commit('base rename')
        self.merge_conflict(head, base)
        self.assert_stages({target: {2, 3}})
        self.run_merge([], [{'path': target, 'kind': 'renamed', 'by': 'both'}])

    def test_both_sides_rename_away_from_one_path(self):
        self.write('source', 'common file\n')
        self.commit('common')
        self.git('branch', 'base')
        self.git('mv', 'source', 'head-name')
        head = self.commit('head rename')
        self.git('checkout', '-q', 'base')
        self.git('mv', 'source', 'base-name')
        base = self.commit('base rename')
        self.merge_conflict(head, base)
        self.assert_stages({'source': {1}, 'head-name': {2}, 'base-name': {3}})
        self.run_merge([], [{'path': 'base-name', 'kind': 'unknown'},
                            {'path': 'head-name', 'kind': 'unknown'},
                            {'path': 'source', 'kind': 'renamed', 'by': 'both'}])

    def test_rename_delete_with_no_removal_of_index_path_is_unknown(self):
        self.write('source', 'common file\n')
        self.commit('common')
        self.git('branch', 'base')
        self.git('mv', 'source', 'destination')
        head = self.commit('head rename')
        self.git('checkout', '-q', 'base')
        self.git('rm', 'source')
        base = self.commit('base deletes source')
        self.merge_conflict(head, base)
        self.assert_stages({'destination': {1, 2}})
        self.run_merge([], [{'path': 'destination', 'kind': 'unknown'}])

    def test_path_with_newline_is_left(self):
        path = 'temper/other\n"\\é[*].rs'
        self.write(path, 'const X: u8 = 0;\n')
        self.conflict(baseline=False, other=True, other_path=path)
        previous = self.snapshot()
        self.run_merge([], [path])
        self.assertEqual(self.snapshot(), previous)

    def test_many_conflicts_list_the_paths_that_fit_and_count_the_rest(self):
        paths = [f'temper/z{number:02d}-' + 'x' * 43 + '.rs' for number in range(20)]
        for path in paths:
            self.write(path, 'const X: u8 = 0;\n')
        self.conflict(other=True, other_path=paths[0], other_paths=paths[1:])
        self.assert_stages({path: {1, 2, 3} for path in [BASELINE, *paths]})
        self.run_merge([BASELINE], paths, truncated=True)
        result = json.loads((self.job / 'result.json').read_bytes())
        self.assertTrue(result['left'])
        self.assertEqual(result['settled'], [])
        self.assertEqual(result['more']['settled'], 1)
        self.assert_recounted()

    def test_oversized_left_omits_later_records_but_not_settled(self):
        path = 'a/' + ('x' * 200 + '/') * 3 + 'file'
        self.write(path, 'common\n')
        self.write('z', 'common\n')
        self.conflict(other=True, other_path=path, other_paths=['z'])
        self.assert_stages({BASELINE: {1, 2, 3}, path: {1, 2, 3}, 'z': {1, 2, 3}})
        self.run_merge([BASELINE], [path, 'z'], truncated=True)
        result = json.loads((self.job / 'result.json').read_bytes())
        self.assertNotIn('left', result)
        self.assertEqual(result['settled'], [BASELINE])
        self.assertEqual(result['more']['left'], 2)
        self.assert_recounted()

    def test_one_path_larger_than_the_result_is_counted(self):
        path = 'temper/' + ('x' * 200 + '/') * 3 + 'file.rs'
        self.write(path, 'const X: u8 = 0;\n')
        self.conflict(baseline=False, other=True, other_path=path)
        previous = self.snapshot()
        self.run_merge([], [path], truncated=True)
        result = json.loads((self.job / 'result.json').read_bytes())
        self.assertNotIn('left', result)
        self.assertEqual(result['more']['left'], 1)
        self.assertEqual(self.snapshot(), previous)


if __name__ == '__main__':
    unittest.main()
