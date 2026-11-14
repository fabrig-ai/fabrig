"""One-process suite runner and its lanes (tools/tiers/design.md, 11).

The files crossing the summary boundary follow
 temper/docs/plan/inproc-runner/notes/contracts.md, 2 and
 temper/docs/plan/inproc-fixes/notes/contracts.md, 1.
"""
import contextlib
import fnmatch
import io
import json
import math
import os
from pathlib import Path
import re
import runpy
import selectors
import shutil
import signal
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
LANES = None
TERM_GRACE_S = 10.0
WALL_GRACE_S = 15.0
LEAK_S = 0.1
EVENT = 'tiers-one-process: '


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

    def atom():
        nonlocal position
        if take('not') or take('!'):
            child = atom()
            return lambda test: not child(test)
        if take('('):
            child = union()
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
            return lambda test: name == 'all'
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
            return lambda test: pattern.search(test[name]) is not None
        if operator == '=':
            return lambda test: test[name] == pattern
        if operator == '~':
            return lambda test: pattern in test[name]
        return lambda test: fnmatch.fnmatchcase(test[name], pattern)

    def intersection():
        child = atom()
        while True:
            if take('and') or take('&'):
                left, right = child, atom()
                child = lambda test, left=left, right=right: left(test) and right(test)
            elif take('-'):
                left, right = child, atom()
                child = lambda test, left=left, right=right: left(test) and not right(test)
            else:
                return child

    def union():
        child = intersection()
        while take('or') or take('|') or take('+'):
            left, right = child, intersection()
            child = lambda test, left=left, right=right: left(test) or right(test)
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
        for key in ('threads-required', 'run-extra-args', 'platform'):
            if key in override:
                raise ValueError(f'unsupported override {key}')
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
    directories.extend(str(target / path) for path in build.get('linked-paths', {}))
    directories.append(host_libdir)
    key = 'DYLD_FALLBACK_LIBRARY_PATH' if sys.platform == 'darwin' else 'LD_LIBRARY_PATH'
    if result.get(key):
        directories.append(result[key])
    result[key] = os.pathsep.join(directories)
    return result


def make_plan(listing, metadata, env, cargo, profile, overrides, default_filter, caller_filter, groups, settings):
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
            binaries.append(dict(binary_id=binary_id, path=suite['binary-path'], cwd=suite['cwd'], tests=tests,
                                 env=binary_environment(binary_env, cargo, suite, package, build, profile, host_libdir)))
    return dict(profile=profile, binaries=binaries, groups=groups,
                global_timeout=duration(settings.get('global-timeout')), lanes=LANES,
                term_grace_s=TERM_GRACE_S, wall_grace_s=WALL_GRACE_S, leak_s=LEAK_S,
                times_path=str(times_path))


def split_jobs(plan, lanes):
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
    return sorted(jobs, key=job_order)


def job_order(job):
    return (-max(test['priority'] for test in job['tests']),
            -sum(test['estimate'] for test in job['tests']))


def emit(kind, **fields):
    print(EVENT + json.dumps(dict(event=kind, **fields), allow_nan=False), flush=True)


