"""The delivery check (tools/pre-pr/design.md, 1–9).

The private snapshot leaves the caller's index alone; only exceeded counts need a
base worktree. Every tool streams before the bounded, final verdict.
"""
import contextlib
import fnmatch
import hashlib
import json
import os
from pathlib import Path
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


def git(arguments, cwd, env):
    return subprocess.run(['git'] + arguments, cwd=cwd, env=env,
                          stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=False)


def base(root, env, destination=None, frozen_heads=None):
    """Read the landing pin and merge heads, never moving the pin (section 2)."""
    pin = git(['rev-parse', '--verify', '--quiet', 'refs/temper/landing^{commit}'], root, env)
    if pin.returncode:
        return None, 'no refs/temper/landing'
    if destination is not None and pin.stdout.strip() != destination.encode():
        return None, 'destination differs from the worker landing pin'
    if frozen_heads is None:
        heads = ['refs/temper/landing', 'HEAD']
        if not git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD'], root, env).returncode:
            heads.append('MERGE_HEAD')
    else:
        heads = [destination, *frozen_heads]
    result = git((['merge-base'] if destination is None else ['merge-base', '--all']) + heads, root, env)
    if result.returncode or not result.stdout.strip():
        return None, 'no merge base of refs/temper/landing and HEAD'
    if destination is not None and len(result.stdout.splitlines()) != 1:
        return None, 'child delivery requires one merge base'
    commit = result.stdout.decode().splitlines()[0]
    while git(['log', '-1', '--format=%s', commit], root, env).stdout.decode().rstrip('\n') == (
            'Unchecked: work in progress, saved before a question'):
        parent = git(['rev-parse', '--verify', '--quiet', commit + '^1'], root, env)
        if parent.returncode:
            if destination is not None:
                return None, 'cannot find the parent of the unchecked save'
            break
        commit = parent.stdout.decode().strip()
    return commit, None


FULL_COMMIT = re.compile(r'(?:[0-9a-f]{40}|[0-9a-f]{64})')


def verify_commit(name, commit, root, env):
    if not FULL_COMMIT.fullmatch(commit):
        raise ValueError(f'{name} must be a full commit ID')
    actual = git(['rev-parse', '--verify', commit + '^{commit}'], root, env)
    if actual.returncode or actual.stdout.strip() != commit.encode():
        raise ValueError(f'{name} commit unavailable: {commit}')


def goal_base(goal, root, git_dir, env):
    """Verify the exact completed checkout and find the goal's unambiguous branch point."""
    if len(goal) != 2:
        raise ValueError('expected the full goal head and destination commit IDs')
    head, destination = goal
    for name, commit in zip(('head', 'destination'), goal):
        verify_commit(name, commit, root, env)
    current = git(['rev-parse', '--verify', 'HEAD'], root, env)
    if current.returncode or current.stdout.strip() != head.encode():
        raise ValueError(f'checkout HEAD differs from goal head {head}')
    if (git_dir / 'MERGE_HEAD').exists():
        raise ValueError('checkout has a pending merge')
    status = git(['status', '--porcelain=v1', '--untracked-files=all', '--ignore-submodules=none'], root, env)
    if status.returncode or status.stdout:
        raise ValueError('goal checkout must have a clean index and worktree')
    bases = git(['merge-base', '--all', destination, head], root, env)
    if bases.returncode or len(bases.stdout.splitlines()) != 1:
        raise ValueError('goal head and destination require one common merge base')
    return bases.stdout.decode().strip()


def policy_context(scope, commits, root, git_dir, env):
    """Bind the comparison base and the child parents used by per-change rules."""
    if scope == 'goal':
        return goal_base(commits, root, git_dir, env), None
    if scope != 'child' or len(commits) != 1 or not FULL_COMMIT.fullmatch(commits[0]):
        raise ValueError('expected a full child destination commit ID')
    current = git(['rev-parse', '--verify', 'HEAD^{commit}'], root, env)
    if current.returncode:
        raise ValueError('child checkout requires HEAD')
    heads = [current.stdout.decode().strip()]
    try:
        merging = (git_dir / 'MERGE_HEAD').read_text(encoding='ascii').splitlines()
    except FileNotFoundError:
        pass
    else:
        if not merging:
            raise ValueError('empty child MERGE_HEAD')
        for commit in merging:
            verify_commit('pending merge head', commit, root, env)
        heads.extend(merging)
    base_id, why = base(root, env, commits[0], heads)
    if not base_id:
        raise ValueError(why)
    return base_id, heads


