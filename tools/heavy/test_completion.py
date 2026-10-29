#!/usr/bin/env python3
"""Read-only completion query against isolated ledger/receipt fixtures."""
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
KIT = Path(__file__).resolve().parent


class CompletionQueryTests(unittest.TestCase):
    def setUp(self):
        os.umask(0o077)
        self.temp = tempfile.TemporaryDirectory(dir=KIT.parents[2], prefix='.heavy-completion-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tmp = self.root / 'tmp'
        self.base = self.tmp / 'heavy'
        self.logs = self.tmp / 'logs'
        (self.base / 'receipts').mkdir(parents=True)
        self.logs.mkdir()
        (self.base / 'host.lock').touch(mode=0o600)
        self.label = 'mq-' + 'a'*64 + '-0'
        self.token = 'b'*32
        self.queue = hashlib.sha256(str(self.base).encode()).hexdigest()
        self.task = {'token': self.token, 'id': '1', 'queue': self.queue, 'label': self.label,
            'repo': str(self.root), 'args': ['python3', '-c', 'print(1)'], 'commit': 'c'*40,
            'tree': 'd'*40, 'state': 'released', 'log': str(self.logs / 'job.log')}
        self.receipt = {'schema': 'heavy-receipt-v1', 'token': self.token, 'id': '1',
            'queue': self.queue, 'lane': 'gate', 'tree': 'd'*40, 'policy': 'fixture',
            'args': self.task['args'], 'outcome': 'completed', 'code': 0, 'payload_started': True,
            'payload_ended': True, 'release_complete': True, 'cgroup_empty': True, 'elapsed_s': 0,
            'log': self.task['log'], 'log_sha256': hashlib.sha256(b'ordinary log\n').hexdigest()}
        Path(self.task['log']).write_bytes(b'ordinary log\n')
        (self.base / 'events.jsonl').write_text('')
        row = ['1', 'gate', 'time', '0', '0', '0', '0', str(self.root), self.task['log'], 'command', self.label]
        (self.logs / 'heavy.tsv').write_text('\t'.join(row) + '\n')
        self.save()

    def save(self, *, version=1):
        (self.base / 'state.json').write_text(json.dumps({'version': version, 'queue': self.queue,
            'tasks': {self.token: self.task}, 'maintenance': {}, 'gate': None, 'pending': []}))
        (self.base / 'receipts' / (self.token + '.json')).write_text(json.dumps(self.receipt))

    def query(self, label=None):
        before = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in self.tmp.rglob('*') if p.is_file()}
        result = subprocess.run([str(KIT / 'heavy'), '--completion-query', label or self.label],
            env={'PATH': os.defpath, 'HOME': str(self.root), 'DEV_ROOT': str(self.root), 'DEV_TMP': str(self.tmp)},
            stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        after = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in self.tmp.rglob('*') if p.is_file()}
        self.assertEqual(after, before, 'query mutated ledger, journals or receipts')
        return result

    def test_released_exact_identity_receipt_and_sealed_log_returned_without_writes(self):
        result = self.query()
        self.assertEqual(result.returncode, 0, result.stderr)
        value = json.loads(result.stdout)
        self.assertEqual(value['label'], self.label)
        task = value['matches'][0]
        self.assertEqual(task['commit'], self.task['commit'])
        self.assertTrue(task['receipt']['row_durable'])
        self.assertEqual(base64.b64decode(task['log_base64']), b'ordinary log\n')

    def test_unreleased_task_returns_inventory_without_completion(self):
        self.task['state'] = 'active'
        self.save()
        result = self.query()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('receipt', json.loads(result.stdout)['matches'][0])

    def test_absence_is_empty_inventory_not_a_release_receipt(self):
        result = self.query('mq-' + 'e'*64 + '-0')
        self.assertEqual(result.returncode, 0)
        self.assertEqual(json.loads(result.stdout)['matches'], [])

    def test_bad_containment_receipt_and_missing_row_refused_read_only(self):
        self.receipt['cgroup_empty'] = False
        self.save()
        self.assertEqual(self.query().returncode, 2)
        self.receipt['cgroup_empty'] = True
        self.save()
        (self.logs / 'heavy.tsv').write_text('')
        self.assertEqual(self.query().returncode, 2)

    def test_corrupt_log_and_receipt_identity_refused_read_only(self):
        Path(self.task['log']).write_text('corrupt')
        self.assertEqual(self.query().returncode, 2)
        Path(self.task['log']).write_text('ordinary log\n')
        self.receipt['token'] = 'e'*32
        self.save()
        self.assertEqual(self.query().returncode, 2)

    def test_unknown_version_wrong_queue_and_unsafe_label_refused_without_writes(self):
        self.save(version=2)
        self.assertEqual(self.query().returncode, 2)
        self.save()
        self.queue = 'e'*64
        self.save()
        self.assertEqual(self.query().returncode, 2)
        self.assertEqual(self.query('../token').returncode, 2)

    def test_oversized_sealed_log_refused_before_read(self):
        with Path(self.task['log']).open('wb') as log:
            log.truncate(64 * 1024 * 1024 + 1)
        self.assertEqual(self.query().returncode,2)

    def test_missing_receipt_refuses_without_mutating_history(self):
        (self.base / 'receipts' / (self.token + '.json')).unlink()
        self.assertEqual(self.query().returncode, 2)


if __name__ == '__main__':
    unittest.main()
