"""Settle known conflicts (temper/docs/design/protocol/tooling.md, 4.1)."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys


BASELINES = {f'{tree}/scripts/old-forms.txt'
             for tree in ('skein', 'smith', 'jig', 'temper', 'hearth')}
ROW = re.compile(rb'(\S+)([ \t]+)([0-9]+)([ \t]*(?:\r?\n)?)')
RECORDINGS = {
    'temper/tests/playground/tests/fixtures/story-a.log': (
        'temper/tests/playground/tests/suite/walk.rs',
        ['cargo', 'nextest', 'run', '-p', 'temper-playground-world', '--test', 'suite',
         '-E', 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)']),
}
MARKER = re.compile(rb'^(?:<{7}|={7}|>{7}|\|{7})(?:[ \r\n]|$)', re.MULTILINE)


def git(repository, *args, stderr=None):
    return subprocess.run(['git', *args], cwd=repository, check=True,
                          stdout=subprocess.PIPE, stderr=stderr).stdout


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


def regenerate(repository, path, stages, conflicts):
    source, command = RECORDINGS[path]
    if source in conflicts or not (repository / source).is_file():
        print(f'{path}: generator source is missing or unmerged: {source}', file=sys.stderr)
        return False
    if any(stage not in stages or stages[stage][0] not in (b'100644', b'100755')
           for stage in (2, 3)):
        return False
    target = repository / path
    if target.is_symlink() or not target.is_file():
        return False
    previous = target.read_bytes()
    mode = target.stat().st_mode
    try:
        # Require a fresh output: success without writing cannot settle a file.
        target.unlink()
        subprocess.run(command, cwd=repository, check=True, stdin=subprocess.DEVNULL,
                       env={**os.environ, 'SKEIN_CODEGEN_WRITE': '1', 'TEMPER_INTERNAL_BUNDLE': 'none'})
        if target.is_symlink() or not target.is_file():
            raise ValueError('generator did not produce a regular file')
        generated = target.read_bytes()
        if not generated or MARKER.search(generated):
            raise ValueError('generator left an empty file or conflict markers')
        target.chmod(mode)
        git(repository, 'add', '--', path)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'{path}: {error}', file=sys.stderr)
        if target.is_symlink():
            target.unlink()
        target.write_bytes(previous)
        target.chmod(mode)
        return False
    return True


def history_change(repository, side, other, path, removed):
    commit = git(repository, '--literal-pathspecs', 'log', '-1', '-z', '--format=%H',
                 '--diff-filter=D' if removed else '--diff-filter=A', '--no-renames',
                 side, '^' + other, '--', path).split(b'\0', 1)[0].strip()
    if not commit:
        return None
    commit = commit.decode('ascii')
    changes = iter(git(repository, 'diff-tree', '-r', '-z', '-M', '--name-status',
                       commit + '^1', commit).split(b'\0'))
    for status in changes:
        if not status:
            continue
        source = os.fsdecode(next(changes))
        if status.startswith(b'R'):
            target = os.fsdecode(next(changes))
            if removed and source == path:
                return {'kind': 'renamed', 'to': target, 'commit': commit[:12]}
            if not removed and target == path:
                return {'kind': 'renamed', 'from': source, 'commit': commit[:12]}
        elif removed and status == b'D' and source == path:
            return {'kind': 'deleted', 'commit': commit[:12]}
    return None


def left_record(repository, path, stages, head, base):
    changes = {}
    for name, stage, side, other in (('head', 2, head, base), ('base', 3, base, head)):
        if (1 in stages) != (stage in stages):
            changes[name] = history_change(repository, side, other, path, stage not in stages)
        else:
            changes[name] = None
    record = {'path': path, 'kind': 'unknown'}
    renamed = [name for name, change in changes.items()
               if change is not None and change['kind'] == 'renamed']
    if len(renamed) == 2:
        return {'path': path, 'kind': 'renamed', 'by': 'both'}
    for name, other_stage, other in (('head', 3, 'base'), ('base', 2, 'head')):
        change = changes[name]
        if change is not None and other_stage in stages and changes[other] is None:
            record['kind'] = change['kind']
            record['by'] = name
            for member in ('from', 'to', 'commit'):
                if member in change:
                    record[member] = change[member]
            return record
    if not any(changes.values()):
        if set(stages) == {1, 2, 3}:
            record['kind'] = 'modified'
        elif set(stages) == {2, 3}:
            record['kind'] = 'added'
    return record


def result_bytes(settled, left):
    def encode(result):
        return (json.dumps(result, ensure_ascii=True, separators=(',', ':')) + '\n').encode('utf-8')

    result = {'settled': settled}
    if left:
        result['left'] = left
    encoded = encode(result)
    if len(encoded) <= 512:
        return encoded
    kept = {'settled': [], 'left': []}

    def bounded():
        result = {'settled': kept['settled']}
        if kept['left']:
            result['left'] = kept['left']
        omitted = {'left': len(left) - len(kept['left']),
                   'settled': len(settled) - len(kept['settled'])}
        if any(omitted.values()):
            result['more'] = {**omitted, 'read': 'git status --short; git log --name-status '
                                               '--diff-filter=DR HEAD...MERGE_HEAD'}
        return encode(result)

    for group, entries in (('left', left), ('settled', settled)):
        for entry in entries:
            kept[group].append(entry)
            if len(bounded()) > 512:
                kept[group].pop()
                break
    return bounded()


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
    conflicts = unmerged(repository)
    for path, stages in sorted(conflicts.items()):
        try:
            if path in BASELINES and settle(repository, path, stages):
                settled.append(path)
            elif path in RECORDINGS and regenerate(repository, path, stages, conflicts):
                settled.append(path)
        except (OSError, ValueError, subprocess.CalledProcessError) as error:
            print(f'{path}: {error}', file=sys.stderr)
    left = [left_record(repository, path, stages, request['head'], request['base'])
            for path, stages in unmerged(repository).items()]
    (Path(os.environ['TEMPER_JOB']) / 'result.json').write_bytes(result_bytes(settled, left))
    return 3 if left else 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(f'merge: {error}', file=sys.stderr)
        sys.exit(1)
