"""Time builds and score first-parent landings (tools/crates/design.md, 4)."""
import argparse
from collections import deque
from datetime import datetime, timezone
import heapq
import json
import math
import os
from pathlib import Path, PurePosixPath
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile


SHAPES = (('libraries', '--lib'), ('tests', '--all-targets'))
WRAPPERS = ('RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER')
ENVIRONMENT = {'TEMPER_INTERNAL_BUNDLE': 'none', 'CARGO_INCREMENTAL': '0'}
LIB_KINDS = {'lib', 'rlib', 'dylib', 'cdylib', 'staticlib', 'proc-macro'}


def run(arguments, env=None):
    result = subprocess.run(arguments, env=env, capture_output=True, text=True)
    if result.returncode:
        reason = result.stderr.strip() or result.stdout.strip() or f'exit {result.returncode}'
        raise ValueError(f'{shlex.join(arguments)}: {reason}')
    return result.stdout


def git(*arguments):
    return run(['git', *arguments])


def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise ValueError(f'{path}: {error}') from error


def build(out):
    if git('status', '--porcelain', '--untracked-files=all'):
        raise ValueError('working tree has changes or untracked files')
    if out.exists() and (not out.is_dir() or any(out.iterdir())):
        raise ValueError(f'{out}: output directory is not empty')
    commit = git('rev-parse', 'HEAD').strip()
    branch = git('rev-parse', '--abbrev-ref', 'HEAD').strip()
    env = os.environ.copy()
    for name in WRAPPERS:
        env.pop(name, None)
    env.update(ENVIRONMENT)
    toolchain = run(['rustc', '-vV'], env=env).splitlines()
    release = next((line for line in toolchain if line.startswith('release:')), None)
    if not toolchain or release is None:
        raise ValueError('rustc -vV: missing toolchain or release line')
    record = {'commit': commit, 'toolchain': {'first_line': toolchain[0], 'release': release},
              'cpu_count': os.cpu_count(), 'environment': ENVIRONMENT,
              'removed_environment': list(WRAPPERS), 'builds': {}}
    if branch != 'HEAD':
        record['branch'] = branch
    out.mkdir(parents=True, exist_ok=True)
    (out / 'metadata.json').write_text(run(
        ['cargo', 'metadata', '--format-version', '1', '--locked'], env=env))
    for name, flag in SHAPES:
        with tempfile.TemporaryDirectory(prefix=f'{name}-', dir=out) as target:
            build_env = dict(env, CARGO_TARGET_DIR=str(Path(target).resolve()))
            command = ['cargo', 'build', '--workspace', flag, '--locked', '--timings']
            record['builds'][name] = {'start': datetime.now(timezone.utc).isoformat(),
                                      'command': command,
                                      'environment': dict(ENVIRONMENT,
                                                          CARGO_TARGET_DIR=build_env['CARGO_TARGET_DIR'])}
            run(command, env=build_env)
            shutil.copyfile(Path(target) / 'cargo-timings/cargo-timing.html', out / f'{name}.html')
    (out / 'build.json').write_text(json.dumps(record, indent=2) + '\n')


class Workspace:
    def __init__(self, data):
        self.packages = {package['id']: package for package in data['packages']}
        self.identities = {}
        for identity, package in self.packages.items():
            key = (package['name'], package['version'])
            if key in self.identities:
                raise ValueError(f'ambiguous package: {key}')
            self.identities[key] = identity
        self.members = set(data['workspace_members'])
        root = Path(data['workspace_root'])
        self.directories = {identity: PurePosixPath(
            Path(self.packages[identity]['manifest_path']).parent.relative_to(root).as_posix())
                            for identity in self.members}
        self.dependencies = {}
        for node in data['resolve']['nodes']:
            kinds = {None: set(), 'dev': set(), 'build': set()}
            for dependency in node['deps']:
                for kind in dependency['dep_kinds']:
                    kinds[kind['kind']].add(dependency['pkg'])
            self.dependencies[node['id']] = kinds

    def owner(self, path):
        path = PurePosixPath(path)
        owners = [(len(directory.parts), identity, path.relative_to(directory))
                  for identity, directory in self.directories.items()
                  if path.is_relative_to(directory)]
        if not owners:
            return None, None
        _, identity, relative = max(owners)
        return identity, relative