def tree(root, git_dir, index, env):
    """Write all nonignored sources through a private index (section 3)."""
    private = env.copy()
    private['GIT_INDEX_FILE'] = str(index)
    source = Path(env.get('GIT_INDEX_FILE', git_dir / 'index'))
    if source.exists():
        shutil.copy2(source, index)  # mtime kept: git still sees a same-size edit in the same second
    for arguments in (['add', '-A'], ['write-tree']):
        result = git(arguments, root, private)
        if result.returncode:
            raise RuntimeError(result.stderr.decode(errors='replace'))
    return result.stdout.decode().strip(), private


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


def select(data, changed, always):
    """C, direct D and prefixed R, sorted by package name (section 3)."""
    packages = members(data)
    directories = package_directories(packages)
    changed_packages = {name for path in changed if (name := owner(path, directories)) is not None}
    tests = changed_packages.copy()
    for name, package in packages.items():
        if any(d.get('path') and d['name'] in changed_packages for d in package['dependencies']):
            tests.add(name)
    for line in always.splitlines():
        fields = line.split('#', 1)[0].split()
        if len(fields) != 2:
            continue
        name, prefix = fields
        if name not in packages:
            print(f'pre-pr: path reader {name}: not a workspace member, skipped', flush=True)
        elif any(path.startswith(prefix) for path in changed.values()):
            tests.add(name)
    return sorted(changed_packages), sorted(tests)


def boundary(data, always) -> list[str]:
    """A, the boundary packages for every selected delivery (section 3)."""
    packages = members(data)
    names = {'fabrig-boundary'}
    for line in always.splitlines():
        fields = line.split('#', 1)[0].split()
        if len(fields) == 1:
            names.add(fields[0])
    for name in sorted(names - packages.keys()):
        print(f'pre-pr: boundary package {name}: not a workspace member, skipped', flush=True)
    return sorted(names & packages.keys())


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


def merged_tips(base_id, root, env, frozen_heads=None):
    """Check 7's pending merge first, then first-parent history's other parents."""
    if frozen_heads is None:
        head = 'HEAD'
        pending = git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD'], root, env)
        merged = [] if pending.returncode else [pending.stdout.decode().strip()]
    else:
        head, *merged = frozen_heads
    history = git(['rev-list', '--first-parent', '--merges', '--parents', base_id + '..' + head], root, env)
    if history.returncode:
        raise RuntimeError(history.stderr.decode(errors='replace'))
    for line in history.stdout.decode().splitlines():
        merged.extend(line.split()[2:])
    return list(dict.fromkeys(merged))


def figures(base_id, tree_id, root, env, merged):
    """Run check 7 (tools/pre-pr/design.md, 4; tools/tiers/budgets.md, 10.1)."""
    arguments = ['python3', '-I', '-B', 'tools/tiers/figures.py',
                 '--base', base_id, '--head', tree_id]
    for revision in merged:
        arguments += ['--merged', revision]
    code, log = run(arguments, root, env, stdout=True)
    if code == 1:
        errors = [failure('figures', log)]
    else:
        errors = [failure('check figures', log)] if code else []
    return code, errors


def merge_tests(root, env):
    """Run check 9 for deliveries changing the merge job (tools/pre-pr/design.md, 4)."""
    code, log = run(['python3', '-I', '-B', 'tools/merge/test_merge.py'], root, env, stdout=True)
    return code, [failure('merge', log)] if code else []


JOURNEY_ROOTS = tuple(f'temper/docs/design/client/{kind}/journeys' for kind in ('web', 'tui'))
JOURNEY_WRITTEN = re.compile(
    r'(?:[^/]+/(?:recording|[0-9]{2,}\.(?:html|txt))|skein-ui\.css|journey\.js|theme\.css|tui-journey\.js)')


def delivery_paths(base_id, tree_id, root, env, merged):
    """Paths changed here, excluding entries brought by a merge (sections 3 and 4)."""
    def against(revision):
        diff = git(['diff-tree', '-r', '--no-renames', '--name-only', '-z', revision, tree_id], root, env)
        if diff.returncode:
            raise RuntimeError(diff.stderr.decode(errors='replace'))
        return {os.fsdecode(path) for path in diff.stdout.split(b'\0') if path}

    paths = against(base_id or 'HEAD')
    if base_id and paths:
        for tip in merged:
            # A diff includes absent entries and mode changes as well as blobs.
            paths.intersection_update(against(tip))
    return sorted(paths)


