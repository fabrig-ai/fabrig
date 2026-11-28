"""Plan the assembled goal's focused tests from changed paths and one Cargo package map."""
from collections import defaultdict
from pathlib import Path


# Tests which read the repository's landed files. Other cases in each binary still run.
LANDED_CASES = {
    'client_layouts': ('client_versions_are_frozen',),
    'formats': ('released_format_fixtures_are_frozen',),
    'host_tools': ('host_tool_contract_history_is_kept',),
}


def workspace_filter(packages):
    if 'fabrig-boundary' not in packages:
        return []
    cases = ' | '.join(f'test(={case})' for names in LANDED_CASES.values() for case in names)
    return ['-E', f'default() & not (package(=fabrig-boundary) & ({cases}))']


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


def dependent_packages(packages, failed):
    """Follow reverse path dependencies, including through packages outside the test selection."""
    directories = {Path(package['manifest_path']).parent: name for name, package in packages.items()}
    consumers = defaultdict(set)
    for name, package in packages.items():
        for dep in package['dependencies']:
            if dep.get('path') and (provider := directories.get(Path(dep['path']))) is not None:
                consumers[provider].add(name)
    affected, pending = set(failed), list(failed)
    while pending:
        new = consumers[pending.pop()] - affected
        affected.update(new)
        pending.extend(new)
    return affected


def rust_test_command(groups, directory, profile, runner, omitted=()):
    """One run builds the selected targets in parallel and runs their binaries side by side."""
    runs = [(targets, [name for name in names if name not in omitted])
            for targets, names in sorted(groups.items()) if targets]
    runs = [(targets, names) for targets, names in runs if names]
    if not runs:
        return None
    names = sorted({name for _, members in runs for name in members})
    selected = ' | '.join(
        '((' + ' | '.join(f'package(={name})' for name in members) + ') & (' + target_filter(targets) + '))'
        for targets, members in runs)
    expression = f'default() & ({selected})'
    boundary = workspace_filter(names)
    if boundary:
        expression += ' & ' + boundary[1].removeprefix('default() & ')
    # default() is each profile's own filter, so one expression selects both.
    return [runner, '--out', str(directory), profile, '--offline', '--locked',
            '--no-tests=warn', *[arg for name in names for arg in ('-p', name)], '-E', expression]


def rust_commands(groups, prerequisites, directory, runner='tools/tiers/one-process'):
    """Helper builds, a check of crates without tests, then the focused and fuzzy profiles."""
    tested = {name for targets, names in groups.items() if targets for name in names}
    helpers = {(entry['package'], entry['bin']) for name in tested for entry in prerequisites.get(name, [])}
    for package, binary in sorted(helpers):
        yield ['cargo', 'build', '--offline', '--locked', '-p', package, '--bin', binary]
    checked = sorted(name for targets, names in groups.items() if not targets for name in names)
    if checked:
        yield ['cargo', 'check', '--offline', '--locked', *[arg for name in checked for arg in ('-p', name)]]
    if tested:
        for step, profile in enumerate(('default', 'fuzzy')):
            yield rust_test_command(groups, directory / str(step), profile, runner)