def target_kind(description, mode, package):
    if description == '' and mode == 'todo':
        if not any(LIB_KINDS.intersection(target['kind']) for target in package['targets']):
            raise ValueError('library target missing from metadata')
        return 'lib'
    if description == ' lib (test)' and mode == 'todo':
        return 'lib-test'
    if description == ' build script' and mode == 'todo':
        return 'build'
    if description == ' build script (run)' and mode == 'run-custom-build':
        return 'run'
    match = re.fullmatch(r' (bin|test|bench|example) ("(?:[^"\\]|\\.)*")( \(test\)| \(bench\))?',
                         description)
    if match and mode == 'todo':
        kind, name, suffix = match.groups()
        allowed = {'bin': {None, ' (test)'}, 'test': {' (test)'},
                   'bench': {' (test)', ' (bench)'}, 'example': {None, ' (test)'}}
        if suffix in allowed[kind] and any(
                kind in target['kind'] and target['name'] == json.loads(name)
                for target in package['targets']):
            return kind
    raise ValueError(f'unreadable target/mode: {description!r}, {mode!r}')


def seconds(value, field):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
        raise ValueError(f'invalid {field}: {value!r}')
    return value


def read_units(path, workspace):
    text = path.read_text()
    marker = text.find('UNIT_DATA =')
    if marker == -1:
        raise ValueError(f'{path}: report without UNIT_DATA')
    tail = text[marker + len('UNIT_DATA ='):].lstrip()
    if not tail.startswith('['):
        raise ValueError(f'{path}: report without UNIT_DATA array')
    rows, _ = json.JSONDecoder().raw_decode(tail)
    units = []
    for index, row in enumerate(rows):
        try:
            key = (row['name'], row['version'])
            if key not in workspace.identities:
                raise ValueError(f'package not in metadata.json: {key}')
            identity = workspace.identities[key]
            package = workspace.packages[identity]
            kind = target_kind(row['target'], row['mode'], package)
            duration = seconds(row['duration'], 'duration')
            rmeta = row.get('rmeta_time')
            rmeta = duration if rmeta is None else seconds(rmeta, 'rmeta_time')
            if rmeta > duration:
                raise ValueError('rmeta_time exceeds duration')
            units.append(dict(row, package=identity, kind=kind, duration=duration, rmeta_time=rmeta,
                              start=seconds(row['start'], 'start'),
                              proc_macro=kind == 'lib' and any('proc-macro' in target['kind']
                                                              for target in package['targets'])))
        except (KeyError, TypeError, ValueError) as error:
            raise ValueError(f'{path}: unit {index}: {error}') from error
    return units


def waits_for(units, workspace):
    libraries = {}
    builds = {}
    runs = {}
    for index, unit in enumerate(units):
        table = {'lib': libraries, 'build': builds, 'run': runs}.get(unit['kind'])
        if table is not None:
            identity = unit['package']
            previous = table.get(identity)
            if previous is None or (unit['start'] + unit['duration'], index) > (
                    units[previous]['start'] + units[previous]['duration'], previous):
                table[identity] = index
    waits = []
    for index, unit in enumerate(units):
        identity, kind = unit['package'], unit['kind']
        dependencies = workspace.dependencies[identity]
        needed = set()
        if kind == 'run':
            if identity in builds:
                needed.add(builds[identity])
        elif kind == 'build':
            needed.update(libraries[dep] for dep in dependencies['build'] if dep in libraries)
        else:
            needed.update(libraries[dep] for dep in dependencies[None] if dep in libraries)
            if identity in runs:
                needed.add(runs[identity])
            if kind in ('test', 'bin', 'bench', 'example') and identity in libraries:
                needed.add(libraries[identity])
            if kind in ('lib-test', 'test', 'bench', 'example') or unit['target'].endswith(' (test)'):
                needed.update(libraries[dep] for dep in dependencies['dev'] if dep in libraries)
        needed.discard(index)
        waits.append(needed)
    return waits


