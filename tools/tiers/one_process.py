"""One-process suite runner and its lanes (tools/tiers/design.md, 11).

The files crossing the summary boundary follow
 temper/docs/plan/inproc-runner/notes/contracts.md, 2 and
 temper/docs/plan/inproc-fixes/notes/contracts.md, 1.
"""
import contextlib
import errno
import fnmatch
import functools
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import runpy
import selectors
import shutil
import signal
import socket
import statistics
import struct
import subprocess
import sys
import tempfile
import time
import tomllib
import xml.etree.ElementTree as ET

CPU = runpy.run_path(str(Path(__file__).with_name('cpu.py')))
save = CPU['save']
session_processes = CPU['session_processes']
signal_group = CPU['signal_group']
stop = CPU['stop']
SUITE_PROCESS_KINDS = CPU['SUITE_PROCESS_KINDS']
CHECK_LINE = re.compile(rb'(' + CPU['CHECK_KIND'].pattern.encode() + rb') ([0-9]+)\n')
LANES = None
TERM_GRACE_S = 10.0
WALL_GRACE_S = 15.0
LEAK_S = 0.1
EVENT = 'tiers-one-process: '

# Loaded before the lanes fork (tools/tiers/design.md, 11.4, Instructions).
try:
    import ctypes
    SYSCALL = ctypes.CDLL(None, use_errno=True).syscall
    SYSCALL.restype = ctypes.c_long
    PERF_EVENT_OPEN = {'x86_64': 298, 'aarch64': 241}.get(platform.machine())
except ImportError:
    PERF_EVENT_OPEN = None


def perf_event_open(attr, pid, cpu, group_fd, flags):
    fd = SYSCALL(PERF_EVENT_OPEN, attr, pid, cpu, group_fd, flags)
    if fd == -1:
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error))
    return fd


def instruction_attr(*, enable_on_exec):
    flags = (1 << 0) | (1 << 1) | (1 << 5) | (1 << 6)
    if enable_on_exec:
        flags |= 1 << 12
    return struct.pack('=IIQQQQQIIQ', 0, 64, 1, 0, 0, 3, flags, 0, 0, 0)


def counting(paranoid=Path('/proc/sys/kernel/perf_event_paranoid')):
    """Decide once before a suite, in design.md, 11.4's order."""
    try:
        text = paranoid.read_text()
    except OSError as error:
        return dict(counted=False, why=f'no counter: cannot read {paranoid}: {error}')
    try:
        level = int(text)
    except ValueError:
        return dict(counted=False, why=f'no counter: {paranoid} holds no integer')
    if level > 2:
        return dict(counted=False, why=f'kernel.perf_event_paranoid is {level}, past 2')
    if PERF_EVENT_OPEN is None:
        return dict(counted=False, why=f'no counter: no perf_event_open on {platform.machine()}')
    try:
        fd = perf_event_open(instruction_attr(enable_on_exec=False), 0, -1, -1, 8)
        os.close(fd)
    except OSError as error:
        why = 'perf_event_open refused' if error.errno in (errno.EACCES, errno.EPERM) else 'no counter: perf_event_open'
        return dict(counted=False, why=f'{why}: {error}')
    return dict(counted=True, why=None)


def default_lanes():
    """Count affinity CPUs, else machine CPUs, else one (tools/tiers/design.md, 11.4)."""
    if hasattr(os, 'sched_getaffinity'):
        return len(os.sched_getaffinity(0))
    return os.cpu_count() or 1


def filterset(expression):
    """Compile precisely the filterset subset in design.md, 11.3."""
    position = 0

    def space():
        nonlocal position
        while position < len(expression) and expression[position].isspace():
            position += 1

    def take(token):
        nonlocal position
        space()
        if expression.startswith(token, position):
            end = position + len(token)
            if token.isalpha() and end < len(expression) and (expression[end].isalnum() or expression[end] == '_'):
                return False
            position = end
            return True
        return False

    def atom(negated=False):
        nonlocal position
        if take('not') or take('!'):
            return atom(not negated)
        if take('('):
            child = union(negated)
            if not take(')'):
                raise ValueError('missing closing parenthesis')
            return child
        space()
        name = re.match(r'[a-z_]+', expression[position:])
        if name is None:
            raise ValueError('expected filter predicate')
        name = name[0]
        position += len(name)
        if name not in ('all', 'none', 'package', 'binary', 'binary_id', 'kind', 'test') or not take('('):
            raise ValueError('unsupported filter predicate')
        space()
        if name in ('all', 'none'):
            if not take(')'):
                raise ValueError('unexpected matcher')
            return lambda test: (name == 'all') != negated
        start = position
        # A regex may contain parentheses and escaped slashes.
        if expression[position:position + 1] == '/':
            position += 1
            while position < len(expression):
                if expression[position] == '\\':
                    position += 2
                elif expression[position] == '/':
                    position += 1
                    break
                else:
                    position += 1
        else:
            while position < len(expression) and expression[position] != ')':
                position += 1
        matcher = expression[start:position].strip()
        if not matcher or not take(')'):
            raise ValueError('invalid matcher')
        operator = matcher[0]
        if operator in '=~#':
            pattern = matcher[1:]
        elif operator == '/':
            if not matcher.endswith('/'):
                raise ValueError('unterminated regex')
            pattern = re.compile(matcher[1:-1])
        else:
            operator = '~' if name == 'test' else '=' if name == 'kind' else '#'
            pattern = matcher
        if operator == '/':
            match = pattern.search
        elif operator == '=':
            match = lambda value: value == pattern
        elif operator == '~':
            match = lambda value: pattern in value
        else:
            match = lambda value: fnmatch.fnmatchcase(value, pattern)
        # Before linking, a case name is unknown: either polarity could match.
        return lambda test: name not in test or bool(match(test[name])) != negated

    def intersection(negated):
        child = atom(negated)
        while True:
            if take('and') or take('&'):
                right = atom(negated)
            elif take('-'):
                right = atom(not negated)
            else:
                return child
            child = combine(child, right, either=negated)

    def combine(left, right, either):
        if either:
            return lambda test: left(test) or right(test)
        return lambda test: left(test) and right(test)

    def union(negated=False):
        child = intersection(negated)
        while take('or') or take('|') or take('+'):
            child = combine(child, intersection(negated), either=not negated)
        return child

    try:
        result = union()
        space()
        if position != len(expression):
            raise ValueError('unexpected trailing expression')
        return result
    except (ValueError, re.error) as error:
        raise ValueError(f'unsupported filter {expression!r}: {error}') from error


def duration(value):
    if value is None:
        return None
    parts = re.findall(r'(\d+(?:\.\d+)?)\s*(ms|s|m|h|d)', value)
    if not parts or re.sub(r'(\d+(?:\.\d+)?)\s*(ms|s|m|h|d)\s*', '', value).strip():
        raise ValueError(f'invalid duration {value!r}')
    return sum(float(number) * {'ms': 0.001, 's': 1, 'm': 60, 'h': 3600, 'd': 86400}[unit]
               for number, unit in parts)


def profile_wall(path, profile):
    """Read only profile global timeouts (design.md, 11.7)."""
    try:
        data = tomllib.loads(Path(path).read_text())
        if set(data) != {'profile'} or not isinstance(data['profile'], dict):
            raise ValueError('expected only profile tables')
        for name, table in data['profile'].items():
            if not isinstance(table, dict) or set(table) != {'global-timeout'}:
                raise ValueError(f'profile {name!r} must hold global-timeout alone')
            value = table['global-timeout']
            if not isinstance(value, str):
                raise ValueError(f'profile {name!r} global-timeout must be a string')
            duration(value)
        if profile not in data['profile']:
            raise ValueError(f'has no wall for profile {profile!r}')
        return data['profile'][profile]['global-timeout']
    except ValueError as error:
        raise ValueError(f'walls file {path}: {error}') from error


