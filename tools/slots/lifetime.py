#!/usr/bin/env python3
"""Local slot maintenance protocol (tools/slots/design.md, 5.1–5.4).

The flock excludes maintenance; durable holds survive loss of the supervisor.
No process absence, released flock, or report text clears a durable hold.
"""

import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import tempfile
import uuid

sys.dont_write_bytecode = True


def no_links(path):
    path = Path(path).absolute()
    for part in (*reversed(path.parents), path):
        if part.is_symlink():
            raise RuntimeError(f"symlink path: {part}")
    return path


def identity(path):
    no_links(path)
    info = path.lstat()
    return [info.st_dev, info.st_ino]


def slot_path(directory, home=None):
    """Map any cwd within a numeric slot; other checkouts need no slot lease."""
    home = Path(home or Path.home()).absolute()
    root = home / "src/fabrig/slots"
    supplied = Path(os.path.abspath(directory))
    if not supplied.is_relative_to(root):
        return None
    relative = supplied.relative_to(root)
    if not relative.parts or not relative.parts[0].isascii() or not relative.parts[0].isdecimal():
        return None
    no_links(supplied)
    slot = root / relative.parts[0]
    if not slot.is_dir():
        raise RuntimeError(f"not a slot directory: {slot}")
    return slot


def private_directory(path):
    no_links(path)
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = path.stat()
    if info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise RuntimeError(f"state directory must be owned mode 0700: {path}")


def sync(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def write(path, value):
    no_links(path)
    fd, temporary = tempfile.mkstemp(prefix=".write-", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as output:
            json.dump(value, output, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
        sync(path.parent)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def fingerprint(path):
    no_links(path)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as source:
        first = os.fstat(source.fileno())
        if not stat.S_ISREG(first.st_mode) or first.st_uid != os.getuid():
            raise RuntimeError(f"not an owned regular file: {path}")
        digest = hashlib.file_digest(source, "sha256").hexdigest()
        last = os.fstat(source.fileno())
    key = lambda s: [s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns]
    if key(first) != key(last) or key(last) != key(path.lstat()):
        raise RuntimeError(f"file changed while inspecting: {path}")
    return {"path": str(path), "identity": key(last), "sha256": digest}


def read(path):
    before = fingerprint(path)
    if stat.S_IMODE(path.stat().st_mode) != 0o600:
        raise RuntimeError(f"state file must be mode 0600: {path}")
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise RuntimeError(f"state must be a JSON object: {path}")
    if fingerprint(path) != before:
        raise RuntimeError(f"state changed: {path}")
    return value


@contextmanager
def locked(path, exclusive=True):
    no_links(path)
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        info = os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or stat.S_IMODE(info.st_mode) != 0o600 or info.st_nlink != 1):
            raise RuntimeError(f"unsafe lock: {path}")
        try:
            fcntl.flock(fd, (fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH) | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise RuntimeError(f"slot maintenance lease busy: {path}") from error
        if identity(path) != [info.st_dev, info.st_ino]:
            raise RuntimeError(f"lock replaced: {path}")
        yield fd
    finally:
        os.close(fd)


def state_root(home):
    root = Path(home) / "src/fabrig/slot-maintenance"
    private_directory(root)
    return root


class Lease:
    def __init__(self, slot, home=None):
        self.home = Path(home or Path.home())
        self.slot = slot_path(slot, self.home)
        self.fd = None
        self.clean = False

    def __enter__(self):
        if self.slot is None:
            return self
        self.directory = state_root(self.home) / self.slot.name
        private_directory(self.directory)
        self.lock = locked(self.directory / "lease", exclusive=False)
        self.fd = self.lock.__enter__()
        self.token = uuid.uuid4().hex
        self.hold = self.directory / (self.token + ".hold")
        write(self.hold, {"slot": str(self.slot), "identity": identity(self.slot),
                          "pid": os.getpid(), "purpose": "source/build lifetime"})
        # Any command using a slot invalidates the prior report acknowledgement.
        ack = self.directory / "ack.json"
        no_links(ack)
        if ack.exists():
            ack.unlink(missing_ok=True)
            sync(self.directory)
        return self

    def adopt(self, token):
        if not token or self.slot is None:
            return
        if len(token) != 32 or any(c not in "0123456789abcdef" for c in token):
            raise RuntimeError("invalid launch hold token")
        path = self.directory / (token + ".hold")
        record = read(path)
        if record["slot"] != str(self.slot) or record["identity"] != identity(self.slot):
            raise RuntimeError("launch hold slot identity mismatch")
        path.unlink()
        sync(self.directory)

    def __exit__(self, kind, value, traceback):
        if self.fd is None:
            return
        try:
            if (self.clean and kind is None and self.hold.exists()
                    and not read(self.hold).get("uncertain")):
                self.hold.unlink()
                sync(self.directory)
        finally:
            self.lock.__exit__(kind, value, traceback)


def inherited(slot):
    """Validate the inherited open lease, rather than trusting an env sentinel."""
    try:
        fd = int(os.environ["FABRIG_SLOT_LEASE_FD"])
        token = os.environ["FABRIG_SLOT_HOLD"]
        directory = state_root(Path.home()) / slot.name
        if identity(directory / "lease") != [os.fstat(fd).st_dev, os.fstat(fd).st_ino]:
            return False
        record = read(directory / (token + ".hold"))
        return record["slot"] == str(slot) and record["identity"] == identity(slot)
    except (KeyError, ValueError, OSError, RuntimeError):
        return False


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("run", "launch", "verify", "uncertain"))
    parser.add_argument("directory", type=Path)
    parser.add_argument("--adopt", default="")
    args, command = parser.parse_known_args()
    slot = slot_path(args.directory)
    if args.mode == "verify":
        return 0 if slot is None or inherited(slot) else 1
    if args.mode == "uncertain":
        if slot is None:
            return 0
        if not inherited(slot):
            raise RuntimeError("cannot mark uncertainty without an inherited lease")
        hold = state_root(Path.home()) / slot.name / (os.environ["FABRIG_SLOT_HOLD"] + ".hold")
        write(hold, {**read(hold), "uncertain": True})
        return 0
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("a foreground command is required after --")
    with Lease(args.directory) as lease:
        lease.adopt(args.adopt)
        env = dict(os.environ)
        descriptors = ()
        if lease.fd is not None:
            env.update(FABRIG_SLOT_LEASE_FD=str(lease.fd), FABRIG_SLOT_HOLD=lease.token,
                       FABRIG_SLOT_LEASE_MODE=args.mode)
            descriptors = (lease.fd,)
        # Signals leave a durable hold, including when children outlive this wrapper.
        def interrupted(number, frame):
            raise RuntimeError(f"interrupted by {number}; ownership remains uncertain")
        for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            signal.signal(number, interrupted)
        child = subprocess.Popen(command, env=env, pass_fds=descriptors,
                                 stdin=subprocess.DEVNULL)
        code = child.wait()
        lease.clean = code == 0 and args.mode != "launch"
        return code if code >= 0 else 128 - code


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError) as error:
        sys.exit(f"slot lifetime: {error}")
