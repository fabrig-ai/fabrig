#!/usr/bin/env python3
"""Compare deployment completion shapes with world samples (tools/shapes/design.md)."""
import argparse
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import shlex
import subprocess
import sys
import xml.etree.ElementTree as ET

VOCABULARY = Path(__file__).resolve().parents[2] / 'smith/testing/smith-shapes/vocabulary.txt'
FIELDS = ('tool', 'answer', 'reference', 'class', 'end', 'begin', 'wake')
REFERENCES = ('path', 'offset', 'job', 'task', 'question', 'message')
RUN_FIELDS = ('end', 'begin', 'wake')
VERSION = r'[a-z][a-z0-9_-]*/[0-9]+'
WORD = r'[a-z][a-z0-9_]*'
SWEEP = r'[A-Za-z0-9_][A-Za-z0-9_.:-]*'
UINT = r'[0-9]+'
U64_MAX = (1 << 64) - 1
UNKNOWN = 'unknown'


class Refusal(Exception):
    """A report cannot be made. Only safe diagnostics are passed here."""


class BadPage(Exception):
    """A ctl invocation failed or its page cannot be read."""


class Usage(argparse.ArgumentParser):
    def error(self, _message):
        self.print_usage(sys.stderr)
        self.exit(2, 'shapes: invalid arguments\n')


def wall(text):
    if not re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z', text):
        raise ValueError
    epoch = datetime(1970, 1, 1, tzinfo=timezone.utc)
    delta = datetime.fromisoformat(text[:-1] + '+00:00') - epoch
    return (delta.days * 86400 + delta.seconds) * 10**9


def arguments(argv):
    parser = Usage(prog='python3 -I -B tools/shapes/shapes.py', allow_abbrev=False,
                   usage='%(prog)s --project NAME [--project NAME ...] --from TIME --to TIME '
                         '--worlds PATH [--worlds PATH ...] [--ctl COMMAND]')
    parser.add_argument('--project', action='append', required=True)
    parser.add_argument('--from', dest='start', required=True)
    parser.add_argument('--to', dest='stop', required=True)
    parser.add_argument('--worlds', action='append', required=True)
    parser.add_argument('--ctl', default='temper ctl')
    args = parser.parse_args(argv)
    try:
        args.start, args.stop = wall(args.start), wall(args.stop)
        args.ctl = shlex.split(args.ctl)
    except ValueError:
        parser.error(None)
    if args.start >= args.stop or not args.ctl or not args.ctl[0] or not all(args.project):
        parser.error(None)
    return args


@dataclass
class Vocabulary:
    version: str
    fields: dict

    @classmethod
    def read(cls, path):
        try:
            lines = path.read_bytes().decode('ascii').split('\n')
            if len(lines) != len(FIELDS) + 2 or not re.fullmatch(VERSION, lines[0]) or lines[-1]:
                raise ValueError
            fields = {}
            for name, line in zip(FIELDS, lines[1:-1]):
                key, *words = line.split(' ')
                if (key != name or not words or len(set(words)) != len(words)
                        or not all(re.fullmatch(WORD, word) for word in words)
                        or (name != 'reference' and UNKNOWN not in words)):
                    raise ValueError
                fields[name] = tuple(words)
            if fields['reference'] != REFERENCES:
                raise ValueError
        except OSError:
            raise Refusal('vocabulary unreadable') from None
        except ValueError:
            raise Refusal('vocabulary not in form') from None
        return cls(lines[0], fields)

    def word(self, name, value):
        return value if value in self.fields[name] else UNKNOWN


