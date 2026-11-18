#!/usr/bin/env python3
"""Refuse raised figures (tools/tiers/budgets.md, 10.1).

Usage: python3 -I -B tools/tiers/figures.py --base BASE [--head HEAD] [--merged REV]…
Reads committed trees from the current repository; exits 0 passing, 1 refusing,
2 on invalid input. The budgets file's form belongs to cpu.py's budgets().
"""

import argparse
from collections import Counter
import math
from pathlib import Path
import runpy
import subprocess
import sys
import tomllib

CPU = runpy.run_path(str(Path(__file__).with_name('cpu.py')))
GROUPS = CPU['GROUPS']
REPAIR_LINE = CPU['REPAIR_LINE']
BUDGETS = 'tools/tiers/budgets.toml'
LIMITS = ('cpu_suite_s', 'cpu_test_s', 'wall_hard_s')


def git(*args):
    return subprocess.run(('git', *args), check=True, capture_output=True, text=True).stdout


def read_table(revision, path):
    # ls-tree distinguishes an absent path from any other git show failure.
    if not git('ls-tree', revision, '--', path):
        return {}
    try:
        return tomllib.loads(git('show', f'{revision}:{path}'))
    except tomllib.TOMLDecodeError as error:
        raise ValueError(f'{revision}:{path}: {error}') from error


def number(table, key, place):
    if key not in table:
        return None
    value = table[key]
    if type(value) not in (int, float) or not math.isfinite(value):
        raise ValueError(f'{place}: expected a finite number')
    return value


def profile_figures(table, side, profile):
    """Read only the figure places, keeping absent bounds distinct from zero."""
    label = f'{side} gate.{profile}'
    references = {layer: number(table['references'], layer, f'{label}.references.{layer}')
                  for layer in table.get('references', {})}
    ideals, rows = {}, {}
    normalized = dict(table)
    for group in GROUPS:
        if group not in table:
            continue
        values = table[group]
        ideals[group] = number(values, 'ideal_s', f'{label}.{group}.ideal_s')
        normalized[group] = {}
        for layer, row in values.items():
            if layer == 'ideal_s':
                continue
            place = f'{label}.{group}.{layer}'
            figures = {key: number(row, key, f'{place}.{key}')
                       for key in ('cpu_s', 'reference_s', 'ideal_s')}
            rows[group, layer] = figures
            normalized[group][layer] = {'cpu_s': figures['cpu_s'] or 0}
    moves = Counter()
    for index, move in enumerate(table.get('moves', [])):
        seconds = number(move, 'cpu_s', f'{label}.moves[{index}].cpu_s')
        if seconds is None:
            raise ValueError(f'{label}.moves[{index}].cpu_s: expected a finite number')
        moves[move['from'], move['to'], seconds, move['why']] += 1
    reasons = {}
    for index, line in enumerate(table.get('reasons', [])):
        seconds = number(line, 'cpu_s', f'{label}.reasons[{index}].cpu_s')
        if seconds is None:
            raise ValueError(f'{label}.reasons[{index}].cpu_s: expected a finite number')
        key = line['binary'], line['test']
        reasons[key] = max(reasons.get(key, seconds), seconds)
    return dict(ceilings=CPU['gate_ceilings'](normalized),
                total=sum(row['cpu_s'] or 0 for row in rows.values()),
                references=references, ideals=ideals, rows=rows, moves=moves, reasons=reasons)


def down(source, target, layers):
    parts = [endpoint.split('.') for endpoint in (source, target)]
    if any(len(p) != 2 or p[0] not in GROUPS[:-1] or p[1] not in layers for p in parts):
        return False
    return layers.index(parts[1][1]) < layers.index(parts[0][1])


def text(value):
    return 'none' if value is None else str(value)


