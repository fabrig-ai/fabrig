"""Settle in-flight status entries (temper/docs/design/protocol/tooling.md, 4.2)."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys


TAG = re.compile(rb'\s\[in flight: ([^\]\r\n]*)\][ \t\r]*$')
CHANGE = re.compile(rb'change ([0-9]+)')


def git(repository, *args):
    return subprocess.run(['git', *args], cwd=repository, check=True,
                          stdout=subprocess.PIPE).stdout


def parse_request(data):
    request = json.loads(data)
    if (not isinstance(request, dict) or set(request) != {'version', 'goal', 'changes'}
            or type(request['version']) is not int or request['version'] != 1):
        raise ValueError('expected a version 1 close request')
    goal = request['goal']
    if type(goal) is not int or goal <= 0:
        raise ValueError('goal must be a positive integer')
    if not isinstance(request['changes'], list):
        raise ValueError('changes must be a list')
    ends = {}
    for change in request['changes']:
        if not isinstance(change, dict) or set(change) != {'task', 'landed'}:
            raise ValueError('expected task and landed in each change')
        task = change['task']
        if type(task) is not int or task <= 0:
            raise ValueError('task must be a positive integer')
        if task in ends:
            raise ValueError(f'change {task} given twice')
        if type(change['landed']) is not bool:
            raise ValueError(f'change {task}: landed must be a bool')
        ends[task] = change['landed']
    return goal, ends


def file_lines(text):
    parts = text.split(b'\n')
    return [part + b'\n' for part in parts[:-1]] + ([parts[-1]] if parts[-1] else [])


def entries(text):
    lines = file_lines(text)
    found = []
    index = 0
    while index < len(lines):
        if not lines[index].startswith(b'- **Not yet:**'):
            index += 1
            continue
        start = index
        index += 1
        while index < len(lines) and lines[index].startswith(b'  ') and lines[index].strip():
            index += 1
        tag = TAG.search(lines[index - 1])
        found.append((start, index, tag[1] if tag is not None else None))
    return found


def settle(text, ends):
    lines = file_lines(text)
    unknown = []
    for start, end, tag in reversed(entries(text)):
        if tag is None:
            continue
        change = CHANGE.fullmatch(tag)
        task = int(change[1]) if change is not None else None
        if task not in ends:
            unknown.append((end, tag.decode('utf-8', errors='backslashreplace')))
        elif ends[task]:
            del lines[start:end]
        else:
            last = lines[end - 1]
            match = TAG.search(last)
            # Keep the last line's ending and the bytes after the tag.
            lines[end - 1] = last[:match.start()] + last[match.end(1) + 1:]
    return b''.join(lines), sorted(unknown)


def result_bytes(settled):
    def encode(result):
        return (json.dumps(result, ensure_ascii=True, separators=(',', ':')) + '\n').encode('utf-8')

    result = {'settled': settled}
    if len(encode(result)) <= 512:
        return encode(result)
    result = {'settled': [], 'more': len(settled)}
    for path in settled:
        result['settled'].append(path)
        result['more'] -= 1
        if len(encode(result)) > 512:
            result['settled'].pop()
            result['more'] += 1
            break
    return encode(result)


def main():
    if len(sys.argv) != 3 or sys.argv[1] != 'close':
        raise ValueError('usage: .temper/close close <request.json>')
    _, ends = parse_request(Path(sys.argv[2]).read_bytes())
    repository = Path(__file__).resolve().parents[2]
    result = Path(os.environ['TEMPER_JOB']) / 'result.json'
    changed = {}
    unknown = []
    paths = sorted(os.fsdecode(path) for path in git(repository, 'ls-files', '-z').split(b'\0')
                   if path)
    for path in paths:
        if 'docs' not in Path(path).parts[:-1] or not path.endswith('.status.md'):
            continue
        previous = (repository / path).read_bytes()
        updated, tags = settle(previous, ends)
        unknown.extend(f'{path}:{line}: unknown tag [in flight: {tag}]' for line, tag in tags)
        if updated != previous:
            changed[path] = (previous, updated)
    if unknown:
        raise ValueError('\n'.join(unknown))
    index = Path(os.fsdecode(git(repository, 'rev-parse', '--git-path', 'index')).strip())
    if not index.is_absolute():
        index = repository / index
    previous_index = index.read_bytes()
    try:
        for path, (_, updated) in changed.items():
            (repository / path).write_bytes(updated)
        result.write_bytes(result_bytes(list(changed)))
        if changed:
            git(repository, 'add', '--', *changed)
    except (OSError, subprocess.CalledProcessError):
        for path, (previous, _) in changed.items():
            (repository / path).write_bytes(previous)
        index.write_bytes(previous_index)
        raise
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'close: {error}', file=sys.stderr)
        sys.exit(1)