@dataclass(frozen=True)
class Shape:
    calls: tuple
    run: tuple

    @classmethod
    def parse(cls, text, vocabulary):
        if not 1 <= len(text) <= 1024:
            raise ValueError
        tools, *run = text.split(' ')
        if len(run) != len(RUN_FIELDS):
            raise ValueError
        words = []
        for name, part in zip(RUN_FIELDS, run):
            key, word = part.split('=', 1)
            if key != name or word not in vocabulary.fields[name]:
                raise ValueError
            words.append(word)
        calls = []
        if tools != 'text':
            for call in tools.split(','):
                tool, answer, *refs = call.split(':')
                if tool not in vocabulary.fields['tool'] or answer not in vocabulary.fields['answer']:
                    raise ValueError
                references = []
                previous = -1
                for ref in refs:
                    name, kind = ref.split('=', 1)
                    index = REFERENCES.index(name)
                    if (index < previous or (index == previous and name != 'message')
                            or kind not in vocabulary.fields['class']):
                        raise ValueError
                    references.append((name, kind))
                    previous = index
                calls.append((tool, answer, tuple(references)))
        return cls(tuple(calls), tuple(words))

    def text(self):
        calls = [':'.join((tool, answer, *(f'{ref}={kind}' for ref, kind in refs)))
                 for tool, answer, refs in self.calls]
        return ' '.join((','.join(calls) or 'text',
                         *(f'{name}={word}' for name, word in zip(RUN_FIELDS, self.run))))

    def signature(self):
        return tuple((tool, tuple(ref for ref, _ in refs)) for tool, _, refs in self.calls)

    def compared_words(self):
        return tuple(word for _, answer, refs in self.calls for word in (answer, *(c for _, c in refs))) + self.run


@dataclass
class Sample:
    cases: int = 0
    beyond: int = 0
    shapes: Counter = field(default_factory=Counter)


HEADER = re.compile(rf'fuzzy shapes: ({SWEEP}) ({VERSION}) cases ({UINT}) distinct ({UINT}) beyond ({UINT})')
SHAPE_LINE = re.compile(rf'fuzzy shape: ({SWEEP}) ({VERSION}) ({UINT}) (.*)')


def count(text):
    number = int(text)
    if number > U64_MAX:
        raise ValueError
    return number


def world_path(path):
    # Keep even a user-supplied path with control characters on one error line.
    return ''.join(ch if ch.isprintable() else ascii(ch)[1:-1] for ch in path)


def world_streams(path):
    try:
        with open(path, encoding='utf-8-sig') as file:
            text = file.read()
    except (OSError, ValueError):
        raise Refusal(f'{world_path(path)}: worlds file unreadable') from None
    if not re.match(r'\s*<(?:\?xml\b|testsuites?\b)', text) and Path(path).suffix.lower() != '.xml':
        yield text
        return
    try:
        root = ET.fromstring(text)
        if root.tag not in ('testsuites', 'testsuite'):
            raise ET.ParseError
    except (ET.ParseError, ValueError):
        raise Refusal(f'{world_path(path)}: worlds JUnit not in form') from None
    for case in root.iter('testcase'):
        for output in case:
            if output.tag in ('system-out', 'system-err'):
                yield ''.join(output.itertext())


def world_samples(text, path, vocabulary):
    """Read complete header/shape groups from one text stream (design, 2.3)."""
    pending = None
    for lineno, line in enumerate(text.split('\n'), 1):
        header = line.startswith('fuzzy shapes:')
        if not header and not line.startswith('fuzzy shape:'):
            continue
        match = (HEADER if header else SHAPE_LINE).fullmatch(line)
        location = f'{world_path(path)}:{lineno}'
        kind = 'header' if header else 'shape'
        if not match:
            raise Refusal(f'{location}: worlds {kind} not in form')
        sweep, version = match[1], match[2]
        if version != vocabulary.version:
            raise Refusal(f'{location}: worlds version {version}, expected {vocabulary.version}')
        try:
            if header:
                if pending:
                    yield complete_sample(pending)
                cases, distinct, beyond = map(count, match.groups()[2:])
                if distinct > 4096 or beyond > cases:
                    raise ValueError
                pending = sweep, Sample(cases, beyond), distinct, location
            else:
                if not pending or sweep != pending[0]:
                    raise ValueError
                sample = pending[1]
                reached = count(match[3])
                shape = Shape.parse(match[4], vocabulary)
                if not 0 < reached <= sample.cases or shape in sample.shapes or len(sample.shapes) == pending[2]:
                    raise ValueError
                sample.shapes[shape] = reached
        except ValueError:
            raise Refusal(f'{location}: worlds {kind} not in form') from None
    if pending:
        yield complete_sample(pending)


