"""Repair and opt-in train seams; tools/queue/design.md, 3.7.1 and 3.12."""
import sys
sys.dont_write_bytecode = True

from contextlib import redirect_stderr
import importlib.machinery
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch


DIRECTORY = Path(__file__).resolve().parent


def load(name, filename):
    loader = importlib.machinery.SourceFileLoader(name, str(DIRECTORY / filename))
    spec = importlib.util.spec_from_loader(name, loader)
    result = importlib.util.module_from_spec(spec)
    loader.exec_module(result)
    return result


mq = load('integration_mq', 'mq')
restart = load('integration_restart', 'restart')
operation = load('integration_operation', 'operation.py')


class QueueIntegrationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='.queue-integration-', dir=DIRECTORY.parents[2])
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.home = self.root / 'home'
        self.home.mkdir(mode=0o700)
        self.repair = {'name': 'ready/ops/fixture', 'tip': 'a' * 40, 'reason': 'Fixture repair'}

    def mixed_modes(self, program):
        selected = mq if program == 'mq' else restart
        action = ['once'] if program == 'mq' else []
        argv = [program, *action, '--train-config', str(self.root / 'config'),
                '--repair-ready=' + self.repair['name'], '--repair-tip=' + self.repair['tip'],
                '--repair-reason=' + self.repair['reason'], '--repair-hold']
        fake = SimpleNamespace(configuration=Mock(return_value={'batch': 16}),
                               run=Mock(return_value=0), restart=Mock())
        spec = SimpleNamespace(loader=SimpleNamespace(exec_module=lambda _: None))
        with patch.dict(os.environ, {'HOME': str(self.home)}), patch.object(sys, 'argv', argv), \
             patch.object(restart, 'runtime_module', return_value=(mq, DIRECTORY)), \
             patch.object(selected.importlib.util, 'spec_from_file_location', return_value=spec), \
             patch.object(selected.importlib.util, 'module_from_spec', return_value=fake), \
             redirect_stderr(io.StringIO()):
            code = selected.main()
        self.assertNotEqual(code, 0, 'train mode silently discarded the pinned repair')
        fake.run.assert_not_called();fake.restart.assert_not_called()
        audit = json.loads((self.home / 'src/fabrig/notes/queue.jsonl').read_text())
        self.assertEqual(audit['repair'], self.repair)
        self.assertEqual((audit['event'], audit['outcome']), ('repair-refusal', 'error'))
        self.assertFalse((self.home / 'src/fabrig/queue.stop').exists())

    def test_train_cli_refuses_and_audits_pinned_legacy_repair(self):
        self.mixed_modes('mq')

    def test_train_restart_refuses_and_audits_pinned_legacy_repair(self):
        self.mixed_modes('restart')

    def test_unknown_legacy_heavy_blocks_train_before_any_client(self):
        layout = operation.authority.Layout(self.root / 'shared')
        marker = layout.root / 'queue.unresolved'
        operation.durable.publish(marker, b'unknown heavy ownership\n', immutable=True)
        before = marker.read_bytes()
        with patch.object(operation, 'preparation_budget') as prepare, \
             patch.object(operation.os, 'fork') as fork, self.assertRaisesRegex(RuntimeError, 'unresolved queue'):
            with operation.open_controller({'authority_dir': str(layout.root)}):
                self.fail('train acquired legacy unresolved ownership')
        prepare.assert_not_called();fork.assert_not_called()
        self.assertEqual(marker.read_bytes(), before)
        with operation.authority.Lock(layout.root / 'authority.lock'):
            pass


if __name__ == '__main__':
    unittest.main()
