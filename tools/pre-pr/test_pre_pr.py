"""Delivery policy checks with recording fakes and a scratch counts repository."""
from contextlib import redirect_stderr, redirect_stdout
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
POPEN = subprocess.Popen


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
        directory = self.root / 'target/close'
        self.assertEqual(list(PLAN.rust_commands(groups, {}, directory)),
                         [['tools/tiers/one-process', '--out', str(directory / '0'), '--fail-fast', 'default',
                           '--offline', '--locked', '--no-tests=warn', '-p', 'leaf', '--test', 'suite']])

    def test_boundary_filter_keeps_candidate_checks_and_the_profile(self):
        flag, expression = PLAN.workspace_filter(['fabrig-boundary', 'leaf'])
        self.assertEqual(flag, '-E')
        self.assertIn('default() & not (package(=fabrig-boundary)', expression)
        self.assertIn('test(=client_layout_changes_move_their_versions)', expression)
        self.assertNotIn('current_client_wire_layouts_match_the_schema', expression)
        self.assertNotIn('generated_patch_modules_are_current', expression)
        self.assertEqual(PLAN.workspace_filter(['leaf']), [])

    def test_nested_test_helper_stays_with_its_owner(self):
        self.assertEqual(list(self.plan('skein/leaf/tests/shared/input.rs').values()), [['leaf']])

    def test_unknown_root_file_and_later_tier_do_not_build(self):
        self.assertEqual(self.plan('tools/pre-pr/pre_pr.py', 'skein/top/tests/live.rs'), {})

    def test_changed_dependency_also_widens_a_changed_test_owner(self):
        self.assertEqual(self.plan('skein/leaf/src/lib.rs', 'skein/middle/tests/suite.rs'),
                         {(('--lib',), ('--test', 'suite')): ['leaf', 'middle']})

    def test_changed_product_crate_without_test_targets_still_compiles(self):
        self.packages['top']['targets'] = []
        self.assertEqual(list(PLAN.rust_commands(self.plan('skein/top/src/lib.rs'), {}, None)),
                         [['cargo', 'check', '--offline', '--locked', '-p', 'top']])

    def test_always_run_packages_add_no_dependents(self):
        groups = PLAN.package_plan(self.root, [], self.packages, {}, also=['leaf'])
        self.assertEqual(sorted(name for names in groups.values() for name in names), ['leaf'])

    def test_close_inputs_reach_the_workspace_the_gate_lists_and_nothing_for_prose(self):
        root = self.root
        self.enterContext(mock.patch.object(PRE, 'lines', return_value=['leaf', 'middle skein/middle/', 'ghost']))
        self.assertEqual(PRE.always_packages(root, ['skein/top/src/lib.rs'], self.packages), ['leaf'])
        self.assertEqual(PRE.always_packages(root, ['skein/middle/src/lib.rs'], self.packages), ['leaf', 'middle'])
        self.assertEqual(PRE.always_packages(root, ['tools/citations/maps/a.map'], self.packages), [])
        for path in ('Cargo.lock', 'Cargo.toml', 'rust-toolchain.toml', '.cargo/config.toml', '.config/nextest.toml'):
            self.assertTrue(PRE.workspace_wide(path), path)
        self.assertFalse(PRE.workspace_wide('skein/mid/Cargo.toml'))

    def test_helper_builds_once_before_suites(self):
        groups = self.plan('skein/leaf/src/lib.rs')
        helper = [{'package': 'fixture', 'bin': 'serve'}]
        commands = list(PLAN.rust_commands(groups, {'leaf': helper, 'middle': helper}, self.root / 'target/close'))
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
                         ['tools/pre-pr/test_pre_pr.py', 'tools/citations/citations.py'])
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
                 mock.patch.object(PRE, 'scope_base', return_value=('b' * 40, 'refs/temper/landing', None)), \
                 mock.patch.object(PRE, 'changed_paths', return_value=[]), \
                 mock.patch.object(PRE, 'workspace_checks', return_value=[]):
                self.assertEqual(PRE.main([]), expected)

    def test_no_base_refuses_with_its_reason_and_checks_nothing(self):
        with mock.patch.object(Path, 'cwd', return_value=self.root), \
             mock.patch.object(PRE, 'checked_git', return_value=b''), \
             mock.patch.object(PRE, 'snapshot', side_effect=['first', 'first']), \
             mock.patch.object(PRE, 'workspace_checks', return_value=[]) as checks:
            self.assertEqual(PRE.main([]), 1)
        checks.assert_not_called()
        self.assertIn('no base: this is no git checkout', self.log.getvalue())

    def test_shell_entry_points_forward_to_the_expected_mode(self):
        for hook, suffix in [('pre-pr', ''), ('goal-child-pre-pr', ' --child "$@"'), ('goal-pre-pr', ' --goal "$@"'),
                             ('checked-close', ' --close')]:
            script = HERE.parents[1] / '.temper' / hook
            commands = [line for line in script.read_text().splitlines() if line.startswith('exec ')]
            self.assertEqual(commands, ['exec python3 -I -B tools/pre-pr/pre_pr.py' + suffix])
            self.assertTrue(script.stat().st_mode & 0o111)

    def test_changed_python_module_runs_its_sibling_suite_once(self):
        directory = self.root / 'tools/example'
        directory.mkdir(parents=True)
        (directory / 'test_rules.py').touch()
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
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

    def test_changed_rust_files_format_their_packages_and_wide_inputs_the_workspace(self):
        data = {'packages': [{'id': 'leaf', 'name': 'leaf', 'edition': '2021',
                              'manifest_path': str(self.root / 'leaf/Cargo.toml')},
                             {'id': 'twig', 'name': 'twig', 'edition': '2024',
                              'manifest_path': str(self.root / 'twig/Cargo.toml')}],
                'workspace_members': ['leaf', 'twig']}
        cases = [(['leaf/src/lib.rs', 'twig/Cargo.toml', 'fixtures/orphan.rs', 'docs/a.md'],
                  [['cargo', 'fmt', '-p', 'leaf', '-p', 'twig', '--', '--check']]),
                 (['fixtures/orphan.rs'], []),
                 (['docs/a.md'], []),
                 (['leaf/rustfmt.toml'], [['cargo', 'fmt', '--all', '--', '--check']]),
                 (['Cargo.toml'], [['cargo', 'fmt', '--all', '--', '--check']])]
        for paths, expected in cases:
            with self.subTest(paths=paths), mock.patch.object(PRE, 'metadata', return_value=data), \
                 mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
                 mock.patch.object(PRE, 'citations', return_value=(0, [])), \
                 mock.patch.object(PRE, 'outward', return_value=(0, [])), mock.patch.object(PRE, 'journeys', return_value=[]), \
                 mock.patch.object(PRE, 'baselines', return_value=(0, [])):
                PRE.workspace_checks(self.root, paths, 'tree', {}, self.errors, {}, whole=False)
                self.assertEqual([call.args[0] for call in run.call_args_list if call.args[0][0] == 'cargo'], expected)
        self.assertEqual(self.errors, [])

    def test_a_goal_changing_no_tooling_runs_no_python_suite(self):
        (self.root / 'tools/example').mkdir(parents=True)
        (self.root / 'tools/example/test_rules.py').touch()
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])), mock.patch.object(PRE, 'journeys', return_value=[]):
            self.assertEqual(PRE.workspace_checks(self.root, ['docs/guide.md', 'temper/docs/design/a.md'], 'tree',
                                                  {}, self.errors, {}, completed_goal=True, whole=False), [])
        run.assert_not_called()

    def test_a_path_without_cargo_gains_the_users_cargo_tools(self):
        home = self.root / 'home'
        (home / '.cargo/bin').mkdir(parents=True)
        (home / '.cargo/bin/cargo').touch()
        user = mock.Mock(pw_dir=str(home))
        with mock.patch.object(PRE.pwd, 'getpwuid', return_value=user), \
             mock.patch.object(PRE.shutil, 'which', return_value=None):
            env = {'PATH': '/usr/bin:/bin', 'HOME': '/nonexistent'}
            PRE.with_rust_tools(env)
        self.assertEqual(env, {'PATH': f'{home}/.cargo/bin:/usr/bin:/bin', 'HOME': '/nonexistent',
                               'CARGO_HOME': f'{home}/.cargo', 'RUSTUP_HOME': f'{home}/.rustup'})
        with mock.patch.object(PRE.shutil, 'which', return_value='/usr/bin/cargo'):
            env = {'PATH': '/usr/bin'}
            PRE.with_rust_tools(env)
        self.assertEqual(env, {'PATH': '/usr/bin'})

    def test_a_refused_close_names_its_first_failure_on_stderr(self):
        errors = [PRE.failure('budgets', 'temper default simulated: instruction growth over allowance, ' + 'x' * 200)]
        with redirect_stderr(io.StringIO()) as err:
            self.assertEqual(PRE.finish('tree', 'b' * 40, errors, self.root, {}, 'close', [], ['head']), 1)
        self.assertEqual(err.getvalue(),
                         ('budgets: temper default simulated: instruction growth over allowance, ' + 'x' * 200)[:127] + '\n')
        with redirect_stderr(io.StringIO()) as err:
            PRE.finish('tree', 'b' * 40, errors, self.root, {}, 'delivery', [], ['head'])
        self.assertEqual(err.getvalue(), '')

    def test_close_receipt_records_only_the_checked_tree(self):
        self.assertEqual(PRE.finish('staged', None, [], self.root, {}, 'close', ['dest'], ['head']), 0)
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual((receipt['goal_head'], receipt['requested_destination'], receipt['tree']),
                         ('head', 'dest', 'staged'))
        self.assertEqual((receipt['comparison'], receipt['base'], receipt['destination']), ('tree-only', None, None))

    def test_known_outward_failure_requires_a_complete_scan(self):
        key = ('test', 'fabrig-boundary::outward', 'no_file_names_the_lab')
        with mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
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

    def test_close_one_process_assertion_survives_the_generic_summary(self):
        case = 'registered_value_type_numbers_are_pinned'
        site = 'jig/crates/jig-protocol-accounts/src/tests/value_types.rs:98:9'
        output = (f"thread 'tests::value_types::{case}' (16248) panicked at {site}:\n"
                  "assertion `left == right` failed: jig-store-core\n  left: 9\n right: 8\n")
        for outcome in ('failed', 'crashed'):
            with self.subTest(outcome=outcome):
                (self.root / 'results.json').write_text(json.dumps({'tests': [
                    dict(test_name='tests::earlier', outcome='passed', output=None),
                    dict(test_name=f'tests::value_types::{case}', outcome=outcome, output=output),
                    dict(test_name='tests::later', outcome='failed', output=None),
                ]}))
                diagnostic = PRE.OneProcessDiagnostic(self.root)
                log = PRE.tail('other output\n' * 500 + 'error: test run failed\n')
                self.assertNotIn(case, log)
                error = dict(PRE.failure('focused tests', log), diagnostic=diagnostic.reason())
                self.assertEqual(PRE.finish('tree', 'base', [error], self.root, {}, 'close', ['main'], ['head']), 1)
                expected = f'{case} {site}: jig-store-core: 9 != 8\n'.encode()
                self.assertEqual(self.log.getvalue().encode()[-128:], expected)

    def test_failed_case_without_a_panic_keeps_its_name(self):
        (self.root / 'results.json').write_text(json.dumps({'tests': [
            dict(test_name='tests::first', outcome='crashed', output='aborted'),
            dict(test_name='tests::second', outcome='failed',
                 output="thread 'tests::second' panicked at second.rs:8:1:\nother panic"),
        ]}))
        diagnostic = PRE.OneProcessDiagnostic(self.root)
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

    def test_child_destination_is_only_a_request_label(self):
        PRE.finish('tree', None, [], self.root, {}, 'child', ['dest'], ['head'])
        record = json.loads((self.root / 'target/temper/goal-child-pre-pr.json').read_text())
        self.assertEqual((record['scope'], record['delivery_head'], record['requested_destination'],
                          record['destination']), ('child', 'head', 'dest', None))


