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

    def test_default_entry_and_explicit_replay(self):
        with mock.patch.object(PRE, 'delivery', return_value=0) as local:
            self.assertEqual(PRE.main([]), 0)
            local.assert_called_once_with('delivery', ())
        with mock.patch.object(PRE, 'goal_check', return_value=1) as goal:
            self.assertEqual(PRE.main(['--goal', 'head', 'base']), 1)
            goal.assert_called_once_with(['head', 'base'])
        with self.assertRaises(ValueError):
            PRE.main(['--goal', 'head'])

    def test_checker_changes_run_only_python_and_write_local_receipt(self):
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=(0, [])):
            selected = PRE.workspace_checks(self.root, ['tools/pre-pr/pre_pr.py'], 'tree', 'base', [],
                                            self.env, self.env, self.errors, self.outcomes)
        self.assertEqual([call.args[0][3] for call in run.call_args_list],
                         ['tools/pre-pr/test_pre_pr.py'])
        outward.assert_called_once_with(self.root, self.env, ['tools/pre-pr/pre_pr.py'])
        self.assertTrue(all(call.args[1] == self.root for call in run.call_args_list))
        self.assertEqual(selected, [])
        self.assertEqual(self.errors, [])
        name = PRE.failure_file('tree', 'base', False, False, [], self.root, {}, 'delivery')
        receipt = json.loads((self.root / name.removeprefix('every key in ')).read_text())
        self.assertEqual((receipt['scope'], receipt['tree'], receipt['verdict']), ('light', 'tree', 'passed'))

    def test_registered_codec_is_tested_but_only_handwritten_rust_is_formatted(self):
        crate = self.root / 'smith/crates/smith-transcript'
        generated = 'smith/crates/smith-transcript/src/generated/transcript.rs'
        handwritten = 'other/src/generated/transcript.rs'
        for path in (generated, handwritten, 'tools/merge/codecs.txt',
                     'tools/tiers/layers.toml', 'tools/tiers/packages.toml'):
            file = self.root / path
            file.parent.mkdir(parents=True, exist_ok=True)
            file.touch()
        (self.root / 'tools/merge/codecs.txt').write_text(f'# Outputs and inputs\n{generated} schema\n')
        target = dict(kind=['lib'], name='transcript', src_path=str(crate / 'src/lib.rs'), test=True)
        package = dict(id='transcript', name='smith-transcript', manifest_path=str(crate / 'Cargo.toml'),
                       edition='2024', dependencies=[], targets=[target])
        data = dict(packages=[package], workspace_members=['transcript'])
        self.enterContext(mock.patch.object(PRE, 'metadata', return_value=data))
        for check in ('baselines', 'outward', 'journeys'):
            self.enterContext(mock.patch.object(PRE, check, return_value=(0, [])))
        run = self.enterContext(mock.patch.object(PRE, 'run', return_value=(0, '')))
        args = (self.root, [generated, handwritten], 'tree', 'base', [], {}, {}, self.errors, {})
        self.assertEqual(PRE.workspace_checks(*args), [])
        fmt, = [call.args[0] for call in run.call_args_list]
        self.assertEqual(fmt[0], 'rustfmt')
        self.assertEqual(fmt[fmt.index('--') + 1:], [handwritten])
        run.reset_mock()
        selected = PRE.workspace_checks(*args, completed_goal=True)
        fmt, tests = [call.args[0] for call in run.call_args_list]
        self.assertEqual(fmt[fmt.index('--') + 1:], [handwritten])
        self.assertEqual(selected, ['smith-transcript'])
        self.assertEqual(tests[:3], ['cargo', 'nextest', 'run'])
        self.assertIn('smith-transcript', tests)

    def test_failed_checker_refuses_and_publishes_reason(self):
        with mock.patch.object(PRE, 'run', side_effect=[(1, 'fixture failed'), (0, '')]), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=(0, [])):
            PRE.workspace_checks(self.root, ['.temper/pre-pr'], 'tree', None, [], {}, {}, self.errors, {})
        self.assertEqual(PRE.finish('tree', None, self.errors, self.root, {}, 'delivery', (), None), 1)
        self.assertIn('fixture failed', self.log.getvalue())

    def test_text_change_and_budget_rules(self):
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=(0, [])):
            PRE.workspace_checks(self.root, ['doc.md', 'tools/tiers/budgets.toml'], 'snapshot', 'start', [],
                                 {}, {}, self.errors, {})
        commands = [call.args[0] for call in run.call_args_list]
        self.assertEqual(commands, [
            ['python3', '-I', '-B', 'tools/citations/citations.py', '--all', '--head', 'snapshot'],
            ['python3', '-I', '-B', 'tools/tiers/figures.py', '--base', 'start', '--head', 'snapshot']])

    def test_changed_paths_include_deletions_and_exclude_cleanly_imported_entries(self):
        results = [subprocess.CompletedProcess([], 0, value, b'') for value in
                   (b'deleted\0added\0from-parent\0', b'added\0deleted\0')]
        with mock.patch.object(PRE, 'git', side_effect=results) as git:
            self.assertEqual(PRE.delivery_paths('start', 'tree', self.root, {}, ['parent']), ['added', 'deleted'])
        self.assertEqual([call.args[0] for call in git.call_args_list],
                         [['diff-tree', '-r', '--no-renames', '--name-only', '-z', ref, 'tree'] for ref in ('start', 'parent')])

    def test_scope_requires_the_checked_base_and_exact_published_tree(self):
        tip = 'a' * 40
        PRE.remember_merges(self.root, 'base', 'published', [tip])
        PRE.remember_merges(self.root, 'other-base', 'unrelated', [tip])
        for tree, expected in [('published', [tip]), ('unrelated', None)]:
            answer = subprocess.CompletedProcess([], 0, f'head\n{tree}\n'.encode(), b'')
            with mock.patch.object(PRE, 'git', return_value=answer):
                if expected:
                    self.assertEqual(PRE.local_scope(self.root, 'base', self.root, {}, []), expected)
                else:
                    with self.assertRaisesRegex(ValueError, 'formatting scope unavailable'):
                        PRE.local_scope(self.root, 'base', self.root, {}, ['older-merge'])

    def test_main_fetch_must_follow_prepare_and_be_integrated(self):
        main = 'a' * 40
        pin = self.root / 'refs/temper/landing'
        pin.parent.mkdir(parents=True)
        pin.touch()
        fetch = self.root / 'FETCH_HEAD'
        fetch.write_text(f"{main}\t\tbranch 'main' of example\n")
        os.utime(pin, (20, 20))
        head = subprocess.CompletedProcess([], 0, b'head\ntree\n', b'')
        missing = subprocess.CompletedProcess([], 1, b'', b'')
        passed = subprocess.CompletedProcess([], 0, b'', b'')
        for stamp, answers, allowed in [(10, [head], False), (21, [head, missing, missing], False),
                                        (21, [head, missing, passed], True)]:
            os.utime(fetch, (stamp, stamp))
            with self.subTest(stamp=stamp, allowed=allowed), mock.patch.object(PRE, 'git', side_effect=answers):
                if allowed:
                    self.assertEqual(PRE.local_scope(self.root, 'base', self.root, {}, []), [main])
                else:
                    with self.assertRaises(PRE.UnknownScope):
                        PRE.local_scope(self.root, 'base', self.root, {}, [])

    def test_cold_delivery_fetches_the_mapped_main_once_in_its_worktree(self):
        url = 'https://example.org/team/repo.git'
        main = 'a' * 40
        revision = subprocess.CompletedProcess([], 0, b'head\nlanding\n', b'')
        effective = subprocess.CompletedProcess([], 0, (url + '\n').encode(), b'')
        fetched = subprocess.CompletedProcess([], 0, b'', b'')
        head = subprocess.CompletedProcess([], 0, b'head\n', b'')
        with mock.patch.object(PRE, 'public_main_source', return_value=url), \
             mock.patch.object(PRE, 'checkout_main', side_effect=[None, (main, url[:-4])]), \
             mock.patch.object(PRE, 'git', side_effect=[revision, effective, fetched, revision, head]) as git, \
             mock.patch.object(PRE, 'integrated_main', return_value=True):
            self.assertEqual(PRE.recover_main(self.root, self.root, {}), main)
        commands = [call.args[0] for call in git.call_args_list]
        self.assertEqual([c for c in commands if c[0] == 'fetch'],
                         [['fetch', '--no-tags', '--no-recurse-submodules', '--', url, 'main']])
        self.assertTrue(all(call.args[1] == self.root for call in git.call_args_list))

    def test_waiting_for_goal_integration_reuses_the_fetched_tip(self):
        url = 'https://example.org/team/repo.git'
        main = 'b' * 40
        head = subprocess.CompletedProcess([], 0, b'head\n', b'')
        with mock.patch.object(PRE, 'public_main_source', return_value=url), \
             mock.patch.object(PRE, 'checkout_main', return_value=(main, url)), \
             mock.patch.object(PRE, 'git', return_value=head) as git, \
             mock.patch.object(PRE, 'integrated_main', return_value=False):
            with self.assertRaisesRegex(PRE.UnknownScope, 'goal owns main integration'):
                PRE.recover_main(self.root, self.root, {})
        self.assertEqual([call.args[0] for call in git.call_args_list], [['rev-parse', '--verify', 'HEAD']])

    def test_clean_automatic_import_is_excluded_from_formatted_files(self):
        files = ['owned.rs', 'working.rs', 'combined-on-main.rs']
        values = [subprocess.CompletedProcess([], 0, b'owned.rs\0', b''),
                  subprocess.CompletedProcess([], 1, b'', b''),
                  subprocess.CompletedProcess([], 0, b'working.rs\0', b'')]
        with mock.patch.object(PRE, 'handwritten_rust', return_value=files), \
             mock.patch.object(PRE, 'git', side_effect=values):
            self.assertEqual(PRE.producer_format_paths(self.root, {}, 'base', 'tree', files), files[:2])

    def test_remaining_checks_and_receipt_use_the_formatted_tree(self):
        with mock.patch.object(Path, 'cwd', return_value=self.root), \
             mock.patch.object(subprocess, 'check_output', return_value=str(self.root).encode()), \
             mock.patch.object(PRE, 'base', return_value=('base', None)), \
             mock.patch.object(PRE, 'local_scope', return_value=[]), \
             mock.patch.object(PRE, 'merged_tips', return_value=[]), \
             mock.patch.object(PRE, 'goal_reds', return_value=(set(), None, False)), \
             mock.patch.object(PRE, 'tree', side_effect=[('before', {}), ('formatted', {})]), \
             mock.patch.object(PRE, 'delivery_paths', side_effect=[['one.rs', 'two.rs'], ['two.rs']]), \
             mock.patch.object(PRE, 'format_delivery', return_value='formatted'), \
             mock.patch.object(PRE, 'workspace_checks', return_value=[]) as checks:
            self.assertEqual(PRE.delivery('delivery', ()), 0)
        self.assertEqual(checks.call_args.args[1:3], (['two.rs'], 'formatted'))
        receipt = json.loads((self.root / 'target/temper/pre-pr-light-failures.json').read_text())
        self.assertEqual((receipt['tree'], receipt['verdict']), ('formatted', 'passed'))

    def test_candidate_snapshot_uses_only_a_private_index(self):
        gitdir = self.root / '.git'
        gitdir.mkdir()
        (gitdir / 'index').write_bytes(b'original')
        index = self.root / 'private-index'
        with mock.patch.object(PRE, 'git', return_value=subprocess.CompletedProcess([], 0, b'tree', b'')) as git:
            tree, env = PRE.tree(self.root, gitdir, index, {})
        self.assertEqual(tree, 'tree')
        self.assertEqual(env['GIT_INDEX_FILE'], str(index))
        self.assertEqual([call.args[0] for call in git.call_args_list], [['add', '-A'], ['write-tree']])
        self.assertEqual((gitdir / 'index').read_bytes(), b'original')

    def test_changed_workspace_never_gets_a_pass(self):
        for snapshots, expected in [(('first', 'second'), 1), (('first', 'first'), 0)]:
            with self.subTest(snapshots=snapshots), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.object(subprocess, 'check_output', return_value=str(self.root).encode()), \
                 mock.patch.object(PRE, 'base', return_value=('start', None)), \
                 mock.patch.object(PRE, 'local_scope', return_value=[]), \
                 mock.patch.object(PRE, 'merged_tips', return_value=[]), \
                 mock.patch.object(PRE, 'delivery_paths', return_value=[]), \
                 mock.patch.object(PRE, 'workspace_checks', return_value=[]), \
                 mock.patch.object(PRE, 'goal_reds', return_value=(set(), None, True)), \
                 mock.patch.object(PRE, 'check_results') as report, \
                 mock.patch.object(PRE, 'tree', side_effect=[(value, {}) for value in snapshots]):
                self.assertEqual(PRE.main([]), expected)
                self.assertEqual(report.call_args.args[:3], ('first', 'start', expected))

    def test_journey_generated_page_mixed_with_code_refuses(self):
        with mock.patch.object(PRE, 'git', return_value=subprocess.CompletedProcess([], 0, b'', b'')):
            code, failures = PRE.journeys('base', 'tree', ['temper/docs/design/client/web/journeys/a/01.html',
                                                         'other.rs'], self.root, {})
        self.assertEqual(code, 1)
        self.assertEqual(failures[0]['name'], 'journeys')

    def test_explicit_goal_rejects_wrong_head(self):
        head, dest = 'a' * 40, 'b' * 40
        with mock.patch.object(PRE, 'git', side_effect=[subprocess.CompletedProcess([], 0, commit.encode(), b'')
                                                       for commit in (head, dest, dest)]):
            with self.assertRaisesRegex(ValueError, 'checkout HEAD'):
                PRE.goal_base([head, dest], self.root, self.root, {})

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
             mock.patch.object(PRE, 'journeys', return_value=(0, [])), mock.patch.object(PRE, 'outward', return_value=(0, [])):
            args = (self.root, ['tools/example/rules.py', 'tools/example/test_rules.py'],
                    'tree', 'base', [], {}, {}, self.errors, {})
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

    def test_goal_close_checks_the_whole_tree_without_reapplying_delivery_mixing(self):
        with mock.patch.object(PRE, 'outward', return_value=(0, [])), \
             mock.patch.object(PRE, 'figures') as figures, \
             mock.patch.object(PRE, 'journeys', return_value=(0, [])) as journeys:
            PRE.workspace_checks(self.root, ['tools/tiers/budgets.toml', 'notes'], 'tree', 'base',
                                 [], {}, {}, self.errors, {}, completed_goal=True)
        figures.assert_not_called()
        journeys.assert_called_once_with('base', 'tree', [], self.root, {})

    def test_close_receipt_identifies_pre_settlement_head_and_post_settlement_tree(self):
        self.assertEqual(PRE.finish('staged', 'base', [], self.root, {}, 'close', ('head', 'main'), None), 0)
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual((receipt['goal_head'], receipt['destination'], receipt['base'], receipt['tree'],
                          receipt['verdict']), ('head', 'main', 'base', 'staged', 'passed'))

    def test_known_outward_failure_requires_a_complete_scan(self):
        key = ('test', 'fabrig-boundary::outward', 'no_file_names_the_lab')
        with mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'journeys', return_value=(0, [])):
            PRE.workspace_checks(self.root, ['other'], 'tree', 'base', [], {}, {}, self.errors, self.outcomes, [key])
        outward.assert_called_once_with(self.root, {}, None)
        self.assertEqual(self.outcomes[key[1:]], {'outcome': 'passed'})

    def test_large_receipt_preserves_every_key_and_bounds_summary(self):
        errors = [PRE.failure(f'test api::suite case_{i}', 'x' * 9000, ('api::suite', f'case_{i}'))
                  for i in range(450)]
        PRE.inherited({('test', 'api::suite', 'case_0')}, errors)
        errors[1]['flake'] = (1, 0.1)
        self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'goal', ['head', 'dest'], None), 1)
        receipt = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(receipt['counts'], {'new': 448, 'flake': 1, 'inherited': 1})
        self.assertEqual({entry['key']['test'] for entry in receipt['failures']}, {f'case_{i}' for i in range(450)})
        self.assertLessEqual(len(self.log.getvalue().encode()), 8192)

    def test_diagnostic_cpu_measurements_do_not_enable_budget_refusals(self):
        for profile in ('default', 'fuzzy'):
            with mock.patch.object(PRE, 'run', return_value=(2, 'one-process: nextest listing exited with 101')) as run:
                result, errors, _ = PRE.one_process(['api'], self.root, {}, profile=profile, ledger='main-ledger')
            self.assertEqual(result, 2)
            self.assertTrue(errors[0]['name'].startswith('build'))
            cmd = run.call_args.args[0]
            self.assertEqual(cmd[cmd.index('--ledger') + 1:], ['main-ledger', profile, '-p', 'api', '--no-tests=warn'])
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

    def test_pending_merge_and_destination_bound_child_receipt(self):
        PRE.finish('tree', 'base', [], self.root, {}, 'child', ['dest'], ['head', 'other'])
        record = json.loads((self.root / 'target/temper/goal-child-pre-pr.json').read_text())
        self.assertEqual((record['scope'], record['delivery_head'], record['merge_heads'], record['destination']),
                         ('child', 'head', ['other'], 'dest'))


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
