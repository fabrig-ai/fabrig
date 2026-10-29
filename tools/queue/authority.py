"""Sole queue writer and concrete effects (tools/queue/design.md, 3.10).

No executable train mode or job transport. All subprocesses use argv. Tests
inject runners and use disposable origins; this module never discovers hosts.
"""

from contextlib import contextmanager
import fcntl
import importlib.util
import os
from pathlib import Path
import re
import stat
import subprocess

DIRECTORY = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location("authority_durable", DIRECTORY / "durable.py")
durable = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(durable)
OID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
ADDRESS = re.compile(r"[A-Za-z0-9][A-Za-z0-9_./-]*@[A-Za-z0-9][A-Za-z0-9.-]*\Z")


def oid(value):
    if not isinstance(value, str) or not OID.fullmatch(value):
        raise ValueError("expected a full commit OID")
    return value


class Lock:
    """Never unlink/unlock a live lock inode; close each inherited descriptor."""

    def __init__(self, path, *, inherited=None):
        self.path = Path(path)
        self.pid = os.getpid()
        self.fd = None
        fd = (os.open(path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK, 0o600)
              if inherited is None else os.dup(inherited))
        try:
            info = os.fstat(fd)
            if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or
                    stat.S_IMODE(info.st_mode) != 0o600 or self.path.is_symlink() or
                    (info.st_dev, info.st_ino) != (self.path.stat().st_dev, self.path.stat().st_ino)):
                raise ValueError("lock must be an owned private regular inode")
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise RuntimeError("another queue/worker holds this checkout or queue authority") from error
            self.fd = fd
        except BaseException:
            os.close(fd)
            raise

    def guard(self):
        if self.fd is None or self.pid != os.getpid():
            raise RuntimeError("sole authority is not held by this process")
        info, current = os.fstat(self.fd), self.path.lstat()
        if (info.st_dev, info.st_ino) != (current.st_dev, current.st_ino):
            raise RuntimeError("authority lock inode changed")

    def close(self):
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None

    def __enter__(self):
        return self

    def __exit__(self, *ignored):
        self.close()


class Layout:
    def __init__(self, path=None, *, home=None):
        # Independent of clone names: every production queue mode uses this
        # host-wide default. An override denotes an isolated queue, not a clone.
        default = (Path.home() if home is None else Path(home)) / "src/fabrig/queue-authority"
        configured = path or os.environ.get("MQ_AUTHORITY_DIR") or default
        if not Path(configured).is_absolute():
            raise ValueError("authority directory must be absolute and shared across clones")
        self.root = durable.directory(configured, create=True)
        self.train = durable.directory(self.root / "train", create=True)

    def acquire(self, *, inherited=None):
        lease = Lock(self.root / "authority.lock", inherited=inherited)
        try:
            self.queue_guard()
            return lease
        except BaseException:
            lease.close()
            raise

    def queue_guard(self):
        marker = self.root / "queue.unresolved"
        if marker.exists() or marker.is_symlink():
            raise RuntimeError("unresolved queue ownership; heavy reconciliation is unsupported")

    def restart_lock(self):
        return Lock(self.root / "restart.lock")

    def legacy_guard(self, repo):
        self.queue_guard()
        # Any live train snapshot requires explicit retirement. Even an empty
        # active list says nothing about remote job lifetime or pending effects.
        if any(p.exists() or p.is_symlink() for p in (
                self.root / "train-owner.json", self.train / "state.json")):
            raise RuntimeError("train recovery state exists; explicit reconciliation/retirement required")
        checkout_guard(repo)


def checkout_guard(repo):
    repo = Path(repo).absolute()
    if any(p.is_symlink() for p in (repo, *repo.parents)):
        raise RuntimeError("checkout path contains a symlink")
    gitdir = repo / ".git"
    if not gitdir.is_dir() or gitdir.is_symlink():
        raise RuntimeError("queue mutation requires an independent clone")
    for name in ("mq-worker-unresolved.json", "precheck.unresolved", "mq-preparation-unresolved.json", "queue.unresolved"):
        marker = gitdir / name
        if marker.exists() or marker.is_symlink():
            raise RuntimeError("unresolved checkout ownership; explicit reconciliation required: " + str(marker))
    # precheck uses its own lock, in addition to the common checkout lock.
    for path in (gitdir / "precheck.lock", repo.with_name(repo.name + ".precheck.lock")):
        if path.exists() or path.is_symlink():
            with Lock(path):
                pass


@contextmanager
def checkout_lock(repo):
    with Lock(Path(repo).with_name(Path(repo).name + ".queue.lock")) as lease:
        checkout_guard(repo)
        yield lease


