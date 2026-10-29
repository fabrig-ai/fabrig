"""Bounded, exact binary artifact envelope; design.md, 3.11."""
import importlib.util
import os
from pathlib import Path
import selectors
import struct
import subprocess
import time

SPEC = importlib.util.spec_from_file_location('wire_durable', Path(__file__).with_name('durable.py'))
durable = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(durable)
MAGIC = b'MQ-WIRE-1\n'
JSON_LIMIT = 8 * 1024 * 1024
BUNDLE_LIMIT = 512 * 1024 * 1024
LOG_LIMIT = 128 * 1024 * 1024
FRAME_LIMIT = BUNDLE_LIMIT + LOG_LIMIT + JSON_LIMIT * 2


def pack(value, blobs=()):
    blobs = tuple(blobs)
    header = durable.encoded({'value': value, 'blobs': [durable.fingerprint(b) for b in blobs]})
    if len(header) > JSON_LIMIT or sum(map(len, blobs)) > FRAME_LIMIT:
        raise ValueError('artifact envelope exceeds protocol limit')
    return MAGIC + struct.pack('!Q', len(header)) + header + b''.join(blobs)


def unpack(data):
    prefix = len(MAGIC) + 8
    if len(data) < prefix or data[:len(MAGIC)] != MAGIC or len(data) > FRAME_LIMIT + JSON_LIMIT:
        raise ValueError('contaminated, truncated or oversized artifact channel')
    length, = struct.unpack('!Q', data[len(MAGIC):prefix])
    if length > JSON_LIMIT or length > len(data) - prefix:
        raise ValueError('invalid envelope header length')
    header = durable.read_json(data[prefix:prefix + length])
    if set(header) != {'value', 'blobs'} or not isinstance(header['blobs'], list):
        raise ValueError('invalid artifact envelope')
    offset, blobs = prefix + length, []
    for identity in header['blobs']:
        size = identity['bytes']
        if type(size) is not int or not 0 <= size <= FRAME_LIMIT:
            raise ValueError('invalid artifact length')
        blob = data[offset:offset + size]
        if durable.fingerprint(blob) != identity:
            raise ValueError('truncated or corrupt artifact')
        blobs.append(blob)
        offset += size
    if offset != len(data):
        raise ValueError('trailing artifact channel contamination')
    return header['value'], blobs


def exchange(argv, data, *, timeout, pass_fds=(), env=None, raw=False):
    """Bound both pipes while communicating; killing this client grants no release."""
    with subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, pass_fds=pass_fds, env=env) as child:
        chunks = {child.stdout: bytearray(), child.stderr: bytearray()}
        remaining = memoryview(data)
        deadline = time.monotonic() + timeout
        with selectors.DefaultSelector() as selector:
            for stream in (child.stdin, child.stdout, child.stderr):
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_WRITE if stream == child.stdin else selectors.EVENT_READ)
            try:
                while selector.get_map():
                    left = deadline - time.monotonic()
                    if left <= 0:
                        raise TimeoutError('endpoint reply deadline; job lifetime remains unknown')
                    for key, _ in selector.select(left):
                        stream = key.fileobj
                        if stream == child.stdin:
                            if remaining:
                                remaining = remaining[os.write(stream.fileno(), remaining[:65536]):]
                            if not remaining:
                                selector.unregister(stream)
                                stream.close()
                        else:
                            chunk = os.read(stream.fileno(), 65536)
                            if not chunk:
                                selector.unregister(stream)
                            else:
                                chunks[stream].extend(chunk)
                                limit = JSON_LIMIT if stream == child.stderr else FRAME_LIMIT + JSON_LIMIT
                                if len(chunks[stream]) > limit:
                                    raise ValueError('endpoint channel exceeds bounded envelope')
                child.wait(timeout=max(0.001, deadline - time.monotonic()))
            except BaseException:
                child.kill()
                child.wait()
                raise
        if child.returncode:
            raise RuntimeError('endpoint refused: ' + bytes(chunks[child.stderr]).decode(errors='replace'))
        output = bytes(chunks[child.stdout])
        return output if raw else unpack(output)
