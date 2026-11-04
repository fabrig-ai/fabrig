"""Recount conflicted baselines (temper/docs/design/protocol/tooling.md, 4.1)."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys


BASELINES = {f'{tree}/scripts/old-forms.txt'
             for tree in ('skein', 'smith', 'jig', 'temper')}
ROW = re.compile(rb'(\S+)([ \t]+)([0-9]+)([ \t]*(?:\r?\n)?)')


def git(repository, *args):
    return subprocess.run(['git', *args], cwd=repository, check=True,
                          stdout=subprocess.PIPE).stdout


def unmerged(repository):
    paths = {}
    for entry in git(repository, 'ls-files', '--unmerged', '-z').split(b'\0'):
        if entry:
            metadata, path = entry.split(b'\t', 1)
            mode, blob, stage = metadata.split()
            paths.setdefault(os.fsdecode(path), {})[int(stage)] = (mode, blob)
    return paths


def baseline(data):
    counts = {}
    raised = set()
    lines = data.splitlines(keepends=True)
    for line in lines:
        if line.startswith(b'# Raised:'):
            raised.add(line.rstrip(b'\r\n'))
        if not line.strip() or line.startswith(b'#'):
            continue
        row = ROW.fullmatch(line)
        if row is None or row[1] in counts:
            raise ValueError('invalid baseline row')
        counts[row[1]] = int(row[3])
    return lines, counts, raised


def settle(repository, path, stages):
    if any(stage not in stages or stages[stage][0] not in (b'100644', b'100755')
           for stage in (2, 3)):
        return False
    head, head_counts, head_raised = baseline(git(repository, 'cat-file', 'blob', stages[2][1]))
    _, base_counts, base_raised = baseline(git(repository, 'cat-file', 'blob', stages[3][1]))
    if base_raised - head_raised:
        return False
    for index, line in enumerate(head):
        if line.startswith(b'#'):
            continue
        row = ROW.fullmatch(line)
        if row is not None:
            count = max(head_counts[row[1]], base_counts.get(row[1], 0))
            head[index] = row[1] + row[2] + str(count).encode('ascii') + row[4]
    target = repository / path
    if target.is_symlink() or not target.is_file():
        return False
    previous = target.read_bytes()
    try:
        target.write_bytes(b''.join(head))
        subprocess.run(['python3', '-I', '-B', 'scripts/old_forms.py', '--write'],
                       cwd=target.parent.parent, check=True)
        git(repository, 'add', '--', path)
    except (OSError, subprocess.CalledProcessError):
        target.write_bytes(previous)
        return False
    return True


def result_bytes(settled, left):
    def encode(result):
        return (json.dumps(result, ensure_ascii=True, separators=(',', ':')) + '\n').encode('utf-8')

    result = {'settled': settled}
    if left:
        result['left'] = left
    encoded = encode(result)
    if len(encoded) <= 512:
        return encoded
    paths = sorted((path, group) for group in result for path in result[group])
    result = {group: [] for group in result}
    result['more'] = len(paths)
    for path, group in paths:
        result[group].append(path)
        result['more'] -= 1
        if len(encode(result)) > 512:
            result[group].pop()
            result['more'] += 1
            break
    return encode(result)


def main():
    if len(sys.argv) != 3 or sys.argv[1] != 'merge':
        raise ValueError('usage: .temper/merge merge <request.json>')
    request = json.loads(Path(sys.argv[2]).read_bytes())
    if not isinstance(request, dict) or request.get('version') != 1:
        raise ValueError('expected a version 1 merge request')
    for member in ('head', 'base'):
        commit = request.get(member)
        if not isinstance(commit, str) or re.fullmatch(r'(?:[0-9a-f]{40}|[0-9a-f]{64})', commit) is None:
            raise ValueError(f'invalid {member} commit')
    repository = Path(__file__).resolve().parents[2]
    settled = []
    for path, stages in sorted(unmerged(repository).items()):
        if path in BASELINES:
            try:
                if settle(repository, path, stages):
                    settled.append(path)
            except (OSError, ValueError, subprocess.CalledProcessError) as error:
                print(f'{path}: {error}', file=sys.stderr)
    left = sorted(unmerged(repository))
    (Path(os.environ['TEMPER_JOB']) / 'result.json').write_bytes(result_bytes(settled, left))
    return 3 if left else 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'merge: {error}', file=sys.stderr)
        sys.exit(1)