def complete_sample(pending):
    sweep, sample, distinct, location = pending
    if len(sample.shapes) != distinct:
        raise Refusal(f'{location}: worlds header not in form')
    return sweep, sample


def read_worlds(paths, vocabulary):
    samples = defaultdict(Sample)
    for path in paths:
        for text in world_streams(path):
            for sweep, part in world_samples(text, path, vocabulary):
                sample = samples[sweep]
                sample.cases += part.cases
                sample.beyond += part.beyond
                sample.shapes.update(part.shapes)
    if not samples:
        raise Refusal('no worlds sample')
    return samples


def unsigned(value):
    return type(value) is int and 0 <= value <= U64_MAX


def require(value, kind):
    if not (unsigned(value) if kind is int else isinstance(value, kind)):
        raise BadPage
    return value


def not_json(_word):
    raise ValueError


def load_json(text):
    return json.loads(text, parse_constant=not_json)


def ctl_page(command, words):
    try:
        result = subprocess.run([*command, *words, '--json'], stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=True)
        return require(load_json(result.stdout.decode('utf-8')), dict)
    except (OSError, ValueError, RecursionError, subprocess.CalledProcessError):
        raise BadPage from None


def cursor_value(value):
    if not isinstance(value, str) or not re.fullmatch(r'(?:[0-9a-fA-F]{2})+', value):
        raise BadPage
    return value.lower()


def cursor(value):
    return None if value is None else cursor_value(require(value, dict).get('value'))


def list_page(page, ended):
    # Only the first live page has a digest; it lacks the structured cursor.
    if not ended and 'tasks' in page:
        first = require(page['tasks'], dict)
        read_on = page['read_on']
        after = None
        if read_on is not None:
            try:
                words = shlex.split(require(read_on, str))
            except ValueError:
                raise BadPage from None
            if len(words) < 4 or words[:2] != ['task', 'list'] or words[-2] != '--after':
                raise BadPage
            # Never execute the printed call. Only its hex cursor enters our argv.
            after = cursor_value(words[-1])
        if require(first['more'], bool) and after is None:
            raise BadPage
        return require(first['rows'], list), after
    if ended:
        if page['ended'] == 'none':
            return [], None
        group = require(require(page['ended'], dict)['rows'], dict)
        return require(group['rows'], list), cursor(group['after'])
    return require(page['live'], list), cursor(page['live_after'])


@dataclass
class Task:
    last_activity: int
    ended_at: object


def task_lists(command, projects):
    tasks = {}
    try:
        for project in dict.fromkeys(projects):
            for ended in (False, True):
                words = ['task', 'list', '--project', project] + (['--ended'] if ended else [])
                seen = set()
                after = None
                while True:
                    page = ctl_page(command, words + (['--after', after] if after else []))
                    rows, after = list_page(page, ended)
                    for row in rows:
                        row = require(row, dict)
                        number = require(require(row['chip'], dict)['task'], int)
                        last = require(row['last_activity'], int)
                        end = row['ended_at']
                        if ended:
                            require(end, int)
                        elif end is not None:
                            raise BadPage
                        previous = tasks.get(number)
                        if previous:
                            last = max(last, previous.last_activity)
                            if end is None and previous.ended_at is not None:
                                end = previous.ended_at
                        tasks[number] = Task(last, end)
                    if after is None:
                        break
                    if after in seen:
                        raise BadPage
                    seen.add(after)
    except (BadPage, KeyError):
        raise Refusal('task list unreadable') from None
    return tasks


@dataclass
class Row:
    at: int
    event: dict
    archive: object
    raw: dict


def activity(command, task, incomplete):
    rows = {}
    seen = set()
    before = None
    any_cut = False
    while True:
        words = ['task', 'activity', str(task)] + (['--before', before] if before else [])
        page = ctl_page(command, words)
        cut = require(page['cut'], bool)
        before = cursor(page['before'])
        for value in require(page['rows'], list):
            value = require(value, dict)
            position, at = require(value['position'], int), require(value['at'], int)
            owner = require(require(value['event'], dict)['owner'], dict)
            source = require(owner['source'], dict)
            if (position in rows or require(source['position'], int) != position
                    or require(source['at'], int) != at or require(source['task'], int) != task):
                raise BadPage
            event = require(source['line'], dict)
            if len(event) != 1:
                raise BadPage
            for kind in ('turn', 'run_began', 'run_ended'):
                if kind in event:
                    require(require(event[kind], dict)['attempt'], int)
            rows[position] = Row(at, event, owner['decoded'], value)
        incomplete['page cut'] += cut
        any_cut |= cut
        if before is None:
            return [rows[position] for position in sorted(rows)], not any_cut
        if before in seen:
            raise BadPage
        seen.add(before)