def journeys(base_id, tree_id, paths, root, env):
    """Keep written files separate and overlays empty (section 4, check 8)."""
    written, other = [], []
    for path in paths:
        relative = next((path[len(prefix) + 1:] for prefix in JOURNEY_ROOTS
                         if path.startswith(prefix + '/')), '')
        (written if JOURNEY_WRITTEN.fullmatch(relative) else other).append(path)
    overlays = [prefix + '/overlay.css' for prefix in JOURNEY_ROOTS]
    entries = git(['ls-tree', '-l', '-z', tree_id, '--'] + overlays, root, env)
    if entries.returncode:
        raise RuntimeError(entries.stderr.decode(errors='replace'))
    nonempty = []
    for entry in entries.stdout.split(b'\0'):
        if entry:
            header, path = entry.split(b'\t', 1)
            _, kind, _, size = header.split()
            if kind == b'blob' and int(size) > 0:
                nonempty.append(os.fsdecode(path))
    if not (written and other) and not nonempty:
        return 0, []
    lines = ['written journey file: ' + path for path in written[:20]]
    if len(written) > 20:
        lines.append(f'… {len(written) - 20} more written journey files')
    if other:
        lines.append('other path: ' + other[0])
    lines += ['nonempty overlay: ' + path for path in nonempty]
    lines.append("a journey's recording, pages and the files beside them are written on main by "
                 "the journeys' writer alone (AGENTS.md, 4): restore them with git "
                 f"checkout {base_id or 'HEAD'} -- PATH..., and compare pages by cargo run -p "
                 "temper-client-walk -- print --out DIR in your own tree")
    log = '\n'.join(lines) + '\n'
    print(log, end='', flush=True)
    return 1, [failure('journeys', tail(log))]


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


def _configuration(source, root):
    """Find the target's rustfmt configuration without leaving the repository."""
    source.relative_to(root)
    directory = source.parent
    while True:
        for name in ('.rustfmt.toml', 'rustfmt.toml'):
            candidate = directory / name
            if candidate.is_file():
                return candidate
        if directory == root:
            return None
        directory = directory.parent


def _groups(data, root):
    """Deduplicate target roots sharing an edition and rustfmt configuration."""
    groups = {}
    for package in data['packages']:
        for target in package['targets']:
            source = Path(target['src_path'])
            edition = target.get('edition', package['edition'])
            key = (edition, _configuration(source, root))
            groups.setdefault(key, set()).add(str(source))
    if not groups:
        raise ValueError('cargo metadata has no formatting targets')
    return groups


def formatting(data, root, env):
    failures = []
    for (edition, configuration), sources in _groups(data, root).items():
        arguments = ['rustfmt', '--check', '--edition', edition]
        if configuration is not None:
            arguments.extend(['--config-path', str(configuration)])
        batch = []
        size = sum(len(argument.encode()) + 1 for argument in arguments)
        for source in sorted(sources):
            length = len(source.encode()) + 1
            if batch and size + length > 65536:
                code, log = run(arguments + batch, root, env)
                if code:
                    failures.append(failure('format', log))
                batch = []
                size = sum(len(argument.encode()) + 1 for argument in arguments)
            batch.append(source)
            size += length
        if batch:
            code, log = run(arguments + batch, root, env)
            if code:
                failures.append(failure('format', log))
    return failures


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


def main_ledgers(setting, directory, root):
    """Copy and validate the builder's ledgers before keying the cache (sections 4, 7)."""
    if not setting:
        print('pre-pr: budgets: warnings only (TIERS_LEDGER unset)', flush=True)
        return {}, '-'
    copies, digests = {}, []
    try:
        reader = runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/cpu.py'))['ledger']
        load_error = None
    except Exception as error:
        reader, load_error = None, error
    for profile in ('default', 'fuzzy'):
        name = profile + '.json'
        copy = directory / (profile + '-ledger.json')
        digest = '-'
        why = None
        reading = True
        try:
            if load_error is not None:
                raise ValueError(str(load_error))
            content = (root / setting / name).read_bytes()
            reading = False
            copy.write_bytes(content)
            data = reader(copy)
            if data['profile'] != profile:
                why = f'{name} holds the {data["profile"]} ledger'
            else:
                copies[profile] = copy
                digest = hashlib.sha256(content).hexdigest()
        except (OSError, ValueError, KeyError, TypeError) as error:
            if reading and isinstance(error, FileNotFoundError):
                why = f'no {name} in TIERS_LEDGER'
            else:
                kind = 'unreadable' if isinstance(error, OSError) else 'refused'
                why = f'{name} {kind}: {error_line(error)}'
        if why is None:
            print(f"pre-pr: budgets: {profile} warnings only with main's ledger (TIERS_LEDGER/{name})", flush=True)
        else:
            print(f'pre-pr: budgets: {profile} warnings only ({why})', flush=True)
        digests.append(f'{profile} {digest}\n')
    digest = hashlib.sha256(''.join(digests).encode()).hexdigest() if copies else '-'
    return copies, digest