class Changes(unittest.TestCase):
    """The checks' base and paths come from the checkout's own refs, and its snapshot from its own index."""

    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory())) / 'checkout'
        self.root.mkdir()
        self.env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull,
                    'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.test',
                    'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.test'}
        self.enterContext(mock.patch.object(subprocess, 'Popen', side_effect=self.git_only))
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_SOURCE', None))
        self.enterContext(mock.patch.dict(PRE.PUBLISHED, clear=True))
        self.git('init', '-q')
        for name in ('kept.rs', 'edited.rs', 'unstaged.md', 'gone.md'):
            (self.root / name).write_text(name + '\n')
        self.git('add', '-A')
        self.git('commit', '-qm', 'main')
        self.main = self.git('rev-parse', 'HEAD')

    def git_only(self, args, **kwargs):
        self.assertEqual(args[0], 'git')
        return POPEN(args, **kwargs)

    def git(self, *args):
        return subprocess.run(('git', *args), cwd=self.root, env=self.env, capture_output=True, text=True,
                              check=True).stdout.strip()

    def change(self):
        (self.root / 'edited.rs').write_text('edited\n')
        self.git('commit', '-qam', 'committed')
        (self.root / 'staged.rs').write_text('staged\n')
        self.git('add', 'staged.rs')
        (self.root / 'unstaged.md').write_text('unstaged\n')
        (self.root / 'gone.md').unlink()
        (self.root / 'new.md').write_text('new\n')
        (self.root / 'ignored.log').write_text('ignored\n')
        (self.root / '.git/info/exclude').write_text('*.log\n')

    def test_delivery_checks_only_what_changed_since_the_pin(self):
        self.git('update-ref', 'refs/temper/landing', self.main)
        self.change()
        self.assertEqual(PRE.change_base(self.root, self.env, 'delivery'),
                         (self.main, 'merge base of refs/temper/landing and HEAD'))
        self.assertEqual(PRE.changed_paths(self.root, self.env, self.main),
                         ['edited.rs', 'gone.md', 'new.md', 'staged.rs', 'unstaged.md'])

    def test_close_compares_with_main_not_the_pin(self):
        self.git('update-ref', 'refs/remotes/origin/main', self.main)
        self.change()
        self.git('update-ref', 'refs/temper/landing', 'HEAD')
        self.assertEqual(PRE.change_base(self.root, self.env, 'close'),
                         (self.main, 'merge base of refs/remotes/origin/main and HEAD'))
        self.assertEqual(PRE.change_base(self.root, self.env, 'delivery'),
                         (self.git('rev-parse', 'HEAD'), 'merge base of refs/temper/landing and HEAD'))

    def publish(self, commits):
        data = json.dumps(dict(version=1, commits=commits, profiles={}))
        blob = subprocess.run(['git', 'hash-object', '-w', '--stdin'], cwd=self.root, env=self.env, input=data,
                              capture_output=True, text=True, check=True).stdout.strip()
        tree = subprocess.run(['git', 'mktree'], cwd=self.root, env=self.env, input=f'100644 blob {blob}\tcounts.json\n',
                              capture_output=True, text=True, check=True).stdout.strip()
        self.git('update-ref', 'refs/temper/counts', self.git('commit-tree', tree, '-m', 'counts'))

    def test_close_starts_from_mains_newest_published_commit_in_the_goal(self):
        later = self.git('commit-tree', 'HEAD^{tree}', '-p', self.main, '-m', 'main moved on')
        self.change()
        self.git('update-ref', 'refs/remotes/origin/main', later)
        self.publish(['0' * 40, self.main, later])
        self.assertEqual(PRE.change_base(self.root, self.env, 'close'),
                         (self.main, "main's newest published commit in HEAD"))

    def test_unchecked_saves_and_a_pending_merge_stay_checked(self):
        (self.root / 'edited.rs').write_text('saved\n')
        self.git('commit', '-qam', PRE.UNCHECKED_SAVE)
        self.git('update-ref', 'refs/temper/landing', 'HEAD')
        self.assertEqual(PRE.change_base(self.root, self.env, 'delivery')[0], self.main)
        self.git('checkout', '-q', '-b', 'side', self.main)
        (self.root / 'side.rs').write_text('side\n')
        self.git('add', 'side.rs')
        self.git('commit', '-qm', 'side')
        side = self.git('rev-parse', 'HEAD')
        self.git('checkout', '-q', '-')
        self.git('update-ref', 'refs/temper/landing', side)
        self.assertEqual(PRE.change_base(self.root, self.env, 'delivery')[0], self.main)
        # Merging the moved landing branch in: its own changes aren't the delivery's.
        (self.root / '.git/MERGE_HEAD').write_text(side + '\n')
        self.assertEqual(PRE.change_base(self.root, self.env, 'delivery')[0], side)

    def test_without_a_usable_base_the_check_refuses_loudly(self):
        for scope in ('delivery', 'child', 'close'):
            base, name, why = PRE.scope_base(self.root, self.env, scope, True)
            self.assertEqual((base, name), (None, None))
            self.assertIn('update from main' if scope == 'close' else 'sharing history with HEAD', why)
            self.assertIn('no base', PRE.scope_base(self.root, self.env, scope, False)[2])
        self.git('update-ref', 'refs/remotes/origin/main', 'HEAD')
        self.assertEqual(PRE.scope_base(self.root, self.env, 'delivery', True)[:2],
                         (self.main, 'merge base of refs/remotes/origin/main and HEAD'))
        base, name, why = PRE.scope_base(self.root, self.env, 'close', True)
        self.assertIsNone(base)
        self.assertIn('is HEAD itself', why)

    def test_skipped_entries_are_not_seeded(self):
        work = self.root.parent / 'work'
        subprocess.run(['git', 'init', '-q', str(work)], env=self.env, check=True, capture_output=True)
        self.git('update-index', '--skip-worktree', 'kept.rs')
        self.assertFalse(PRE.seed(self.root, work, self.env, work / '.git/index'))
        self.git('update-index', '--no-skip-worktree', 'kept.rs')
        self.git('update-index', '--assume-unchanged', 'kept.rs')
        self.assertFalse(PRE.seed(self.root, work, self.env, work / '.git/index'))
        self.git('update-index', '--no-assume-unchanged', 'kept.rs')
        self.assertTrue(PRE.seed(self.root, work, self.env, work / '.git/index'))

    def test_without_a_base_reference_there_is_no_base(self):
        for scope in ('delivery', 'child', 'close', 'goal'):
            self.assertEqual(PRE.change_base(self.root, self.env, scope), (None, None))

    def test_the_seeded_snapshot_is_the_fresh_one(self):
        self.change()
        trees = []
        for seeded in (True, False):
            work = self.root.parent / f'work-{seeded}'
            subprocess.run(['git', 'init', '-q', str(work)], env=self.env, check=True, capture_output=True)
            with mock.patch.object(PRE, 'seed', wraps=PRE.seed) if seeded else \
                    mock.patch.object(PRE, 'seed', return_value=False) as seed:
                trees.append(PRE.snapshot(self.root, work, self.env, self.env, work / '.git/index'))
                self.assertEqual(seed.call_count, 1)
        self.assertEqual(trees[0], trees[1])
        self.assertTrue((self.root.parent / 'work-True/.git/objects/info/alternates').is_file())


