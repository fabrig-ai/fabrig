"""Committed-tree checks of the figures rule (tools/tiers/budgets.md, 10.1)."""

from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('figures.py').resolve()
REPAIR_LINE = runpy.run_path(str(SCRIPT.with_name('cpu.py')))['REPAIR_LINE']
BUDGETS = 'tools/tiers/budgets.toml'
LAYERS = 'tools/tiers/layers.toml'
BASE = '''[gate.default]
references = { step = 0.04, domain = 0.2, simulated = 1 }
[gate.default.temper]
ideal_s = 100
[gate.default.temper.domain]
cpu_s = 136
ideal_s = 25
[gate.default.temper.simulated]
cpu_s = 160
reference_s = 2
ideal_s = 30
[gate.fuzzy]
references = { step = 10 }
[browser-skein]
cpu_suite_s = 217.5
cpu_test_s = 10
wall_hard_s = 300
[tools."tools/tiers/test_run.py"]
cpu_suite_s = 3
cpu_test_s = 2
wall_hard_s = 30
'''
MOVE = '''\n[[gate.default.moves]]
from = "temper.simulated"
to = "temper.domain"
cpu_s = 1.5
why = "place the mechanics lower"
'''
INSTRUCTION_BASE = (BASE.replace('ideal_s = 25\n', 'ideal_s = 25\ninstructions = 9007199254740992\n'
                                 'ideal_instructions = 250000000000\n')
                    .replace('ideal_s = 30\n', 'ideal_s = 30\ninstructions = 1500000000000\n'))
INSTRUCTION_MOVE = MOVE + 'instructions = 15000000000\n'
MOVE_PLACE = "default move temper.simulated to temper.domain instructions ('place the mechanics lower')"
REASON = '''\n[[gate.default.reasons]]
binary = "temper-system::suite"
test = "suite::chat::a_whole_chat_lands"
cpu_s = 3
why = "a whole chat"
'''
REASON_PLACE = 'default reason temper-system::suite suite::chat::a_whole_chat_lands'


