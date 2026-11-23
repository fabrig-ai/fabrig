"""Check a snapshot of the current working tree (tools/pre-pr/design.md, 1)."""
from collections import defaultdict
from contextlib import nullcontext
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import runpy
import shlex
import subprocess
import sys
import tempfile
import time
import tomllib
import xml.etree.ElementTree as ET


class Tail:
    """Keep a bounded byte tail, including how much preceded it (section 6)."""

    def __init__(self):
        self.data = b''
        self.total = 0

    def add(self, data):
        self.total += len(data)
        self.data = (self.data + data)[-1536:]

    def text(self):
        data = self.data
        text = data.decode('utf-8', errors='replace').rstrip('\r\n') + '\n'
        if self.total == len(data) and len(text.encode()) <= 1536:
            return text
        # The marker and final newline count too. Discard only whole lines;
        # malformed tool bytes may expand when rendered as valid UTF-8.
        data = data[-1480:]
        while True:
            newline = data.find(b'\n')
            data = data[newline + 1:] if newline >= 0 else b''
            text = data.decode('utf-8', errors='replace').rstrip('\r\n') + '\n'
            marker = f'[… {self.total - len(data)} bytes cut]\n'
            if len((marker + text).encode()) <= 1536:
                return marker + text


def tail(text):
    result = Tail()
    result.add(text.encode())
    return result.text()


def error_line(error):
    """The first error line within section 4 and 6's 512-byte bound."""
    message = str(error).split('\n', 1)[0].split('\r', 1)[0]
    return message.encode('utf-8', errors='backslashreplace')[:512].decode('utf-8', errors='ignore')