class BinaryProcess:
    """One owned child, reaped only by wait4 (design.md, 11.4 and 11.6)."""
    def __init__(self, job, index, selector):
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
        names = b'|'.join(re.escape(test['name'].encode()) for test in job['tests'])
        self.announcement = re.compile(rb'(?m)^test (' + names + rb')(?: - should panic)? \.\.\. ')
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
        try:
            self.child = subprocess.Popen(
                [binary['path'], '--exact', '--nocapture', '--test-threads=1', '--include-ignored',
                 '--color', 'never', '--logfile', str(logfile), *[t['name'] for t in job['tests']]],
                cwd=binary['cwd'], env=binary['env'], stdin=subprocess.DEVNULL,
                stdout=output_write, stderr=subprocess.STDOUT, process_group=0)
        except BaseException:
            os.close(read)
            os.close(self.log_writer)
            os.close(output_read)
            self.log_directory.cleanup()
            raise
        finally:
            os.close(output_write)
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
            lines = output.rstrip().splitlines()
            starved = bool(lines and lines[-1].startswith('skein-world: starved:'))
            outcome = 'starved' if starved else 'passed' if result == b'ok' else 'failed'
            self.finish(name, outcome, max(0.0, ended_at - announced_at), output,
                        [entry[0] for entry in self.announcements[:offset]])

    def finish(self, name, outcome, wall, output, before):
        item = dict(binary_id=self.job['binary']['binary_id'], test_name=name, outcome=outcome,
                    wall_s=wall, process=self.index, before=before if outcome in ('failed', 'crashed') else [],
                    alone=None, output=None if outcome == 'passed' else output)
        self.done[name] = item
        if self.job['kind'] != 'alone':
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

    def settle(self, selector, leaked):
        self.completed()
        code = os.waitstatus_to_exitcode(self.status)
        announced = [entry[0] for entry in self.announcements]
        for offset, (name, _, start, announced_at) in enumerate(self.announcements):
            if self.cancelled_at is not None and announced_at >= self.cancelled_at:
                continue
            if name not in self.done:
                end = self.announcements[offset + 1][1] if offset + 1 < len(self.announcements) else len(self.output)
                self.finish(name, 'cancelled' if self.cancelled_at is not None else 'crashed',
                            max(0.0, self.reaped_at - announced_at),
                            bytes(self.output[start:end]).decode('utf-8', 'backslashreplace'), announced[:offset])
        unannounced = [test for test in self.job['tests'] if test['name'] not in announced]
        fresh = []
        if self.cancelled_at is None:
            if not announced:
                for test in unannounced:
                    self.finish(test['name'], 'crashed', self.reaped_at - self.start,
                                bytes(self.output).decode('utf-8', 'backslashreplace'), [])
            elif unannounced and any(t['outcome'] == 'crashed' for t in self.done.values()):
                fresh = unannounced
            else:
                for test in unannounced:
                    self.finish(test['name'], 'crashed', 0.0, 'not run by its binary', [])
            expected = 101 if any(value[0] == b'failed' for value in self.logs.values()) else 0
            if announced and not fresh and code != expected and all(t['outcome'] not in ('crashed', 'cancelled') for t in self.done.values()):
                last = self.done[announced[-1]]
                self.finish(last['test_name'], 'crashed', last['wall_s'],
                            (last['output'] or '') + f'\nprocess ended with status {code}', announced[:-1])
        signal_group(self.child, signal.SIGKILL)
        for fd in list(self.open_fds):
            self.close(fd, selector)
        self.log_directory.cleanup()
        record = dict(binary_id=self.job['binary']['binary_id'], kind=self.job['kind'], tests=announced,
                      user_s=self.usage.ru_utime, system_s=self.usage.ru_stime,
                      wall_s=self.reaped_at - self.start, start_s=self.start,
                      exit_code=code if code >= 0 else None, signal=-code if code < 0 else None, leaked=leaked)
        emit('process', index=self.index, item=record)
        return fresh


def lanes(plan):
    count = plan['lanes'] or len(os.sched_getaffinity(0))
    jobs = split_jobs(plan, count)
    active, processes = [], 0
    cancelled, first, kill_at = None, None, None
    terminated = False

    def terminate(sig, frame):
        nonlocal terminated
        terminated = True

    previous = signal.signal(signal.SIGTERM, terminate)
    emit('lanes', lanes=count, session=os.getsid(0))
    try:
        with selectors.DefaultSelector() as selector:
            while jobs or active:
                now = time.monotonic()
                if cancelled is None and (terminated or (first is not None and plan['global_timeout'] is not None and now - first >= plan['global_timeout'])):
                    cancelled = 'wall-hard' if terminated else 'global-timeout'
                    jobs.clear()
                    emit('cancel', reason=cancelled)
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
                    child = BinaryProcess(jobs.pop(candidate), processes, selector)
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
                    if cancelled:
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