class CloseBudgets(unittest.TestCase):
    G = 1_000_000_000
    BINARY = 'leaf::suite'

    def setUp(self):
        scratch = self.enterContext(tempfile.TemporaryDirectory())
        self.repository = Path(scratch) / 'source'
        self.repository.mkdir()
        self.root = Path(scratch) / 'snapshot'
        self.tiers = self.root / 'tools/tiers'
        self.tiers.mkdir(parents=True)
        (self.tiers / 'packages.toml').write_text('')
        (self.tiers / 'layers.toml').write_text('[skein]\n"leaf::suite" = "step"\n')
        self.env = {**os.environ, 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': os.devnull,
                    'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.test',
                    'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.test'}
        self.enterContext(mock.patch.object(subprocess, 'Popen', side_effect=self.git_only))
        self.log = self.enterContext(redirect_stdout(io.StringIO()))
        self.git('init', '-q')
        self.git('commit', '-qm', 'base', '--allow-empty')
        self.base = self.git('rev-parse', 'HEAD')
        self.main = self.git('commit-tree', 'HEAD^{tree}', '-p', self.base, '-m', 'new main')
        self.git('commit', '-qm', 'goal', '--allow-empty')
        self.profile = {'moves': [], 'binaries': {
            self.BINARY: self.binary(25 * self.G, 8),
            'leaf::other': self.binary(10 * self.G, 2),
            'held::suite': self.binary(65 * self.G, 10),
            'future::suite': self.binary(110 * self.G, 80, commit=self.main),
        }}
        self.profile['binaries'][self.BINARY]['counts'][self.main] = 50 * self.G
        self.enterContext(mock.patch.object(PRE, 'WORKER_COUNTS', self.root / 'no-worker-counts.json'))
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_SOURCE', None))
        self.enterContext(mock.patch.dict(PRE.PUBLISHED, clear=True))
        self.budget_table()

    def git_only(self, args, **kwargs):
        self.assertEqual(args[0], 'git')
        self.assertEqual(kwargs['cwd'], self.repository)
        return POPEN(args, **kwargs)

    def git(self, *args, content=None):
        return subprocess.run(('git', *args), cwd=self.repository, env=self.env, input=content,
                              capture_output=True, text=True, check=True).stdout.strip()

    def binary(self, instructions, cpu, *, commit=None):
        return dict(tree='skein', layer='step', cpu_s=cpu, counts={commit or self.base: instructions})

    def publish(self, commits=None):
        data = dict(version=1, commits=commits or [self.base, self.main], profiles={'default': self.profile})
        blob = self.git('hash-object', '-w', '--stdin', content=json.dumps(data))
        tree = self.git('mktree', content=f'100644 blob {blob}\tcounts.json\n')
        commit = self.git('commit-tree', tree, '-m', 'published counts')
        self.git('update-ref', 'refs/temper/counts', commit)

    def budget_table(self, ceiling=200 * G, moves='', accepted=''):
        (self.tiers / 'budgets.toml').write_text(
            f'[gate.default.skein.step]\ninstructions = {ceiling}\n{accepted}'
            f'[gate.default.skein.simulated]\ninstructions = {100 * self.G}\n' + moves)

    def process(self, instructions, *, binary=BINARY, kind='tests', tree='skein', layer='step'):
        return dict(binary_id=binary, kind=kind, tree=tree, layer=layer, instructions=instructions)

    def result(self, *processes, counting=True, why=None, tests=(), unobserved=(), directory=None):
        directory = directory or Path(self.enterContext(tempfile.TemporaryDirectory(dir=self.root)))
        directory.mkdir(exist_ok=True)
        (directory / 'results.json').write_text(json.dumps(dict(
            selected=len(processes) + len(tests), counting=dict(counted=counting, why=why) if counting is not None else None,
            processes=processes, tests=tests, unobserved=unobserved)))
        return directory

    def verdict(self, *runs):
        return PRE.close_budgets(self.root, self.repository, self.env, runs)

    def test_no_counts_warns_once_and_passes(self):
        self.assertEqual(self.verdict(self.result(self.process(50 * self.G))), [])
        self.assertEqual(self.log.getvalue(),
                         "budgets: no main's counts in this repository (refs/temper/counts); "
                         'no verdict at close, an infrastructure fault to repair; the gate still judges\n')

    def test_counts_missing_here_are_fetched_from_main(self):
        self.publish()
        public = self.root.parent / 'public.git'
        self.git('init', '-q', '--bare', str(public))
        self.git('push', '-q', str(public), 'refs/temper/counts:refs/temper/counts')
        self.git('update-ref', '-d', 'refs/temper/counts')
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_SOURCE', 'file://' + str(public)))
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_PROTOCOLS', 'file'))
        self.enterContext(mock.patch.object(subprocess, 'Popen', side_effect=POPEN))
        error, = self.verdict(self.result(self.process(30 * self.G)))
        self.assertIn('instruction growth over allowance', error['log'])
        self.assertIsNone(module(HERE.parent / 'tiers/counts.py').read(self.repository, self.env))

    def test_selected_baseline_can_be_used_without_git(self):
        self.publish()
        counts = module(HERE.parent / 'tiers/counts.py')
        packet = PRE.budget_input(self.repository, self.env, counts.read(self.repository, self.env))
        path = self.root / 'worker-counts.json'
        path.write_text(json.dumps(packet))
        self.enterContext(mock.patch.object(PRE, 'WORKER_COUNTS', path))
        with mock.patch.object(subprocess, 'Popen', side_effect=AssertionError('sandbox accessed Git history')):
            error, = self.verdict(self.result(self.process(30 * self.G)))
            self.assertIn('instruction growth over allowance', error['log'])

    def test_tree_without_head_has_no_budget_baseline(self):
        self.publish()
        self.git('symbolic-ref', 'HEAD', 'refs/heads/unborn')
        self.assertEqual(self.verdict(self.result(self.process(30 * self.G))), [])
        self.assertIn('source checkout has no commit', self.log.getvalue())

    def test_stale_base_refuses_close(self):
        self.publish([self.main])
        error, = self.verdict(self.result(self.process(25 * self.G)))
        self.assertEqual(error['name'], 'budgets')
        self.assertIn("older than main's last twenty landings; update from main and close again", error['log'])

    def test_one_uncounted_step_warns_once_and_suppresses_red(self):
        self.publish()
        red = self.result(self.process(30 * self.G))
        off = self.result(self.process(None, binary='leaf::other'), counting=False, why='perf_event_paranoid is 3')
        self.assertEqual(self.verdict(red, off), [])
        self.assertEqual(self.log.getvalue(),
                         'budgets: no verdict at close, instruction counts unavailable: perf_event_paranoid is 3\n')

    def test_one_missing_counter_suppresses_red(self):
        self.publish()
        run = self.result(self.process(30 * self.G), self.process(None, binary='leaf::other'))
        self.assertEqual(self.verdict(run), [])
        self.assertIn('leaf::other has no instruction count', self.log.getvalue())

    def test_at_the_allowance_with_an_incomplete_binary(self):
        self.publish()
        partial = (dict(binary_id='leaf::other', outcome='starved', process=1),)
        run = self.result(self.process(27 * self.G), self.process(50 * self.G, binary='leaf::other'), tests=partial)
        self.assertEqual(self.verdict(run), [])
        self.assertIn(f'within instruction budgets at close (base {self.base[:10]})', self.log.getvalue())

    def test_growth_combines_whole_runs_and_names_each_red_layer(self):
        self.publish()
        first = self.result(self.process(26 * self.G), self.process(2 * self.G, kind='fresh'),
                            self.process(50 * self.G, kind='reference'), self.process(None, kind='empty'))
        second = self.result(self.process(11 * self.G, binary='leaf::other'),
                             self.process(2 * self.G, binary='jig::world', tree='jig', layer='domain'))
        error, = self.verdict(first, second)
        self.assertEqual(error['name'], 'budgets')
        lines = error['log'].splitlines()
        self.assertIn('jig default domain: instruction growth over allowance, 0 at base, 2000000000 at head', lines[0])
        self.assertIn('skein default step: instruction growth over allowance, 100000000000 at base, '
                      '104000000000 at head, growth 4.00%, allowance 2000000000 instructions', lines[2])
        self.assertEqual(lines[3:5], ['  leaf::suite: +3000000000 instructions', '  leaf::other: +1000000000 instructions'])
        self.assertEqual(lines[-1], module(HERE.parent / 'tiers/cpu.py').REPAIR_LINE)

    def test_ceiling_only_refuses_when_base_was_within(self):
        self.publish()
        run = self.result(self.process(26 * self.G))
        self.budget_table(100 * self.G)
        error, = self.verdict(run)
        self.assertIn('over instruction ceiling 100000000000', error['log'])
        self.assertIn('100000000000 at base, 101000000000 at head, growth 1.00%, allowance 2000000000', error['log'])
        self.budget_table(99 * self.G)
        self.assertEqual(self.verdict(run), [])

    def test_accepted_level_passes_growth_up_to_it(self):
        self.publish()
        self.budget_table(accepted=f'accepted_instructions = {105 * self.G}\n')
        self.assertEqual(self.verdict(self.result(self.process(30 * self.G))), [])
        error, = self.verdict(self.result(self.process(31 * self.G)))
        self.assertIn('100000000000 at base, 106000000000 at head, growth 6.00%, allowance 5000000000', error['log'])

    def test_accepted_level_raises_the_close_limit_above_the_ceiling(self):
        self.publish()
        self.budget_table(100 * self.G + 1, accepted=f'accepted_instructions = {101 * self.G}\n')
        self.assertEqual(self.verdict(self.result(self.process(25 * self.G + 2))), [])
        self.assertEqual(self.verdict(self.result(self.process(26 * self.G))), [])
        error, = self.verdict(self.result(self.process(26 * self.G + 1)))
        self.assertIn('over accepted instruction level 101000000000', error['log'])

    def test_credit_counts_only_new_moves_and_uses_the_base_binaries(self):
        run = self.result(self.process(28 * self.G))
        for amount in ('', 'instructions = 1000000000\n'):
            with self.subTest(explicit=bool(amount)):
                self.profile['moves'] = []
                self.publish()
                self.budget_table(moves='[[gate.default.moves]]\nfrom = "skein.simulated"\nto = "skein.step"\n'
                                  'cpu_s = 0.2\nwhy = "lower the fixture"\n' + amount)
                self.assertEqual(self.verdict(run), [])
                move = dict(to='skein.step', cpu_s=0.2, why='lower the fixture', **{'from': 'skein.simulated'})
                if amount:
                    move['instructions'] = self.G
                self.profile['moves'] = [move]
                self.publish()
                error, = self.verdict(run)
                self.assertIn('growth 3.00%, allowance 2000000000 instructions', error['log'])

    def close(self, fake_run, *, completed_goal=True):
        source = self.root / 'skein/leaf/tests/suite.rs'
        package = dict(name='leaf', id='leaf', manifest_path=str(source.parents[1] / 'Cargo.toml'),
                       dependencies=[], targets=[dict(name='suite', kind=['test'], test=True, src_path=str(source))])
        target = self.repository / 'target'
        metadata = dict(packages=[package], workspace_members=['leaf'], target_directory=str(target))
        errors = []
        with mock.patch.object(PRE, 'run', side_effect=fake_run) as run, \
             mock.patch.object(PRE, 'format_workspace', return_value=[]), \
             mock.patch.object(PRE, 'baselines', return_value=(0, [])), \
             mock.patch.object(PRE, 'metadata', return_value=metadata), \
             mock.patch.object(PRE, 'journeys', return_value=[]), \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])):
            PRE.workspace_checks(self.root, [str(source.relative_to(self.root))], 'tree', self.env, errors, {},
                                 completed_goal=completed_goal, counts_source=(self.repository, self.env))
        if completed_goal:
            run.assert_called_once()
            command = run.call_args.args[0]
            self.assertEqual(command[:2], [str(PRE.TIERS / 'one-process'), '--out'])
            self.assertEqual(Path(command[2]).parent.parent, target)
            self.assertFalse(Path(command[2]).parent.exists())
            self.assertEqual(command[3:], ['--fail-fast', 'default', '--offline', '--locked', '--no-tests=warn',
                                           '-p', 'leaf', '--test', 'suite'])
        else:
            run.assert_not_called()
        return errors

    def test_empty_selection_passes_even_without_a_counting_decision(self):
        self.publish()

        def empty(command, *_args, **_kwargs):
            directory = Path(command[2])
            self.assertFalse(directory.exists())
            self.result(counting=None, directory=directory)
            return 0, 'warning: no tests to run\n'

        self.assertEqual(self.close(empty), [])
        self.assertNotIn('instruction counts unavailable', self.log.getvalue())

    def test_close_uses_source_ref_and_only_judges_successful_focused_runs(self):
        self.publish()

        def fake(command, *_args, **_kwargs):
            self.result(self.process(30 * self.G), directory=Path(command[2]))
            return status, 'fixture\n'

        self.assertEqual(self.close(fake, completed_goal=False), [])
        status = 100
        error, = self.close(fake)
        self.assertEqual(error['name'], 'focused tests')
        status = 0
        self.assertEqual(self.close(fake), [])
        self.assertIn('pre-pr: budgets at close, a warning: the landing gate judges them\n', self.log.getvalue())
        self.assertIn('instruction growth over allowance', self.log.getvalue())