def one_process(packages, root, env, reds=(), rerun=0, outcomes=None, profile='default', ledger=None):
    """Read one-process's exit and run directory, keeping starvation a warning (section 4)."""
    check_name = 'tests' if profile == 'default' else 'fuzzy tests'
    suffix = '' if profile == 'default' else ' fuzzy'
    with tempfile.TemporaryDirectory(prefix='temper-pre-pr-') as directory:
        directory = Path(directory) / ('tests' if profile == 'default' else 'fuzzy')
        arguments = ['python3', '-I', '-B', 'tools/tiers/one-process', '--out', str(directory)]
        if ledger is not None:
            arguments += ['--ledger', str(ledger)]
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
    """Wait for and read the goal's whole reds, before the cache (sections 5, 7)."""
    paths = [root / os.fsdecode(git(['rev-parse', '--git-path', 'temper/' + name], root, env).stdout).strip()
             for name in ('reds', 'reds-wait')]
    path, marker = paths
    milliseconds = None
    try:
        match = re.fullmatch(rb'temper-reds-wait 1 ([0-9]+)\n?', marker.read_bytes())
        if match:
            digits = match[1].lstrip(b'0')
            if len(digits) <= 6:
                value = int(digits or b'0')
                if 1 <= value <= 600000:
                    milliseconds = value
        if milliseconds is None:
            print('pre-pr: reds: ignored temper/reds-wait (malformed)', flush=True)
    except FileNotFoundError:
        pass
    if milliseconds is not None:
        deadline = time.monotonic() + milliseconds / 1000
        while not path.exists():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            time.sleep(min(0.05, remaining))
    try:
        with path.open('rb') as source:
            size = os.fstat(source.fileno()).st_size
            if size > 2097152:
                print(f'pre-pr: reds: none (temper/reds oversized: {size} bytes)', flush=True)
                return set(), None, milliseconds is not None
            content = source.read()
    except FileNotFoundError:
        if milliseconds is not None:
            print(f'pre-pr: reds: none (temper/reds not written within {milliseconds} ms)', flush=True)
        return set(), None, milliseconds is not None
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
        return set(), digest, milliseconds is not None
    print(f'pre-pr: reds: {len(entries)} entries (temper/reds)', flush=True)
    return entries, digest, milliseconds is not None


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


def judge_base(base_id, git_dir, count_errors, root, env):
    """Compare exceeded counts alone in the kept detached worktree (section 5)."""
    base_env = env.copy()
    base_env.pop('GIT_INDEX_FILE', None)
    target = (git_dir / 'temper-pre-pr/target').resolve()
    target.mkdir(parents=True, exist_ok=True)
    base_env['CARGO_TARGET_DIR'] = str(target)
    worktree = git_dir / 'temper-pre-pr/base'
    if (worktree / '.git').exists():
        commands = [(['checkout', '--detach', '--force', base_id], worktree),
                    (['clean', '-ffdxq'], worktree)]
    else:
        commands = [(['worktree', 'add', '--detach', '--force', str(worktree), base_id], root)]
    for arguments, cwd in commands:
        code, log = run(['git'] + arguments, cwd, base_env)
        if code:
            return 'base could not judge: ' + log
    reasons = []
    for name in sorted({e['key'][0] for e in count_errors}):
        code, base_errors = timed('base old-forms ' + name,
                                  lambda: baselines(name, worktree, base_env))
        if code not in (0, 1) or any(e['key'] is None for e in base_errors):
            reasons.append('base could not judge old-forms ' + name + ': ' +
                           ''.join(e['log'] for e in base_errors))
            continue
        counts = {e['key'][1]: e['key'][2:] for e in base_errors}
        for error in count_errors:
            tree_name, form, count, limit = error['key']
            if tree_name == name and form in counts:
                base_count, base_limit = counts[form]
                error['inherited'] = count <= base_count and count - limit <= base_count - base_limit
    if reasons:
        for error in count_errors:
            error['inherited'] = False
    return '\n'.join(reasons)


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
    temporary = path.with_name('check-results.tmp')
    temporary.write_text('\n'.join(lines) + '\n')
    temporary.replace(path)


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
                entries.append({'class': kind, 'name': error['name'], 'key': structured})
        header = {'form': 'pre-pr-failures 1', 'tree': tree_id, 'base': base_id,
                  'cached': cached, 'verdict': 'refused' if verdict else 'passed',
                  'counts': failure_counts(errors)}
        if scope == 'delivery':
            header.update(form='pre-pr-light-failures 1', scope='light')
        elif scope == 'close':
            header.update(form='goal-close-pre-pr 1', scope='close', goal_head=commits[0], destination=commits[1])
        elif scope == 'goal':
            header.update(goal_head=commits[0], destination=commits[1])
        elif scope == 'child':
            header.update(form='goal-child-pre-pr 1', scope='child', destination=commits[0],
                          delivery_head=heads[0], merge_heads=heads[1:])
        text = json.dumps(header, ensure_ascii=False)[:-1] + ', "failures": [\n'
        text += ',\n'.join(json.dumps(entry, ensure_ascii=False) for entry in entries) + '\n]}\n'
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_name(path.name + '.tmp')
        temporary.write_bytes(text.encode('utf-8', errors='backslashreplace'))
        temporary.replace(path)
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
    base_text = f'base {base_id}, merge base of refs/temper/landing and HEAD' if base_id else 'no base'
    if scope == 'delivery':
        base_text = 'light checks, ' + base_text
    elif scope in ('close', 'goal'):
        base_text = f'goal head {commits[0]}, destination {commits[1]}, merge base {base_id}'
    elif scope == 'child':
        base_text = f'child checks, destination {commits[0]}, base {base_id}'
    header = f'pre-pr: {"refused" if refused else "passed"} ({base_text})\n'
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
    output = header
    for index, entry in enumerate(entries):
        remaining = len(entries) - index
        marker = f'[… {remaining} more failures]\n'
        if len((output + entry).encode()) + len(marker.encode()) > 8192:
            output += marker
            break
        output += entry
    print(output, end='', flush=True)
    return int(refused)


