"""Cheap file contracts for the look experiments; capture decisions live in the lab."""
import json
import re
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
LOOK = ROOT / 'lab' / 'look'
PLANTED = ROOT / 'lab' / 'look-planted'
SCENES = ('chats', 'chat', 'inbox', 'task', 'accounts', 'first-run', 'adoption', 'connect-model')
WIDTHS = [dict(name='desktop', width=1440, height=900, mobile=False),
          dict(name='phone', width=390, height=844, mobile=True)]


# Copied from the lab's `look/style.py`: the capture's 5.3 properties.
# The lab's tools are unavailable as imports under python3 -I.
PROPERTIES = ["font-family", "font-size", "font-weight", "line-height", "letter-spacing",
              "color", "background-color"] + [
    f"border-{side}-{part}" for side in ("top", "right", "bottom", "left")
    for part in ("width", "style", "color")
] + [f"border-{corner}-radius" for corner in ("top-left", "top-right", "bottom-right", "bottom-left")] + [
    "box-shadow", "padding-top", "padding-right", "padding-bottom", "padding-left", "row-gap", "column-gap"]
STYLE_PROPERTIES = PROPERTIES + ["faces", "x", "y", "width", "height", "shown"]


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':'),
                       ensure_ascii=False) + '\n').encode('utf-8')


def read(path):
    return json.loads(path.read_bytes())


