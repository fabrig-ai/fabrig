"""Tiny cold tools fixtures: prepared Git, literal artifacts and retained custody."""

import sys
sys.dont_write_bytecode = True

from contextlib import redirect_stdout
import importlib.machinery
import importlib.util
import io
import json
import os
from pathlib import Path
import signal
import stat
import subprocess
import unittest
import threading
from types import SimpleNamespace
from unittest.mock import patch

DIRECTORY = Path(__file__).resolve().parent
LOADER = importlib.machinery.SourceFileLoader('cold_fixture', str(DIRECTORY / 'test_precheck.py'))
SPEC = importlib.util.spec_from_loader(LOADER.name, LOADER)
fixtures = importlib.util.module_from_spec(SPEC)
LOADER.exec_module(fixtures)
p = fixtures.precheck
cold = p.COLD_TOOLS


class ColdTests(fixtures.Fixture):
    def run_tool(self, **options):
        real_popen = subprocess.Popen
        def offline(argv, **kwargs):
            if argv[0] in ('cargo', 'heavy'):
                raise AssertionError('cold fixture must inject Cargo/heavy checks')
            return real_popen(argv, **kwargs)
        with patch.object(p.subprocess, 'Popen', side_effect=offline):
            return super().run_tool(**options)

    def snapshot(self):
        return {'head': (self.repo / '.git/HEAD').read_bytes(), 'refs': self.git('show-ref'),
                'index': (self.repo / '.git/index').read_bytes(),
                'source': {n: ((self.repo / n).read_bytes(), stat.S_IMODE((self.repo / n).stat().st_mode))
                           for n in self.git('ls-files', '-z').split('\0') if n},
                'config': (self.repo / '.git/config').read_bytes(),
                'warm': (self.repo / 'target/keep').read_bytes(),
                'warm_identity': [(self.repo / 'target').stat().st_dev, (self.repo / 'target').stat().st_ino]}

    def checked(self, before):
        self.assertEqual(self.snapshot(), before)
        self.restored()
        r = self.tool.record['cold_tools']
        self.assertEqual((r['cleanup'], r['completion'], r['state']), ('verified', 'known', 'removed'))
        self.assertFalse(Path(r['root']).exists())
        self.assertEqual(r['input'], {k: r['observed'][k] for k in r['input']})
        self.assertLessEqual(r['capacity']['copied_allocated_bytes'], r['capacity']['required_bytes'])

    def suite(self, body):
        self.change('tip', 'tools/test_fixture.py', body)

    def test_warm_literal_and_environment_helper_cannot_mask_cold_failure(self):
        self.write('target/helper', 'warm helper')
        self.suite("""import os
from pathlib import Path
raise SystemExit(0 if Path('target/helper').exists() or Path(os.environ.get('TIERS_CPU_WRAPPER', 'target/missing')).exists() else 7)
""")
        self.git('switch', 'tip')
        direct = subprocess.run([sys.executable, '-I', '-B', 'tools/test_fixture.py'], cwd=self.repo,
                                stdin=subprocess.DEVNULL)
        self.assertEqual(direct.returncode, 0)
        self.git('switch', 'original')
        before = self.snapshot()
        with patch.dict(os.environ, TIERS_CPU_WRAPPER=str(self.repo / 'target/helper'),
                        TIERS_CPU_SETTLEMENT_FIXTURE=str(self.repo / 'target/helper'),
                        CARGO_TARGET_DIR=str(self.repo / 'target')):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertEqual(self.tool.record['outcome'], 'failed')
        self.assertEqual(self.tool.record['steps'][0]['returncode'], 7)
        self.assertIn('TIERS_CPU_WRAPPER', self.tool.record['cold_tools']['environment']['removed'])
        self.assertEqual((self.repo / 'target/helper').read_text(), 'warm helper')
        self.checked(before)

    def test_private_helper_passes_and_real_cold_receipt_is_ready_evidence(self):
        self.suite("""import tempfile
from pathlib import Path
assert not Path('target').exists()
with tempfile.TemporaryDirectory() as d:
    p = Path(d) / 'helper'
    p.write_text('private')
    assert p.read_text() == 'private'
""")
        before = self.snapshot()
        self.assertEqual(self.run_tool(production_runner=True), 0)
        self.checked(before)
        loader = importlib.machinery.SourceFileLoader('cold_ready', str(DIRECTORY / 'ready-check'))
        spec = importlib.util.spec_from_loader(loader.name, loader)
        ready = importlib.util.module_from_spec(spec); loader.exec_module(ready)
        record = ready.evidence(self.tool.log_directory / 'result.json', self.tool.record['tip'])
        self.assertEqual(record['cold_tools']['state'], 'removed')

    def test_tools_scope_without_discovered_suites_still_binds_cold_receipt(self):
        self.change('tip', 'tools/README.md', 'tools input\n')
        before = self.snapshot()
        self.assertEqual(self.run_tool(), 0)
        self.assertEqual(self.tool.record['plan'], [])
        self.assertEqual(self.tool.record['cold_tools']['tools_plan'], [])
        self.checked(before)

    def test_tools_environment_and_cwd_do_not_change_original_cargo(self):
        self.change('tip', 'Cargo.toml', 'full fixture\n')
        self.suite('# fixture\n')
        observed = []
        original = self.runner
        def runner(argv, **options):
            observed.append((argv, options['cwd'], options.get('env')))
            return original(argv, **options)
        with patch.dict(os.environ, CARGO_TARGET_DIR=str(self.repo / 'target')), \
                patch.object(self, 'runner', side_effect=runner):
            self.assertEqual(self.run_tool(), 0)
            self.assertEqual(os.environ['CARGO_TARGET_DIR'], str(self.repo / 'target'))
        self.assertEqual(observed[0][1], Path(self.tool.record['cold_tools']['checkout']))
        self.assertNotIn('CARGO_TARGET_DIR', observed[0][2])
        self.assertEqual([argv[0] for argv, _, _ in observed[1:]], ['cargo', 'heavy', 'heavy'])
        self.assertTrue(all(cwd == self.repo and env is None for _, cwd, env in observed[1:]))
        self.restored()

    def test_staged_index_unstaged_ratchet_history_and_raw_bytes_are_separate(self):
        historic = self.ancestor
        self.change('tip', 'jig/added.md', 'staged addition\n')
        self.suite(f"""import subprocess
from pathlib import Path
assert Path('jig/added.md').read_text() == 'staged addition\\n'
assert Path('jig/scripts/old-forms.txt').read_text() == 'ceiling 7\\n'
assert subprocess.check_output(['git','show',':jig/scripts/old-forms.txt']) == b'ceiling 10\\n'
assert subprocess.check_output(['git','show','{historic}:README.md']) == b'base\\n'
assert Path('.git/MERGE_HEAD').exists()
assert not Path('target').exists()
""")
        self.effect = lambda argv: self.write('jig/scripts/old-forms.txt', 'ceiling 7\n') if '--write' in argv else None
        fake = self.runner
        def runner(argv, **options):
            return fake(argv, **options) if '--write' in argv else subprocess.run(argv, **options)
        before = self.snapshot()
        with patch.object(self, 'runner', side_effect=runner):
            self.assertEqual(self.run_tool(), 0)
        r = self.tool.record['cold_tools']
        self.assertNotEqual(r['input']['index_tree'], r['input']['raw_tree'])
        self.checked(before)

    def test_raw_tree_handles_literal_newline_paths_and_crlf_without_filters(self):
        self.change('tip', 'tools/.gitattributes', '*.bin -text\n')
        self.change('tip', 'tools/line\nbreak.bin', 'raw\r\nbytes\r\n')
        self.suite("from pathlib import Path\nassert Path('tools/line\\nbreak.bin').read_bytes() == b'raw\\r\\nbytes\\r\\n'\n")
        before = self.snapshot()
        self.assertEqual(self.run_tool(production_runner=True), 0, str(self.tool.record))
        self.checked(before)

    def test_before_copy_capacity_refusal_restores_without_tools_or_aux(self):
        self.suite('# fixture\n')
        before = self.snapshot()
        with patch.object(cold['shutil'], 'disk_usage', return_value=type('Usage', (), {'free': 1})()):
            self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.calls, [])
        r = self.tool.record['cold_tools']
        self.assertFalse(r['capacity']['admitted'])
        self.assertIsNone(r['root_identity'])
        self.assertFalse(Path(r['root']).exists())
        self.assertEqual(self.snapshot(), before)
        self.restored()

    def test_unexpected_target_from_first_suite_never_warms_second(self):
        self.change('tip', 'tools/test_a.py', "from pathlib import Path\nPath('target').mkdir()\nPath('target/helper').write_text('bad')\n")
        self.change('tip', 'tools/test_b.py', "raise AssertionError('must not run warm')\n")
        before = self.snapshot()
        self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertIn('generated target', self.tool.record['error'])
        self.assertEqual(len(self.tool.record['steps']), 1)
        self.checked(before)

    def test_escaping_tracked_symlink_refuses_before_copy(self):
        self.git('switch', 'tip')
        (self.repo / 'tools').mkdir()
        (self.repo / 'tools/escape').symlink_to(self.repo / 'target')
        self.write('tools/test_fixture.py', '# fixture\n')
        self.commit(); self.git('switch', 'original')
        self.assertEqual(self.run_tool(), 1)
        self.assertIn('symlink escapes', self.tool.record['error'])
        self.assertEqual(self.calls, [])
        self.restored()

    def test_borrowed_objects_refuse_without_following_or_mutating_store(self):
        self.suite('# fixture\n')
        alternate = self.repo / '.git/objects/info/alternates'
        (self.root / 'unused-object-store').mkdir()
        alternate.write_text(str(self.root / 'unused-object-store') + '\n')
        self.assertEqual(self.run_tool(), 1)
        self.assertIn('self-contained', self.tool.record['error'])
        self.assertEqual(alternate.read_text(), str(self.root / 'unused-object-store') + '\n')
        self.restored()

    def test_cleanup_failure_retains_marker_and_aux_and_refuses_reuse(self):
        home = fixtures.SlotMaintenanceTests.numeric_slot(self)
        self.suite('# fixture\n')
        cls = cold['ColdTools']
        with patch.dict(os.environ, HOME=str(home)), patch.object(cls, 'cleanup', side_effect=OSError('fixture cleanup denied')):
            self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record['outcome'], 'unresolved')
        self.assertTrue(self.tool.marker.exists())
        self.assertTrue(Path(self.tool.record['cold_tools']['root']).exists())
        self.assertEqual(len(list((home / 'src/fabrig/slot-maintenance/1').glob('*.hold'))), 1)
        with self.assertRaisesRegex(RuntimeError, 'durable unresolved'):
            self.run_tool()

    def test_imposed_timeout_keeps_aux_and_candidate(self):
        home = fixtures.SlotMaintenanceTests.numeric_slot(self)
        self.suite('# fixture\n')
        def timeout(argv, **options):
            raise subprocess.TimeoutExpired(argv, 60)
        with patch.dict(os.environ, HOME=str(home)), patch.object(self, 'runner', side_effect=timeout):
            self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record['outcome'], 'unresolved')
        self.assertTrue(self.tool.executor.unknown)
        self.assertTrue(self.tool.marker.exists())
        self.assertTrue(Path(self.tool.record['cold_tools']['root']).exists())
        self.assertEqual((self.repo / 'tools/test_fixture.py').read_text(), '# fixture\n')
        self.assertEqual(len(list((home / 'src/fabrig/slot-maintenance/1').glob('*.hold'))), 1)

    def test_executor_timeout_drains_only_immediate_child_and_retains_custody(self):
        self.suite('# fixture\n')
        real_popen = subprocess.Popen
        events = []
        class Child:
            returncode = None
            def wait(self, timeout=None):
                events.append(('wait', timeout))
                if timeout is not None:
                    raise subprocess.TimeoutExpired('fixture', timeout)
                return self.returncode
            def kill(self):
                events.append(('kill',))
                self.returncode = -signal.SIGKILL
        def launch(argv, **options):
            return Child() if argv[:3] == ['python3', '-I', '-B'] else real_popen(argv, **options)
        with patch.object(p.subprocess, 'Popen', side_effect=launch):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertEqual(events, [('wait', 60), ('kill',), ('wait', None)])
        self.assertTrue(self.tool.executor.unknown)
        self.assertTrue(self.tool.marker.exists())
        self.assertTrue(Path(self.tool.record['cold_tools']['root']).exists())

    def test_unavailable_descendant_authority_retains_inputs_without_launch(self):
        self.suite('# fixture\n')
        with patch.object(p, 'subreaper', side_effect=OSError('authority unavailable')):
            self.assertEqual(self.run_tool(production_runner=True), 1)
        self.assertEqual(self.tool.record['outcome'], 'unresolved')
        self.assertEqual(self.tool.record['steps'], [])
        self.assertTrue(self.tool.marker.exists())
        self.assertTrue(Path(self.tool.record['cold_tools']['root']).exists())

    def inherited_ignored_status(self, body, *, abnormal=False):
        self.suite(body)
        result_file = self.root / 'inherited-status.json'
        driver = """import importlib.machinery, importlib.util, json, signal, subprocess, sys
from pathlib import Path
assert signal.getsignal(signal.SIGCHLD) == signal.SIG_IGN
loader = importlib.machinery.SourceFileLoader('inherited_fixture', sys.argv[1])
spec = importlib.util.spec_from_loader(loader.name, loader)
fixture = importlib.util.module_from_spec(spec)
loader.exec_module(fixture)
executor = fixture.precheck.Executor(lambda *a: None, lambda *a: None)
try:
    probe = executor([sys.executable, '-I', '-B', '-c', sys.argv[4]], cwd=Path(sys.argv[2]),
                     timeout=60, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    direct = {'returncode': probe.returncode, 'unknown': executor.unknown}
except fixture.precheck.UnknownCompletion as error:
    direct = {'unknown': executor.unknown, 'error': str(error)}
signal.signal(signal.SIGCHLD, signal.SIG_IGN)
tool = fixture.precheck.Precheck(Path(sys.argv[2]), 'base', 'tip', mq=fixture.mq,
    always=[], read_by_tests=[], busy=lambda _: 'offline inherited-ignore fixture', min_free_bytes=1)
status = tool.run()
Path(sys.argv[3]).write_text(json.dumps({'status': status, 'record': tool.record, 'direct': direct}))
raise SystemExit(status)
"""
        launcher = """import os, signal, sys
signal.signal(signal.SIGCHLD, signal.SIG_IGN)
os.execv(sys.executable, [sys.executable, '-I', '-B', '-c', *sys.argv[1:]])
"""
        before = self.snapshot()
        result = subprocess.run([sys.executable, '-I', '-B', '-c', launcher, driver,
                                 str(DIRECTORY / 'test_precheck.py'), str(self.repo), str(result_file), body],
                                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.assertTrue(result_file.exists(), result.stdout.decode() + result.stderr.decode())
        payload = json.loads(result_file.read_text())
        if abnormal:
            self.assertTrue(payload['direct']['unknown'], payload['direct'])
            self.assertIn(f'checker exited {-signal.SIGTERM}', payload['direct']['error'])
        else:
            self.assertEqual(payload['direct']['returncode'], 7)
            self.assertFalse(payload['direct']['unknown'])
        self.assertEqual(result.returncode, 1, result.stdout.decode() + result.stderr.decode())
        self.assertEqual(payload['status'], 1)
        return before, payload['record']

    def test_inherited_ignored_sigchld_preserves_nonzero_suite_exit(self):
        before, record = self.inherited_ignored_status('raise SystemExit(7)\n')
        self.assertEqual(record['outcome'], 'failed')
        self.assertEqual(record['steps'][0]['returncode'], 7)
        self.assertFalse(record['steps'][0]['green'])
        self.assertTrue(record['restored'])
        self.assertEqual(self.snapshot(), before)
        self.assertFalse((self.repo / '.git/precheck.unresolved').exists())
        self.assertFalse(Path(record['cold_tools']['root']).exists())

    def test_inherited_ignored_sigchld_preserves_signal_suite_exit(self):
        _, record = self.inherited_ignored_status(
            'import os, signal\nos.kill(os.getpid(), signal.SIGTERM)\n', abnormal=True)
        self.assertEqual(record['outcome'], 'unresolved')
        self.assertIn(f'checker exited {-signal.SIGTERM}', record['error'])
        self.assertFalse(record['restored'])
        self.assertEqual(record['cold_tools']['completion'], 'unknown')
        self.assertTrue((self.repo / '.git/precheck.unresolved').exists())
        self.assertTrue((self.repo / 'tools/test_fixture.py').exists())
        self.assertTrue(Path(record['cold_tools']['checkout']).exists())
        self.assertEqual((self.repo / 'target/keep').read_text(), 'warm target\n')

    def test_unavailable_waitable_status_refuses_before_git(self):
        before = self.snapshot()
        tool = p.Precheck(self.repo, 'base', 'tip', mq=fixtures.mq, always=[], read_by_tests=[])
        with patch.object(p.signal, 'signal', side_effect=OSError('unsupported disposition')), \
                patch.object(tool, 'git') as git:
            with self.assertRaisesRegex(p.InfrastructureError, 'waitable child status'):
                tool.run()
            git.assert_not_called()
        self.assertEqual(self.snapshot(), before)
        self.assertFalse((self.repo / '.git/precheck.unresolved').exists())

    def test_drained_signal_cleans_aux_then_restores(self):
        self.suite('# fixture\n')
        before = self.snapshot()
        original = self.runner
        def drained(argv, **options):
            result = original(argv, **options)
            self.tool.executor.interrupted = signal.SIGTERM
            return result
        restore = p.Precheck.restore
        def restoring(tool):
            self.assertEqual(tool.record['cold_tools']['cleanup'], 'verified')
            self.assertFalse(Path(tool.record['cold_tools']['root']).exists())
            restore(tool)
        with patch.object(self, 'runner', side_effect=drained), patch.object(p.Precheck, 'restore', restoring):
            self.assertEqual(self.run_tool(), 1)
        self.assertEqual(self.tool.record['outcome'], 'infrastructure-error')
        self.checked(before)

    def test_detached_session_descendant_retains_inputs_until_known_terminal(self):
        home = fixtures.SlotMaintenanceTests.numeric_slot(self)
        release = self.root / 'release.fifo'
        os.mkfifo(release, 0o600)
        pidfile, done = self.root / 'child.pid', self.root / 'child.done'
        child_code = ("import os,sys; from pathlib import Path; "
                      "f=os.open(sys.argv[2],os.O_RDWR); os.write(int(sys.argv[1]),b'R'); "
                      "os.close(int(sys.argv[1])); os.read(f,1); os.close(f); "
                      "body=Path('tools/test_fixture.py').read_bytes(); "
                      "Path(sys.argv[3]).write_bytes(body)")
        self.suite(f"""import os,subprocess,sys
from pathlib import Path
r,w=os.pipe()
c=subprocess.Popen([sys.executable,'-c',{child_code!r},str(w),{str(release)!r},{str(done)!r}],
                   pass_fds=(w,),start_new_session=True,stdin=subprocess.DEVNULL,
                   stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
os.close(w)
assert os.read(r,1)==b'R'
os.close(r)
Path({str(pidfile)!r}).write_text(str(c.pid))
""")
        previous = p.subreaper()
        p.subreaper(1)
        try:
            with patch.dict(os.environ, HOME=str(home)):
                self.assertEqual(self.run_tool(production_runner=True), 1)
            self.assertEqual(self.tool.record['outcome'], 'unresolved')
            self.assertEqual(p.subreaper(), 1)
            self.assertTrue(self.tool.marker.exists())
            self.assertTrue(Path(self.tool.record['cold_tools']['checkout']).exists())
            self.assertTrue((self.repo / 'tools/test_fixture.py').exists())
            self.assertEqual(len(list((home / 'src/fabrig/slot-maintenance/1').glob('*.hold'))), 1)
            self.assertFalse(done.exists())
        finally:
            if pidfile.exists():
                with release.open('wb', buffering=0) as output:
                    output.write(b'X')
                _, status = os.waitpid(int(pidfile.read_text()), 0)
                self.assertEqual(os.waitstatus_to_exitcode(status), 0)
            p.subreaper(previous)
        self.assertTrue(done.exists())
        self.assertTrue(self.tool.marker.exists())  # Terminal child does not reconcile custody.

    def test_cleanup_root_replacement_after_verification_is_never_deleted(self):
        self.suite('# fixture\n')
        verified, replaced = threading.Event(), threading.Event()
        errors = []
        old_save = cold['ColdTools'].save
        once = []
        paths = {}
        def save(aux):
            old_save(aux)
            if aux.receipt['state'] == 'removing' and not once:
                once.append(True)
                paths['root'] = aux.root
                paths['aside'] = aux.root.with_name('owned-aside')
                verified.set()
                if not replaced.wait(5):
                    raise RuntimeError('replacement barrier timed out')
        def replace():
            try:
                if not verified.wait(5):
                    raise RuntimeError('verification barrier timed out')
                paths['root'].rename(paths['aside'])
                paths['root'].mkdir()
                (paths['root'] / 'foreign').write_text('preserve replacement')
            except BaseException as error:
                errors.append(error)
            finally:
                replaced.set()
        writer = threading.Thread(target=replace)
        writer.start()
        try:
            with patch.object(cold['ColdTools'], 'save', save):
                self.assertEqual(self.run_tool(), 1)
        finally:
            writer.join(5)
        self.assertFalse(writer.is_alive())
        self.assertEqual(errors, [])
        self.assertEqual((paths['root'] / 'foreign').read_text(), 'preserve replacement')
        self.assertTrue(paths['aside'].exists())
        self.assertEqual(self.tool.record['outcome'], 'unresolved')
        self.assertTrue(self.tool.marker.exists())

    def test_deep_empty_paths_packed_history_refuse_before_aux_allocation(self):
        depth = 220
        self.change('tip', 'tools/' + 'd/' * depth + 'empty', '')
        self.suite('# fixture\n')
        self.git('gc', '--prune=now')
        before = self.snapshot()
        measured = cold['static_copy_requirement']
        observed = {}
        def measure(rows, objects, logical, full_bytes, block):
            result = measured(rows, objects, logical, full_bytes, block)
            rounded = lambda size: ((size + block - 1) // block) * block
            immediate = {str(Path(row['name']).parent) for row in rows}
            old = 2 * sum(rounded(row.get('size', 0)) for row in rows)
            old += sum(rounded(size) for _, _, size in objects)
            old += block * (len(rows) + len(objects) + len(immediate) + 32)
            old += 2 * rounded(len(os.fsencode(logical)))
            observed.update(old=old, new=result, free=old + block * 2 + 1)
            return result
        with patch.dict(cold, static_copy_requirement=measure), \
                patch.object(cold['shutil'], 'disk_usage', side_effect=lambda _: SimpleNamespace(free=observed['free'])):
            self.assertEqual(self.run_tool(), 1)
        self.assertGreater(observed['new']['directory_count'], depth)
        self.assertGreater(observed['new']['required_bytes'], observed['free'])
        self.assertGreater(observed['free'], observed['old'])  # The prior bound would admit.
        self.assertEqual(self.calls, [])
        self.assertIsNone(self.tool.record['cold_tools']['root_identity'])
        self.assertFalse((self.tool.log_directory / 'cold-input.json').exists())
        self.assertEqual(self.snapshot(), before)
        self.restored()

    def test_linked_worktree_common_grafts_refuse_before_preparation(self):
        self.suite('# fixture\n')
        linked = self.root / 'linked'
        self.git('worktree', 'add', '-b', 'linked-fixture', str(linked), 'original')
        gitdir = Path(subprocess.check_output(['git', '-C', str(linked), 'rev-parse', '--absolute-git-dir'], text=True).strip())
        resolved = Path(subprocess.check_output(['git', '-C', str(linked), 'rev-parse', '--path-format=absolute',
                                                '--git-path', 'info/grafts'], text=True).strip())
        self.assertNotEqual(resolved, gitdir / 'info/grafts')
        self.git('config', 'advice.graftFileDeprecated', 'false')
        resolved.write_text(self.ancestor + '\n')
        original_index = (gitdir / 'index').read_bytes()
        original_head = (gitdir / 'HEAD').read_bytes()
        tool = p.Precheck(linked, 'base', 'tip', mq=fixtures.mq, always=[], read_by_tests=[],
                          runner=lambda *a, **k: self.fail('no checks before graft refusal'), busy=lambda _: 'fixture',
                          min_free_bytes=1)
        with patch.object(tool, 'prepare', side_effect=AssertionError('grafted preparation ran')) as prepare:
            with self.assertRaisesRegex(RuntimeError, 'common-directory grafts'):
                tool.run()
            prepare.assert_not_called()
        self.assertEqual((gitdir / 'HEAD').read_bytes(), original_head)
        self.assertEqual((gitdir / 'index').read_bytes(), original_index)
        self.assertFalse((gitdir / 'precheck.unresolved').exists())


if __name__ == '__main__':
    os.umask(0o077)
    unittest.main()