def configuration(config, profile, env, limits, caller_filter):
    data = tomllib.loads(Path(config).read_text())
    profiles = data.get('profile', {})
    default = profiles.get('default', {})
    chosen = profiles.get(profile, {})
    merged = dict(default, **chosen)
    overrides = chosen.get('overrides', []) + (default.get('overrides', []) if profile != 'default' else [])
    if 'cpu_test_s' in limits.get(profile, {}):
        raise ValueError('cpu_test_s is not measured in one-process mode')
    if 'NEXTEST_TEST_THREADS' in env or env.get('NEXTEST_RETRIES', '0') != '0':
        raise ValueError('NEXTEST_TEST_THREADS or NEXTEST_RETRIES is unsupported')
    for table in (default, chosen):
        for key in ('test-threads', 'inherits'):
            if key in table:
                raise ValueError(f'unsupported profile {key}')
        if table.get('scripts'):
            raise ValueError('profile setup scripts are unsupported')
    for table in [merged, *overrides]:
        if table.get('retries', 0) != 0:
            raise ValueError('retries are unsupported')
        if 'terminate-after' in table.get('slow-timeout', {}):
            raise ValueError('slow-timeout terminate-after is unsupported')
    compiled = []
    for override in overrides:
        for key in ('threads-required', 'run-extra-args'):
            if key in override:
                raise ValueError(f'unsupported override {key}')
        if 'platform' in override:
            if 'priority' in override or 'test-group' in override:
                raise ValueError('unsupported override platform')
            continue
        compiled.append((filterset(override['filter']), override))
    default_filter = None if caller_filter else filterset(merged.get('default-filter', 'all()'))
    groups = {name: value['max-threads'] for name, value in data.get('test-groups', {}).items()}
    return merged, compiled, default_filter, groups


def arguments(command, profile):
    try:
        index = next(index for index, word in enumerate(command) if Path(word).name == 'cargo')
    except StopIteration as error:
        raise ValueError('command must name cargo nextest run') from error
    if command[index + 1:index + 3] != ['nextest', 'run']:
        raise ValueError('command must name cargo nextest run')
    prefix, cargo = command[:index], command[index]
    args = command[index + 3:]
    selected, no_tests = [], 'fail'
    drop = {'--no-fail-fast', '--fail-fast', '--no-capture', '--hide-progress-bar'}
    drop_value = {'--max-fail', '--failure-output', '--success-output', '--status-level',
                  '--final-status-level', '--message-format', '--color', '--output-indent'}
    forbidden = {'--no-run', '--stress-count', '--stress-duration', '--debugger',
                 '--config-file', '--tool-config-file', '--test-threads', '-j'}
    offset = 0
    while offset < len(args):
        argument = args[offset]
        key, equal, value = argument.partition('=')
        if argument.startswith('-P') and len(argument) > 2:
            key, equal, value = '-P', '=', argument[2:]
        if key in forbidden or (argument.startswith('-j') and not argument.startswith('--')):
            raise ValueError(f'unsupported argument {argument}')
        if key in ('--profile', '-P', '--retries', '--no-tests') or key in drop_value:
            if not equal:
                offset += 1
                if offset == len(args):
                    raise ValueError(f'missing value for {key}')
                value = args[offset]
            if key in ('--profile', '-P') and value != profile:
                raise ValueError('another --profile is unsupported')
            if key == '--retries' and value != '0':
                raise ValueError('retries are unsupported')
            if key == '--no-tests':
                no_tests = value
        elif key not in drop:
            selected.append(argument)
        offset += 1
    if no_tests not in ('fail', 'warn', 'pass'):
        raise ValueError('invalid --no-tests rule')
    # Package/build option values are not test filters.
    selection_flags = {'-p', '--package', '--exclude', '--bin', '--test', '--example', '--bench',
                       '--features', '-F', '--target', '--target-dir', '--manifest-path', '--jobs',
                       '--run-ignored', '--cargo-profile'}
    caller_filter = False
    skip = False
    for argument in selected:
        if skip:
            skip = False
            continue
        key = argument.partition('=')[0]
        if key in selection_flags:
            skip = '=' not in argument
        elif key in ('-E', '--filterset', '--filter-expr'):
            caller_filter = True
            skip = '=' not in argument
        elif (argument.startswith('-E') and len(argument) > 2) or key in (
                '--ignore-default-filter', '--partition', '--skip', '--exact') or not argument.startswith('-'):
            caller_filter = True
    return prefix, cargo, selected, no_tests, caller_filter


def json_listing(output):
    offset = 0
    for line in output.splitlines(keepends=True):
        if line.startswith('{'):
            try:
                value, _ = json.JSONDecoder().raw_decode(output, offset)
            except ValueError:
                pass
            else:
                if isinstance(value, dict) and 'rust-suites' in value:
                    return value
        offset += len(line)
    raise ValueError('missing nextest JSON listing')


def cargo_environment(env, root):
    result = {key: value for key, value in env.items() if not key.startswith('NEXTEST')}
    # Ancestors apply from least to most specific; a caller value survives unless forced.
    original = set(result)
    cargo_home = Path(env.get('CARGO_HOME', str(Path.home() / '.cargo')))
    directories = [cargo_home, *[parent / '.cargo' for parent in reversed([root, *root.parents])]]
    seen = set()
    for directory in directories:
        # Cargo prefers the extensionless file when both spellings exist.
        config = directory / 'config'
        if not config.exists():
            config = directory / 'config.toml'
        if not config.exists() or config in seen:
            continue
        seen.add(config)
        parent = directory.parent
        for key, value in tomllib.loads(config.read_text()).get('env', {}).items():
            options = value if isinstance(value, dict) else {'value': value}
            if key in original and not options.get('force', False):
                continue
            text = options['value']
            if options.get('relative', False):
                text = str((parent / text).resolve())
            result[key] = text
    return result


def binary_environment(env, cargo, suite, package, build, profile, host_libdir):
    root = Path(package['manifest_path']).parent
    result = dict(env)
    result.pop('NEXTEST', None)
    version = package.get('version', '0.0.0')
    core, _, pre = version.partition('-')
    numbers = core.split('.')
    result.update(CARGO=str(Path(shutil.which(cargo, path=env.get('PATH')) or cargo).resolve()),
                  CARGO_MANIFEST_DIR=str(root), CARGO_MANIFEST_PATH=package['manifest_path'],
                  CARGO_PKG_NAME=package['name'], CARGO_PKG_VERSION=version,
                  CARGO_PKG_VERSION_MAJOR=numbers[0], CARGO_PKG_VERSION_MINOR=numbers[1],
                  CARGO_PKG_VERSION_PATCH=numbers[2].split('+')[0], CARGO_PKG_VERSION_PRE=pre.split('+')[0],
                  CARGO_PKG_AUTHORS=':'.join(package.get('authors', [])),
                  NEXTEST_PROFILE=profile, TIERS_ONE_PROCESS='1')
    for field in ('description', 'homepage', 'repository', 'license', 'license_file', 'rust_version', 'readme'):
        result['CARGO_PKG_' + field.upper()] = package.get(field) or ''
    package_id = suite['package-id']
    target = Path(build['target-directory'])
    for binary in build.get('non-test-binaries', {}).get(package_id, []):
        if binary.get('kind') == 'bin-exe':
            path = str(target / binary['path'])
            result['CARGO_BIN_EXE_' + binary['name']] = path
            result['NEXTEST_BIN_EXE_' + binary['name']] = path
    out = build.get('build-script-out-dirs', {}).get(package_id)
    if out:
        result['OUT_DIR'] = str(target / out)
    directories = []
    for base in build.get('base-output-directories', ['debug']):
        directories.extend([str(target / base / 'deps'), str(target / base)])
    directories.extend(str(target / path) for path in build.get('linked-paths', []))
    directories.append(host_libdir)
    key = 'DYLD_FALLBACK_LIBRARY_PATH' if sys.platform == 'darwin' else 'LD_LIBRARY_PATH'
    if result.get(key):
        directories.append(result[key])
    result[key] = os.pathsep.join(directories)
    return result


def place(binary_id, manifest_path, workspace_root, table):
    """Place one binary by the ordered rules in tools/tiers/budgets.md, 3."""
    relative = Path(manifest_path).relative_to(workspace_root)
    tree = relative.parts[0]
    if tree not in ('skein', 'smith', 'jig', 'temper', 'hearth'):
        return 'repository', 'checks'
    if '::' not in binary_id or '::bin/' in binary_id:
        return tree, 'step'
    return tree, table.get(tree, {}).get(binary_id) if table is not None else None


def target_binary_id(package, target):
    """Name a test target as nextest does (tools/tiers/layers.md, 3)."""
    if target['test']:
        for kind in target['kind']:
            if kind in ('lib', 'rlib', 'cdylib', 'dylib', 'staticlib', 'proc-macro'):
                return package['name']
            if kind == 'test':
                return f"{package['name']}::{target['name']}"
            if kind in ('bin', 'example', 'bench'):
                return f"{package['name']}::{kind}/{target['name']}"
    return None


def target_flags(binary_id):
    """Cargo selection for one binary id (tools/tiers/references.md, 5)."""
    _, separator, target = binary_id.partition('::')
    if not separator:
        return ['--lib']
    kind, slash, name = target.partition('/')
    if slash and kind in ('bin', 'example', 'bench'):
        return ['--' + kind, name]
    return ['--test', target]