def chain(units, waits, chosen, pipelined=False) -> tuple[float, list]:
    chosen = set(chosen)
    pending = {index: len(waits[index] & chosen) for index in chosen}
    above = {index: [] for index in chosen}
    for index in chosen:
        for dependency in waits[index] & chosen:
            above[dependency].append(index)
    ready = [index for index in chosen if not pending[index]]
    heapq.heapify(ready)
    starts, ends, previous = {}, {}, {}
    while ready:
        index = heapq.heappop(ready)
        dependencies = waits[index] & chosen
        pipeline = (pipelined and units[index]['kind'] == 'lib'
                    and all(units[dep]['kind'] == 'lib' and not units[dep]['proc_macro']
                            for dep in waits[index]))
        def available(dep):
            return starts[dep] + units[dep]['rmeta_time'] if pipeline else ends[dep]
        predecessor = max(dependencies, key=lambda dep: (available(dep), dep), default=None)
        starts[index] = available(predecessor) if predecessor is not None else 0
        ends[index] = starts[index] + units[index]['duration']
        previous[index] = predecessor
        for dependent in above[index]:
            pending[dependent] -= 1
            if not pending[dependent]:
                heapq.heappush(ready, dependent)
    if len(ends) != len(chosen):
        raise ValueError('cycle in unit waits')
    last = max(ends, key=lambda index: (ends[index], index), default=None)
    total = ends[last] if last is not None else 0.0
    path = []
    while last is not None:
        path.append(last)
        last = previous[last]
    return total, list(reversed(path))


def upward(waits, chosen):
    above = [[] for _ in waits]
    for index, dependencies in enumerate(waits):
        for dependency in dependencies:
            above[dependency].append(index)
    result = set(chosen)
    queue = deque(sorted(result))
    while queue:
        for dependent in above[queue.popleft()]:
            if dependent not in result:
                result.add(dependent)
                queue.append(dependent)
    return result


def rebuilt(units, waits, workspace, paths):
    chosen, changed = set(), set()
    for path in paths:
        identity, relative = workspace.owner(path)
        if identity is None:
            continue
        changed.add(identity)
        first = relative.parts[0] if relative.parts else ''
        special = {'tests': 'test', 'benches': 'bench', 'examples': 'example'}.get(first)
        chosen.update(index for index, unit in enumerate(units)
                      if unit['package'] == identity and (special is None or unit['kind'] == special))
    return upward(waits, chosen), changed


def landings(commit):
    history = [line.split() for line in git('rev-list', '--first-parent', '--timestamp',
                                          '--parents', commit).splitlines()]
    end = int(history[0][0])
    result = []
    for row in reversed(history):
        timestamp, identity, *parents = row
        if not parents or not end - 7 * 86400 <= int(timestamp) <= end:
            continue
        date, subject = git('show', '-s', '--format=%cs%x00%s', identity).rstrip('\n').split('\0', 1)
        paths = git('diff-tree', '-r', '--no-renames', '--name-only', '-z',
                    parents[0], identity).split('\0')
        result.append({'commit': identity, 'date': date, 'subject': subject,
                       'paths': [path for path in paths if path]})
    return result


def summary(values):
    if not values:
        return 0.0, 0.0, 0.0
    return (statistics.mean(values), statistics.median(values),
            sorted(values, reverse=True)[math.ceil(len(values) / 10) - 1])


def score(units, waits, workspace, history):
    result = []
    for landing in history:
        chosen, changed = rebuilt(units, waits, workspace, landing['paths'])
        result.append({'units': chosen, 'changed': changed,
                       'recompile': sum(units[index]['duration'] for index in chosen),
                       'chain': chain(units, waits, chosen)[0]})
    return result


def costliest(units, waits, workspace, scores):
    total = sum(score['recompile'] for score in scores)
    result = []
    for identity in workspace.members:
        own = {index for index, unit in enumerate(units) if unit['package'] == identity}
        libraries = {index for index in own if units[index]['kind'] == 'lib'}
        above = {units[index]['package'] for index in upward(waits, libraries)} & workspace.members
        compile_time = sum(units[index]['duration'] for score in scores for index in own & score['units'])
        result.append({'package': identity, 'time': sum(units[index]['duration'] for index in own),
                       'changed': sum(identity in score['changed'] for score in scores),
                       'rebuilt': sum(bool(own & score['units']) for score in scores),
                       'share': compile_time / total if total else 0.0, 'above': len(above - {identity})})
    return sorted(result, key=lambda row: (-row['share'], workspace.packages[row['package']]['name']))[:10]