def part_text(part):
    """Turn parts in ctl JSON carry their omitted byte count (design, 3.1)."""
    if isinstance(part, dict) and part.get('from') == 0 and part.get('bytes_left') == 0:
        text = part.get('text')
        if isinstance(text, str):
            return text
    return None


def call_input(call):
    text = part_text(call.get('input'))
    if text is not None:
        try:
            args = load_json(text)
            if isinstance(args, dict):
                return args
        except (ValueError, RecursionError):
            pass
    return None


def decoded_turn(archive):
    """Return the turn's status and its assistant messages' calls in block order."""
    try:
        turn = require(require(require(archive, dict)['turn'], dict)['decoded'], dict)
        if require(turn['cut'], bool):
            return 'turn cut', ()
        completions = []
        for message in require(turn['messages'], list):
            if require(require(message, dict)['speaker'], str) != 'assistant':
                continue
            calls = []
            for block in require(message['blocks'], list):
                if 'call' in require(block, dict):
                    call = require(block['call'], dict)
                    require(call['tool'], str)
                    calls.append((call, call_input(call)))
            completions.append(calls)
    except (BadPage, KeyError):
        return 'turn unread', ()
    return None, completions


def tag(value):
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and len(value) == 1:
        return next(iter(value))
    return UNKNOWN


def members(value):
    """Walk JSON fields without recursing through deployment data."""
    pending = [(None, value)]
    while pending:
        key, item = pending.pop()
        yield key, item
        if isinstance(item, dict):
            pending.extend(item.items())
        elif isinstance(item, list):
            pending.extend((None, entry) for entry in item)


def numbers_in(value):
    for _, item in members(value):
        if unsigned(item):
            yield item
        elif isinstance(item, str):
            for match in re.finditer(UINT, item):
                if len(match[0]) <= 20:
                    yield int(match[0])


def words_in(event):
    """Message numbers and their kinds, from sent, heard and notified rows."""
    for key in ('words', 'heard', 'notified'):
        value = event.get(key)
        if isinstance(value, dict):
            if key == 'notified':
                value = value.get('word')
            if isinstance(value, dict):
                number = value.get('message' if key == 'words' else 'number')
                if unsigned(number):
                    yield number, tag(value.get('kind'))


JOB_START = re.compile(r'Started as job ([0-9]{1,20})(?=;|\s|$)', re.IGNORECASE)
JOB_END = ('exited', 'exit_nonzero', 'signalled', 'command_timed_out', 'stopped', 'cancelled')
JIG_ANSWERS = {'read_task': 'read', 'message': 'sent', 'subscribe': 'subscribed', 'mark_read': 'marked'}
PATH_TOOLS = {'read', 'list', 'search', 'write', 'edit', 'remove'}


def references(tool, args):
    if tool in PATH_TOOLS and 'path' in args:
        yield 'path', args['path']
    if tool == 'read' and 'offset' in args:
        yield 'offset', args['offset']
    if tool == 'job' and 'job' in args:
        yield 'job', args['job']
    if tool in ('read_task', 'message', 'subscribe'):
        key = 'task' if tool == 'read_task' else 'target'
        if key in args:
            yield 'task', args[key]
    if tool == 'message' and args.get('form') == 'answer' and 'question' in args:
        yield 'question', args['question']
    if tool == 'mark_read' and 'messages' in args:
        messages = args['messages']
        for number in messages if isinstance(messages, list) else (None,):
            yield 'message', number