def cargo_test_roots(cargo, selection, env):
    """Fingerprint each build root with the features and dependencies Cargo resolved for it."""
    raw = subprocess.check_output([*cargo, 'test', '--no-run', '--offline', '--locked',
                                   '-Z', 'unstable-options', '--unit-graph', *selection],
                                  env=env, stdin=subprocess.DEVNULL, stderr=subprocess.PIPE)
    graph = json.loads(raw)
    if graph['version'] != 1:
        raise ValueError('unknown Cargo unit graph version')

    @functools.cache
    def fingerprint(index):
        unit = graph['units'][index]
        dependencies = [dict(edge, index=fingerprint(edge['index'])) for edge in unit['dependencies']]
        data = json.dumps(dict(unit, dependencies=dependencies), sort_keys=True).encode()
        return hashlib.sha256(data).hexdigest()

    roots = set()
    for index in graph['roots']:
        unit = graph['units'][index]
        if unit['mode'] != 'doctest':  # nextest lists executable tests only
            roots.add((unit['pkg_id'], unit['mode'], tuple(unit['target']['kind']),
                       unit['target']['name'], fingerprint(index)))
    return roots


def profile_target_groups(selection, metadata, filters, cargo, env, table=None, selected_layers=None, layer_omissions=None):
    """Skip integration targets that neither delivery profile can select."""
    members = {package['name']: package for package in metadata['packages']}
    if selection == ['--workspace']:
        ids = set(metadata['workspace_members'])
        names = sorted(name for name, package in members.items() if package['id'] in ids)
    elif selection and len(selection) % 2 == 0 and all(flag == '-p' for flag in selection[::2]):
        names = selection[1::2]
    else:
        return [selection]
    broad, narrowed, omitted = [], [], {}
    excluded = set()
    for name in names:
        package = members[name]
        keep, skip = [], {}
        for target in package['targets']:
            binary = target_binary_id(package, target)
            if binary is None:
                continue
            fields = dict(package=name, binary=target['name'], binary_id=binary, kind='test')
            _, layer = place(binary, package['manifest_path'], metadata['workspace_root'], table)
            excluded_layer = selected_layers is not None and layer is not None and layer not in selected_layers
            if excluded_layer:
                excluded.add(binary)
            if excluded_layer or (target['kind'] == ['test'] and not any(matches(fields) for matches in filters)):
                skip[package['id'], 'test', tuple(target['kind']), target['name']] = binary
            else:
                keep.extend(target_flags(binary))
        # Explicit Cargo selectors can require a feature that a broad build would skip.
        if not skip or any(t.get('required-features') for t in package['targets']):
            broad.extend(['-p', name])
        else:
            omitted.update(skip)
            if keep:
                narrowed.append(['-p', name, *keep])
    if not omitted:
        return [selection]

    groups = ([broad] if broad else []) + narrowed
    # Package groups can unify different dependency features. Only use the narrower
    # listings when all retained build roots have the original compilation graph.
    try:
        expected = {root for root in cargo_test_roots(cargo, selection, env) if root[:4] not in omitted}
        actual = set()
        for group in groups:
            actual.update(cargo_test_roots(cargo, group, env))
        if actual != expected:
            print('profile targets: keeping full listing (Cargo build roots differ)', flush=True)
            return [selection]
    except (OSError, subprocess.CalledProcessError, KeyError, ValueError):
        print('profile targets: keeping full listing (Cargo build roots unavailable)', flush=True)
        return [selection]
    if layer_omissions is not None:
        layer_omissions.update(excluded.intersection(omitted.values()))
    print('profile targets: skip linking ' + ', '.join(sorted(omitted.values())), flush=True)
    return groups


def profile_targets(selection, metadata, config, cargo, env, table=None, selected_layers=None, layer_omissions=None):
    """Plan a build or listing from the union of the focused and fuzzy filters."""
    try:
        profiles = tomllib.loads(Path(config).read_text()).get('profile', {})
        focused = profiles.get('default', {}).get('default-filter', 'all()')
        fuzzy = profiles.get('fuzzy', {}).get('default-filter', focused)
        return profile_target_groups(selection, metadata, (filterset(focused), filterset(fuzzy)),
                                     cargo, env, table, selected_layers, layer_omissions)
    except (OSError, KeyError, TypeError, ValueError) as error:
        print(f'profile targets: keeping full listing ({error})', flush=True)
        return [selection]


def read_metadata(cargo, env):
    return json.loads(subprocess.check_output([cargo, 'metadata', '--format-version', '1', '--no-deps'],
                                              env=env, stdin=subprocess.DEVNULL, text=True))


def workspace_binaries(metadata, listing, table):
    """Record every test target, selected or not (tools/tiers/budgets.md, 3)."""
    workspace = []
    for package in metadata['packages']:
        for target in package['targets']:
            binary_id = target_binary_id(package, target)
            if binary_id is not None:
                tree, layer = place(binary_id, package['manifest_path'], metadata['workspace_root'], table)
                listed = listing['rust-suites'].get(binary_id, {}).get('status') == 'listed'
                workspace.append(dict(binary_id=binary_id, tree=tree, layer=layer, listed=listed))
    return sorted(workspace, key=lambda binary: binary['binary_id'])


def make_plan(listing, metadata, env, cargo, profile, overrides, default_filter, caller_filter, groups, settings, table):
    build = listing['rust-build-meta']
    target = Path(build['target-directory'])
    times_path = target / 'tiers-one-process/times.json'
    try:
        kept = json.loads(times_path.read_text()).get(profile, {})
    except FileNotFoundError:
        kept = {}
    packages = {package['id']: package for package in metadata['packages']}
    host_libdir = subprocess.check_output(['rustc', '--print', 'target-libdir'], env=env,
                                         stdin=subprocess.DEVNULL, text=True).strip()
    binary_env = cargo_environment(env, Path(metadata.get('workspace_root', Path.cwd())))
    binaries = []
    for binary_id, suite in listing['rust-suites'].items():
        if suite.get('status') != 'listed':
            continue
        package = packages[suite['package-id']]
        tests = []
        for name, case in sorted(suite['testcases'].items()):
            fields = dict(package=package['name'], binary=suite['binary-name'], binary_id=binary_id,
                          kind=suite['kind'], test=name)
            matched = case['filter-match']['status'] == 'matches'
            excluded_default = case['filter-match'].get('reason') == 'default-filter'
            if not caller_filter and (matched or excluded_default) and default_filter(fields) != matched:
                raise ValueError(f'default-filter disagrees with nextest: {binary_id} {name}')
            if not matched:
                continue
            applied = {}
            for predicate, override in overrides:
                if predicate(fields):
                    for key in ('priority', 'test-group'):
                        if key in override and key not in applied:
                            applied[key] = override[key]
            group = applied.get('test-group')
            if group is not None and group not in groups and group != '@global':
                raise ValueError(f'unknown test-group {group}')
            wall = kept.get(binary_id, {}).get(name)
            if type(wall) not in (int, float) or not math.isfinite(wall) or wall < 0:
                wall = None
            tests.append(dict(name=name, priority=applied.get('priority', 0), group=group, kept_s=wall))
        if tests:
            tree, layer = place(binary_id, package['manifest_path'], metadata['workspace_root'], table)
            binaries.append(dict(binary_id=binary_id, tree=tree, layer=layer,
                                 path=suite['binary-path'], cwd=suite['cwd'], tests=tests,
                                 env=binary_environment(binary_env, cargo, suite, package, build, profile, host_libdir)))
    return dict(profile=profile, binaries=binaries, groups=groups, layers=table['layers'] if table is not None else [],
                workspace=workspace_binaries(metadata, listing, table),
                global_timeout=duration(settings.get('global-timeout')), lanes=LANES,
                term_grace_s=TERM_GRACE_S, wall_grace_s=WALL_GRACE_S, leak_s=LEAK_S,
                times_path=str(times_path))