def goal_check(commits):
    """Explicit diagnostic replay of the accumulated goal with its pinned comparison."""
    root = Path.cwd()
    env = dict(os.environ, GIT_NO_REPLACE_OBJECTS='1')
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')
    git_dir = Path(git(['rev-parse', '--absolute-git-dir'], root, env).stdout.decode().strip())
    prefix = env.pop('TIERS_HOLD', '').split()
    base_id, _ = policy_context('goal', commits, root, git_dir, env)
    errors, warnings = [], []
    lints, tests = [], []
    with tempfile.TemporaryDirectory(prefix='goal-pre-pr-') as directory:
        tree_id, private = tree(root, git_dir, Path(directory) / 'index', env)
        if git(['rev-parse', commits[0] + '^{tree}'], root, env).stdout.strip() != tree_id.encode():
            raise ValueError('checkout snapshot differs from goal tree')
        paths = delivery_paths(base_id, tree_id, root, env, [])
        docs = any(documentation(path) for path in paths)
        selected = code_inputs(paths, {'packages': [], 'workspace_members': []}, root)
        prose = bool(paths) and not selected and all(path.endswith('.md') and 'docs' in Path(path).parts for path in paths)
        ledgers, _ = main_ledgers(env.pop('TIERS_LEDGER', ''), Path(directory), root)
        hold = contextlib.nullcontext()
        if not prose and paths:
            runner = runpy.run_path(str(Path(__file__).resolve().parents[1] / 'tiers/run'))
            hold = runner['hold'](prefix, 'pre-pr', root, env, output=sys.stdout)
        with hold:
            if paths and not prose:
                data = metadata(root, private)
                selected = code_inputs(paths, data, root)
                always = (root / '.config/gate/always.txt').read_text()
                lints, tests = select(data, {root / path: path for path in selected}, always)
                if selected:
                    errors += timed('format', lambda: formatting(data, root, private))
                    for name in ('skein', 'smith', 'jig', 'temper', 'hearth'):
                        errors += timed('old-forms ' + name, lambda: baselines(name, root, private))[1]
                    if lints:
                        errors += timed('clippy', lambda: clippy(lints, data, root, env))[1]
                if docs or selected:
                    tests = sorted(set(tests) | set(boundary(data, always)))
                inventory = test_binaries(data, root)
                helpers = [entry for name in tests for entry in inventory.get(name, [])]
                build_code, found = build_test_binaries(helpers, root, env)
                errors += found
                if tests and not build_code:
                    for profile in ('default', 'fuzzy'):
                        _, found, warning = timed(profile, lambda: one_process(
                            tests, root, env, rerun=rerun_bound(root, env), profile=profile, ledger=ledgers.get(profile)))
                        errors += found
                        if warning:
                            warnings.append(warning)
                        if any(error['name'].startswith('build') for error in found):
                            break
            if docs:
                errors += timed('citations', lambda: citations(tree_id, root, env))[1]
            if prose:
                errors += timed('outward', lambda: outward(root, private))[1]
            if any(p == '.temper/merge' or p.startswith('tools/merge/') and
                   p not in ('tools/merge/README.md', 'tools/merge/codecs.txt') for p in paths):
                errors += timed('merge', lambda: merge_tests(root, env))[1]
            errors += journeys(base_id, tree_id, [], root, env)[1]
            counts = [e for e in errors if e['name'].startswith('old-forms ') and e['key']]
            if counts:
                judge_base(base_id, git_dir, counts, root, env)
        if goal_base(commits, root, git_dir, env) != base_id:
            raise ValueError('goal comparison changed during checks')
        return finish(tree_id, base_id, errors, root, env, 'goal', commits, None, lints, tests, warnings)


