"""The shapes reader against synthetic ctl JSON, never a running deployment."""
import json
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
READER = Path('tools/shapes/shapes.py')
VOCABULARY = Path('smith/testing/smith-shapes/vocabulary.txt')
FROM, TO = '2026-11-28T00:00:00Z', '2026-11-28T00:01:00Z'
START = 1795824000000000000
STOP = START + 60 * 10**9
TASK, CUT, ENDED, OLD, FAILED, BAD = range(812301, 812307)
OLD_JOB, JOB, NEVER, QUESTION, WAKE, ABSENT_MESSAGE, LATER = range(836201, 836208)
IDS = (TASK, CUT, ENDED, OLD, FAILED, BAD, OLD_JOB, JOB, NEVER, QUESTION, WAKE, ABSENT_MESSAGE, LATER)
SENTINEL = 'PRIVATE_SENTINEL_807934'
PROJECT = 'project ' + SENTINEL
LOG = 'worlds.log'
EMPTY_SAMPLE = 'fuzzy shapes: empty smith/3 cases 1 distinct 0 beyond 0\n'
PATTERN = ('job:unknown_job:job=before_opening,write:not_run_after_failure:path=unknown,read:ok:path=unknown'
           ' end=parked begin=message wake=answer')

FAKE = '''import json
from pathlib import Path
import sys

args = sys.argv[1:]
responses = json.loads(Path(__file__).with_name('responses.json').read_text())
if not args or args[-1] != '--json' or '\\x1f'.join(args[:-1]) not in responses:
    print('unexpected ctl call ' + repr(args), file=sys.stderr)
    sys.exit(90)
status, stdout, stderr = responses['\\x1f'.join(args[:-1])]
print(stdout)
print(stderr, file=sys.stderr)
sys.exit(status)
'''


def task_line(task, last=START, end=None):
    return {'chip': {'task': task, 'title': {'text': SENTINEL}}, 'last_activity': last, 'ended_at': end}


def cursor(value):
    return {'value': value} if value else None


def page(*rows, before=None, cut=False):
    return {'rows': list(rows), 'before': cursor(before), 'after': None, 'cut': cut, 'read_on': None}


def part(text, left=0):
    return {'part': 1, 'from': 0, 'text': text, 'bytes_left': left, 'read_on': None}


def call(tool, answer, *, failed=False, outcome=SENTINEL, **inputs):
    return {'call': {'number': 1, 'tool': tool, 'answer': answer, 'failed': failed, 'objects': [], 'parts': [],
                     'input': part(json.dumps({'fixture': SENTINEL, **inputs})), 'outcome': part(outcome)}}


def spoken(*blocks, speaker='assistant'):
    return {'speaker': speaker, 'blocks': [{'text': part(SENTINEL)}, *blocks]}


def row(task, position, at, event, archive=None):
    owner = {'source': {'task': task, 'position': position, 'at': at, 'line': event}, 'decoded': archive}
    return {'position': position, 'at': at, 'event': {'owner': owner}}


def turn(task, position, at, attempt, *messages, cut=False, unread=False):
    name = {'attempt': attempt, 'number': position}
    archive = None if unread else {'task': task, 'name': name, 'at': at, 'read': None, 'cumulative': position,
                                   'turn': {'decoded': {'messages': messages, 'cut': cut}}}
    return row(task, position, at, {'turn': {'attempt': attempt, 'turn': position, 'reply': 1, 'read': None}}, archive)


def began(task, position, at, attempt, why):
    return row(task, position, at, {'run_began': {'attempt': attempt, 'why': why, 'resumed': attempt > 1, 'fresh': None}})


def ended(task, position, at, attempt, end):
    return row(task, position, at, {'run_ended': {'attempt': attempt, 'end': end, 'spent': 0}})