def run(command, env, directory, profile, limits, config=Path('.config/nextest.toml')):
    """Run one suite into a fresh directory; return no verdict (design.md, 11)."""
    directory = Path(directory).resolve()
    if directory.exists() and (not directory.is_dir() or any(directory.iterdir())):
        raise ValueError('run directory must be empty')
    prefix, cargo, selection, no_tests, caller_filter = arguments(command, profile)
    settings, overrides, default_filter, groups = configuration(config, profile, env, limits, caller_filter)
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    directory.chmod(0o700)
    output = bytearray()

    def forward(text):
        print(text, end='', flush=True)
        output.extend(text.encode('utf-8', 'backslashreplace'))

    # Listing and build belong to the caller's prefix; metadata builds nothing.
    listed = subprocess.Popen([*prefix, cargo, 'nextest', 'list', '--message-format', 'json',
                               '--profile', profile, '--config-file', str(Path(config).resolve()), *selection], env=env, stdin=subprocess.DEVNULL,
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
    listing = json_listing(listing_output.decode('utf-8', 'backslashreplace'))
    metadata = json.loads(subprocess.check_output([cargo, 'metadata', '--format-version', '1', '--no-deps'],
                                                env=env, stdin=subprocess.DEVNULL, text=True))
    plan = make_plan(listing, metadata, env, cargo, profile, overrides, default_filter, caller_filter, groups, settings)
    execute_plan(plan, prefix, env, directory, limits, no_tests=no_tests, output=output)


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
                lanes=1, single_process=True, alone_diagnostics=False)


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


def execute_plan(plan, prefix, env, directory, limits, *, no_tests='fail', output=None, keep_times=True):
    """Collect the lanes' events into the run directory (design.md, 11.8)."""
    profile = plan['profile']
    output = bytearray() if output is None else output

    def forward(text):
        print(text, end='', flush=True)
        output.extend(text.encode('utf-8', 'backslashreplace'))

    save(directory / 'plan.json', plan)
    selected = sum(len(binary['tests']) for binary in plan['binaries'])
    results = dict(profile=profile, selected=selected, no_tests=no_tests, cancelled=None,
                   processes=[], tests=[], unobserved=[])
    tests = {}
    starts, active = {}, set()
    lane_count = plan['lanes'] or len(os.sched_getaffinity(0))
    code, kill_at, process = None, None, None
    wall_starved = False
    forced_at = None

    def event(item):
        nonlocal lane_count
        kind = item['event']
        if kind == 'lanes':
            lane_count = item['lanes']
        elif kind == 'start':
            starts[item['index']] = item['start_s']
            active.add(item['index'])
        elif kind == 'process':
            index = item['index']
            active.discard(index)
            while len(results['processes']) <= index:
                results['processes'].append(None)
            results['processes'][index] = item['item']
        elif kind == 'test':
            test = item['item']
            tests[(test['binary_id'], test['test_name'])] = test
            label = dict(passed='PASS', failed='FAIL', crashed='CRASH', starved='STARVED', cancelled='CANCEL')[test['outcome']]
            forward(f"{label} {test['wall_s']:.3f}s {test['binary_id']} {test['test_name']}\n")
            if test['output']:
                forward(test['output'] + '\n')
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
            results['cancelled'] = item['reason']
            forward(f"tiers: starved: {profile}: {item['reason']}\n")

    if selected == 0 and no_tests == 'warn':
        forward('warning: no tests to run\n')
    if selected:
        process = subprocess.Popen([*prefix, 'python3', '-I', '-B', str(Path(__file__).resolve()),
                                    'lanes', str(directory / 'plan.json')], env=env, stdin=subprocess.DEVNULL,
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
    if forced_at is not None:
        recorded = {}
        for index, item in enumerate(results['processes']):
            if item is not None:
                recorded[index] = len(recorded)
        results['processes'] = [item for item in results['processes'] if item is not None]
        tests = {key: dict(test, process=recorded.get(test['process']))
                 for key, test in tests.items()
                 if test['process'] in recorded or test['outcome'] in ('passed', 'failed', 'starved')}
    elif any(item is None for item in results['processes']) or active:
        raise ValueError('lanes ended without reaping every started process')
    if selected and code != 0 and forced_at is None:
        raise ValueError(f'lanes exited with {code}')
    results['tests'] = list(tests.values())
    for binary in plan['binaries']:
        for test in binary['tests']:
            if (binary['binary_id'], test['name']) not in tests:
                results['unobserved'].append(dict(binary_id=binary['binary_id'], test_name=test['name']))
    save(directory / 'results.json', results)
    write_junit(directory, results)
    (directory / 'command-output.txt').write_bytes(output)
    end = max((p['start_s'] + p['wall_s'] for p in results['processes']), default=0.0)
    if forced_at is not None:
        end = max(end, forced_at)
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
    lanes(json.loads(Path(sys.argv[2]).read_text()))