def finish(tree_id, base_id, errors, root, env, scope, commits, heads, lints=(), tests=(), warnings=()):
    refused = any(not settled(e) for e in errors)
    location = failure_file(tree_id, base_id, refused, False, errors, root, env, scope, commits, heads)
    if location.startswith('not written'):
        errors.append(failure('check receipt', location))
    return summary(base_id, lints, tests, errors, None, location, warnings, scope, commits)


def handwritten_rust(root, paths):
    files = [p for p in paths if p.endswith('.rs') and (root / p).is_file() and not (root / p).is_symlink()]
    outputs = {line.split()[0] for line in lines(root, 'tools/merge/codecs.txt')} if files else set()
    return [p for p in files if p not in outputs]


def format_commands(root, files, packages, *, report=False):
    directories = package_directories(packages)
    groups = {}
    for path in files:
        name = owner(root / path, directories)
        edition = packages[name]['edition'] if name else '2024'
        config = _configuration(root / path, root)
        groups.setdefault((edition, config), []).append(path)
    for (edition, config), paths in groups.items():
        args = ['rustfmt', '--emit', 'json'] if report else ['rustfmt', '--check']
        if config:
            args.extend(['--config-path', str(config)])
        yield [*args, '--edition', edition, '--config', 'skip_children=true', '--', *paths]


class FormatError(ValueError):
    """rustfmt refused a selected source file."""


def format_delivery(root, paths, original, env, private):
    """Inspect in place. Only mismatches need new blobs in the existing private index."""
    files = handwritten_rust(root, paths)
    if not files:
        return original
    packages = members(metadata(root, env))
    print(f'pre-pr: formatting {len(files)} changed Rust files', flush=True)
    updates = bytearray()
    for args in format_commands(root, files, packages, report=True):
        scan = subprocess.run(args, cwd=root, env=env, capture_output=True)
        if scan.returncode:
            raise FormatError(tail(scan.stderr.decode(errors='replace')))
        options = args[3:args.index('--')]
        selected = {str(root / p): p for p in args[args.index('--') + 1:]}
        for mismatch in json.loads(scan.stdout):
            path = selected[mismatch['name']]
            entry = git(['--literal-pathspecs', 'ls-tree', '-z', original, '--', path], root, env)
            entry.check_returncode()
            mode, kind, blob = entry.stdout.split(b'\t', 1)[0].split()
            if kind != b'blob' or mode not in (b'100644', b'100755'):
                raise FormatError(f'expected a regular source file: {path}')
            source = git(['cat-file', 'blob', blob], root, env)
            source.check_returncode()
            fmt = subprocess.run(['rustfmt', '--emit', 'stdout', *options], cwd=root, env=env,
                                 input=source.stdout, capture_output=True)
            if fmt.returncode:
                raise FormatError(tail(fmt.stderr.decode(errors='replace')))
            formatted = subprocess.check_output(['git', 'hash-object', '-w', '--stdin'],
                                                cwd=root, env=env, input=fmt.stdout).strip()
            updates.extend(mode + b' ' + formatted + b'\t' + os.fsencode(path) + b'\0')
    if not updates:
        return original
    subprocess.run(['git', 'update-index', '-z', '--index-info'], cwd=root, env=private, input=updates, check=True)
    formatted = subprocess.check_output(['git', 'write-tree'], cwd=root, env=private).decode().strip()
    patch = git(['diff-tree', '-p', '--binary', '--no-renames', '--no-ext-diff', '--no-textconv',
                 '--src-prefix=a/', '--dst-prefix=b/', original, formatted], root, private)
    patch.check_returncode()
    if patch.stdout:
        subprocess.run(['git', 'apply', '--whitespace=nowarn', '-'], input=patch.stdout, cwd=root, env=env, check=True)
    return formatted


class UnknownScope(ValueError):
    """A saved checkout needs its main comparison refreshed before formatting."""


