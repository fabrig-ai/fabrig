"""Disposable transport worlds and inotify barriers; no automatic test cases."""
import ctypes
import importlib.util
import os
from pathlib import Path
import select
import shutil
import sys

sys.dont_write_bytecode = True
DIRECTORY = Path(__file__).resolve().parent

def module(name):
    spec = importlib.util.spec_from_file_location('transport_fixture_' + name, DIRECTORY / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

fixtures, supervisor, operation = module('train_fixture'), module('supervisor'), module('operation')
train, worker, mq = fixtures.train, fixtures.worker, fixtures.mq
transport, durable = supervisor.transport, supervisor.durable


def wait_file(path):
    libc = ctypes.CDLL(None, use_errno=True)
    fd = libc.inotify_init1(os.O_CLOEXEC | os.O_NONBLOCK)
    if fd < 0:
        raise OSError(ctypes.get_errno(), 'inotify_init1')
    try:
        if libc.inotify_add_watch(fd, os.fsencode(path.parent), 0x100 | 0x80 | 0x8) < 0:
            raise OSError(ctypes.get_errno(), 'inotify_add_watch')
        while not path.exists():
            if not select.select([fd], [], [], 10)[0]:
                raise AssertionError('no publication: ' + str(path))
            os.read(fd, 65536)
    finally:
        os.close(fd)


class EndpointFixture(fixtures.TrainFixture):
    def setUp(self):
        super().setUp()
        self.runtime = self.root / "runtime ; $ literal ' path"
        self.runtime.mkdir(mode=0o700)
        for name in worker.RUNTIME_FILES:
            shutil.copyfile(DIRECTORY / name, self.runtime / name)
        self.endpoints, self.configs = {}, {}
        for gate, repo in self.workers.items():
            jobs = self.root / (gate + ' jobs')
            jobs.mkdir(mode=0o700)
            heavy = DIRECTORY.parent / 'heavy/heavy'
            config = {'version': 1, 'gate': gate, 'incarnation': gate + '-incarnation',
                'repo': str(repo), 'artifacts': str(self.worker_artifacts[gate]), 'jobs': str(jobs),
                'runtime': str(self.runtime), 'disk_floor': 0, 'heavy': str(heavy),
                'heavy_sha256': supervisor.heavy_identity(heavy),
                'heavy_root': str(self.root / 'heavy-root'), 'heavy_tmp': str(self.root / 'heavy-tmp')}
            path = self.root / (gate + " endpoint ' ; literal.json")
            durable.publish(path, durable.encoded(config), immutable=True)
            self.configs[gate] = config
            self.endpoints[gate] = {'host': 'local', 'config': str(path), 'endpoint': config,
                'config_sha256': worker.digest(durable.encoded(config)),
                'known_hosts': None, 'known_hosts_sha256': None}
        self.real_transport = transport.Transport(self.endpoints, timeout=10)

    def request(self, *, tools=None):
        self.branch('literal', {'tools/test_fixture.py': tools} if tools is not None else None)
        attempt, record = self.admit()
        return record['request'], record['bundle']

    def job(self, request):
        return Path(self.configs[request['gate']]['jobs']) / request['attempt']

    def finish_real(self, request):
        wait_file(self.job(request) / 'completion.json')
        return self.real_transport.status(request)
