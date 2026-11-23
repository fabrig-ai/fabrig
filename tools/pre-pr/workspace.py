"""Plan the assembled goal's focused tests from changed paths and one Cargo package map."""
from collections import defaultdict
from pathlib import Path


# Tests which read the repository's landed files. Some goal trees still carry
# the combined current/landed tests. Other cases in each binary still run.
LANDED_CASES = {
    'client_layouts': ('client_layout_changes_move_their_versions',),
    'layouts': ('client_wire_layouts_are_current_and_landed', 'landing_history_preserves_client_wire_layouts'),
    'host_tools': ('host_tool_contract_history_is_kept',),
    'patches': ('stored_changes_have_patches', 'patch_modules_are_current'),
}


def workspace_filter(packages):
    if 'fabrig-boundary' not in packages:
        return []
    cases = ' | '.join(f'test(={case})' for names in LANDED_CASES.values() for case in names)
    return ['-E', f'default() & not (package(=fabrig-boundary) & ({cases}))']


def owns_package(root, path):
    return any((directory / 'Cargo.toml').is_file()
               for directory in (root / path).parents if directory.is_relative_to(root) and directory != root)


def package_plan(root, paths, packages, layers):
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


def rust_commands(groups, prerequisites, directory, runner='tools/tiers/one-process'):
    tested = {name for targets, names in groups.items() if targets for name in names}
    helpers = {(entry['package'], entry['bin']) for name in tested for entry in prerequisites.get(name, [])}
    for package, binary in sorted(helpers):
        yield ['cargo', 'build', '--offline', '--locked', '-p', package, '--bin', binary]
    for step, (targets, names) in enumerate(sorted(groups.items())):
        packages = [arg for name in names for arg in ('-p', name)]
        if not targets:
            yield ['cargo', 'check', '--offline', '--locked', *packages]
            continue
        flags = [arg for target in targets for arg in target]
        yield [runner, '--out', str(directory / str(step)), '--fail-fast',
               'default', '--offline', '--locked', '--no-tests=warn', *packages, *flags, *workspace_filter(names)]
