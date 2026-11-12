"""Settle in-flight status entries (temper/docs/design/protocol/tooling.md, 4.2)."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys


TAG = re.compile(rb'\s\[in flight: ([^\]\r\n]*)\][.,;:!?]*[ \t\r]*$')
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
    labels = []
    index = 0
    while index < len(lines):
        if not lines[index].startswith(b'- **Not yet:**'):
            index += 1
            continue
        start = index
        index += 1
        while index < len(lines) and lines[index].startswith(b'  ') and lines[index].strip():
            index += 1
        if lines[start].strip() == b'- **Not yet:**':
            children = [at for at in range(start + 1, index) if lines[at].startswith(b'  - ')]
            if children:
                labels.append((start, index))
            bounds = list(zip(children, children[1:] + [index]))
        else:
            bounds = [(start, index)]
        for first, end in bounds:
            tag = TAG.search(lines[end - 1])
            found.append((first, end, tag[1] if tag is not None else None))
    return found, labels


def settle(text, ends):
    lines = file_lines(text)
    kept = []
    found, labels = entries(text)
    for start, end, tag in found:
        if tag is None:
            continue
        change = CHANGE.fullmatch(tag)
        task = int(change[1]) if change is not None else None
        if task not in ends:
            kept.append((end, tag.decode('utf-8', errors='backslashreplace')))
        elif ends[task]:
            # Keep offsets stable until empty labels have been checked.
            lines[start:end] = [b''] * (end - start)
        else:
            last = lines[end - 1]
            match = TAG.search(last)
            # Keep the last line's ending and the bytes after the tag.
            lines[end - 1] = last[:match.start()] + last[match.end(1) + 1:]
    for start, end in labels:
        if not any(lines[start + 1:end]):
            lines[start] = b''
    return b''.join(lines), sorted(kept)


def result_bytes(settled, kept):
    def encode(result):
        return (json.dumps(result, ensure_ascii=True, separators=(',', ':')) + '\n').encode('utf-8')

    result = {'settled': settled}
    if kept:
        result['kept'] = kept
    encoded = encode(result)
    if len(encoded) <= 512:
        return encoded
    result = {group: [] for group in result}
    groups = [('settled', settled, 'more'), ('kept', kept, 'kept_more')]
    for _, items, counter in groups:
        if items:
            result[counter] = len(items)
    for group, counter, item in ((group, counter, item)
                                 for group, items, counter in groups for item in items):
        result[group].append(item)
        result[counter] -= 1
        if not result[counter]:
            del result[counter]
        if len(encode(result)) > 512:
            result[group].pop()
            result[counter] = result.get(counter, 0) + 1
            break
    return encode(result)


def main():
    if len(sys.argv) != 3 or sys.argv[1] != 'close':
        raise ValueError('usage: .temper/close close <request.json>')
    _, ends = parse_request(Path(sys.argv[2]).read_bytes())
    repository = Path(__file__).resolve().parents[2]
    result = Path(os.environ['TEMPER_JOB']) / 'result.json'
    changed = {}
    kept = []
    paths = sorted(os.fsdecode(path) for path in git(repository, 'ls-files', '-z').split(b'\0')
                   if path)
    for path in paths:
        if 'docs' not in Path(path).parts[:-1] or not path.endswith('.status.md'):
            continue
        previous = (repository / path).read_bytes()
        updated, tags = settle(previous, ends)
        kept.extend(f'{path}:{line}: [in flight: {tag}]' for line, tag in tags)
        if updated != previous:
            changed[path] = (previous, updated)
    index = Path(os.fsdecode(git(repository, 'rev-parse', '--git-path', 'index')).strip())
    if not index.is_absolute():
        index = repository / index
    previous_index = index.read_bytes()
    try:
        for path, (_, updated) in changed.items():
            (repository / path).write_bytes(updated)
        result.write_bytes(result_bytes(list(changed), kept))
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
