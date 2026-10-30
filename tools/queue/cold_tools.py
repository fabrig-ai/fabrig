"""Private prepared tools checkout; precheck.md, sections 1.1 and 4.

The receipt and full manifest outlive the owned, removable auxiliary root.
"""

import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess


def encoded(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n").encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def identity(path):
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or path.is_symlink() or info.st_uid != os.getuid():
        raise RuntimeError(f"unsafe cold directory: {path}")
    return [info.st_dev, info.st_ino]


def safe_path(root, name):
    if not name or name.startswith("/") or any(p in ("", ".", "..") or p.lower() == ".git" for p in name.split("/")):
        raise RuntimeError("unsafe cold tracked path")
    path = root / name
    for parent in path.parents:
        if parent == root:
            break
        if parent.is_symlink():
            raise RuntimeError("tracked path traverses a symlink")
    return path


def no_links(path):
    if any(part.is_symlink() for part in (path, *path.parents)):
        raise RuntimeError('symlink in cold input path')


def manifest(repo, entries):
    result = []
    for entry in entries:
        attributes, name = entry.split("\t", 1)
        mode, oid, stage = attributes.split()
        if stage != "0" or mode not in ("100644", "100755", "120000"):
            raise RuntimeError("cold tools require a resolved ordinary index")
        if any(part == "target" for part in Path(name).parts):
            raise RuntimeError("tracked generated target path is not cold")
        path = safe_path(repo, name)
        row = {"name": name, "index_mode": mode, "index_oid": oid}
        try:
            info = path.lstat()
        except FileNotFoundError:
            row.update(kind="missing")
        else:
            if stat.S_ISLNK(info.st_mode):
                target = os.readlink(path)
                resolved = (path.parent / target).resolve()
                if os.path.isabs(target) or not resolved.is_relative_to(repo):
                    raise RuntimeError("tracked symlink escapes cold checkout")
                row.update(kind="symlink", target=target)
                body = os.fsencode(target)
            elif stat.S_ISREG(info.st_mode):
                row.update(kind="file")
                body = path.read_bytes()
            else:
                raise RuntimeError("unsupported cold tracked file")
            row.update(mode=stat.S_IMODE(info.st_mode), size=len(body), sha256=digest(body))
            algorithm = 'sha256' if len(oid) == 64 else 'sha1'
            row['raw_oid'] = hashlib.new(algorithm, b'blob ' + str(len(body)).encode() + b'\0' + body).hexdigest()
        result.append(row)
    return result


def raw_tree(rows, algorithm):
    tree = {}
    for row in rows:
        if row['kind'] == 'missing':
            continue
        directory = tree
        parts = row['name'].split('/')
        for part in parts[:-1]:
            directory = directory.setdefault(part, {})
        mode = '120000' if row['kind'] == 'symlink' else ('100755' if row['mode'] & stat.S_IXUSR else '100644')
        directory[parts[-1]] = (mode, row['raw_oid'])
    def encode(directory):
        data = b''
        for name, value in sorted(directory.items(), key=lambda item: os.fsencode(item[0]) +
                                  (b'/' if isinstance(item[1], dict) else b'')):
            mode, oid = ('40000', encode(value)) if isinstance(value, dict) else value
            data += mode.encode() + b' ' + os.fsencode(name) + b'\0' + bytes.fromhex(oid)
        return hashlib.new(algorithm, b'tree ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    return encode(tree)


def logical_tree(logical, algorithm):
    rows = []
    for entry in logical.split('\0'):
        if not entry:
            continue
        attributes, name = entry.split('\t', 1)
        mode, oid, stage = attributes.split()
        if stage != '0' or mode not in ('100644', '100755', '120000'):
            raise RuntimeError('invalid cold index manifest')
        safe_path(Path('/cold'), name)
        rows.append({'name': name, 'kind': 'symlink' if mode == '120000' else 'file',
                     'mode': 0o755 if mode == '100755' else 0o644, 'raw_oid': oid})
    return raw_tree(rows, algorithm)


# Names are recorded, never their possibly sensitive values.
ARTIFACT_OVERRIDES = {"CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_ENCODED_RUSTFLAGS",
                      "RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTC",
                      "TIERS_CPU_WRAPPER", "TIERS_CPU_SETTLEMENT_FIXTURE", "CARGO_HOME",
                      "TMPDIR", "TMP", "TEMP", "GIT_CONFIG_GLOBAL", "GIT_CONFIG_SYSTEM",
                      "GIT_CONFIG_NOSYSTEM", "GIT_CONFIG_COUNT", "GIT_CONFIG_PARAMETERS"}


def environment(root):
    removed = sorted(key for key in os.environ if key in ARTIFACT_OVERRIDES or
                     key.startswith(("TIERS_CPU_", "CARGO_TARGET_", "CARGO_BUILD_", "GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_")))
    env = {key: value for key, value in os.environ.items() if key not in removed}
    env.update(TMPDIR=str(root / "scratch"), TMP=str(root / "scratch"), TEMP=str(root / "scratch"),
               CARGO_HOME=str(root / "cargo-home"), GIT_CONFIG_GLOBAL="/dev/null",
               GIT_CONFIG_SYSTEM="/dev/null", GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0")
    return env, {"removed": removed, "redirected": ["TMPDIR", "TMP", "TEMP", "CARGO_HOME"],
                 "git_config": "isolated"}


class ColdTools:
    def __init__(self, owner, commands, policy):
        self.owner = owner
        self.root = owner.log_directory / "cold-tools"
        self.repo = self.root / "checkout"
        self.env, handling = environment(self.root)
        self.receipt = {"version": 1, "policy": policy, "root": str(self.root), "checkout": str(self.repo),
                        "parent": str(owner.log_directory), "parent_identity": identity(owner.log_directory),
                        "root_identity": None, "checkout_identity": None, "state": "planned",
                        "base": owner.base, "tip": owner.tip, "head": owner.base,
                        "environment": handling, "observations": [], "cleanup": "pending",
                        "completion": "not-started", "outcomes": []}
        owner.record["cold_tools"] = self.receipt
        owner.save()  # Durable intent precedes creation, including capacity refusal.
        self.commands = commands

    def save(self):
        self.owner.save()

    def git(self, *args, input=None, env=None):
        result = subprocess.run(["git", "--no-optional-locks", "--no-replace-objects", "-c", "core.hooksPath=/dev/null",
                                 *args], cwd=self.repo, env=self.env if env is None else env,
                                input=input, stdin=subprocess.DEVNULL if input is None else None,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode:
            raise RuntimeError(f"cold git {args[0]} failed: {os.fsdecode(result.stderr).strip()}")
        return result.stdout

    def prepare(self):
        owner = self.owner
        logical = owner.git("ls-files", "--stage", "-z")
        entries = [entry for entry in logical.split("\0") if entry]
        rows = manifest(owner.repo, entries)
        objects = Path(owner.git("rev-parse", "--path-format=absolute", "--git-path", "objects"))
        no_links(objects)
        if ((objects / "info/alternates").exists() or (objects / "info/alternates").is_symlink() or
                owner.git("rev-parse", "--is-shallow-repository") != "false" or
                owner.git("for-each-ref", "--format=%(refname)", "refs/replace/") or
                owner.state_path("info/grafts").exists()):
            raise RuntimeError("cold object store must be self-contained, complete and unreplaced")
        object_files = []
        for directory, directories, files in os.walk(objects, followlinks=False):
            for name in directories:
                if (Path(directory) / name).is_symlink():
                    raise RuntimeError("symlink in cold object store")
            for name in files:
                source = Path(directory) / name
                info = source.lstat()
                if not stat.S_ISREG(info.st_mode):
                    raise RuntimeError("non-regular cold object")
                if name.endswith('.promisor'):
                    raise RuntimeError("promisor object stores are not cold self-contained inputs")
                object_files.append((source, source.relative_to(objects), info.st_size))
        merge = {name: owner.state_path(name).read_bytes() for name in
                 ("HEAD", "MERGE_HEAD", "MERGE_MSG", "MERGE_MODE", "ORIG_HEAD") if owner.state_path(name).exists()}
        index_tree = owner.git("write-tree")
        full = {"entries": logical, "manifest": rows,
                "merge": {name: digest(body) for name, body in merge.items()}}
        full_path = owner.log_directory / "cold-input.json"
        owner.artifact(full_path, full)
        self.receipt.update(input_manifest={"path": str(full_path), "sha256": digest(full_path.read_bytes())},
                            input={"entries_sha256": digest(os.fsencode(logical)), "manifest_sha256": digest(encoded(rows)),
                                   "index_tree": index_tree, "merge": full['merge']})
        self.receipt['input']['raw_tree'] = raw_tree(rows, owner.git('rev-parse', '--show-object-format'))
        # Charge physical blocks for copied contents, objects, index/raw-tree construction,
        # and the measured directory/entry metadata. This is static admission, not a quota.
        block = os.statvfs(owner.log_directory).f_frsize
        rounded = lambda size: ((size + block - 1) // block) * block
        contents = sum(rounded(row.get('size', 0)) for row in rows)
        object_bytes = sum(rounded(size) for _, _, size in object_files)
        directories = {str(Path(row['name']).parent) for row in rows}
        metadata = block * (len(rows) + len(object_files) + len(directories) + 32)
        requirement = contents * 2 + object_bytes + metadata + rounded(len(os.fsencode(logical))) * 2
        free = shutil.disk_usage(owner.log_directory).free
        admitted = free - requirement >= owner.min_free_bytes
        self.receipt['capacity'] = {"required_bytes": requirement, "tracked_bytes": contents,
                                    "object_bytes": object_bytes, "metadata_bytes": metadata,
                                    "free_bytes": free, "min_free_bytes": owner.min_free_bytes,
                                    "admitted": admitted, "policy": "measured-static-copy"}
        self.save()
        if not admitted:
            raise RuntimeError("cold tools copy would consume caller minimum free bytes")
        self.root.mkdir(mode=0o700)
        self.receipt.update(root_identity=identity(self.root), state="creating")
        self.save()
        self.repo.mkdir(mode=0o700)
        self.receipt['checkout_identity'] = identity(self.repo)
        self.save()
        for name in ('scratch', 'cargo-home'):
            (self.root / name).mkdir(mode=0o700)
        object_format = owner.git("rev-parse", "--show-object-format")
        self.git("init", "--template=", "--object-format=" + object_format)
        for source, relative, size in object_files:
            dest = self.repo / '.git/objects' / relative
            dest.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            shutil.copyfile(source, dest)
            os.chmod(dest, 0o600)
            if dest.stat().st_size != size:
                raise RuntimeError("object store changed during cold copy")
        self.git('fsck', '--full', '--no-reflogs', '--no-dangling')
        self.git('update-index', '-z', '--index-info', input=os.fsencode(logical))
        for name, body in merge.items():
            (self.repo / '.git' / name).write_bytes(body)
        for row in rows:
            if row['kind'] == 'missing':
                continue
            dest = safe_path(self.repo, row['name'])
            dest.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
            if row['kind'] == 'symlink':
                dest.symlink_to(row['target'])
            else:
                shutil.copyfile(owner.repo / row['name'], dest)
                os.chmod(dest, row['mode'])
        copied = manifest(self.repo, entries)
        if copied != rows or self.git('ls-files', '--stage', '-z') != os.fsencode(logical):
            raise RuntimeError("cold prepared source/index copy differs")
        if self.git('rev-parse', 'HEAD').decode().strip() != owner.base:
            raise RuntimeError("cold HEAD differs from prepared base")
        observed_merge = {name: digest((self.repo / '.git' / name).read_bytes()) for name in merge}
        if observed_merge != full['merge']:
            raise RuntimeError('cold merge state differs from prepared source')
        regular = [row for row in copied if row['kind'] == 'file']
        # Git's stdin-paths accepts C-quoted paths, including literal newlines and
        # non-UTF8 names. Batch raw hashing avoids one process per tracked file.
        quoted = lambda path: b'"' + b''.join(bytes([n]) if 32 <= n < 127 and n not in (34, 92)
                                              else ('\\%03o' % n).encode() for n in os.fsencode(path)) + b'"\n'
        hashed = self.git('hash-object', '-w', '--no-filters', '--stdin-paths',
                          input=b''.join(quoted(self.repo / row['name']) for row in regular)).decode().splitlines()
        if hashed != [row['raw_oid'] for row in regular]:
            raise RuntimeError('cold raw blob hashes differ')
        raw_entries = []
        for row in copied:
            if row['kind'] == 'missing':
                continue
            oid = row['raw_oid']
            if row['kind'] == 'symlink':
                if self.git('hash-object', '-w', '--stdin', input=os.fsencode(row['target'])).decode().strip() != oid:
                    raise RuntimeError('cold symlink blob hash differs')
            mode = '120000' if row['kind'] == 'symlink' else ('100755' if row['mode'] & stat.S_IXUSR else '100644')
            raw_entries.append(os.fsencode(f"{mode} {oid}\t{row['name']}\0"))
        raw_index = self.root / 'raw.index'
        raw_env = dict(self.env, GIT_INDEX_FILE=str(raw_index))
        self.git('read-tree', '--empty', env=raw_env)
        self.git('update-index', '-z', '--index-info', input=b''.join(raw_entries), env=raw_env)
        working_tree = self.git('write-tree', env=raw_env).decode().strip()
        if working_tree != self.receipt['input']['raw_tree']:
            raise RuntimeError('independent raw working tree differs')
        expected = [(name, argv) for name, argv, _, timeout in self.commands if timeout is not None]
        observed = [(name, argv) for name, argv, _ in owner.mq.tools_commands(self.repo)]
        if observed != expected:
            raise RuntimeError("cold discovered tools plan differs")
        self.receipt.update(observed={"entries_sha256": digest(self.git('ls-files', '--stage', '-z')),
                                      "manifest_sha256": digest(encoded(copied)),
                                      "index_tree": self.git('write-tree').decode().strip(),
                                      "raw_tree": working_tree, "head": owner.base, "merge": observed_merge,
                                      "independent_objects": True,
                                      "objects_identity": identity(self.repo / '.git/objects')},
                            tools_plan=[{'name': name, 'argv': argv} for name, argv in observed],
                            state='prepared', completion='known')
        full['ownership'] = {key: self.receipt[key] for key in
                             ('parent', 'parent_identity', 'root', 'root_identity', 'checkout', 'checkout_identity')}
        full['ownership']['objects_identity'] = self.receipt['observed']['objects_identity']
        owner.artifact(full_path, full)
        self.receipt['input_manifest']['sha256'] = digest(full_path.read_bytes())
        self.save()
        allocated = sum(Path(directory).lstat().st_blocks * 512 +
                        sum((Path(directory) / name).lstat().st_blocks * 512 for name in names)
                        for directory, _, names in os.walk(self.root))
        post_free = shutil.disk_usage(owner.log_directory).free
        self.receipt['capacity'].update(copied_allocated_bytes=allocated, postcopy_free_bytes=post_free,
                                        postcopy_admitted=post_free >= owner.min_free_bytes)
        self.save()
        if post_free < owner.min_free_bytes:
            raise RuntimeError("cold static copy did not retain caller minimum free bytes")
        self.rows, self.entries, self.logical = rows, entries, logical
        self.git_state = {name: (self.repo / '.git' / name).read_bytes() for name in
                          ('HEAD', 'config', 'MERGE_HEAD', 'MERGE_MSG', 'MERGE_MODE', 'ORIG_HEAD')
                          if (self.repo / '.git' / name).exists()}
        self.observe('initial')
        return [(name, argv, self.repo if timeout is not None else cwd, timeout)
                for name, argv, cwd, timeout in self.commands]

    def verify_owned(self):
        r = self.receipt
        if (identity(self.owner.log_directory) != r['parent_identity'] or
                identity(self.root) != r['root_identity'] or
                (r['checkout_identity'] is not None and identity(self.repo) != r['checkout_identity'])):
            raise RuntimeError('cold directory ownership identity changed')

    def observe(self, name):
        self.verify_owned()
        roots = [self.repo / 'target', *[self.repo / tree / 'target' for tree in self.owner.mq.TREES]]
        unexpected = [str(path.relative_to(self.repo)) for path in roots if path.exists() or path.is_symlink()]
        observation = {'before': name, 'empty_roots': not unexpected, 'unexpected_roots': unexpected}
        self.receipt['observations'].append(observation)
        self.save()
        if unexpected:
            raise RuntimeError('cold generated target roots unexpectedly exist')

    def unchanged(self):
        if (manifest(self.repo, self.entries) != self.rows or
                self.git('ls-files', '--stage', '-z') != os.fsencode(self.logical) or
                self.git('for-each-ref') or
                {name: (self.repo / '.git' / name).read_bytes() for name in
                 ('HEAD', 'config', 'MERGE_HEAD', 'MERGE_MSG', 'MERGE_MODE', 'ORIG_HEAD')
                 if (self.repo / '.git' / name).exists()} != self.git_state):
            raise RuntimeError('tools modified cold prepared tracked inputs')

    def cleanup(self):
        if self.receipt['root_identity'] is None:
            if self.root.exists() or self.root.is_symlink():
                raise RuntimeError('unowned cold root exists')
        else:
            self.verify_owned()
            self.receipt.update(state='removing', cleanup='pending')
            self.save()
            shutil.rmtree(self.root)
            if self.root.exists() or self.root.is_symlink():
                raise RuntimeError('cold root removal not verified')
        self.receipt.update(state='removed', cleanup='verified', completion='known')
        self.save()
