"""Fail-stop in every profile (skein's programming-model.md, 2).

The dev profile unwinds, so a panic aborts a process only once
skein_fail_stop's hook is installed. Every binary of the workspace calls
`skein_fail_stop::install()` in each of its `main` functions, before any
statement: only `use` and `const` items and comments may come first.

    python3 -I -B scripts/fail_stop.py
"""
import json
import re
import subprocess
import sys

MAIN = re.compile(r'\bfn main\(\)[^{]*\{')
FIRST = re.compile(r'(?:\s+|//[^\n]*|(?:use|const)\s[^;]*;)*skein_fail_stop::install\(\);')

metadata = json.loads(subprocess.run(['cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked'],
                                     check=True, capture_output=True, text=True).stdout)
missing = []
for package in metadata['packages']:
    for target in package['targets']:
        if 'bin' not in target['kind']:
            continue
        with open(target['src_path'], encoding='utf-8') as source:
            text = source.read()
        starts = [found.end() for found in MAIN.finditer(text)]
        if not starts or not all(FIRST.match(text, start) for start in starts):
            missing.append(f"{package['name']} {target['name']}: {target['src_path']}")
if missing:
    print("Each binary's main installs skein_fail_stop's hook first (skein's programming-model.md, 2):",
          *missing, sep='\n  ', file=sys.stderr)
    sys.exit(1)
