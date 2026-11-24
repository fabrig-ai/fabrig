#!/usr/bin/env python3
"""Refuse raised figures (tools/tiers/budgets.md, 10.1).

Usage: python3 -I -B tools/tiers/figures.py --base BASE [--head HEAD] [--merged REV]…
Reads committed trees from the current repository; exits 0 passing, 1 refusing,
2 on invalid input. The budgets file's form belongs to cpu.py's budgets().
"""

import argparse
from collections import Counter
import json
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
BUDGET_REASON = 'budget check reason added: '


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


def instructions(table, key, place):
    if key not in table:
        return None
    value = table[key]
    CPU['positive_instructions'](place, value)
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
            figures.update({key: instructions(row, key, f'{place}.{key}')
                            for key in ('instructions', 'ideal_instructions', 'accepted_instructions')})
            rows[group, layer] = figures
            normalized[group][layer] = {'cpu_s': figures['cpu_s'] or 0}
    moves = Counter()
    instruction_moves = {}
    for index, move in enumerate(table.get('moves', [])):
        place = f'{label}.moves[{index}]'
        seconds = number(move, 'cpu_s', f'{place}.cpu_s')
        if seconds is None:
            raise ValueError(f'{place}.cpu_s: expected a finite number')
        source, target, why = move['from'], move['to'], move['why']
        moves[source, target, seconds, why] += 1
        amount = instructions(move, 'instructions', f'{place}.instructions')
        if amount is not None:
            instruction_moves.setdefault((source, target, why), []).append(amount)
    reasons = {}
    for index, line in enumerate(table.get('reasons', [])):
        seconds = number(line, 'cpu_s', f'{label}.reasons[{index}].cpu_s')
        if seconds is None:
            raise ValueError(f'{label}.reasons[{index}].cpu_s: expected a finite number')
        key = line['binary'], line['test']
        reasons[key] = max(reasons.get(key, seconds), seconds)
    return dict(ceilings=CPU['gate_ceilings'](normalized),
                total=sum(row['cpu_s'] or 0 for row in rows.values()),
                references=references, ideals=ideals, rows=rows, moves=moves,
                instruction_moves=instruction_moves, reasons=reasons)


def down(source, target, layers):
    parts = [endpoint.split('.') for endpoint in (source, target)]
    if any(len(p) != 2 or p[0] not in GROUPS[:-1] or p[1] not in layers for p in parts):
        return False
    return layers.index(parts[1][1]) < layers.index(parts[0][1])


def text(value):
    return 'none' if value is None else str(value)


def compare(base, head, layers, base_name, head_name):
    findings = []

    def raised(place, old, new, *, ceiling=False, added=False):
        before, after = (old or 0, new or 0) if ceiling else (old, new)
        if old is None and new is not None and added:
            findings.append((place, f'figure added: {place}: {text(new)}'))
        elif before is not None and (after is None or round(after, 3) > round(before, 3)):
            findings.append((place, f'figure raised: {place}: {text(old)} to {text(new)}'))

    before, after = base.get('gate', {}), head.get('gate', {})
    for profile in sorted(before.keys() | after.keys()):
        previous = before.get(profile, {})
        old_reasons = previous.get('check_reasons', previous.get('coverage_reasons', []))
        for reason in after.get(profile, {}).get('check_reasons', []):
            if reason['kind'] == 'budget' and reason not in old_reasons:
                place = f"{profile} budget check reason {reason['binary']}"
                detail = json.dumps(reason, sort_keys=True)
                findings.append((f'{place} {detail}', f'{BUDGET_REASON}{place}: {detail}'))
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
        for group, layer in sorted(old['rows'].keys() | new['rows'].keys()):
            key = group, layer
            previous, row = old['rows'].get(key, {}), new['rows'].get(key, {})
            for field, label in (('instructions', 'ceiling'), ('ideal_instructions', 'ideal')):
                raised(f'{profile} {group} {layer} instruction {label}',
                       previous.get(field), row.get(field), added=True)
            # Without an accepted level, the layer grows by its usual allowance alone (budgets.md, 8).
            raised(f'{profile} {group} {layer} instruction accepted level',
                   previous.get('accepted_instructions'), row.get('accepted_instructions'), ceiling=True, added=True)
        for (source, target, why), amounts in sorted(new['instruction_moves'].items()):
            remaining = Counter(old['instruction_moves'].get((source, target, why), ()))
            changed = []
            for rank, amount in enumerate(sorted(amounts, reverse=True), start=1):
                if remaining[amount]:
                    remaining[amount] -= 1
                else:
                    changed.append((rank, amount))
            previous = sorted(remaining.elements(), reverse=True)
            place = f'{profile} move {source} to {target} instructions ({why!r})'
            for index, (rank, amount) in enumerate(changed):
                label = f'{place} #{rank}' if rank > 1 else place
                raised(label, previous[index] if index < len(previous) else None, amount, added=True)
        for (binary, test), seconds in sorted(new['reasons'].items()):
            place = f'{profile} reason {binary} {test}'
            raised(place, old['reasons'].get((binary, test)), seconds, added=True)
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


