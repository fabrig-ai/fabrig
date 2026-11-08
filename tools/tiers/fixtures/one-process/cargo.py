#!/usr/bin/env -S python3 -I -B
"""Offline nextest listing and cargo metadata (tools/tiers/design.md, 11.10)."""
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parent
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
