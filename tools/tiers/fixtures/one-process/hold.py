#!/usr/bin/env -S python3 -I -B
"""Record the hold's lifetime beside cargo's calls (tools/tiers/design.md, 11.10)."""
import json
import os
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
assert Path.cwd() == root
assert 'TIERS_HOLD' not in os.environ, 'hold setting leaked to hold'


def record(event):
    with (root / 'hold-events.jsonl').open('a') as output:
        output.write(json.dumps(event) + '\n')


record(['hold-start', sys.argv[1:], os.getpgrp(), os.getsid(0)])
if sys.argv[1:] == ['fail']:
    sys.exit(3)
print('held', flush=True)
sys.stdin.read()
logs = list((root / 'target/tiers-one-process/runs').glob('*/command-output.txt'))
summary = logs[0].read_text().splitlines()[-1] if logs else None
record(['hold-end', summary])