def run(arguments, cwd, env, output=None, stdout=False):
    """Stream tools and keep only a tail; a JSON consumer prints rendered text."""
    log = Tail()
    try:
        process = subprocess.Popen(arguments, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except OSError as error:
        message = str(error) + '\n'
        print(message, end='', file=sys.stderr, flush=True)
        return 127, tail(message)
    with process:
        if output is not None:
            for line in process.stdout:
                output(line, log)
        else:
            while data := process.stdout.read1(4096):
                log.add(data)
                stream = sys.stdout if stdout else sys.stderr
                stream.buffer.write(data)
                stream.flush()
        code = process.wait()
    return code, log.text()


class RustcDiagnostic:
    """Stream the build log and keep its first coded error for the close hold."""

    def __init__(self):
        self.code = self.message = ''
        self.location = None

    def consume(self, line, log):
        log.add(line)
        sys.stdout.buffer.write(line)
        sys.stdout.flush()
        if not self.code:
            error = re.match(rb'^error\[(E[0-9]{4})\]: (.*)', line[:512])
            if error:
                self.code = error[1].decode()
                self.message = error[2].decode(errors='replace').strip()
        elif self.location is None:
            arrow = line.lstrip()
            self.location = arrow[4:516].strip().decode(errors='replace') if arrow.startswith(b'--> ') else ''

    def reason(self):
        if not self.code:
            return ''
        location = f' {self.location}' if self.location else ''
        text = f'rustc {self.code}{location}: {self.message}'
        # Jig keeps the last 128 bytes, including the newline.
        return text.encode()[:127].decode(errors='ignore') + '\n'


class OneProcessDiagnostic(RustcDiagnostic):
    """Keep the first failed case and its panic alongside compiler diagnostics (section 4.1)."""

    def __init__(self, directory):
        super().__init__()
        self.directory = directory

    def reason(self):
        try:
            results = json.loads((self.directory / 'results.json').read_bytes())
            test = next((test for test in results['tests'] if test['outcome'] in ('failed', 'crashed')), None)
            if test is None:
                return super().reason()
            case, output = test['test_name'], test.get('output') or ''
        except (OSError, ValueError, KeyError, TypeError):
            return super().reason()
        lines = iter(output.splitlines())
        for line in lines:
            panic = re.fullmatch(r"thread '.+' (?:\(\d+\) )?panicked at (.+:\d+:\d+):", line[:1024].strip())
            if not panic:
                continue
            site = panic[1]
            detail = next(lines, '')[:1024].strip()
            assertion = re.fullmatch(r'assertion `left (==|!=) right` failed(?:: (.*))?', detail)
            if assertion:
                detail = assertion[2] or ''
                left, right = (next(lines, '')[:1024].strip() for _ in range(2))
                if left.startswith('left: ') and right.startswith('right: '):
                    op = '!=' if assertion[1] == '==' else '=='
                    values = f'{left[6:].strip()} {op} {right[7:].strip()}'
                    detail = f'{detail}: {values}' if detail else values
            name = case.rsplit('::', 1)[-1]
            text = f'{name} {site}' + (f': {detail}' if detail else '')
            if len(text.encode()) > 127:
                text = text.replace(site, site.rsplit('/', 1)[-1], 1)
            break
        else:
            text = 'failed test ' + case
        return text.encode()[:127].decode(errors='ignore') + '\n'


def git(arguments, cwd, env):
    return subprocess.run(['git'] + arguments, cwd=cwd, env=env,
                          stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=False)


def checked_git(args, root, env, data=None):
    result = subprocess.run(['git', *args], cwd=root, env=env, input=data,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode:
        raise ValueError(result.stderr.decode(errors='replace').strip())
    return result.stdout


def snapshot(root, work, env, source_env, index):
    """Index present file contents, including ignored files tracked by the caller."""
    private = dict(env, GIT_DIR=str(work / '.git'), GIT_WORK_TREE=str(root), GIT_INDEX_FILE=str(index))
    checked_git(['add', '-A', '--', '.'], root, private)
    if (root / '.git').exists():
        if checked_git(['ls-files', '--unmerged', '-z'], root, source_env):
            raise ValueError('resolve and stage the remaining merge conflicts before delivery')
        tracked = checked_git(['ls-files', '-ci', '--exclude-standard', '-z'], root, source_env)
        ignored = [p for p in tracked.split(b'\0') if p and os.path.lexists(root / os.fsdecode(p))]
        if ignored:
            checked_git(['--literal-pathspecs', 'add', '-f', '--pathspec-from-file=-', '--pathspec-file-nul'],
                        root, private, b'\0'.join(ignored) + b'\0')
    return checked_git(['write-tree'], root, private).decode().strip()


def metadata(root, env):
    result = subprocess.run(['cargo', 'metadata', '--offline', '--no-deps', '--format-version', '1'],
                            cwd=root, env=env, stdin=subprocess.DEVNULL,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    sys.stderr.buffer.write(result.stderr)
    sys.stderr.flush()
    if result.returncode:
        raise RuntimeError('cargo metadata: ' + result.stderr.decode(errors='replace'))
    return json.loads(result.stdout)


def members(data):
    return {p['name']: p for p in data['packages'] if p['id'] in data['workspace_members']}


def package_directories(packages):
    return {Path(p['manifest_path']).parent.parts: name for name, p in packages.items()}


def owner(path, directories):
    """The longest leading run of whole path components (section 3), lexically."""
    parts = path.parts
    for length in range(len(parts), -1, -1):
        name = directories.get(parts[:length])
        if name is not None:
            return name
    return None


def lines(root, name):
    path = root / name
    return [line.split('#', 1)[0].strip() for line in path.read_text().splitlines()
            if line.split('#', 1)[0].strip()] if path.exists() else []


def documentation(path):
    return path.endswith('.md') or 'docs' in Path(path).parts[:-1]


def code_inputs(paths, data, root, *, local_manifests=False):
    """Use package ownership and the gate's declared readers (section 3)."""
    directories = package_directories(members(data))
    if local_manifests:
        # Light checks need only know whether any changed file can belong to a package.
        parents = {parent for path in paths if not documentation(path) for parent in Path(path).parents}
        for parent in parents:
            manifest = root / parent / 'Cargo.toml'
            if manifest.is_file() and (parent != Path('.') or 'package' in tomllib.loads(manifest.read_text())):
                directories[(root / parent).parts] = True
    globs = lines(root, '.config/gate/read-by-tests.txt')
    registry = root / '.config/fixture-readers.toml'
    if registry.exists():
        for fixture in tomllib.loads(registry.read_text()).get('fixture', []):
            if fixture.get('reader'):
                globs.extend(fixture['paths'])
            globs.extend(fixture.get('regenerate', []))
    prefixes = [fields[1] for line in lines(root, '.config/gate/always.txt')
                if len(fields := line.split()) == 2]
    selected = []
    for path in paths:
        name = Path(path).name
        registered = any(fnmatch.fnmatchcase(path, pattern) for pattern in globs)
        source = path.endswith('.rs') or name in (
            'Cargo.toml', 'Cargo.lock', 'clippy.toml', 'rustfmt.toml', '.rustfmt.toml',
            'rust-toolchain', 'rust-toolchain.toml')
        # Prose is code only when a reader is declared; directory ownership
        # alone must not turn a package README into a build.
        if source or registered or (not documentation(path) and (
                '.cargo' in Path(path).parts[:-1] or path.startswith(('.config/', 'tools/'))
                or path in ('.temper/pre-pr', '.temper/goal-pre-pr', '.temper/goal-child-pre-pr')
                or (path.startswith('skein/scripts/') and path.endswith('.sh'))
                or any(path.startswith(prefix) for prefix in prefixes)
                or owner(root / path, directories) is not None
                or any(path in (f'{tree}/scripts/old_forms.py', f'{tree}/scripts/old-forms.txt')
                       for tree in ('skein', 'smith', 'jig', 'temper', 'hearth')))):
            selected.append(path)
    return selected


def baselines(name, root, env):
    """Keep each exceeded count, without a bounded-log parsing limit (section 5)."""
    errors = []

    def consume(line, log):
        log.add(line)
        sys.stderr.buffer.write(line)
        sys.stderr.flush()
        match = re.fullmatch(r'([^\n]+): count (\d+), baseline (\d+)\r?\n?',
                             line.decode(errors='replace'))
        if match:
            form, count, limit = match.groups()
            if int(count) > int(limit):
                errors.append(failure(f'old-forms {name} {form}', line.decode(errors='replace'),
                                      (name, form, int(count), int(limit))))

    code, log = run(['python3', '-I', '-B', 'scripts/old_forms.py'], root / name, env, consume)
    if code != 1 or not errors:
        errors = [failure('old-forms ' + name, log)] if code else []
    return code, errors


def citations(tree_id, root, env):
    """Check citations and design sizes in the delivery snapshot."""
    code, log = run(['python3', '-I', '-B', 'tools/citations/citations.py',
                     '--all', '--head', tree_id], root, env, stdout=True)
    return code, [failure('citations', log)] if code else []


def budget_table(root):
    """Validate this tree's figures using the budget runner's own reader."""
    repair = None
    try:
        cpu = runpy.run_path(str(root / 'tools/tiers/cpu.py'))
        repair = cpu.get('REPAIR_LINE')
        cpu['budgets'](root / 'tools/tiers/budgets.toml', layers=root / 'tools/tiers/layers.toml')
    except (OSError, ValueError, KeyError, TypeError, OverflowError, RuntimeError) as error:
        detail = f'tools/tiers/budgets.toml: {error}'
        if repair:
            detail += '\n' + repair
        return [failure('budgets', detail)]
    return []


def close_measurements(runs, partial_binaries):
    """Read the counts of whole binaries from the focused steps (section 4.1)."""
    measured = {}
    for directory in runs:
        results = json.loads((directory / 'results.json').read_bytes())
        counting = results['counting']
        if counting is None and results['selected'] == 0:
            continue
        counting = counting or {}
        if not counting.get('counted'):
            raise ValueError(counting.get('why') or 'counting did not start')
        partial = partial_binaries(results)
        for process in results['processes']:
            binary = process['binary_id']
            key = process['tree'], process['layer']
            if process['kind'] not in ('tests', 'fresh') or binary in partial or None in key:
                continue
            count = process['instructions']
            if type(count) is not int or count < 0:
                raise ValueError(f'{binary} has no instruction count')
            _, total = measured.get(binary, (key, 0))
            measured[binary] = key, total + count
    return measured


def close_budgets(root, repository, env, runs):
    """Judge the default profile's whole-binary measurements at the goal's base (section 4.1)."""
    tools = Path(__file__).resolve().parents[1] / 'tiers'
    counts = runpy.run_path(str(tools / 'counts.py'))
    data = counts['read'](repository, env)
    try:
        published = data['profiles']['default']
        base = counts['base'](data, repository, env)
        old = counts['binaries'](data, 'default', base)
        base_layers, base_cpu = defaultdict(int), defaultdict(float)
        for binary, entry in old.items():
            key = entry['tree'], entry['layer']
            base_layers[key] += entry['instructions']
            base_cpu[key] += published['binaries'][binary]['cpu_s']
    except (KeyError, TypeError):
        print("budgets: no main's counts in this repository (refs/temper/counts); "
              'no verdict at close, an infrastructure fault to repair; the gate still judges', flush=True)
        return []
    if base is None:
        return [failure('budgets', "the goal is older than main's last twenty landings; update from main and close again")]

    cpu = runpy.run_path(str(tools / 'cpu.py'))
    try:
        measured = close_measurements(runs, cpu['partial_binaries'])
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f'budgets: no verdict at close, instruction counts unavailable: {error_line(error)}', flush=True)
        return []
    if not measured:
        return []

    with (root / 'tools/tiers/budgets.toml').open('rb') as file:
        profile = tomllib.load(file).get('gate', {}).get('default', {})
    ceilings = cpu['gate_ceilings'](profile, 'instructions')
    credits = cpu['instruction_move_credits'](profile, published, base_layers, base_cpu)
    head = base_layers.copy()
    grew = defaultdict(list)
    for binary, (key, count) in measured.items():
        before = 0
        if binary in old:
            entry = old[binary]
            old_key = entry['tree'], entry['layer']
            head[old_key] -= entry['instructions']
            if old_key == key:
                before = entry['instructions']
        head[key] += count
        if count > before:
            grew[key].append((count - before, binary))

    findings = []
    for key in sorted({key for key, _ in measured.values()}):
        before, after = base_layers[key], head[key]
        allowance = counts['allowance'](before, credits.get(key, 0))
        ceiling = ceilings.get(key)
        over = ceiling is not None and before <= ceiling < after
        increase = after - before
        if not over and increase <= allowance:
            continue
        growth = f'{100 * increase / before:.2f}%' if before else 'n/a (zero at base)'
        reason = f'over instruction ceiling {ceiling}' if over else 'instruction growth over allowance'
        findings.append(f'{key[0]} default {key[1]}: {reason}, {before} at base, {after} at head, '
                        f'growth {growth}, allowance {allowance} instructions')
        causes = sorted(grew[key], key=lambda cause: (-cause[0], cause[1]))
        findings.extend(f'  {binary}: +{increase} instructions' for increase, binary in causes[:10])
        if len(causes) > 10:
            findings.append(f'  {len(causes) - 10} more binaries')
    if not findings:
        print(f'budgets: within instruction budgets at close (base {base[:10]})', flush=True)
        return []
    report = '\n'.join([*findings, cpu['REPAIR_LINE']])
    if len(report.encode()) >= 1536:
        print(report, flush=True)
    return [failure('budgets', report)]


def merge_tests(root, env):
    """Run check 9 for deliveries changing the merge job (tools/pre-pr/design.md, 4)."""
    code, log = run(['python3', '-I', '-B', 'tools/merge/test_merge.py'], root, env, stdout=True)
    return code, [failure('merge', log)] if code else []


JOURNEY_ROOTS = tuple(f'temper/docs/design/client/{kind}/journeys' for kind in ('web', 'tui'))
def journeys(root):
    errors = []
    for prefix in JOURNEY_ROOTS:
        overlay = root / prefix / 'overlay.css'
        if overlay.is_symlink() or (overlay.exists() and overlay.stat().st_size):
            errors.append(failure('journeys', f'{prefix}/overlay.css must be empty'))
    return errors


def outward(root, env, paths=None):
    """A policy change or a known outward failure requires the repository-wide scan."""
    args = ['python3', '-I', '-B', 'tools/outward/outward.py']
    if paths is None:
        code, log = run(args, root, env)
    else:
        files = [os.fsencode(p) for p in paths if os.path.lexists(root / p)]
        result = subprocess.run([*args, '--paths0'], cwd=root, env=env, input=b'\0'.join(files),
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        log = result.stdout.decode(errors='replace')
        if log:
            print(log, end='', flush=True)
        code = result.returncode
    if code == 1:
        errors = [failure('test fabrig-boundary::outward no_file_names_the_lab', log,
                          ('fabrig-boundary::outward', 'no_file_names_the_lab'))]
    else:
        errors = [failure('check outward', log)] if code else []
    return code, errors


def format_workspace(root, env):
    """Stream the formatter diff and keep its first source location in the verdict."""
    location = ''
    def consume(line, log):
        nonlocal location
        line = line.replace(os.fsencode(root) + b'/', b'')
        log.add(line)
        sys.stdout.buffer.write(line)
        if not location and line.startswith(b'Diff in '):
            location = line[len(b'Diff in '):].decode(errors='replace').rstrip('\r\n:')
    code, log = run(['cargo', 'fmt', '--all', '--', '--check'], root, env, output=consume)
    if code:
        repair = f'{location}: run cargo fmt --all' if location else 'cargo fmt --all -- --check: ' + log
        return [failure('format', repair)]
    return []


def failure(name, log, key=None, package=None):
    return {'name': name, 'log': tail(log), 'key': key, 'package': package, 'inherited': False}


def package_args(packages):
    return [arg for name in packages for arg in ('-p', name)]


def clippy(packages, data, root, env):
    """Key compiler errors by package, code, primary file and message (section 5)."""
    errors = []
    names = {p['id']: p['name'] for p in data['packages']}

    def consume(line, log):
        try:
            event = json.loads(line)
        except (ValueError, UnicodeDecodeError):
            log.add(line)
            sys.stdout.buffer.write(line)
            sys.stdout.flush()
            return
        if event.get('reason') != 'compiler-message':
            return
        message = event['message']
        rendered = message.get('rendered') or message['message'] + '\n'
        print(rendered, end='', flush=True)
        log.add(rendered.encode())
        if message['level'] != 'error':
            return
        package = names.get(event['package_id'], event['package_id'])
        code = (message.get('code') or {}).get('code', '')
        file = next((s['file_name'] for s in message.get('spans', []) if s['is_primary']), '')
        if Path(file).is_absolute() and Path(file).is_relative_to(root):
            file = str(Path(file).relative_to(root))
        key = (package, code, file, message['message'])
        errors.append(failure(f'clippy {package} {code} {file}', rendered, key, package))

    code, log = run(['cargo', 'clippy', '--all-targets', '--message-format=json'] +
                    package_args(packages) + ['--', '-D', 'warnings'], root, env, consume)
    if code and not errors:
        errors.append(failure('check clippy', f'exit {code}\n' + log))
    return code, errors


def test_binaries(data, root):
    """Validate focused-test prerequisites before the checks (tools/tiers/design.md, 2.1)."""
    path = 'tools/tiers/packages.toml'
    with (root / path).open('rb') as source:
        inventory = tomllib.load(source).get('test-binaries', {})
    if not isinstance(inventory, dict):
        raise ValueError(f'refused test-binaries inventory in {path} (need a table)')
    workspace = members(data)
    for name, binaries in inventory.items():
        if name not in workspace:
            raise ValueError(f'refused test-binaries key {name} in {path} (not a workspace package)')
        valid = isinstance(binaries, list)
        if valid:
            valid = all(isinstance(binary, dict) and set(binary) == {'package', 'bin'}
                        and all(isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_-]+', value)
                                for value in binary.values()) for binary in binaries)
        if valid:
            keys = [(binary['package'], binary['bin']) for binary in binaries]
            valid = (keys == sorted(set(keys))
                     and len({binary['bin'] for binary in binaries}) == len(binaries))
        if not valid:
            raise ValueError(f'refused invalid test-binaries list for {name} in {path} '
                             '(need sorted, unique package/bin records and distinct binary names)')
    return inventory


def build_test_binaries(binaries, root, env):
    for binary in binaries:
        code, log = run(['cargo', 'build', '-p', binary['package'], '--bin', binary['bin']], root, env)
        if code:
            return code, [failure('build', f"{binary['package']}/{binary['bin']}: exit {code}\n" + log)]
    return 0, []


def one_process(packages, root, env, reds=(), rerun=0, outcomes=None, profile='default'):
    """Read one-process's exit and run directory, keeping starvation a warning (section 4)."""
    check_name = 'tests' if profile == 'default' else 'fuzzy tests'
    suffix = '' if profile == 'default' else ' fuzzy'
    with tempfile.TemporaryDirectory(prefix='temper-pre-pr-') as directory:
        directory = Path(directory) / ('tests' if profile == 'default' else 'fuzzy')
        arguments = ['python3', '-I', '-B', 'tools/tiers/one-process', '--out', str(directory)]
        arguments += [profile] + package_args(packages) + ['--no-tests=warn']
        test_env = dict(env, TIERS_PRE_PR_TARGETS='1')
        code, log = run(arguments, root, test_env, stdout=True)
        if code == 2 and log.rstrip('\r\n').split('\n')[-1] == 'one-process: nextest listing exited with 101':
            return code, [failure('build' + suffix, 'no tests ran\n' + log)], None
        if code not in (0, 100):
            return code, [failure('check one-process' + suffix, f'exit {code}\n' + log)], None
        starved = None
        errors = []
        source = 'run.json'
        try:
            state = json.loads((directory / source).read_text())
            if not isinstance(state['starved'], bool):
                raise ValueError('starved must be a boolean')
            source = 'results.json'
            results = json.loads((directory / source).read_text())
            if not isinstance(results['tests'], list) or not isinstance(results['unobserved'], list):
                raise ValueError('tests and unobserved must be lists')
            observed = {}
            count = 0
            for test in results['tests']:
                binary, name, outcome = test['binary_id'], test['test_name'], test['outcome']
                if not all(isinstance(value, str) for value in (binary, name, outcome)):
                    raise ValueError('binary_id, test_name and outcome must be strings')
                if test.get('alone') not in (None, 'passed', 'failed'):
                    raise ValueError('invalid alone result')
                before = test.get('before', [])
                if not isinstance(before, list) or not all(isinstance(name, str) for name in before):
                    raise ValueError('before must be a list of names')
                observed[binary, name] = test
                count += outcome in ('starved', 'cancelled')
            if state['starved'] or count:
                starved = (f'pre-pr: {check_name} starved: {count} starved or cancelled, '
                           f'{len(results["unobserved"])} unobserved, a warning '
                           '(skein/docs/foundation/testing-strategy.md, 8.1)')
                print(starved, flush=True)
            if outcomes is not None:
                outcomes.update(observed)
            if code == 0:
                return code, [], starved
            source = 'junit.xml'
            document = ET.parse(directory / source)
            for suite in document.iter('testsuite'):
                binary = suite.get('name', '')
                for case in suite.findall('testcase'):
                    faults = case.findall('failure') + case.findall('error')
                    if not faults:
                        continue
                    name = case.get('name', '')
                    test = observed.get((binary, name), {})
                    if test.get('outcome') == 'cancelled':
                        continue
                    output = '\n'.join(''.join(node.itertext()) for node in faults +
                                       case.findall('system-out') + case.findall('system-err'))
                    if test.get('alone') == 'passed':
                        output += '\nmay depend on an earlier test: ' + ', '.join(test.get('before', []))
                    # Cargo's binary id starts with its owning package's name.
                    package = binary.split('::', 1)[0]
                    errors.append(failure(f'test {binary} {name}', output, (binary, name), package))
        except (OSError, ValueError, KeyError, TypeError, ET.ParseError) as error:
            errors.append(failure('check one-process' + suffix, f'{source}: {error}\n' + log))
            return code or 1, errors, starved
        if not errors:
            errors.append(failure('check one-process' + suffix, 'exit 100 without JUnit failures\n' + log))
        inherited(reds, errors)
        tests_again(errors, observed, rerun, directory, root, env)
        return code, errors, starved


def tests_again(errors, observed, bound, directory, root, env):
    """Replay unlisted failures serially while the first run still exists (section 4)."""
    candidates = sorted((error for error in errors if error['name'].startswith('test ')
                         and not error['inherited']
                         and observed.get(error['key'], {}).get('outcome') in ('failed', 'crashed')),
                        key=lambda error: error['key'])
    if len(candidates) > bound:
        print(f'pre-pr: tests again: none ({len(candidates)} failures, at most {bound} run again)', flush=True)
        return
    for number, error in enumerate(candidates, 1):
        binary, name = error['key']
        test = observed[binary, name]
        names = test.get('before', []) + [name]
        if 'before' not in test:
            print(f'pre-pr: tests again: {binary} {name}: no before recorded; running the test alone', flush=True)
        print(f'pre-pr: tests again: {binary} {name} ({len(names)} tests) …', flush=True)
        start = time.monotonic()
        again = directory.parent / f'again-{number}'
        code, _ = run(['python3', '-I', '-B', 'tools/tiers/one-process', '--again', str(directory),
                       '--out', str(again), binary] + names, root, env, stdout=True)
        passed = False
        if code in (0, 100):
            try:
                results = json.loads((again / 'results.json').read_text())
                passed = any(test['binary_id'] == binary and test['test_name'] == name
                             and test['outcome'] == 'passed' for test in results['tests'])
                if code == 100 and any(test['binary_id'] == binary and test['test_name'] == name
                                       and test['outcome'] in ('failed', 'crashed') for test in results['tests']):
                    error['confirmed_at'] = time.monotonic()
            except (OSError, ValueError, KeyError, TypeError):
                pass
        seconds = time.monotonic() - start
        if passed:
            error['flake'] = (len(names), seconds)
        print(f'pre-pr: tests again: {binary} {name}: {"passed" if passed else "failed"}, '
              f'{seconds:.3f} s', flush=True)


def rerun_bound(root, env):
    """Read temper's one-line process bound once before the cache (section 4)."""
    path = root / os.fsdecode(git(['rev-parse', '--git-path', 'temper/rerun'], root, env).stdout).strip()
    try:
        content = path.read_bytes()
    except FileNotFoundError:
        return 0
    match = re.fullmatch(rb'temper-rerun 1 ([0-9]+)\n?', content)
    if match:
        digits = match[1].lstrip(b'0')
        if len(digits) <= 2:
            value = int(digits or b'0')
            if value <= 16:
                return value
    print('pre-pr: rerun: ignored temper/rerun (malformed)', flush=True)
    return 0


def goal_reds(root, env):
    """Read optional reporting keys without waiting or changing the verdict."""
    paths = [root / os.fsdecode(git(['rev-parse', '--git-path', 'temper/' + name], root, env).stdout).strip()
             for name in ('reds', 'reds-wait')]
    path, marker = paths
    requested = marker.is_file()
    try:
        with path.open('rb') as source:
            size = os.fstat(source.fileno()).st_size
            if size > 2097152:
                print(f'pre-pr: reds: none (temper/reds oversized: {size} bytes)', flush=True)
                return set(), None, requested
            content = source.read()
    except FileNotFoundError:
        return set(), None, requested
    digest = hashlib.sha256(content).hexdigest()
    lines = content.split(b'\n')
    if lines[-1] == b'':
        lines.pop()
    entries = set()
    malformed = 1 if not lines or lines[0] != b'temper-reds 1' else None
    if malformed is None:
        for number, raw in enumerate(lines[1:], 2):
            try:
                line = raw.decode('utf-8')
            except UnicodeDecodeError:
                malformed = number
                break
            test = re.fullmatch(r'- test (\S+) (.+)', line)
            lint = re.fullmatch(r'- lint (\S+) (\S+) (.+)', line)
            if not (test or lint) or line[-1].isspace():
                malformed = number
                break
            entries.add(('test', *test.groups()) if test else ('lint', *lint.groups()))
    if malformed is not None:
        print(f'pre-pr: reds: none (temper/reds malformed at line {malformed})', flush=True)
        return set(), digest, requested
    print(f'pre-pr: reds: {len(entries)} entries (temper/reds)', flush=True)
    return entries, digest, requested


def inherited(reds, errors):
    """Only the goal's tests and coded lints inherit, without their messages (section 5)."""
    for error in errors:
        key = error['key']
        if key is None:
            continue
        if error['name'].startswith('test '):
            entry = ('test', *key)
        elif error['name'].startswith('clippy ') and key[1]:
            entry = ('lint', *key[:3])
        else:
            continue
        if entry in reds:
            error['inherited'] = True


def check_results(tree_id, base_id, verdict, reds, lints, outcomes, errors, root, env):
    """Report observed goal reds and flakes atomically (section 6; forge/reds.md, 5.5)."""
    lines = ['temper-check 1', f'check {tree_id} {base_id or "-"} {"refused" if verdict else "passed"}']
    failed_tests = {e['key'] for e in errors if e['name'].startswith('test ') and 'flake' not in e}
    lint_errors = [e for e in errors if e['name'].startswith('clippy ')]
    clippy_valid = not any(e['name'] == 'check clippy' for e in errors)
    for entry in sorted(reds):
        status = None
        if entry[0] == 'test':
            key = entry[1:]
            outcome = outcomes.get(key, {}).get('outcome')
            if outcome == 'passed':
                status = 'passed'
            elif outcome in ('failed', 'crashed') and key in failed_tests:
                status = 'failed'
        elif entry[1] in lints and clippy_valid:
            if any(e['key'][:3] == entry[1:] for e in lint_errors):
                status = 'failed'
            elif not any(e['package'] == entry[1] and not e['key'][1] for e in lint_errors):
                status = 'passed'
        if status:
            lines.append(status + ' ' + ' '.join(entry))
    for error in sorted((e for e in errors if 'flake' in e), key=lambda e: e['key']):
        lines.append('flake test ' + ' '.join(error['key']))
        lines.extend('| ' + line if line else '|' for line in error['log'].splitlines())
    path = root / os.fsdecode(git(['rev-parse', '--git-path', 'temper/check-results'], root, env).stdout).strip()
    with tempfile.NamedTemporaryFile(mode='w', dir=path.parent, delete=False) as out:
        out.write('\n'.join(lines) + '\n')
    os.replace(out.name, path)


def settled(error):
    return error['inherited'] or 'flake' in error


def failure_groups(errors):
    """The summary's class order; inherited takes precedence over a flake."""
    return ([e for e in errors if not settled(e)],
            sorted((e for e in errors if 'flake' in e and not e['inherited']), key=lambda e: e['key']),
            [e for e in errors if e['inherited']])


def failure_counts(errors):
    return dict(zip(('new', 'flake', 'inherited'), map(len, failure_groups(errors))))


def failure_file(tree_id, base_id, verdict, cached, errors, root, env, scope='full', commits=(), heads=None):
    """Keep every structured failure atomically, before the summary (sections 6–7)."""
    relative = {'child': 'target/temper/goal-child-pre-pr.json',
                'close': 'target/temper/goal-close-pre-pr.json',
                'delivery': 'target/temper/pre-pr-light-failures.json'}.get(scope, 'target/temper/pre-pr-failures.json')
    path = root / relative
    try:
        entries = []
        for kind, group in zip(('new', 'flake', 'inherited'), failure_groups(errors)):
            for error in group:
                key = error['key']
                structured = None
                if key and error['name'].startswith('test '):
                    structured = dict(zip(('kind', 'binary', 'test'), ('test', *key)))
                elif key and error['name'].startswith('clippy '):
                    structured = dict(zip(('kind', 'package', 'code', 'file', 'message'), ('lint', *key)))
                elif key and len(key) == 4 and error['name'].startswith('old-forms '):
                    structured = dict(zip(('kind', 'tree', 'form', 'count', 'baseline'), ('old-forms', *key)))
                entry = {'class': kind, 'name': error['name'], 'key': structured}
                if error['name'] == 'format':
                    entry['detail'] = error['log'].strip()
                entries.append(entry)
        header = {'form': 'pre-pr-failures 1', 'tree': tree_id, 'base': base_id,
                  'cached': cached, 'verdict': 'refused' if verdict else 'passed',
                  'counts': failure_counts(errors)}
        header['comparison'] = 'tree-only'
        if scope == 'delivery':
            header.update(form='pre-pr-light-failures 1', scope='light')
        elif scope == 'child':
            header.update(form='goal-child-pre-pr 1', scope=scope,
                          delivery_head=heads[0], destination=None, requested_destination=commits[0] if commits else None)
        elif scope in ('goal', 'close'):
            header.update(scope=scope, goal_head=heads[0], destination=None, requested_destination=commits[-1] if commits else None)
            if scope == 'close':
                header['form'] = 'goal-close-pre-pr 1'
        text = json.dumps(header, ensure_ascii=False)[:-1] + ', "failures": [\n'
        text += ',\n'.join(json.dumps(entry, ensure_ascii=False) for entry in entries) + '\n]}\n'
        path.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as out:
            out.write(text.encode('utf-8', errors='backslashreplace'))
        os.replace(out.name, path)
        return f'every key in {relative}'
    except (OSError, ValueError) as error:
        return f'not written to {relative}: {error_line(error)}'


def timed(name, action, pending=None):
    print(f'pre-pr: {name} …', flush=True)
    start = time.monotonic()
    result = action()
    if isinstance(result, tuple):
        failed = result[0] != 0
    else:
        failed = bool(result)
    seconds = time.monotonic() - start
    if failed and pending is not None:
        pending[name] = (seconds, result[1] if isinstance(result, tuple) else result)
    else:
        status = 'failed' if failed else 'passed'
        if isinstance(result, tuple) and result[1] and isinstance(result[1], list):
            if all(settled(error) for error in result[1]):
                status = 'inherited'
        print(f'pre-pr: {name}: {status}, {seconds:.3f} s', flush=True)
    return result


def summary(base_id, lints, tests, errors, reason, failures_tail, starved=(), scope='full', commits=()):
    """Keep the first failure whole and count what the host's tail cannot hold."""
    refused = any(not settled(e) for e in errors)
    header = f'pre-pr: {"refused" if refused else "passed"} ({scope}, current tree)\n'
    counts = failure_counts(errors)
    header += (f'failures: {counts["new"]} new, {counts["flake"]} flakes, '
               f'{counts["inherited"]} inherited; {failures_tail}\n')
    sets = f'lints: {" ".join(lints)}\ntests: {" ".join(tests)}\n'
    sets += ''.join(line + '\n' for line in starved)
    if reason:
        sets += tail(reason)
    new, flakes, inherited_errors = failure_groups(errors)
    entries = [e['name'] + '\n' + e['log'] for e in new]
    entries += [f'flake: {e["name"]} (failed under load; passed when its process ran again: '
                f'{e["flake"][0]} tests, {e["flake"][1]:.3f} s)\n' + e['log']
                for e in flakes]
    entries += ['inherited: ' + e['name'] +
                (' (red, temper/reds)' if e['name'].startswith(('test ', 'clippy ')) else '') + '\n'
                for e in inherited_errors]
    # Keep the verdict and file pointer whole, cutting only the sets and warnings.
    room = max(0, 4096 - len(header.encode()) - 1)
    header += sets.encode()[:room].decode('utf-8', errors='ignore') + '\n'
    diagnostic = next((e['diagnostic'] for e in new if e.get('diagnostic')), '') if scope == 'close' else ''
    output = header
    for index, entry in enumerate(entries):
        remaining = len(entries) - index
        marker = f'[… {remaining} more failures]\n'
        if len((output + entry).encode()) + len((marker + diagnostic).encode()) > 8192:
            output += marker
            break
        output += entry
    print(output + diagnostic, end='', flush=True)
    return int(refused)


def goal_checks(root, paths, env, errors, rerun):
    """Run the explicit full diagnostic against the current workspace packages."""
    data = metadata(root, env)
    active = members(data)
    lints = tests = sorted(active)
    errors += timed('clippy', lambda: clippy(lints, data, root, env))[1]
    inventory = test_binaries(data, root)
    helpers = [entry for name in tests for entry in inventory.get(name, [])]
    build_code, found = build_test_binaries(helpers, root, env)
    errors += found
    warnings = []
    if tests and not build_code:
        for profile in ('default', 'fuzzy'):
            _, found, warning = timed(profile, lambda: one_process(tests, root, env, rerun=rerun, profile=profile))
            errors += found
            if warning:
                warnings.append(warning)
            if any(error['name'].startswith('build') for error in found):
                break
    if '.temper/merge' in paths:
        errors += timed('merge', lambda: merge_tests(root, env))[1]
    return lints, tests, warnings


def finish(tree_id, base_id, errors, root, env, scope, commits, heads, lints=(), tests=(), warnings=()):
    refused = any(not settled(e) for e in errors)
    location = failure_file(tree_id, base_id, refused, False, errors, root, env, scope, commits, heads)
    if location.startswith('not written'):
        errors.append(failure('check receipt', location))
    return summary(base_id, lints, tests, errors, None, location, warnings, scope, commits)


def workspace_checks(root, paths, tree_id, env, errors, outcomes, reds=(), *, completed_goal=False, counts_source=None):
    start = time.monotonic()
    if any(p.endswith('.rs') for p in paths):
        errors += format_workspace(root, env)
        if errors:
            return []
    if completed_goal:
        planner = runpy.run_path(str(Path(__file__).with_name('workspace.py')))
        package_paths = [p for p in paths if p.endswith('.rs') or planner['owns_package'](root, p)]
    else:
        package_paths = []
    data = metadata(root, env) if package_paths else {'packages': [], 'workspace_members': []}
    packages = members(data)

    suites, groups, prerequisites = [], {}, {}
    if completed_goal and packages:
        with (root / 'tools/tiers/layers.toml').open('rb') as file:
            layers = tomllib.load(file)
        groups = planner['package_plan'](root, code_inputs(package_paths, data, root), packages, layers)
        suites = sorted(name for targets, names in groups.items() if targets for name in names)
        prerequisites = test_binaries(data, root)
    print(f'pre-pr: selection: {len(paths)} paths, {len(suites)} test packages, '
          f'{time.monotonic() - start:.3f} s', flush=True)

    def command(label, args, diagnostic=None):
        print(f"pre-pr: {label}: {shlex.join(args)}", flush=True)
        start = time.monotonic()
        rc, log = run(args, root, env, output=diagnostic.consume if diagnostic else None, stdout=True)
        print(f'pre-pr: {label}: exit {rc}, {time.monotonic() - start:.3f} s', flush=True)
        if rc:
            error = failure(label, log)
            if diagnostic:
                error['diagnostic'] = diagnostic.reason()
            errors.append(error)
        return rc

    for name in ('skein', 'smith', 'jig', 'temper', 'hearth'):
        if any((p.startswith(name + '/') and not documentation(p)) or p.startswith('tools/old_forms/') for p in paths):
            errors += timed('old-forms ' + name, lambda: baselines(name, root, env))[1]
    python_suites = set()
    for path in paths:
        file = Path(path)
        if path in HOOKS or (path.startswith('tools/pre-pr/') and file.suffix == '.py'):
            python_suites.add('tools/pre-pr/test_pre_pr.py')
        elif completed_goal and file.suffix == '.py':
            suite = file if file.name.startswith('test_') else file.with_name('test_' + file.name)
            if (root / suite).is_file():
                python_suites.add(str(suite))
    for suite in sorted(python_suites):
        command('Python tests', ['python3', '-I', '-B', suite])
    if any(documentation(p) for p in paths):
        errors += timed('citations', lambda: citations(tree_id, root, env))[1]
    if paths:
        key = ('fabrig-boundary::outward', 'no_file_names_the_lab')
        full_scan = ('test', *key) in reds or any(p.startswith('tools/outward/') for p in paths)
        rc, found = timed('outward', lambda: outward(root, env, None if full_scan else paths))
        errors += found
        if full_scan and rc in (0, 1):
            outcomes[key] = {'outcome': 'passed' if rc == 0 else 'failed'}
    if 'tools/tiers/cpu.py' in paths or 'tools/tiers/budgets.toml' in paths:
        errors += timed('budgets', lambda: budget_table(root))
    errors += journeys(root)
    if groups:
        scratch = nullcontext()
        if suites:
            target = Path(data['target_directory'])
            target.mkdir(parents=True, exist_ok=True)
            scratch = tempfile.TemporaryDirectory(prefix='pre-pr-close-', dir=target)
        with scratch as directory:
            runs = []
            for args in planner['rust_commands'](groups, prerequisites, Path(directory) if suites else None):
                if args[0] == 'tools/tiers/one-process':
                    out = Path(args[2])
                    runs.append(out)
                    if command('focused tests', args, OneProcessDiagnostic(out)):
                        break
                elif command('build check', args, RustcDiagnostic()):
                    break
            else:
                if runs:
                    repository, git_env = counts_source if counts_source is not None else (root, env)
                    errors += close_budgets(root, repository, git_env, runs)
    return suites


HOOKS = {'.temper/pre-pr', '.temper/goal-child-pre-pr', '.temper/goal-pre-pr'}


def check(scope, commits):
    root = Path.cwd()
    if any(not re.fullmatch(r'(?:[0-9a-f]{40}|[0-9a-f]{64})', commit) for commit in commits):
        raise ValueError('expected full hexadecimal commit IDs')
    source_env = dict(os.environ, GIT_ALLOW_PROTOCOL='', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env = {k: v for k, v in source_env.items() if not k.startswith('GIT_')}
    env.update(CARGO_NET_OFFLINE='true', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')
    env.setdefault('CARGO_TARGET_DIR', str(root / 'target'))
    env.pop('TIERS_LEDGER', None)
    has_git = (root / '.git').exists()

    def current_head():
        if not has_git:
            return None
        result = git(['rev-parse', '--verify', '--quiet', 'HEAD'], root, source_env)
        return result.stdout.decode().strip() if result.returncode == 0 else None

    head = current_head()
    if scope == 'goal' and commits and head and head != commits[0]:
        raise ValueError('checkout HEAD differs from the requested goal')
    errors, outcomes, warnings, lints, tests = [], {}, [], [], []
    with tempfile.TemporaryDirectory(prefix='pre-pr-tree-') as directory:
        work = Path(directory) / 'current'
        fmt = checked_git(['rev-parse', '--show-object-format'], root, source_env).decode().strip() if has_git else 'sha1'
        checked_git(['init', '-q', '--object-format=' + fmt, str(work)], root, env)
        index = work / '.git/index'
        tree_id = snapshot(root, work, env, source_env, index)
        checked_git(['checkout-index', '-a'], work, env)
        paths = [os.fsdecode(p) for p in checked_git(['ls-files', '-z'], work, env).split(b'\0') if p]
        receipt_head = head
        try:
            if scope == 'goal' and head:
                # Only the current commit can bind this snapshot to a goal head.
                current = git(['rev-parse', '--verify', '--quiet', 'HEAD^{tree}'], root, source_env)
                if current.returncode or current.stdout.decode().strip() != tree_id:
                    receipt_head = None
                    if commits and current.returncode == 0:
                        raise ValueError('working tree differs from the requested goal head')
            if scope == 'close' and has_git and checked_git(['write-tree', '--missing-ok'], root, source_env).decode().strip() != tree_id:
                raise ValueError('working tree differs from the staged close tree')
            tests = workspace_checks(work, paths, tree_id, env, errors, outcomes, completed_goal=scope == 'close',
                                     counts_source=(root, source_env))
            if scope == 'goal' and not errors:
                lints, tests, warnings = goal_checks(work, paths, env, errors, rerun_bound(root, source_env) if has_git else 0)
            if scope == 'close' and has_git and checked_git(['write-tree', '--missing-ok'], root, source_env).decode().strip() != tree_id:
                raise ValueError('staged close tree changed during checks')
            if tree_id != snapshot(root, work, env, source_env, Path(directory) / 'final-index') or head != current_head():
                raise ValueError('the checkout changed during checks')
        except (OSError, ValueError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
            errors.append(failure('workspace checks', str(error)))
        verdict = finish(tree_id, None, errors, root, env, scope, commits, [receipt_head], lints, tests, warnings)
        if has_git and scope == 'delivery':
            reds, _, marker = goal_reds(root, source_env)
            if marker:
                check_results(tree_id, None, verdict, reds, lints, outcomes, errors, root, source_env)
        return verdict


def main(args):
    if not args:
        return check('delivery', ())
    if args[0] == '--close' and len(args) in (1, 2):
        return check('close', args[1:])
    if args[0] == '--child' and len(args) in (1, 2):
        return check('child', args[1:])
    if args[0] == '--goal' and len(args) in (1, 3):
        return check('goal', args[1:])
    raise ValueError('expected no arguments, --goal [HEAD DESTINATION], --child [DESTINATION], or --close [DESTINATION]')


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1:]))
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
        sys.exit(f'pre-pr: refused: {error}')
