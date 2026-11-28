"""Replay candidate selection and delegation with a recorded stand-in for the host Gate."""
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

SCRIPTS = Path(os.environ.get('GATE_TEST_SCRIPTS', Path(__file__).resolve().parent))
SCOPE = runpy.run_path(str(SCRIPTS / 'gate_scope.py'))
VALIDATE = runpy.run_path(str(SCRIPTS / 'gate_validate.py'))


class GateRoutes(unittest.TestCase):
    def replay(self, paths, check_status, host_status, want_status, want_full, want_checks):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            observed = []

            def execute(cmd, **kwargs):
                observed.append(cmd[-1] if cmd[-1] in ('routing', 'reporting') else 'checker')
                return subprocess.CompletedProcess(cmd, check_status)

            changed = b'\x00'.join(p.encode() for p in paths) + b'\x00'
            with mock.patch.object(subprocess, 'check_output', return_value=changed):
                decision = SCOPE['checks'](work, 'base', 'head', work, execute)
            self.assertEqual(observed, want_checks)
            host = work / 'host'
            host.write_text('#!/bin/sh\nprintf "%s\\n" "$@" "$CI_GATE_PROBE" "$CI_GATE_TESTS" >> calls\nexit "$HOST_STATUS"\n')
            host.chmod(0o755)
            result = subprocess.run(['sh', SCRIPTS / 'gate_route.sh', sys.executable,
                                     '-c', 'import sys; sys.exit(int(sys.argv[1]))', str(decision)],
                                    cwd=work, env={**os.environ, 'CI_GATE': str(host), 'BASE': 'base',
                                    'CANDIDATE': 'head', 'HOST_STATUS': str(host_status),
                                    'CI_GATE_PROBE': 'yes', 'CI_GATE_TESTS': 'selected'})
            self.assertEqual(result.returncode, want_status)
            calls = work / 'calls'
            self.assertEqual(calls.read_text().splitlines() if calls.exists() else [],
                             ['head', 'base', 'yes', 'selected'] if want_full else [])
            if not want_full:
                self.assertEqual(bool((work / 'failed-tests.txt').read_text()), bool(want_status))

    def test_contract(self):
        checker = 'tools/pre-pr/pre_pr.py'
        router = '.forgejo/scripts/gate_scope.py'
        reporter = '.forgejo/scripts/gate_log.py'
        product = 'jig/src/lib.rs'
        for case in [
            ([checker], 0, 0, 0, False, ['checker']),
            ([checker], 3, 0, 1, False, ['checker']),
            ([checker, product], 0, 0, 0, True, ['checker']),
            ([checker, product], 0, 17, 17, True, ['checker']),
            ([reporter], 0, 0, 0, False, ['reporting']),
            ([router], 0, 0, 0, False, ['routing']),
            ([router, product], 0, 0, 0, True, ['routing']),
            ([router, product], 19, 0, 1, False, ['routing']),
            (['.forgejo/workflows/tier-main.yml'], 0, 0, 0, True, []),
        ]:
            with self.subTest(paths=case[0], statuses=case[1:3]):
                self.replay(*case)

    def test_unexpected_selector_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            result = subprocess.run(['sh', SCRIPTS / 'gate_route.sh', 'sh', '-c', 'exit 7'],
                                    cwd=temporary, env={'PATH': os.defpath}, capture_output=True)
            self.assertEqual(result.returncode, 1)

    def test_workflow_edits(self):
        original = 'name: Gate\njobs:\n  gate:\n    steps:\n      - name: display\n        run: echo hello\n'
        parse = VALIDATE['workflow']
        self.assertEqual(parse(original), parse('# doc\n' + original.replace('display', 'label')))
        self.assertNotEqual(parse(original), parse(original.replace('echo hello', 'false')))
        with self.assertRaises(ValueError):
            parse(original + 'jobs: {}\n')
        with self.assertRaises(subprocess.CalledProcessError):
            parse(original.replace('echo hello', 'if false'))


if __name__ == '__main__':
    unittest.main()