def compare(base, head, layers, base_name, head_name):
    findings = []

    def raised(place, old, new, *, ceiling=False):
        before, after = (old or 0, new or 0) if ceiling else (old, new)
        if before is not None and (after is None or round(after, 3) > round(before, 3)):
            findings.append((place, f'figure raised: {place}: {text(old)} to {text(new)}'))

    before, after = base.get('gate', {}), head.get('gate', {})
    for profile in sorted(before.keys() | after.keys()):
        old = profile_figures(before.get(profile, {}), base_name, profile)
        new = profile_figures(after.get(profile, {}), head_name, profile)
        excused = {}
        for (source, target, seconds, _), count in (new['moves'] - old['moves']).items():
            if down(source, target, layers):
                key = tuple(target.split('.'))
                excused[key] = excused.get(key, 0) + seconds * count
        start = len(findings)
        for group, layer in sorted(old['ceilings'].keys() | new['ceilings'].keys()):
            key = group, layer
            ceiling = new['ceilings'].get(key)
            adjusted = None if ceiling is None else ceiling - excused.get(key, 0)
            raised(f'{profile} {group} {layer} ceiling', old['ceilings'].get(key), adjusted, ceiling=True)
        if len(findings) == start:
            raised(f"{profile} ceilings' sum", old['total'], new['total'])
        for layer in sorted(old['references'].keys() | new['references'].keys()):
            raised(f'{profile} references {layer}', old['references'].get(layer), new['references'].get(layer))
        for (group, layer), row in sorted(new['rows'].items()):
            previous = old['rows'].get((group, layer), {})
            raised(f'{profile} {group} {layer} reference',
                   previous.get('reference_s') if previous.get('reference_s') is not None else old['references'].get(layer),
                   row['reference_s'] if row['reference_s'] is not None else new['references'].get(layer))
        for group in sorted(old['ideals'].keys() | new['ideals'].keys()):
            raised(f'{profile} {group} ideal', old['ideals'].get(group), new['ideals'].get(group))
        for (group, layer), row in sorted(new['rows'].items()):
            raised(f'{profile} {group} {layer} ideal',
                   old['rows'].get((group, layer), {}).get('ideal_s'), row['ideal_s'])
        for (binary, test), seconds in sorted(new['reasons'].items()):
            place = f'{profile} reason {binary} {test}'
            previous = old['reasons'].get((binary, test))
            if previous is None:
                findings.append((place, f'figure added: {place}: {seconds}'))
            else:
                raised(place, previous, seconds)
    for pair in sorted((base.keys() | head.keys()) - {'gate', 'tools'}):
        for key in LIMITS:
            raised(f'{pair} {key}', number(base.get(pair, {}), key, f'{base_name} {pair}.{key}'),
                   number(head.get(pair, {}), key, f'{head_name} {pair}.{key}'))
    before, after = base.get('tools', {}), head.get('tools', {})
    for suite in sorted(before.keys() | after.keys()):
        for key in LIMITS:
            raised(f'tools {suite} {key}', number(before.get(suite, {}), key, f'{base_name} tools.{suite}.{key}'),
                   number(after.get(suite, {}), key, f'{head_name} tools.{suite}.{key}'))
    return findings


def compare_revision(revision, head):
    """A short-cut comparison names no figure places."""
    paths = git('diff', '--no-renames', '--name-only', revision, head).splitlines()
    if BUDGETS not in paths:
        print(f'figures: {BUDGETS} unchanged since {revision}')
        return None
    if paths == [BUDGETS]:
        print(f"figures: {BUDGETS} alone changed since {revision}: the user's figures")
        return None
    return compare(read_table(revision, BUDGETS), read_table(head, BUDGETS),
                   read_table(head, 'tools/tiers/layers.toml')['layers'], revision, head)


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    parser.add_argument('--head', default='HEAD')
    parser.add_argument('--merged', action='append')
    args = parser.parse_args(argv)
    try:
        findings = compare_revision(args.base, args.head)
        if findings is None and not args.merged:
            return 0
        findings = findings or []
        for revision in args.merged or []:
            places = {place for place, _ in compare_revision(revision, args.head) or []}
            findings = [(place, line) for place, line in findings if place in places]
    except (subprocess.CalledProcessError, OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print(f'figures: {getattr(error, "stderr", None) or error}', file=sys.stderr)
        return 2
    for _, line in findings:
        print(line)
    since = ' and '.join([args.base, *(args.merged or [])])
    print(f'figures: {len(findings)} raised or added since {since}')
    if findings:
        print(REPAIR_LINE)
    return 1 if findings else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
