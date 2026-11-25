"""Read and write main's published window of instruction counts (tools/tiers/budgets.md, 7.1)."""

import argparse
import json
from pathlib import Path
import runpy
import statistics
import subprocess
import sys
import tomllib


INSTRUCTION_GROWTH_FLOOR = 1_000_000_000


def git(root, env, *arguments):
    return subprocess.run(
        ('git', *arguments), cwd=root, env={**env, 'GIT_NO_LAZY_FETCH': '1'},
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
    )


def read(root, env, *, ref='refs/temper/counts'):
    """Return the local version 1 counts file, optionally pinned to its commit."""
    try:
        result = git(root, env, 'cat-file', 'blob', f'{ref}:counts.json')
        if result.returncode == 0:
            data = json.loads(result.stdout)
            if isinstance(data, dict) and type(data.get('version')) is int and data['version'] == 1:
                return data
    except (OSError, ValueError):
        pass
    return None


def base(data, root, env, head='HEAD'):
    """Return the newest published commit reachable from head."""
    try:
        for commit in reversed(data['commits']):
            if git(root, env, 'merge-base', '--is-ancestor', commit, head).returncode == 0:
                return commit
    except OSError:
        pass
    return None


def binaries(data, profile, commit):
    """Map binary IDs to their tree, layer and instructions at commit."""
    commits = data['commits']
    if commit not in commits:
        return {}
    window = commits[:commits.index(commit) + 1]
    result = {}
    for binary, entry in data['profiles'].get(profile, {}).get('binaries', {}).items():
        for landed in reversed(window):
            if landed in entry['counts']:
                result[binary] = dict(tree=entry['tree'], layer=entry['layer'],
                                      instructions=entry['counts'][landed])
                break
    return result


def layers(data, profile, commit):
    """Sum the available binary counts by (tree, layer) at commit."""
    result = {}
    for entry in binaries(data, profile, commit).values():
        key = entry['tree'], entry['layer']
        result[key] = result.get(key, 0) + entry['instructions']
    return result


def allowance(count, credit=0, accepted=None):
    usual = max(count // 50, INSTRUCTION_GROWTH_FLOOR) + credit
    return usual if accepted is None else max(usual, accepted - count)


def export(ledgers, commits, profiles):
    """counts.json's object and allowances.tsv's text from main's ledgers by profile, main's last
    commits oldest first, and the gate's profiles of main's budgets file (tools/tiers/budgets.md, 7.1)."""
    window = set(commits)
    data = {'version': 1, 'commits': list(commits), 'profiles': {}}
    for profile in ('default', 'fuzzy'):
        ledger = ledgers.get(profile, {})
        placed = {}
        for binary, entry in sorted(ledger.get('binaries', {}).items()):
            figures, carried = {}, None
            for landed in entry['counts']:
                if landed['commit'] in window:
                    figures[landed['commit']] = landed['instructions']
                elif not figures:
                    carried = landed['instructions']
            # A count from before the window still stands at its first commit.
            if carried is not None and commits[0] not in figures:
                figures = {commits[0]: carried, **figures}
            if figures:
                placed[binary] = dict(tree=entry['tree'], layer=entry['layer'],
                                        cpu_s=statistics.median(entry['cpu']), counts=figures)
        data['profiles'][profile] = dict(binaries=placed, moves=ledger.get('moves', []))

    lines = ['version\t1', f'main\t{commits[-1]}']
    sums = {}
    for profile in data['profiles']:
        for binary, entry in sorted(binaries(data, profile, commits[-1]).items()):
            if entry['layer'] is None:
                continue
            lines.append(f'binary\t{profile}\t{binary}\t{entry["tree"]}\t{entry["layer"]}')
            key = profile, entry['tree'], entry['layer']
            sums[key] = sums.get(key, 0) + entry['instructions']
    for (profile, tree, layer), count in sorted(sums.items()):
        accepted = profiles.get(profile, {}).get(tree, {}).get(layer, {}).get('accepted_instructions')
        lines.append(f'layer\t{profile}\t{tree}\t{layer}\t{count}\t{allowance(count, 0, accepted)}')
    return data, '\n'.join(lines) + '\n'


def main(arguments):
    parser = argparse.ArgumentParser(prog='tools/tiers/counts.py', description=(
        "Write main's counts.json and allowances.tsv into OUT, from the checkout root "
        '(tools/tiers/budgets.md, 7.1).'))
    commands = parser.add_subparsers(dest='command', required=True)
    write = commands.add_parser('export')
    write.add_argument('--ledger', action='append', default=[], metavar='PROFILE=PATH', required=True)
    write.add_argument('--head', default='HEAD', metavar='REV')
    write.add_argument('--out', required=True, type=Path)
    options = parser.parse_args(arguments)
    cpu = runpy.run_path(str(Path(__file__).resolve().with_name('cpu.py')))
    try:
        ledgers = {}
        for given in options.ledger:
            profile, separator, path = given.partition('=')
            if not separator or profile not in ('default', 'fuzzy') or profile in ledgers:
                raise ValueError(f'--ledger takes default=PATH or fuzzy=PATH once each, got {given!r}')
            ledgers[profile] = cpu['ledger'](path)
            if ledgers[profile]['profile'] != profile:
                raise ValueError(f'{path} is the {ledgers[profile]["profile"]} ledger, not {profile}')
        listed = subprocess.run(('git', 'rev-list', '--first-parent', '--max-count=20', options.head, '--'),
                                capture_output=True, text=True, check=True)
        commits = listed.stdout.split()[::-1]
        if not commits:
            raise ValueError(f'{options.head} has no commits')
        shown = subprocess.run(('git', 'show', f'{options.head}:tools/tiers/budgets.toml'),
                               capture_output=True, text=True)
        budgets = tomllib.loads(shown.stdout) if shown.returncode == 0 else {}
        data, table = export(ledgers, commits, budgets.get('gate', {}))
        options.out.mkdir(parents=True, exist_ok=True)
        (options.out / 'counts.json').write_text(json.dumps(data, separators=(',', ':'), allow_nan=False) + '\n')
        (options.out / 'allowances.tsv').write_text(table)
    except subprocess.CalledProcessError as error:
        print(f'counts: {error.stderr.strip() or error}', file=sys.stderr)
        return 2
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'counts: {error}', file=sys.stderr)
        return 2
    print(f'counts: {sum(len(p["binaries"]) for p in data["profiles"].values())} binaries over '
          f'{len(commits)} commits, main {commits[-1][:10]}, in {options.out}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