def split_jobs(plan, lanes):
    if 'reference_picks' in plan:
        return reference_jobs(plan['binaries'], plan['reference_picks'])
    tests = [test for binary in plan['binaries'] for test in binary['tests']]
    known = [test['kept_s'] for test in tests if test['kept_s'] is not None]
    mean = sum(known) / len(known) if known else 1.0
    for test in tests:
        test['estimate'] = test['kept_s'] if test['kept_s'] is not None else mean
    if plan.get('single_process', False):
        return [dict(binary=binary, tests=binary['tests'], kind='tests', diagnostic=None)
                for binary in plan['binaries']]
    share = sum(test['estimate'] for test in tests) / lanes
    jobs = []
    for binary in plan['binaries']:
        estimate = sum(test['estimate'] for test in binary['tests'])
        count = min(len(binary['tests']), lanes, math.ceil(estimate / share)) if share and estimate > share else 1
        # Contiguous partitions near equal cumulative estimates, leaving a test per part.
        remaining = list(binary['tests'])
        chunks = []
        for parts in range(count, 0, -1):
            if parts == 1:
                chunks.append(remaining)
                break
            chunk, used = [], 0.0
            goal = sum(test['estimate'] for test in remaining) / parts
            while remaining and (not chunk or (used < goal and len(remaining) >= parts)):
                test = remaining.pop(0)
                chunk.append(test)
                used += test['estimate']
            chunks.append(chunk)
        for chunk in chunks:
            ordinary, size = [], 0
            for test in chunk:
                length = len(os.fsencode(test['name'])) + 1
                if length > 256 * 1024:
                    raise ValueError('one test name exceeds argument bound')
                if test['group'] == 'process-of-its-own':
                    jobs.append(dict(binary=binary, tests=[test], kind='tests', diagnostic=None))
                else:
                    if size + length > 256 * 1024 and ordinary:
                        jobs.append(dict(binary=binary, tests=ordinary, kind='tests', diagnostic=None))
                        ordinary, size = [], 0
                    ordinary.append(test)
                    size += length
            if ordinary:
                jobs.append(dict(binary=binary, tests=ordinary, kind='tests', diagnostic=None))
    jobs.sort(key=job_order)
    if repair := plan.get('repair_case'):
        binary, name = repair
        jobs.sort(key=lambda job: not (job['binary']['binary_id'] == binary
                                      and any(test['name'] == name for test in job['tests'])))
    return jobs


def job_order(job):
    return (-max(test['priority'] for test in job['tests']),
            -sum(test['estimate'] for test in job['tests']))


def emit(kind, **fields):
    print(EVENT + json.dumps(dict(event=kind, **fields), allow_nan=False), flush=True)


def checks_path(directory, index):
    return directory / f'checks-{index}.txt'


def read_checks(path, index, binary):
    """Read the process's checks once after it ends (tools/tiers/design.md, 11.8)."""
    try:
        data = path.read_bytes()
    except FileNotFoundError:
        return {}
    counts = {}
    for number, line in enumerate(data.splitlines(keepends=True), 1):
        match = CHECK_LINE.fullmatch(line)
        try:
            count = int(match[2]) if match else 0
        except ValueError:
            count = 0
        if count < 1:
            raise ValueError(f'process {index} ({binary}): {path}, malformed check line {number}: {line!r}')
        kind = match[1].decode('ascii')
        counts[kind] = counts.get(kind, 0) + count
    return counts


class BinaryProcess:
    """One owned child, reaped only by wait4 (design.md, 11.4 and 11.6)."""
    def __init__(self, job, index, selector, directory, *, count=False):
        self.job, self.index = job, index
        self.output = bytearray()
        self.log = bytearray()
        self.announcements = []
        self.logs = {}
        self.done = {}
        self.status = None
        self.reaped_at = None
        self.open_fds = set()
        self.cancelled_at = None
        self.counter = None
        names = b'|'.join(re.escape(test['name'].encode()) for test in job['tests'])
        self.announcement = re.compile(rb'(?m)^test (' + names + rb') \.\.\. ')
        # Libtest opens its logfile close-on-exec. A named pipe lets it do so
        # without inheriting a writer that its spawned fixtures would also keep.
        self.log_directory = tempfile.TemporaryDirectory(prefix='tiers-one-log-')
        logfile = Path(self.log_directory.name) / 'log'
        os.mkfifo(logfile, 0o600)
        read = os.open(logfile, os.O_RDONLY | os.O_NONBLOCK)
        self.log_writer = os.open(logfile, os.O_WRONLY | os.O_NONBLOCK)
        output_read, output_write = os.pipe()
        binary = job['binary']
        self.start = time.monotonic()
        sockets = socket.socketpair(socket.AF_UNIX, socket.SOCK_DGRAM) if count else None

        def open_counter():
            # Nothing may escape preexec_fn: a refused counter must not fail exec.
            try:
                fd = perf_event_open(instruction_attr(enable_on_exec=True), 0, -1, -1, 8)
                socket.send_fds(sockets[1], [b'c'], [fd])
            except BaseException:
                pass

        try:
            self.child = subprocess.Popen(
                [binary['path'], '--exact', '--nocapture', '--test-threads=1', '--include-ignored',
                 '--color', 'never',
                 *(['--logfile', str(logfile)] if job['kind'] != 'empty' else []),
                 *[t['name'] for t in job['tests']]],
                cwd=binary['cwd'], env=dict(binary['env'], TIERS_CHECKS_FILE=str(checks_path(directory, index))),
                stdin=subprocess.DEVNULL,
                stdout=output_write, stderr=subprocess.STDOUT, process_group=0,
                preexec_fn=open_counter if count else None)
            if sockets is not None:
                sockets[0].setblocking(False)
                try:
                    _, descriptors, _, _ = socket.recv_fds(sockets[0], 1, 1)
                    if descriptors:
                        self.counter, = descriptors
                        os.set_inheritable(self.counter, False)
                except OSError:
                    pass
        except BaseException:
            os.close(read)
            os.close(self.log_writer)
            os.close(output_read)
            self.log_directory.cleanup()
            raise
        finally:
            os.close(output_write)
            if sockets is not None:
                for end in sockets:
                    end.close()
        self.pid = self.child.pid
        for fd, kind in ((output_read, 'output'), (read, 'log')):
            os.set_blocking(fd, False)
            selector.register(fd, selectors.EVENT_READ, (self, kind))
            self.open_fds.add(fd)
        emit('start', index=index, start_s=self.start, pid=self.pid)

    def close(self, fd, selector):
        selector.unregister(fd)
        self.open_fds.remove(fd)
        os.close(fd)

    def read(self, fd, kind, selector):
        data = os.read(fd, 65536)
        if not data:
            self.close(fd, selector)
            return
        now = time.monotonic()
        if kind == 'output':
            self.output.extend(data)
            found = list(self.announcement.finditer(self.output))
            # An announcement follows libtest opening its logfile. Keep a writer
            # until then so the reader cannot see an early end of file.
            if found:
                self.close_log_writer()
            for match in found[len(self.announcements):]:
                self.announcements.append((match[1].decode('utf-8', 'backslashreplace'), match.start(), match.end(), now))
        else:
            self.log.extend(data)
            while b'\n' in self.log:
                line, _, tail = self.log.partition(b'\n')
                self.log = bytearray(tail)
                result, _, name = line.partition(b' ')
                if self.cancelled_at is None and result in (b'ok', b'failed', b'ignored'):
                    self.logs[name.decode('utf-8', 'backslashreplace')] = (result, now)
        self.completed()

    def close_log_writer(self):
        if self.log_writer is not None:
            os.close(self.log_writer)
            self.log_writer = None

    def completed(self):
        for offset, (name, _, start, announced_at) in enumerate(self.announcements):
            if (self.cancelled_at is not None and announced_at >= self.cancelled_at) or name in self.done or name not in self.logs:
                continue
            end = self.announcements[offset + 1][1] if offset + 1 < len(self.announcements) else len(self.output)
            segment = bytes(self.output[start:end])
            if offset + 1 == len(self.announcements) and b'\ntest result:' not in segment and self.reaped_at is None:
                continue
            result, ended_at = self.logs[name]
            word = b'ok' if result == b'ok' else b'FAILED' if result == b'failed' else b'ignored'
            words = list(re.finditer(word + rb'\r?\n', segment))
            if not words:
                continue
            output = segment[:words[-1].start()].decode('utf-8', 'backslashreplace')
            outcome = 'passed' if result == b'ok' else 'failed'
            self.finish(name, outcome, max(0.0, ended_at - announced_at), output,
                        [entry[0] for entry in self.announcements[:offset]])

    def finish(self, name, outcome, wall, output, before):
        item = dict(binary_id=self.job['binary']['binary_id'], test_name=name, outcome=outcome,
                    wall_s=wall, process=self.index, before=before if outcome in ('failed', 'crashed') else [],
                    alone=None, output=None if outcome == 'passed' else output)
        self.done[name] = item
        if self.job['kind'] == 'reference':
            emit('reference', item=item)
        elif self.job['kind'] not in ('alone', 'empty'):
            emit('test', item=item)

    def reap(self):
        if self.status is not None:
            return
        pid, status, usage = os.wait4(self.pid, os.WNOHANG)
        if pid:
            self.close_log_writer()
            # Popen must never poll or wait before this assignment: wait4 owns rusage.
            self.child.returncode = os.waitstatus_to_exitcode(status)
            self.status = status
            self.usage = usage
            self.reaped_at = time.monotonic()

    def close_counter(self):
        if self.counter is not None:
            os.close(self.counter)
            self.counter = None

    def instructions(self):
        if self.counter is None:
            return None
        try:
            data = os.read(self.counter, 24)
            if len(data) == 24:
                count, enabled, running = struct.unpack('=QQQ', data)
                if running == enabled:
                    return count
        except OSError:
            pass
        finally:
            self.close_counter()
        return None

    def settle(self, selector, leaked):
        self.completed()
        code = os.waitstatus_to_exitcode(self.status)
        announced = [entry[0] for entry in self.announcements]
        for offset, (name, _, start, announced_at) in enumerate(self.announcements):
            if self.cancelled_at is not None and announced_at >= self.cancelled_at:
                continue
            if name not in self.done:
                end = self.announcements[offset + 1][1] if offset + 1 < len(self.announcements) else len(self.output)
                output = bytes(self.output[start:end]).decode('utf-8', 'backslashreplace')
                lines = output.rstrip().splitlines()
                starved = code == 0 and name not in self.logs and bool(lines and lines[-1].startswith('skein-world: starved:'))
                outcome = 'cancelled' if self.cancelled_at is not None else 'starved' if starved else 'crashed'
                self.finish(name, outcome, max(0.0, self.reaped_at - announced_at), output, announced[:offset])
        unannounced = [test for test in self.job['tests'] if test['name'] not in announced]
        fresh = []
        if self.cancelled_at is None and self.job['kind'] != 'empty':
            if not announced:
                for test in unannounced:
                    self.finish(test['name'], 'crashed', self.reaped_at - self.start,
                                bytes(self.output).decode('utf-8', 'backslashreplace'), [])
            elif unannounced and any(t['outcome'] in ('crashed', 'starved') for t in self.done.values()):
                fresh = unannounced
            else:
                for test in unannounced:
                    self.finish(test['name'], 'crashed', 0.0, 'not run by its binary', [])
            expected = 101 if any(value[0] == b'failed' for value in self.logs.values()) else 0
            if announced and not fresh and code != expected and all(t['outcome'] not in ('crashed', 'cancelled', 'starved') for t in self.done.values()):
                last = self.done[announced[-1]]
                self.finish(last['test_name'], 'crashed', last['wall_s'],
                            (last['output'] or '') + f'\nprocess ended with status {code}', announced[:-1])
        instructions = self.instructions()
        signal_group(self.child, signal.SIGKILL)
        for fd in list(self.open_fds):
            self.close(fd, selector)
        self.log_directory.cleanup()
        record = dict(binary_id=self.job['binary']['binary_id'], kind=self.job['kind'], tests=announced,
                      tree=self.job['binary']['tree'], layer=self.job['binary']['layer'],
                      instructions=instructions, user_s=self.usage.ru_utime, system_s=self.usage.ru_stime,
                      wall_s=self.reaped_at - self.start, start_s=self.start,
                      exit_code=code if code >= 0 else None, signal=-code if code < 0 else None, leaked=leaked)
        if self.job['kind'] == 'reference':
            record['reference_name'] = self.job['tests'][0]['name']
        emit('process', index=self.index, item=record)
        return fresh