def local_scope(git_dir, base_id, root, env, merged):
    """Use only this checked snapshot or a main refresh in this checkout."""
    revision = git(['rev-parse', 'HEAD', 'HEAD^{tree}'], root, env)
    revision.check_returncode()
    head, tree_id = revision.stdout.decode().splitlines()
    if head == base_id:
        return merged
    record = git_dir / 'pre-pr-scope-v2' / base_id / tree_id
    try:
        tips = record.read_text().splitlines()
    except FileNotFoundError:
        pass
    else:
        if not all(FULL_COMMIT.fullmatch(tip) for tip in tips):
            raise ValueError('invalid local delivery scope')
        return list(dict.fromkeys([*merged, *tips]))

    # The worker rewrites its loose landing pin when it installs a checkout view.
    # An older FETCH_HEAD can belong to a previous occupant of the same slot.
    fetch = git_dir / 'FETCH_HEAD'
    pin = git_dir / 'refs/temper/landing'
    try:
        stamp = pin if pin.is_file() else git_dir / 'HEAD'
        fresh = fetch.stat().st_mtime_ns > stamp.stat().st_mtime_ns
        entries = fetch.read_bytes().splitlines() if fresh else []
    except FileNotFoundError:
        entries = []
    mains = [line.split(b'\t', 1)[0].decode('ascii') for line in entries
             if re.fullmatch(rb"[0-9a-f]+\t\tbranch 'main' of .+", line)]
    if len(mains) == 1 and FULL_COMMIT.fullmatch(mains[0]):
        main = mains[0]
        pending = git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD'], root, env)
        if pending.stdout.strip() == main.encode() or git(
                ['merge-base', '--is-ancestor', main, head], root, env).returncode == 0:
            return list(dict.fromkeys([*merged, main]))
    raise UnknownScope('formatting scope unavailable in this checkout. Fetch main into FETCH_HEAD '
                       '(git fetch origin main), merge FETCH_HEAD, and retry delivery.')


def remember_merges(git_dir, base_id, tree_id, tips):
    directory = git_dir / 'pre-pr-scope-v2' / base_id
    directory.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', dir=directory, delete=False) as file:
        file.writelines(tip + '\n' for tip in tips)
    os.replace(file.name, directory / tree_id)


def workspace_checks(root, paths, tree_id, base_id, merged, env, private, errors, outcomes, reds=(), *,
                     completed_goal=False, already_formatted=False):
    start = time.monotonic()
    rust_files = [] if already_formatted else handwritten_rust(root, paths)
    package_paths = rust_files
    if completed_goal:
        planner = runpy.run_path(str(Path(__file__).with_name('workspace.py')))
        package_paths = [p for p in paths if p.endswith('.rs') or planner['owns_package'](root, p)]
    data = metadata(root, env) if package_paths else {'packages': [], 'workspace_members': []}
    packages = members(data)

    suites, rust_commands = [], ()
    if completed_goal and packages:
        with (root / 'tools/tiers/layers.toml').open('rb') as file:
            layers = tomllib.load(file)
        groups = planner['package_plan'](root, code_inputs(package_paths, data, root), packages, layers)
        suites = sorted(name for targets, names in groups.items() if targets for name in names)
        rust_commands = planner['rust_commands'](groups, test_binaries(data, root))
    print(f'pre-pr: selection: {len(paths)} changed paths, {len(suites)} test packages, '
          f'{time.monotonic() - start:.3f} s', flush=True)

    def command(label, args):
        print(f"pre-pr: {label}: {shlex.join(args)}", flush=True)
        start = time.monotonic()
        rc, log = run(args, root, env, stdout=True)
        print(f'pre-pr: {label}: exit {rc}, {time.monotonic() - start:.3f} s', flush=True)
        if rc:
            errors.append(failure(label, log))
        return rc

    for args in format_commands(root, rust_files, packages):
        if command('format', args):
            return []
    for name in ('skein', 'smith', 'jig', 'temper', 'hearth'):
        if any((p.startswith(name + '/') and not documentation(p)) or p.startswith('tools/old_forms/') for p in paths):
            errors += timed('old-forms ' + name, lambda: baselines(name, root, private))[1]
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
        rc, found = timed('outward', lambda: outward(root, private, None if full_scan else paths))
        errors += found
        if full_scan and rc in (0, 1):
            outcomes[key] = {'outcome': 'passed' if rc == 0 else 'failed'}
    if not completed_goal and 'tools/tiers/budgets.toml' in paths:
        errors += figures(base_id or 'HEAD', tree_id, root, env, merged)[1]
    errors += journeys(base_id, tree_id, [] if completed_goal else paths, root, env)[1]
    for args in rust_commands:
        if command('focused tests' if args[1] == 'nextest' else 'build check', args):
            break
    return suites


HOOKS = {'.temper/pre-pr', '.temper/goal-child-pre-pr', '.temper/goal-pre-pr'}


