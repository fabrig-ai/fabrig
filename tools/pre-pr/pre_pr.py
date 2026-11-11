"""The delivery check (tools/pre-pr/design.md, 2–9).

The private snapshot leaves the caller's index alone; only head failures need a
base worktree. Every tool streams before the bounded, final verdict.
"""
import fnmatch
import json
import os
from pathlib import Path
import re
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


def base(root, env):
    """Read the landing pin and merge heads, never moving the pin (section 2)."""
    if git(['rev-parse', '--verify', '--quiet', 'refs/temper/landing^{commit}'], root, env).returncode:
        return None, 'no refs/temper/landing'
    heads = ['refs/temper/landing', 'HEAD']
    if not git(['rev-parse', '--verify', '--quiet', 'MERGE_HEAD'], root, env).returncode:
        heads.append('MERGE_HEAD')
    result = git(['merge-base'] + heads, root, env)
    if result.returncode or not result.stdout.strip():
        return None, 'no merge base of refs/temper/landing and HEAD'
    commit = result.stdout.decode().splitlines()[0]
    while git(['log', '-1', '--format=%s', commit], root, env).stdout.decode().rstrip('\n') == (
            'Unchecked: work in progress, saved before a question'):
        parent = git(['rev-parse', '--verify', '--quiet', commit + '^1'], root, env)
        if parent.returncode:
            break
        commit = parent.stdout.decode().strip()
    return commit, None


def tree(root, git_dir, index, env):
    """Write all nonignored sources through a private index (section 3)."""
    private = env.copy()
    private['GIT_INDEX_FILE'] = str(index)
    source = Path(env.get('GIT_INDEX_FILE', git_dir / 'index'))
    if source.exists():
        shutil.copyfile(source, index)
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


def select(data, changed, always):
    """C, direct D and prefixed R, sorted by package name (section 3)."""
    packages = members(data)
    directories = sorted(((Path(p['manifest_path']).parent, name) for name, p in packages.items()),
                         key=lambda entry: len(entry[0].parts), reverse=True)
    changed_packages = set()
    for path in changed:
        for directory, name in directories:
            if path.is_relative_to(directory):
                changed_packages.add(name)
                break
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


def lines(root, name):
    path = root / name
    return [line.split('#', 1)[0].strip() for line in path.read_text().splitlines()
            if line.split('#', 1)[0].strip()] if path.exists() else []


def documentation(path):
    return path.endswith('.md') or 'docs' in Path(path).parts[:-1]


def code_inputs(paths, data, root):
    """Use package ownership and the gate's declared readers (section 3)."""
    directories = [Path(p['manifest_path']).parent for p in members(data).values()]
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
                or path == '.temper/pre-pr'
                or (path.startswith('skein/scripts/') and path.endswith('.sh'))
                or any(path.startswith(prefix) for prefix in prefixes)
                or any((root / path).is_relative_to(directory) for directory in directories)
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


def citations(base_id, tree_id, root, env):
    """The citation gate compares counted findings and design sizes itself."""
    inherited_count = 0

    def consume(line, log):
        nonlocal inherited_count
        log.add(line)
        sys.stdout.buffer.write(line)
        sys.stdout.flush()
        match = re.search(rb'citations: (\d+) new broken since .* \((\d+) in all\)', line)
        if match:
            inherited_count = int(match[2]) - int(match[1])

    code, log = run(['python3', '-I', '-B', 'tools/citations/citations.py',
                     '--base', base_id, '--head', tree_id], root, env, consume)
    errors = [failure('citations', log)] if code else []
    if inherited_count:
        error = failure(f'citations ({inherited_count} broken at base)', '')
        error['inherited'] = True
        errors.append(error)
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


def nextest(packages, root, env, expression=None, build=False):
    """Use an isolated JUnit overlay and nextest's exit verdict (section 4)."""
    with tempfile.TemporaryDirectory(prefix='temper-pre-pr-') as directory:
        directory = Path(directory)
        junit = directory / 'junit.xml'
        overlay = directory / 'nextest.toml'
        overlay.write_text('[profile.default.junit]\npath = ' + json.dumps(str(junit)) + '\n')
        arguments = ['cargo', 'nextest', 'run', '--profile', 'default']
        arguments += package_args(packages) + ['--tool-config-file', f'temper-pre-pr:{overlay}']
        if expression:
            arguments += ['-E', expression]
        arguments += ['--no-run'] if build else ['--no-fail-fast', '--no-tests=warn']
        code, log = run(arguments, root, env, stdout=True)
        if code == 0:
            return code, []
        if code == 101:
            return code, [failure('build', 'no tests ran\n' + log)]
        if code != 100:
            return code, [failure('check nextest', f'exit {code}\n' + log)]
        errors = []
        try:
            document = ET.parse(junit)
            for suite in document.iter('testsuite'):
                binary = suite.get('name', '')
                for case in suite.findall('testcase'):
                    faults = case.findall('failure') + case.findall('error')
                    if not faults:
                        continue
                    name = case.get('name', '')
                    output = '\n'.join(''.join(node.itertext()) for node in faults +
                                       case.findall('system-out') + case.findall('system-err'))
                    # Cargo's binary id starts with its owning package's name.
                    package = binary.split('::', 1)[0]
                    errors.append(failure(f'test {binary} {name}', output, (binary, name), package))
        except (OSError, ET.ParseError) as error:
            return code, [failure('check nextest', f'JUnit: {error}\n' + log)]
        if not errors:
            errors.append(failure('check nextest', 'exit 100 without JUnit failures\n' + log))
        return code, errors


