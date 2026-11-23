"""Read main's published window of instruction counts (tools/tiers/budgets.md, 7)."""

import json
import subprocess


INSTRUCTION_GROWTH_FLOOR = 1_000_000_000


def git(root, env, *arguments):
    return subprocess.run(
        ('git', *arguments), cwd=root, env={**env, 'GIT_NO_LAZY_FETCH': '1'},
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
    )


def read(root, env):
    """Return the local version 1 counts file, if available."""
    try:
        result = git(root, env, 'cat-file', 'blob', 'refs/temper/counts:counts.json')
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
