#!/usr/bin/env python3
"""Fixture interruption and automatic controller-suite coverage."""

import sys
sys.dont_write_bytecode = True

import ast
from collections import Counter
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import select
import shutil
import signal
import subprocess
import unittest

DIRECTORY = Path(__file__).resolve().parent
ROOT = DIRECTORY.parent.parent


def load_module(path):
    spec = importlib.util.spec_from_file_location(path.stem, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FixtureIsolationTests(unittest.TestCase):
    def test_killed_fixture_child_leaves_source_checkout_unchanged(self):
        def status():
            return subprocess.run(["git", "status", "--porcelain", "--untracked-files=all"],
                                  cwd=ROOT, stdin=subprocess.DEVNULL, capture_output=True,
                                  check=True, text=True).stdout
        before = status()
        caches = set(DIRECTORY.glob("__pycache__/*"))
        code = '''import importlib.util, json, pathlib, sys, unittest
spec = importlib.util.spec_from_file_location('fixture', sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
original = module.tempfile.TemporaryDirectory
def allocate(*args, **kwargs):
    directory = original(*args, **kwargs)
    print(json.dumps({'allocated': directory.name}), flush=True)
    return directory
module.tempfile.TemporaryDirectory = allocate
fixture = type('ChildFixture', (module.TrainFixture, unittest.TestCase), {})('runTest')
fixture.setUp()
print(json.dumps({'ready': True, 'root': str(fixture.root),
                  'paths': [str(fixture.prep), *map(str, fixture.workers.values())],
                  'bytecode_disabled': sys.dont_write_bytecode}), flush=True)
sys.stdin.buffer.read(1)
fixture.doCleanups()
'''
        process = subprocess.Popen([sys.executable, "-I", "-B", "-c", code,
                                    str(DIRECTORY / "train_fixture.py")],
                                   cwd=ROOT.parent, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, bufsize=0, start_new_session=True)
        created = None
        try:
            pending = b""
            def event():
                nonlocal pending
                while b"\n" not in pending:
                    self.assertTrue(select.select([process.stdout], [], [], 20)[0], "child barrier missing")
                    chunk = os.read(process.stdout.fileno(), 65536)
                    self.assertTrue(chunk, "child exited before fixture barrier")
                    pending += chunk
                line, pending = pending.split(b"\n", 1)
                return json.loads(line)
            allocated = event()
            created = Path(allocated["allocated"])
            self.assertIn(created.parent, (ROOT, ROOT.parent))
            self.assertTrue(created.name.startswith((".train-test-", ".mq-train-test-")))
            ready = event()
            self.assertTrue(ready["ready"])
            self.assertTrue(ready["bytecode_disabled"])
            self.assertEqual(ready["root"], str(created))
            self.assertIsNone(process.poll())
            process.kill()
            self.assertEqual(process.wait(timeout=10), -signal.SIGKILL)
            self.assertTrue(created.is_dir(), "SIGKILL must bypass fixture cleanup")
            self.assertEqual(created.stat().st_mode & 0o777, 0o700)
            self.assertEqual(created.parent, ROOT.parent)
            for path in [created, *map(Path, ready["paths"])]:
                self.assertNotEqual(path, ROOT)
                self.assertNotIn(ROOT, path.parents, f"fixture is inside tested source: {path}")
            self.assertEqual(status(), before)
            self.assertEqual(set(DIRECTORY.glob("__pycache__/*")), caches)
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=10)
            for pipe in (process.stdin, process.stdout, process.stderr):
                pipe.close()
            if created is not None and created.parent in (ROOT, ROOT.parent) and created.name.startswith(
                    (".train-test-", ".mq-train-test-")):
                shutil.rmtree(created)
        self.assertFalse(created.exists())
        self.assertEqual(status(), before)

    def test_automatic_discovery_preserves_each_controller_case_once(self):
        support = load_module(DIRECTORY / "train_fixture.py")
        commands = [command for command in support.mq.tools_commands(ROOT)
                    if Path(command[0]).parent == Path("tools/queue") and
                    Path(command[0]).name.startswith("test_train")]
        entries = {"test_train.py", "test_train_decisions.py", "test_train_recovery.py",
                   "test_train_isolation.py"}
        self.assertEqual({Path(command[0]).name for command in commands}, entries)
        self.assertFalse(issubclass(support.TrainFixture, unittest.TestCase))
        self.assertFalse(any(name.startswith("test_") for name in vars(support.TrainFixture)))
        expected = json.loads((DIRECTORY / "train_cases.json").read_text())
        self.assertEqual(len(expected), 63)
        seen = Counter()
        fingerprints = {}
        def cases(suite):
            for test in suite:
                if isinstance(test, unittest.TestSuite):
                    yield from cases(test)
                else:
                    yield test
        for name, argv, cwd in commands:
            self.assertEqual(argv, ["python3", "-I", "-B", name])
            self.assertEqual(cwd, ROOT)
            path = ROOT / name
            module = load_module(path)
            for case in cases(unittest.defaultTestLoader.loadTestsFromModule(module)):
                method = case._testMethodName
                self.assertIn(method, vars(type(case)), "test inherited from another suite")
                seen[method] += 1
            tree = ast.parse(path.read_text())
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef) and node.name in expected:
                    fingerprints[node.name] = hashlib.sha256(
                        ast.dump(node, include_attributes=False).encode()).hexdigest()
        extras = {"test_killed_fixture_child_leaves_source_checkout_unchanged",
                  "test_automatic_discovery_preserves_each_controller_case_once"}
        self.assertEqual(seen, Counter({name: 1 for name in expected.keys() | extras}))
        self.assertEqual(fingerprints, expected, "a moved controller case changed semantics")


if __name__ == "__main__":
    unittest.main()
