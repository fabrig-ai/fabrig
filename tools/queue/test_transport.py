#!/usr/bin/env python3
"""Bounded local/SSH envelopes and real supervised disposable Git jobs."""
import copy
import importlib.util
import os
from pathlib import Path
import shlex
import subprocess
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location('transport_fixture', Path(__file__).with_name('transport_fixture.py'))
f = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(f)
t, s, w, d = f.transport, f.supervisor, f.worker, f.durable


class WireTests(unittest.TestCase):
    def test_binary_envelope_rejects_corruption_truncation_and_contamination(self):
        value, blobs = {'request': "spaces ; ' $()"}, [b'abc\0def']
        frame = t.wire.pack(value, blobs)
        self.assertEqual(t.wire.unpack(frame), (value, blobs))
        for damaged in (frame[:-1], b'banner\n'+frame, frame+b'\n', frame[:-1]+b'x', b'MQ-WIRE-1\n'):
            with self.subTest(damaged=damaged[-8:]), self.assertRaises((ValueError, KeyError)):
                t.wire.unpack(damaged)

    def test_duplicate_and_oversized_headers_refused(self):
        import struct
        for raw in (b'{"value":{},"value":{},"blobs":[]}', b''):
            frame = t.wire.MAGIC + struct.pack('!Q', len(raw) if raw else t.wire.JSON_LIMIT+1) + raw
            with self.assertRaises(ValueError):
                t.wire.unpack(frame)

    def test_bounded_diagnostic_channel_and_timeout_do_not_claim_completion(self):
        with self.assertRaises(TimeoutError):
            t.wire.exchange(['python3', '-I', '-B', '-c', 'import signal; signal.pause()'], b'', timeout=0.01)
        with patch.object(t.wire, 'JSON_LIMIT', 64), self.assertRaises(ValueError):
            t.wire.exchange(['python3', '-I', '-B', '-c', 'import os; os.write(2,b"x"*200)'], b'', timeout=2)


