#!/usr/bin/env -S python3 -I -B
"""A child's standard descriptors alone (tools/tiers/design.md, 11.4)."""
import os
import sys

for name in os.listdir('/dev/fd'):
    descriptor = int(name)
    if descriptor <= 2:
        continue
    try:
        os.fstat(descriptor)
    except OSError:
        continue
    sys.exit(f'inherited descriptor {descriptor}')