class Figures(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='figures-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.git('init', '-q', '-b', 'main')
        self.git('config', 'user.email', 'test@example.invalid')
        self.git('config', 'user.name', 'test')
        self.write(LAYERS, 'layers = ["step", "domain", "simulated"]\n')
        self.write('other', 'base\n')
        self.set_base(BASE)

    def git(self, *args):
        return subprocess.run(('git', *args), cwd=self.root, check=True,
                              capture_output=True, text=True).stdout.strip()

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)

    def commit(self, message):
        self.git('add', '-A')
        self.git('commit', '--allow-empty', '-qm', message)
        return self.git('rev-parse', 'HEAD')

    def set_base(self, content):
        self.write(BUDGETS, content)
        self.base = self.commit('base')

    def head(self, content, *, other=True):
        if content is None:
            (self.root / BUDGETS).unlink()
        else:
            self.write(BUDGETS, content)
        if other:
            self.write('other', f'head from {self.base}\n')
        return self.commit('head')

    def run_check(self, *args):
        return subprocess.run((sys.executable, '-I', '-B', str(SCRIPT), *args),
                              cwd=self.root, capture_output=True, text=True)

    def check(self, lines=(), *, head='HEAD'):
        result = self.run_check('--base', self.base, '--head', head)
        expected = [*lines, f'figures: {len(lines)} raised or added since {self.base}']
        if lines:
            expected.append(REPAIR_LINE)
        self.assertEqual(result.returncode, int(bool(lines)), result.stdout + result.stderr)
        self.assertEqual(result.stdout.splitlines(), expected)
        self.assertEqual(result.stderr, '')

    def test_unchanged(self):
        self.head(BASE)
        result = self.run_check('--base', self.base)
        self.assertEqual((result.returncode, result.stdout, result.stderr),
                         (0, f'figures: {BUDGETS} unchanged since {self.base}\n', ''))

    def test_budgets_alone_are_the_users_figures(self):
        self.head(INSTRUCTION_BASE.replace('cpu_s = 136', 'cpu_s = 140') + INSTRUCTION_MOVE + REASON,
                  other=False)
        result = self.run_check('--base', self.base)
        self.assertEqual((result.returncode, result.stdout, result.stderr),
                         (0, f"figures: {BUDGETS} alone changed since {self.base}: the user's figures\n", ''))

    def merge_head(self, content, resolution=None):
        self.git('checkout', '-q', '--detach', self.base)
        self.write(BUDGETS, content)
        merged = self.commit('main figures alone')
        self.git('checkout', '-q', '--detach', self.base)
        self.write('other', 'branch change\n')
        self.commit('branch')
        self.git('merge', '--no-ff', '--no-commit', merged)
        if resolution is not None:
            self.write(BUDGETS, resolution)
        return merged, self.commit('merge')

    def test_merge_passes_landed_reason_but_without_merged_still_refuses(self):
        merged, head = self.merge_head(BASE + REASON)
        result = self.run_check('--base', self.base, '--merged', merged, '--head', head)
        self.assertEqual((result.returncode, result.stdout.splitlines(), result.stderr),
                         (0, [f'figures: {BUDGETS} unchanged since {merged}',
                              f'figures: 0 raised or added since {self.base} and {merged}'], ''))
        self.check([f'figure added: {REASON_PLACE}: 3'], head=head)

    def test_merge_resolution_raise_is_named_once_by_place(self):
        merged, head = self.merge_head(BASE.replace('cpu_s = 136', 'cpu_s = 138') + REASON,
                                       BASE.replace('cpu_s = 136', 'cpu_s = 140') + REASON)
        result = self.run_check('--base', self.base, '--merged', merged, '--head', head)
        self.assertEqual((result.returncode, result.stdout.splitlines(), result.stderr),
                         (1, ['figure raised: default temper domain ceiling: 136 to 140',
                              f'figures: 1 raised or added since {self.base} and {merged}', REPAIR_LINE], ''))

    def test_merge_keeps_figures_equal_to_either_side(self):
        for merged_cpu, head_cpu in ((138, 138), (134, 136)):
            with self.subTest(merged_cpu=merged_cpu):
                self.setUp()
                merged, head = self.merge_head(BASE.replace('cpu_s = 136', f'cpu_s = {merged_cpu}'),
                                               BASE.replace('cpu_s = 136', f'cpu_s = {head_cpu}')
                                               .replace('ideal_s = 25', 'ideal_s = 24'))
                result = self.run_check('--base', self.base, '--merged', merged, '--head', head)
                self.assertEqual((result.returncode, result.stdout.splitlines(), result.stderr),
                                 (0, [f'figures: 0 raised or added since {self.base} and {merged}'], ''))

    def test_every_merged_tip_must_name_the_place_in_base_order(self):
        self.write(BUDGETS, BASE.replace('cpu_s = 136', 'cpu_s = 139') + REASON)
        second = self.commit('second figures')
        merged, head = self.merge_head(BASE.replace('cpu_s = 136', 'cpu_s = 138'),
                                       BASE.replace('cpu_s = 136', 'cpu_s = 140')
                                       .replace('reference_s = 2', 'reference_s = 3') + REASON)
        result = self.run_check('--base', self.base, '--head', head,
                                '--merged', merged, '--merged', second)
        self.assertEqual((result.returncode, result.stdout.splitlines(), result.stderr),
                         (1, ['figure raised: default temper domain ceiling: 136 to 140',
                              'figure raised: default temper simulated reference: 2 to 3',
                              f'figures: 2 raised or added since {self.base} and {merged} and {second}',
                              REPAIR_LINE], ''))

    def test_merged_budgets_alone_short_cut_names_no_places(self):
        merged = self.head(BASE)
        head = self.head(BASE.replace('cpu_s = 136', 'cpu_s = 140'), other=False)
        result = self.run_check('--base', self.base, '--head', head, '--merged', merged)
        self.assertEqual((result.returncode, result.stdout.splitlines(), result.stderr),
                         (0, [f"figures: {BUDGETS} alone changed since {merged}: the user's figures",
                              f'figures: 0 raised or added since {self.base} and {merged}'], ''))

    def test_invalid_merged_revision_is_a_git_error(self):
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 140'))
        result = self.run_check('--base', self.base, '--merged', 'missing-merged-revision')
        self.assertEqual(result.returncode, 2)
        self.assertIn('missing-merged-revision', result.stderr)
        self.assertEqual(result.stdout, '')

    def test_ceiling_with_existing_move(self):
        self.set_base(BASE + MOVE)
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 140') + MOVE)
        self.check(['figure raised: default temper domain ceiling: 137.5 to 141.5'])

    def test_lowering_and_why_edits_pass(self):
        self.set_base(BASE + MOVE + REASON)
        for content in (
                BASE.replace('cpu_s = 136', 'cpu_s = 135') + MOVE + REASON,
                BASE.replace('reference_s = 2', 'reference_s = 1') + MOVE + REASON,
                BASE.replace('ideal_s = 100', 'ideal_s = 99') + MOVE + REASON,
                BASE.replace('cpu_suite_s = 217.5', 'cpu_suite_s = 210') + MOVE + REASON,
                BASE + MOVE,
                BASE + MOVE + REASON.replace('cpu_s = 3', 'cpu_s = 2'),
                BASE + MOVE.replace('place the mechanics lower', 'the lower mechanics') + REASON,
                BASE + MOVE + REASON.replace('a whole chat', 'the chat')):
            with self.subTest(content=content):
                self.head(content)
                self.check()

    def test_moves_down_growing_and_folded_pass(self):
        self.head(BASE + MOVE)
        self.check()
        self.set_base(BASE + MOVE)
        self.head(BASE + MOVE.replace('1.5', '2.5'))
        self.check()
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 137.5').replace('cpu_s = 160', 'cpu_s = 158.5'))
        self.check()

    def test_move_removed_or_shrunk_raises_source(self):
        self.set_base(BASE + MOVE)
        for move, value in (('', 160), (MOVE.replace('1.5', '0.5'), 159.5)):
            with self.subTest(move=move):
                self.head(BASE + move)
                self.check([f'figure raised: default temper simulated ceiling: 158.5 to {value}'])

    def test_layer_instruction_figures_added(self):
        self.head(INSTRUCTION_BASE)
        self.check(['figure added: default temper domain instruction ceiling: 9007199254740992',
                    'figure added: default temper domain instruction ideal: 250000000000',
                    'figure added: default temper simulated instruction ceiling: 1500000000000'])

    def test_layer_instruction_figures_raised_or_removed(self):
        self.set_base(INSTRUCTION_BASE)
        self.head(INSTRUCTION_BASE.replace('9007199254740992', '9007199254740993')
                  .replace('250000000000', '250000000001'))
        self.check(['figure raised: default temper domain instruction ceiling: 9007199254740992 to 9007199254740993',
                    'figure raised: default temper domain instruction ideal: 250000000000 to 250000000001'])
        self.head(BASE)
        self.check(['figure raised: default temper domain instruction ceiling: 9007199254740992 to none',
                    'figure raised: default temper domain instruction ideal: 250000000000 to none',
                    'figure raised: default temper simulated instruction ceiling: 1500000000000 to none'])
        start = INSTRUCTION_BASE.index('[gate.default.temper.domain]')
        end = INSTRUCTION_BASE.index('[gate.default.temper.simulated]')
        self.head(INSTRUCTION_BASE[:start] + INSTRUCTION_BASE[end:])
        self.check(['figure raised: default temper domain instruction ceiling: 9007199254740992 to none',
                    'figure raised: default temper domain instruction ideal: 250000000000 to none'])

    def test_layer_instruction_figures_lowered(self):
        self.set_base(INSTRUCTION_BASE)
        self.head(INSTRUCTION_BASE.replace('9007199254740992', '9007199254740991')
                  .replace('250000000000', '249999999999'))
        self.check()

    def test_move_instructions_added_or_raised(self):
        self.set_base(INSTRUCTION_BASE)
        self.head(INSTRUCTION_BASE + INSTRUCTION_MOVE)
        self.check([f'figure added: {MOVE_PLACE}: 15000000000'])
        self.set_base(INSTRUCTION_BASE + MOVE)
        self.head(INSTRUCTION_BASE + INSTRUCTION_MOVE)
        self.check([f'figure added: {MOVE_PLACE}: 15000000000'])
        self.set_base(INSTRUCTION_BASE + INSTRUCTION_MOVE)
        self.head(INSTRUCTION_BASE + INSTRUCTION_MOVE.replace('15000000000', '15000000001').replace('1.5', '2.5'))
        self.check([f'figure raised: {MOVE_PLACE}: 15000000000 to 15000000001'])

    def test_move_instructions_lowered_or_removed(self):
        self.set_base(INSTRUCTION_BASE + INSTRUCTION_MOVE)
        for move in (INSTRUCTION_MOVE.replace('15000000000', '14999999999'), MOVE):
            with self.subTest(move=move):
                self.head(INSTRUCTION_BASE + move)
                self.check()

    def test_instruction_moves_with_same_endpoints_and_reason_match_unchanged_amounts(self):
        smaller = INSTRUCTION_MOVE.replace('15000000000', '10000000000')
        larger = INSTRUCTION_MOVE.replace('15000000000', '16000000000')
        self.set_base(INSTRUCTION_BASE + INSTRUCTION_MOVE + smaller)
        self.head(INSTRUCTION_BASE + smaller + INSTRUCTION_MOVE)
        self.check()
        self.head(INSTRUCTION_BASE + smaller + INSTRUCTION_MOVE + smaller)
        self.check([f'figure added: {MOVE_PLACE} #3: 10000000000'])
        self.head(INSTRUCTION_BASE + smaller + INSTRUCTION_MOVE + larger)
        self.check([f'figure added: {MOVE_PLACE}: 16000000000'])

    def test_instruction_figures_require_positive_integers(self):
        for content, key, original in (
                (INSTRUCTION_BASE, 'domain.instructions', '9007199254740992'),
                (INSTRUCTION_BASE, 'domain.ideal_instructions', '250000000000'),
                (INSTRUCTION_BASE + INSTRUCTION_MOVE, 'moves[0].instructions', '15000000000')):
            for value in ('true', '1.0', '0', '-1'):
                with self.subTest(key=key, value=value):
                    self.head(content.replace(f' = {original}\n', f' = {value}\n'))
                    result = self.run_check('--base', self.base)
                    self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                    self.assertIn(f'{key} must be a positive integer', result.stderr)
                    self.assertEqual(result.stdout, '')

    def test_only_downward_moves_are_excused_using_heads_order(self):
        self.head(BASE + MOVE)
        self.write(LAYERS, 'layers = ["step", "simulated", "domain"]\n')
        self.commit('head order')
        self.check(['figure raised: default temper domain ceiling: 136 to 137.5'])
        for source in ('skein.domain', 'repository.simulated', 'temper.unknown'):
            with self.subTest(source=source):
                self.head(BASE + MOVE.replace('temper.simulated', source))
                self.check(['figure raised: default temper domain ceiling: 136 to 137.5'])

    def test_sum_catches_a_raise_hidden_by_a_move(self):
        self.head(BASE.replace('cpu_s = 160', 'cpu_s = 161.5') + MOVE)
        self.check(["figure raised: default ceilings' sum: 296 to 297.5"])

    def test_findings_follow_design_order(self):
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 140')
                  .replace('reference_s = 2', 'reference_s = 3')
                  .replace('ideal_s = 100', 'ideal_s = 101')
                  .replace('ideal_s = 25', 'ideal_s = 26')
                  .replace('step = 10', '')
                  .replace('cpu_suite_s = 217.5', 'cpu_suite_s = 218.5')
                  .replace('wall_hard_s = 30\n', 'wall_hard_s = 31\n') + REASON)
        self.check(['figure raised: default temper domain ceiling: 136 to 140',
                    'figure raised: default temper simulated reference: 2 to 3',
                    'figure raised: default temper ideal: 100 to 101',
                    'figure raised: default temper domain ideal: 25 to 26',
                    f'figure added: {REASON_PLACE}: 3',
                    'figure raised: fuzzy references step: 10 to none',
                    'figure raised: browser-skein cpu_suite_s: 217.5 to 218.5',
                    'figure raised: tools tools/tiers/test_run.py wall_hard_s: 30 to 31'])

    def test_references_effective_override_and_missing(self):
        self.head(BASE.replace('reference_s = 2', 'reference_s = 3').replace('step = 10', ''))
        self.check(['figure raised: default temper simulated reference: 2 to 3',
                    'figure raised: fuzzy references step: 10 to none'])
        self.head(BASE.replace('reference_s = 2\n', '').replace('simulated = 1', 'simulated = 3'))
        self.check(['figure raised: default references simulated: 1 to 3',
                    'figure raised: default temper simulated reference: 2 to 3'])

    def test_ideals_raised_or_dropped(self):
        for content, group, layer in (
                (BASE.replace('ideal_s = 100', 'ideal_s = 101').replace('ideal_s = 25', 'ideal_s = 26'), '101', '26'),
                (BASE.replace('ideal_s = 100\n', '').replace('ideal_s = 25\n', ''), 'none', 'none')):
            with self.subTest(group=group):
                self.head(content)
                self.check([f'figure raised: default temper ideal: 100 to {group}',
                            f'figure raised: default temper domain ideal: 25 to {layer}'])

    def test_reasons_added_raised_and_duplicate_maximum(self):
        self.head(BASE + REASON)
        self.check([f'figure added: {REASON_PLACE}: 3'])
        self.set_base(BASE + REASON + REASON.replace('cpu_s = 3', 'cpu_s = 2'))
        self.head(BASE + REASON.replace('cpu_s = 3', 'cpu_s = 4') + REASON)
        self.check([f'figure raised: {REASON_PLACE}: 3 to 4'])

    def test_suite_limits_raised_or_dropped(self):
        self.head(BASE.replace('cpu_suite_s = 217.5', 'cpu_suite_s = 218.5')
                  .replace('cpu_test_s = 10', 'cpu_test_s = 11')
                  .replace('wall_hard_s = 300', 'wall_hard_s = 301')
                  .replace('cpu_suite_s = 3', 'cpu_suite_s = 4')
                  .replace('cpu_test_s = 2', 'cpu_test_s = 3')
                  .replace('wall_hard_s = 30\n', 'wall_hard_s = 31\n'))
        self.check(['figure raised: browser-skein cpu_suite_s: 217.5 to 218.5',
                    'figure raised: browser-skein cpu_test_s: 10 to 11',
                    'figure raised: browser-skein wall_hard_s: 300 to 301',
                    'figure raised: tools tools/tiers/test_run.py cpu_suite_s: 3 to 4',
                    'figure raised: tools tools/tiers/test_run.py cpu_test_s: 2 to 3',
                    'figure raised: tools tools/tiers/test_run.py wall_hard_s: 30 to 31'])
        self.head(BASE.replace('cpu_suite_s = 217.5\n', ''))
        self.check(['figure raised: browser-skein cpu_suite_s: 217.5 to none'])

    def test_tables_dropped(self):
        self.head(BASE.replace('[gate.default.temper.domain]\ncpu_s = 136\nideal_s = 25\n', ''))
        self.check()
        start, end = BASE.index('[gate.default.temper]'), BASE.index('[gate.fuzzy]')
        self.head(BASE[:start] + BASE[end:])
        self.check(['figure raised: default temper ideal: 100 to none'])

    def test_missing_side_is_empty(self):
        self.head(None)
        self.check(['figure raised: default references domain: 0.2 to none',
                    'figure raised: default references simulated: 1 to none',
                    'figure raised: default references step: 0.04 to none',
                    'figure raised: default temper ideal: 100 to none',
                    'figure raised: fuzzy references step: 10 to none',
                    'figure raised: browser-skein cpu_suite_s: 217.5 to none',
                    'figure raised: browser-skein cpu_test_s: 10 to none',
                    'figure raised: browser-skein wall_hard_s: 300 to none',
                    'figure raised: tools tools/tiers/test_run.py cpu_suite_s: 3 to none',
                    'figure raised: tools tools/tiers/test_run.py cpu_test_s: 2 to none',
                    'figure raised: tools tools/tiers/test_run.py wall_hard_s: 30 to none'])
        self.base = self.git('rev-parse', 'HEAD')
        self.head(BASE)
        self.check(['figure raised: default temper domain ceiling: none to 136',
                    'figure raised: default temper simulated ceiling: none to 160'])

    def test_head_tree_and_committed_content_only(self):
        self.write(BUDGETS, BASE.replace('cpu_s = 136', 'cpu_s = 140'))
        self.write('other', f'head from {self.base}\n')
        self.git('add', '-A')
        tree = self.git('write-tree')
        self.write(BUDGETS, BASE)
        self.check(['figure raised: default temper domain ceiling: 136 to 140'], head=tree)

    def test_new_moves_are_a_multiset_of_whole_moves(self):
        self.set_base(BASE + MOVE)
        self.head(BASE + MOVE + MOVE)
        self.check()
        self.head(BASE + MOVE.replace('place the mechanics lower', 'new wording'))
        self.check()
        # One old move remains; only the extra move can excuse new room.
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 137') + MOVE + MOVE)
        self.check(['figure raised: default temper domain ceiling: 137.5 to 138.5'])

    def test_comparisons_are_to_the_millisecond(self):
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 136.0004').replace('reference_s = 2', 'reference_s = 2.0004'))
        self.check()
        self.head(BASE.replace('cpu_s = 136', 'cpu_s = 136.0006').replace('reference_s = 2', 'reference_s = 2.0006'))
        self.check(['figure raised: default temper domain ceiling: 136 to 136.0006',
                    'figure raised: default temper simulated reference: 2 to 2.0006'])

    def test_usage_git_and_toml_errors(self):
        for args in ((), ('--unknown',), ('--base',), ('--base', 'missing-revision')):
            with self.subTest(args=args):
                result = self.run_check(*args)
                self.assertEqual(result.returncode, 2)
                self.assertTrue(result.stderr)
                self.assertEqual(result.stdout, '')
        self.head('not toml')
        result = self.run_check('--base', self.base)
        self.assertEqual(result.returncode, 2)
        self.assertIn(f'HEAD:{BUDGETS}', result.stderr)
        self.assertEqual(result.stdout, '')

    def test_non_numbers_name_the_side_and_place(self):
        for value in ('"3"', 'true', 'nan', 'inf', '{ bad = 3 }'):
            for figure in ('cpu_s = 136', 'reference_s = 2', 'ideal_s = 25', 'step = 10',
                           'cpu_suite_s = 217.5', 'wall_hard_s = 30\n'):
                with self.subTest(value=value, figure=figure):
                    key = figure.split(' = ')[0]
                    self.head(BASE.replace(figure, f'{key} = {value}' + ('\n' if figure.endswith('\n') else '')))
                    result = self.run_check('--base', self.base)
                    self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                    self.assertIn('HEAD ', result.stderr)
                    self.assertIn(key, result.stderr)
                    self.assertEqual(result.stdout, '')
        self.set_base(BASE.replace('cpu_s = 136', 'cpu_s = false'))
        self.head(BASE)
        result = self.run_check('--base', self.base)
        self.assertEqual(result.returncode, 2)
        self.assertIn(self.base + ' gate.default.temper.domain.cpu_s', result.stderr)


if __name__ == '__main__':
    unittest.main()