class TransportTests(f.EndpointFixture, unittest.TestCase):
    def test_lost_start_ack_retries_same_job_once_and_returns_full_artifacts(self):
        request, bundle = self.request()
        exchange = self.real_transport.exchange
        def lose(argv, data, **kwargs):
            reply = exchange(argv, data, **kwargs)
            self.real_transport.exchange = exchange
            raise OSError('lost SSH acknowledgement after durable acceptance')
        self.real_transport.exchange = lose
        with self.assertRaises(OSError):
            self.real_transport.start(request, bundle)
        self.real_transport.start(request, bundle)
        reply = self.finish_real(request)
        claim = d.read_json(d.read_bytes(self.job(request) / 'claim.json'))
        self.assertEqual(reply['state'], 'terminal')
        self.assertEqual(d.read_json(reply['result'])['outcome'], 'green')
        self.assertEqual(reply['request'], request)
        self.assertEqual(list(Path(self.configs[request['gate']]['artifacts']).iterdir()),
                         [Path(self.configs[request['gate']]['artifacts']) / request['attempt']])
        self.assertEqual(self.real_transport.verify_completion(request, reply['evidence'], {}), reply['evidence'])
        self.assertEqual(d.read_json(d.read_bytes(self.job(request) / 'claim.json')), claim)
        self.assertEqual(f.mq.git(self.origin, 'rev-parse', 'main').stdout.strip(), self.base)
        self.assertFalse((self.workers[request['gate']] / '.git' / s.MARKER).exists())

    def test_real_nonempty_tools_command_is_checked_and_supervised(self):
        request, bundle = self.request(tools='print("ordinary fixture command")\n')
        self.assertTrue(request['plan'])
        self.real_transport.start(request, bundle)
        reply = self.finish_real(request)
        result = d.read_json(reply['result'])
        self.assertEqual(result['outcome'], 'green', result)
        proof = d.read_json(d.read_bytes(self.job(request) / 'completion.json'))
        self.assertEqual(len(proof['commands']), 1)
        self.assertIn(b'ordinary fixture command', reply['log'])
        self.assertTrue(proof['reaped'])
        self.assertEqual(result['requested']['candidate'], request['candidate'])

    def test_red_command_has_complete_evidence_and_never_pushes(self):
        request, bundle = self.request(tools='raise SystemExit(1)\n')
        self.real_transport.start(request, bundle)
        reply = self.finish_real(request)
        self.assertEqual(d.read_json(reply['result'])['outcome'], 'red')
        self.assertEqual(f.mq.git(self.origin, 'rev-parse', 'main').stdout.strip(), self.base)

    def test_attempt_reuse_and_incarnation_mismatch_are_refused(self):
        request, bundle = self.request()
        config = self.configs[request['gate']]
        s.accept(config, request, bundle, d.checked(bundle['path'], bundle), launch=lambda *_: None)
        with self.assertRaises(ValueError):
            s.accept(config, request, dict(bundle, refs={}), d.checked(bundle['path'], bundle), launch=lambda *_: self.fail('rerun'))
        changed = dict(config, incarnation='replacement')
        with self.assertRaises(ValueError):
            s.status(changed, request)
        self.assertEqual(s.status(config, request)[0]['state'], 'unknown')
        with self.assertRaises(RuntimeError):
            with f.mq.authority.checkout_lock(Path(config['repo'])):
                self.fail('unresolved endpoint reused')

    def test_corrupt_bundle_is_refused_before_claim_and_checkout_mutation(self):
        request, bundle = self.request()
        config = self.configs[request['gate']]
        before = s.worker.snapshot(Path(config['repo']))
        with self.assertRaises(ValueError):
            s.accept(config, request, bundle, d.checked(bundle['path'], bundle)[:-1], launch=lambda *_: self.fail('launch'))
        self.assertFalse(self.job(request).exists())
        self.assertEqual(s.worker.snapshot(Path(config['repo'])), before)

    def test_corrupt_or_missing_returned_artifacts_never_release(self):
        request, bundle = self.request()
        self.real_transport.start(request, bundle)
        f.wait_file(self.job(request) / 'completion.json')
        log = self.job(request) / 'checks.log'
        log.write_bytes(b'corrupt')
        with self.assertRaises(RuntimeError):
            self.real_transport.status(request)
        self.assertTrue((Path(self.configs[request['gate']]['repo']) / '.git' / s.MARKER).exists())
        log.unlink()
        with self.assertRaises(RuntimeError):
            self.real_transport.status(request)

    def test_ssh_uses_single_double_quoted_login_command_and_bounded_body(self):
        request, bundle = self.request()
        endpoint = copy.deepcopy(self.endpoints[request['gate']])
        hosts = self.root / "known hosts ' ; literal"
        d.publish(hosts, b'fixture host key placeholder\n', immutable=True)
        endpoint.update(host='fixture-host', known_hosts=str(hosts), known_hosts_sha256=w.digest(hosts.read_bytes()))
        remote = t.Transport({request['gate']: endpoint}, timeout=10)
        argv = remote.argv(request['gate'], 'start')
        self.assertEqual(argv[-2], 'fixture-host')
        login = shlex.split(argv[-1])
        self.assertEqual(login[:2], ['bash', '-lc'])
        ordinary = shlex.split(login[2])
        self.assertEqual(ordinary[0:4], ['exec', 'python3', '-I', '-B'])
        self.assertEqual(ordinary[4], str(self.runtime / 'supervisor.py'))
        # A fake SSH endpoint executes exactly the one shell command locally;
        # no SSH client, host connection or service is invoked.
        def fake_ssh(args, data, **kwargs):
            self.assertEqual(args, argv)
            return t.wire.exchange(['sh', '-c', args[-1]], data, **kwargs)
        remote.exchange = fake_ssh
        remote.start(request, bundle)
        reply = self.finish_real(request)
        self.assertEqual(reply['state'], 'terminal')
        self.assertFalse((self.root / 'literal').exists())
        hosts.write_bytes(b'replaced')
        with self.assertRaises(ValueError):
            remote.status(request)

    def test_expired_drain_deadline_never_starts_an_rpc_client(self):
        request,_ = self.request()
        self.real_transport.deadline = 99
        with patch('time.monotonic', return_value=100), patch.object(self.real_transport, 'exchange') as exchange:
            with self.assertRaises(TimeoutError):
                self.real_transport.status(request)
            exchange.assert_not_called()
            self.real_transport.deadline = 105
            exchange.return_value = ({'request':request,'endpoint_sha256':self.endpoints[request['gate']]['config_sha256'],'state':'running'},[])
            self.assertEqual(self.real_transport.status(request)['state'],'running')
            self.assertEqual(exchange.call_args.kwargs['timeout'],5)
            self.assertEqual(d.encoded(self.real_transport.identity()),self.real_transport.frozen)

    def test_untrusted_host_and_config_version_refused(self):
        endpoint = copy.deepcopy(self.endpoints['gate-one'])
        endpoint['host'] = 'host; touch /tmp/no'
        with self.assertRaises(ValueError):
            t.Transport({'gate-one': endpoint})
        config = dict(endpoint['endpoint'], version=2)
        with self.assertRaises(ValueError):
            t.endpoint_config(config)


if __name__ == '__main__':
    unittest.main()