def lanes(plan, directory):
    count = plan['lanes'] or default_lanes()
    jobs = split_jobs(plan, count)
    active, processes = [], plan.get('process_offset', 0)
    cancelled, first, kill_at = None, plan.get('first_start'), None
    terminated = False

    def terminate(sig, frame):
        nonlocal terminated
        terminated = True

    previous = signal.signal(signal.SIGTERM, terminate)
    emit('lanes', lanes=count, session=os.getsid(0))
    try:
        decision = counting() if plan.get('count', False) else None
        if decision is not None:
            emit('counting', **decision)
        with selectors.DefaultSelector() as selector:
            while jobs or active:
                now = time.monotonic()
                hard = plan.get('wall_hard_s')
                wall_expired = first is not None and hard is not None and now - first >= hard
                if cancelled is None and (terminated or wall_expired or (first is not None and plan['global_timeout'] is not None and now - first >= plan['global_timeout'])):
                    cancelled = 'wall-hard' if terminated or wall_expired else 'global-timeout'
                    jobs.clear()
                    emit('cancel', reason=cancelled, at_s=now)
                    kill_at = now + plan['term_grace_s']
                    for child in active:
                        if child.reaped_at is None:
                            child.cancelled_at = now
                            signal_group(child.child, signal.SIGTERM)
                if cancelled and kill_at is not None and now >= kill_at:
                    for child in active:
                        signal_group(child.child, signal.SIGKILL)
                    kill_at = None
                while not cancelled and len(active) < count:
                    candidate = None
                    for index, job in enumerate(jobs):
                        groups = {t['group'] for t in job['tests']} - {None, '@global'}
                        if all(sum(group in {t['group'] for t in child.job['tests']} for child in active) <
                               (count if plan['groups'][group] == 'num-cpus' else plan['groups'][group]) for group in groups):
                            candidate = index
                            break
                    if candidate is None:
                        break
                    job = jobs.pop(candidate)
                    child = BinaryProcess(job, processes, selector, directory,
                                          count=bool(decision and decision['counted']
                                                     and job['kind'] in SUITE_PROCESS_KINDS))
                    processes += 1
                    first = child.start if first is None else first
                    active.append(child)
                for key, _ in selector.select(0.01):
                    child, kind = key.data
                    child.read(key.fd, kind, selector)
                for child in list(active):
                    child.reap()
                    if child.reaped_at is None:
                        continue
                    expired = time.monotonic() - child.reaped_at >= plan['leak_s']
                    if child.open_fds and not expired:
                        continue
                    fresh = child.settle(selector, bool(child.open_fds))
                    active.remove(child)
                    if cancelled or child.job['kind'] in ('reference', 'empty'):
                        continue
                    if child.job['kind'] == 'alone':
                        failed = any(t['outcome'] != 'passed' for t in child.done.values())
                        emit('alone', binary_id=child.job['binary']['binary_id'],
                             test_name=child.job['tests'][0]['name'], outcome='failed' if failed else 'passed',
                             output=''.join(t['output'] or '' for t in child.done.values()) if failed else None)
                    else:
                        if fresh and not plan.get('single_process', False):
                            job = dict(child.job, tests=fresh, kind='fresh')
                            position = next((index for index, waiting in enumerate(jobs)
                                             if waiting['kind'] != 'alone' and job_order(waiting) > job_order(job)), len(jobs))
                            jobs.insert(position, job)
                        for item in reversed(list(child.done.values())):
                            if plan.get('alone_diagnostics', True) and item['outcome'] in ('failed', 'crashed') and item['before']:
                                test = next(t for t in child.job['tests'] if t['name'] == item['test_name'])
                                jobs.insert(0, dict(child.job, tests=[test], kind='alone'))
            emit('end')
    finally:
        for child in active:
            signal_group(child.child, signal.SIGKILL)
            child.close_counter()
            if child.status is None:
                _, status, _ = os.wait4(child.pid, 0)
                child.child.returncode = os.waitstatus_to_exitcode(status)
        signal.signal(signal.SIGTERM, previous)


def write_junit(directory, results):
    root = ET.Element('testsuites')
    suites = {}
    for test in results['tests']:
        binary_id = test['binary_id']
        if binary_id not in suites:
            suites[binary_id] = ET.SubElement(root, 'testsuite', name=binary_id)
        case = ET.SubElement(suites[binary_id], 'testcase', name=test['test_name'],
                             classname=binary_id, time=str(test['wall_s']))
        outcome = test['outcome']
        if outcome in ('failed', 'crashed', 'cancelled'):
            process = results['processes'][test['process']] if test['process'] is not None else None
            sig = process['signal'] if process is not None else None
            if outcome == 'cancelled' and sig is None:
                sig = signal.SIGTERM
            if sig is not None and outcome != 'failed':
                message = f'process aborted with signal {sig} ({signal.Signals(sig).name})'
                kind = 'test abort'
            else:
                message = f"process exited with code {process['exit_code']}" if outcome == 'crashed' else 'test failed'
                kind = 'test failure'
            ET.SubElement(case, 'failure', type=kind, message=message).text = message
        if outcome != 'passed':
            ET.SubElement(case, 'system-out').text = test['output'] or ''
    for suite in suites.values():
        suite.set('tests', str(len(suite)))
        suite.set('failures', str(sum(case.find('failure') is not None for case in suite)))
    ET.ElementTree(root).write(directory / 'junit.xml', encoding='utf-8', xml_declaration=True)