def refresh(roots, paths):
    """Make both checkouts' changed inputs newer than shared artifacts (section 5)."""
    stamp = time.time_ns()
    for root in roots:
        for name in paths:
            path = root / name
            if path.is_file() and path.resolve().is_relative_to(root):
                os.utime(path, ns=(path.stat().st_atime_ns, stamp))


def inherited(base_id, git_dir, data, lints, tests, errors, root, env, paths):
    """Restore head freshness after any base judgment (section 5)."""
    try:
        return judge_base(base_id, git_dir, data, lints, tests, errors, root, env, paths)
    finally:
        refresh((root, git_dir / 'temper-pre-pr/base'), paths)


def judge_base(base_id, git_dir, data, lints, tests, errors, root, env, paths):
    """Judge only failures seen at head, in a kept detached worktree (section 5)."""
    base_env = env.copy()
    base_env.pop('GIT_INDEX_FILE', None)
    if data is not None:
        base_env['CARGO_TARGET_DIR'] = str(data['target_directory'])
    worktree = git_dir / 'temper-pre-pr/base'
    worktree.parent.mkdir(parents=True, exist_ok=True)
    if (worktree / '.git').exists():
        commands = [(['checkout', '--detach', '--force', base_id], worktree),
                    (['clean', '-ffdxq'], worktree)]
    else:
        commands = [(['worktree', 'add', '--detach', '--force', str(worktree), base_id], root)]
    for arguments, cwd in commands:
        code, log = run(['git'] + arguments, cwd, base_env)
        if code:
            return 'base could not judge: ' + log
    refresh((worktree, root), paths)
    reasons = []
    count_errors = [e for e in errors if e['name'].startswith('old-forms ') and e['key']]
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
    if not any(e['name'].startswith(('clippy ', 'test ')) or e['name'] == 'build' for e in errors):
        if reasons:
            for error in count_errors:
                error['inherited'] = False
        return '\n'.join(reasons)
    try:
        base_data = metadata(worktree, base_env)
    except (OSError, ValueError, RuntimeError) as error:
        for finding in count_errors:
            finding['inherited'] = False
        return f'base could not judge: {error}'
    available = members(base_data)
    lint_errors = [e for e in errors if e['name'].startswith('clippy ')]
    selected = sorted({e['package'] for e in lint_errors} & available.keys() & set(lints))
    if selected:
        code, base_errors = timed('base clippy', lambda: clippy(selected, base_data, worktree, base_env))
        if code not in (0, 101) or any(e['name'] == 'check clippy' for e in base_errors):
            reasons.append('base could not judge clippy: ' + ''.join(e['log'] for e in base_errors))
        else:
            keys = {e['key'] for e in base_errors}
            for error in lint_errors:
                error['inherited'] = error['key'] in keys
    test_errors = [e for e in errors if e['name'].startswith('test ')]
    selected = sorted({e['package'] for e in test_errors} & available.keys())
    if selected:
        expression = ' | '.join(f'(binary_id(={e["key"][0]}) & test(={e["key"][1]}))'
                                for e in test_errors if e['package'] in selected)
        code, base_errors = timed('base tests', lambda: nextest(selected, worktree, base_env, expression))
        if code not in (0, 100) or any(e['name'] == 'check nextest' for e in base_errors):
            reasons.append('base could not judge tests: ' + ''.join(e['log'] for e in base_errors))
        else:
            keys = {e['key'] for e in base_errors}
            for error in test_errors:
                error['inherited'] = error['key'] in keys
    build_errors = [e for e in errors if e['name'] == 'build']
    selected = sorted(set(tests) & available.keys())
    if build_errors and selected:
        code, base_errors = timed('base build', lambda: nextest(selected, worktree, base_env, build=True))
        if code == 101:
            for error in build_errors:
                error['inherited'] = True
        elif code:
            reasons.append('base could not judge build: ' + ''.join(e['log'] for e in base_errors))
    if reasons:
        for error in errors:
            if error['key'] is not None or error['name'] == 'build':
                error['inherited'] = False
    return '\n'.join(reasons)


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
        pending[name] = seconds
    else:
        status = 'failed' if failed else 'passed'
        if isinstance(result, tuple) and result[1] and isinstance(result[1], list):
            if all(error['inherited'] for error in result[1]):
                status = 'inherited'
        print(f'pre-pr: {name}: {status}, {seconds:.3f} s', flush=True)
    return result


