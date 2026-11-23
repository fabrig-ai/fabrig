"""Delivery policy checks; every process boundary is a recording fake."""
from contextlib import redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock


def module(path):
    spec = importlib.util.spec_from_file_location(path.stem, path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


HERE = Path(__file__).resolve().parent
PRE = module(HERE / 'pre_pr.py')
PLAN = module(HERE / 'workspace.py')
GATE = module(HERE.parents[1] / '.forgejo/scripts/gate_scope.py')


def setUpModule():
    guard = mock.patch.object(subprocess, 'Popen', side_effect=AssertionError('unexpected real subprocess'))
    guard.start()
    unittest.addModuleCleanup(guard.stop)


class Packages(unittest.TestCase):
    def setUp(self):
        self.root = Path('/workspace')
        self.packages = {}
        for name, dep in [('leaf', None), ('middle', 'leaf'), ('top', 'middle')]:
            directory = self.root / 'skein' / name
            self.packages[name] = dict(name=name, manifest_path=str(directory / 'Cargo.toml'),
                                       dependencies=[{'path': str(directory.parent / dep)}] if dep else [],
                                       targets=[dict(name=target, kind=[kind], test=True, src_path=str(directory / path))
                                                for target, kind, path in [(name, 'lib', 'src/lib.rs'),
                                                                           ('suite', 'test', 'tests/suite.rs'),
                                                                           ('live', 'test', 'tests/live.rs')]])
        self.layers = {'skein': {name + '::live': 'end-to-end' for name in self.packages}}

    def plan(self, *paths):
        return PLAN.package_plan(self.root, paths, self.packages, self.layers)

    def test_only_changed_crate_and_direct_consumers(self):
        self.assertEqual(self.plan('skein/leaf/src/item.rs'), {(('--lib',), ('--test', 'suite')): ['leaf', 'middle']})

    def test_one_integration_test_uses_one_binary(self):
        groups = self.plan('skein/leaf/tests/suite.rs')
        self.assertEqual(list(PLAN.rust_commands(groups, {})),
                         [['cargo', 'nextest', 'run', '--offline', '--locked', '--profile', 'default',
                           '--no-tests=warn', '-p', 'leaf', '--test', 'suite']])

    def test_nested_test_helper_stays_with_its_owner(self):
        self.assertEqual(list(self.plan('skein/leaf/tests/shared/input.rs').values()), [['leaf']])

    def test_unknown_root_file_and_later_tier_do_not_build(self):
        self.assertEqual(self.plan('tools/pre-pr/pre_pr.py', 'skein/top/tests/live.rs'), {})

    def test_changed_dependency_also_widens_a_changed_test_owner(self):
        self.assertEqual(self.plan('skein/leaf/src/lib.rs', 'skein/middle/tests/suite.rs'),
                         {(('--lib',), ('--test', 'suite')): ['leaf', 'middle']})

    def test_changed_product_crate_without_test_targets_still_compiles(self):
        self.packages['top']['targets'] = []
        self.assertEqual(list(PLAN.rust_commands(self.plan('skein/top/src/lib.rs'), {})),
                         [['cargo', 'check', '--offline', '--locked', '-p', 'top']])

    def test_helper_builds_once_before_suites(self):
        groups = self.plan('skein/leaf/src/lib.rs')
        helper = [{'package': 'fixture', 'bin': 'serve'}]
        commands = list(PLAN.rust_commands(groups, {'leaf': helper, 'middle': helper}))
        self.assertEqual(commands[0], ['cargo', 'build', '--offline', '--locked', '-p', 'fixture', '--bin', 'serve'])
        self.assertEqual(len(commands), 2)


class Delivery(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.env = {}
        self.errors = []
        self.outcomes = {}
        self.log = io.StringIO()
        self.enterContext(redirect_stdout(self.log))

    def test_modes_share_the_tree_only_checker(self):
        with mock.patch.object(PRE, 'check', return_value=0) as check:
            for args, mode in [([], 'delivery'), (['--child'], 'child'), (['--goal'], 'goal'),
                               (['--child', 'dest'], 'child'), (['--close'], 'close'),
                               (['--close', 'dest'], 'close'), (['--goal', 'head', 'dest'], 'goal')]:
                self.assertEqual(PRE.main(args), 0)
                self.assertEqual(check.call_args.args[0], mode)
        with self.assertRaises(ValueError):
            PRE.main(['--goal', 'head'])

    def test_checker_changes_run_only_python_and_write_local_receipt(self):
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=[]):
            selected = PRE.workspace_checks(self.root, ['tools/pre-pr/pre_pr.py'], 'tree',
                                            self.env, self.errors, self.outcomes)
        self.assertEqual([call.args[0][3] for call in run.call_args_list],
                         ['tools/pre-pr/test_pre_pr.py'])
        outward.assert_called_once_with(self.root, self.env, ['tools/pre-pr/pre_pr.py'])
        self.assertTrue(all(call.args[1] == self.root for call in run.call_args_list))
        self.assertEqual(selected, [])
        self.assertEqual(self.errors, [])
        name = PRE.failure_file('tree', 'base', False, False, [], self.root, {}, 'delivery')
        receipt = json.loads((self.root / name.removeprefix('every key in ')).read_text())
        self.assertEqual((receipt['scope'], receipt['tree'], receipt['verdict']), ('light', 'tree', 'passed'))

    def test_rustfmt_reports_a_relative_location_and_a_repair(self):
        def run(args, root, env, output):
            self.assertEqual(args, ['cargo', 'fmt', '--all', '--', '--check'])
            with redirect_stdout(io.TextIOWrapper(io.BytesIO())):
                log = PRE.Tail()
                output(f'Diff in {root}/src/lib.rs:17:\n'.encode(), log)
                output(b'+ ' + b'x' * 4096 + b'\n', log)
            return 1, log.text()
        with mock.patch.object(PRE, 'run', side_effect=run):
            errors = PRE.format_workspace(self.root, {})
        self.assertEqual(errors[0]['log'], 'src/lib.rs:17: run cargo fmt --all\n')

    def test_failed_checker_refuses_and_publishes_reason(self):
        with mock.patch.object(PRE, 'run', side_effect=[(1, 'fixture failed'), (0, '')]), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=[]):
            PRE.workspace_checks(self.root, ['.temper/pre-pr'], 'tree', {}, self.errors, {})
        self.assertEqual(PRE.finish('tree', None, self.errors, self.root, {}, 'delivery', (), None), 1)
        self.assertIn('fixture failed', self.log.getvalue())

    def test_current_budget_tables_and_empty_overlay_are_checked(self):
        from shutil import copyfile
        tiers = self.root / 'tools/tiers'
        tiers.mkdir(parents=True)
        for name in ('cpu.py', 'layers.toml', 'budgets.toml'):
            copyfile(HERE.parent / 'tiers' / name, tiers / name)
        self.assertEqual(PRE.budget_table(self.root), [])
        (tiers / 'budgets.toml').write_text('[gate.unknown]\n')
        error, = PRE.budget_table(self.root)
        self.assertEqual(error['name'], 'budgets')
        self.assertIn('tools/tiers/budgets.toml', error['log'])
        self.assertEqual(error['log'].splitlines()[-1], module(tiers / 'cpu.py').REPAIR_LINE)
        overlay = self.root / PRE.JOURNEY_ROOTS[0] / 'overlay.css'
        overlay.parent.mkdir(parents=True)
        overlay.touch()
        self.assertEqual(PRE.journeys(self.root), [])
        overlay.write_text('nonempty')
        self.assertIn('overlay.css must be empty', PRE.journeys(self.root)[0]['log'])

    def test_snapshot_changed_during_checks_refuses(self):
        for final, expected in [('first', 0), ('other', 1)]:
            with self.subTest(final=final), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.object(PRE, 'checked_git', return_value=b''), \
                 mock.patch.object(PRE, 'snapshot', side_effect=['first', final]), \
                 mock.patch.object(PRE, 'workspace_checks', return_value=[]):
                self.assertEqual(PRE.main([]), expected)

    def test_shell_entry_points_forward_to_the_expected_mode(self):
        for hook, suffix in [('pre-pr', ''), ('goal-child-pre-pr', ' --child "$@"'), ('goal-pre-pr', ' --goal "$@"')]:
            script = HERE.parents[1] / '.temper' / hook
            commands = [line for line in script.read_text().splitlines() if line.startswith('exec ')]
            self.assertEqual(commands, ['exec python3 -I -B tools/pre-pr/pre_pr.py' + suffix])
            self.assertTrue(script.stat().st_mode & 0o111)

    def test_changed_python_module_runs_its_sibling_suite_once(self):
        directory = self.root / 'tools/example'
        directory.mkdir(parents=True)
        (directory / 'test_rules.py').touch()
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'journeys', return_value=[]), mock.patch.object(PRE, 'outward', return_value=(0, [])):
            args = (self.root, ['tools/example/rules.py', 'tools/example/test_rules.py'],
                    'tree', {}, self.errors, {})
            PRE.workspace_checks(*args)
            run.assert_not_called()
            PRE.workspace_checks(*args, completed_goal=True)
        self.assertEqual([call.args[0] for call in run.call_args_list],
                         [['python3', '-I', '-B', 'tools/example/test_rules.py']])

    def test_scoped_outward_scan_sends_existing_names_literally(self):
        path = 'new file with a newline\n.py'
        (self.root / path).touch()
        answer = subprocess.CompletedProcess([], 0, b'')
        with mock.patch.object(subprocess, 'run', return_value=answer) as run:
            self.assertEqual(PRE.outward(self.root, {}, [path, 'deleted']), (0, []))
        self.assertEqual(run.call_args.args[0][-1], '--paths0')
        self.assertEqual(run.call_args.kwargs['input'], path.encode())

    def test_close_receipt_records_only_the_checked_tree(self):
        self.assertEqual(PRE.finish('staged', None, [], self.root, {}, 'close', ['dest'], ['head']), 0)
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual((receipt['goal_head'], receipt['requested_destination'], receipt['tree']),
                         ('head', 'dest', 'staged'))
        self.assertEqual((receipt['comparison'], receipt['base'], receipt['destination']), ('tree-only', None, None))

    def test_known_outward_failure_requires_a_complete_scan(self):
        key = ('test', 'fabrig-boundary::outward', 'no_file_names_the_lab')
        with mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=[]):
            PRE.workspace_checks(self.root, ['other'], 'tree', {}, self.errors, self.outcomes, [key])
        outward.assert_called_once_with(self.root, {}, None)
        self.assertEqual(self.outcomes[key[1:]], {'outcome': 'passed'})

    def test_close_compiler_error_survives_a_long_log_and_hold_truncation(self):
        location = 'smith/tests/local/src/world.rs:1423:15'
        message = 'non-exhaustive patterns: `smith_domain::Lower::Classify { .. }` not covered'
        raw = (f'error[E0004]: {message}\n    --> {location}\n'
               'error[E0308]: mismatched types\n    --> other.rs:8:2\n' + 'cargo argument ' * 1200 + '\n').encode()
        diagnostic = PRE.RustcDiagnostic()
        process = mock.MagicMock(stdout=io.BytesIO(raw))
        process.wait.return_value = 101
        stream = io.BytesIO()
        with mock.patch.object(subprocess, 'Popen', return_value=process), redirect_stdout(io.TextIOWrapper(stream)) as out:
            status, log = PRE.run(['cargo', 'nextest', 'run'], self.root, {}, output=diagnostic.consume)
            self.assertEqual(out.buffer.getvalue(), raw)
        self.assertEqual(status, 101)
        self.assertNotIn('E0004', log)
        error = dict(PRE.failure('focused tests', log), diagnostic=diagnostic.reason())
        self.assertEqual(PRE.finish('tree', 'base', [error], self.root, {}, 'close', ['dest'], ['head'],
                                    tests=['package'] * 2000), 1)
        self.assertEqual(self.log.getvalue().encode()[-128:], f'rustc E0004 {location}: {message}\n'.encode())
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual(receipt['verdict'], 'refused')
        self.assertEqual(receipt['counts'], {'new': 1, 'flake': 0, 'inherited': 0})

    def test_close_nextest_assertion_survives_the_generic_summary(self):
        case = 'registered_value_type_numbers_are_pinned'
        site = 'jig/crates/jig-protocol-accounts/src/tests/value_types.rs:98:9'
        output = (f"  FAIL [  0.011s] ( 3304/13294) jig-protocol-accounts tests::value_types::{case}\n"
                  f"    thread 'tests::value_types::{case}' (16248) panicked at {site}:\n"
                  "    assertion `left == right` failed: jig-store-core\n      left: 9\n     right: 8\n"
                  + "other output\n" * 500 + "error: test run failed\n").encode()
        diagnostic = PRE.NextestDiagnostic()
        with redirect_stdout(io.TextIOWrapper(io.BytesIO())):
            log = PRE.Tail()
            for line in output.splitlines(keepends=True):
                diagnostic.consume(line, log)
        self.assertNotIn(case, log.text())
        error = dict(PRE.failure('focused tests', log.text()), diagnostic=diagnostic.reason())
        self.assertEqual(PRE.finish('tree', 'base', [error], self.root, {}, 'close', ['main'], ['head']), 1)
        self.assertEqual(self.log.getvalue().encode()[-128:], f'{case} {site}: jig-store-core: 9 != 8\n'.encode())

    def test_failed_case_without_a_panic_keeps_its_name(self):
        diagnostic = PRE.NextestDiagnostic()
        log = PRE.Tail()
        with redirect_stdout(io.TextIOWrapper(io.BytesIO())):
            for line in (b"   FAIL [ 0.01s] (1/2) crate tests::first", b"   FAIL [ 0.02s] (2/2) crate tests::second",
                         b"thread 'tests::second' panicked at second.rs:8:1:", b"other panic"):
                diagnostic.consume(line, log)
        self.assertEqual(diagnostic.reason(), 'failed test tests::first\n')

    def test_large_receipt_preserves_every_key_and_bounds_summary(self):
        errors = [PRE.failure(f'test api::suite case_{i}', 'x' * 9000, ('api::suite', f'case_{i}'))
                  for i in range(450)]
        PRE.inherited({('test', 'api::suite', 'case_0')}, errors)
        errors[1]['flake'] = (1, 0.1)
        self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'goal', ['head', 'dest'], ['head']), 1)
        receipt = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(receipt['counts'], {'new': 448, 'flake': 1, 'inherited': 1})
        self.assertEqual({entry['key']['test'] for entry in receipt['failures']}, {f'case_{i}' for i in range(450)})
        self.assertLessEqual(len(self.log.getvalue().encode()), 8192)

    def test_diagnostic_cpu_measurements_do_not_enable_budget_refusals(self):
        for profile in ('default', 'fuzzy'):
            with mock.patch.object(PRE, 'run', return_value=(2, 'one-process: nextest listing exited with 101')) as run:
                result, errors, _ = PRE.one_process(['api'], self.root, {}, profile=profile)
            self.assertEqual(result, 2)
            self.assertTrue(errors[0]['name'].startswith('build'))
            cmd = run.call_args.args[0]
            self.assertNotIn('--ledger', cmd)
            self.assertNotIn('--gate', cmd)

    def test_diagnostic_preserves_the_failing_case(self):
        def failing(command, cwd, env, stdout=False):
            directory = Path(command[command.index('--out') + 1])
            directory.mkdir()
            (directory / 'run.json').write_text('{"starved": false}')
            (directory / 'results.json').write_text(json.dumps({'tests': [
                {'binary_id': 'api::suite', 'test_name': 'example', 'outcome': 'failed'}], 'unobserved': []}))
            (directory / 'junit.xml').write_text('<testsuites><testsuite name="api::suite"><testcase name="example">'
                                                 '<failure>broken</failure></testcase></testsuite></testsuites>')
            return 100, 'failed'
        with mock.patch.object(PRE, 'run', side_effect=failing):
            _, errors, _ = PRE.one_process(['api'], self.root, {})
        self.assertEqual(errors[0]['key'], ('api::suite', 'example'))

    def test_goal_runner_uses_a_private_main_ledger_copy_and_refuses_coverage_loss(self):
        ledgers = self.root / 'ledgers'
        ledgers.mkdir()
        source = ledgers / 'fuzzy.json'
        source.write_text(json.dumps(dict(ledger=6, profile='fuzzy', binaries={}, budgets={}, moves=[])))
        cpu = HERE.parents[1] / 'tools/tiers/cpu.py'
        destination = self.root / 'tools/tiers/cpu.py'
        destination.parent.mkdir(parents=True)
        destination.write_bytes(cpu.read_bytes())

        def coverage_red(command, cwd, env, stdout=False):
            copy = Path(command[command.index('--ledger') + 1])
            self.assertNotEqual(copy, source)
            self.assertEqual(copy.read_bytes(), source.read_bytes())
            self.assertNotIn('--gate', command)
            return 1, 'layer coverage loss: temper fuzzy domain: api::suite'

        with mock.patch.object(PRE, 'run', side_effect=coverage_red):
            code, errors, _ = PRE.one_process(['api'], self.root, {}, profile='fuzzy', ledger_directory=ledgers)
        self.assertEqual(code, 1)
        self.assertIn('layer coverage loss:', errors[0]['log'])
        self.assertIn('CPU and reference warnings with coverage checked', self.log.getvalue())

    def test_child_destination_is_only_a_request_label(self):
        PRE.finish('tree', None, [], self.root, {}, 'child', ['dest'], ['head'])
        record = json.loads((self.root / 'target/temper/goal-child-pre-pr.json').read_text())
        self.assertEqual((record['scope'], record['delivery_head'], record['requested_destination'],
                          record['destination']), ('child', 'head', 'dest', None))


