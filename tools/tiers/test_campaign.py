#!/usr/bin/env python3
"""Offline checks for the fresh-seed command (tools/tiers/campaign.md, 9)."""

import json
import os
from pathlib import Path
import runpy
import shutil
import signal
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET


TOOLS = Path(__file__).resolve().parent
CAMPAIGN = TOOLS / 'campaign'
FIXTURES = TOOLS / 'fixtures/campaign'
CPU = runpy.run_path(str(TOOLS / 'cpu.py'))
PAIR = 'campaign-temper'
BINARY = 'temper-fixture::suite'
CAMPAIGN_SEED = 'SKEIN_TEST_FUZZY_CAMPAIGN'
CASE_SEED = 'SKEIN_TEST_FUZZY_CASE'
PINNED = ['08b4fda8c892b50e', 'd7cc9674ff5ffa39', 'c810565020d09481']

FAKE = r'''#!/usr/bin/env python3
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

name = Path(sys.argv[0]).name
args = sys.argv[1:]
call = dict(name=name, args=args, cwd=str(Path.cwd()), env={key: os.environ.get(key) for key in (
    'SKEIN_TEST_FUZZY_CAMPAIGN', 'SKEIN_TEST_FUZZY_CASE', 'CALLER_VAR', 'NEXTEST_DOUBLE_SPAWN',
    'CARGO_TARGET_DIR')})
root = Path(os.environ['FIXTURE_ROOT'])
if name == 'run':
    out = Path(args[1])
    report = json.loads((out.parent / 'campaign.json').read_text())
    call['rounds'] = [r['round'] for r in report['rounds']]
    call['replays'] = [c['replay']['verdict'] for r in report['rounds'] for c in r['cases']]
    summary = (out.parent / 'summary.txt').read_text()
    call['summary'] = next(line for line in summary.splitlines() if line.startswith('Rounds: '))
    if out.name.startswith('last-'):
        call['head'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
        call['findings'] = [(f['last'], f['body']) for f in report['findings']]
with (root / 'calls.jsonl').open('a') as log:
    print(json.dumps(call), file=log)

if name == 'df':
    print('Avail\n80000000000')
elif name == 'cargo' and args[0] == 'metadata':
    print(json.dumps({'target_directory': str(Path('relocated artifacts').resolve())}))
elif name == 'wrapper' and args[:3] == ['cargo', 'nextest', 'list']:
    if os.environ.get('LIST_FAIL'):
        for n in range(25):
            print(f'listing {n}')
        sys.exit(17)
    print('wrapper: before JSON')
    print('cargo: listing on stderr', file=sys.stderr, flush=True)
    print(Path('listing.json').read_text())
    print('wrapper: after JSON')
elif name == 'run':
    if os.environ.get('BLOCK_WORKTREE') and out.name == 'round-1':
        (out.parent / 'last-worktree').write_text('not a worktree')
    if os.environ.get('INTERRUPT') and out.name == os.environ.get('INTERRUPT_STEP', 'round-2'):
        def stop(signum, frame):
            print(f'run got {signal.Signals(signum).name}', flush=True)
            sys.exit(23)

        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            signal.signal(sig, stop)
        os.kill(os.getppid(), int(os.environ['INTERRUPT']))
        time.sleep(2)
        sys.exit(91)
    recorded = root / 'reports' / out.name
    if (recorded / 'output').is_file():
        print((recorded / 'output').read_text(), end='', flush=True)
    print(f'run: {out.name}', flush=True)
    if (recorded / 'run').is_dir():
        shutil.copytree(recorded / 'run', out)
    sys.exit(int((recorded / 'status').read_text()))
else:
    sys.exit(f'unexpected command: {name} {args}')
'''


def testcase(name, *, binary=BINARY, output='', count=None, failure=None, message='', time='0.125'):
    case = ET.Element('testcase', name=name, classname=binary, time=time)
    if failure == 'error':
        ET.SubElement(case, 'error', type='test abort', message=message)
    elif failure:
        ET.SubElement(case, 'failure', type=failure, message=message)
    if count is not None:
        output += f'\nfuzzy cases: fixture::{name} {count}\n'
    if output:
        ET.SubElement(case, 'system-err').text = output
    return case


def failed_case(name, seed, words, *, count=None, output=''):
    return testcase(name, output=f'{output}\nfuzzy case: fixture::{name} seed {seed}: {words}\n',
                    count=count, failure='test failure')


def input_line(name, seed, digest):
    return f'fuzzy input: fixture::{name} seed {seed} {digest}\n' if digest is not None else ''


class CampaignTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory(prefix='campaign-test-', dir=TOOLS.parents[1])
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.out = self.root / 'campaign output'
        self.env = dict(os.environ, PATH=f"{self.root / 'bin'}:{os.environ['PATH']}",
                        TIERS_WRAPPER='wrapper', TIERS_DISK_BYTES='80000000000',
                        CALLER_VAR='left alone', NEXTEST_DOUBLE_SPAWN='1', FIXTURE_ROOT=str(self.root),
                        CARGO_TARGET_DIR=str(self.root / 'caller artifacts'),
                        **{CAMPAIGN_SEED: 'caller campaign', CASE_SEED: 'caller case'})
        for key in ('LIST_FAIL', 'INTERRUPT', 'INTERRUPT_STEP', 'BLOCK_WORKTREE'):
            self.env.pop(key, None)
        self.calls = self.root / 'calls.jsonl'
        self.git('init', '-q')
        self.file('.config/nextest.toml', '[profile.campaign-temper]\n')
        self.file('tools/tiers/packages.toml', 'campaign-temper = ["temper-fixture"]\n\n[binaries]\ncampaign-temper = []\n')
        self.file('tools/tiers/layers.toml', 'layers = ["domain"]\n\n[temper]\n"temper-fixture::suite" = "domain"\n')
        self.file('tools/tiers/budgets.toml', '''[gate.fuzzy.references]
domain = 0.1
[gate.fuzzy.temper]
ideal_s = 0.5
[gate.fuzzy.temper.domain]
cpu_s = 0.6
[campaign-temper]
auto_reap = ["fuzzy::numbered"]
''')
        for name in ('bin/cargo', 'bin/df', 'bin/wrapper', 'tools/tiers/run'):
            self.file(name, FAKE).chmod(0o700)
        self.file('tracked.txt', 'last\n')
        self.last = self.commit('Last')
        self.file('tracked.txt', 'landed\n')
        self.head = self.commit('Current')
        self.previous = self.file('previous campaign/campaign.json',
                                  json.dumps(dict(tree='temper', commit=self.last))).parent
        self.select('fuzzy::fixed')

    def commit(self, message):
        self.git('add', '.')
        self.git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.test', 'commit', '-qm', message)
        return self.git('rev-parse', 'HEAD').strip()

    def git(self, *args):
        return subprocess.run(['git', *args], cwd=self.root, stdin=subprocess.DEVNULL, check=True,
                              text=True, capture_output=True).stdout

    def file(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')
        return path

    def select(self, *tests):
        cases = {test: {'filter-match': {'status': 'matches'}} for test in tests}
        cases['filtered_out'] = {'filter-match': {'status': 'mismatch'}}
        suites = {BINARY: {'package-name': 'temper-fixture', 'binary-id': BINARY, 'testcases': cases}}
        self.file('listing.json', json.dumps({'rust-suites': suites}, indent=2))

    def invoke(self, *args, **env):
        self.calls.unlink(missing_ok=True)
        return subprocess.run([sys.executable, '-I', '-B', str(CAMPAIGN), *args], cwd=self.root,
                              env=dict(self.env, **env), stdin=subprocess.DEVNULL, capture_output=True,
                              text=True, timeout=10)

    def campaign(self, *args, exit=0, out=True, **env):
        result = self.invoke('temper', *(['--out', str(self.out)] if out else []), *args, **env)
        self.assertEqual(result.returncode, exit, (result.stdout, result.stderr))
        actual = self.root / result.stdout.splitlines()[0]
        if out:
            self.assertEqual(actual, self.out)
        data = json.loads((actual / 'campaign.json').read_text())
        text = (actual / 'summary.txt').read_text()
        calls = [json.loads(line) for line in self.calls.read_text().splitlines()]
        return result, data, text, calls

    def stage(self, step, status, output=''):
        self.file(f'reports/{step}/status', str(status))
        self.file(f'reports/{step}/output', output)
        return self.root / 'reports' / step / 'run'

    def round_report(self, step, *tests, status=0, junit=True, unobserved=(), timeout=False, missing=()):
        path = self.stage(step, status)
        path.mkdir(parents=True, exist_ok=True)
        (path / 'records').mkdir(exist_ok=True)
        run = dict(profile=PAIR, finished=True, starved=False, wall_s=1, returncode=status, cancel_code=None)
        (path / 'run.json').write_text(json.dumps(run))
        if timeout:
            (path / 'command-output.txt').write_text('  \033[0mCancelling due to global timeout\n')
        root = ET.Element('testsuites', name=PAIR)
        for binary in dict.fromkeys(t.attrib['classname'] for t in tests):
            suite = ET.SubElement(root, 'testsuite', name=binary)
            suite.extend(t for t in tests if t.attrib['classname'] == binary)
        if junit:
            ET.ElementTree(root).write(path / 'junit.xml', encoding='utf-8')
        for i, t in enumerate(tests, 1):
            test = t.attrib['name']
            if test in unobserved or test in missing:
                continue
            sig = 15 if t.find('error') is not None else None
            code = None if sig else (101 if t.find('failure') is not None else 0)
            record = dict(profile=PAIR, binary_id=t.attrib['classname'], test_name=test, attempt=1,
                          start_s=1, user_s=0.2, system_s=0.01, wall_s=0.125, wrapper_cpu_s=0.01,
                          complete=True, pending=False, leaked=0, interrupted=0, exit_code=code, signal=sig)
            (path / 'records' / f'{i}.json').write_text(json.dumps(record) + '\n')

    def test_refusals_do_not_list_or_run(self):
        self.round_report('round-1', testcase('fuzzy::fixed'))
        self.out.mkdir()
        (self.out / 'other').write_text('do not replace')
        refusals = [
            (('other',), 'invalid choice'), (('temper', '--rounds', '0'), 'positive integer'),
            (('temper', '--rounds', 'x'), 'invalid int'),
            (('skein',), 'no campaign pair for skein (tools/tiers/layers.md, 9.3)'),
            (('temper', '--out', str(self.out)), 'empty directory'),
        ]
        refusals.extend((('temper', '--seed', seed), 'sixteen hex digits') for seed in
                        ('0' * 15, '0' * 17, '0x' + '0' * 14, 'Y' + '0' * 15))
        for args, why in refusals:
            with self.subTest(args=args):
                result = self.invoke(*args)
                self.assertEqual(result.returncode, 2)
                self.assertIn(why, result.stderr)
                self.assertFalse(self.calls.exists())
        self.file('tracked.txt', 'changed\n')
        for staged in (False, True):
            if staged:
                self.git('add', 'tracked.txt')
            with self.subTest(staged=staged):
                result = self.invoke('temper')
                self.assertEqual(result.returncode, 2)
                self.assertIn('tracked files differ', result.stderr)
                self.assertFalse(self.calls.exists())
        self.assertEqual((self.out / 'other').read_text(), 'do not replace')

    def test_last_refusals_do_not_list_or_run(self):
        name = 'invalid last/campaign.json'
        prefix = '--last invalid last'
        report = dict(tree='temper', commit=self.last)
        refusals = [
            (None, 'holds no campaign: campaign.json is missing'),
            ('', 'holds no campaign: invalid campaign.json'),
            ('[]', 'holds no campaign: campaign.json is not a JSON object'),
            (json.dumps(dict(report, tree=None)), 'holds no campaign: campaign.json has no tree string'),
            (json.dumps(dict(report, commit='F' * 40)), 'holds no campaign: campaign.json has no commit of 40 or 64 lower-case hex digits'),
            (json.dumps(dict(report, commit='a' * 41)), 'holds no campaign: campaign.json has no commit of 40 or 64 lower-case hex digits'),
            (json.dumps(dict(report, tree='jig')), 'is a campaign of jig, not temper'),
            (json.dumps(dict(report, commit='0' * 40)), f"ran at {'0' * 40}, which this repository doesn't have"),
        ]
        for content, why in refusals:
            with self.subTest(content=content):
                if content is not None:
                    self.file(name, content)
                result = self.invoke('temper', '--last', 'invalid last', '--out', str(self.out))
                self.assertEqual(result.returncode, 2)
                self.assertIn(f'{prefix} {why}', result.stderr)
                self.assertFalse(self.calls.exists())
                self.assertFalse(self.out.exists())
        source = self.root / name
        source.unlink()
        source.mkdir()  # An existing campaign.json that cannot be read as a file.
        result = self.invoke('temper', '--last', 'invalid last', '--out', str(self.out))
        self.assertEqual(result.returncode, 2)
        self.assertIn(f'{prefix} holds no campaign: campaign.json is unreadable', result.stderr)
        self.assertFalse(self.calls.exists())
        self.assertFalse(self.out.exists())

    def test_seeded_rounds_and_listing(self):
        for r in range(1, 4):
            self.round_report(f'round-{r}', testcase('fuzzy::fixed'))
        self.out.mkdir()  # An existing, empty directory is accepted.
        result, data, _, calls = self.campaign('--rounds', '3', '--seed', '0' * 16)
        self.assertEqual([run['seed'] for run in data['rounds']], PINNED)
        self.assertEqual(data['seed'], '0' * 16)
        self.assertEqual(data['selection'], [{'binary': BINARY, 'test': 'fuzzy::fixed'}])
        self.assertIn('wrapper: before JSON\nwrapper: after JSON\n', result.stdout)
        self.assertIn('cargo: listing on stderr', result.stderr)
        self.assertNotIn('rust-suites', result.stdout)
        self.assertEqual([c['name'] for c in calls], ['df', 'wrapper', 'run', 'run', 'run'])
        self.assertEqual(calls[0]['args'], ['-B1', '--output=avail', '/'])
        self.assertEqual(calls[1]['args'], ['cargo', 'nextest', 'list', '-p', 'temper-fixture',
                                           '--profile', PAIR, '--message-format', 'json'])
        self.assertEqual(calls[1]['env']['NEXTEST_DOUBLE_SPAWN'], '0')
        for r, (seed, call) in enumerate(zip(PINNED, calls[2:]), 1):
            self.assertEqual(call['args'], ['--cpu', str(self.out / f'round-{r}'), 'temper', 'campaign'])
            self.assertEqual(call['env'], {CAMPAIGN_SEED: seed, CASE_SEED: None,
                                          'CALLER_VAR': 'left alone', 'NEXTEST_DOUBLE_SPAWN': '1',
                                          'CARGO_TARGET_DIR': self.env['CARGO_TARGET_DIR']})
            self.assertLess(result.stdout.index(f'campaign: round {r} of 3 seed {seed}'),
                            result.stdout.index(f'run: round-{r}'))

    def test_mix_matches_skein_workload_seed_zero(self):
        mix = runpy.run_path(str(CAMPAIGN))['mix']
        # streams.rs, pinned_derivation: Workload's tag and the FNV-1a hash of its empty part.
        self.assertEqual(mix(mix(0x64616f6c6b726f77) ^ 0xcbf29ce484222325), 0xdf3ba3ee200f4529)

    def test_drawn_seeds_and_default_output_directory(self):
        for r in (1, 2):
            self.round_report(f'round-{r}', testcase('fuzzy::fixed'))
        result, data, _, calls = self.campaign('--rounds', '2', out=False)
        directory = Path(result.stdout.splitlines()[0])
        self.assertEqual(directory.parent, self.root / 'relocated artifacts/tiers-campaign/temper')
        self.assertRegex(directory.name, r'^\d{8}T\d{6}\.\d{6}Z-\d+$')
        self.assertEqual(calls[0]['args'], ['metadata', '--format-version', '1', '--no-deps'])
        self.assertIsNone(data['seed'])
        seeds = [r['seed'] for r in data['rounds']]
        self.assertNotEqual(*seeds)
        for r, seed in enumerate(seeds, 1):
            self.assertRegex(seed, '^[0-9a-f]{16}$')
            self.assertLess(result.stdout.index(f'campaign: round {r} of 2 seed {seed}'),
                            result.stdout.index(f'run: round-{r}'))

    def test_recorded_report_and_replay_commands(self):
        path = self.stage('round-1', 100)
        shutil.copytree(FIXTURES / 'reading', path)
        names = ('numbered', 'panic', 'two_sweeps', 'fixed', 'setup', 'after', 'crash')
        self.select(*(f'fuzzy::{n}' for n in names))
        self.round_report('round-1-replay-1', failed_case('fuzzy::numbered', '0000000000000011',
                                                         'expected 7, saw 8', count=1), status=100)
        self.round_report('round-1-replay-2', testcase('fuzzy::panic', count=1))
        _, data, text, calls = self.campaign('--seed', 'AB' * 8, exit=1)
        self.assertEqual(set(data), set('tree pair commit seed rounds_asked selection rounds findings '
                                        'nonreplaying unreplayed reach never_fell ended last'.split()))
        self.assertIsNone(data['last'])
        self.assertEqual((data['tree'], data['pair'], data['commit'], data['seed'], data['rounds_asked'], data['ended']),
                         ('temper', PAIR, self.head, 'ab' * 8, 1, None))
        report, = data['rounds']
        self.assertEqual(set(report), set('round seed directory returncode starved tests cases failures reach '
                                          'unfinished layers warnings'.split()))
        self.assertEqual((report['round'], report['directory'], report['returncode'], report['starved']),
                         (1, 'round-1', 100, False))
        self.assertEqual([(t['test'], t['outcome'], t['cases']) for t in report['tests']],
                         [(f'fuzzy::{n}', outcome, count) for n, outcome, count in zip(
                             names, ['failed', 'failed', 'passed', 'passed', 'failed', 'failed', 'failed'],
                             [3, 1, 6, None, None, 9, None])])
        self.assertEqual(report['tests'][3], dict(binary=BINARY, test='fuzzy::fixed', outcome='passed',
                                                  cases=None, time_s=1.25))
        self.assertEqual(report['failures'], [dict(binary=BINARY, test=f'fuzzy::{name}', first_line=line)
                                               for name, line in [('setup', 'fuzzy setup: no seed'),
                                                                  ('after', 'after the sweep'), ('crash', 'no message')]])
        case1, case2 = report['cases']
        for k, (case, name, seed, first) in enumerate((
                (case1, 'numbered', '0000000000000011', 'expected 3, saw 4'),
                (case2, 'panic', '0000000000000022', 'boom')), 1):
            binary, test = BINARY, f'fuzzy::{name}'
            expr = f'binary_id(={binary}) & test(={test})'
            command = f"{CASE_SEED}={seed} tools/tiers/run temper campaign -- -E '{expr}'"
            self.assertEqual(case, dict(binary=binary, test=test, seed=seed, first_line=first,
                                        identity=f'fixture::{test}', command=command,
                                        replay=dict(directory=f'round-1-replay-{k}', why=None,
                                                    verdict='failed' if k == 1 else 'passed',
                                                    first_line='expected 7, saw 8' if k == 1 else None, input=None)))
            call = calls[2 + k]
            self.assertEqual(call['args'], ['--cpu', str(self.out / f'round-1-replay-{k}'), 'temper',
                                            'campaign', '--', '-E', expr])
            self.assertEqual(call['env'], {CAMPAIGN_SEED: None, CASE_SEED: seed, 'CALLER_VAR': 'left alone',
                                           'NEXTEST_DOUBLE_SPAWN': '1',
                                            'CARGO_TARGET_DIR': self.env['CARGO_TARGET_DIR']})
            self.assertIn(command, text)
        self.assertEqual(data['nonreplaying'], [dict(case2, round=1)])
        nonreplaying, = data['nonreplaying']
        round_directory = data['rounds'][nonreplaying['round'] - 1]['directory']
        self.assertTrue((self.out / round_directory / 'junit.xml').is_file())
        self.assertTrue((self.out / nonreplaying['replay']['directory'] / 'junit.xml').is_file())
        finding, = data['findings']
        self.assertEqual(set(finding), set('binary test identity signature first_line round command count '
                                           'seeds title body last'.split()))
        self.assertIsNone(finding['last'])
        self.assertIn('Last campaign: none\n', text)
        self.assertEqual(finding['title'], 'fixture::fuzzy::numbered fails on a fresh seed: expected 3, saw 4')
        self.assertIn('kept detail', finding['body'])
        self.assertEqual(data['unreplayed'], [])
        reach = [dict(sweep='walking', kind='forge/busy', cases=8), dict(sweep='walking', kind='forge/slow', cases=0)]
        self.assertEqual(report['reach'], reach)
        self.assertEqual(data['reach'], reach)
        for part in (self.head, 'Campaign seed: ' + 'ab' * 8, 'Rounds: 1 of 1', report['seed'],
                     'fuzzy::fixed: passed, took no case (1.25 s)', 'fuzzy setup: no seed',
                     'walking forge/busy: 8', 'walking forge/slow: 0', 'Cases that did not replay:\n  From round-1:',
                     'replay round-1-replay-2: passed', 'Kinds that fell in no case:\n  walking forge/slow',
                     finding['title'], finding['body'].rstrip('\n')):
            self.assertIn(part, text)
        limits = CPU['budgets'](self.root / 'tools/tiers/budgets.toml', layers=self.root / 'tools/tiers/layers.toml')
        summary = CPU['summary'](path, limits, pair=PAIR, layers=self.root / 'tools/tiers/layers.toml')
        self.assertEqual(report['layers'], summary['layers'])
        self.assertEqual(report['warnings'], summary['warnings'])
        self.assertIn('auto-reaped', text)
        for layer in summary['layers']:
            self.assertIn(f"CPU {layer['tree']} {layer['layer']}", text)

    def test_starvation_is_green_and_does_not_finish_tests(self):
        live, cut, unobserved, unstarted = 'fuzzy::live', 'fuzzy::cut', 'fuzzy::unobserved', 'fuzzy::unstarted'
        self.select(live, cut, unobserved, unstarted)
        self.round_report('round-1', testcase(live, count=2),
                          testcase(cut, failure='error', message='process aborted with signal 15'),
                          testcase(unobserved, failure='error', message='process aborted with signal 9'),
                          timeout=True, unobserved=[unobserved])
        _, data, text, _ = self.campaign()
        report, = data['rounds']
        self.assertTrue(report['starved'])
        self.assertEqual([(t['test'], t['outcome']) for t in report['tests']],
                         [(live, 'passed'), (cut, 'unfinished'), (unobserved, 'unfinished')])
        self.assertEqual(report['unfinished'], [dict(binary=BINARY, test=name)
                                                for name in (cut, unobserved, unstarted)])
        self.assertEqual(report['failures'], [])
        self.assertIn('Unfinished: ' + BINARY + ' ' + unstarted, text)

    def test_starved_without_a_junit_report_still_ran(self):
        self.round_report('round-1', timeout=True, junit=False)
        _, data, _, _ = self.campaign()
        report, = data['rounds']
        self.assertTrue(report['starved'])
        self.assertEqual(report['unfinished'], data['selection'])
        self.assertEqual(report['tests'], [])
        self.assertIsNone(data['ended'])

    def test_failure_outside_a_case_is_red_on_its_own(self):
        self.round_report('round-1', testcase('fuzzy::fixed', failure='test failure', message='outside'),
                          status=100)
        _, data, _, calls = self.campaign(exit=1)
        report, = data['rounds']
        self.assertEqual(report['cases'], [])
        self.assertEqual(report['failures'], [dict(binary=BINARY, test='fuzzy::fixed', first_line='outside')])
        self.assertEqual([c['name'] for c in calls if c['name'] == 'run'], ['run'])

    def test_summary_refusal_does_not_discard_junit(self):
        self.round_report('round-1', testcase('fuzzy::fixed', count=3), status=2, missing=['fuzzy::fixed'])
        _, data, text, _ = self.campaign()
        report, = data['rounds']
        self.assertEqual(report['layers'], [])
        self.assertEqual(len(report['warnings']), 1)
        self.assertRegex(report['warnings'][0], r'CPU summary refused: record coverage: missing')
        test, = report['tests']
        self.assertEqual((test['outcome'], test['cases']), ('passed', 3))
        self.assertIn('CPU summary refused', text)

    def test_unreplayed_verdicts(self):
        seed = '000000000000000b'
        names = ['empty', 'empty_report', 'two_tests', 'two_parts', 'outside', 'wrong_seed', 'cut', 'no_case']
        names = [f'fuzzy::{name}' for name in names]
        self.select(*names)
        self.round_report('round-1', *(failed_case(name, seed, 'oops', count=1) for name in names), status=100)
        self.round_report('round-1-replay-1', status=4, junit=False)
        self.round_report('round-1-replay-2', status=4)
        self.round_report('round-1-replay-3', testcase(names[2], count=1), testcase('another', count=1))
        self.round_report('round-1-replay-4', testcase(names[3], output=(
            f'fuzzy cases: fixture::{names[3]}/one 1\nfuzzy cases: fixture::{names[3]}/two 1')))
        self.round_report('round-1-replay-5', testcase(names[4], failure='test failure'), status=100)
        self.round_report('round-1-replay-6', failed_case(names[5], '000000000000000c', 'oops'), status=100)
        self.round_report('round-1-replay-7', testcase(names[6], failure='test timeout'), status=100)
        self.round_report('round-1-replay-8', testcase(names[7]))
        _, data, text, _ = self.campaign(exit=1)
        self.assertEqual(data['findings'], [])
        self.assertEqual(data['nonreplaying'], [])
        self.assertEqual([case['test'] for case in data['unreplayed']], names)
        self.assertEqual([case['replay']['verdict'] for case in data['unreplayed']], ['unreplayed'] * 8)
        reasons = [case['replay']['why'] for case in data['unreplayed']]
        self.assertEqual(reasons, ['no test selected (exit 4)', 'no test selected (exit 4)',
                                   '2 tests reported, expected one', '2 cases ran, expected one',
                                   'failure outside a case', 'another case failed', 'test unfinished',
                                   '0 cases ran, expected one'])
        for reason in reasons:
            self.assertIn(reason, text)

    def assert_last_removed(self):
        worktree = self.out / 'last-worktree'
        self.assertFalse(worktree.exists())
        self.assertNotIn(str(worktree), self.git('worktree', 'list', '--porcelain'))

    def test_last_without_findings_does_not_add_a_worktree(self):
        self.round_report('round-1', testcase('fuzzy::fixed', count=1))
        trace = self.root / 'git-trace'
        _, data, text, calls = self.campaign('--last', self.previous.name, GIT_TRACE=str(trace))
        self.assertEqual(data['last'], dict(directory=str(self.previous), commit=self.last))
        self.assertIn(f'Last campaign: {self.last} ({self.previous})\n', text)
        self.assertEqual(data['findings'], [])
        self.assertEqual([c['args'][1] for c in calls if c['name'] == 'run'], [str(self.out / 'round-1')])
        self.assertNotIn('git worktree add', trace.read_text())
        self.assert_last_removed()

    def test_last_replays_each_first_case_after_all_rounds(self):
        name, other = 'numbered', 'alphabetically_first'
        seed, other_seed, later_seed = ('%016x' % n for n in (10, 11, 12))
        self.select(name, other)
        first = failed_case(name, seed, 'count 7')
        second = failed_case(other, other_seed, 'other failure')
        later = failed_case(name, later_seed, 'count 5')
        self.round_report('round-1', first, second, status=100)
        self.round_report('round-1-replay-1', failed_case(name, seed, 'changed on replay'), status=100)
        self.round_report('round-1-replay-2', second, status=100)
        self.round_report('round-2', later, status=100)
        self.round_report('round-2-replay-1', later, status=100)
        self.round_report('last-1', failed_case(name, seed, 'count 12'), status=100)
        self.round_report('last-2', testcase(other, count=1))
        _, data, text, calls = self.campaign('--last', self.previous.name, '--rounds', '2',
                                             '--out', self.out.name, exit=1, out=False)
        runs = [c for c in calls if c['name'] == 'run']
        self.assertEqual([Path(c['args'][1]).name for c in runs], [
            'round-1', 'round-1-replay-1', 'round-1-replay-2', 'round-2', 'round-2-replay-1', 'last-1', 'last-2'])
        worktree = self.out / 'last-worktree'
        for f, (test, first_seed, call) in enumerate(zip((name, other), (seed, other_seed), runs[-2:]), 1):
            self.assertEqual(call['args'], ['--cpu', str(self.out / f'last-{f}'), 'temper', 'campaign', '--',
                                             '-E', f'binary_id(={BINARY}) & test(={test})'])
            self.assertEqual((call['cwd'], call['head']), (str(worktree), self.last))
            self.assertEqual(call['env'], {CAMPAIGN_SEED: None, CASE_SEED: first_seed, 'CALLER_VAR': 'left alone',
                                           'NEXTEST_DOUBLE_SPAWN': '1', 'CARGO_TARGET_DIR': str(worktree / 'target')})
            self.assertEqual(call['rounds'], [1, 2])
            self.assertEqual(call['replays'], ['failed', 'failed', 'failed'])
        first, second = data['findings']
        self.assertEqual((first['test'], first['signature'], first['count']), (name, 'count N', 2))
        self.assertEqual(first['seeds'], [dict(seed=seed, round=1), dict(seed=later_seed, round=2)])
        self.assertEqual((second['test'], second['count']), (other, 1))
        place = f"At the last campaign's commit, {self.last}: "
        for last, body in runs[-2]['findings']:
            self.assertEqual(last, dict(directory=None, verdict=None, why=None, first_line=None, input=None, compared=None))
            self.assertIn(place + 'pending.\n', body)
        self.assertEqual(runs[-1]['findings'][0], [first['last'], first['body']])
        self.assertEqual(runs[-1]['findings'][1], runs[-2]['findings'][1])
        for finding, verdict, words in ((first, 'same', 'failed the same way'), (second, 'passed', 'passed')):
            self.assertEqual(finding['last']['verdict'], verdict)
            self.assertEqual(finding['last']['compared'], 'not compared')
            line = place + words + '; input not compared.'
            self.assertIn(line, finding['body'].splitlines())
            self.assertIn(line, text.splitlines())
        self.assertIsNone(data['ended'])
        self.assert_last_removed()

    def test_last_absence_other_failures_and_unobserved_runs(self):
        seed = '0000000000000011'
        rows = [
            ('build', (), dict(status=2, junit=False), 'unobserved', 'run did not run (exit 2)', None),
            ('before', (), dict(timeout=True, junit=False), 'unobserved', 'cut short by the wall', None),
            ('during', (testcase('during', failure='test timeout'),), dict(status=100), 'unobserved', 'test unfinished', None),
            ('many', (testcase('many', count=1), testcase('extra', count=1)), {}, 'unobserved', '2 tests reported, expected one', None),
            ('other', (testcase('other', binary='other', count=1),), {}, 'unobserved', 'another test ran', None),
            ('gone', (), dict(status=4, junit=False), 'absent', 'no test selected (exit 4)', None),
            ('no_seam', (testcase('no_seam', output='fuzzy cases: fixture::another 1'),), {},
             'absent', 'no case count for fixture::no_seam', None),
            ('differ', (failed_case('differ', seed, 'got stuck', output=
                                    'fuzzy case: fixture::elsewhere seed 0000000000000011: unrelated'),),
             dict(status=100), 'otherwise', None, 'got stuck'),
            ('wrong_case', (failed_case('wrong_case', '000000000000000a', 'count 12'),),
             dict(status=100), 'otherwise', None, 'count 12'),
            ('outside', (testcase('outside', failure='test failure', message='setup failed'),),
             dict(status=100), 'otherwise', None, 'setup failed'),
        ]
        cases = [failed_case(name, seed, 'count 7') for name, *_ in rows]
        self.round_report('round-1', *cases, status=100)
        for f, (case, (_, tests, options, *_)) in enumerate(zip(cases, rows), 1):
            self.round_report(f'round-1-replay-{f}', case, status=100)
            self.round_report(f'last-{f}', *tests, **options)
        _, data, text, calls = self.campaign('--last', str(self.previous), exit=1)
        self.assertIsNone(data['ended'])
        self.assertEqual([Path(c['args'][1]).name for c in calls if 'head' in c],
                         [f'last-{f}' for f in range(1, len(rows) + 1)])
        self.assertEqual(len(data['findings']), len(rows))
        words = {'unobserved': 'unobserved', 'absent': 'ran no case there', 'otherwise': 'failed otherwise'}
        for f, (finding, (name, _, _, verdict, why, first_line)) in enumerate(zip(data['findings'], rows), 1):
            with self.subTest(name=name):
                self.assertEqual(finding['test'], name)
                self.assertEqual(finding['last'], dict(directory=f'last-{f}', verdict=verdict, why=why,
                                                       first_line=first_line, input=None, compared='not compared'))
                line = (f"At the last campaign's commit, {self.last}: {words[verdict]}: "
                        f'{why or first_line}; input not compared.')
                self.assertIn(line, finding['body'].splitlines())
                self.assertIn(line, text.splitlines())
        self.assert_last_removed()

    def test_last_compares_only_matching_inputs(self):
        seed = '00000000000000ab'
        a, b = '0a' * 32, '1b' * 32
        rows = [('same', a, a, 'same', 'the same'), ('differs', a, b, 'differs', 'differs'),
                ('neither', None, None, 'not compared', 'not compared'),
                ('current_only', a, None, 'not compared', 'not compared'),
                ('last_only', None, b, 'not compared', 'not compared')]
        self.round_report('round-1', *(failed_case(name, seed, 'count 7', output=input_line(name, seed, b))
                                      for name, *_ in rows), status=100)
        for f, (name, current, previous, _, _) in enumerate(rows, 1):
            decoys = (input_line('another', seed, b) + input_line(name, '00000000000000ac', b)
                      + input_line(name, seed, 'B' * 64) + input_line(name, seed, 'c' * 63)
                      + 'ignore ' + input_line(name, seed, b))
            output = input_line(name, seed.upper(), current)
            if current is not None:
                output += input_line(name, seed, b)  # The first matching line wins.
            self.round_report(f'round-1-replay-{f}', failed_case(name, seed, 'count 8', output=decoys + output), status=100)
            self.round_report(f'last-{f}', testcase(name, count=1, output=decoys + input_line(name, seed, previous)))
        _, data, text, _ = self.campaign('--last', str(self.previous), exit=1)
        for case, finding, (name, current, previous, comparison, words) in zip(data['rounds'][0]['cases'], data['findings'], rows):
            with self.subTest(name=name):
                self.assertEqual((case['replay']['input'], finding['last']['input'], finding['last']['compared']),
                                 (current, previous, comparison))
                line = f"At the last campaign's commit, {self.last}: passed; input {words}."
                self.assertIn(line, finding['body'].splitlines())
                self.assertIn(line, text.splitlines())

    def test_last_worktree_add_failure_is_unobserved(self):
        case = failed_case('numbered', '0000000000000011', 'count 7')
        self.round_report('round-1', case, status=100)
        self.round_report('round-1-replay-1', case, status=100)
        _, data, text, calls = self.campaign('--last', str(self.previous), exit=1, BLOCK_WORKTREE='1')
        self.assertIsNone(data['ended'])
        self.assertEqual([Path(c['args'][1]).name for c in calls if c['name'] == 'run'], ['round-1', 'round-1-replay-1'])
        finding, = data['findings']
        why = finding['last']['why']
        self.assertIn('last-worktree', why)
        self.assertNotIn('\n', why)
        self.assertEqual(finding['last'], dict(directory=None, verdict='unobserved', why=why,
                                               first_line=None, input=None, compared='not compared'))
        self.assertIn(f"At the last campaign's commit, {self.last}: unobserved: {why}; input not compared.", text)
        self.assertEqual((self.out / 'last-worktree').read_text(), 'not a worktree')

    def test_early_end_keeps_last_replays_pending(self):
        case = failed_case('numbered', '0000000000000011', 'count 7')
        self.round_report('round-1', case, status=100)
        self.round_report('round-1-replay-1', case, status=100)
        self.stage('round-2', 2, 'build failed\n')
        self.round_report('last-1', testcase('numbered', count=1))
        _, data, text, calls = self.campaign('--last', str(self.previous), '--rounds', '2', exit=2)
        self.assertEqual([Path(c['args'][1]).name for c in calls if c['name'] == 'run'],
                         ['round-1', 'round-1-replay-1', 'round-2'])
        self.assertEqual(data['ended']['step'], 'round')
        self.assertIsNone(data['findings'][0]['last']['verdict'])
        self.assertIn(f"At the last campaign's commit, {self.last}: pending.", text)
        self.assert_last_removed()

    def test_interruption_during_last_replay_removes_worktree(self):
        first = failed_case('numbered', '0000000000000011', 'count 7')
        second = failed_case('other', '0000000000000012', 'count 8')
        self.round_report('round-1', first, second, status=100)
        self.round_report('round-1-replay-1', first, status=100)
        self.round_report('round-1-replay-2', second, status=100)
        self.round_report('last-1', testcase('numbered', count=1))
        _, data, text, _ = self.campaign('--last', str(self.previous), exit=2,
                                         INTERRUPT=str(int(signal.SIGTERM)), INTERRUPT_STEP='last-2')
        self.assertEqual([finding['last']['verdict'] for finding in data['findings']], ['passed', None])
        self.assertEqual(data['ended'], dict(step='interrupted', round=None, replay=2, returncode=23,
                                             output='run got SIGTERM'))
        self.assertIn('Ended at interrupted', text)
        self.assert_last_removed()

    def test_grouping_in_order_bounded_seeds_and_zero_reach(self):
        name, other = 'fuzzy::numbered', 'fuzzy::alphabetically_first'
        self.select(name, other)
        seeds = [f'{n:016x}' for n in range(10, 19)]
        for r, seed in enumerate(seeds, 1):
            message = 'expected 3, saw 4' if r == 1 else 'expected 12, saw 1'
            test = failed_case(name, seed, message, count=1)
            if r <= 2:
                test.find('system-err').text += ('fuzzy reach: walking forge/busy 0\n'
                                                 f'fuzzy reach: walking forge/slow {r - 1}\n')
            selected = [test]
            if r == 1:
                selected.append(failed_case(other, '00000000000000ff', 'split 7'))
            self.round_report(f'round-{r}', *selected, status=100)
            for k, test in enumerate(selected, 1):
                self.round_report(f'round-{r}-replay-{k}', test, status=100)
        _, data, _, _ = self.campaign('--rounds', '9', exit=1)
        self.assertEqual(len(data['findings']), 2)
        first, second = data['findings']
        self.assertEqual((first['test'], first['signature'], first['count']), (name, 'expected N, saw N', 9))
        self.assertEqual(first['seeds'], [dict(seed=seed, round=i) for i, seed in enumerate(seeds[:8], 1)])
        self.assertEqual((second['test'], second['signature'], second['count']), (other, 'split N', 1))
        self.assertIn(f'{seeds[7]} (round 8)', first['body'])
        self.assertNotIn(seeds[8], first['body'])
        self.assertEqual(data['never_fell'], [dict(sweep='walking', kind='forge/busy')])
        self.assertEqual(data['reach'], [dict(sweep='walking', kind='forge/busy', cases=0),
                                         dict(sweep='walking', kind='forge/slow', cases=1)])

    def test_title_and_body_bounds(self):
        name, seed = 'fuzzy::bounded', '00000000000000ab'
        title_prefix = f'fixture::{name} fails on a fresh seed: '
        plain = '\t' + 'a' * (198 - len(title_prefix))
        first = plain + '界 rest'  # The character starts at byte 200, so none of it fits.
        self.select(name)
        self.round_report('round-1', failed_case(name, seed, f'{first}\nlast kept line\n' + '文' * 1400), status=100)
        self.round_report('round-1-replay-1', failed_case(name, seed, 'same case'), status=100)
        _, data, _, _ = self.campaign('--seed', 'ab' * 8, exit=1)
        finding, = data['findings']
        self.assertEqual(finding['title'], title_prefix + plain.replace('\t', ' '))
        self.assertEqual(len(finding['title'].encode()), 199)
        body = finding['body']
        self.assertLessEqual(len(body.encode()), 4096)
        self.assertEqual(body.splitlines(), [f'Commit: {self.head}', 'Campaign seed: ' + 'ab' * 8,
                         f"Round 1 seed: {data['rounds'][0]['seed']}", f'Case seeds: {seed} (round 1)',
                         'Replay: ' + finding['command'], '', f'fuzzy case: fixture::{name} seed {seed}: {first}',
                         'last kept line'])

    def test_checkpoints_before_each_replay_and_round(self):
        names = ('fuzzy::one', 'fuzzy::two')
        seed = '0' * 16
        self.round_report('round-1', *(failed_case(name, seed, name) for name in names), status=100)
        for k, name in enumerate(names, 1):
            self.round_report(f'round-1-replay-{k}', testcase(name, count=1))
        self.round_report('round-2', testcase('fuzzy::fixed'))
        _, data, text, calls = self.campaign('--rounds', '2', exit=1)
        self.assertEqual([(c['rounds'], c['replays'], c['summary']) for c in calls if c['name'] == 'run'], [
            ([], [], 'Rounds: 0 of 2'), ([1], [None, None], 'Rounds: 1 of 2'),
            ([1], ['passed', None], 'Rounds: 1 of 2'), ([1], ['passed', 'passed'], 'Rounds: 1 of 2')])
        self.assertEqual(len(data['rounds']), 2)
        self.assertIn('Rounds: 2 of 2', text)

    def test_interruption_reaches_run_and_keeps_finished_round(self):
        self.round_report('round-1', testcase('fuzzy::fixed', count=2))
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            with self.subTest(signal=sig.name):
                self.out = self.root / sig.name
                _, data, text, _ = self.campaign('--rounds', '2', exit=2, INTERRUPT=str(int(sig)))
                self.assertEqual([r['round'] for r in data['rounds']], [1])
                self.assertEqual(data['ended'], dict(step='interrupted', round=2, replay=None,
                                                     returncode=23, output=f'run got {sig.name}'))
                self.assertIn('Rounds: 1 of 2', text)
                self.assertIn('fuzzy::fixed: passed, 2 cases', text)
                self.assertIn('Ended at interrupted', text)

    def test_early_end_records_step_status_and_tail(self):
        self.out = self.root / 'list failed'
        _, data, text, calls = self.campaign(exit=2, LIST_FAIL='1')
        self.assertEqual(data['ended'], dict(step='listing', round=None, replay=None, returncode=17,
                                             output='\n'.join(f'listing {n}' for n in range(5, 25))))
        self.assertEqual([c['name'] for c in calls], ['df', 'wrapper'])
        self.assertIn('Ended at listing', text)
        self.assertEqual(data['rounds'], [])

        tail = ''.join(f'build failed {n}\n' for n in range(25))
        for step, replay in (('round-1', None), ('round-1-replay-1', 1)):
            with self.subTest(step=step):
                self.out = self.root / (step + '-failed')
                if replay:
                    self.round_report('round-1', failed_case('fuzzy::fixed', '0' * 16, 'oops'), status=100)
                self.stage(step, 19, tail)
                _, data, text, _ = self.campaign(exit=2)
                self.assertEqual(data['ended'], dict(step='replay' if replay else 'round', round=1,
                                                     replay=replay, returncode=19,
                                                     output='\n'.join([*(f'build failed {n}' for n in range(6, 25)),
                                                                       f'run: {step}'])))
                self.assertEqual(len(data['rounds']), 1 if replay else 0)
                self.assertIn(f"Ended at {'replay' if replay else 'round'}", text)


if __name__ == '__main__':
    unittest.main()