def delivery(scope, commits):
    root = Path.cwd()
    env = dict(os.environ, CARGO_NET_OFFLINE='true', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')
    git_dir = Path(subprocess.check_output(['git', 'rev-parse', '--absolute-git-dir'], cwd=root, env=env).decode().strip())
    if scope == 'child':
        base_id, heads = policy_context(scope, commits, root, git_dir, env)
    else:
        base_id, _ = base(root, env)
        heads = None
    errors, outcomes = [], {}
    with tempfile.TemporaryDirectory(prefix='pre-pr-index-') as directory:
        tree_id, private = tree(root, git_dir, Path(directory) / 'index', env)
        merged = merged_tips(base_id or 'HEAD', root, env, heads)
        tested = []
        known_scope = True
        paths = None
        if base_id:
            try:
                merged = local_scope(git_dir, base_id, root, env, merged)
            except UnknownScope as error:
                known_scope = False
                paths = delivery_paths(base_id, tree_id, root, env, merged)
                if scope == 'delivery' and handwritten_rust(root, paths):
                    errors.append(failure('formatting scope', str(error)))
                    verdict = finish(tree_id, base_id, errors, root, env, scope, commits, heads)
                    if (git_dir / 'temper/reds-wait').is_file():
                        check_results(tree_id, base_id, verdict, (), (), {}, errors, root, env)
                    return verdict
        reds, _, marker = goal_reds(root, env)
        try:
            if paths is None:
                paths = delivery_paths(base_id or 'HEAD', tree_id, root, env, merged)
            if scope == 'delivery':
                formatted = format_delivery(root, paths, tree_id, env, private)
                if formatted != tree_id:
                    tree_id = formatted
                    paths = delivery_paths(base_id or 'HEAD', tree_id, root, env, merged)
            tested = workspace_checks(root, paths, tree_id, base_id, merged, env, private, errors, outcomes, reds,
                                      already_formatted=scope == 'delivery')
        except FormatError as error:
            errors.append(failure('format', str(error)))
        except (OSError, ValueError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
            errors.append(failure('workspace checks', str(error)))
        if scope == 'child' and policy_context(scope, commits, root, git_dir, env) != (base_id, heads):
            errors.append(failure('child checkout', 'The head or destination changed during checks'))
        current, _ = tree(root, git_dir, Path(directory) / 'final-index', env)
        if tree_id != current:
            errors.append(failure('workspace', 'The delivery tree changed during checks'))
        inherited(reds, errors)
        if scope == 'delivery' and base_id and known_scope and all(settled(error) for error in errors):
            remember_merges(git_dir, base_id, tree_id, merged)
        verdict = finish(tree_id, base_id, errors, root, env, scope, commits, heads, tests=tested)
        if marker and scope == 'delivery':
            check_results(tree_id, base_id, verdict, reds, [], outcomes, errors, root, env)
        return verdict


def close_check(destination):
    """Check the entire staged goal against its main comparison, including an empty close."""
    root = Path.cwd()
    env = dict(os.environ, CARGO_NET_OFFLINE='true', GIT_NO_LAZY_FETCH='1', GIT_NO_REPLACE_OBJECTS='1')
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')

    def read(*args):
        return subprocess.check_output(['git', *args], cwd=root, env=env).decode().strip()

    verify_commit('destination', destination, root, env)
    head = read('rev-parse', 'HEAD')
    bases = read('merge-base', '--all', head, destination).splitlines()
    if len(bases) != 1:
        raise ValueError('goal close requires one merge base with the destination')
    base_id = bases[0]
    staged = read('write-tree')
    git_dir = Path(read('rev-parse', '--absolute-git-dir'))
    errors, tested = [], []
    with tempfile.TemporaryDirectory(prefix='goal-close-index-') as scratch:
        def verify_snapshot(index):
            current, private = tree(root, git_dir, Path(scratch) / index, env)
            if current != staged or read('write-tree') != staged or read('rev-parse', 'HEAD') != head:
                raise ValueError('goal checkout differs from the staged close tree')
            return private

        try:
            private = verify_snapshot('before')
            paths = delivery_paths(base_id, staged, root, env, [])
            tested = workspace_checks(root, paths, staged, base_id, [], env, private, errors, {}, completed_goal=True)
            verify_snapshot('after')
        except (OSError, KeyError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
            errors.append(failure('goal close', str(error)))
        return finish(staged, base_id, errors, root, env, 'close', (head, destination), None, tests=tested)


def main(args):
    if not args:
        return delivery('delivery', ())
    if len(args) == 2 and args[0] == '--close':
        return close_check(args[1])
    if len(args) == 2 and args[0] == '--child':
        return delivery('child', args[1:])
    if len(args) == 3 and args[0] == '--goal':
        return goal_check(args[1:])
    raise ValueError('expected no arguments, --goal HEAD DESTINATION, --child DESTINATION, or --close DESTINATION')


if __name__ == '__main__':
    try:
        sys.exit(main(sys.argv[1:]))
    except (OSError, ValueError, KeyError, RuntimeError, subprocess.SubprocessError) as error:
        sys.exit(f'pre-pr: refused: {error}')