def merge_listings(listings):
    """Unite package listings and their build environments (references.md, 5)."""
    listing = listings[0]
    build = listing['rust-build-meta']
    for other in listings[1:]:
        extra = other['rust-build-meta']
        if build['target-directory'] != extra['target-directory']:
            raise ValueError('nextest listings have different target directories')
        listing['rust-suites'].update(other['rust-suites'])
        directories = build.setdefault('base-output-directories', ['debug'])
        for directory in extra.get('base-output-directories', ['debug']):
            if directory not in directories:
                directories.append(directory)
        binaries = build.setdefault('non-test-binaries', {})
        for name, values in extra.get('non-test-binaries', {}).items():
            entries = binaries.setdefault(name, [])
            entries.extend(value for value in values if value not in entries)
        paths = build.setdefault('linked-paths', [])  # nextest lists them
        paths.extend(path for path in extra.get('linked-paths', []) if path not in paths)
        build.setdefault('build-script-out-dirs', {}).update(extra.get('build-script-out-dirs', {}))
    return listing


def layer_selection(path, profile, selected_layers):
    """Resolve the run's layers before any command (tools/tiers/layers.md, 9.1)."""
    try:
        with Path(path).open('rb') as source:
            table = tomllib.load(source)
    except FileNotFoundError:
        table = None
    if table is not None and (not isinstance(table.get('layers'), list)
                              or any(not isinstance(layer, str) for layer in table['layers'])):
        raise ValueError('layers: expected a list of strings')
    names = [*table['layers'], 'checks'] if table is not None else []
    if selected_layers is not None:
        if not selected_layers or any(name not in [*names, 'all'] for name in selected_layers):
            raise ValueError('layers: expected all or names from the layers list and checks')
        if table is None and tuple(selected_layers) != ('all',):
            raise ValueError('layers: without the layers file only all is supported')
    if table is None:
        return None, None
    if selected_layers is None:
        selected_layers = names if profile not in ('default', 'fuzzy') else [
            name for name in names if name not in ('real-loop', 'end-to-end')]
    elif 'all' in selected_layers:
        selected_layers = names
    return table, [name for name in names if name in selected_layers]


