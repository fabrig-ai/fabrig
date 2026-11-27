"""Check a snapshot of the current working tree (tools/pre-pr/design.md, 1)."""
from collections import defaultdict
from contextlib import nullcontext
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import pwd
import re
import runpy
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import xml.etree.ElementTree as ET


TIERS = Path(__file__).resolve().parents[1] / 'tiers'
# Close and the gate count some binaries differently (cargo-spawning boundary tests about 7% lower,
# client worlds up to 22% higher, 11-23), so a layer's sum can swing about 2% between them: close
# refuses only growth past the allowance by more than this share of the layer's base (section 4.1).
CLOSE_MARGIN = 0.03
# None is a decision: the checkout holds no counts.
UNSELECTED_COUNTS = object()


def planner():
    return runpy.run_path(str(Path(__file__).with_name('workspace.py')))


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
    result.add(text.encode('utf-8', errors='backslashreplace'))
    return result.text()


def first_line(error):
    return str(error).split('\n', 1)[0].split('\r', 1)[0]


def error_line(error):
    """The first error line within section 4 and 6's 512-byte bound."""
    return first_line(error).encode('utf-8', errors='backslashreplace')[:512].decode('utf-8', errors='ignore')


def stderr_line(text):
    """Bound the last line by the worker's stderr tail, including the newline."""
    return error_line(text).encode()[:127].decode('utf-8', errors='ignore') + '\n'


def print_broke(detail):
    sys.stderr.write('smith-check: not judged\n' + stderr_line(f'pre-pr: broke: {detail}'))
    sys.stderr.flush()


