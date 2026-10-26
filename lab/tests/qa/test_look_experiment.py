"""The experiment's public stage command and immutable data (lab/README.md)."""
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

# Canonical scene writer is part of the explicitly permitted public surface.
import sys
command = os.environ.get('TEMPER_LAB')
if not command:
    raise RuntimeError('TEMPER_LAB must name an executable lab command')
LAB = Path(command).resolve()
if not LAB.is_file() or not os.access(LAB, os.X_OK):
    raise RuntimeError(f'TEMPER_LAB: expected an executable at {LAB}')
RUNNER = LAB.parent
sys.path.insert(0, str(RUNNER))
from runner.canonical import record_bytes

ROOT = Path(__file__).resolve().parents[3]
DIGEST = '0123456789abcdef'
CSS = b':root { --temper-line: #112233; }\n'


class LookExperiment(unittest.TestCase):
    def setUp(self):
        os.umask(0o077)
        self.temp = tempfile.TemporaryDirectory(prefix='temper-look-')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.scratch = self.base / 'tmp'
        self.scratch.mkdir()

    def archive(self, files):
        path = self.base / 'gallery.tar'
        with tarfile.open(path, 'w', format=tarfile.USTAR_FORMAT) as archive:
            for name, data in sorted(files.items()):
                item = tarfile.TarInfo(name)
                item.mode = 0o644
                item.size = len(data)
                archive.addfile(item, io.BytesIO(data))
        return path

    def stage(self, files, experiment='look', target='page'):
        artifact = self.archive(files)
        destination = self.base / target
        result = subprocess.run(['/bin/sh', ROOT / 'lab' / experiment / 'stage', artifact,
                                 destination, ROOT / 'lab' / experiment], cwd=self.base,
                                env=dict(os.environ, TMPDIR=str(self.scratch)),
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        return result, destination

    def test_stage_preserves_gallery_bytes_and_unrelated_files(self):
        files = {f'bundle/{DIGEST}/temper.css': CSS, 'chat-solo-T10.html': b'<main>chat</main>',
                 'nested/space name.html': b'<main>space</main>', 'bundle/icon.svg': b'<svg/>'}
        result, destination = self.stage(files)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual({f.relative_to(destination).as_posix(): f.read_bytes()
                          for f in destination.rglob('*') if f.is_file()}, files)

    def test_stage_requires_one_stylesheet(self):
        for files in [{}, {'chat.html': b'<main>no CSS</main>'},
                      {f'bundle/{DIGEST}/other.css': CSS}, {'temper.css': CSS}]:
            with self.subTest(files=list(files)):
                result, _ = self.stage(files, target='missing-' + str(len(list(self.base.glob('missing-*')))))
                self.assertNotEqual(result.returncode, 0)
                self.assertTrue(result.stderr, 'zero matching bundles must fail loudly')

    def test_stage_refuses_multiple_digest_bundles(self):
        files = {f'bundle/{DIGEST}/temper.css': CSS,
                 'bundle/fedcba9876543210/temper.css': b'/* second */'}
        result, _ = self.stage(files)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('temper.css', result.stderr.decode())

    def test_planted_css_is_appended_once_after_the_subject_stylesheet(self):
        files = {f'bundle/{DIGEST}/temper.css': CSS, 'chat.html': b'<main>chat</main>'}
        result, destination = self.stage(files, 'look-planted')
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        planted = (ROOT / 'lab/look-planted/planted.css').read_bytes()
        actual = (destination / f'bundle/{DIGEST}/temper.css').read_bytes()
        self.assertTrue(actual.startswith(CSS))
        self.assertTrue(actual.endswith(planted))
        self.assertEqual(actual.count(planted), 1)
        self.assertEqual((destination / 'chat.html').read_bytes(), files['chat.html'])

    def test_healthy_and_planted_stage_share_all_other_files(self):
        files = {f'bundle/{DIGEST}/temper.css': CSS, 'index.html': b'<main>page</main>',
                 'bundle/fonts/font.woff2': b'font bytes'}
        healthy, left = self.stage(files, target='healthy')
        planted, right = self.stage(files, 'look-planted', target='planted')
        self.assertEqual((healthy.returncode, planted.returncode), (0, 0))
        self.assertEqual({f.relative_to(left) for f in left.rglob('*')},
                         {f.relative_to(right) for f in right.rglob('*')})
        for name, data in files.items():
            with self.subTest(name=name):
                if name.endswith('/temper.css'):
                    self.assertNotEqual((left / name).read_bytes(), (right / name).read_bytes())
                else:
                    self.assertEqual((left / name).read_bytes(), data)
                    self.assertEqual((right / name).read_bytes(), data)

    def test_invalid_tar_fails_loudly(self):
        path = self.base / 'invalid.tar'
        path.write_bytes(b'This is not an archive.\n')
        result = subprocess.run(['/bin/sh', ROOT / 'lab/look/stage', path, self.base / 'page', ROOT / 'lab/look'],
                                cwd=self.base, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(result.stderr)

    def test_staging_does_not_change_subject_tar(self):
        files = {f'bundle/{DIGEST}/temper.css': CSS}
        path = self.archive(files)
        before = path.read_bytes()
        result, _ = self.stage(files, 'look-planted')
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        self.assertEqual(path.read_bytes(), before)

    def test_planted_twin_differs_by_only_declared_fault_data(self):
        healthy = ROOT / 'lab/look'
        planted = ROOT / 'lab/look-planted'
        left = {p.relative_to(healthy).as_posix(): p.read_bytes() for p in healthy.rglob('*') if p.is_file()}
        right = {p.relative_to(planted).as_posix(): p.read_bytes() for p in planted.rglob('*') if p.is_file()}
        self.assertEqual(set(right) - set(left), {'planted.css'})
        self.assertEqual(set(left) - set(right), set())
        for name in left:
            with self.subTest(name=name):
                if name == 'experiment.json':
                    a, b = json.loads(left[name]), json.loads(right[name])
                    self.assertEqual(a['planted'], [])
                    self.assertTrue(b['planted'])
                    b['planted'] = []
                    self.assertEqual(a, b)
                else:
                    self.assertEqual(left[name], right[name])

    def test_every_scene_reaches_capture_and_both_checks_in_the_same_order(self):
        value = json.loads((ROOT / 'lab/look/experiment.json').read_bytes())
        expected = {'adoption', 'accounts', 'chats', 'chat', 'inbox', 'task', 'first-run'}
        self.assertEqual((value['name'], value['cost'], value['dirs']), ('look', 'look', ['page']))
        self.assertEqual((value['processes'], value['driver'], value['faults']), ([], None, []))
        capture = value['collect'][0]['argv']
        scenes = [arg for arg in capture if arg.startswith('{experiment}/scenes/')]
        self.assertEqual({Path(arg).stem for arg in scenes}, expected)
        self.assertEqual(len(scenes), len(expected))
        self.assertIn('{source}/docs/design/client/mockups', capture)
        for check, kind in zip(value['checks'], ['structure', 'style']):
            with self.subTest(kind=kind):
                self.assertEqual(check['name'], 'look-' + kind)
                self.assertEqual(check['argv'][:4], ['{runner}/look/check', kind, '--capture',
                                                      '{collected.capture}/capture'])
                self.assertEqual(check['argv'][4:], scenes)

    def test_planted_signatures_require_style_and_composer_at_both_widths(self):
        experiment = json.loads((ROOT / 'lab/look-planted/experiment.json').read_bytes())
        scene = json.loads((ROOT / 'lab/look-planted/scenes/chat.json').read_bytes())
        regions = {r['name'] for r in scene['regions']}
        expected = set()
        for width in ['desktop', 'phone']:
            expected.add(f'check/look-structure/chat/{width}/order/composer')
            styles = [s for s in experiment['planted'] if s.startswith(f'check/look-style/chat/{width}/style/')]
            self.assertTrue(styles, width)
            for signature in styles:
                self.assertIn(signature.split('/')[-2], regions)
                self.assertFalse(any(d.get('signature') == '/'.join(signature.split('/')[4:])
                                     and d['width'] == width for d in scene['departures']))
                expected.add(signature)
        self.assertEqual(set(experiment['planted']), expected)
        self.assertEqual(len(experiment['planted']), len(expected))

    def test_existing_capture_output_is_preserved_with_real_scene_arguments(self):
        browser = Path('/home/free/src/rust/tmp/lab/browser/skein-lab-browser')
        chromium = Path('/home/free/.local/opt/chromium-147.0.7727.101/usr/lib/chromium/chromium')
        for name, path in [('SKEIN_LAB_BROWSER', browser), ('SKEIN_TEST_CHROMIUM', chromium)]:
            self.assertTrue(path.is_file() and os.access(path, os.X_OK), f'missing {name}: {path}')
        out = self.base / 'exists'
        out.mkdir()
        marker = out / 'keep'
        marker.write_bytes(b'existing output')
        for experiment in ['look', 'look-planted']:
            with self.subTest(experiment=experiment):
                scenes = sorted((ROOT / 'lab' / experiment / 'scenes').glob('*.json'))
                result = subprocess.run([RUNNER / 'look/capture', '--browser', browser,
                    '--chromium', chromium, '--fonts', '/home/free/src/rust/tmp/lab/browser/fonts',
                    '--mockups', ROOT / 'docs/design/client/mockups', '--page', self.base,
                    '--ready-seconds', '1', '--out', out, *scenes], cwd=self.base,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
                self.assertEqual(result.returncode, 2, result.stderr.decode())
                self.assertIn('out', result.stderr.decode())
                self.assertNotIn('expectations', result.stderr.decode())
                self.assertNotIn('departures', result.stderr.decode())
                self.assertEqual(list(out.iterdir()), [marker])

    def test_staged_fault_reaches_real_capture_and_both_check_commands(self):
        browser = Path('/home/free/src/rust/tmp/lab/browser/skein-lab-browser')
        chromium = Path('/home/free/.local/opt/chromium-147.0.7727.101/usr/lib/chromium/chromium')
        for name, path in [('SKEIN_LAB_BROWSER', browser), ('SKEIN_TEST_CHROMIUM', chromium)]:
            self.assertTrue(path.is_file() and os.access(path, os.X_OK), f'missing {name}: {path}')
        html = f'''<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<link rel="stylesheet" href="/bundle/{DIGEST}/temper.css">
<header aria-label="Banner">Banner</header><main class="temper-chat">
<section role="log" aria-label="Conversation">Conversation</section>
<div class="temper-composer-area"><textarea aria-label="Message">Draft</textarea></div></main>'''.encode()
        style = CSS + b'''body{margin:0;font-family:"DejaVu Sans"}header{border-bottom:2px solid var(--temper-line)}
.temper-chat{display:flex;flex-direction:column}section{height:40px}textarea{height:20px}'''
        files = {f'bundle/{DIGEST}/temper.css': style, 'chat.html': html}
        mockups = self.base / 'mockups'
        mockups.mkdir()
        (mockups / 'bundle' / DIGEST).mkdir(parents=True)
        (mockups / 'bundle' / DIGEST / 'temper.css').write_bytes(style)
        (mockups / 'chat.html').write_bytes(html)
        def finder(role, name=''):
            return dict(role=role, name=name, match='exact' if name else 'words')
        scene = dict(format='look-scene/1', name='chat', widths=[
            dict(name='desktop', width=1440, height=900, mobile=False),
            dict(name='phone', width=390, height=844, mobile=True)],
            mockup=dict(path='/chat.html', ready=finder('main')),
            page=dict(path='/chat.html', ready=finder('main')), names=['Message', 'Conversation'],
            regions=[dict(name=name, mockup=finder(role, title), page=finder(role, title)) for name, role, title in
                     [('banner', 'banner', 'Banner'), ('conversation', 'log', 'Conversation'),
                      ('composer', 'textbox', 'Message')]], expectations=[], departures=[])
        scene_path = self.base / 'chat.json'
        scene_path.write_bytes(record_bytes(scene))
        for experiment in ['look', 'look-planted']:
            with self.subTest(experiment=experiment):
                staged, page = self.stage(files, experiment, target=experiment + '-page')
                self.assertEqual(staged.returncode, 0, staged.stderr.decode())
                out = self.base / (experiment + '-capture')
                result = subprocess.run([RUNNER / 'look/capture', '--browser', browser,
                    '--chromium', chromium, '--fonts', '/home/free/src/rust/tmp/lab/browser/fonts',
                    '--mockups', mockups, '--page', page, '--ready-seconds', '3', '--out', out, scene_path],
                    cwd=self.base, env=dict(os.environ, TMPDIR=str(self.scratch)),
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=240)
                self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
                for kind in ['structure', 'style']:
                    checked = subprocess.run([RUNNER / 'look/check', kind, '--capture', out, scene_path],
                                             cwd=self.base, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
                    lines = [json.loads(line) for line in checked.stdout.splitlines()]
                    if experiment == 'look':
                        self.assertEqual(checked.returncode, 0, (lines, checked.stderr.decode()))
                        self.assertEqual(lines, [])
                    else:
                        self.assertEqual(checked.returncode, 1, (lines, checked.stderr.decode()))
                        signatures = {line['fail'] for line in lines}
                        tail = 'order/composer' if kind == 'structure' else 'style/banner/border-bottom-color'
                        for width in ['desktop', 'phone']:
                            self.assertIn(f'chat/{width}/{tail}', signatures)

