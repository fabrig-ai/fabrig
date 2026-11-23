#!/usr/bin/env python3
"""Offline checks of the counts published at refs/temper/counts."""

import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import counts


class CountsRepository(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull,
                    'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.test',
                    'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.test'}
        self.git('init', '-q')
        self.git('commit', '-qm', 'first', '--allow-empty')
        self.first = self.git('rev-parse', 'HEAD')
        self.git('commit', '-qm', 'second', '--allow-empty')
        self.second = self.git('rev-parse', 'HEAD')
        self.third = self.git('commit-tree', 'HEAD^{tree}', '-p', self.second, '-m', 'third')
        self.data = {'version': 1, 'commits': [self.first, self.second, self.third], 'profiles': {
            'default': {'binaries': {
                'jig-suite::domain': self.binary('jig', 'domain', {self.first: 10, self.third: 40}),
                'jig-new::domain': self.binary('jig', 'domain', {self.second: 5}),
                'jig-lib': self.binary('jig', 'step', {self.first: 0}),
                'temper-lib': self.binary('temper', 'step', {self.second: 30}),
            }, 'moves': []},
            'fuzzy': {'binaries': {'jig-suite::domain': self.binary('jig', 'domain', {self.first: 9})},
                      'moves': []},
        }}

    @staticmethod
    def binary(tree, layer, figures):
        return dict(tree=tree, layer=layer, cpu_s=9.7, counts=figures)

    def git(self, *args, content=None):
        return subprocess.run(('git', *args), cwd=self.root, env=self.env, input=content,
                              capture_output=True, text=True, check=True).stdout.strip()

    def publish(self, filename='counts.json'):
        blob = self.git('hash-object', '-w', '--stdin', content=json.dumps(self.data))
        tree = self.git('mktree', content=f'100644 blob {blob}\t{filename}\n')
        commit = self.git('commit-tree', tree, '-m', 'publish counts')
        self.git('update-ref', 'refs/temper/counts', commit)
        return blob


class CountsTests(CountsRepository):
    def test_read_and_base(self):
        self.assertIsNone(counts.read(self.root, self.env))
        self.publish('other.json')
        self.assertIsNone(counts.read(self.root, self.env))
        self.publish()
        with patch.object(counts.subprocess, 'run', wraps=subprocess.run) as run:
            data = counts.read(self.root, self.env)
            self.assertEqual(data, self.data)
            self.assertEqual(counts.base(data, self.root, self.env), self.second)
        self.assertEqual([call.args[0] for call in run.call_args_list], [
            ('git', 'cat-file', 'blob', 'refs/temper/counts:counts.json'),
            ('git', 'merge-base', '--is-ancestor', self.third, 'HEAD'),
            ('git', 'merge-base', '--is-ancestor', self.second, 'HEAD'),
        ])
        for call in run.call_args_list:
            self.assertEqual(call.kwargs['env']['GIT_NO_LAZY_FETCH'], '1')
            self.assertNotIn('GIT_NO_LAZY_FETCH', self.env)
        self.assertEqual(counts.base(data, self.root, self.env, self.first), self.first)
        self.assertIsNone(counts.base({**data, 'commits': [self.third]}, self.root, self.env))

    def test_ref_with_missing_blob_is_unavailable(self):
        blob = self.publish()
        (self.root / '.git' / 'objects' / blob[:2] / blob[2:]).unlink()
        self.assertIsNone(counts.read(self.root, self.env))

    def test_sparse_binary_counts_and_layers(self):
        self.assertEqual(counts.binaries(self.data, 'default', self.first), {
            'jig-suite::domain': dict(tree='jig', layer='domain', instructions=10),
            'jig-lib': dict(tree='jig', layer='step', instructions=0),
        })
        self.assertEqual(counts.layers(self.data, 'default', self.second), {
            ('jig', 'domain'): 15, ('jig', 'step'): 0, ('temper', 'step'): 30,
        })
        self.assertEqual(counts.layers(self.data, 'default', self.third), {
            ('jig', 'domain'): 45, ('jig', 'step'): 0, ('temper', 'step'): 30,
        })
        self.assertEqual(counts.layers(self.data, 'fuzzy', self.second), {('jig', 'domain'): 9})
        self.assertEqual(counts.layers(self.data, 'unknown', self.second), {})
        self.assertEqual(counts.binaries(self.data, 'default', 'unknown'), {})


class AllowanceTests(unittest.TestCase):
    def test_floor_and_two_percent_agree_with_gate(self):
        cpu = runpy.run_path(str(HERE / 'cpu.py'))
        floor = cpu['INSTRUCTION_GROWTH_FLOOR']
        self.assertEqual(floor, counts.INSTRUCTION_GROWTH_FLOOR)
        self.assertEqual(floor, 1_000_000_000)
        state = dict(instructions_budget=10**15, instructions_ceiling=10**15,
                     main_instructions_budget=10**15, main_instructions_lowest=None)
        for main, limit in ((0, floor), (49_999_999_999, floor), (50_000_000_049, floor),
                            (50_000_000_050, floor + 1), (239_100_000_099, 4_782_000_001)):
            for credit in (0, 10):
                with self.subTest(main=main, credit=credit):
                    allowed = counts.allowance(main, credit)
                    self.assertEqual(allowed, limit + credit)
                    verdict = cpu['instruction_verdict']
                    self.assertIsNone(verdict('fixture', main + allowed, main, None, state, [],
                                              move_credit=credit))
                    self.assertTrue(verdict('fixture', main + allowed + 1, main, None, state, [],
                                            move_credit=credit))


if __name__ == '__main__':
    unittest.main()
