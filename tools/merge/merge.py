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


def git(home, *args):
    return subprocess.run(['git', *args], cwd=home, check=True,
                          stdout=subprocess.PIPE).stdout


def unmerged(home):
    paths = {}
    for entry in git(home, 'ls-files', '--unmerged', '-z').split(b'\0'):
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


def settle(home, path, stages):
    if any(stage not in stages or stages[stage][0] not in (b'100644', b'100755')
           for stage in (2, 3)):
        return False
    head, head_counts, head_raised = baseline(git(home, 'cat-file', 'blob', stages[2][1]))
    _, base_counts, base_raised = baseline(git(home, 'cat-file', 'blob', stages[3][1]))
    if base_raised - head_raised:
        return False
    for index, line in enumerate(head):
        if line.startswith(b'#'):
            continue
        row = ROW.fullmatch(line)
        if row is not None:
            count = max(head_counts[row[1]], base_counts.get(row[1], 0))
            head[index] = row[1] + row[2] + str(count).encode('ascii') + row[4]
    target = home / path
    if target.is_symlink() or not target.is_file():
        return False
    previous = target.read_bytes()
    try:
        target.write_bytes(b''.join(head))
        subprocess.run(['python3', '-I', '-B', 'scripts/old_forms.py', '--write'],
                       cwd=target.parent.parent, check=True)
        git(home, 'add', '--', path)
    except (OSError, subprocess.CalledProcessError):
        target.write_bytes(previous)
        return False
    return True


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
    home = Path.cwd() / 'home'
    settled = []
    for path, stages in sorted(unmerged(home).items()):
        if path in BASELINES:
            try:
                if settle(home, path, stages):
                    settled.append(path)
            except (OSError, ValueError, subprocess.CalledProcessError) as error:
                print(f'{path}: {error}', file=sys.stderr)
    left = sorted(unmerged(home))
    result = {'settled': settled}
    if left:
        result['left'] = left
    (Path(os.environ['TEMPER_JOB']) / 'result.json').write_text(
        json.dumps(result, ensure_ascii=True) + '\n', encoding='utf-8')
    return 3 if left else 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'merge: {error}', file=sys.stderr)
        sys.exit(1)
