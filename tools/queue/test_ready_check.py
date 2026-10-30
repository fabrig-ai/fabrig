"""Before-ready freshness against disposable bare origins (precheck.md, 5)."""

import sys
sys.dont_write_bytecode = True

from copy import deepcopy
import importlib.machinery
import importlib.util
import json
import hashlib
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import unittest
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader("ready_check", str(DIRECTORY / "ready-check"))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
ready = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(ready)


class ReadyCheckTests(unittest.TestCase):
    def setUp(self):
        # Origin, writer, candidate, and evidence are siblings outside the checked source.
        self.temp = tempfile.TemporaryDirectory(prefix=".ready-check-test-", dir=DIRECTORY)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.origin = self.root / "origin.git"
        self.writer = self.root / "writer"
        self.repo = self.root / "candidate ; $(touch BAD) space"
        self.result = self.root / "result.json"
        self.env = dict(os.environ, GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1",
                        GIT_TERMINAL_PROMPT="0")
        self.environment = patch.dict(os.environ, self.env)
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.git(self.root, "init", "--bare", "--initial-branch=main", str(self.origin))
        self.git(self.root, "clone", str(self.origin), str(self.writer))
        self.git(self.writer, "config", "user.name", "Fixture")
        self.git(self.writer, "config", "user.email", "fixture@example.invalid")
        self.write(self.writer, "touched.md", "base\n")
        self.write(self.writer, "other.md", "base\n")
        self.commit(self.writer)
        self.base = self.git(self.writer, "rev-parse", "HEAD")
        self.git(self.writer, "push", "origin", "HEAD:refs/heads/main")
        self.git(self.root, "clone", str(self.origin), str(self.repo))
        self.git(self.repo, "config", "user.name", "Fixture")
        self.git(self.repo, "config", "user.email", "fixture@example.invalid")
        self.git(self.repo, "switch", "-c", "lead/task")
        self.write(self.repo, "touched.md", "candidate\n")
        self.commit(self.repo)
        self.tip = self.git(self.repo, "rev-parse", "HEAD")
        self.record = {"outcome": "passed", "restored": True, "base": self.base, "tip": self.tip,
                       "original": self.tip, "slot": str(self.repo), "started": "2026-01-01T00:00:00+00:00",
                       "heavy_observation": "fixture idle", "paths": ["touched.md"], "selection": "docs",
                       "plan": [], "steps": []}
        self.save()

    def git(self, repo, *args):
        return subprocess.check_output(["git", "--no-optional-locks", "-C", str(repo), *args],
                                       stdin=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                       env=self.env, text=True).strip()

    def write(self, repo, name, contents):
        filename = repo / name
        filename.parent.mkdir(parents=True, exist_ok=True)
        filename.write_text(contents)

    def commit(self, repo):
        self.git(repo, "add", "--all")
        self.git(repo, "commit", "-m", "Fixture change")

    def save(self):
        self.result.write_text(json.dumps(self.record))
        self.result.chmod(0o600)

    def copied_run_receipt(self):
        bundle = self.root
        self.root = bundle / 'slots/4.precheck-logs/run-jeidj41x'
        self.root.mkdir(parents=True, mode=0o700)
        self.result = self.root / 'result.json'
        self.plan()
        original = self.result
        copied = bundle / 'ops/cold-source-proof-20261030T1659/result.json'
        copied.parent.mkdir(parents=True, mode=0o700)
        shutil.copyfile(original, copied)
        copied.chmod(0o600)
        self.result = copied
        return original, copied

    def test_copied_source_receipt_uses_its_bound_original_run(self):
        original, copied = self.copied_run_receipt()
        original_bytes = original.read_bytes()
        manifest = self.root / 'cold-input.json'
        manifest_bytes = manifest.read_bytes()
        self.assertNotEqual(copied.parent, self.root)
        self.assertEqual(copied.read_bytes(), original_bytes)
        self.assertEqual(self.check()['verdict'], 'passed')
        self.assertEqual(original.read_bytes(), original_bytes)
        self.assertEqual(manifest.read_bytes(), manifest_bytes)
        self.assertEqual(copied.read_bytes(), original_bytes)

    def test_copied_receipt_requires_unchanged_private_original(self):
        original, copied = self.copied_run_receipt()
        valid = original.read_bytes()
        self.record['started'] = '2026-01-02T00:00:00+00:00'
        self.save()
        self.refuse('copied receipt differs from bound original')
        copied.write_bytes(valid)
        original.chmod(0o644)
        self.refuse('bound original receipt is not private')
        original.chmod(0o600)
        held = original.with_name('retained-result.json')
        original.rename(held)
        self.refuse('bound original receipt is unavailable')
        original.symlink_to(held.name)
        self.refuse('bound original receipt is a symlink')
        original.unlink()
        held.rename(original)
        copied.chmod(0o644)
        self.refuse('receipt transport file is not private')
        copied.chmod(0o600)
        alias = copied.with_name('linked-result.json')
        alias.symlink_to(copied.name)
        self.result = alias
        self.refuse('receipt transport path is a symlink')

    def test_copied_receipt_still_checks_auxiliary_and_manifest_bindings(self):
        original, copied = self.copied_run_receipt()
        valid = deepcopy(self.record)
        for mutation, diagnostic in (
                (lambda r: r['cold_tools'].update(parent_identity=[1, 2]), 'cold parent identity'),
                (lambda r: r['cold_tools'].update(root=str(self.root / 'another-root')), 'cold auxiliary path'),
                (lambda r: r['cold_tools']['input_manifest'].update(path=str(self.root / 'another.json')), 'cold manifest path'),
                (lambda r: r['cold_tools']['input_manifest'].update(sha256='0' * 64), 'cold manifest digest'),
                (lambda r: r['cold_tools']['input'].update(raw_tree='0' * 40), 'cold observed input')):
            with self.subTest(diagnostic=diagnostic):
                self.record = deepcopy(valid)
                mutation(self.record)
                self.save()
                original.write_bytes(copied.read_bytes())
                self.refuse(diagnostic)

    def advance(self, name="other.md", contents="advance\n"):
        self.write(self.writer, name, contents)
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:refs/heads/main")
        return self.git(self.writer, "rev-parse", "HEAD")

    def check(self):
        return ready.check(self.result, self.repo, self.tip)

    def refuse(self, expected):
        result = self.check()
        self.assertEqual(result["verdict"], "rerun-precheck", result)
        self.assertIn(expected, result["diagnostic"])
        return result

    def plan(self):
        command = ["python3", "-I", "-B", "tools/fixture/test_fixture.py"]
        root = self.root / 'cold-tools'
        checkout = root / 'checkout'
        root.mkdir(mode=0o700)
        checkout.mkdir(mode=0o700)
        identity = lambda path: [path.stat().st_dev, path.stat().st_ino]
        ownership = {'parent': str(self.root), 'parent_identity': identity(self.root),
                     'root': str(root), 'root_identity': identity(root),
                     'checkout': str(checkout), 'checkout_identity': identity(checkout),
                     'objects_identity': [checkout.stat().st_dev, checkout.stat().st_ino + 1]}
        checkout.rmdir(); root.rmdir()
        blob = hashlib.sha1(b'blob 0\0').hexdigest()
        logical = f'100644 {blob} 0\ttools/fixture/test_fixture.py\0'
        rows = [{'name': 'tools/fixture/test_fixture.py', 'index_mode': '100644', 'index_oid': blob,
                 'kind': 'file', 'mode': 0o644, 'size': 0, 'sha256': hashlib.sha256(b'').hexdigest(), 'raw_oid': blob}]
        payload = {'entries': logical, 'manifest': rows, 'merge': {}, 'ownership': ownership}
        manifest = self.root / 'cold-input.json'
        manifest.write_text(json.dumps(payload))
        manifest.chmod(0o600)
        encoded_rows = (json.dumps(rows, sort_keys=True, separators=(',', ':')) + '\n').encode()
        tree = ready.COLD_VALIDATOR['raw_tree'](rows, 'sha1')
        inputs = {'entries_sha256': hashlib.sha256(logical.encode()).hexdigest(),
                  'manifest_sha256': hashlib.sha256(encoded_rows).hexdigest(),
                  'index_tree': tree, 'raw_tree': tree, 'merge': {}}
        self.record['min_free_bytes'] = 1
        self.record['paths'] = ['touched.md', 'tools/fixture/test_fixture.py']
        self.record['cold_tools'] = {
            'version': 1, 'policy': ready.COLD_POLICY, **{k: v for k, v in ownership.items() if k != 'objects_identity'},
            'base': self.record['base'], 'tip': self.tip, 'head': self.record['base'],
            'state': 'removed', 'cleanup': 'verified', 'completion': 'known',
            'input_manifest': {'path': str(manifest), 'sha256': hashlib.sha256(manifest.read_bytes()).hexdigest()},
            'input': inputs, 'observed': {**inputs, 'head': self.record['base'], 'independent_objects': True,
                                        'objects_identity': ownership['objects_identity']},
            'tools_plan': [{'name': 'tools/fixture/test_fixture.py', 'argv': command}],
            'observations': [{'before': n, 'empty_roots': True, 'unexpected_roots': []}
                             for n in ('initial', 'tools/fixture/test_fixture.py', 'after:tools/fixture/test_fixture.py')],
            'environment': {'removed': [], 'redirected': ['TMPDIR', 'TMP', 'TEMP', 'CARGO_HOME'], 'git_config': 'isolated'},
            'capacity': {'required_bytes': 1, 'free_bytes': 2, 'min_free_bytes': 1,
                         'postcopy_free_bytes': 1, 'copied_allocated_bytes': 1, 'postcopy_admitted': True,
                         'admitted': True, 'policy': 'measured-static-copy'},
            'outcomes': [{'name': 'tools/fixture/test_fixture.py', 'returncode': 0, 'cwd': str(checkout)}]}
        self.record.update(plan=[{"name": "tools/fixture/test_fixture.py", "argv": command,
                                  "cwd": str(checkout), "timeout": 60}],
                           steps=[{"name": "tools/fixture/test_fixture.py", "command": command,
                                   "returncode": 0, "green": True, "duration": 0.1, "cwd": str(checkout)}],
                           active_command={"argv": command, "cwd": str(checkout),
                                           "completion": "completed", "returncode": 0})
        self.save()

    def test_unchanged_main_accepts_exact_tip(self):
        self.assertEqual(self.check(), {"verdict": "passed", "tip": self.tip, "checkedbase": self.base,
                                       "currentmain": self.base, "finalmain": self.base,
                                       "touchedscope": ["touched.md"], "overlap": [],
                                       "diagnostic": "reported touched files unchanged at observed origin/main; point-in-time only"})

    def test_cold_receipt_cutover_identity_and_manifest_tampering_refuse(self):
        self.plan()
        valid = deepcopy(self.record)
        mutations = [lambda r: r.pop('cold_tools'),
                     lambda r: r['cold_tools'].update(version=2),
                     lambda r: r['cold_tools'].update(policy={}),
                     lambda r: r['cold_tools'].update(checkout=str(self.repo)),
                     lambda r: r['cold_tools']['root_identity'].__setitem__(1, 1),
                     lambda r: r['cold_tools']['input'].update(raw_tree=self.tip),
                     lambda r: r['cold_tools'].update(tools_plan=[]),
                     lambda r: r['cold_tools'].update(cleanup='pending'),
                     lambda r: r['cold_tools'].update(completion='unknown'),
                     lambda r: r['cold_tools']['observations'][0].update(empty_roots=False),
                     lambda r: r['cold_tools']['environment'].update(removed=None),
                     lambda r: r['cold_tools']['capacity'].update(free_bytes=0),
                     lambda r: r['cold_tools'].update(outcomes=[])]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                self.record = deepcopy(valid)
                mutate(self.record)
                self.save()
                self.assertEqual(self.check()['verdict'], 'rerun-precheck')
        self.record = valid
        self.save()
        manifest = self.root / 'cold-input.json'
        manifest.write_bytes(manifest.read_bytes() + b' ')
        self.refuse('manifest digest')

    def test_tools_and_cargo_cwds_cannot_be_substituted_together(self):
        self.compile_plan('full')
        valid = deepcopy(self.record)
        for index, cwd in ((0, str(self.repo)), (1, valid['cold_tools']['checkout'])):
            with self.subTest(index=index):
                self.record = deepcopy(valid)
                self.record['plan'][index]['cwd'] = cwd
                self.record['steps'][index]['cwd'] = cwd
                self.save()
                self.assertEqual(self.check()['verdict'], 'rerun-precheck')

    def test_coordinated_tools_evidence_erasure_cannot_become_docs_success(self):
        self.plan()
        self.record.update(plan=[], steps=[])
        self.record.pop('active_command')
        self.record.pop('cold_tools')
        self.save()
        self.refuse('requires cold evidence')

    def test_coordinated_suite_substitution_cannot_change_bound_discovery(self):
        self.plan()
        name = 'tools/fixture/test_substituted.py'
        argv = ['python3', '-I', '-B', name]
        self.record['plan'][0].update(name=name, argv=argv)
        self.record['steps'][0].update(name=name, command=argv)
        self.record['active_command']['argv'] = argv
        cold = self.record['cold_tools']
        cold['tools_plan'] = [{'name': name, 'argv': argv}]
        cold['outcomes'][0]['name'] = name
        cold['observations'][1]['before'] = name
        cold['observations'][2]['before'] = 'after:' + name
        self.save()
        self.refuse('indexed discovery')

    def test_empty_index_cannot_claim_a_discovered_suite(self):
        self.plan()
        filename = self.root / 'cold-input.json'
        full = json.loads(filename.read_text())
        full.update(entries='', manifest=[])
        filename.write_text(json.dumps(full))
        cold = self.record['cold_tools']
        cold['input_manifest']['sha256'] = hashlib.sha256(filename.read_bytes()).hexdigest()
        empty = ready.COLD_VALIDATOR['raw_tree']([], 'sha1')
        for key in ('input', 'observed'):
            cold[key].update(entries_sha256=hashlib.sha256(b'').hexdigest(),
                             manifest_sha256=hashlib.sha256(b'[]\n').hexdigest(), index_tree=empty, raw_tree=empty)
        self.save()
        self.refuse('indexed discovery')

    def test_indexed_suite_requires_a_matching_raw_manifest_entry(self):
        self.plan()
        filename = self.root / 'cold-input.json'
        full = json.loads(filename.read_text())
        full['manifest'] = []
        filename.write_text(json.dumps(full))
        receipt = self.record['cold_tools']
        receipt['input_manifest']['sha256'] = hashlib.sha256(filename.read_bytes()).hexdigest()
        for key in ('input', 'observed'):
            receipt[key].update(manifest_sha256=hashlib.sha256(b'[]\n').hexdigest(),
                                raw_tree=ready.COLD_VALIDATOR['raw_tree']([], 'sha1'))
        self.save()
        self.refuse('index/raw manifest')

    def test_coordinated_missing_suite_cannot_claim_success(self):
        for name, mode, argv in (
                ('tools/fixture/test_fixture.py', '100644', ['python3', '-I', '-B', 'tools/fixture/test_fixture.py']),
                ('tools/fixture/test-shell', '100755', ['./tools/fixture/test-shell'])):
            with self.subTest(name=name):
                self.plan()
                filename = self.root / 'cold-input.json'
                full = json.loads(filename.read_text())
                blob = full['manifest'][0]['index_oid']
                full['entries'] = f'{mode} {blob} 0\t{name}\0'
                full['manifest'] = [{'name': name, 'index_mode': mode, 'index_oid': blob, 'kind': 'missing'}]
                filename.write_text(json.dumps(full))
                cold = self.record['cold_tools']
                cold['input_manifest']['sha256'] = hashlib.sha256(filename.read_bytes()).hexdigest()
                for key in ('input', 'observed'):
                    cold[key].update(
                        entries_sha256=hashlib.sha256(full['entries'].encode()).hexdigest(),
                        manifest_sha256=hashlib.sha256(ready.COLD_VALIDATOR['encoded'](full['manifest'])).hexdigest(),
                        index_tree=ready.COLD_VALIDATOR['logical_tree'](full['entries'], 'sha1'),
                        raw_tree=ready.COLD_VALIDATOR['raw_tree'](full['manifest'], 'sha1'))
                cold['tools_plan'] = [{'name': name, 'argv': argv}]
                cold['outcomes'][0]['name'] = name
                cold['observations'][1]['before'] = name
                cold['observations'][2]['before'] = 'after:' + name
                self.record['plan'][0].update(name=name, argv=argv)
                self.record['steps'][0].update(name=name, command=argv)
                self.record['active_command']['argv'] = argv
                self.record['paths'] = ['touched.md', name]
                self.save()
                self.refuse('suite source')

    def test_index_modes_order_and_exact_recovery_exclusion_define_discovery(self):
        entries = [('100755', 'tools/heavy/test-heavy-recovery'),
                   ('100755', 'tools/other/test-heavy-recovery'),
                   ('100644', 'tools/test-noexec'), ('100755', 'tools/test-shell'),
                   ('100644', 'tools/test_python.py')]
        logical = ''.join(f'{mode} {self.base} 0\t{name}\0' for mode, name in entries)
        self.assertEqual(ready.COLD_VALIDATOR['indexed_tools'](logical), [
            {'name': 'tools/other/test-heavy-recovery', 'argv': ['./tools/other/test-heavy-recovery']},
            {'name': 'tools/test-shell', 'argv': ['./tools/test-shell']},
            {'name': 'tools/test_python.py', 'argv': ['python3', '-I', '-B', 'tools/test_python.py']}])

    def test_symlink_suite_requires_present_bound_source_without_cycles(self):
        rows = [{'name': 'tools/fixture/helper.py', 'index_mode': '100644', 'index_oid': self.base,
                 'kind': 'file', 'mode': 0o644},
                {'name': 'tools/fixture/test_link.py', 'index_mode': '120000', 'index_oid': self.base,
                 'kind': 'symlink', 'target': 'helper.py'}]
        logical = ''.join(f"{r['index_mode']} {r['index_oid']} 0\t{r['name']}\0" for r in rows)
        validate = lambda logical, rows: ready.COLD_VALIDATOR['prepared_tools'](
            logical, rows, ['tools/fixture/source.md'], ready.AUTHOR_POLICY.author_tools_select)
        self.assertEqual(validate(logical, rows), [
            {'name': 'tools/fixture/test_link.py', 'argv': ['python3', '-I', '-B', 'tools/fixture/test_link.py']}])
        rows[0]['kind'] = 'missing'
        with self.assertRaisesRegex(RuntimeError, 'suite source is missing'):
            validate(logical, rows)
        rows[1]['target'] = 'test_link.py'
        with self.assertRaisesRegex(RuntimeError, 'suite source has a symlink cycle'):
            validate(logical, rows)

    def test_coordinated_nonexecutable_raw_shell_cannot_claim_success(self):
        self.plan()
        filename = self.root / 'cold-input.json'
        full = json.loads(filename.read_text())
        name = 'tools/fixture/test-shell'
        argv = ['./' + name]
        blob = full['manifest'][0]['index_oid']
        full['entries'] = f'100755 {blob} 0\t{name}\0'
        full['manifest'][0].update(name=name, index_mode='100755', mode=0o644)
        cold = self.record['cold_tools']
        cold['tools_plan'] = [{'name': name, 'argv': argv}]
        cold['outcomes'][0]['name'] = name
        cold['observations'][1]['before'] = name
        cold['observations'][2]['before'] = 'after:' + name
        self.record['plan'][0].update(name=name, argv=argv)
        self.record['steps'][0].update(name=name, command=argv)
        self.record['active_command']['argv'] = argv
        self.record['paths'] = ['touched.md', name]
        for mode, verdict in ((0o755, 'passed'), (0o644, 'rerun-precheck')):
            with self.subTest(mode=oct(mode)):
                full['manifest'][0]['mode'] = mode
                filename.write_text(json.dumps(full))
                cold['input_manifest']['sha256'] = hashlib.sha256(filename.read_bytes()).hexdigest()
                for key in ('input', 'observed'):
                    cold[key].update(
                        entries_sha256=hashlib.sha256(full['entries'].encode()).hexdigest(),
                        manifest_sha256=hashlib.sha256(ready.COLD_VALIDATOR['encoded'](full['manifest'])).hexdigest(),
                        index_tree=ready.COLD_VALIDATOR['logical_tree'](full['entries'], 'sha1'),
                        raw_tree=ready.COLD_VALIDATOR['raw_tree'](full['manifest'], 'sha1'))
                self.save()
                result = self.check()
                self.assertEqual(result['verdict'], verdict, result)
                if verdict != 'passed':
                    self.assertIn('execute permission', result['diagnostic'])

    def faulting_diff_git(self, abnormal):
        directory = self.root / 'git-bin'
        directory.mkdir()
        executable = directory / 'git'
        executable.write_text(f'''#!{sys.executable}
import os, signal, sys
signal.signal(signal.SIGCHLD, signal.SIG_DFL)
if 'diff-tree' in sys.argv[1:]:
    {'os.kill(os.getpid(), signal.SIGTERM)' if abnormal else 'raise SystemExit(7)'}
os.execv({shutil.which('git')!r}, ['git', *sys.argv[1:]])
''')
        executable.chmod(0o700)
        return str(directory) + os.pathsep + os.environ['PATH']

    def test_python_raw_source_needs_read_permission_without_execute(self):
        self.plan()
        self.assertEqual(self.check()['verdict'], 'passed')  # The bound Python file is 0644.
        filename = self.root / 'cold-input.json'
        full = json.loads(filename.read_text())
        cold = self.record['cold_tools']
        for mode, diagnostic in ((0o100, 'read permission'), (-1, 'invalid cold suite source permissions'),
                                 (0o10000, 'invalid cold suite source permissions')):
            with self.subTest(mode=mode):
                full['manifest'][0]['mode'] = mode
                filename.write_text(json.dumps(full))
                cold['input_manifest']['sha256'] = hashlib.sha256(filename.read_bytes()).hexdigest()
                for key in ('input', 'observed'):
                    cold[key].update(
                        manifest_sha256=hashlib.sha256(ready.COLD_VALIDATOR['encoded'](full['manifest'])).hexdigest(),
                        raw_tree=ready.COLD_VALIDATOR['raw_tree'](full['manifest'], 'sha1'))
                self.save()
                self.refuse(diagnostic)

    def ignored_readiness_diff(self, abnormal):
        self.advance('touched.md', 'overlapping main change\n')
        path = self.faulting_diff_git(abnormal)
        response = self.root / 'response.json'
        driver = """import importlib.machinery, importlib.util, json, signal, sys
from pathlib import Path
assert signal.getsignal(signal.SIGCHLD) == signal.SIG_IGN
loader = importlib.machinery.SourceFileLoader('ready_inherited', sys.argv[1])
spec = importlib.util.spec_from_loader(loader.name, loader)
module = importlib.util.module_from_spec(spec)
loader.exec_module(module)
result, repo, tip, response = sys.argv[2:]
sys.argv = [sys.argv[1], result, '--repo', repo, '--tip', tip]
status = module.main()
Path(response).write_text(json.dumps({'status': status, 'handler': int(signal.getsignal(signal.SIGCHLD))}))
raise SystemExit(status)
"""
        launcher = """import os, signal, sys
signal.signal(signal.SIGCHLD, signal.SIG_IGN)
os.execv(sys.executable, [sys.executable, '-I', '-B', '-c', *sys.argv[1:]])
"""
        result = subprocess.run([sys.executable, '-I', '-B', '-c', launcher, driver,
                                 str(DIRECTORY / 'ready-check'), str(self.result), str(self.repo), self.tip, str(response)],
                                env=dict(os.environ, PATH=path), stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        decision = json.loads(result.stdout)
        self.assertEqual(decision['verdict'], 'rerun-precheck')
        self.assertIn('diff-tree failed', decision['diagnostic'])
        self.assertEqual(json.loads(response.read_text()),
                         {'status': 1, 'handler': int(signal.SIG_DFL if abnormal else signal.SIG_IGN)})

    def test_ignored_sigchld_failed_empty_diff_tree_refuses(self):
        self.ignored_readiness_diff(False)

    def test_ignored_sigchld_signaled_empty_diff_tree_refuses(self):
        self.ignored_readiness_diff(True)

    def test_readiness_known_results_and_preflight_restore_exact_handler(self):
        def custom(number, frame):
            pass
        for handler in (signal.SIG_IGN, custom):
            for tip, verdict in ((self.tip, 'passed'), ('HEAD', 'rerun-precheck')):
                with self.subTest(handler=handler, tip=tip):
                    previous = signal.signal(signal.SIGCHLD, handler)
                    try:
                        result = ready.check(self.result, self.repo, tip)
                        self.assertEqual(result['verdict'], verdict, result)
                        self.assertIs(signal.getsignal(signal.SIGCHLD), handler)
                    finally:
                        signal.signal(signal.SIGCHLD, previous)

    def test_readiness_nonzero_git_restores_custom_handler(self):
        self.advance('touched.md', 'overlapping main change\n')
        path = self.faulting_diff_git(False)
        def custom(number, frame):
            pass
        previous = signal.signal(signal.SIGCHLD, custom)
        try:
            with patch.dict(os.environ, PATH=path):
                self.refuse('diff-tree failed')
            self.assertIs(signal.getsignal(signal.SIGCHLD), custom)
        finally:
            signal.signal(signal.SIGCHLD, previous)

    def test_remote_same_file_change_refuses_stale_tracking(self):
        current = self.advance("touched.md")
        result = self.refuse("touched.md")
        self.assertEqual((result["checkedbase"], result["currentmain"], result["overlap"]),
                         (self.base, current, ["touched.md"]))
        self.assertEqual(self.git(self.repo, "rev-parse", "origin/main"), self.base)

    def test_remote_disjoint_change_passes_without_tracking_update(self):
        current = self.advance()
        self.assertEqual((self.check()["verdict"], self.check()["currentmain"]), ("passed", current))
        self.assertEqual(self.git(self.repo, "rev-parse", "origin/main"), self.base)

    def test_remote_advance_during_check_refuses_without_sleep(self):
        original = ready.Repository.changed

        def advance_after_diff(repository, base, current):
            paths = original(repository, base, current)
            self.advance()
            return paths

        with patch.object(ready.Repository, "changed", advance_after_diff):
            result = self.refuse("advanced during check")
        self.assertNotEqual(result["currentmain"], result["finalmain"])

    def test_remote_advance_during_fetch_refuses_without_sleep(self):
        original = ready.Repository.git

        def advance_before_fetch(repository, *args, **kwargs):
            if args[0] == "fetch":
                self.advance()
            return original(repository, *args, **kwargs)

        with patch.object(ready.Repository, "git", advance_before_fetch):
            self.refuse("advanced during check")

    def test_renamed_deleted_and_literal_paths_overlap(self):
        names = ["space name.md", "$(touch BAD);*.md", ":(glob)*.md", "line\nbreak.md", "-option.md"]
        for name in names:
            self.write(self.writer, name, "base\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.repo = self.root / "literal candidate"
        self.git(self.root, "clone", str(self.origin), str(self.repo))
        self.git(self.repo, "config", "user.name", "Fixture")
        self.git(self.repo, "config", "user.email", "fixture@example.invalid")
        for name in names:
            self.write(self.repo, name, "candidate\n")
        self.commit(self.repo)
        self.tip = self.git(self.repo, "rev-parse", "HEAD")
        self.record.update(tip=self.tip, original=self.tip, slot=str(self.repo))
        self.record["base"] = self.git(self.writer, "rev-parse", "HEAD")
        self.record["paths"] = names
        self.save()
        self.git(self.writer, "mv", "--", names[0], "renamed.md")
        self.git(self.writer, "rm", "--", names[1])
        for name in names[2:]:
            self.write(self.writer, name, "changed\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        result = self.refuse("changed precheck files")
        self.assertEqual(result["overlap"], sorted(names))
        self.assertFalse((self.repo / "BAD").exists())
        self.record["paths"] = ["renamed.md"]
        self.save()
        self.assertEqual(self.refuse("renamed.md")["overlap"], ["renamed.md"])

    def test_reverted_main_change_still_requires_precheck(self):
        self.advance("touched.md")
        self.advance("touched.md", "base\n")
        self.refuse("touched.md")

    def test_merge_does_not_count_changes_already_in_checked_main(self):
        self.git(self.writer, "switch", "-c", "old-side")
        self.write(self.writer, "side.md", "side\n")
        self.commit(self.writer)
        self.git(self.writer, "switch", "main")
        base = self.advance("touched.md", "checked main\n")
        self.git(self.repo, "fetch", "origin")
        self.git(self.repo, "reset", "--hard", base)
        self.write(self.repo, "touched.md", "candidate on checked main\n")
        self.commit(self.repo)
        self.tip = self.git(self.repo, "rev-parse", "HEAD")
        self.record.update(base=base, tip=self.tip, original=self.tip)
        self.save()
        self.git(self.writer, "merge", "--no-ff", "-m", "Merge old side", "old-side")
        self.git(self.writer, "push", "origin", "HEAD:main")
        result = self.check()
        self.assertEqual(result["verdict"], "passed", result)
        self.assertEqual(result["overlap"], [])

    def test_merge_branch_changes_after_checked_main_still_refuse(self):
        self.git(self.writer, "switch", "-c", "side")
        self.write(self.writer, "touched.md", "side change\n")
        self.commit(self.writer)
        self.git(self.writer, "switch", "main")
        self.advance()
        self.git(self.writer, "merge", "--no-ff", "-m", "Merge side", "side")
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.refuse("touched.md")

    def test_checked_main_on_second_parent_requires_fresh_precheck(self):
        root = self.base
        base = self.advance()
        self.git(self.repo, "fetch", "origin")
        self.record["base"] = base
        self.save()
        self.git(self.writer, "switch", "-c", "new-main-line", root)
        self.write(self.writer, "side.md", "side\n")
        self.commit(self.writer)
        self.git(self.writer, "merge", "--no-ff", "-m", "Merge old main as side", "main")
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.refuse("first-parent")

    def test_merge_resolution_changes_are_seen(self):
        self.git(self.writer, "switch", "-c", "side")
        self.write(self.writer, "other.md", "side\n")
        self.commit(self.writer)
        self.git(self.writer, "switch", "main")
        self.write(self.writer, "extra.md", "main\n")
        self.commit(self.writer)
        self.git(self.writer, "merge", "--no-ff", "--no-commit", "side")
        self.write(self.writer, "touched.md", "merge resolution\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "origin", "HEAD:main")
        self.refuse("touched.md")

    def test_rewritten_or_removed_main_refuses(self):
        self.git(self.writer, "switch", "--orphan", "replacement")
        self.write(self.writer, "unrelated.md", "unrelated\n")
        self.commit(self.writer)
        self.git(self.writer, "push", "--force", "origin", "HEAD:main")
        self.refuse("rewrote history")
        self.git(self.origin, "update-ref", "-d", "refs/heads/main")
        self.refuse("ls-remote failed")

    def test_remote_unavailable_refuses(self):
        self.git(self.repo, "remote", "set-url", "origin", str(self.root / "missing.git"))
        self.refuse("ls-remote failed")

    def test_changed_tip_and_missing_objects_refuse(self):
        self.write(self.repo, "touched.md", "new tip\n")
        self.commit(self.repo)
        tip = self.git(self.repo, "rev-parse", "HEAD")
        result = ready.check(self.result, self.repo, tip)
        self.assertIn("tip mismatch", result["diagnostic"])
        self.tip = "1" * 40
        self.record["tip"] = self.tip
        self.save()
        self.refuse("rev-parse failed")

    def test_missing_base_object_refuses(self):
        self.record["base"] = "2" * 40
        self.save()
        self.refuse("rev-parse failed")

    def test_tip_must_be_full_literal_oid(self):
        for value in ("HEAD", self.tip[:12], "--help", "$(touch BAD)", self.tip.upper(), "0" * 39):
            with self.subTest(value=value):
                self.assertIn("full lowercase", ready.check(self.result, self.repo, value)["diagnostic"])

    def test_failed_unrestored_unknown_or_incomplete_evidence_refuses(self):
        self.plan()
        valid = deepcopy(self.record)
        patches = [({"outcome": "failed"}, "passed"), ({"restored": False}, "restoration"),
                   ({"restored": 1}, "restoration"), ({"plan": [None]}, "planned"),
                   ({"steps": []}, "incomplete"), ({"active_command": None}, "active-command"),
                   ({"active_command": {"completion": "unknown"}}, "active-command"),
                   ({"selection": None}, "selection"), ({"paths": None}, "paths"),
                   ({"error": "failed"}, "unresolved"), ({"base": self.base[:12]}, "full precheck"),
                   ({"started": "tomorrow"}, "time"), ({"heavy_observation": ""}, "heavy"),
                   ({"slot": "/a/../b"}, "slot")]
        for changes, diagnostic in patches:
            with self.subTest(changes=changes):
                self.record = dict(deepcopy(valid), **changes)
                self.save()
                self.refuse(diagnostic)

    def test_step_plan_and_completion_contradictions_refuse(self):
        self.plan()
        valid = deepcopy(self.record)
        mutations = [("steps", "returncode", 137), ("steps", "green", False),
                     ("steps", "duration", -1), ("steps", "command", ["other"]),
                     ("steps", "cwd", "/wrong"), ("plan", "timeout", -1),
                     ("plan", "cwd", "/wrong")]
        for section, key, value in mutations:
            with self.subTest(section=section, key=key):
                self.record = deepcopy(valid)
                self.record[section][0][key] = value
                self.save()
                self.assertEqual(self.check()["verdict"], "rerun-precheck")
        self.record = deepcopy(valid)
        self.record["active_command"]["returncode"] = 4
        self.save()
        self.refuse("active-command")

    def test_missing_executed_command_cwd_requires_fresh_evidence(self):
        self.plan()
        del self.record["steps"][0]["cwd"]
        self.save()
        self.refuse("malformed or failed precheck step")

    def test_dangerous_and_duplicate_scopes_refuse(self):
        for paths in (["../escape"], ["/absolute"], ["a/./b"], ["a//b"], [".git/config"],
                      ["a/.GiT/b"], ["nul\0name"], [""], ["same", "same"], [42]):
            with self.subTest(paths=paths):
                self.record["paths"] = paths
                self.save()
                self.assertEqual(self.check()["verdict"], "rerun-precheck")

    def test_malformed_duplicate_or_missing_json_refuses(self):
        for data in ("{", "[]", "null", '{"outcome":"passed","outcome":"passed"}',
                     '{"restored":NaN}', "{}"):
            with self.subTest(data=data):
                self.result.write_text(data)
                self.assertEqual(self.check()["verdict"], "rerun-precheck")
        self.result.unlink()
        self.refuse("No such file")

    def test_completed_ratchet_only_evidence_passes(self):
        argv = ["python3", "-I", "-B", f"{self.repo}/skein/scripts/old_forms.py", "--write"]
        self.record.update(paths=["skein/docs/example.md"],
                           steps=[{"name": "ratchet:skein", "command": argv, "returncode": 0,
                                   "green": True, "duration": 0, "cwd": f"{self.repo}/skein"}],
                           active_command={"argv": argv, "cwd": f"{self.repo}/skein",
                                           "completion": "completed", "returncode": 0})
        self.save()
        self.assertEqual(self.check()["verdict"], "passed")
        self.record["steps"] = []
        self.save()
        self.refuse("ratchet")

    def test_no_command_run_with_active_command_refuses(self):
        self.record["active_command"] = {"completion": "unknown"}
        self.save()
        self.refuse("active command")

    def test_incomplete_compile_plan_and_unexpected_commands_refuse(self):
        self.record["selection"] = ["fixture"]
        self.save()
        self.refuse("compile/lint plan")
        self.record["selection"] = "docs"
        self.plan()
        self.record["plan"][0]["name"] = "unknown"
        self.save()
        self.refuse("unknown planned command")
        self.plan()
        self.record["steps"].insert(0, {"name": "extra", "command": ["true"],
                                        "returncode": 0, "green": True, "duration": 0, "cwd": str(self.repo)})
        self.save()
        self.refuse("outside precheck plan")

    def compile_plan(self, selection):
        self.plan()
        self.record["selection"] = selection
        for name, argv, code in (("clippy", ["heavy", "cargo", "clippy", "-p", "fixture"], 0),
                                 ("build", ["heavy", "cargo", "nextest", "run", "--no-run"], 4)):
            self.record["plan"].append({"name": name, "argv": argv, "cwd": str(self.repo), "timeout": None})
            self.record["steps"].append({"name": name, "command": argv, "green": True,
                                         "returncode": code, "duration": 0.01, "cwd": str(self.repo)})
            self.record["active_command"] = {"argv": argv, "cwd": str(self.repo),
                                             "completion": "completed", "returncode": code}
        self.save()

    def test_completed_planned_tools_and_build_exit_four_pass(self):
        self.plan()
        self.assertEqual(self.check()["verdict"], "passed")
        self.compile_plan(["fixture"])
        self.assertEqual(self.check()["verdict"], "passed")

    def test_full_selection_accepts_workspace_format_command(self):
        self.compile_plan("full")
        argv = ["cargo", "fmt", "--all", "--check"]
        self.record["plan"].insert(0, {"name": "fmt", "argv": argv, "cwd": str(self.repo), "timeout": None})
        self.record["steps"].insert(0, {"name": "fmt", "command": argv, "green": True,
                                        "returncode": 0, "duration": 0.01, "cwd": str(self.repo)})
        self.save()
        self.assertEqual(self.check()["verdict"], "passed")

    def snapshot(self, repo):
        files = {}
        for filename in repo.rglob("*"):
            if filename.is_file() and ".git" not in filename.relative_to(repo).parts:
                files[str(filename.relative_to(repo))] = (filename.read_bytes(), filename.stat().st_mode)
        return (self.git(repo, "symbolic-ref", "HEAD"), self.git(repo, "rev-parse", "HEAD"),
                self.git(repo, "for-each-ref", "--format=%(refname) %(objectname)"),
                (repo / ".git/index").read_bytes(), files, (repo / ".git/HEAD").read_bytes())

    def test_cli_preserves_source_index_refs_and_origin(self):
        self.plan()
        self.advance()
        self.write(self.repo, "staged.md", "staged\n")
        self.git(self.repo, "add", "staged.md")
        self.write(self.repo, "touched.md", "dirty source\n")
        self.write(self.repo, "untracked ; *.md", "untracked\n")
        self.git(self.repo, "update-index", "--assume-unchanged", "other.md")
        before = self.snapshot(self.repo)
        bytecode = set((DIRECTORY / "__pycache__").glob("*"))
        remote = self.git(self.origin, "for-each-ref", "--format=%(refname) %(objectname)")
        result = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY / "ready-check"),
                                 str(self.result), "--repo", str(self.repo), "--tip", self.tip],
                                text=True, capture_output=True, stdin=subprocess.DEVNULL, env=self.env)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["verdict"], "passed")
        self.assertEqual(self.snapshot(self.repo), before)
        self.assertEqual(self.git(self.origin, "for-each-ref", "--format=%(refname) %(objectname)"), remote)
        self.assertFalse((self.repo / ".git/FETCH_HEAD").exists())
        self.assertEqual(set((DIRECTORY / "__pycache__").glob("*")), bytecode)
        self.record["restored"] = False
        self.save()
        result = subprocess.run([sys.executable, "-I", "-B", str(DIRECTORY / "ready-check"),
                                 str(self.result), "--repo", str(self.repo), "--tip", self.tip],
                                text=True, capture_output=True, stdin=subprocess.DEVNULL, env=self.env)
        self.assertEqual(result.returncode, 1)
        self.assertEqual(json.loads(result.stdout)["verdict"], "rerun-precheck")
        self.assertIn("precheckbase=", result.stderr)
        self.assertEqual(self.snapshot(self.repo), before)

    def test_shallow_replace_refs_and_git_overrides_refuse(self):
        self.git(self.repo, "replace", self.tip, self.base)
        self.refuse("replace refs")
        self.git(self.repo, "replace", "-d", self.tip)
        with patch.dict(os.environ, GIT_INDEX_FILE=str(self.root / "other.index")):
            self.refuse("Git environment override")
        self.git(self.root, "clone", "--depth=1", self.origin.as_uri(), str(self.root / "shallow"))
        self.assertIn("shallow", ready.check(self.result, self.root / "shallow", self.tip)["diagnostic"])


if __name__ == "__main__":
    unittest.main()