def time_text(value):
    return f'{value:.0f} s'


def label(unit):
    return f"{unit['name']} {unit['version']}{unit['target']}"


def chain_text(units, waits, pipelined=False):
    total, path = chain(units, waits, range(len(units)), pipelined)
    route = ' → '.join(f'`{label(units[index])}` ({time_text(units[index]["duration"])})' for index in path)
    return f'{time_text(total)}; {route or "none"}'


def figure(out):
    record = read_json(out / 'build.json')
    workspace = Workspace(read_json(out / 'metadata.json'))
    history = landings(record['commit'])
    shapes = {}
    for name, _ in SHAPES:
        units = read_units(out / f'{name}.html', workspace)
        waits = waits_for(units, workspace)
        shapes[name] = units, waits, score(units, waits, workspace, history)
    lines = ['# Build', '', f"Commit: `{record['commit']}`; branch: {record.get('branch', 'detached')}.",
             f"Toolchain: {record['toolchain']['first_line']}; {record['toolchain']['release']}.",
             f"CPUs: {record['cpu_count']}."]
    if history:
        first, last = history[0], history[-1]
        lines.append(f"Window: `{first['commit'][:10]}` ({first['date']}) to "
                     f"`{last['commit'][:10]}` ({last['date']}); {len(history)} landings.")
    else:
        lines.append('Window: no landings.')
    for name, _ in SHAPES:
        units, waits, scores = shapes[name]
        wall = (max(unit['start'] + unit['duration'] for unit in units)
                - min(unit['start'] for unit in units)) if units else 0
        recompile_mean, _, recompile_tenth = summary([score['recompile'] for score in scores])
        chain_mean, chain_median, chain_tenth = summary([score['chain'] for score in scores if score['units']])
        lines.extend(['', f'## {name.capitalize()}', '', '### From scratch', '',
                      f'{len(units)} units; total compile: {time_text(sum(unit["duration"] for unit in units))}; '
                      f'wall: {time_text(wall)}.',
                      f'Full chain: {chain_text(units, waits)}.',
                      f'Pipelined chain: {chain_text(units, waits, True)}.', '', '### Landings', '',
                      f'{sum(bool(score["units"]) for score in scores)} rebuilt anything.',
                      f'Recompile: mean {time_text(recompile_mean)}; tenth slowest {time_text(recompile_tenth)}.',
                      f'Chain: mean {time_text(chain_mean)}; median {time_text(chain_median)}; '
                      f'tenth slowest {time_text(chain_tenth)}.', '', '### Costliest members', '',
                      '| Crate | Time | Changed | Rebuilt | Share | Above it |',
                      '|---|---|---|---|---|---|'])
        for row in costliest(units, waits, workspace, scores):
            package = workspace.packages[row['package']]['name']
            lines.append(f'| `{package}` | {time_text(row["time"])} | {row["changed"]} | '
                         f'{row["rebuilt"]} | {row["share"]:.0%} | {row["above"]} |')
    lines.extend(['', '## Each landing', '',
                  '| Commit | Date | Subject | Libraries recompile | Libraries chain | Tests recompile | Tests chain |',
                  '|---|---|---|---|---|---|---|'])
    for index, landing in enumerate(history):
        values = [time_text(shapes[name][2][index][field])
                  for name, _ in SHAPES for field in ('recompile', 'chain')]
        subject = landing['subject'].replace('|', '\\|')
        lines.append(f'| `{landing["commit"][:10]}` | {landing["date"]} | {subject} | ' + ' | '.join(values) + ' |')
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    builds = commands.add_parser('build', help='time two clean builds')
    builds.add_argument('--out', type=Path, required=True)
    figures = commands.add_parser('figure', help='score landings against a saved build')
    figures.add_argument('directory', type=Path)
    arguments = parser.parse_args()
    try:
        if arguments.command == 'build':
            build(arguments.out)
        else:
            print(figure(arguments.directory), end='')
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        print(f'crates: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