def path_short_cut(revision, head):
    paths = git('diff', '--no-renames', '--name-only', revision, head).splitlines()
    if BUDGETS not in paths:
        return f'figures: {BUDGETS} unchanged since {revision}'
    if paths == [BUDGETS]:
        return f"figures: {BUDGETS} alone changed since {revision}: the user's figures"
    return None


def compare_figures(revision, head):
    return compare(read_table(revision, BUDGETS), read_table(head, BUDGETS),
                   read_table(head, 'tools/tiers/layers.toml')['layers'], revision, head)


def compare_revision(revision, head):
    """Print the short cut, or return the figure findings for a mixed change."""
    short_cut = path_short_cut(revision, head)
    if short_cut:
        print(short_cut)
        return None
    return compare_figures(revision, head)


def batch_members(base, head):
    """Return the merge prefixes, lowest first, only for a complete stack."""
    try:
        start = git('rev-parse', '--verify', '--quiet', '--end-of-options', f'{base}^{{commit}}').strip()
        tip = git('rev-parse', '--verify', '--quiet', '--end-of-options', f'{head}^{{commit}}').strip()
        history = git('rev-list', '--first-parent', '--parents', f'{start}..{tip}')
    except subprocess.CalledProcessError:
        # Trees and checkouts without the history still have a tree diff.
        return []
    commits = [line.split() for line in history.splitlines()]
    if len(commits) < 2 or any(len(row) < 3 for row in commits) or commits[-1][1] != start:
        return []
    return [row[0] for row in reversed(commits)]


def print_findings(findings):
    for _, line in findings:
        print(line)


def check_batch(base, head, members):
    print(f'figures: {head} stacks {len(members)} members on {base}: each judged by its own change')
    below, count = base, 0
    for member in members:
        findings = compare_revision(below, member)
        if findings is not None:
            print_findings(findings)
            print(f'figures: {len(findings)} raised or added since {below}')
            count += len(findings)
        below = member
    return count


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    parser.add_argument('--head', default='HEAD')
    parser.add_argument('--merged', action='append')
    args = parser.parse_args(argv)
    try:
        short_cut = path_short_cut(args.base, args.head)
        if short_cut:
            print(short_cut)
            if not args.merged:
                return 0
        members = batch_members(args.base, args.head) if not (short_cut or args.merged) else []
        if members:
            count = check_batch(args.base, args.head, members)
        else:
            findings = [] if short_cut else compare_figures(args.base, args.head)
            for revision in args.merged or []:
                places = {place for place, _ in compare_revision(revision, args.head) or []}
                findings = [(place, line) for place, line in findings if place in places or line.startswith(BUDGET_REASON)]
            print_findings(findings)
            count = len(findings)
    except (subprocess.CalledProcessError, OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        print(f'figures: {getattr(error, "stderr", None) or error}', file=sys.stderr)
        return 2
    since = ' and '.join([args.base, *(args.merged or [])])
    print(f'figures: {count} raised or added since {since}')
    if count:
        print(REPAIR_LINE)
    return int(bool(count))


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
