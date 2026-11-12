#!/usr/bin/env -S python3 -I -B
"""Offline nextest listing and cargo metadata (tools/tiers/design.md, 11.10)."""
import json
import os
from pathlib import Path
import sys
import time

root = Path(__file__).resolve().parent
assert 'TIERS_HOLD' not in os.environ, 'hold setting leaked to cargo'
with (root / 'hold-events.jsonl').open('a') as output:
    output.write(json.dumps(['cargo', sys.argv[1:], os.getpid()]) + '\n')
if os.environ.get('FIXTURE_HOLD_BLOCK'):
    time.sleep(30)
with (root / 'cargo-argv.jsonl').open('a') as output:
    output.write(json.dumps(sys.argv[1:]) + '\n')
if sys.argv[1:3] == ['nextest', 'list']:
    print('fixture listing prefix', flush=True)
    print((root / 'listing.json').read_text(), end='', flush=True)
    print('fixture listing suffix', flush=True)
elif sys.argv[1:] == ['metadata', '--format-version', '1', '--no-deps']:
    print((root / 'metadata.json').read_text(), end='')
else:
    sys.exit('unexpected fake cargo arguments')