class Gate(unittest.TestCase):
    def test_complete_candidate_decides_scope(self):
        self.assertEqual(GATE.route(set()), 'full')
        self.assertEqual(GATE.route({'.temper/pre-pr', 'tools/pre-pr/workspace.py'}), 'checker')
        self.assertEqual(GATE.route({'.temper/checked-close', 'tools/pre-pr/pre_pr.py'}), 'checker')
        self.assertEqual(GATE.HOOKS, PRE.HOOKS)
        self.assertEqual(GATE.route({'.temper/pre-pr', 'src/lib.rs'}), 'mixed')
        self.assertEqual(GATE.route({'docs/guide.md'}), 'classify')
        self.assertEqual(GATE.route({'tools/tiers/cpu.py', 'tools/tiers/one-process', 'tools/close/README.md'}), 'tooling')
        for product in ('tools/tiers/budgets.toml', 'tools/tiers/fixtures/one-process/tests.rs'):
            self.assertEqual(GATE.route({'tools/tiers/cpu.py', product}), 'classify', product)

    def test_tooling_runs_the_suites_beside_it_unless_tests_read_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ('tools/example/rules.py', 'tools/example/test_rules.py', 'tools/example/test_more.py',
                         'tools/lonely/tool.py'):
                (root / name).parent.mkdir(parents=True, exist_ok=True)
                (root / name).touch()
            (root / 'tools/pre-pr').mkdir(parents=True)
            (root / 'tools/pre-pr/pre_pr.py').write_text('def test_input_globs(root):\n    return ["tools/read/*"]\n')
            for paths, status, commands in [
                    (b'tools/example/rules.py\0', 0, [['python3', '-I', '-B', 'tools/example/test_more.py'],
                                                     ['python3', '-I', '-B', 'tools/example/test_rules.py']]),
                    (b'tools/lonely/tool.py\0', 1, []),
                    (b'tools/read/x.py\0', 1, [])]:
                with self.subTest(paths=paths):
                    execute = mock.Mock(return_value=subprocess.CompletedProcess([], 0))
                    with mock.patch.object(subprocess, 'check_output', return_value=paths), redirect_stdout(io.StringIO()):
                        self.assertEqual(GATE.checks(root, 'base', 'head', root, execute), status)
                    self.assertEqual([call.args[0] for call in execute.call_args_list], commands)

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
