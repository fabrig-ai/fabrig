"""Plan the assembled goal's focused tests from changed paths and one Cargo package map."""
from collections import defaultdict
from pathlib import Path


# Tests which read the repository's landed files. Some goal trees still carry
# the combined current/landed tests. Other cases in each binary still run.
LANDED_CASES = {
    'client_layouts': ('client_layout_changes_move_their_versions',),
    'layouts': ('client_wire_layouts_are_current_and_landed', 'landing_history_preserves_client_wire_layouts'),
    'host_tools': ('host_tool_contract_history_is_kept',),
}


def workspace_filter(packages):
    if 'fabrig-boundary' not in packages:
        return []
    cases = ' | '.join(f'test(={case})' for names in LANDED_CASES.values() for case in names)
    return ['-E', f'default() & not (package(=fabrig-boundary) & ({cases}))']


def owns_package(root, path):
    return any((directory / 'Cargo.toml').is_file()
               for directory in (root / path).parents if directory.is_relative_to(root) and directory != root)


def package_plan(root, paths, packages, layers, also=()):
    dirs = {Path(package['manifest_path']).parent: name for name, package in packages.items()}
    affected = defaultdict(set)
    for path in paths:
        file = root / path
        directory = next((parent for parent in file.parents if parent in dirs), None)
        if directory:
            affected[dirs[directory]].add(file)

    selected = {}
    changed = set()
    for name, files in affected.items():
        package = packages[name]
        tests = {Path(target['src_path']): target['name'] for target in package['targets']
                 if 'test' in target['kind']}
        if files <= tests.keys():
            selected[name] = {('--test', tests[file]) for file in files}
        else:
            selected[name] = None
            directory = Path(package['manifest_path']).parent
            if any(not file.is_relative_to(directory / 'tests') for file in files):
                changed.add(directory)
    for name, package in packages.items():
        if any(dep.get('path') and Path(dep['path']) in changed for dep in package['dependencies']):
            selected[name] = None
    # The gate's always-run packages: every test, without widening to their dependents.
    for name in also:
        selected[name] = None

    # Group only identical Cargo target selections. A --test flag applies to every -p.
    groups = defaultdict(list)
    for name, wanted in sorted(selected.items()):
        package = packages[name]
        tree = Path(package['manifest_path']).relative_to(root).parts[0]
        targets = set()
        for target in package['targets']:
            if not target['test']:
                continue
            kind = target['kind'][0]
            if kind == 'test':
                flags = ('--test', target['name'])
                if layers.get(tree, {}).get(f"{name}::{target['name']}") in ('end-to-end', 'real-loop'):
                    continue
            elif kind == 'bin':
                flags = ('--bin', target['name'])
            elif kind in ('lib', 'rlib', 'staticlib', 'cdylib', 'dylib', 'proc-macro'):
                flags = ('--lib',)
            else:
                continue
            if wanted is None or flags in wanted:
                targets.add(flags)
        if targets or Path(package['manifest_path']).parent in changed:
            groups[tuple(sorted(targets))].append(name)
    return groups


def target_filter(targets):
    """A filterset for one group's target flags, as nextest names its binaries."""
    parts = []
    for flags in targets:
        if flags == ('--lib',):
            parts.append('kind(lib) | kind(proc-macro)')
        else:
            parts.append(f"kind({flags[0].lstrip('-')}) & binary(={flags[1]})")
    return ' | '.join(f'({part})' for part in parts)


def rust_commands(groups, prerequisites, directory, runner='tools/tiers/one-process'):
    """Helper builds, a check of crates without tests, then every group's tests in one run a profile.

    One run builds the selection in parallel and runs its binaries side by side, where a run per
    group of identical target flags paid a build, a listing and a serial pass each (11-23: 47 groups).
    """
    tested = {name for targets, names in groups.items() if targets for name in names}
    helpers = {(entry['package'], entry['bin']) for name in tested for entry in prerequisites.get(name, [])}
    for package, binary in sorted(helpers):
        yield ['cargo', 'build', '--offline', '--locked', '-p', package, '--bin', binary]
    checked = sorted(name for targets, names in groups.items() if not targets for name in names)
    if checked:
        yield ['cargo', 'check', '--offline', '--locked', *[arg for name in checked for arg in ('-p', name)]]
    runs = sorted((targets, names) for targets, names in groups.items() if targets)
    if not runs:
        return
    names = sorted({name for _, members in runs for name in members})
    selected = ' | '.join(
        '((' + ' | '.join(f'package(={name})' for name in members) + ') & (' + target_filter(targets) + '))'
        for targets, members in runs)
    expression = f'default() & ({selected})'
    boundary = workspace_filter(names)
    if boundary:
        expression += ' & ' + boundary[1].removeprefix('default() & ')
    # Then the same packages' fuzzy tests, which the gate judges as their own profile: default() is each
    # profile's own filter, so one expression selects both.
    for step, profile in enumerate(('default', 'fuzzy')):
        yield [runner, '--out', str(directory / str(step)), '--fail-fast', profile, '--offline', '--locked',
               '--no-tests=warn', *[arg for name in names for arg in ('-p', name)], '-E', expression]
