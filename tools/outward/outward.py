#!/usr/bin/env python3
"""The build-free outward scan (tools/outward/design.md, 2–4)."""
import os
from pathlib import Path
import re
import stat
import subprocess
import sys


LIST = 'tools/outward/names.txt'
OWN = 'tests/boundary/tests/outward.rs'
WORD = rb'[a-z0-9_]'


def names(text):
    """Compile the list in its matching order, with ASCII byte boundaries."""
    patterns, literals = [], []
    for number, line in enumerate(text.split(b'\n'), 1):
        if not line.isascii():
            raise ValueError(f'names.txt:{number}: malformed entry')
        if not line or line.startswith(b'#'):
            continue
        left, right, name = line[:1], line[2:3], line[4:]
        if (left not in (b'w', b'-') or right not in (b'w', b'-', b'd')
                or line[1:2] != b' ' or line[3:4] != b' ' or not name
                or name.startswith(b' ') or name.endswith(b' ')
                or any(byte < 0x20 or byte > 0x7e for byte in name)
                or name.lower() != name):
            raise ValueError(f'names.txt:{number}: malformed entry')
        before = rb'(?<!' + WORD + b')' if left == b'w' else b''
        after = rb'(?!' + WORD + b')' if right == b'w' else rb'[0-9]+' if right == b'd' else b''
        literals.append(re.escape(name))
        patterns.append(before + literals[-1] + after)
    # The literal-only prefilter lets the regex engine skip non-starting bytes.
    # An empty list matches nothing, not every empty position.
    return tuple(re.compile(b'|'.join(parts) if parts else rb'(?!)', re.IGNORECASE | re.ASCII)
                 for parts in (patterns, literals))


def left_out(path):
    parts = path.split('/')
    return path in (LIST, OWN) or (len(parts) >= 5 and parts[1:3] == ['docs', 'plan'])


def scan(pattern, prefilter):
    listed = subprocess.run(['git', 'ls-files', '-z'], capture_output=True, check=True).stdout
    findings = []
    for raw in sorted(set(listed.split(b'\0')) - {b''}):
        path = raw.decode('utf-8')
        if left_out(path):
            continue
        mode = os.lstat(raw).st_mode
        if stat.S_ISLNK(mode):
            text = os.readlink(raw)
        elif stat.S_ISREG(mode):
            text = Path(path).read_bytes()
        else:
            raise ValueError(f'{path}: neither a file nor a symlink')
        if b'\0' in text[:8000] or prefilter.search(text) is None:
            continue
        for number, line in enumerate(text.split(b'\n'), 1):
            for match in pattern.finditer(line):
                findings.append(f'{path}:{number}: {match[0].decode("ascii")}\n')
    return findings


def main():
    try:
        patterns = names(Path(__file__).with_name('names.txt').read_bytes())
        findings = scan(*patterns)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'outward: {error}', file=sys.stderr)
        return 2
    if findings:
        print("Files name the lab running fabrig's development (AGENTS.md, 1):")
        print(''.join(findings), end='')
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