class Shapes(unittest.TestCase):
    maxDiff = None

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='shapes-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for path in (READER, VOCABULARY):
            self.write(path, (ROOT / path).read_text(encoding='utf-8'))
        self.write('fake ctl.py', FAKE)
        self.responses = {}
        self.lists()
        self.write(READER.parent / LOG, EMPTY_SAMPLE)

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')
        return path

    def reply(self, words, document=None, *, status=0, text=None):
        self.responses['\x1f'.join(words)] = (status, json.dumps(document) if text is None else text, SENTINEL)

    def lists(self, live=(), ended=(), *, project=PROJECT, live_after=None, ended_after=None):
        words = ['task', 'list', '--project', project]
        next_read = shlex.join(words + ['--after', live_after]) if live_after else None
        self.reply(words, {'tasks': {'rows': list(live), 'more': live_after is not None}, 'read_on': next_read})
        self.reply(words + ['--ended'], {'live': [], 'live_after': None, 'read_on': None,
                                         'ended': {'rows': {'rows': list(ended), 'after': cursor(ended_after),
                                                            'cut': ended_after is not None}}})

    def activity(self, task, document=None, *, before=None, **failure):
        if document and document.get('before'):
            document['read_on'] = f'task activity {task} --before {document["before"]["value"]}'
        self.reply(['task', 'activity', str(task)] + (['--before', before] if before else []), document, **failure)

    def run_reader(self, *extra, worlds=(LOG,), status=0, start=FROM, stop=TO, ctl=None):
        self.write('responses.json', json.dumps(self.responses))
        command = ctl if ctl is not None else shlex.join([sys.executable, '-I', '-B', str(self.root / 'fake ctl.py')])
        result = subprocess.run([sys.executable, '-I', '-B', str(self.root / READER), '--ctl', command,
                                 '--from', start, '--to', stop, '--project', PROJECT,
                                 *(word for path in worlds for word in ('--worlds', path)), *extra],
                                cwd=self.root / READER.parent, capture_output=True, text=True, timeout=10)
        self.assert_clean(result.stdout + result.stderr)
        self.assertEqual(result.returncode, status, result.stdout + result.stderr)
        self.assertEqual(result.stdout if status else result.stderr, '')
        return result.stderr if status else result.stdout

    def assert_clean(self, text):
        self.assertNotIn(SENTINEL, text)
        for number in IDS:
            self.assertNotRegex(text, rf'\b{number}\b')

    def assert_report(self, want, *args, **kwargs):
        self.assert_clean(want)
        self.assertEqual(self.run_reader(*args, **kwargs), want)

    def test_shapes_every_page_and_uses_prior_attempts_and_later_run_rows(self):
        self.lists([task_line(TASK, STOP)], [task_line(ENDED, START, START)], live_after='aa', ended_after='bb')
        words = ['task', 'list', '--project', PROJECT]
        self.reply(words + ['--after', 'aa'], {'live': [task_line(CUT), task_line(LATER, STOP)],
                                               'live_after': None, 'ended': 'none', 'read_on': None})
        self.reply(words + ['--ended', '--after', 'bb'], {'live': [], 'live_after': None, 'read_on': None,
                   'ended': {'rows': {'rows': [task_line(OLD, START - 1, START - 1)], 'after': None, 'cut': False}}})
        self.activity(LATER, page(turn(LATER, 1, START, 1, unread=True)))

        issue = spoken(call('shell', 'running', outcome=f'Started as job {OLD_JOB}; {SENTINEL}', command=SENTINEL),
                       {'text': part(f'{SENTINEL} names task {ENDED}.')})
        self.activity(TASK, page(
            began(TASK, 4, START - 1, 2, 'ready'), ended(TASK, 3, START - 2, 1, 'parked'),
            turn(TASK, 2, START - 3, 1, issue), began(TASK, 1, START - 4, 1, 'made')), before='ab')
        case = turn(TASK, 7, START + 1, 2,
                    spoken(call('job', 'unknown_job', job=OLD_JOB),
                           call('write', 'not_run_after_failure', path=SENTINEL, content=SENTINEL),
                           call('read', 'ok', path=SENTINEL)),
                    spoken(call('read_task', 'delegated', task=ENDED),
                           call('message', 'delegated', failed=True, target=ENDED, form='answer', question=QUESTION),
                           call('message', 'delegated', target=TASK, form='words', words=SENTINEL)),
                    spoken(call('job', 'exited', job=JOB), call('job', 'unknown_job', job=JOB)),
                    spoken(call('mark_read', 'delegated', messages=[QUESTION, ABSENT_MESSAGE]),
                           call('subscribe', 'delegated', target=TASK),
                           call('read', 'ok', path=SENTINEL, offset=ABSENT_MESSAGE)),
                    spoken(call(SENTINEL, None), call(SENTINEL, 'delegated'),
                           call(SENTINEL, 'unknown_tool'), call('message', 'not_run_stopped', target=NEVER)),
                    spoken(), spoken(), spoken(speaker='person'), spoken(speaker='tool'))
        heard = {'number': QUESTION, 'kind': 'question', 'words': SENTINEL, 'from': {'person': 1}}
        self.activity(TASK, page(
            turn(TASK, 11, STOP, 3, spoken()), began(TASK, 10, STOP, 3, {'message': {'message': WAKE}}),
            row(TASK, 9, STOP, {'words': {'message': WAKE, 'kind': {'answer': {'question': QUESTION}}, 'words': SENTINEL}}),
            ended(TASK, 8, STOP, 2, 'parked'),
            turn(TASK, 6, START, 2, spoken(call('shell', 'running', outcome=f'Started as job {JOB}; {SENTINEL}'))),
            case, row(TASK, 5, START - 1, {'heard': heard}), before='ab'))

        self.activity(CUT, page(ended(CUT, 2, START + 1, 2, {'failed': {'class': SENTINEL}}),
                                turn(CUT, 1, START, 2, spoken(call('job', 'unknown_job', job=NEVER))), cut=True))
        self.activity(ENDED, page(ended(ENDED, 2, START, 1, 'finished'),
                                  turn(ENDED, 1, START, 1, spoken(call('job', 'unknown_job', job=NEVER)))))
        self.assert_report(
            'shapes: smith/3\n'
            'deployment: tasks 4, turns 5, completions 10, shaped 10\n'
            'incomplete: task unread 0, turn unread 1, turn cut 0, input unread 0, page cut 1\n'
            'worlds: samples 1, cases 1, shapes 0, samples cut 0\n'
            'missed: 9 shapes in 10 completions\n'
            '2 text end=parked begin=message wake=answer\n'
            '1 delegated:unanswered,delegated:delegated,unknown:unknown_tool,message:not_run_stopped:task=unissued'
            ' end=parked begin=message wake=answer\n'
            '1 job:exited:job=unknown,job:unknown_job:job=consumed end=parked begin=message wake=answer\n'
            f'1 {PATTERN}\n'
            '1 job:unknown_job:job=unissued end=finished begin=none wake=none\n'
            '1 job:unknown_job:job=unknown end=failed begin=unknown wake=unknown\n'
            '1 mark_read:marked:message=live:message=unissued,subscribe:subscribed:task=live,'
            'read:ok:path=unknown:offset=unknown end=parked begin=message wake=answer\n'
            '1 read_task:read:task=ended,message:unknown:task=ended:question=live,message:sent:task=live'
            ' end=parked begin=message wake=answer\n'
            '1 shell:running end=parked begin=message wake=answer\n'
            'reached through unknown: 0 completions\n'
            'sample empty cases 1 distinct 0 beyond 0\n')

    def test_counts_cut_turns_and_unread_inputs_without_losing_other_completions(self):
        self.lists([task_line(TASK)])
        withheld = call('read', 'ok', path=SENTINEL)
        withheld['call']['input'] = None
        prefixes = [part('{"' + SENTINEL), part(json.dumps({'path': SENTINEL}), left=1),
                    part(json.dumps([SENTINEL]))]
        unusable = [withheld]
        for prefix in prefixes:
            block = call('read', 'malformed')
            block['call']['input'] = prefix
            unusable.append(block)
        self.activity(TASK, page(turn(TASK, 2, START, 1, spoken(call('shell', 'running')), cut=True),
                                 turn(TASK, 1, START, 1, *(spoken(block) for block in unusable),
                                      spoken(call('mark_read', 'delegated', messages=[NEVER]),
                                             call('read', SENTINEL, path=SENTINEL)))))
        self.assert_report(
            'shapes: smith/3\n'
            'deployment: tasks 1, turns 2, completions 5, shaped 1\n'
            'incomplete: task unread 0, turn unread 0, turn cut 1, input unread 4, page cut 0\n'
            'worlds: samples 1, cases 1, shapes 0, samples cut 0\n'
            'missed: 1 shapes in 1 completions\n'
            '1 mark_read:marked:message=unknown,read:unknown:path=unknown end=unknown begin=unknown wake=unknown\n'
            'reached through unknown: 0 completions\n'
            'sample empty cases 1 distinct 0 beyond 0\n')

    def test_matches_unknown_in_both_directions_and_prefers_an_exact_match(self):
        self.lists([task_line(TASK)])
        self.activity(TASK, page(ended(TASK, 2, START, 1, 'parked'), turn(
            TASK, 1, START, 1, spoken(call('read', 'ok', path=SENTINEL)), spoken(),
            spoken(call('write', 'ok', path=SENTINEL)), spoken(call('write', 'ok', path=SENTINEL)))))
        lines = ('fuzzy shapes: sweep smith/3 cases 8 distinct 4 beyond 2\n'
                 'fuzzy shape: sweep smith/3 5 read:ok:path=live end=unknown begin=unknown wake=unknown\n'
                 'fuzzy shape: sweep smith/3 4 text end=parked begin=unknown wake=unknown\n'
                 'fuzzy shape: sweep smith/3 4 text end=unknown begin=unknown wake=unknown\n'
                 'fuzzy shape: sweep smith/3 1 unknown:unknown:path=unknown end=unknown begin=unknown wake=unknown\n')
        self.write(READER.parent / LOG, lines)
        xml = self.root / READER.parent / 'worlds.xml'
        suite = ET.Element('testsuite')
        ET.SubElement(ET.SubElement(suite, 'testcase'), 'system-err').text = lines
        ET.ElementTree(suite).write(xml, encoding='utf-8')
        expected = ('shapes: smith/3\n'
                    'deployment: tasks 1, turns 1, completions 4, shaped 4\n'
                    'incomplete: task unread 0, turn unread 0, turn cut 0, input unread 0, page cut 0\n'
                    'worlds: samples 1, cases 8, shapes 4, samples cut 1\n'
                    'missed: 1 shapes in 2 completions\n'
                    '2 write:ok:path=unknown end=parked begin=unknown wake=unknown\n'
                    'reached through unknown: 1 completions\n'
                    'sample sweep cases 8 distinct 4 beyond 2\n')
        self.assert_report(expected, worlds=(str(self.root / READER.parent / LOG),))
        self.assert_report(expected, worlds=('worlds.xml',))
        merged = expected.replace('cases 8', 'cases 16').replace('beyond 2', 'beyond 4')
        self.assert_report(merged, worlds=(str(self.root / READER.parent / LOG), 'worlds.xml'))

    def test_task_activity_failures_are_nonfatal_and_discard_partial_activity(self):
        # Exercise a second project's list too. Its page failure must not stop the first project's shape.
        self.lists([task_line(FAILED), task_line(TASK)])
        other = 'other project ' + SENTINEL
        self.lists([task_line(BAD)], project=other)
        self.activity(FAILED, page(turn(FAILED, 2, START, 1, spoken()), before='ab'))
        self.activity(FAILED, before='ab', status=7, text=SENTINEL)
        self.activity(BAD, {'cut': False, 'rows': SENTINEL, 'before': None})
        self.activity(TASK, page(turn(TASK, 1, START, 1, spoken())))
        self.assert_report(
            'shapes: smith/3\n'
            'deployment: tasks 3, turns 1, completions 1, shaped 1\n'
            'incomplete: task unread 2, turn unread 0, turn cut 0, input unread 0, page cut 0\n'
            'worlds: samples 1, cases 1, shapes 0, samples cut 0\n'
            'missed: 1 shapes in 1 completions\n'
            '1 text end=unknown begin=unknown wake=unknown\n'
            'reached through unknown: 0 completions\n'
            'sample empty cases 1 distinct 0 beyond 0\n', '--project', other)

    def test_world_refusals_give_safe_version_and_locations(self):
        run = 'end=unknown begin=unknown wake=unknown'
        for text, expected in (
                ('fuzzy shapes: sample smith/2 cases 0 distinct 0 beyond 0\n',
                 f'{LOG}:1: worlds version smith/2, expected smith/3'),
                (f'{EMPTY_SAMPLE}fuzzy shape: empty smith/2 1 text {run}\n',
                 f'{LOG}:2: worlds version smith/2, expected smith/3'),
                (f'fuzzy shapes: empty smith/3 cases 1 distinct 1 beyond 0\n'
                 f'fuzzy shape: empty smith/3 1 read:ok:path={SENTINEL} {run}\n',
                 f'{LOG}:2: worlds shape not in form'),
                (f'fuzzy shapes: empty smith/3 cases {SENTINEL} distinct 0 beyond 0\n',
                 f'{LOG}:1: worlds header not in form'),
                (SENTINEL, 'no worlds sample')):
            with self.subTest(expected=expected):
                self.write(READER.parent / LOG, text)
                self.assertEqual(self.run_reader(status=1), f'shapes: {expected}\n')

    def test_unreadable_task_list_is_fatal_without_repeating_ctl_output(self):
        self.reply(['task', 'list', '--project', PROJECT, '--ended'], status=9, text=SENTINEL)
        self.assertEqual(self.run_reader(status=1), 'shapes: task list unreadable\n')

    def test_refuses_a_vocabulary_not_in_form(self):
        self.write(VOCABULARY, (ROOT / VOCABULARY).read_text(encoding='utf-8').rstrip('\n') + ' ' + SENTINEL + '\n')
        self.assertEqual(self.run_reader(status=1), 'shapes: vocabulary not in form\n')

    def test_bad_usage_is_fixed_text(self):
        for options in ({'start': SENTINEL}, {'stop': FROM}, {'ctl': '"' + SENTINEL}):
            with self.subTest(options=tuple(options)):
                diagnostic = self.run_reader(status=2, **options)
                self.assertTrue(re.fullmatch(r'usage: [^\n]+\nshapes: invalid arguments\n', diagnostic))


if __name__ == '__main__':
    unittest.main()
