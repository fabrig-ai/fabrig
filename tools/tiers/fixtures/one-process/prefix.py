#!/usr/bin/env -S python3 -I -B
"""Record and exec the caller's prefix (tools/tiers/design.md, 11.10)."""
import json
import os
from pathlib import Path
import sys

assert 'TIERS_HOLD' not in os.environ, 'hold setting leaked to prefix'
with Path(__file__).with_name('prefix-argv.jsonl').open('a') as output:
    output.write(json.dumps(sys.argv[1:]) + '\n')
with Path(__file__).with_name('prefix-sessions').open('a') as output:
    output.write(str(os.getsid(0)) + '\n')
print('fixture prefix', flush=True)
os.execvp(sys.argv[1], sys.argv[1:])