@dataclass
class History:
    whole: bool
    numbers: set
    questions: set = field(default_factory=set)
    messages: set = field(default_factory=set)
    jobs: dict = field(default_factory=lambda: defaultdict(set))
    consumed: dict = field(default_factory=lambda: defaultdict(set))
    job_gaps: set = field(default_factory=set)

    def row(self, row):
        self.numbers.update(numbers_in(row.raw))
        for key, item in members(row.event):
            if unsigned(item):
                if key == 'question':
                    self.questions.add(item)
                elif key == 'message':
                    self.messages.add(item)
        for number, kind in words_in(row.event):
            self.messages.add(number)
            if kind == 'question':
                self.questions.add(number)

    def call(self, call, args, attempt):
        outcome = JOB_START.match(part_text(call.get('outcome')) or '')
        if call['tool'] == 'shell':
            if outcome:
                self.jobs[int(outcome[1])].add(attempt)
            elif call.get('answer') == 'running':
                self.job_gaps.add(attempt)
        if call['tool'] == 'job' and args is not None and call.get('answer') in JOB_END:
            number = args.get('job')
            if unsigned(number):
                self.consumed[attempt].add(number)

    def classify(self, ref, number, attempt, at, tasks):
        absent = 'unissued' if self.whole else UNKNOWN
        if ref in ('path', 'offset') or not unsigned(number):
            return UNKNOWN
        if ref == 'job':
            issued = self.jobs.get(number, ())
            if attempt in issued:
                return 'consumed' if number in self.consumed[attempt] else UNKNOWN
            if attempt in self.job_gaps:
                return UNKNOWN
            if issued:
                return 'before_opening'
            return UNKNOWN if self.job_gaps else absent
        if ref == 'task':
            if number not in self.numbers:
                return absent
            if number not in tasks:
                return UNKNOWN
            end = tasks[number].ended_at
            return 'ended' if end is not None and end <= at else 'live'
        if ref == 'question':
            return 'live' if number in self.questions else UNKNOWN
        return 'live' if number in self.messages else absent


def shape_call(call, args, history, attempt, at, tasks, vocabulary):
    tool = call['tool']
    answer = call.get('answer', UNKNOWN)
    if answer == 'unknown_tool':
        tool = UNKNOWN
    elif tool in ('delegated', UNKNOWN) or tool not in vocabulary.fields['tool']:
        tool = 'delegated'
    if answer is None:
        answer = 'unanswered'
    elif answer == 'delegated' and tool in JIG_ANSWERS:
        answer = JIG_ANSWERS[tool] if call.get('failed') is False else UNKNOWN
    refs = tuple((name, vocabulary.word('class', history.classify(name, value, attempt, at, tasks)))
                 for name, value in references(tool, args))
    return vocabulary.word('tool', tool), vocabulary.word('answer', answer), refs


def run_words(attempt, ends, begins, last_attempt, wakes, ended, whole, vocabulary):
    why = begins.get(attempt + 1)
    if why is None:
        begin = 'none' if ended and whole and attempt == last_attempt else UNKNOWN
    else:
        begin = vocabulary.word('begin', tag(why))
    if begin == 'message':
        message = why['message'] if isinstance(why, dict) else None
        number = message.get('message') if isinstance(message, dict) else None
        wake = wakes.get(number, UNKNOWN) if unsigned(number) else UNKNOWN
    else:
        wake = UNKNOWN if begin == UNKNOWN else 'none'
    return (vocabulary.word('end', ends.get(attempt, UNKNOWN)), vocabulary.word('begin', begin),
            vocabulary.word('wake', wake))


