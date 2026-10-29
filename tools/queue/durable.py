"""Private snapshot/evidence publication (tools/queue/design.md, 3.10)."""

import hashlib
import fcntl
import json
import os
import re
from pathlib import Path
import stat
import tempfile


def encoded(value):
    return (json.dumps(value, sort_keys=True, allow_nan=False, separators=(",", ":")) + "\n").encode()


def read_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate JSON key")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs, parse_constant=lambda _: (
        _ for _ in ()).throw(ValueError("nonfinite JSON")))


def directory(path, *, create=False, private=True):
    path = Path(os.path.abspath(path))
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError("configured directory contains a symlink")
    if create and not path.exists():
        path.mkdir(mode=0o700)  # The configured parent must already exist.
        sync_directory(path.parent)
    info = path.stat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or
            private and stat.S_IMODE(info.st_mode) != 0o700):
        raise ValueError("directory must be owned and mode 0700")
    return path


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def read_bytes(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as source:
        info = os.fstat(source.fileno())
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or
                stat.S_IMODE(info.st_mode) != 0o600):
            raise ValueError("artifact must be an owned private regular file")
        return source.read()


def publish(path, data, *, immutable=False, private_parent=True):
    path = Path(path)
    directory(path.parent, private=private_parent)
    if not immutable and (path.exists() or path.is_symlink()):
        read_bytes(path)  # Refuse unsafe publication targets before replacement.
    if immutable and (path.exists() or path.is_symlink()):
        if read_bytes(path) != data:
            raise ValueError("immutable evidence already exists with different bytes")
        sync_directory(path.parent)
        return
    fd, temporary = tempfile.mkstemp(prefix=".publish-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as output:
            os.fchmod(output.fileno(), 0o600)
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        if immutable:
            # Serialize the check and rename on the stable directory inode.
            # An immutable publisher never replaces a conflicting identity.
            lock_fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                if path.exists() or path.is_symlink():
                    if read_bytes(path) != data:
                        raise ValueError("immutable evidence publication conflict")
                else:
                    os.replace(temporary, path)
                os.fsync(lock_fd)
            finally:
                os.close(lock_fd)
        else:
            os.replace(temporary, path)
        sync_directory(path.parent)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
            sync_directory(path.parent)


def atomic_json(path, value, *, private_parent=True):
    publish(path, encoded(value), private_parent=private_parent)


def fingerprint(data):
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def checked(path, identity):
    if (not isinstance(identity, dict) or type(identity.get("bytes")) is not int or identity["bytes"] < 0 or
            not isinstance(identity.get("sha256"), str) or not re.fullmatch(r"[0-9a-f]{64}", identity["sha256"])):
        raise ValueError("invalid artifact identity")
    data = read_bytes(path)
    if fingerprint(data) != {k: identity[k] for k in ("bytes", "sha256")}:
        raise ValueError("corrupt artifact: " + str(path))
    return data