class Gate(unittest.TestCase):
    def test_complete_candidate_decides_scope(self):
        self.assertEqual(GATE.route(set()), 'full')
        self.assertEqual(GATE.route({'.temper/pre-pr', 'tools/pre-pr/workspace.py'}), 'checker')
        self.assertEqual(GATE.route({'.temper/pre-pr', 'src/lib.rs'}), 'mixed')
        self.assertEqual(GATE.route({'docs/guide.md'}), 'classify')

    def test_preflight_runs_tests_and_writes_real_outcome(self):
        for paths, status, expected in [(b'tools/pre-pr/pre_pr.py\0', 0, 0),
                                        (b'tools/pre-pr/pre_pr.py\0', 1, 2),
                                        (b'.temper/pre-pr\0crate/src/lib.rs\0', 0, 1)]:
            with self.subTest(paths=paths, status=status), tempfile.TemporaryDirectory() as tmp:
                execute = mock.Mock(return_value=subprocess.CompletedProcess([], status))
                root = Path(tmp)
                with mock.patch.object(subprocess, 'check_output', return_value=paths), redirect_stdout(io.StringIO()):
                    self.assertEqual(GATE.checks(root, 'base', 'head', root, execute), expected)
                execute.assert_called_once_with(['python3', '-I', '-B', 'tools/pre-pr/test_pre_pr.py'], cwd=root, timeout=600)
                if expected != 1:
                    self.assertEqual((root / 'failed-tests.txt').read_text(), 'gate::checker\n' if status else '')


if __name__ == '__main__':
    unittest.main()