class GitAuthority:
    """Effects under a held Layout authority, bound to exact origin/lead config."""

    def __init__(self, layout, lock, repo, origin, leads, *, runner=subprocess.run):
        self.layout, self.lock = layout, lock
        self.repo = Path(repo)
        self.runner = runner
        if (not isinstance(origin, str) or not origin or origin.startswith("-") or
                any(ord(c) < 32 for c in origin) or "\0" in origin):
            raise ValueError("invalid explicit origin")
        if not (Path(origin).is_absolute() or
                re.fullmatch(r"(?:ssh|https)://[A-Za-z0-9_.@:/-]+", origin) or
                re.fullmatch(r"[A-Za-z0-9_.-]+@[A-Za-z0-9_.-]+:[A-Za-z0-9_./-]+", origin)):
            raise ValueError("origin must be an absolute local path or explicit ssh/https URL")
        if not isinstance(leads, dict) or "ops" not in leads or any(
                not isinstance(k, str) or not re.fullmatch(r"[A-Za-z0-9_.-]+", k) or
                not isinstance(v, str) or not ADDRESS.fullmatch(v) for k, v in leads.items()):
            raise ValueError("invalid frozen leads mapping")
        self.origin, self.leads = origin, dict(leads)
        self.frozen = durable.encoded(self.identity())
        self.store = None
        self.checkout_lease = None
        self.failed = False
        self.guard()

    def bind(self, store, preparer, *, config):
        self.guard()
        if store.directory != self.layout.train:
            raise ValueError("production train state must use the shared authority layout")
        checkout_guard(self.repo)
        if self.repo == preparer.artifacts or preparer.artifacts in self.repo.parents:
            raise ValueError("authority clone must be separate from artifacts")
        receipt = {"version": 1, "state": str(store.path), "config": config}
        try:
            durable.publish(self.layout.root / "train-owner.json", durable.encoded(receipt), immutable=True)
        except BaseException:
            self.failed = True
            raise
        self.store = store

    def identity(self):
        return {"version": 1, "layout": str(self.layout.root), "repo": str(self.repo),
                "origin": self.origin, "leads": dict(self.leads)}

    def guard(self):
        if self.failed:
            raise RuntimeError("authority publication failed; reload/reconcile required")
        self.lock.guard()
        if durable.encoded(self.identity()) != self.frozen:
            raise RuntimeError("authority configuration changed")

    def git(self, *args, check=True):
        self.guard()
        result = self.runner(["git", *args], cwd=self.repo, stdin=subprocess.DEVNULL,
                             text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             pass_fds=(self.lock.fd, *((self.checkout_lease.fd,) if self.checkout_lease else ())))
        if check and result.returncode:
            raise RuntimeError("Git effect failed: " + result.stderr.strip())
        return result

    def remote(self, ref):
        result = self.git("ls-remote", "--refs", "--", self.origin, ref)
        rows = result.stdout.splitlines()
        if not rows:
            return None
        if len(rows) != 1 or rows[0].split()[1:] != [ref]:
            raise ValueError("invalid authoritative ref response")
        return oid(rows[0].split()[0])

    def main(self):
        value = self.remote("refs/heads/main")
        if value is None:
            raise RuntimeError("authoritative main is absent")
        return value

    def promote(self, parent, candidate, event_id):
        oid(parent)
        oid(candidate)
        if not isinstance(event_id, str) or not event_id:
            raise ValueError("promotion requires durable intent id")
        if self.store is None:
            raise RuntimeError("promotion requires a bound durable controller intent")
        state = self.store.load()
        if state is None or not state["active"]:
            raise ValueError("promotion has no durable head intent")
        record = state["records"][state["active"][0]]
        if (record.get("promotion") != {"id": event_id, "parent": parent, "candidate": candidate} or
                record["phase"] != "promotion-intent" or record["outcome"] != "green" or
                record.get("invalidated") or state["config"]["authority"] != self.identity()):
            raise ValueError("promotion differs from durable certified head intent")
        if self.main() != parent:
            raise RuntimeError("fresh main differs from certified parent")
        with checkout_lock(self.repo) as checkout:
            self.checkout_lease = checkout
            try:
                if self.git("cat-file", "-e", candidate + "^{commit}", check=False).returncode:
                    bundle = record["bundle"]
                    path = Path(bundle["path"])
                    if not path.is_absolute() or path != path.resolve():
                        raise ValueError("invalid certified bundle path")
                    durable.checked(path, bundle)
                    heads = self.git("bundle", "list-heads", str(path)).stdout.splitlines()
                    if {ref: value for value, ref in (line.split() for line in heads)} != bundle["refs"]:
                        raise ValueError("certified bundle refs changed")
                    self.git("bundle", "verify", str(path))
                    self.git("bundle", "unbundle", str(path))
                for value in (parent, candidate):
                    if self.git("cat-file", "-t", value).stdout.strip() != "commit":
                        raise ValueError("certified identity is not a commit")
                self.git("merge-base", "--is-ancestor", parent, candidate)
                if self.main() != parent:
                    raise RuntimeError("fresh main differs from certified parent")
                # Import only the original objects; never reconstruct certified C.
                self.git("push", "--", self.origin, candidate + ":refs/heads/main")
            finally:
                self.checkout_lease = None

    def cleanup(self, name, tip, event_id):
        oid(tip)
        if not isinstance(name, str) or len(name.split("/")) != 3 or not name.startswith("ready/"):
            raise ValueError("cleanup requires a ready branch")
        ref = "refs/heads/" + name
        if self.git("check-ref-format", ref, check=False).returncode:
            raise ValueError("invalid ready ref")
        result = self.git("push", "--force-with-lease=" + ref + ":" + tip,
                          "--", self.origin, ":" + ref, check=False)
        if result.returncode == 0:
            return True
        # Failure to read remains pending. A different tip/absence is harmless.
        return self.remote(ref) != tip

    def post(self, event):
        self.guard()
        identity, body = event["id"], event["body"]
        if (not isinstance(identity, str) or "\n" in identity or
                "Event: " + identity + "\n" not in body or "\nEvidence: " not in body):
            raise ValueError("message requires stable event id and durable evidence location")
        kind = event["kind"]
        if kind in ("landed", "rejected"):
            area = event["branch"]["name"].split("/")[1]
            if area not in self.leads:
                raise ValueError("ready branch has no frozen lead route")
        elif kind in ("infrastructure", "selection-gap"):
            area = "ops"
        else:
            raise ValueError("unsupported notification kind")
        result = self.runner(["post", "send", "--as", "queue", self.leads[area],
                              "mq " + kind + " " + identity], cwd=self.repo,
                             input=body, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             pass_fds=(self.lock.fd,))
        return result.returncode
