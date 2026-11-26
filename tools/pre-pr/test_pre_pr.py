"""Delivery policy checks with recording fakes and a scratch counts repository."""
import contextlib
from contextlib import redirect_stderr, redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
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
                         [['tools/tiers/one-process', '--out', str(directory / str(step)), '--fail-fast', profile,
                           '--offline', '--locked', '--no-tests=warn', '-p', 'leaf',
                           '-E', 'default() & (((package(=leaf)) & ((kind(test) & binary(=suite)))))']
                          for step, profile in enumerate(('default', 'fuzzy'))])

    def test_every_group_runs_in_one_run_with_its_own_targets(self):
        groups = {(('--lib',), ('--test', 'suite')): ['leaf', 'middle'], (('--bin', 'serve'),): ['top'],
                  (): ['quiet'], (('--lib',),): ['fabrig-boundary']}
        commands = list(PLAN.rust_commands(groups, {}, self.root / 'target/close'))
        self.assertEqual(commands[0], ['cargo', 'check', '--offline', '--locked', '-p', 'quiet'])
        run, fuzzy = commands[1:]
        same = lambda command: [arg for i, arg in enumerate(command) if i not in (2, 4)]
        self.assertEqual(same(fuzzy), same(run))
        self.assertEqual((run[4], fuzzy[4], fuzzy[2]), ('default', 'fuzzy', str(self.root / 'target/close/1')))
        self.assertEqual(run[run.index('--no-tests=warn') + 1:run.index('-E')],
                         ['-p', 'fabrig-boundary', '-p', 'leaf', '-p', 'middle', '-p', 'top'])
        expression = run[-1]
        self.assertTrue(expression.startswith('default() & ('))
        self.assertIn('((package(=leaf) | package(=middle)) & ((kind(lib) | kind(proc-macro)) | '
                      '(kind(test) & binary(=suite))))', expression)
        self.assertIn('((package(=top)) & ((kind(bin) & binary(=serve))))', expression)
        self.assertIn('& not (package(=fabrig-boundary) & (', expression)
        self.assertNotIn('quiet', expression)

    def test_boundary_filter_keeps_candidate_checks_and_the_profile(self):
        flag, expression = PLAN.workspace_filter(['fabrig-boundary', 'leaf'])
        self.assertEqual(flag, '-E')
        self.assertIn('default() & not (package(=fabrig-boundary)', expression)
        self.assertIn('test(=client_versions_are_frozen)', expression)
        self.assertNotIn('a_field_added_at_the_same_version_passes', expression)
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
        self.assertEqual(len(commands), 3)


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

    def receipt(self, name='goal-close-pre-pr.json'):
        return json.loads((self.root / 'target/temper' / name).read_text())

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
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward:
            selected = PRE.workspace_checks(self.root, ['tools/pre-pr/pre_pr.py'], 'tree',
                                            self.env, self.errors, self.outcomes)
        self.assertEqual([call.args[0][3] for call in run.call_args_list],
                         ['tools/pre-pr/test_pre_pr.py', 'tools/citations/citations.py'])
        outward.assert_called_once_with(self.root, self.env, ['tools/pre-pr/pre_pr.py'])
        self.assertTrue(all(call.args[1] == self.root for call in run.call_args_list))
        self.assertEqual(selected, [])
        self.assertEqual(self.errors, [])
        self.assertEqual(PRE.finish('tree', 'base', [], self.root, {}, 'delivery', (), None), 0)
        receipt = json.loads((self.root / 'target/temper/pre-pr-light-failures.json').read_text())
        self.assertEqual((receipt['scope'], receipt['tree'], receipt['verdict']), ('light', 'tree', 'passed'))

    def test_a_helper_that_cannot_start_or_is_killed_breaks(self):
        process = mock.MagicMock(stdout=io.BytesIO(b''))
        process.wait.return_value = -9
        for unable, detail in [(True, 'python3 is missing'), (False, 'Python tests: killed by signal 9')]:
            options = {'side_effect': FileNotFoundError(detail)} if unable else {'return_value': process}
            with self.subTest(unable=unable), mock.patch.object(subprocess, 'Popen', **options), \
                 mock.patch.object(PRE, 'citations', return_value=(0, [])), \
                 mock.patch.object(PRE, 'outward', return_value=(0, [])), redirect_stderr(io.StringIO()) as err:
                errors = []
                PRE.workspace_checks(self.root, ['.temper/pre-pr'], 'tree', {}, errors, {})
                self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'delivery', (), None), 1)
            receipt = self.receipt('pre-pr-light-failures.json')
            self.assertEqual(receipt['verdict'], 'broke')
            self.assertEqual(receipt['failures'], [{'class': 'broke', 'name': 'check Python tests', 'key': None,
                                                   'detail': detail}])
            self.assertTrue(err.getvalue().endswith(f'smith-check: not judged\npre-pr: broke: {detail}\n'))

    def test_candidate_tool_diagnostics_refuse(self):
        lint = {'reason': 'compiler-message', 'package_id': 'api', 'message': {
            'level': 'error', 'code': {'code': 'clippy::rule'}, 'spans': [], 'message': 'lint failed'}}
        steps = iter([(b'flavor: count 9, baseline 8\n', 1), (b'broken citation\n', 1),
                      (b'outward path\n', 1), (json.dumps(lint).encode() + b'\n', 101)])

        def tool(args, root, env, output=None, stdout=False):
            raw, status = next(steps)
            if output:
                log = PRE.Tail()
                output(raw, log)
            return status, raw.decode()

        with mock.patch.object(PRE, 'run', side_effect=tool), \
             redirect_stderr(io.TextIOWrapper(io.BytesIO())), redirect_stdout(io.TextIOWrapper(io.BytesIO())):
            for run, expected in [
                (lambda: PRE.baselines('skein', self.root, {}), 'old-forms skein flavor'),
                (lambda: PRE.citations('tree', self.root, {}), 'citations'),
                (lambda: PRE.outward(self.root, {}), 'test fabrig-boundary::outward no_file_names_the_lab'),
                (lambda: PRE.clippy(['api'], {'packages': [{'id': 'api', 'name': 'api'}]}, self.root, {}),
                 'clippy api clippy::rule '),
            ]:
                with self.subTest(expected=expected):
                    _, errors = run()
                    self.assertEqual([error['name'] for error in errors], [expected])
                    self.assertEqual(PRE.outcome(PRE.failure_groups(errors)), 'refused')

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
        self.assertEqual(PRE.outcome(PRE.failure_groups(errors)), 'refused')

    def test_failed_checker_refuses_and_publishes_reason(self):
        with mock.patch.object(PRE, 'run', side_effect=[(1, 'fixture failed'), (0, '')]), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward:
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
        self.assertEqual(PRE.journeys(self.root, [], 'HEAD'), [])
        overlay.write_text('nonempty')
        self.assertIn('overlay.css must be empty', PRE.journeys(self.root, [], 'HEAD')[0]['log'])

    def test_written_journey_paths(self):
        web, tui = PRE.JOURNEY_ROOTS
        cases = [
            (f'{web}/chat-draft/03.html', True),
            (f'{tui}/design-session/12.txt', True),
            (f'{web}/a/100.html', True),
            (f'{web}/project-landing/recording', True),
            (f'{web}/skein-ui.css', True),
            (f'{web}/journey.js', True),
            (f'{web}/theme.css', True),
            (f'{web}/tui-journey.js', True),
            (f'{tui}/skein-ui.css', True),
            (f'{tui}/journey.js', True),
            (f'{tui}/theme.css', True),
            (f'{tui}/tui-journey.js', True),
            (f'{web}/looks.html', True),
            (f'{tui}/looks.html', True),
            (f'{web}/chat-draft/looks.html', False),
            ('temper/docs/design/client/web/looks.html', False),
            (f'{web}/chat-draft/3.html', False),
            (f'{web}/chat-draft/walk', False),
            (f'{web}/chat-draft/README.md', False),
            (f'{web}/chat-draft/scripts.json', False),
            (f'{web}/overlay.css', False),
            (f'{web}/README.md', False),
            (f'{web}/03.html', False),
            (f'{web}/a/b/03.html', False),
            ('temper/docs/design/client/web/03.html', False),
            ('skein/journeys/a/03.html', False),
        ]
        for path, written in cases:
            with self.subTest(path=path):
                self.assertEqual(PRE.written_journey_file(path), written)

    def journey_failure(self, paths, base='HEAD'):
        errors = PRE.journeys(self.root, paths, base)
        self.assertEqual(len(errors), 1)
        self.assertEqual(errors[0]['name'], 'journeys')
        return errors[0]

    def test_mixed_paths_name_both_kinds_and_base_repair(self):
        web, tui = PRE.JOURNEY_ROOTS
        paths = [f'{web}/chat-draft/03.html', f'{tui}/theme.css', 'skein/a.rs']
        base = 'b' * 40
        error = self.journey_failure(paths, base)
        self.assertFalse(PRE.settled(error))
        for path in paths:
            self.assertIn(path, error['log'])
        self.assertEqual(error['log'].splitlines()[-1],
                         "a journey's recording, pages and the files beside them are written on main by "
                         f"the journeys' writer alone (AGENTS.md, 4): restore them with git checkout {base} -- PATH..., "
                         'and compare pages by cargo run -p temper-client-walk -- print --out DIR in your own tree')

    def test_deleted_recording_counts_as_written(self):
        recording = f'{PRE.JOURNEY_ROOTS[0]}/x/recording'
        error = self.journey_failure([recording, 'skein/a.rs'])
        self.assertIn(recording, error['log'])

    def test_mixture_names_only_first_twenty_written_paths(self):
        paths = [f'{PRE.JOURNEY_ROOTS[0]}/x/{number:02}.html' for number in range(25)]
        error = self.journey_failure(['skein/a.rs', *reversed(paths)])
        lines = error['log'].splitlines()
        self.assertEqual(lines[1:22], [*paths[:20], '… and 5 more written files'])
        for path in paths[20:]:
            self.assertNotIn(path, error['log'])

    def test_written_paths_alone_pass(self):
        web, tui = PRE.JOURNEY_ROOTS
        paths = [f'{web}/chat-draft/03.html', f'{web}/x/recording', f'{tui}/theme.css']
        self.assertEqual(PRE.journeys(self.root, paths, 'HEAD'), [])

    def test_looks_html_passes_alone_or_with_pages(self):
        for prefix in PRE.JOURNEY_ROOTS:
            looks = f'{prefix}/looks.html'
            for paths in ([looks], [looks, f'{prefix}/chat-draft/03.html']):
                with self.subTest(paths=paths):
                    self.assertEqual(PRE.journeys(self.root, paths, 'HEAD'), [])

    def test_looks_html_with_a_source_is_refused(self):
        for prefix in PRE.JOURNEY_ROOTS:
            looks = f'{prefix}/looks.html'
            for source in (f'{prefix}/chat-draft/README.md', 'tools/x.py'):
                with self.subTest(looks=looks, source=source):
                    error = self.journey_failure([looks, source])
                    self.assertEqual(error['log'].splitlines()[:3],
                                     ['written files:', looks, f'other path: {source}'])

    def retired_paths(self, prefix):
        return [f'{prefix}/old/{name}' for name in ('01.html', 'README.md', 'scripts.json', 'walk')]

    def test_whole_journey_deletion_passes_with_only_written_paths(self):
        for prefix in PRE.JOURNEY_ROOTS:
            for extra in ([], [f'{prefix}/new/01.html'], [f'{prefix}/looks.html']):
                with self.subTest(prefix=prefix, extra=extra):
                    paths = self.retired_paths(prefix) + extra
                    self.assertEqual(PRE.journeys(self.root, paths, 'HEAD'), [])

    def test_whole_journey_deletion_with_a_source_is_a_mixture(self):
        for prefix in PRE.JOURNEY_ROOTS:
            with self.subTest(prefix=prefix):
                paths = self.retired_paths(prefix)
                error = self.journey_failure([*paths, 'skein/a.rs'])
                self.assertEqual(error['log'].splitlines()[:-1],
                                 ['written files:', *paths, 'other path: skein/a.rs'])

    def test_deleted_walk_with_written_files_still_present_is_refused(self):
        for prefix in PRE.JOURNEY_ROOTS:
            walk = f'{prefix}/old/walk'
            for name in ('01.html', 'recording'):
                written = self.root / prefix / 'old' / name
                written.parent.mkdir(parents=True, exist_ok=True)
                written.touch()
                for extra in ([], ['skein/a.rs']):
                    with self.subTest(prefix=prefix, name=name, extra=extra):
                        error = self.journey_failure([walk, *extra])
                        lines = error['log'].splitlines()
                        self.assertEqual(lines[:-1], [f"{walk} deleted while its pages stay: "
                                                      "delete the journey's directory whole, or keep its walk"])
                        self.assertIn('git checkout HEAD -- PATH...', lines[-1])
                        self.assertTrue(lines[-1].endswith('cargo run -p temper-client-walk -- print --out DIR '
                                                           'in your own tree'))
                written.unlink()

    def test_journey_deletion_that_leaves_a_source_is_not_whole(self):
        for prefix in PRE.JOURNEY_ROOTS:
            with self.subTest(prefix=prefix):
                directory = f'{prefix}/old'
                (self.root / directory).mkdir(parents=True)
                (self.root / directory / 'README.md').touch()
                walk, page = f'{directory}/walk', f'{directory}/01.html'
                error = self.journey_failure([walk, page])
                self.assertEqual(error['log'].splitlines()[:-1], ['written files:', page, f'other path: {walk}'])

    def test_no_written_paths_pass(self):
        for prefix in PRE.JOURNEY_ROOTS:
            with self.subTest(prefix=prefix):
                walk = f'{prefix}/chat-draft/walk'
                (self.root / walk).parent.mkdir(parents=True)
                (self.root / walk).write_text('walk 2\n')
                paths = ['skein/a.rs', walk, f'{prefix}/README.md']
                self.assertEqual(PRE.journeys(self.root, paths, 'HEAD'), [])

    def test_nonempty_overlay_refuses_even_without_changed_paths(self):
        for prefix in PRE.JOURNEY_ROOTS:
            with self.subTest(prefix=prefix):
                overlay = self.root / prefix / 'overlay.css'
                overlay.parent.mkdir(parents=True)
                overlay.write_text(' ')
                error = self.journey_failure([])
                self.assertIn(f'{prefix}/overlay.css must be empty', error['log'])
                overlay.unlink()

    def test_mixture_and_overlays_give_one_failure(self):
        overlays = [self.root / prefix / 'overlay.css' for prefix in PRE.JOURNEY_ROOTS]
        for overlay in overlays:
            overlay.parent.mkdir(parents=True)
            overlay.write_text(' ')
        paths = [f'{PRE.JOURNEY_ROOTS[0]}/chat-draft/03.html', 'skein/a.rs']
        error = self.journey_failure(paths)
        for path in paths + [str(overlay.relative_to(self.root)) for overlay in overlays]:
            self.assertIn(path, error['log'])

    def test_journeys_run_last_even_after_an_earlier_failure(self):
        def workspace(root, paths, tree, env, errors, outcomes, **options):
            print('workspace checks finished')
            if failed:
                errors.append(PRE.failure('format', 'an earlier failure'))
            return []

        def goal(*args):
            print('goal checks finished')
            return [], [], []

        for args in ([], ['--child'], ['--goal'], ['--close']):
            for failed in (False, True):
                with self.subTest(args=args, failed=failed), contextlib.ExitStack() as stack:
                    out = stack.enter_context(redirect_stdout(io.StringIO()))
                    stack.enter_context(redirect_stderr(io.StringIO()))
                    stack.enter_context(mock.patch.object(Path, 'cwd', return_value=self.root))
                    stack.enter_context(mock.patch.object(PRE, 'checked_git', return_value=b''))
                    stack.enter_context(mock.patch.object(PRE, 'snapshot', side_effect=['first', 'first']))
                    stack.enter_context(mock.patch.object(PRE, 'scope_base', return_value=('b' * 40, 'base', None)))
                    stack.enter_context(mock.patch.object(PRE, 'changed_paths', return_value=[]))
                    stack.enter_context(mock.patch.object(PRE, 'workspace_checks', side_effect=workspace))
                    stack.enter_context(mock.patch.object(PRE, 'goal_checks', side_effect=goal))
                    errors = [PRE.failure('journeys', 'mixture')] if failed else []
                    stack.enter_context(mock.patch.object(PRE, 'journeys', return_value=errors))
                    self.assertEqual(PRE.main(args), int(failed))
                    log = out.getvalue()
                    previous = 'goal checks' if args == ['--goal'] and not failed else 'workspace checks'
                    result = 'failed' if failed else 'passed'
                    self.assertRegex(log, rf'{previous} finished\npre-pr: journeys …\n'
                                          rf'pre-pr: journeys: {result}, \d+\.\d{{3}} s\n')

    def test_a_checker_exception_breaks_with_a_bounded_stderr_reason(self):
        first_line = 'Cannot check: ' + 'é' * 350
        for exception in (ValueError, OSError):
            with self.subTest(exception=exception), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.object(PRE, 'checked_git', return_value=b''), \
                 mock.patch.object(PRE, 'snapshot', return_value='tree'), \
                 mock.patch.object(PRE, 'scope_base', return_value=('base', 'pin', None)), \
                 mock.patch.object(PRE, 'changed_paths', return_value=[]), \
                 mock.patch.object(PRE, 'workspace_checks', side_effect=exception(first_line + '\nsecond line')), \
                 redirect_stderr(io.StringIO()) as err:
                self.log.truncate(0)
                self.log.seek(0)
                self.assertEqual(PRE.main([]), 1)
            receipt = self.receipt('pre-pr-light-failures.json')
            self.assertEqual(receipt['verdict'], 'broke')
            self.assertEqual(receipt['counts'], {'new': 0, 'flake': 0, 'inherited': 0, 'broke': 1})
            self.assertEqual(receipt['failures'], [{'class': 'broke', 'name': 'check workspace checks', 'key': None,
                                                   'detail': first_line}])
            summary = self.log.getvalue().split('pre-pr: broke', 1)[1].splitlines()
            self.assertTrue(summary[0].startswith(' (delivery, base base, pin)'))
            self.assertEqual(summary[1], 'failures: 0 new, 0 flakes, 0 inherited, 1 broke; '
                             'every key in target/temper/pre-pr-light-failures.json')
            marker, reason = err.getvalue().splitlines(keepends=True)[-2:]
            self.assertEqual(marker, 'smith-check: not judged\n')
            last = reason.encode()
            self.assertEqual(last, ('pre-pr: broke: ' + first_line).encode()[:127].decode(errors='ignore').encode() + b'\n')
            self.assertLessEqual(len(last), 128)

    def test_snapshot_changed_during_checks_breaks(self):
        for final, expected, verdict in [('first', 0, 'passed'), ('other', 1, 'broke')]:
            with self.subTest(final=final), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.object(PRE, 'checked_git', return_value=b''), \
                 mock.patch.object(PRE, 'snapshot', side_effect=['first', final]), \
                 mock.patch.object(PRE, 'scope_base', return_value=('b' * 40, 'refs/temper/landing', None)), \
                 mock.patch.object(PRE, 'changed_paths', return_value=[]), \
                 mock.patch.object(PRE, 'workspace_checks', return_value=[]), redirect_stderr(io.StringIO()):
                self.assertEqual(PRE.main([]), expected)
            self.assertEqual(self.receipt('pre-pr-light-failures.json')['verdict'], verdict)

    def test_no_tree_leaves_the_previous_receipt_and_reports_not_judged(self):
        receipt = self.root / 'target/temper/pre-pr-light-failures.json'
        receipt.parent.mkdir(parents=True)
        receipt.write_bytes(b'previous receipt')
        with mock.patch.object(Path, 'cwd', return_value=self.root), \
             mock.patch.object(PRE, 'checked_git', return_value=b''), \
             mock.patch.object(PRE, 'snapshot', side_effect=OSError('snapshot unavailable')), \
             redirect_stderr(io.StringIO()) as err:
            self.assertEqual(PRE.main([]), 1)
        self.assertEqual(err.getvalue(), 'smith-check: not judged\npre-pr: broke: snapshot unavailable\n')
        self.assertEqual(receipt.read_bytes(), b'previous receipt')

    def test_top_level_exception_reports_not_judged(self):
        script = str(HERE / 'pre_pr.py')
        with mock.patch.object(sys, 'argv', [script]), \
             mock.patch.object(Path, 'cwd', side_effect=OSError('no current directory')), \
             redirect_stderr(io.StringIO()) as err, self.assertRaises(SystemExit) as ended:
            runpy.run_path(script, run_name='__main__')
        self.assertEqual(ended.exception.code, 1)
        self.assertEqual(err.getvalue(), 'smith-check: not judged\npre-pr: broke: no current directory\n')

    def test_no_checkout_breaks_and_checks_nothing(self):
        with mock.patch.object(Path, 'cwd', return_value=self.root), \
             mock.patch.object(PRE, 'checked_git', return_value=b''), \
             mock.patch.object(PRE, 'snapshot', return_value='tree'), \
             mock.patch.object(PRE, 'workspace_checks') as checks, \
             mock.patch.object(PRE, 'journeys') as journeys, redirect_stderr(io.StringIO()) as err:
            self.assertEqual(PRE.main([]), 1)
        checks.assert_not_called()
        journeys.assert_not_called()
        self.assertEqual(self.receipt('pre-pr-light-failures.json')['verdict'], 'broke')
        self.assertIn('pre-pr: broke: no base: this is no git checkout', err.getvalue())

    def test_shell_entry_points_forward_to_the_expected_mode(self):
        for hook, suffix in [('pre-pr', ''), ('goal-child-pre-pr', ' --child "$@"'), ('goal-pre-pr', ' --goal "$@"'),
                             ('checked-close', ' --close')]:
            script = HERE.parents[1] / '.temper' / hook
            commands = [line for line in script.read_text().splitlines() if line.startswith('exec ')]
            self.assertEqual(commands, ['exec python3 -I -B tools/pre-pr/pre_pr.py' + suffix])
            self.assertTrue(script.stat().st_mode & 0o111)

    def test_goal_hook_installs_only_a_missing_rerun_marker_before_goal_mode(self):
        from shutil import copy2
        checkout = self.root / 'checkout'
        git_dir = self.root / 'metadata'
        hooks = checkout / '.temper'
        hooks.mkdir(parents=True)
        hook = copy2(HERE.parents[1] / '.temper/goal-pre-pr', hooks)
        stand_in = checkout / 'tools/pre-pr/pre_pr.py'
        stand_in.parent.mkdir(parents=True)
        stand_in.write_text(
            'import sys\n'
            'from pathlib import Path\n'
            'import subprocess\n'
            'assert sys.argv[1:] == ["--goal", "head", "dest"]\n'
            'marker = Path(subprocess.check_output(\n'
            '    ["git", "rev-parse", "--git-path", "temper/rerun"], text=True).strip())\n'
            'print(marker.read_text(), end="")\n'
        )
        with mock.patch.object(subprocess, 'Popen', side_effect=POPEN):
            subprocess.run(['git', 'init', '-q', '--separate-git-dir', git_dir], cwd=checkout, check=True)
            marker = checkout / subprocess.check_output(
                ['git', 'rev-parse', '--git-path', 'temper/rerun'], cwd=checkout, text=True).strip()
            marker.parent.mkdir()
            temporary = marker.with_suffix('.tmp')
            temporary.write_bytes(b'unfinished\n')
            staged_inode = temporary.stat().st_ino
            call = [hook, 'head', 'dest']
            self.assertEqual(subprocess.check_output(call, cwd=self.root), b'temper-rerun 1 0\n')
            self.assertEqual(marker.stat().st_ino, staged_inode, 'the staged marker was renamed')
            self.assertFalse(temporary.exists())

            marker.write_bytes(b'temper-rerun 1 4\n')
            self.assertEqual(subprocess.check_output(call, cwd=self.root), b'temper-rerun 1 4\n')
            self.assertEqual(marker.stat().st_ino, staged_inode, 'the worker marker was not replaced')

    def test_changed_python_module_runs_its_sibling_suite_once(self):
        directory = self.root / 'tools/example'
        directory.mkdir(parents=True)
        (directory / 'test_rules.py').touch()
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])):
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
                 mock.patch.object(PRE, 'outward', return_value=(0, [])), \
                 mock.patch.object(PRE, 'baselines', return_value=(0, [])):
                PRE.workspace_checks(self.root, paths, 'tree', {}, self.errors, {}, whole=False)
                self.assertEqual([call.args[0] for call in run.call_args_list if call.args[0][0] == 'cargo'], expected)
        self.assertEqual(self.errors, [])

    def test_a_goal_changing_no_tooling_runs_no_python_suite(self):
        (self.root / 'tools/example').mkdir(parents=True)
        (self.root / 'tools/example/test_rules.py').touch()
        with mock.patch.object(PRE, 'run', return_value=(0, '')) as run, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])):
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

    def test_a_red_stands_beside_a_broken_check(self):
        errors = [PRE.broken('helper', 'not executable'),
                  PRE.failure('test api::suite inherited', 'known', ('api::suite', 'inherited')),
                  PRE.failure('test api::suite new', 'assertion failed', ('api::suite', 'new'))]
        PRE.inherited({('test', 'api::suite', 'inherited')}, errors)
        self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'delivery', (), None), 1)
        self.assertIn('pre-pr: refused', self.log.getvalue())
        receipt = self.receipt('pre-pr-light-failures.json')
        self.assertEqual(receipt['verdict'], 'refused')
        self.assertEqual(receipt['counts'], {'new': 1, 'flake': 0, 'inherited': 1, 'broke': 1})
        self.assertEqual([entry['class'] for entry in receipt['failures']], ['new', 'inherited', 'broke'])
        self.assertLess(self.log.getvalue().index('inherited: test'), self.log.getvalue().index('broke: check helper'))

    def test_typed_close_needs_the_exact_opt_in_and_only_applies_to_close(self):
        result = self.root / 'result.json'
        settlement = b'{"settled":[]}\n'
        red = PRE.failure('format', 'diff')
        broke = PRE.broken('format', 'missing')
        cases = [('close', None, [red], 1), ('close', '', [red], 1), ('close', '1', [red], 1),
                 ('close', None, [broke], 1), ('close', '', [broke], 1), ('close', '1', [broke], 1),
                 ('close', '3', [broke], 2), ('close', '3', [], 0)]
        cases += [(scope, '3', errors, 1) for scope in ('delivery', 'child', 'goal') for errors in ([red], [broke])]
        for scope, value, errors, status in cases:
            with self.subTest(scope=scope, value=value, errors=errors), redirect_stderr(io.StringIO()) as err:
                result.write_bytes(settlement)
                env = {'TEMPER_JOB': str(self.root)}
                if value is not None:
                    env['TEMPER_CLOSE_REFUSAL'] = value
                self.assertEqual(PRE.finish('tree', 'base', errors, self.root, env, scope, (), ['head']), status)
                if errors == [broke]:
                    self.assertTrue(err.getvalue().endswith('smith-check: not judged\npre-pr: broke: missing\n'))
                else:
                    self.assertNotIn('smith-check: not judged', err.getvalue())
                self.assertEqual(result.read_bytes(), settlement)

    def test_a_refusal_the_checker_cannot_write_exits_two(self):
        result = self.root / 'result.json'
        settlement = b'{"settled":[]}\n'
        result.write_bytes(settlement)
        replace = os.replace

        def denied(source, path):
            if path == result:
                raise PermissionError('result directory is not writable')
            replace(source, path)

        for job_set in (False, True):
            with self.subTest(job_set=job_set), mock.patch.object(os, 'replace', side_effect=denied), \
                 redirect_stderr(io.StringIO()) as err:
                env = {'TEMPER_CLOSE_REFUSAL': '3'}
                if job_set:
                    env['TEMPER_JOB'] = str(self.root)
                self.assertEqual(PRE.finish('tree', 'base', [PRE.failure('format', 'diff')], self.root,
                                            env, 'close', (), ['head']), 2)
            detail = 'result directory is not writable' if job_set else 'TEMPER_JOB is missing'
            self.assertTrue(err.getvalue().endswith(
                f'smith-check: not judged\npre-pr: broke: cannot write close refusal: {detail}\n'))
            self.assertEqual(result.read_bytes(), settlement)
            self.assertEqual(sorted(path.name for path in self.root.iterdir()), ['result.json', 'target'])

    def test_close_refusal_fits_whole_ascii_escaped_names_with_its_newline(self):
        head, tree, base, counts = ('a' * 64, 'b' * 64, 'c' * 64, 'd' * 64)
        stem = (f'{{"refused":{{"head":"{head}","tree":"{tree}","base":"{base}","counts":"{counts}",'
                '"failures":[').encode('ascii')
        first = 'test A::rules rule_é'
        encoded = b'"test A::rules rule_\\u00e9"'
        suffix = b']}}\n'
        size = 512 - len(stem + encoded + b',""' + suffix)
        names = [first, 'x' * size]
        exact = stem + encoded + b',"' + b'x' * size + b'"' + suffix
        self.assertEqual(len(exact), 512)
        self.assertEqual(PRE.refusal_bytes(head, tree, base, counts, names), exact)
        errors = [PRE.failure(first, 'test failed'), PRE.failure('format', 'diff'),
                  PRE.failure('flakes are omitted', 'flake', ('A::rules', 'flake')),
                  PRE.failure('inherited are omitted', 'known'), PRE.broken('helper', 'also broke')]
        errors[2]['flake'] = (1, 0.0)
        errors[3]['inherited'] = True
        errors += [PRE.failure('ñ' * 100, 'too many bytes')] + [PRE.failure(f'test later {i}', 'failed') for i in range(6)]
        env = {'TEMPER_CLOSE_REFUSAL': '3', 'TEMPER_JOB': str(self.root)}
        with redirect_stderr(io.StringIO()) as err:
            self.assertEqual(PRE.finish(tree, base, errors, self.root, env, 'close', (), [head], counts_ref=counts), 3)
        self.assertNotIn('smith-check: not judged', err.getvalue())
        result = (self.root / 'result.json').read_bytes()
        self.assertEqual(result, stem + encoded + b',"format"],"more":7}}\n')
        self.assertLessEqual(len(result), 512)

    def test_a_refused_close_names_its_first_failure_on_stderr(self):
        errors = [PRE.failure('budgets', 'temper default simulated: instruction growth over allowance, ' + 'x' * 200)]
        with redirect_stderr(io.StringIO()) as err:
            self.assertEqual(PRE.finish('tree', 'b' * 40, errors, self.root, {}, 'close', [], ['head']), 1)
        self.assertEqual(err.getvalue(),
                         ('budgets: temper default simulated: instruction growth over allowance, ' + 'x' * 200)[:127] + '\n')
        with redirect_stderr(io.StringIO()) as err:
            PRE.finish('tree', 'b' * 40, errors, self.root, {}, 'delivery', [], ['head'])
        self.assertEqual(err.getvalue(), '')

    def test_a_refused_close_names_the_last_lines_of_a_cut_log(self):
        errors = [PRE.failure('focused tests', 'x' * 3000 + '\nFAIL 0.1s leaf::suite a_case\n'
                              'warning: default leaf::suite b_case: unobserved, the run stopped after a failure\n')]
        with redirect_stderr(io.StringIO()) as err:
            PRE.finish('tree', 'b' * 40, errors, self.root, {}, 'close', [], ['head'])
        self.assertEqual(err.getvalue(), 'focused tests: FAIL 0.1s leaf::suite a_case\n')

    def test_close_receipt_records_only_the_checked_tree(self):
        self.assertEqual(PRE.finish('staged', None, [], self.root, {}, 'close', ['dest'], ['head']), 0)
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual((receipt['goal_head'], receipt['requested_destination'], receipt['tree']),
                         ('head', 'dest', 'staged'))
        self.assertEqual((receipt['comparison'], receipt['base'], receipt['destination']), ('tree-only', None, None))

    def test_known_outward_failure_requires_a_complete_scan(self):
        key = ('test', 'fabrig-boundary::outward', 'no_file_names_the_lab')
        with mock.patch.object(PRE, 'outward', return_value=(0, [])) as outward, \
             mock.patch.object(PRE, 'citations', return_value=(0, [])):
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
        self.assertEqual(receipt['counts'], {'new': 1, 'flake': 0, 'inherited': 0, 'broke': 0})

    def test_close_one_process_assertion_survives_the_generic_summary(self):
        case = 'registered_value_type_numbers_are_pinned'
        site = 'jig/crates/jig-protocol-accounts/src/tests/value_types.rs:98:9'
        output = (f"thread 'tests::value_types::{case}' (16248) panicked at {site}:\n"
                  "assertion `left == right` failed: jig-store-core\n  left: 9\n right: 8\n")
        for outcome in ('failed', 'crashed'):
            with self.subTest(outcome=outcome):
                (self.root / 'results.json').write_text(json.dumps({'tests': [
                    dict(binary_id='api::suite', test_name='tests::earlier', outcome='passed', output=None),
                    dict(binary_id='api::suite', test_name=f'tests::value_types::{case}', outcome=outcome, output=output),
                    dict(binary_id='api::suite', test_name='tests::later', outcome='failed', output=None),
                ]}))
                log = PRE.tail('other output\n' * 500 + 'error: test run failed\n')
                self.assertNotIn(case, log)
                errors = PRE.close_test_errors(self.root, 'focused tests', 100, log, '')
                with redirect_stderr(io.StringIO()):
                    self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'close', ['main'], ['head']), 1)
                expected = f'{case} {site}: jig-store-core: 9 != 8\n'.encode()
                self.assertEqual(self.log.getvalue().encode()[-128:], expected)

    def test_failed_case_without_a_panic_keeps_its_name(self):
        (self.root / 'results.json').write_text(json.dumps({'tests': [
            dict(binary_id='api::suite', test_name='tests::first', outcome='crashed', output='aborted'),
            dict(binary_id='api::suite', test_name='tests::second', outcome='failed',
                 output="thread 'tests::second' panicked at second.rs:8:1:\nother panic"),
        ]}))
        first, _ = PRE.close_test_errors(self.root, 'focused tests', 100, '', '')
        self.assertEqual(first['diagnostic'], 'failed test tests::first\n')

    def test_large_receipt_preserves_every_key_and_bounds_summary(self):
        errors = [PRE.failure(f'test api::suite case_{i}', 'x' * 9000, ('api::suite', f'case_{i}'))
                  for i in range(450)]
        PRE.inherited({('test', 'api::suite', 'case_0')}, errors)
        errors[1]['flake'] = (1, 0.1)
        self.assertEqual(PRE.finish('tree', 'base', errors, self.root, {}, 'goal', ['head', 'dest'], ['head']), 1)
        receipt = json.loads((self.root / 'target/temper/pre-pr-failures.json').read_text())
        self.assertEqual(receipt['counts'], {'new': 448, 'flake': 1, 'inherited': 1, 'broke': 0})
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
        self.git('init', '-q')
        for name in ('kept.rs', 'edited.rs', 'unstaged.md', 'gone.md'):
            (self.root / name).write_text(name + '\n')
        (self.root / '.gitignore').write_text('/target/\n')
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

    def commit_journeys(self, *, recording=False):
        page = self.root / PRE.JOURNEY_ROOTS[0] / 'x' / '01.html'
        page.parent.mkdir(parents=True)
        page.write_text('page\n')
        page.with_name('walk').write_text('walk\n')
        if recording:
            page.with_name('recording').write_text('old recording\n')
        (self.root / '.gitignore').write_text('/target/\n')
        self.git('add', '-A')
        self.git('commit', '-q', '--amend', '--no-edit')
        return page

    def check_journeys(self, args):
        with mock.patch.object(Path, 'cwd', return_value=self.root), mock.patch.dict(os.environ, self.env), \
             mock.patch.object(PRE, 'workspace_checks', return_value=[]) as checks, \
             mock.patch.object(PRE, 'goal_checks', return_value=([], [], [])), redirect_stdout(io.StringIO()) as output:
            status = PRE.main(args)
        return status, output.getvalue(), checks.call_args.args[1]

    def test_no_base_refuses_unstaged_page_with_untracked_notes(self):
        page = self.commit_journeys()
        page.write_text('edited page\n')
        (self.root / 'notes.txt').write_text('notes\n')
        status, output, _ = self.check_journeys(['--goal'])
        self.assertEqual(status, 1)
        self.assertIn(str(page.relative_to(self.root)), output)
        self.assertIn('other path: notes.txt', output)
        self.assertIn('git checkout HEAD -- PATH...', output)

    def test_no_base_passes_a_staged_page_alone(self):
        page = self.commit_journeys()
        page.write_text('edited page\n')
        self.git('add', str(page.relative_to(self.root)))
        status, output, _ = self.check_journeys(['--goal'])
        self.assertEqual(status, 0)
        self.assertIn('pre-pr: journeys: passed,', output)

    def test_no_base_passes_a_clean_tree(self):
        self.commit_journeys()
        status, output, _ = self.check_journeys(['--goal'])
        self.assertEqual(status, 0)
        self.assertIn('pre-pr: journeys: passed,', output)

    def merge_journey_edit(self, *, pending):
        page = self.commit_journeys(recording=True)
        recording = page.with_name('recording')
        base = self.git('rev-parse', 'HEAD')
        self.git('update-ref', 'refs/temper/landing', base)
        self.git('checkout', '-qb', 'writer')
        page.write_text('printed by writer\n')
        recording.unlink()
        self.git('commit', '-qam', 'printed page')
        self.git('checkout', '-qb', 'delivery', base)
        source = self.root / 'skein/a.rs'
        source.parent.mkdir()
        source.write_text('source\n')
        self.git('add', 'skein/a.rs')
        self.git('commit', '-qm', 'delivery edit')
        self.git('merge', '-q', '--no-edit', '--no-commit' if pending else '--commit', 'writer')
        return page, base

    def test_page_from_merge_tip_passes_alongside_source_edit(self):
        page, _ = self.merge_journey_edit(pending=True)
        for pending in (True, False):
            with self.subTest(pending=pending):
                if not pending:
                    self.git('commit', '-qm', 'merged writer')
                status, output, selected = self.check_journeys([])
                self.assertEqual(status, 0)
                self.assertIn('pre-pr: journeys: passed,', output)
                self.assertEqual(selected, ['skein/a.rs', str(page.relative_to(self.root)),
                                            str(page.with_name('recording').relative_to(self.root))])

    def test_page_edited_after_merge_is_refused_alongside_source(self):
        page, base = self.merge_journey_edit(pending=False)
        page.write_text('changed after merge\n')
        status, output, _ = self.check_journeys([])
        self.assertEqual(status, 1)
        self.assertIn(f'written files:\n{page.relative_to(self.root)}\nother path: skein/a.rs', output)
        self.assertIn(f'git checkout {base} -- PATH...', output)

    def test_page_mode_change_after_merge_counts_as_an_edit(self):
        page, _ = self.merge_journey_edit(pending=False)
        page.chmod(page.stat().st_mode | 0o111)
        name = str(page.relative_to(self.root))
        self.git('update-index', '--chmod=+x', name)
        status, output, _ = self.check_journeys([])
        self.assertEqual(status, 1)
        self.assertIn(f'written files:\n{name}\nother path: skein/a.rs', output)

    def publish(self, commits, **extra):
        data = json.dumps(dict(version=1, commits=commits, profiles={}, **extra))
        blob = subprocess.run(['git', 'hash-object', '-w', '--stdin'], cwd=self.root, env=self.env, input=data,
                              capture_output=True, text=True, check=True).stdout.strip()
        tree = subprocess.run(['git', 'mktree'], cwd=self.root, env=self.env, input=f'100644 blob {blob}\tcounts.json\n',
                              capture_output=True, text=True, check=True).stdout.strip()
        self.git('update-ref', 'refs/temper/counts', self.git('commit-tree', tree, '-m', 'counts'))

    def test_close_starts_from_mains_newest_published_commit_in_the_goal(self):
        later = self.git('commit-tree', 'HEAD^{tree}', '-p', self.main, '-m', 'main moved on')
        self.change()
        self.git('update-ref', 'refs/remotes/origin/main', later)
        self.git('update-ref', 'refs/temper/landing', 'HEAD')
        self.publish(['0' * 40, self.main, later])
        published, _ = PRE.close_counts(self.root, self.env)
        self.assertEqual(PRE.scope_base(self.root, self.env, 'close', True, published),
                         (self.main, "main's newest published commit in HEAD", None))

    def close_checkout(self):
        self.git('update-ref', 'refs/remotes/origin/main', self.main)
        (self.root / 'unstaged.md').write_text('goal edit\n')
        self.git('commit', '-qam', 'goal edit')
        (self.root / 'unstaged.md').write_text('settlement edit\n')
        self.git('add', 'unstaged.md')
        return self.git('rev-parse', 'HEAD'), self.git('write-tree')

    def assert_close_breaks_before_checks(self, why):
        job = self.root.parent / 'job'
        job.mkdir()
        result = job / 'result.json'
        settlement = b'{"settled":["unstaged.md"]}\n'
        result.write_bytes(settlement)
        env = {**self.env, 'TEMPER_JOB': str(job)}
        env.pop('TEMPER_CLOSE_REFUSAL', None)
        for opt_in, status in (({}, 1), ({'TEMPER_CLOSE_REFUSAL': '3'}, 2)):
            with self.subTest(opt_in=opt_in), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.dict(os.environ, {**env, **opt_in}, clear=True), \
                 mock.patch.object(PRE, 'workspace_checks') as checks, mock.patch.object(PRE, 'run') as run, \
                 redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()) as err:
                self.assertEqual(PRE.main(['--close']), status)
                checks.assert_not_called()
                run.assert_not_called()
                self.assertTrue(err.getvalue().startswith('smith-check: not judged\npre-pr: broke: '), err.getvalue())
                self.assertIn(why, err.getvalue())
                self.assertEqual(result.read_bytes(), settlement)
                receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_bytes())
                self.assertEqual(receipt['verdict'], 'broke')
                self.assertEqual(receipt['counts']['new'], 0)
                self.assertEqual(receipt['counts']['broke'], 1)
                entry, = receipt['failures']
                self.assertEqual(entry['class'], 'broke')
                self.assertIn(why, entry['detail'])

    def test_close_breaks_when_public_counts_fetch_fails_even_with_origin_main(self):
        self.close_checkout()
        run = subprocess.run

        def fail_fetch(args, **kwargs):
            if args[:2] == ['git', 'fetch']:
                raise subprocess.CalledProcessError(128, args)
            return run(args, **kwargs)

        with mock.patch.object(PRE, 'PUBLIC_SOURCE', 'fixture'), \
             mock.patch.object(subprocess, 'run', side_effect=fail_fetch) as commands:
            self.assert_close_breaks_before_checks('public fetch failed')
        fetches = [call for call in commands.call_args_list if call.args[0][:2] == ['git', 'fetch']]
        self.assertEqual(len(fetches), 2)  # One for each close attempt, including the retry.

    def test_close_breaks_when_published_counts_are_marked_unavailable(self):
        self.close_checkout()
        self.publish([self.main], unavailable='the ledger has not been published yet')
        self.assert_close_breaks_before_checks('the ledger has not been published yet')

    def test_close_breaks_when_head_is_outside_the_published_window(self):
        self.close_checkout()
        later = self.git('commit-tree', 'HEAD^{tree}', '-p', self.main, '-m', 'main moved on')
        self.git('update-ref', 'refs/remotes/origin/main', later)
        self.publish([later])
        self.assert_close_breaks_before_checks('update from main and close again')

    def test_typed_close_binds_the_refusal_to_the_checked_tree_and_local_counts(self):
        head, tree = self.close_checkout()
        self.assertNotEqual(tree, self.git('rev-parse', 'HEAD^{tree}'))
        job = self.root.parent / 'job'
        job.mkdir()
        result = job / 'result.json'
        check_file = self.root / 'target/temper/goal-close-pre-pr.json'
        replace = PRE.replace_file
        output = io.StringIO()

        def refuse(root, paths, tree_id, env, errors, outcomes, **options):
            self.assertEqual((root, paths, tree_id, options['base_id'], options['published']['base']),
                             (self.root, ['unstaged.md'], tree, self.main, self.main))
            if publish:
                # The refusal names the counts read, not the ref's tip after the check.
                self.git('update-ref', 'refs/temper/counts', head)
            errors.append(PRE.failure('test A::rules rule_00', 'failure', ('A::rules', 'rule_00')))
            return ['A']

        def after_report(path, data):
            if path == result:
                self.assertEqual(json.loads(check_file.read_bytes())['verdict'], 'refused')
                self.assertIn('pre-pr: refused (close,', output.getvalue())
            replace(path, data)

        for publish in (False, True):
            with self.subTest(publish=publish):
                if publish:
                    self.publish([self.main])
                counts = self.git('rev-parse', 'refs/temper/counts') if publish else None
                result.write_text('{"settled":["unstaged.md"]}\n')
                check_file.unlink(missing_ok=True)
                output.seek(0)
                output.truncate()
                with mock.patch.object(Path, 'cwd', return_value=self.root), \
                     mock.patch.dict(os.environ, {**self.env, 'TEMPER_CLOSE_REFUSAL': '3', 'TEMPER_JOB': str(job)}), \
                     mock.patch.object(PRE, 'PUBLIC_SOURCE', 'fixture'), \
                     mock.patch.object(PRE, 'fetched_counts', return_value=dict(commits=[self.main], profiles={})) as fetch, \
                     mock.patch.object(PRE, 'workspace_checks', side_effect=refuse), \
                     mock.patch.object(PRE, 'replace_file', side_effect=after_report), \
                     redirect_stdout(output), redirect_stderr(io.StringIO()):
                    self.assertEqual(PRE.main(['--close']), 3)
                self.assertEqual(fetch.call_count, int(not publish))
                receipt = json.loads(check_file.read_bytes())
                self.assertEqual((receipt['goal_head'], receipt['tree'], receipt['base']), (head, tree, self.main))
                expected = (f'{{"refused":{{"head":"{head}","tree":"{tree}","base":"{self.main}",'
                            f'"counts":{json.dumps(counts)},"failures":["test A::rules rule_00"]}}}}\n')
                self.assertEqual(result.read_bytes(), expected.encode('ascii'))

    def test_two_identical_close_refusals_both_run_without_touching_the_old_cache(self):
        self.close_checkout()
        self.publish([self.main])
        home = self.root.parent / 'home'
        store = home / '.cache/fabrig-pre-pr/close'
        store.mkdir(parents=True)
        old = store / 'old.json'
        old.write_text('old cache\n')
        os.utime(old, (0, 0))

        def refuse(root, paths, tree_id, env, errors, outcomes, **options):
            errors.append(PRE.failure('budgets', 'too much growth'))
            return []

        with mock.patch.object(Path, 'cwd', return_value=self.root), \
             mock.patch.dict(os.environ, {**self.env, 'TEMPER_CLOSE_REFUSAL': ''}), \
             mock.patch.object(PRE.pwd, 'getpwuid', return_value=mock.Mock(pw_dir=str(home))), \
             mock.patch.object(PRE, 'workspace_checks', side_effect=refuse) as checks, \
             redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(PRE.main(['--close']), 1)
            self.assertEqual(PRE.main(['--close']), 1)
        self.assertEqual(checks.call_count, 2)
        for index in (1, 2):  # Selected paths and tree.
            self.assertEqual(checks.call_args_list[0].args[index], checks.call_args_list[1].args[index])
        for key in ('base_id', 'published'):
            self.assertEqual(checks.call_args_list[0].kwargs[key], checks.call_args_list[1].kwargs[key])
        self.assertEqual(list(store.iterdir()), [old])
        self.assertEqual(old.read_text(), 'old cache\n')

    def test_a_broken_delivery_discards_stale_temper_results(self):
        self.git('update-ref', 'refs/temper/landing', self.main)
        directory = self.root / '.git/temper'
        directory.mkdir()
        (directory / 'reds-wait').touch()
        result = directory / 'check-results'
        tree = self.git('write-tree')

        for verdict, finding in [('passed', None), ('refused', PRE.failure('format', 'diff')),
                                 ('broke', PRE.broken('helper', 'not executable'))]:
            def check(root, paths, tree_id, env, errors, outcomes, **options):
                if finding is not None:
                    errors.append(finding)
                return []

            with self.subTest(verdict=verdict), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.dict(os.environ, self.env), mock.patch.object(PRE, 'workspace_checks', side_effect=check), \
                 redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                result.write_text('stale\n')
                self.assertEqual(PRE.main([]), int(finding is not None))
                if verdict == 'broke':
                    self.assertFalse(result.exists())
                else:
                    self.assertEqual(result.read_text(), f'temper-check 1\ncheck {tree} {self.main} {verdict}\n')

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

    def test_without_a_usable_base_the_check_breaks_loudly(self):
        for scope in ('delivery', 'child'):
            base, name, why = PRE.scope_base(self.root, self.env, scope, True)
            self.assertEqual((base, name), (None, None))
            self.assertIn('sharing history with HEAD', why)
            self.assertIn('no base', PRE.scope_base(self.root, self.env, scope, False)[2])
        self.git('update-ref', 'refs/remotes/origin/main', 'HEAD')
        self.assertEqual(PRE.scope_base(self.root, self.env, 'delivery', True)[:2],
                         (self.main, 'merge base of refs/remotes/origin/main and HEAD'))
        self.publish([self.main])
        for args, name, why in [([], 'pre-pr-light-failures.json', 'sharing history with HEAD'),
                                (['--close'], 'goal-close-pre-pr.json', 'is HEAD itself')]:
            if not args:
                self.git('update-ref', '-d', 'refs/remotes/origin/main')
            else:
                self.git('update-ref', 'refs/remotes/origin/main', 'HEAD')
            with self.subTest(args=args), mock.patch.object(Path, 'cwd', return_value=self.root), \
                 mock.patch.dict(os.environ, {**self.env, 'TEMPER_CLOSE_REFUSAL': ''}), \
                 mock.patch.object(PRE, 'workspace_checks') as checks, redirect_stdout(io.StringIO()), \
                 redirect_stderr(io.StringIO()) as err:
                self.assertEqual(PRE.main(args), 1)
            self.assertIn(why, err.getvalue())
            checks.assert_not_called()
            receipt = json.loads((self.root / 'target/temper' / name).read_text())
            self.assertEqual(receipt['verdict'], 'broke')
            self.assertEqual(receipt['counts']['broke'], 1)

    def test_a_lock_moving_only_workspace_edges_reaches_no_more_packages(self):
        lock = ('version = 4\n\n[[package]]\nname = "leaf"\nversion = "0.1.0"\ndependencies = [%s]\n\n'
                '[[package]]\nname = "serde"\nversion = "%s"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n')
        (self.root / 'Cargo.lock').write_text(lock % ('', '1.0.0'))
        self.git('add', 'Cargo.lock')
        self.git('commit', '-qm', 'lock')
        base = self.git('rev-parse', 'HEAD')
        (self.root / 'Cargo.lock').write_text(lock % ('"serde"', '1.0.0'))
        self.assertFalse(PRE.lock_moves_externals(self.root, self.env, base, self.root))
        (self.root / 'Cargo.lock').write_text(lock % ('"serde"', '1.0.1'))
        self.assertTrue(PRE.lock_moves_externals(self.root, self.env, base, self.root))
        (self.root / 'Cargo.lock').write_text('not toml [')
        self.assertTrue(PRE.lock_moves_externals(self.root, self.env, base, self.root))

    def test_a_root_manifest_adding_a_crate_reaches_no_more_packages(self):
        manifest = ('[workspace]\nmembers = [%s]\n\n[workspace.dependencies]\nserde = "%s"\n%s\n'
                    '[profile.dev]\ndebug = false\n')
        (self.root / 'Cargo.toml').write_text(manifest % ('"a"', '1', ''))
        self.git('add', 'Cargo.toml')
        self.git('commit', '-qm', 'manifest')
        base = self.git('rev-parse', 'HEAD')
        (self.root / 'Cargo.toml').write_text(manifest % ('"a", "b"', '1', 'b = { path = "b" }'))
        self.assertFalse(PRE.root_manifest_moves_more(self.root, self.env, base, self.root))
        (self.root / 'Cargo.toml').write_text(manifest % ('"a", "b"', '2', 'b = { path = "b" }'))
        self.assertTrue(PRE.root_manifest_moves_more(self.root, self.env, base, self.root))

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
        for scope in ('delivery', 'child', 'goal'):
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
        # The allowance's own rules; the close's margin has a test of its own.
        self.enterContext(mock.patch.object(PRE, 'CLOSE_MARGIN', 0))
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

    def publish(self, commits=None, profiles=None):
        data = dict(version=1, commits=commits or [self.base, self.main], profiles=profiles or {'default': self.profile})
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

    def test_no_counts_breaks_budget_evaluation(self):
        error, = self.verdict(self.result(self.process(50 * self.G)))
        self.assertIn('cannot read refs/temper/counts', error['broke'])
        self.assertEqual(self.log.getvalue(), '')

    def test_a_budget_refusal_names_its_figures_in_the_hold_and_the_receipt(self):
        self.publish()
        error, = self.verdict(self.result(self.process(30 * self.G)))
        self.assertEqual(error['diagnostic'], 'budgets skein default step: +5.00%, allowance 2.00%, 100.0e9 to '
                                              '105.0e9, most leaf::suite\n')
        figure, = error['figures']
        self.assertEqual((figure['tree'], figure['layer'], figure['base'], figure['head'], figure['allowance']),
                         ('skein', 'step', 100 * self.G, 105 * self.G, 2 * self.G))
        self.assertEqual(figure['binaries'], [{'binary': 'leaf::suite', 'growth': 5 * self.G}])
        with redirect_stderr(io.StringIO()) as err:
            PRE.finish('tree', 'b' * 40, [error], self.root, {}, 'close', [], ['head'])
        self.assertEqual(err.getvalue(), error['diagnostic'])
        receipt = json.loads((self.root / 'target/temper/goal-close-pre-pr.json').read_text())
        self.assertEqual(receipt['verdict'], 'refused')
        entry, = receipt['failures']
        self.assertEqual(entry['figures'], [figure])
        self.assertIn('instruction growth over allowance', entry['detail'])

    def test_the_fuzzy_profile_is_judged_by_its_own_counts(self):
        self.publish(profiles={'default': self.profile, 'fuzzy': json.loads(json.dumps(self.profile))})
        focused, fuzzy = self.result(self.process(25 * self.G)), self.result(self.process(30 * self.G))
        error, = PRE.close_budgets(self.root, self.repository, self.env, {'default': [focused], 'fuzzy': [fuzzy]})
        self.assertIn('skein fuzzy step: instruction growth over allowance', error['log'])
        self.assertNotIn('skein default step', error['log'])
        self.assertTrue(error['diagnostic'].startswith('budgets skein fuzzy step: +5.00%'))
        self.assertEqual(error['figures'][0]['profile'], 'fuzzy')

    def test_a_profile_main_never_counted_gives_no_verdict_of_its_own(self):
        self.publish()
        self.assertEqual(PRE.close_budgets(self.root, self.repository, self.env,
                                           {'default': [self.result(self.process(25 * self.G))],
                                            'fuzzy': [self.result(self.process(30 * self.G))]}), [])
        self.assertIn("budgets: no fuzzy counts in main's published counts; no fuzzy verdict at close",
                      self.log.getvalue())

    def test_growth_within_the_margin_warns_and_past_it_refuses(self):
        self.publish()
        self.enterContext(mock.patch.object(PRE, 'CLOSE_MARGIN', 0.03))
        # leaf::suite's base is 25e9 of a 100e9 layer; the allowance is 2e9 and the margin 3e9.
        self.assertEqual(self.verdict(self.result(self.process(29 * self.G))), [])
        self.assertIn('by less than its 3% margin, a warning', self.log.getvalue())
        error, = self.verdict(self.result(self.process(31 * self.G)))
        self.assertIn('instruction growth over allowance', error['log'])

    def test_counts_missing_here_are_fetched_from_main(self):
        self.publish()
        public = self.root.parent / 'public.git'
        self.git('init', '-q', '--bare', str(public))
        self.git('push', '-q', str(public), 'refs/temper/counts:refs/temper/counts')
        self.git('update-ref', '-d', 'refs/temper/counts')
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_SOURCE', 'file://' + str(public)))
        self.enterContext(mock.patch.object(PRE, 'PUBLIC_PROTOCOLS', 'file'))
        self.enterContext(mock.patch.object(subprocess, 'Popen', side_effect=POPEN))
        data, source = PRE.close_counts(self.repository, self.env)
        self.assertEqual(data['base'], self.base)
        self.assertIsNone(source)
        self.assertIsNone(module(HERE.parent / 'tiers/counts.py').read(self.repository, self.env))
        with mock.patch.object(subprocess, 'Popen', side_effect=AssertionError('counts were selected again')):
            runs = [self.result(self.process(30 * self.G))]
            error, = PRE.close_budgets(self.root, self.repository, self.env, runs, published=data)
        self.assertIn('instruction growth over allowance', error['log'])

    def test_selected_baseline_can_be_used_without_git(self):
        self.publish()
        packet, _ = PRE.close_counts(self.repository, self.env)
        path = self.root / 'worker-counts.json'
        path.write_text(json.dumps(packet))
        self.enterContext(mock.patch.object(PRE, 'WORKER_COUNTS', path))
        with mock.patch.object(subprocess, 'Popen', side_effect=AssertionError('sandbox accessed Git history')):
            error, = self.verdict(self.result(self.process(30 * self.G)))
            self.assertIn('instruction growth over allowance', error['log'])

    def test_tree_without_head_has_no_budget_baseline(self):
        self.publish()
        self.git('symbolic-ref', 'HEAD', 'refs/heads/unborn')
        error, = self.verdict(self.result(self.process(30 * self.G)))
        self.assertIn('source checkout has no commit', error['broke'])

    def test_stale_base_breaks_budget_evaluation(self):
        self.publish([self.main])
        error, = self.verdict(self.result(self.process(25 * self.G)))
        self.assertIn("older than main's last twenty landings; update from main and close again", error['broke'])

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
             mock.patch.object(PRE, 'citations', return_value=(0, [])), \
             mock.patch.object(PRE, 'outward', return_value=(0, [])):
            PRE.workspace_checks(self.root, [str(source.relative_to(self.root))], 'tree', self.env, errors, {},
                                 completed_goal=completed_goal, counts_source=(self.repository, self.env))
        if completed_goal:
            # The focused run, then the fuzzy one unless the focused run failed.
            commands = [call.args[0] for call in run.call_args_list]
            self.assertIn(len(commands), (1, 2))
            for step, (command, profile) in enumerate(zip(commands, ('default', 'fuzzy'))):
                self.assertEqual(command[:2], [str(PRE.TIERS / 'one-process'), '--out'])
                self.assertEqual(Path(command[2]).name, str(step))
                self.assertEqual(Path(command[2]).parent.parent, target)
                self.assertFalse(Path(command[2]).parent.exists())
                self.assertEqual(command[3:], ['--fail-fast', profile, '--offline', '--locked', '--no-tests=warn',
                                               '-p', 'leaf', '-E',
                                               'default() & (((package(=leaf)) & ((kind(test) & binary(=suite)))))'])
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

    def test_close_test_runs_without_a_verdict_break_but_listing_build_errors_refuse(self):
        cases = [
            (2, 'other listing error', None, 'broke', 'exit 2'),
            (1, 'unexpected exit', '{"tests":[]}', 'broke', 'exit 1'),
            (0, '', None, 'broke', 'results.json:'),
            (100, '', None, 'broke', 'results.json:'),
            (100, '', '{bad json}', 'broke', 'results.json:'),
            (100, '', '{"tests":[]}', 'broke', 'exit 100 without a failed or crashed test'),
            (2, 'one-process: nextest listing exited with 101\n', None, 'refused', 'nextest listing exited with 101'),
        ]
        for code, log, results, verdict, detail in cases:
            def fake(command, *_args, **_kwargs):
                directory = Path(command[2])
                if results is not None:
                    directory.mkdir()
                    (directory / 'results.json').write_text(results)
                return code, log

            with self.subTest(code=code, log=log, results=results):
                errors = self.close(fake)
                self.assertEqual(PRE.outcome(PRE.failure_groups(errors)), verdict)
                error, = errors
                self.assertIn(detail, error['broke'] if verdict == 'broke' else error['log'])

    def test_close_uses_source_ref_and_only_judges_successful_focused_runs(self):
        self.publish()

        def fake(command, *_args, **_kwargs):
            tests = [dict(binary_id=self.BINARY, test_name='example', outcome='failed')] if status == 100 else []
            self.result(self.process(30 * self.G), tests=tests, directory=Path(command[2]))
            return status, 'fixture\n'

        self.assertEqual(self.close(fake, completed_goal=False), [])
        status = 100
        error, = self.close(fake)
        self.assertEqual(error['name'], 'test leaf::suite example')
        self.assertNotIn('broke', error)
        status = 0
        error, = self.close(fake)
        self.assertEqual(error['name'], 'budgets')


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