def listed_plan(command, env, profile, limits, config, walls, layers, picks=None, *,
                metadata=None, selections=None, selected_layers=('all',)):
    """Build and plan a listing; base picks use any filter match (references.md, 5)."""
    prefix, cargo, selection, no_tests, caller_filter = arguments(command, profile)
    settings, overrides, default_filter, groups = configuration(
        config, profile, env, limits, caller_filter or picks is not None)
    if walls is not None:
        settings['global-timeout'] = profile_wall(walls, profile)
    table, selected_layers = layer_selection(layers, profile, selected_layers)
    output = bytearray()

    def forward(text):
        print(text, end='', flush=True)
        output.extend(text.encode('utf-8', 'backslashreplace'))

    env = env.copy()
    layer_omissions = set()
    pre_pr_targets = env.pop('TIERS_PRE_PR_TARGETS', '') == '1'
    profile_build = env.pop('TIERS_PROFILE_TARGETS', '') == '1'
    if ((pre_pr_targets or profile_build) and profile in ('default', 'fuzzy')
            and picks is None and not caller_filter and selections is None):
        metadata = read_metadata(cargo, env)
        selections = profile_targets(selection, metadata, config, [*prefix, cargo], env,
                                     table, selected_layers, layer_omissions)

    # Listing and build belong to the caller's prefix; metadata builds nothing.
    listings = []
    for selected in [selection] if selections is None else selections:
        listed = subprocess.Popen([*prefix, cargo, 'nextest', 'list', '--message-format', 'json',
                                   '--profile', profile, '--config-file', str(Path(config).resolve()), *selected], env=env, stdin=subprocess.DEVNULL,
                                  stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
        listing_output = bytearray()
        try:
            with listed.stdout:
                for line in iter(listed.stdout.readline, b''):
                    listing_output.extend(line)
                    forward(line.decode('utf-8', 'backslashreplace'))
            if listed.wait():
                raise ValueError(f'nextest listing exited with {listed.returncode}')
        except BaseException:
            signal_group(listed, signal.SIGTERM)
            stop(listed, time.monotonic() + WALL_GRACE_S)
            raise
        listings.append(json_listing(listing_output.decode('utf-8', 'backslashreplace')))
    if metadata is None:
        metadata = read_metadata(cargo, env)
    listing = merge_listings(listings) if listings else {
        'rust-build-meta': {'target-directory': metadata['target_directory']}, 'rust-suites': {}}
    if picks is not None:
        wanted = {(binary, name) for binary, name, _ in picks}
        listing['rust-suites'] = {
            binary: dict(suite, testcases={
                name: dict(case, **{'filter-match': {'status': 'matches'}})
                for name, case in suite.get('testcases', {}).items() if (binary, name) in wanted})
            for binary, suite in listing['rust-suites'].items() if suite.get('status') == 'listed'}
    plan = make_plan(listing, metadata, env, cargo, profile, overrides, default_filter,
                     caller_filter or picks is not None, groups, settings, table)
    kept = []
    for binary in plan['binaries']:
        if selected_layers is not None and binary['layer'] is not None and binary['layer'] not in selected_layers:
            layer_omissions.add(binary['binary_id'])
        else:
            kept.append(binary)
    if layer_omissions:
        left_out = [name for name in [*table['layers'], 'checks'] if name not in selected_layers]
        forward(f"layers: leaving out {', '.join(left_out)}: {len(layer_omissions)} binaries\n")
    plan.update(binaries=kept, selected_layers=selected_layers)
    return plan, prefix, no_tests, output, metadata['workspace_root']


def checkout_commit(root, env):
    try:
        result = subprocess.run(['git', 'rev-parse', '--verify', 'HEAD'], cwd=root, env=env,
                                stdin=subprocess.DEVNULL, capture_output=True, text=True)
    except OSError:
        return None
    commit = result.stdout.strip()
    return commit if result.returncode == 0 and re.fullmatch(r'(?:[0-9a-f]{40}|[0-9a-f]{64})', commit) else None


def run(command, env, directory, profile, limits, config=Path('.config/nextest.toml'), walls=None,
        layers=Path('tools/tiers/layers.toml'), ledger=None, selected_layers=None):
    """Run one suite into a fresh directory; return no verdict (design.md, 11)."""
    directory = Path(directory).resolve()
    if directory.exists() and (not directory.is_dir() or any(directory.iterdir())):
        raise ValueError('run directory must be empty')
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    directory.chmod(0o700)
    env = env.copy()
    repair = env.pop('TIERS_PRE_PR_REPAIR_CASE', None)
    plan, prefix, no_tests, output, root = listed_plan(
        command, env, profile, limits, config, walls, layers, selected_layers=selected_layers)
    if repair is not None:
        binary, name = json.loads(repair)
        if not any(b['binary_id'] == binary and any(t['name'] == name for t in b['tests'])
                   for b in plan['binaries']):
            (directory / 'repair-case-missing').touch()
            raise ValueError('repair case absent from focused listing')
        plan['repair_case'] = [binary, name]
    plan['count'] = True
    execute_plan(plan, prefix, env, directory, limits, no_tests=no_tests, output=output, ledger=ledger,
                 commit=checkout_commit(root, env))


def alone_at_base(head_directory, command, env, directory, config=Path('.config/nextest.toml'),
                  layers=Path('tools/tiers/layers.toml')):
    """Run a head's flagged tests at this checkout, with no suite (references.md, 5)."""
    head = json.loads((Path(head_directory) / 'results.json').read_text())
    picks = sorted((entry['binary_id'], entry['test_name'], 'base')
                   for entry in head['references'] if entry['why'] == 'flagged')
    profile = head['profile']
    directory = Path(directory).resolve()
    if directory.exists():
        raise ValueError('base run directory must be fresh')
    directory.mkdir(mode=0o700, parents=True)
    output = bytearray()
    prefix = []
    plan = dict(profile=profile, binaries=[], groups={}, layers=[], workspace=[],
                global_timeout=None, lanes=LANES, term_grace_s=TERM_GRACE_S,
                wall_grace_s=WALL_GRACE_S, leak_s=LEAK_S)
    present = set()
    if picks:
        prefix, cargo, selection, _, _ = arguments(command, profile)
        metadata = json.loads(subprocess.check_output([cargo, 'metadata', '--format-version', '1', '--no-deps'],
                                                    env=env, stdin=subprocess.DEVNULL, text=True))
        wanted = {binary for binary, _, _ in picks}
        packages = {}
        for package in metadata['packages']:
            if package['id'] not in metadata['workspace_members']:
                continue
            found = wanted.intersection(target_binary_id(package, target) for target in package['targets'])
            if found:
                packages[package['name']] = sorted(found)
        selections = [['-p', package, *(flag for binary in packages[package] for flag in target_flags(binary)),
                       *selection] for package in sorted(packages)]
        if selections:
            found = {binary for binaries in packages.values() for binary in binaries}
            found_picks = [pick for pick in picks if pick[0] in found]
            plan, prefix, _, output, _ = listed_plan(command, env, profile, {}, config, None, layers, found_picks,
                                                  metadata=metadata, selections=selections)
        names = {(binary['binary_id'], test['name']) for binary in plan['binaries'] for test in binary['tests']}
        present = {pick for pick in picks if pick[:2] in names}
    results = execute_plan(reference_plan(plan, present), prefix, env, directory, {},
                           output=output, keep_times=False, junit=False, at_base=True)
    missing = [dict(binary_id=binary, test_name=name, why=why, outcome='missing',
                    process=None, empty_process=None, cpu_s=None, runs=[], run_limit_s=None)
               for binary, name, why in picks if (binary, name, why) not in present]
    if missing:
        results['references'] = sorted(results['references'] + missing,
                                       key=lambda entry: (entry['binary_id'], entry['test_name']))
        save(directory / 'results.json', results)
        with (directory / 'command-output.txt').open('a') as log:
            for entry in missing:
                text = f"REFERENCE {entry['binary_id']} {entry['test_name']}: base, missing, CPU unmeasured s\n"
                print(text, end='', flush=True)
                log.write(text)
    return 0


def pick_references(tests, ledger):
    """New and flagged passing tests, in identity order (references.md, 3)."""
    if ledger is None:
        return []
    picked = []
    for test in tests:
        if test['outcome'] != 'passed':
            continue
        binary, name = test['binary_id'], test['test_name']
        walls = ledger['binaries'].get(binary, {}).get('tests', {}).get(name)
        if walls is None:
            picked.append((binary, name, 'new'))
        else:
            median = statistics.median(walls)
            if test['wall_s'] > 2 * median and test['wall_s'] - median >= 0.05:
                picked.append((binary, name, 'flagged'))
    return sorted(picked)


def reference_jobs(binaries, picks):
    """Unsplit reference and empty jobs from binaries and names (references.md, 4)."""
    binaries = {binary['binary_id']: binary for binary in binaries}
    jobs, emptied = [], set()
    for binary_id, name, _ in sorted(picks):
        binary = binaries[binary_id]
        test = next(test for test in binary['tests'] if test['name'] == name)
        jobs.append(dict(binary=binary, tests=[test], kind='reference', diagnostic=None))
        if binary_id not in emptied:
            empty = dict(name='tiers-one-process: no test', priority=0, group=None, kept_s=None)
            jobs.append(dict(binary=binary, tests=[empty], kind='empty', diagnostic=None))
            emptied.add(binary_id)
    return jobs


def reference_plan(plan, picks, first_start=None):
    """A phase executable without a suite through execute_plan (references.md, 4).

    Binaries stay once in the plan, not repeated with each test's job. The lanes
    make its jobs, which get neither a fresh process nor an alone diagnostic.
    """
    return dict(plan, reference_picks=sorted(picks), first_start=first_start, count=False)


def again_plan(run_directory, binary_id, names):
    """Select one finished run's process without listing or building (design.md, 11.9)."""
    plan = json.loads((Path(run_directory) / 'plan.json').read_text())
    binary = next((b for b in plan['binaries'] if b['binary_id'] == binary_id), None)
    if binary is None:
        raise ValueError(f'no binary in plan: {binary_id}')
    if not names:
        raise ValueError('again requires test names')
    if len(set(names)) != len(names):
        raise ValueError('a test name was given twice')
    selected = {test['name']: test for test in binary['tests']}
    for name in names:
        if name not in selected:
            raise ValueError(f'test not selected in plan: {binary_id} {name}')
    return dict(plan, binaries=[dict(binary, tests=[selected[name] for name in sorted(names)])],
                lanes=1, single_process=True, alone_diagnostics=False, layers=[], workspace=[],
                selected_layers=None, count=False)


def again(plan, prefix, env, directory):
    """Run an again plan, keeping its results but not its times (design.md, 11.9)."""
    directory = Path(directory).resolve()
    if directory.exists():
        raise ValueError('again run directory must be fresh')
    directory.mkdir(mode=0o700, parents=True)
    results = execute_plan(plan, prefix, env, directory, {}, keep_times=False)
    summary = CPU['summary'](directory)
    # The ordinary entry point reports through cpu.execute_one_process; again's
    # verdict also treats starvation and unobserved names as non-passes.
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        CPU['print_summary'](summary)
    text = output.getvalue()
    print(text, end='', flush=True)
    with (directory / 'command-output.txt').open('a') as log:
        log.write(text)
    return 100 if results['unobserved'] or any(t['outcome'] != 'passed' for t in results['tests']) else 0


def execute_plan(plan, prefix, env, directory, limits, *, no_tests='fail', output=None, keep_times=True, ledger=None,
                 junit=True, at_base=False, commit=None):
    """Collect the lanes' events into the run directory (design.md, 11.8)."""
    profile = plan['profile']
    output = bytearray() if output is None else output

    def forward(text):
        print(text, end='', flush=True)
        output.extend(text.encode('utf-8', 'backslashreplace'))

    plan = dict(plan, wall_hard_s=limits.get(profile, {}).get('wall_hard_s'))
    save(directory / 'plan.json', plan)
    selected = 0 if 'reference_picks' in plan else sum(len(binary['tests']) for binary in plan['binaries'])
    results = dict(profile=profile, selected=selected, no_tests=no_tests, cancelled=None, counting=None, commit=commit,
                   processes=[], tests=[], unobserved=[], references=[], layers=plan['layers'], workspace=plan['workspace'],
                   selected_layers=plan.get('selected_layers'))
    tests, reference_tests = {}, {}
    picks = plan.get('reference_picks', [])
    reasons = {(binary, name): why for binary, name, why in picks}
    reference_indices, empty_indices = {}, {}
    starts, active = {}, set()
    lane_count = plan['lanes'] or default_lanes()
    code, kill_at = None, None
    wall_starved = False
    forced_at, cancelled_at = None, None

    reported = set()
    round_index = 0
    run_counts, run_limits = {}, {}

    def reference_run(binary, name, why, round_number, final=False):
        key = round_number, binary, name
        test = reference_tests.get(key)
        reference = reference_indices.get(key)
        empty = empty_indices.get((round_number, binary))
        if not final and (reference is None or empty is None):
            return None
        outcome = test['outcome'] if test is not None else 'unobserved'
        if reference is None and outcome not in ('passed', 'failed', 'starved'):
            outcome = 'unobserved'
        cpu = None
        if outcome == 'passed' and reference is not None and empty is not None:
            process, baseline = results['processes'][reference], results['processes'][empty]
            if baseline['exit_code'] == 0:
                cpu = round(max(0.0, process['user_s'] + process['system_s']
                                - baseline['user_s'] - baseline['system_s']), 3)
        if key not in reported:
            forward(f"REFERENCE {binary} {name}: {why}, round {round_number + 1}, {outcome}, "
                    f"CPU {cpu if cpu is not None else 'unmeasured'} s\n")
            reported.add(key)
        return dict(outcome=outcome, process=reference, empty_process=empty, cpu_s=cpu)

    def reference_rows():
        rows = []
        for binary, name, why in picks:
            runs = [reference_run(binary, name, why, number, final=True)
                    for number in range(run_counts.get((binary, name), 1))]
            measured = [run for run in runs if run['cpu_s'] is not None]
            figure = (max if at_base else min)(measured, key=lambda run: run['cpu_s']) if measured else runs[0]
            rows.append(dict(binary_id=binary, test_name=name, why=why, runs=runs,
                             run_limit_s=run_limits.get((binary, name)), **figure))
            if len(runs) > 1:
                forward(f"REFERENCE FIGURE {binary} {name}: {'highest' if at_base else 'lowest'}, "
                        f"CPU {figure['cpu_s'] if measured else 'unmeasured'} s\n")
        return rows

    def event(item):
        nonlocal lane_count, cancelled_at
        kind = item['event']
        if kind == 'lanes':
            lane_count = item['lanes']
        elif kind == 'counting':
            results['counting'] = dict(counted=item['counted'], why=item['why'])
            if not item['counted']:
                forward(f"tiers: not counting instructions: {item['why']}\n")
        elif kind == 'start':
            starts[item['index']] = item['start_s']
            active.add(item['index'])
        elif kind == 'process':
            index = item['index']
            active.discard(index)
            while len(results['processes']) <= index:
                results['processes'].append(None)
            record = item['item']
            binary = record['binary_id']
            record['checks'] = read_checks(checks_path(directory, index), index, binary)
            results['processes'][index] = record
            if record['kind'] == 'reference':
                name = record['reference_name']
                reference_indices[round_index, binary, name] = index
                reference_run(binary, name, reasons[binary, name], round_index)
            elif record['kind'] == 'empty':
                empty_indices[round_index, binary] = index
                for picked_binary, name, why in picks:
                    if picked_binary == binary:
                        reference_run(binary, name, why, round_index)
        elif kind == 'test':
            test = item['item']
            tests[(test['binary_id'], test['test_name'])] = test
            label = dict(passed='PASS', failed='FAIL', crashed='CRASH', starved='STARVED', cancelled='CANCEL')[test['outcome']]
            forward(f"{label} {test['wall_s']:.3f}s {test['binary_id']} {test['test_name']}\n")
            if test['output']:
                forward(test['output'] + '\n')
        elif kind == 'reference':
            test = item['item']
            reference_tests[(round_index, test['binary_id'], test['test_name'])] = test
        elif kind == 'alone':
            test = tests[(item['binary_id'], item['test_name'])]
            test['alone'] = item['outcome']
            if item['outcome'] == 'passed':
                forward(f"{test['binary_id']} {test['test_name']}: may depend on an earlier test: {', '.join(test['before'])}\n")
            else:
                forward(f"{test['binary_id']} {test['test_name']}: failed alone too\n")
            if item['output'] is not None:
                forward(item['output'] + '\n')
        elif kind == 'cancel':
            cancelled_at = item['at_s']
            results['cancelled'] = item['reason']
            forward(f"tiers: starved: {profile}: {item['reason']}\n")

    if selected == 0 and no_tests == 'warn':
        forward('warning: no tests to run\n')

    def run_lanes(path):
        nonlocal code, kill_at, wall_starved, forced_at
        process = subprocess.Popen([*prefix, 'python3', '-I', '-B', str(Path(__file__).resolve()),
                                    'lanes', str(path)], env=env, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
        pending = bytearray()
        try:
            with process.stdout, selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                while selector.get_map():
                    for key, _ in selector.select(0.02):
                        data = os.read(key.fd, 65536)
                        if not data:
                            selector.unregister(key.fileobj)
                            continue
                        pending.extend(data)
                        while b'\n' in pending:
                            line, _, tail = pending.partition(b'\n')
                            pending = bytearray(tail)
                            text = line.decode('utf-8', 'backslashreplace')
                            if text.startswith(EVENT):
                                event(json.loads(text[len(EVENT):]))
                            else:
                                forward(text + '\n')
                    hard = limits.get(profile, {}).get('wall_hard_s')
                    if hard and active and starts and not wall_starved and time.monotonic() - min(starts.values()) > hard:
                        wall_starved = True
                        results['cancelled'] = 'wall-hard'
                        save(directory / 'starved.json', {'wall_hard_s': hard})
                        forward(f'tiers: starved: {profile} exceeded wall hard limit {hard}s\n')
                        signal_group(process, signal.SIGTERM)
                        kill_at = time.monotonic() + WALL_GRACE_S
                    if kill_at is not None and time.monotonic() >= kill_at:
                        forced_at = time.monotonic()
                        break
                if pending:
                    forward(pending.decode('utf-8', 'backslashreplace'))
            if kill_at is not None:
                stop(process, kill_at)
            code = process.wait()
        except BaseException:
            signal_group(process, signal.SIGTERM)
            try:
                stop(process, time.monotonic() + WALL_GRACE_S)
            except BaseException:
                stop(process, time.monotonic())
            raise
        if forced_at is None:
            if active or any(item is None for item in results['processes']):
                raise ValueError('lanes ended without reaping every started process')
            if code != 0:
                raise ValueError(f'lanes exited with {code}')

    if selected or plan.get('reference_picks'):
        run_lanes(directory / 'plan.json')
    if ledger is not None and results['cancelled'] is None and 'reference_picks' not in plan:
        picks = pick_references(tests.values(), ledger)
        reasons = {(binary, name): why for binary, name, why in picks}
        if picks:
            phase = reference_plan(plan, picks, min(starts.values()) if starts else None)
            phase['process_offset'] = len(results['processes'])
            save(directory / 'references-plan.json', phase)
            run_lanes(directory / 'references-plan.json')
    if picks:
        placements = {entry['binary_id']: (entry.get('tree'), entry.get('layer'))
                      for entry in results['workspace']}
        for process in results['processes']:
            if process is not None and process['kind'] in ('tests', 'fresh', 'alone'):
                placements[process['binary_id']] = process.get('tree'), process.get('layer')
        repeated = []
        for binary, name, why in picks:
            first = reference_run(binary, name, why, 0, final=True)
            limit = None
            if not at_base and first['cpu_s'] is not None:
                history = (ledger or {}).get('binaries', {}).get(binary, {}).get('alone', {}).get(name, [])
                main_cpu = statistics.median(history) if history else None
                tree, layer = placements.get(binary, (None, None))
                limit = CPU['reference_limit'](limits.get('gate', {}).get(profile, {}), tree, layer,
                                               binary, name, why, main_cpu, running=True)
            run_limits[binary, name] = limit
            if at_base or (limit is not None and first['cpu_s'] > limit):
                repeated.append((binary, name, why))
                run_counts[binary, name] = 3
        for round_index in (1, 2):
            if not repeated or results['cancelled'] is not None:
                break
            phase = reference_plan(plan, repeated, min(starts.values()) if starts else None)
            phase['process_offset'] = len(results['processes'])
            path = directory / ('plan.json' if at_base else 'references-plan.json')
            save(path, phase)
            run_lanes(path)
        if at_base:
            save(directory / 'plan.json', plan)
    if forced_at is not None:
        recorded = {}
        for index, item in enumerate(results['processes']):
            if item is not None:
                recorded[index] = len(recorded)
        results['processes'] = [item for item in results['processes'] if item is not None]
        reference_indices = {key: recorded[index] for key, index in reference_indices.items()}
        empty_indices = {key: recorded[index] for key, index in empty_indices.items()}
        tests = {key: dict(test, process=recorded.get(test['process']))
                 for key, test in tests.items()
                 if test['process'] in recorded or test['outcome'] in ('passed', 'failed', 'starved')}
    results['tests'] = list(tests.values())
    for binary in ([] if 'reference_picks' in plan else plan['binaries']):
        for test in binary['tests']:
            if (binary['binary_id'], test['name']) not in tests:
                results['unobserved'].append(dict(binary_id=binary['binary_id'], test_name=test['name']))
    results['references'] = reference_rows()
    save(directory / 'results.json', results)
    if junit:
        write_junit(directory, results)
    (directory / 'command-output.txt').write_bytes(output)
    end = max((p['start_s'] + p['wall_s'] for p in results['processes']), default=0.0)
    end = max(end, forced_at or 0.0, cancelled_at or 0.0)
    first = min(starts.values()) if starts else end
    save(directory / 'run.json', dict(profile=profile, mode='one-process', finished=True,
                                     starved=results['cancelled'] is not None, wall_s=end - first,
                                     returncode=code, cancel_code=None, lanes=lane_count))
    if keep_times and results['tests']:
        path = Path(plan['times_path'])
        path.parent.mkdir(parents=True, exist_ok=True)
        try:
            kept = json.loads(path.read_text())
        except FileNotFoundError:
            kept = {}
        for test in results['tests']:
            kept.setdefault(profile, {}).setdefault(test['binary_id'], {})[test['test_name']] = test['wall_s']
        save(path, kept)
    return results


if __name__ == '__main__':
    os.umask(0o077)
    if len(sys.argv) != 3 or sys.argv[1] != 'lanes':
        sys.exit('usage: one_process.py lanes PLAN')
    path = Path(sys.argv[2]).resolve()
    lanes(json.loads(path.read_text()), path.parent)