class LookTests(unittest.TestCase):
    def test_scenes(self):
        for name in SCENES:
            with self.subTest(scene=name):
                path = LOOK / 'scenes' / (name + '.json')
                scene = read(path)
                self.assertEqual(path.read_bytes(), canonical(scene))
                self.assertEqual(scene['format'], 'look-scene/1')
                self.assertEqual(scene['name'], path.stem)
                self.assertEqual(scene['widths'], WIDTHS)
                self.assertRegex(scene['page']['path'], r'^/[A-Za-z0-9-]+\.html$')
                widths = {width['name'] for width in scene['widths']}
                regions = {region['name'] for region in scene['regions']}
                ids = set()
                for kind in ('expectations', 'departures'):
                    self.assertIsInstance(scene[kind], list)
                    for item in scene[kind]:
                        with self.subTest(kind=kind, id=item.get('id')):
                            self.assertIsInstance(item['id'], str)
                            self.assertRegex(item['id'], r'^[a-z][a-z0-9-]*\Z')
                            self.assertNotIn(item['id'], ids)
                            ids.add(item['id'])
                            self.assertIn(item['width'], widths)
                            self.assertIsInstance(item['cites'], str)
                            self.assertTrue(item['cites'])
                            common = {'id', 'width', 'cites'}
                            if kind == 'expectations':
                                if 'region' in item:
                                    self.assertEqual(set(item), common | {
                                        'region', 'property', 'value'})
                                    self.assertIn(item['region'], regions)
                                    self.assertIn(item['property'], STYLE_PROPERTIES)
                                    self.assertIsInstance(item['value'], str)
                                    self.assertTrue(item['value'])
                                else:
                                    self.assertEqual(set(item), common | {'entry', 'count'})
                                    self.assertIs(type(item['count']), int)
                                    self.assertGreaterEqual(item['count'], 0)
                            elif 'signature' in item:
                                self.assertEqual(set(item), common | {'signature'})
                                signature = item['signature']
                                self.assertIsInstance(signature, str)
                                self.assertTrue(signature)
                                self.assertFalse(any(char.isspace() for char in signature))
                                self.assertFalse(signature.startswith('/'))
                                self.assertFalse(signature.startswith(name + '/'))
                            else:
                                self.assertEqual(set(item), common | {'entry'})
                            if 'entry' in item:
                                self.assertIsInstance(item['entry'], str)
                                self.assertTrue(item['entry'])
                if scene['mockup'] is not None:
                    # Both standing qa-web 11 trouble signatures in the original scenes.
                    for width in widths:
                        for signature in ('trouble/mockup/network/fonts.googleapis.com:443',
                                          'trouble/mockup/log'):
                            self.assertTrue(any(
                                departure['width'] == width
                                and departure.get('signature') == signature
                                for departure in scene['departures']))
                self.assertEqual(scene['page']['ready'],
                                 dict(role='main', name='', match='words'))
                for region in scene['regions']:
                    self.assertEqual(set(region['page']), {'role', 'name', 'match'})
                # The lab's `lab-browser.md`, 4: the regions whose words look-words holds to the mockup's.
                words = scene.get('words', [])
                if scene['mockup'] is None:
                    self.assertEqual(words, [])
                named = set()
                for item in words:
                    with self.subTest(words=item.get('id')):
                        self.assertEqual(set(item), {'id', 'width', 'region', 'data', 'cites'})
                        self.assertRegex(item['id'], r'^[a-z][a-z0-9-]*\Z')
                        self.assertNotIn(item['id'], ids)
                        ids.add(item['id'])
                        self.assertIn(item['width'], widths)
                        self.assertIn(item['region'], regions)
                        self.assertNotIn((item['width'], item['region']), named)
                        named.add((item['width'], item['region']))
                        self.assertTrue(item['cites'])
                        self.assertEqual(len(set(item['data'])), len(item['data']))
                        for pattern in item['data']:
                            self.assertIsNone(re.fullmatch(pattern, ''))
                if scene['mockup'] is not None:
                    self.assertEqual({(width, 'banner') for width in widths} - named, set())

    def test_gallery_names(self):
        path = ROOT / 'tools' / 'gallery' / 'names.txt'
        if not path.exists():
            self.skipTest('tools/gallery/names.txt does not exist; it lands with the gallery')
        names = path.read_text(encoding='utf-8').splitlines()
        for name in SCENES:
            with self.subTest(scene=name):
                path = read(LOOK / 'scenes' / (name + '.json'))['page']['path']
                self.assertTrue(path.startswith('/'))
                self.assertTrue(path.endswith('.html'))
                self.assertIn(path[1:-5], names)

    def test_mockup_paths(self):
        for name in SCENES:
            with self.subTest(scene=name):
                scene = read(LOOK / 'scenes' / (name + '.json'))
                if scene['mockup'] is None:
                    self.assertEqual(name, 'adoption')
                    self.assertTrue(all(part['mockup'] is None for part in scene['regions']))
                    continue
                path, query = scene['mockup']['path'].split('?', 1)
                self.assertTrue(path.startswith('/'))
                self.assertTrue(query.endswith('bare'))
                self.assertTrue((ROOT / 'docs/design/client/mockups' / path[1:]).is_file())
                self.assertTrue(all(part['mockup'] is not None for part in scene['regions']))

    def test_experiment(self):
        path = LOOK / 'experiment.json'
        experiment = read(path)
        self.assertEqual(path.read_bytes(), canonical(experiment))
        self.assertEqual(experiment['format'], 'lab-experiment/1')
        self.assertEqual(experiment['name'], 'look')
        self.assertEqual(experiment['cost'], 'look')
        self.assertEqual(experiment['dirs'], ['page'])
        for field in ('processes', 'faults', 'planted'):
            self.assertEqual(experiment[field], [])
        self.assertIsNone(experiment['driver'])
        self.assertEqual(experiment['actions'], [dict(
            name='stage', env={'PATH': '/usr/bin:/bin'},
            argv=['/bin/sh', '{experiment}/stage', '{artifact.gallery}',
                  '{dir.page}/gallery', '{experiment}'])])
        expected = ['{experiment}/scenes/' + name + '.json' for name in SCENES]
        self.assertEqual(sorted(p.name for p in (LOOK / 'scenes').iterdir()),
                         sorted(name + '.json' for name in SCENES))
        self.assertEqual([item['name'] for item in experiment['collect']], ['capture'])
        self.assertEqual([item['name'] for item in experiment['checks']],
                         ['look-structure', 'look-style', 'look-words'])
        for command in experiment['collect'] + experiment['checks']:
            with self.subTest(command=command['name']):
                argv = command['argv']
                first = argv.index(expected[0])
                self.assertEqual(argv[first:], expected)
                self.assertEqual([arg for arg in argv if '/scenes/' in arg], expected)
                self.assertEqual(command['env'], {'PATH': '/usr/bin:/bin'})
        self.assertEqual(experiment['collect'][0]['scan'], [])

    def test_planted_twin(self):
        def files(directory):
            return {path.relative_to(directory): path.read_bytes()
                    for path in directory.rglob('*') if path.is_file()}
        healthy = files(LOOK)
        planted = files(PLANTED)
        css = Path('planted.css')
        exp = Path('experiment.json')
        self.assertNotIn(css, healthy)
        self.assertIn(css, planted)
        self.assertEqual(set(planted), set(healthy) | {css})
        for path in healthy.keys() - {exp}:
            with self.subTest(path=str(path)):
                self.assertEqual(planted[path], healthy[path])
        experiment = json.loads(planted[exp])
        self.assertEqual(planted[exp], canonical(experiment))
        signatures = experiment['planted']
        self.assertTrue(signatures)
        self.assertEqual(signatures, sorted(set(signatures)))
        for signature in signatures:
            self.assertRegex(signature, r'^check/look-(?:(?:structure|style)/chat|words/task)/(desktop|phone)/.+$')
        experiment['planted'] = []
        self.assertEqual(canonical(experiment), healthy[exp])


if __name__ == '__main__':
    unittest.main()