@dataclass
class Deployment:
    tasks: int = 0
    turns: int = 0
    completions: int = 0
    incomplete: Counter = field(default_factory=Counter)
    shapes: Counter = field(default_factory=Counter)

    def shape_task(self, task, rows, whole, tasks, args, vocabulary):
        """Read ahead for run endings and whole-history evidence, then shape in row order."""
        ends, begins, wakes = {}, {}, {}
        last_attempt = 0
        turns = []
        for row in rows:
            wakes.update(words_in(row.event))
            if 'run_ended' in row.event:
                event = row.event['run_ended']
                ends[event['attempt']] = tag(event.get('end'))
            elif 'run_began' in row.event:
                event = row.event['run_began']
                begins[event['attempt']] = event.get('why', UNKNOWN)
            elif 'turn' in row.event:
                event = row.event['turn']
                problem, completions = decoded_turn(row.archive)
                if problem or any(body is None for calls in completions for _, body in calls):
                    whole = False
                turns.append((problem, completions))
            else:
                continue
            last_attempt = max(last_attempt, event['attempt'])

        history = History(whole, {task})
        decoded = iter(turns)
        for row in rows:
            if 'turn' in row.event:
                problem, completions = next(decoded)
                inside = args.start <= row.at < args.stop
                if inside:
                    self.turns += 1
                if problem:
                    if inside:
                        self.incomplete[problem] += 1
                else:
                    attempt = row.event['turn']['attempt']
                    run = run_words(attempt, ends, begins, last_attempt, wakes, tasks[task].ended_at is not None,
                                    whole, vocabulary)
                    for calls in completions:
                        usable = all(body is not None for _, body in calls)
                        if inside:
                            self.completions += 1
                            self.incomplete['input unread'] += not usable
                        shaped = []
                        for call, body in calls:
                            if inside and usable:
                                shaped.append(shape_call(call, body, history, attempt, row.at, tasks, vocabulary))
                            history.call(call, body, attempt)
                        if inside and usable:
                            self.shapes[Shape(tuple(shaped), run)] += 1
            history.row(row)


def read_deployment(args, vocabulary):
    tasks = task_lists(args.ctl, args.project)
    deployment = Deployment()
    for task, item in tasks.items():
        if item.last_activity < args.start:
            continue
        deployment.tasks += 1
        try:
            rows, whole = activity(args.ctl, task, deployment.incomplete)
        except (BadPage, KeyError):
            deployment.incomplete['task unread'] += 1
            continue
        deployment.shape_task(task, rows, whole, tasks, args, vocabulary)
    return deployment


def compare(shapes, worlds):
    by_signature = defaultdict(set)
    for shape in worlds:
        by_signature[shape.signature()].add(shape.compared_words())
    missed = Counter()
    through_unknown = 0
    for shape, count in shapes.items():
        candidates = by_signature.get(shape.signature(), ())
        words = shape.compared_words()
        if words in candidates:
            continue
        if any(all(a == b or UNKNOWN in (a, b) for a, b in zip(words, candidate)) for candidate in candidates):
            through_unknown += count
        else:
            missed[shape.text()] = count
    return missed, through_unknown


def report(vocabulary, deployment, samples):
    cases = sum(sample.cases for sample in samples.values())
    worlds = {shape for sample in samples.values() for shape in sample.shapes}
    cuts = sum(sample.beyond > 0 for sample in samples.values())
    missed, through_unknown = compare(deployment.shapes, worlds)
    incomplete = ', '.join(f'{key} {deployment.incomplete[key]}'
                           for key in ('task unread', 'turn unread', 'turn cut', 'input unread', 'page cut'))
    yield f'shapes: {vocabulary.version}'
    yield (f'deployment: tasks {deployment.tasks}, turns {deployment.turns}, completions {deployment.completions}, '
           f'shaped {sum(deployment.shapes.values())}')
    yield f'incomplete: {incomplete}'
    yield f'worlds: samples {len(samples)}, cases {cases}, shapes {len(worlds)}, samples cut {cuts}'
    yield f'missed: {len(missed)} shapes in {sum(missed.values())} completions'
    for shape in sorted(missed, key=lambda shape: (-missed[shape], shape)):
        yield f'{missed[shape]} {shape}'
    yield f'reached through unknown: {through_unknown} completions'
    for sweep, sample in sorted(samples.items()):
        yield f'sample {sweep} cases {sample.cases} distinct {len(sample.shapes)} beyond {sample.beyond}'


def main(argv):
    args = arguments(argv)
    try:
        vocabulary = Vocabulary.read(VOCABULARY)
        samples = read_worlds(args.worlds, vocabulary)
        deployment = read_deployment(args, vocabulary)
    except Refusal as error:
        print(f'shapes: {error}', file=sys.stderr)
        return 1
    print('\n'.join(report(vocabulary, deployment, samples)))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