def summary(base_id, lints, tests, errors, reason):
    """Keep the first failure whole and count what the host's tail cannot hold."""
    refused = any(not e['inherited'] for e in errors)
    base_text = f'base {base_id}, merge base of refs/temper/landing and HEAD' if base_id else 'no base'
    header = f'pre-pr: {"refused" if refused else "passed"} ({base_text})\n'
    header += f'lints: {" ".join(lints)}\ntests: {" ".join(tests)}\n'
    if reason:
        header += tail(reason)
    entries = [e['name'] + '\n' + e['log'] for e in errors if not e['inherited']]
    entries += ['inherited: ' + e['name'] + (' (no tests ran)' if e['name'] == 'build' else '') + '\n'
                for e in errors if e['inherited']]
    # Even large package sets must leave room for the first failure and the count.
    header = header.encode()[:4096].decode('utf-8', errors='ignore') + '\n'
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


def main():
    root = Path.cwd()
    env = os.environ.copy()
    env.setdefault('TEMPER_INTERNAL_BUNDLE', 'none')
    git_dir = Path(git(['rev-parse', '--absolute-git-dir'], root, env).stdout.decode().strip())
    base_id, why = base(root, env)
    errors = []
    lints, tests = [], []
    with tempfile.TemporaryDirectory(prefix='temper-pre-pr-') as directory:
        tree_id, private = tree(root, git_dir, Path(directory) / 'index', env)
        cache = git_dir / 'pre-pr-passed'
        record = f'{tree_id} {base_id or "-"}'
        if cache.exists() and cache.read_text().strip() == record:
            print(f'pre-pr: passed before (tree {tree_id}, base {base_id or "-"})', flush=True)
            return 0
        data = None
        paths, selected = [], []
        docs = False
        if not base_id:
            print(f'pre-pr: no base ({why}): the format check and the counted baselines alone', flush=True)
            code_checks = True
        else:
            diff = git(['diff-tree', '-r', '--no-renames', '--name-only', '-z', base_id, tree_id], root, env)
            if diff.returncode:
                errors.append(failure('check selection', diff.stderr.decode(errors='replace')))
            else:
                paths = [os.fsdecode(p) for p in diff.stdout.split(b'\0') if p]
            docs = any(documentation(path) for path in paths)
            try:
                # Plain documentation needs neither Cargo nor package selection.
                empty = {'packages': [], 'workspace_members': []}
                selected = code_inputs(paths, empty, root)
                if selected or any(not documentation(path) for path in paths):
                    data = metadata(root, private)
                    selected = code_inputs(paths, data, root)
            except (OSError, ValueError, RuntimeError) as error:
                errors.append(failure('check selection', str(error)))
            code_checks = bool(selected)
            kind = 'code and docs' if code_checks and docs else 'code only' if code_checks else 'docs only'
            if code_checks:
                print(f'pre-pr: {kind}: format, counted baselines, clippy, focused tests '
                      f'(code input {selected[0]!r})', flush=True)
            if docs:
                path = next(path for path in paths if documentation(path))
                print(f'pre-pr: {kind}: citations and design size limits (docs input {path!r})', flush=True)
            if not code_checks and not docs:
                print('pre-pr: no code or docs inputs changed', flush=True)

        def format_check():
            nonlocal data
            try:
                if data is None:
                    data = metadata(root, private)
                return formatting(data, root, private)
            except (OSError, ValueError, RuntimeError) as error:
                return [failure('format', str(error))]

        pending = {}
        if code_checks:
            errors += timed('format', format_check)
            for name in ('skein', 'smith', 'jig', 'temper', 'hearth'):
                _, found = timed('old-forms ' + name, lambda: baselines(name, root, private), pending)
                errors += found
            if base_id and data is not None:
                always = root / '.config/gate/always.txt'
                lints, tests = select(data, {root / p: p for p in selected},
                                      always.read_text() if always.exists() else '')
                if not lints and not tests:
                    print('pre-pr: no package changed', flush=True)
                if lints:
                    _, found = timed('clippy', lambda: clippy(lints, data, root, env), pending)
                    errors += found
                if tests:
                    _, found = timed('tests', lambda: nextest(tests, root, env), pending)
                    errors += found
        if docs:
            _, found = timed('citations', lambda: citations(base_id, tree_id, root, env), pending)
            errors += found
        if base_id and any(e['key'] is not None or e['name'] == 'build' for e in errors):
            why = inherited(base_id, git_dir, data, lints, tests, errors, root, env, paths)
        for name, seconds in pending.items():
            if name == 'clippy':
                prefixes = ('clippy ', 'check clippy')
            elif name == 'tests':
                prefixes = ('test ', 'build', 'check nextest')
            else:
                prefixes = (name,)
            found = [e for e in errors if e['name'].startswith(prefixes)]
            status = 'inherited' if found and all(e['inherited'] for e in found) else 'failed'
            print(f'pre-pr: {name}: {status}, {seconds:.3f} s', flush=True)
        verdict = summary(base_id, lints, tests, errors, why if base_id else None)
        if not verdict:
            temporary = git_dir / 'pre-pr-passed.tmp'
            temporary.write_text(record + '\n')
            temporary.replace(cache)
        return verdict


if __name__ == '__main__':
    sys.exit(main())