def run(arguments, cwd, env, output=None, stdout=False):
    """Stream tools and keep only a tail; a JSON consumer prints rendered text."""
    log = Tail()
    try:
        process = subprocess.Popen(arguments, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except OSError as error:
        message = str(error) + '\n'
        print(message, end='', file=sys.stderr, flush=True)
        return 127, first_line(error) + '\n'
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
        return stderr_line(text)


def test_diagnostic(case, output):
    """Keep a failed case and its panic for the close hold (section 4.1)."""
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
        if len(text.encode('utf-8', errors='backslashreplace')) > 127:
            text = text.replace(site, site.rsplit('/', 1)[-1], 1)
        return stderr_line(text)
    return stderr_line('failed test ' + case)


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


# Delivery and child change selection starts from the worker's landing pin (section 1).
BASES = {'delivery': ('refs/temper/landing', 'refs/remotes/origin/main'),
         'child': ('refs/temper/landing', 'refs/remotes/origin/main')}


UNCHECKED_SAVE = 'Unchecked: work in progress, saved before a question'


def change_base(root, env, scope):
    """The merge base of HEAD, any pending MERGE_HEAD and the scope's first base reference present.

    The worker's unchecked saves are stepped past along first parents, so their edits are checked.
    """
    heads = ['HEAD']
    if git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD^{commit}'], root, env).returncode == 0:
        heads.append('MERGE_HEAD')
    for name in BASES.get(scope, ()):
        result = git(['merge-base', name, *heads], root, env)
        if result.returncode or not result.stdout.strip():
            continue
        commit = result.stdout.decode().split()[0]
        name = f'merge base of {name} and HEAD'
        while git(['log', '-1', '--format=%s', commit], root, env).stdout.decode().rstrip('\n') == UNCHECKED_SAVE:
            parent = git(['rev-parse', '--verify', '--quiet', commit + '^1'], root, env)
            if parent.returncode:
                break
            commit = parent.stdout.decode().strip()
        return commit, name
    return None, None


def close_job_goal(env):
    """Read the task number only from a valid close request (tooling.md, 4.2)."""
    if job := env.get('TEMPER_JOB'):
        try:
            request = (Path(job) / 'request.json').read_bytes()
            parse = runpy.run_path(str(TIERS.parent / 'close/close.py'))['parse_request']
            goal, _ = parse(request)
            return str(goal).encode('ascii')
        except (OSError, ValueError):
            pass
    return None


def main_goal_landing(root, env):
    """Find the newest other goal's landing that HEAD holds (section 4.1)."""
    if git(['rev-parse', '--verify', '--quiet', 'HEAD^{commit}'], root, env).returncode:
        return None
    own_goal = close_job_goal(env)
    # Descendants come before ancestors even when their commit dates disagree.
    history = checked_git(['log', '--merges', '--topo-order', '--no-notes', '--no-show-signature', '-z',
                           '--format=%H%n%(trailers:key=Goal,valueonly,unfold)', 'HEAD'], root, env)
    for record in history.split(b'\0'):
        commit, _, goals = record.partition(b'\n')
        if any(goal != own_goal and re.fullmatch(rb'[1-9][0-9]*', goal) for goal in goals.split(b'\n')):
            return commit.decode('ascii')
    return None


def scope_base(root, env, scope, has_git, published=None):
    """The base a delivery, child or close check compares with, or why there is none."""
    if not has_git:
        return None, None, 'no base: this is no git checkout, so nothing says what changed'
    if scope == 'close':
        if published is None:
            try:
                base = main_goal_landing(root, env)
            except ValueError as error:
                return None, None, f'no base: {error_line(error)}'
            if base is None:
                return None, None, ("no base: main's counts are not in this checkout, "
                                    "and HEAD holds no other goal's landing into main")
            name = "main's newest goal landing in HEAD"
        else:
            # Main as last merged by the goal, also the base of its budget comparison (section 4.1).
            if problem := budget_problem(published):
                return None, None, f'no base: {problem}'
            base, name = published['base'], "main's newest published commit in HEAD"
    else:
        try:
            base, name = change_base(root, env, scope)
        except ValueError as error:
            return None, None, f'no base: {error_line(error)}'
    lacks = "; the worker's checkout lacks what pre-PR compares with"
    if base is None:
        return None, None, f"no base: this checkout has no {' or '.join(BASES[scope])} sharing history with HEAD" + lacks
    if scope == 'close' and base == git(['rev-parse', 'HEAD'], root, env).stdout.decode().strip():
        # The goal's commits are already on main as this checkout has it, or there are none.
        return None, None, f'no base: the base found, {name}, is HEAD itself, so nothing says what the goal changed' + lacks
    return base, name, None


def changed_paths(root, env, base):
    """Paths whose present files differ from base: committed, staged, unstaged or new.

    A merge of main brings main's paths in too, which only widens the checks.
    """
    names = set()
    # diff-index reads no diff configuration and leaves the checkout's index as it is.
    for args in (['diff-index', '--name-only', '--no-renames', '-z', base, '--'],
                 ['ls-files', '--others', '--exclude-standard', '-z']):
        names.update(os.fsdecode(p) for p in checked_git(args, root, env).split(b'\0') if p)
    return sorted(names)


def git_path(root, env, name):
    return Path(os.fsdecode(checked_git(['rev-parse', '--path-format=absolute', '--git-path', name],
                                        root, env)).strip())


def seed(root, work, source_env, index):
    """Start from the checkout's own index and objects, so that only changed files are hashed.

    copy2 keeps the index's time, so git still sees a same-size edit made in its second.
    """
    try:
        source = git_path(root, source_env, 'index')
        objects = git_path(root, source_env, 'objects')
    except ValueError:
        return False
    if not source.is_file() or not objects.is_dir():
        return False
    # Assume-unchanged and skip-worktree entries would keep their indexed blob, not the file.
    tags = checked_git(['ls-files', '-v', '-z'], root, source_env).split(b'\0')
    if any(tag[:1].islower() or tag[:1] == b'S' for tag in tags if tag):
        return False
    shutil.copy2(source, index)
    borrow_objects(work, objects)
    return True


def borrow_objects(work, objects):
    alternates = work / '.git/objects/info/alternates'
    alternates.parent.mkdir(parents=True, exist_ok=True)
    alternates.write_text(str(objects) + '\n')


def snapshot(root, work, env, source_env, index):
    """Index present file contents, including ignored files tracked by the caller."""
    private = dict(env, GIT_DIR=str(work / '.git'), GIT_WORK_TREE=str(root), GIT_INDEX_FILE=str(index))
    seeded = (root / '.git').exists() and not index.exists() and seed(root, work, source_env, index)
    try:
        checked_git(['add', '-A', '--', '.'], root, private)
    except ValueError:
        if not seeded:
            raise
        # An index git can't extend here (a split index, say) is replaced by a fresh one.
        index.unlink()
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


class CheckFindings(Exception):
    """A check prevented selection, with its own findings rather than a checker exception."""

    def __init__(self, errors):
        super().__init__()
        self.errors = errors


def metadata(root, env):
    try:
        result = subprocess.run(['cargo', 'metadata', '--offline', '--no-deps', '--format-version', '1'],
                                cwd=root, env=env, stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    except OSError as error:
        raise CheckFindings([broken('cargo metadata', error)]) from error
    sys.stderr.buffer.write(result.stderr)
    sys.stderr.flush()
    if result.returncode:
        log = result.stderr.decode(errors='replace')
        raise CheckFindings(command_errors('cargo metadata', result.returncode, log))
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


def test_input_globs(root):
    """The gate's declared test inputs: read-by-tests globs and the fixture readers' paths."""
    globs = lines(root, '.config/gate/read-by-tests.txt')
    registry = root / '.config/fixture-readers.toml'
    if registry.exists():
        for fixture in tomllib.loads(registry.read_text()).get('fixture', []):
            if fixture.get('reader'):
                globs.extend(fixture['paths'])
            globs.extend(fixture.get('regenerate', []))
    return globs


def workspace_wide(path):
    """An input that can change how every package builds or tests."""
    return (path in ('Cargo.lock', 'Cargo.toml', 'rust-toolchain', 'rust-toolchain.toml', '.config/nextest.toml')
            or path.startswith('.cargo/'))


def external_locks(text):
    """The registry and git packages a Cargo.lock pins, by name, version and source."""
    return {(package['name'], package['version'], package['source'])
            for package in tomllib.loads(text).get('package', []) if package.get('source')}


def lock_moves_externals(repository, env, base, root):
    """Whether Cargo.lock moved a package from outside the workspace since base.

    A workspace member's own edges in the lock come with its manifest, which selects it already.
    """
    try:
        before = checked_git(['show', f'{base}:Cargo.lock'], repository, env).decode()
        return external_locks(before) != external_locks((root / 'Cargo.lock').read_text())
    except (ValueError, OSError, UnicodeDecodeError, tomllib.TOMLDecodeError):
        return True


def manifest_beyond_members(text):
    """The root manifest without its members and the workspace's own path dependencies."""
    manifest = tomllib.loads(text)
    workspace = manifest.get('workspace', {})
    workspace.pop('members', None)
    dependencies = workspace.get('dependencies', {})
    for name in [name for name, value in dependencies.items() if isinstance(value, dict) and 'path' in value]:
        del dependencies[name]
    return manifest


def root_manifest_moves_more(repository, env, base, root):
    """Whether the root manifest changed beyond adding or removing the workspace's own crates."""
    try:
        before = checked_git(['show', f'{base}:Cargo.toml'], repository, env).decode()
        return manifest_beyond_members(before) != manifest_beyond_members((root / 'Cargo.toml').read_text())
    except (ValueError, OSError, UnicodeDecodeError, tomllib.TOMLDecodeError):
        return True


def always_packages(root, inputs, packages):
    """The gate's always-run packages for these code inputs (.config/gate/README.md, The gate's lists):
    a boundary package where the inputs reach a package, a prefixed one where an input has its prefix."""
    directories = package_directories(packages)
    reached = any(owner(root / path, directories) is not None for path in inputs)
    selected = []
    for line in lines(root, '.config/gate/always.txt'):
        name, *prefix = line.split()
        if name in packages and (any(path.startswith(prefix[0]) for path in inputs) if prefix else reached):
            selected.append(name)
    return selected


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
    globs = test_input_globs(root)
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
                or path in ('.temper/hooks/deliver', '.temper/hooks/finish', HOOK_CONFIG,
                            '.temper/pre-pr', '.temper/goal-pre-pr', '.temper/goal-child-pre-pr')
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
    if code < 0:
        errors += [command_break('old-forms ' + name, code, log)]
    elif code != 1 or not errors:
        errors = command_errors('old-forms ' + name, code, log)
    return code, errors


def citations(tree_id, root, env):
    """Check citations and design sizes in the delivery snapshot."""
    code, log = run(['python3', '-I', '-B', 'tools/citations/citations.py',
                     '--all', '--head', tree_id], root, env, stdout=True)
    return code, command_errors('citations', code, log)


def budget_table(root):
    """Validate this tree's figures using the budget runner's own reader."""
    cpu = runpy.run_path(str(root / 'tools/tiers/cpu.py'))
    repair = cpu.get('REPAIR_LINE')
    try:
        cpu['budgets'](root / 'tools/tiers/budgets.toml', layers=root / 'tools/tiers/layers.toml')
    except (ValueError, KeyError, TypeError, OverflowError, RuntimeError) as error:
        detail = f'tools/tiers/budgets.toml: {error}'
        if repair:
            detail += '\n' + repair
        return [failure('budgets', detail)]
    return []


def close_measurements(runs, partial_binaries):
    """Read the counts of whole binaries from the focused steps (section 4.1)."""
    measured = {}
    landed = {f'fabrig-boundary::{name}' for name in planner()['LANDED_CASES']}
    for directory in runs:
        results = json.loads((directory / 'results.json').read_bytes())
        counting = results['counting']
        if counting is None and results['selected'] == 0:
            continue
        counting = counting or {}
        if not counting.get('counted'):
            raise ValueError(counting.get('why') or 'counting did not start')
        # The gate measures these binaries with their landed cases included.
        partial = partial_binaries(results) | landed
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


def budget_input(counts, repository, env, data):
    """Select the applicable published counts while the real checkout is available."""
    if 'unavailable' in data:
        return data
    if not (repository / '.git').exists() or git(['rev-parse', '--verify', '--quiet', 'HEAD^{commit}'],
                                                repository, env).returncode:
        return {'unavailable': 'the source checkout has no commit for budget selection'}
    window = data.get('commits')
    if not isinstance(window, list) or any(not isinstance(commit, str) or
            not re.fullmatch(r'(?:[0-9a-f]{40}|[0-9a-f]{64})', commit) for commit in window):
        return {'unavailable': 'invalid published commit window'}
    base = counts['base'](data, repository, env)
    if base is None:
        shallow = git(['rev-parse', '--is-shallow-repository'], repository, env)
        if shallow.stdout.strip() != b'false' or git(['rev-list', '--count', 'HEAD'], repository, env).returncode:
            return {'unavailable': 'source ancestry is incomplete; update from main and close again'}
    return {**data, 'base': base}


def budget_problem(data):
    """Why close cannot compare the goal with the published window (section 4.1)."""
    prefix = "main's published counts unavailable"
    if 'unavailable' in data:
        return f"{prefix}: {data['unavailable']}"
    if 'base' not in data or not isinstance(data.get('profiles'), dict):
        return f'{prefix}: invalid published counts'
    if data['base'] is None:
        return "the goal is older than main's last twenty landings; update from main and close again"
    return None


def close_counts(repository, env):
    """Select main's counts once for both close checks, also retaining the local source (section 4.1)."""
    if not (repository / '.git').exists():
        return None, None
    env = dict(env, GIT_ALLOW_PROTOCOL='', GIT_NO_LAZY_FETCH='1')
    tip = git(['rev-parse', '--verify', '--quiet', 'refs/temper/counts^{commit}'], repository, env)
    if tip.returncode == 0:
        counts = runpy.run_path(str(TIERS / 'counts.py'))
        ref = tip.stdout.decode().strip()
        data = counts['read'](repository, env, ref=ref)
        if data is not None:
            return budget_input(counts, repository, env, data), ref
    return None, None


def close_budgets(root, repository, env, runs, published=UNSELECTED_COUNTS):
    """Judge whole binaries against the selected published budget input, profile by profile (section 4.1).

    runs maps each profile to its run directories; a bare list is the focused profile's.
    Standalone callers can omit published; an explicit None needs no lookup or judgement.
    """
    data = published
    if data is UNSELECTED_COUNTS:
        try:
            data, _ = close_counts(repository, env)
        except (ValueError, OSError) as error:
            data = {'unavailable': error_line(error)}
    if data is None:
        return []
    if problem := budget_problem(data):
        return [broken('budgets', problem)]
    by_profile = runs if isinstance(runs, dict) else {'default': runs}
    base = data['base']

    counts = runpy.run_path(str(TIERS / 'counts.py'))
    cpu = runpy.run_path(str(TIERS / 'cpu.py'))
    with (root / 'tools/tiers/budgets.toml').open('rb') as file:
        gate = tomllib.load(file).get('gate', {})
    findings, warnings, figures, judged = [], [], [], 0
    for name, profile_runs in by_profile.items():
        which = '' if name == 'default' else name + ' '
        try:
            profile_counts = data['profiles'][name]
            old = counts['binaries'](data, name, base)
            base_layers, base_cpu = defaultdict(int), defaultdict(float)
            for binary, entry in old.items():
                key = entry['tree'], entry['layer']
                base_layers[key] += entry['instructions']
                base_cpu[key] += profile_counts['binaries'][binary]['cpu_s']
        except (KeyError, TypeError, ValueError):
            print(f"budgets: no {which}counts in main's published counts; no {which}verdict at close", flush=True)
            continue
        try:
            measured = close_measurements(profile_runs, cpu['partial_binaries'])
        except (OSError, ValueError, KeyError, TypeError) as error:
            print(f'budgets: no {which}verdict at close, instruction counts unavailable: {error_line(error)}',
                  flush=True)
            continue
        if not measured:
            continue
        judged += 1
        profile = gate.get(name, {})
        ceilings = cpu['gate_ceilings'](profile, 'instructions')
        credits = cpu['instruction_move_credits'](profile, profile_counts, base_layers, base_cpu)
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

        for key in sorted({key for key, _ in measured.values()}):
            before, after = base_layers[key], head[key]
            accepted = profile.get(key[0], {}).get(key[1], {}).get('accepted_instructions')
            allowance = counts['allowance'](before, credits.get(key, 0), accepted)
            ceiling = ceilings.get(key)
            limit = max(ceiling, accepted) if ceiling is not None and accepted is not None else ceiling
            over = limit is not None and before <= limit < after
            increase = after - before
            if not over and increase <= allowance:
                continue
            margin = int(before * CLOSE_MARGIN)
            sure = after > limit + margin if over else increase > allowance + margin
            found = findings if sure else warnings
            growth = f'{100 * increase / before:.2f}%' if before else 'n/a (zero at base)'
            reason = 'instruction growth over allowance'
            if over:
                bound = 'accepted instruction level' if limit != ceiling else 'instruction ceiling'
                reason = f'over {bound} {limit}'
            found.append(f'{key[0]} {name} {key[1]}: {reason}, {before} at base, {after} at head, '
                         f'growth {growth}, allowance {allowance} instructions')
            causes = sorted(grew[key], key=lambda cause: (-cause[0], cause[1]))
            if sure:
                figures.append(dict(tree=key[0], profile=name, layer=key[1], base=before, head=after,
                                    allowance=allowance, limit=limit if over else None, margin=margin,
                                    base_commit=base,
                                    binaries=[dict(binary=binary, growth=grown) for grown, binary in causes[:10]]))
            found.extend(f'  {binary}: +{increase} instructions' for increase, binary in causes[:10])
            if len(causes) > 10:
                found.append(f'  {len(causes) - 10} more binaries')
    if warnings:
        print(f'budgets: past the allowance at close by less than its {CLOSE_MARGIN:.0%} margin, a warning; '
              'the landing gate judges:\n' + '\n'.join(warnings), flush=True)
    if not findings:
        if judged and not warnings:
            print(f'budgets: within instruction budgets at close (base {base[:10]})', flush=True)
        return []
    report = '\n'.join([*findings, cpu['REPAIR_LINE']])
    if len(report.encode()) >= 1536:
        print(report, flush=True)
    error = failure('budgets', report)
    error['figures'] = figures
    error['diagnostic'] = budget_line(figures[0]) + '\n'
    return [error]


def budget_line(figure):
    """One refusing layer in the hold's 128 bytes: what grew, by how much, against what."""
    head = f"budgets {figure['tree']} {figure.get('profile', 'default')} {figure['layer']}: "
    if figure['limit'] is not None:
        text = f"over {figure['limit'] / 1e9:.1f}e9 at {figure['head'] / 1e9:.1f}e9"
    else:
        base = figure['base'] or 1
        text = (f"+{100 * (figure['head'] - figure['base']) / base:.2f}%, allowance "
                f"{100 * figure['allowance'] / base:.2f}%")
    text += f", {figure['base'] / 1e9:.1f}e9 to {figure['head'] / 1e9:.1f}e9"
    if figure['binaries']:
        text += f", most {figure['binaries'][0]['binary']}"
    return (head + text).encode()[:127].decode(errors='ignore')


def merge_tests(root, env):
    """Run check 9 for deliveries changing the merge job (tools/pre-pr/design.md, 4)."""
    code, log = run(['python3', '-I', '-B', 'tools/merge/test_merge.py'], root, env, stdout=True)
    return code, command_errors('merge', code, log)


JOURNEY_ROOTS = tuple(f'temper/docs/design/client/{kind}/journeys' for kind in ('web', 'tui'))
JOURNEY_ASSETS = {'skein-ui.css', 'journey.js', 'theme.css', 'tui-journey.js', 'looks.html'}


def written_journey_file(path):
    for prefix in JOURNEY_ROOTS:
        if path.startswith(prefix + '/'):
            parts = path[len(prefix) + 1:].split('/')
            if len(parts) == 1:
                return parts[0] in JOURNEY_ASSETS
            return len(parts) == 2 and bool(parts[0]) and (
                parts[1] == 'recording' or re.fullmatch(r'[0-9]{2,}\.(?:html|txt)', parts[1]) is not None)
    return False


def journey_paths(root, source_env, work, env, tree_id, paths, base_id, head):
    """For check 8 only, leave out entries a merged tip holds as they stand in the snapshot."""
    if base_id is None:
        return changed_paths(root, source_env, head) if head else []
    if not paths or not head:
        return paths

    pending = git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD^{commit}'], root, source_env)
    tips = pending.stdout.decode().split() if pending.returncode == 0 else []
    merges = checked_git(['rev-list', '--first-parent', '--merges', '--parents', f'{base_id}..HEAD'], root, source_env)
    for line in merges.decode().splitlines():
        tips.extend(line.split()[2:])
    if not tips:
        return paths

    # Both the snapshot and the source's tips must be visible to diff-tree, even if seed skipped the index.
    if not (work / '.git/objects/info/alternates').exists():
        borrow_objects(work, git_path(root, source_env, 'objects'))
    own = set(paths)
    for tip in dict.fromkeys(tips):
        differences = checked_git(['diff-tree', '-r', '--no-renames', '--name-only', '-z',
                                   '--ignore-submodules=none', tip, tree_id], work, env)
        # Intersecting with the differences also removes entries absent from both trees.
        own.intersection_update(os.fsdecode(p) for p in differences.split(b'\0') if p)
        if not own:
            break
    return sorted(own)


def journeys(root, paths, base):
    """Check writer/source mixtures, overlays and retirements (tools/pre-pr/design.md, 4, check 8)."""
    paths = sorted(paths)
    retired, walks_deleted = [], []
    for path in paths:
        parent, _, name = path.rpartition('/')
        prefix, _, directory = parent.rpartition('/')
        if name != 'walk' or not directory or prefix not in JOURNEY_ROOTS:
            continue
        journey = root / parent
        entries = list(journey.iterdir()) if not journey.is_symlink() and journey.is_dir() else []
        if not entries:
            retired.append(parent + '/')
            continue
        files = {f'{parent}/{entry.name}' for entry in entries if entry.is_symlink() or not entry.is_dir()}
        if path not in files and any(map(written_journey_file, files)):
            walks_deleted.append(path if path.isprintable() else ascii(path))

    retired = tuple(retired)
    written, others = [], []
    for path in paths:
        names = written if path.startswith(retired) or written_journey_file(path) else others
        names.append(path if path.isprintable() else ascii(path))
    log = []
    if written and others:
        log = ['written files:', *written[:20]]
        if len(written) > 20:
            log.append(f'… and {len(written) - 20} more written files')
        log.append(f'other path: {others[0]}')
    for prefix in JOURNEY_ROOTS:
        overlay = root / prefix / 'overlay.css'
        if overlay.is_symlink() or (overlay.exists() and overlay.stat().st_size):
            log.append(f'{prefix}/overlay.css must be empty')
    for walk in walks_deleted:
        log.append(f"{walk} deleted while its pages stay: delete the journey's directory whole, or keep its walk")
    if not log:
        return []
    log.append("a journey's recording, pages and the files beside them are written on main by "
               f"the journeys' writer alone (AGENTS.md, 4): restore them with git checkout {base} -- PATH..., "
               'and compare pages by cargo run -p temper-client-walk -- print --out DIR in your own tree')
    return [failure('journeys', '\n'.join(log))]


def outward(root, env, paths=None):
    """A policy change or a known outward failure requires the repository-wide scan."""
    args = ['python3', '-I', '-B', 'tools/outward/outward.py']
    if paths is None:
        code, log = run(args, root, env)
    else:
        files = [os.fsencode(p) for p in paths if os.path.lexists(root / p)]
        try:
            result = subprocess.run([*args, '--paths0'], cwd=root, env=env, input=b'\0'.join(files),
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        except OSError as error:
            return 127, [broken('outward', error)]
        log = result.stdout.decode(errors='replace')
        if log:
            print(log, end='', flush=True)
        code = result.returncode
    if code == 1:
        errors = [failure('test fabrig-boundary::outward no_file_names_the_lab', log,
                          ('fabrig-boundary::outward', 'no_file_names_the_lab'))]
    else:
        errors = command_errors('outward', code, log, name='check outward')
    return code, errors


def format_check(args, root, env, label):
    """Stream the formatter diff and keep its first source location in the verdict."""
    location = ''
    def consume(line, log):
        nonlocal location
        line = line.replace(os.fsencode(root) + b'/', b'')
        log.add(line)
        sys.stdout.buffer.write(line)
        if not location and line.startswith(b'Diff in '):
            location = line[len(b'Diff in '):].decode(errors='replace').rstrip('\r\n:')
    code, log = run(args, root, env, output=consume)
    errors = command_errors('format', code, log, detail=f'{label}: {log}')
    if location and code:
        red = failure('format', f'{location}: run cargo fmt --all')
        return [red, *errors] if code < 0 else [red]
    return errors


def format_workspace(root, env):
    return format_check(['cargo', 'fmt', '--all', '--', '--check'], root, env, 'cargo fmt --all -- --check')


# A change to one of these, or to the root manifest, can move any file's formatting.
FORMAT_WIDE = {'rustfmt.toml', '.rustfmt.toml', 'rust-toolchain', 'rust-toolchain.toml'}


def format_changes(root, paths, env):
    """Check the formatting of the packages owning the changed Rust files, as the gate's format does."""
    if 'Cargo.toml' in paths or any(Path(p).name in FORMAT_WIDE for p in paths):
        return format_workspace(root, env)
    rust = [p for p in paths if p.endswith('.rs') or Path(p).name == 'Cargo.toml']
    if not rust:
        return []
    directories = package_directories(members(metadata(root, env)))
    owners = sorted({name for p in rust if (name := owner(root / p, directories)) is not None})
    if not owners:
        return []
    print(f'pre-pr: formatting {len(owners)} changed packages', flush=True)
    return format_check(['cargo', 'fmt', *package_args(owners), '--', '--check'], root, env,
                        'cargo fmt -- --check')


def failure(name, log, key=None, package=None):
    return {'name': name, 'log': tail(log), 'key': key, 'package': package, 'inherited': False}


def broken(what, error, log=None):
    return dict(failure(f'check {what}', str(error) if log is None else log), broke=first_line(error))


def command_break(what, code, log):
    first = first_line(log)
    if code < 0:
        detail = f'{what}: killed by signal {-code}'
    elif code == 127 and first:
        return broken(what, first, log)
    else:
        detail = f'{what}: exit {code}'
    return broken(what, detail + (f': {first}' if first else ''), f'{detail}\n{log}')


def command_errors(what, code, log, *, name=None, detail=None):
    if not code:
        return []
    if code < 0 or code == 127:
        return [command_break(what, code, log)]
    return [failure(name or what, log if detail is None else detail)]


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
    if not errors or code < 0 or code == 127:
        errors += command_errors('clippy', code, log, name='check clippy', detail=f'exit {code}\n{log}')
    return code, errors


def test_binaries(data, root):
    """Validate focused-test prerequisites before the checks (tools/tiers/design.md, 2.1)."""
    path = 'tools/tiers/packages.toml'
    def refusal(detail):
        return CheckFindings([failure('test-binaries inventory', f'{path}: {detail}')])

    with (root / path).open('rb') as source:
        try:
            inventory = tomllib.load(source).get('test-binaries', {})
        except ValueError as error:
            raise refusal(error) from error
    if not isinstance(inventory, dict):
        raise refusal('test-binaries needs a table')
    workspace = members(data)
    for name, binaries in inventory.items():
        if name not in workspace:
            raise refusal(f'test-binaries key {name} is not a workspace package')
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
            raise refusal(f'invalid list for {name} (need sorted, unique package/bin records and distinct binary names)')
    return inventory


def build_test_binaries(binaries, root, env):
    for binary in binaries:
        code, log = run(['cargo', 'build', '-p', binary['package'], '--bin', binary['bin']], root, env)
        if code:
            return code, command_errors('build', code, log, detail=f"{binary['package']}/{binary['bin']}: exit {code}\n{log}")
    return 0, []


def listing_failed_to_build(code, log):
    return code == 2 and log.rstrip('\r\n').rsplit('\n', 1)[-1] == 'one-process: nextest listing exited with 101'


def close_test_errors(directory, what, code, log, diagnostic):
    """Take only one-process's build or test verdicts as candidate failures (section 6)."""
    if listing_failed_to_build(code, log):
        return [dict(failure('build' + (' fuzzy' if what == 'fuzzy tests' else ''), log), diagnostic=diagnostic)]
    if code not in (0, 100):
        return [command_break(what, code, log)]
    errors = []
    try:
        tests = json.loads((directory / 'results.json').read_bytes())['tests']
        if not isinstance(tests, list):
            raise ValueError('tests must be a list')
        for test in tests:
            if test['outcome'] not in ('failed', 'crashed'):
                continue
            binary, case, output = test['binary_id'], test['test_name'], test.get('output') or ''
            if not all(isinstance(value, str) for value in (binary, case, output)) or not binary or not case:
                raise ValueError('failed test must have a binary id and a case name, with text output if present')
            errors.append(dict(failure(f'test {binary} {case}', output or log, (binary, case)),
                               diagnostic=test_diagnostic(case, output)))
    except (OSError, ValueError, KeyError, TypeError) as error:
        errors.append(broken(what, f'results.json: {error}'))
    if code == 100 and not errors:
        errors.append(broken(what, 'one-process: exit 100 without a failed or crashed test'))
    return errors


def one_process(packages, root, env, reds=(), rerun=0, outcomes=None, profile='default'):
    """Read one-process's exit and run directory, keeping starvation a warning (section 4)."""
    check_name = 'tests' if profile == 'default' else 'fuzzy tests'
    suffix = '' if profile == 'default' else ' fuzzy'
    with tempfile.TemporaryDirectory(prefix='temper-pre-pr-') as directory:
        directory = Path(directory) / ('tests' if profile == 'default' else 'fuzzy')
        arguments = ['python3', '-I', '-B', 'tools/tiers/one-process', '--out', str(directory)]
        arguments += [profile] + package_args(packages) + ['--no-tests=warn'] + planner()['workspace_filter'](packages)
        test_env = dict(env, TIERS_PRE_PR_TARGETS='1')
        code, log = run(arguments, root, test_env, stdout=True)
        if listing_failed_to_build(code, log):
            return code, [failure('build' + suffix, 'no tests ran\n' + log)], None
        if code not in (0, 100):
            return code, [command_break('one-process' + suffix, code, log)], None
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
            errors.append(broken('one-process' + suffix, f'{source}: {error}'))
            return code or 1, errors, starved
        if not errors:
            errors.append(broken('one-process' + suffix, 'exit 100 without JUnit failures'))
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
    lines = ['temper-check 1', f'check {tree_id} {base_id or "-"} {verdict}']
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
    path = git_path(root, env, 'temper/check-results')
    replace_file(path, ('\n'.join(lines) + '\n').encode('utf-8'))


def settled(error):
    return error['inherited'] or 'flake' in error


def failure_groups(errors):
    """The record's and summary's class order; inherited takes precedence over a flake."""
    groups = {kind: [] for kind in ('new', 'flake', 'inherited', 'broke')}
    for error in errors:
        kind = ('broke' if 'broke' in error else 'inherited' if error['inherited'] else
                'flake' if 'flake' in error else 'new')
        groups[kind].append(error)
    groups['flake'].sort(key=lambda e: e['key'])
    return groups


def outcome(groups):
    if groups['new']:
        return 'refused'
    return 'broke' if groups['broke'] else 'passed'


def counts_by_class(groups):
    return {kind: len(group) for kind, group in groups.items()}


def replace_file(path, data):
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as out:
            temporary = Path(out.name)
            out.write(data)
        os.replace(temporary, path)
    except BaseException:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
        raise


def failure_file(tree_id, base_id, verdict, cached, groups, root, scope, commits, head):
    """Keep every structured failure atomically, before the summary (sections 6–7)."""
    relative = {'child': 'target/temper/goal-child-pre-pr.json',
                'close': 'target/temper/goal-close-pre-pr.json',
                'delivery': 'target/temper/pre-pr-light-failures.json'}.get(scope, 'target/temper/pre-pr-failures.json')
    path = root / relative
    try:
        entries = []
        for kind, group in groups.items():
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
                if kind == 'broke':
                    entry['detail'] = error['broke']
                elif error['name'] in ('format', 'budgets'):
                    entry['detail'] = error['log'].strip()
                if error.get('figures'):
                    entry['figures'] = error['figures']
                entries.append(entry)
        header = {'form': 'pre-pr-failures 1', 'tree': tree_id, 'base': base_id,
                  'cached': cached, 'verdict': verdict,
                  'counts': counts_by_class(groups)}
        header['comparison'] = 'tree-only' if base_id is None else 'changes'
        if scope == 'delivery':
            header.update(form='pre-pr-light-failures 1', scope='light')
        elif scope == 'child':
            header.update(form='goal-child-pre-pr 1', scope=scope,
                          delivery_head=head, destination=None, requested_destination=commits[0] if commits else None)
        elif scope in ('goal', 'close'):
            header.update(scope=scope, goal_head=head, destination=None, requested_destination=commits[-1] if commits else None)
            if scope == 'close':
                header['form'] = 'goal-close-pre-pr 1'
        text = json.dumps(header, ensure_ascii=False)[:-1] + ', "failures": [\n'
        text += ',\n'.join(json.dumps(entry, ensure_ascii=False) for entry in entries) + '\n]}\n'
        path.parent.mkdir(parents=True, exist_ok=True)
        replace_file(path, text.encode('utf-8', errors='backslashreplace'))
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


def summary(base_id, lints, tests, groups, verdict, failures_tail, starved, scope, base_name):
    """Keep the first failure whole and count what the host's tail cannot hold."""
    # The whole base id: a red's base is read from this line (forge/reds.md).
    compared = 'current tree' if base_id is None else f'base {base_id}, {base_name}'
    header = f'pre-pr: {verdict} ({scope}, {compared})\n'
    counts = counts_by_class(groups)
    header += (f'failures: {counts["new"]} new, {counts["flake"]} flakes, '
               f'{counts["inherited"]} inherited, {counts["broke"]} broke; {failures_tail}\n')
    sets = f'lints: {" ".join(lints)}\ntests: {" ".join(tests)}\n'
    sets += ''.join(line + '\n' for line in starved)
    new, flakes, inherited_errors, broke = groups.values()
    entries = [e['name'] + '\n' + e['log'] for e in new]
    entries += [f'flake: {e["name"]} (failed under load; passed when its process ran again: '
                f'{e["flake"][0]} tests, {e["flake"][1]:.3f} s)\n' + e['log']
                for e in flakes]
    entries += ['inherited: ' + e['name'] +
                (' (red, temper/reds)' if e['name'].startswith(('test ', 'clippy ')) else '') + '\n'
                for e in inherited_errors]
    entries += ['broke: ' + e['name'] + '\n' + e['log'] for e in broke]
    # Keep the verdict and file pointer whole, cutting only the sets and warnings.
    room = max(0, 4096 - len(header.encode()) - 1)
    header += sets.encode()[:room].decode('utf-8', errors='ignore') + '\n'
    diagnostic = next((e['diagnostic'] for e in new if e.get('diagnostic')), '') if scope == 'close' else ''
    output = header
    for index, entry in enumerate(entries):
        entry = entry.encode('utf-8', errors='backslashreplace').decode('utf-8')
        remaining = len(entries) - index
        marker = f'[… {remaining} more failures]\n'
        if len((output + entry).encode()) + len((marker + diagnostic).encode()) > 8192:
            output += marker
            break
        output += entry
    print(output + diagnostic, end='', flush=True)
    if verdict == 'broke':
        print_broke(broke[0]['broke'])
    elif verdict == 'refused' and scope == 'close':
        # The close job's hold keeps only the last 128 bytes of the check's stderr: name the first failure there.
        first = new[0]
        # A failed case, a compiler error or a panic says most; else the log's first line, not its cut marker.
        texts = [text.strip() for text in first['log'].splitlines() if text.strip() and not text.startswith('[… ')]
        telling = [text for text in texts if text.startswith(('FAIL', 'error', 'thread ')) or 'panicked' in text]
        line = diagnostic.strip() or first['name'] + ': ' + (telling or texts or [''])[0]
        print(stderr_line(line), end='', file=sys.stderr, flush=True)


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
    if any(path in paths for path in ('.temper/hooks/merge', '.temper/merge')):
        errors += timed('merge', lambda: merge_tests(root, env))[1]
    return lints, tests, warnings


def typed_close(scope, env):
    return scope == 'close' and env.get('TEMPER_CLOSE_REFUSAL') == '3'


def refusal_bytes(head, tree, base, counts, failures):
    """Bound the close result in whole names (temper/docs/design/protocol/tooling.md, 4.2)."""
    refused = dict(head=head, tree=tree, base=base, counts=counts, failures=failures)

    def encode():
        return (json.dumps({'refused': refused}, ensure_ascii=True, separators=(',', ':')) + '\n').encode('ascii')

    full = encode()
    if len(full) <= 512:
        return full
    refused['failures'] = []
    for index, name in enumerate(failures):
        refused['failures'].append(name)
        remaining = len(failures) - index - 1
        if remaining:
            refused['more'] = remaining
        else:
            refused.pop('more', None)
        if len(encode()) > 512:
            refused['failures'].pop()
            refused['more'] = remaining + 1
            break
    return encode()


def finish(tree_id, base_id, errors, root, env, scope, commits, heads, lints=(), tests=(), warnings=(), base_name=None,
           *, counts_ref=None):
    groups = failure_groups(errors)
    verdict = outcome(groups)
    head = heads[0] if heads else None
    location = failure_file(tree_id, base_id, verdict, False, groups, root, scope, commits, head)
    summary(base_id, lints, tests, groups, verdict, location, warnings, scope, base_name)
    if verdict == 'passed':
        return 0
    if not typed_close(scope, env):
        return 1
    if verdict == 'broke':
        return 2
    try:
        job = env.get('TEMPER_JOB')
        if not job:
            raise ValueError('TEMPER_JOB is missing')
        if not head or not tree_id:
            raise ValueError('the close did not identify its head and tree')
        names = [error['name'] for error in groups['new']]
        replace_file(Path(job) / 'result.json', refusal_bytes(head, tree_id, base_id, counts_ref, names))
    except (OSError, ValueError) as error:
        print_broke(f'cannot write close refusal: {error}')
        return 2
    return 3


def workspace_checks(root, paths, tree_id, env, errors, outcomes, reds=(), *, completed_goal=False, counts_source=None,
                     whole=True, base_id=None, published=UNSELECTED_COUNTS):
    start = time.monotonic()
    if whole and any(p.endswith('.rs') for p in paths):
        errors += format_workspace(root, env)
    elif not whole:
        errors += format_changes(root, paths, env)
    if errors:
        return []
    plan = planner() if completed_goal else None
    if completed_goal and whole:
        package_paths = [p for p in paths if p.endswith('.rs') or plan['owns_package'](root, p)]
    elif completed_goal:
        # Prose reaches tests only where the gate declares a reader of it.
        globs = test_input_globs(root)
        package_paths = [p for p in paths if not documentation(p)
                         or any(fnmatch.fnmatchcase(p, pattern) for pattern in globs)]
    else:
        package_paths = []
    data = metadata(root, env) if package_paths else {'packages': [], 'workspace_members': []}
    packages = members(data)

    suites, groups, prerequisites = [], {}, {}
    if completed_goal and packages:
        with (root / 'tools/tiers/layers.toml').open('rb') as file:
            layers = tomllib.load(file)
        inputs, also = code_inputs(package_paths, data, root), ()
        if not whole and inputs:
            wide = [p for p in inputs if workspace_wide(p)]
            if base_id and counts_source:
                # Adding a crate edits the lock and the members; the crates that use it changed too.
                narrow = {'Cargo.lock': lock_moves_externals, 'Cargo.toml': root_manifest_moves_more}
                wide = [p for p in wide if p not in narrow or narrow[p](*counts_source, base_id, root)]
            if wide:
                print(f"pre-pr: every package: {', '.join(wide)} changed", flush=True)
                inputs = [str(Path(package['manifest_path']).relative_to(root)) for package in packages.values()]
            also = always_packages(root, inputs, packages)
        groups = plan['package_plan'](root, inputs, packages, layers, also=also)
        suites = sorted(name for targets, names in groups.items() if targets for name in names)
        prerequisites = test_binaries(data, root)
    print(f'pre-pr: selection: {len(paths)} paths, {len(suites)} test packages, '
          f'{time.monotonic() - start:.3f} s', flush=True)

    def command(label, args, *, test_run=None, build=False):
        print(f"pre-pr: {label}: {shlex.join(args)}", flush=True)
        start = time.monotonic()
        diagnostic = RustcDiagnostic() if build or test_run is not None else None
        rc, log = run(args, root, env, output=diagnostic.consume if diagnostic else None, stdout=True)
        reason = diagnostic.reason() if diagnostic else ''
        if test_run is not None:
            found = close_test_errors(test_run, label, rc, log, reason)
        else:
            found = command_errors(label, rc, log)
            if reason and rc:
                red = dict(failure(label, log), diagnostic=reason)
                found = [red, *found] if 'broke' in found[0] else [red]
        errors.extend(found)
        status = f'exit {rc}' if rc or not found else 'broke'
        print(f'pre-pr: {label}: {status}, {time.monotonic() - start:.3f} s', flush=True)
        return bool(found)

    for name in ('skein', 'smith', 'jig', 'temper', 'hearth'):
        if any((p.startswith(name + '/') and not documentation(p)) or p.startswith('tools/old_forms/') for p in paths):
            errors += timed('old-forms ' + name, lambda: baselines(name, root, env))[1]
    python_suites = set()
    for path in paths:
        file = Path(path)
        if path in HOOK_INPUTS or (path.startswith('tools/pre-pr/') and file.suffix == '.py'):
            python_suites.add('tools/pre-pr/test_pre_pr.py')
        elif completed_goal and file.suffix == '.py':
            suite = file if file.name.startswith('test_') else file.with_name('test_' + file.name)
            if (root / suite).is_file():
                python_suites.add(str(suite))
    for suite in sorted(python_suites):
        command('Python tests', ['python3', '-I', '-B', suite])
    # Code cites design too, and a deleted or renamed file may be cited (tools/citations/README.md).
    if paths:
        errors += timed('citations', lambda: citations(tree_id, root, env))[1]
    if paths:
        key = ('fabrig-boundary::outward', 'no_file_names_the_lab')
        full_scan = ('test', *key) in reds or any(p.startswith('tools/outward/') for p in paths)
        rc, found = timed('outward', lambda: outward(root, env, None if full_scan else paths))
        errors += found
        if full_scan and rc in (0, 1):
            outcomes[key] = {'outcome': 'passed' if rc == 0 else 'failed'}
    if any(p in paths for p in ('tools/tiers/cpu.py', 'tools/tiers/budgets.toml', 'tools/tiers/layers.toml')):
        errors += timed('budgets', lambda: budget_table(root))
    if groups:
        scratch = nullcontext()
        if suites:
            target = Path(data['target_directory'])
            target.mkdir(parents=True, exist_ok=True)
            scratch = tempfile.TemporaryDirectory(prefix='pre-pr-close-', dir=target)
        with scratch as directory:
            runs = defaultdict(list)
            runner = str(TIERS / 'one-process')
            for args in plan['rust_commands'](groups, prerequisites, Path(directory) if suites else None, runner):
                if args[0] == runner:
                    out = Path(args[2])
                    profile = args[args.index('--fail-fast') + 1]
                    runs[profile].append(out)
                    label = 'focused tests' if profile == 'default' else f'{profile} tests'
                    if command(label, args, test_run=out):
                        break
                elif command('build check', args, build=True):
                    break
            else:
                if runs:
                    repository, git_env = counts_source if counts_source is not None else (root, env)
                    errors += close_budgets(root, repository, git_env, runs, published)
    return suites


HOOKS = {'.temper/hooks/deliver', '.temper/hooks/finish', '.temper/hooks/close',
         '.temper/pre-pr', '.temper/close', '.temper/checked-close',
         '.temper/goal-child-pre-pr', '.temper/goal-pre-pr'}
HOOK_CONFIG = '.temper/hooks/config.toml'
HOOK_INPUTS = HOOKS | {HOOK_CONFIG}


def with_rust_tools(env):
    """Find the worker's Rust tools where the caller's PATH lacks them, as a close job's does."""
    if shutil.which('cargo', path=env.get('PATH', os.defpath)):
        return
    home = Path(pwd.getpwuid(os.getuid()).pw_dir)
    tools = home / '.cargo' / 'bin'
    if (tools / 'cargo').exists():
        env['PATH'] = str(tools) + os.pathsep + env.get('PATH', os.defpath)
        # rustup and Cargo's configuration, registry and build cache live in that home too.
        env.setdefault('CARGO_HOME', str(home / '.cargo'))
        env.setdefault('RUSTUP_HOME', str(home / '.rustup'))


def check(scope, commits):
    root = Path.cwd()
    source_env = dict(os.environ, GIT_ALLOW_PROTOCOL='', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env = {k: v for k, v in source_env.items() if not k.startswith('GIT_')}
    env.update(CARGO_NET_OFFLINE='true', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')
    env.setdefault('CARGO_TARGET_DIR', str(root / 'target'))
    env.pop('TIERS_LEDGER', None)
    errors, outcomes, warnings, lints, tests = [], {}, [], [], []
    tree_id = base_id = base_name = receipt_head = counts_ref = None
    has_git = False

    def current_head():
        if not has_git:
            return None
        result = git(['rev-parse', '--verify', '--quiet', 'HEAD'], root, source_env)
        return result.stdout.decode().strip() if result.returncode == 0 else None

    try:
        if any(not re.fullmatch(r'(?:[0-9a-f]{40}|[0-9a-f]{64})', commit) for commit in commits):
            raise ValueError('expected full hexadecimal commit IDs')
        with_rust_tools(env)
        has_git = (root / '.git').exists()
        head = receipt_head = current_head()
        if scope == 'goal' and commits and head and head != commits[0]:
            raise ValueError('checkout HEAD differs from the requested goal')
        with tempfile.TemporaryDirectory(prefix='pre-pr-tree-') as directory:
            work = Path(directory) / 'current'
            fmt = checked_git(['rev-parse', '--show-object-format'], root, source_env).decode().strip() if has_git else 'sha1'
            checked_git(['init', '-q', '--object-format=' + fmt, str(work)], root, env)
            tree_id = snapshot(root, work, env, source_env, work / '.git/index')
            checked_git(['checkout-index', '-a'], work, env)
            paths = [os.fsdecode(p) for p in checked_git(['ls-files', '-z'], work, env).split(b'\0') if p]
            published = None
            if scope == 'close' and has_git:
                published, counts_ref = close_counts(root, source_env)
            if scope in BASES or scope == 'close':
                base_id, base_name, no_base = scope_base(root, source_env, scope, has_git, published)
                if no_base:
                    errors.append(broken('base', no_base))
                else:
                    paths = changed_paths(root, source_env, base_id)
                    print(f'pre-pr: base {base_id}, {base_name}: {len(paths)} changed paths', flush=True)
                    if scope == 'close' and published is None:
                        print("pre-pr: WARNING: budgets not judged at close: main's counts (refs/temper/counts) "
                              'are not in this checkout; the landing gate judges them', flush=True)

            if not errors:
                if scope == 'goal' and head:
                    # Only the current commit can bind this snapshot to a goal head.
                    current = git(['rev-parse', '--verify', '--quiet', 'HEAD^{tree}'], root, source_env)
                    if current.returncode or current.stdout.decode().strip() != tree_id:
                        receipt_head = None
                        if commits and current.returncode == 0:
                            raise ValueError('working tree differs from the requested goal head')
                # Close builds in the goal's checkout, warm with the worker's build cache.
                close = scope == 'close' and has_git
                checks_root = root if close else work
                if close and checked_git(['write-tree', '--missing-ok'], root, source_env).decode().strip() != tree_id:
                    raise ValueError('working tree differs from the staged close tree')
                try:
                    tests = workspace_checks(checks_root, paths, tree_id, env, errors, outcomes,
                                             completed_goal=scope == 'close', counts_source=(root, source_env),
                                             whole=base_id is None, base_id=base_id, published=published)
                    if scope == 'goal' and not errors:
                        lints, tests, warnings = goal_checks(work, paths, env, errors,
                                                            rerun_bound(root, source_env) if has_git else 0)
                except CheckFindings as found:
                    errors.extend(found.errors)
                except Exception as error:
                    errors.append(broken('workspace checks', error))

                def check_journeys():
                    own = journey_paths(root, source_env, work, env, tree_id, paths, base_id, head)
                    return journeys(checks_root, own, base_id or 'HEAD')

                errors += timed('journeys', check_journeys)
                if close and checked_git(['write-tree', '--missing-ok'], root, source_env).decode().strip() != tree_id:
                    raise ValueError('staged close tree changed during checks')
                if tree_id != snapshot(root, work, env, source_env, Path(directory) / 'final-index') or head != current_head():
                    raise ValueError('the checkout changed during checks')
    except Exception as error:
        errors.append(broken('workspace checks', error))

    def report():
        if tree_id is None:
            # No snapshot to name in a receipt; leave the last completed check's file alone.
            print_broke(errors[0]['broke'])
            return 2 if typed_close(scope, env) else 1
        return finish(tree_id, base_id, errors, root, env, scope, commits, [receipt_head], lints, tests, warnings,
                      base_name, counts_ref=counts_ref)

    if has_git and scope == 'delivery':
        results = None
        try:
            if git_path(root, source_env, 'temper/reds-wait').is_file():
                results = git_path(root, source_env, 'temper/check-results')
                verdict = outcome(failure_groups(errors))
                if verdict == 'broke':
                    results.unlink(missing_ok=True)
                else:
                    reds, _, _ = goal_reds(root, source_env)
                    status = report()
                    check_results(tree_id, base_id, verdict, reds, lints, outcomes, errors, root, source_env)
                    return status
        except Exception as error:
            errors.append(broken('check-results', error))
            if results is not None:
                try:
                    results.unlink(missing_ok=True)
                except OSError as error:
                    errors.append(broken('check-results cleanup', error))
    return report()


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
    except Exception as error:
        print_broke(error)
        scope = 'close' if sys.argv[1:2] == ['--close'] else None
        sys.exit(2 if typed_close(scope, os.environ) else 1)
