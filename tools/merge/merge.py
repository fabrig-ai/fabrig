"""Settle known conflicts (temper/docs/design/protocol/tooling.md, 4.1)."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys


BASELINES = {f'{tree}/scripts/old-forms.txt'
             for tree in ('skein', 'smith', 'jig', 'temper', 'hearth')}
KNOWN_REDS = '.temper/known-reds.md'
# Entry patterns and cap: tools/pre-pr/design.md, 5.
RED_TEST = re.compile(r'- test (\S+) (.+)')
RED_LINT = re.compile(r'- lint (\S+) (\S+) (.+)')
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


def known_red_key(line):
    line = line.rstrip()
    return line if RED_TEST.fullmatch(line) or RED_LINT.fullmatch(line) else None


def merged_known_reds(merge_base: str, head: str, incoming: str) -> str:
    """Merge entries, keeping incoming prose (temper/docs/design/protocol/tooling.md, 4.1)."""
    merge_base_keys = {known_red_key(line) for line in merge_base.splitlines()} - {None}
    head_keys = list(dict.fromkeys(key for line in head.splitlines()
                                  if (key := known_red_key(line)) is not None))
    incoming_lines = incoming.splitlines(keepends=True)
    incoming_keys = {known_red_key(line) for line in incoming_lines} - {None}
    removed = (merge_base_keys & incoming_keys) - set(head_keys)
    added = [key + '\n' for key in head_keys
             if key not in merge_base_keys and key not in incoming_keys]
    insertion = max((index + 1 for index, line in enumerate(incoming_lines)
                     if known_red_key(line) is not None), default=len(incoming_lines))
    before = [line for line in incoming_lines[:insertion] if known_red_key(line) not in removed]
    # splitlines recognises the delivery check's line endings, not only LF.
    if added and before and before[-1].splitlines() == [before[-1]]:
        before[-1] += '\n'
    return ''.join(before + added + incoming_lines[insertion:])


def known_reds(repository, path, stages):
    """Settle or leave with one reason (temper/docs/design/protocol/tooling.md, 4.1)."""
    target = repository / path
    previous = None
    try:
        if 2 not in stages:
            raise ValueError("missing on the head's side")
        if 3 not in stages:
            raise ValueError('missing on the incoming side')
        sides = {}
        for stage, side in ((1, 'merge base'), (2, 'head'), (3, 'incoming')):
            if stage not in stages:
                sides[stage] = ''
                continue
            mode, blob = stages[stage]
            if mode not in (b'100644', b'100755'):
                raise ValueError(f'{side} stage is not a regular file')
            sides[stage] = git(repository, 'cat-file', 'blob', blob,
                               stderr=subprocess.PIPE).decode('utf-8')
        merged = merged_known_reds(sides[1], sides[2], sides[3])
        entries = sum(known_red_key(line) is not None for line in merged.splitlines())
        data = merged.encode('utf-8')
        if entries > 64 or len(data) > 8192:
            raise ValueError(f'the merged file holds {entries:,} entries, {len(data):,} bytes; '
                             'the cap is 64 entries and 8,192 bytes')
        if target.is_symlink() or not target.is_file():
            raise ValueError('work tree file is not a regular file')
        previous = target.read_bytes()
        target.write_bytes(data)
        git(repository, 'add', '--', path, stderr=subprocess.PIPE)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        reason = str(error)
        if previous is not None:
            try:
                target.write_bytes(previous)
            except OSError as restore_error:
                reason += f'; restoring conflicted bytes failed: {restore_error}'
        print(f'{path}: left: {reason}', file=sys.stderr)
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
    conflicts = unmerged(repository)
    for path, stages in sorted(conflicts.items()):
        try:
            if path in BASELINES and settle(repository, path, stages):
                settled.append(path)
            elif path == KNOWN_REDS and known_reds(repository, path, stages):
                settled.append(path)
            elif path in RECORDINGS and regenerate(repository, path, stages, conflicts):
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
