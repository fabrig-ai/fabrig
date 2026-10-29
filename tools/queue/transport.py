"""Local argv / pinned SSH endpoint transport; design.md, 3.11."""
import importlib.util
from pathlib import Path
import re
import shlex
import sys
import time

sys.dont_write_bytecode = True
DIRECTORY = Path(__file__).resolve().parent

def module(name):
    spec = importlib.util.spec_from_file_location('transport_' + name, DIRECTORY / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

wire = module('wire')
durable = wire.durable


def absolute(value):
    if (not isinstance(value, str) or not Path(value).is_absolute() or
            str(Path(value)) != value or '..' in Path(value).parts or
            any(ord(c) < 32 for c in value)):
        raise ValueError('expected normalized absolute configured path')
    return value


def endpoint_config(value):
    keys = {'version', 'gate', 'incarnation', 'repo', 'artifacts', 'jobs', 'runtime',
            'disk_floor', 'heavy', 'heavy_sha256', 'heavy_root', 'heavy_tmp'}
    if set(value) != keys or type(value['version']) is not int or value['version'] != 1:
        raise ValueError('unknown endpoint configuration/version')
    for key in ('gate', 'incarnation'):
        if not isinstance(value[key], str) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]{0,127}', value[key]):
            raise ValueError('invalid endpoint token')
    for key in ('repo', 'artifacts', 'jobs', 'runtime', 'heavy', 'heavy_root', 'heavy_tmp'):
        absolute(value[key])
    paths = [Path(value[k]) for k in ('repo', 'artifacts', 'jobs', 'runtime')]
    if any(a == b or a in b.parents or b in a.parents for i, a in enumerate(paths) for b in paths[i+1:]):
        raise ValueError('endpoint runtime, checkout and private stores must be disjoint')
    if type(value['disk_floor']) is not int or value['disk_floor'] < 0:
        raise ValueError('invalid endpoint disk floor')
    if not re.fullmatch('[a-f0-9]{64}', value['heavy_sha256']):
        raise ValueError('invalid frozen heavy identity')
    return value


def remaining(endpoint, timeout=None):
    deadline = getattr(endpoint, 'deadline', None)
    if deadline is None:
        return timeout
    left = deadline - time.monotonic()
    if left <= 0:
        raise TimeoutError('train drain deadline exhausted; stop retained')
    return left if timeout is None else min(left, timeout)


class Transport:
    def __init__(self, endpoints, *, timeout=60, exchange=wire.exchange):
        self.endpoints, self.timeout, self.exchange = endpoints, timeout, exchange
        self.deadline = None  # Transient enclosing drain bound, never configuration.
        if type(timeout) not in (int, float) or not 0 < timeout <= 1800:
            raise ValueError('explicit bounded endpoint timeout required')
        for gate, endpoint in endpoints.items():
            if set(endpoint) != {'host', 'config', 'config_sha256', 'endpoint', 'known_hosts', 'known_hosts_sha256'}:
                raise ValueError('invalid transport endpoint')
            config = endpoint_config(endpoint['endpoint'])
            if gate != config['gate'] or durable.fingerprint(durable.encoded(config))['sha256'] != endpoint['config_sha256']:
                raise ValueError('endpoint configuration identity mismatch')
            absolute(endpoint['config'])
            host = endpoint['host']
            if host != 'local':
                if not isinstance(host, str) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', host):
                    raise ValueError('SSH host must be an explicit alias without shell syntax')
                absolute(endpoint['known_hosts'])
                if durable.fingerprint(durable.read_bytes(endpoint['known_hosts']))['sha256'] != endpoint['known_hosts_sha256']:
                    raise ValueError('known-hosts identity mismatch')
            elif endpoint['known_hosts'] is not None or endpoint['known_hosts_sha256'] is not None:
                raise ValueError('local endpoint has no SSH known-hosts file')
        self.frozen = durable.encoded(self.identity())

    def identity(self):
        return {gate: {'repo': e['endpoint']['repo'], 'artifacts': e['endpoint']['artifacts'],
                       'host': e['host'], 'endpoint': e, 'rpc_timeout': self.timeout}
                for gate, e in self.endpoints.items()}

    def argv(self, gate, action):
        endpoint = self.endpoints[gate]
        argv = ['python3', '-I', '-B', str(Path(endpoint['endpoint']['runtime']) / 'supervisor.py'),
                '--config', endpoint['config'], '--config-sha256', endpoint['config_sha256'], action]
        if endpoint['host'] == 'local':
            return argv
        # OpenSSH joins its trailing argv with spaces. Supply ONE command string,
        # quoting bash's single login-shell program once more for the remote shell.
        return ['ssh', '-T', '-oBatchMode=yes', '-oStrictHostKeyChecking=yes',
                '-oUserKnownHostsFile=' + endpoint['known_hosts'], '--', endpoint['host'],
                'bash -lc ' + shlex.quote('exec ' + shlex.join(argv))]

    def rpc(self, request, action, bundle=None, *, timeout=None):
        if durable.encoded(self.identity()) != self.frozen:
            raise ValueError('transport identity changed')
        endpoint = self.endpoints[request['gate']]
        if endpoint['host'] != 'local' and durable.fingerprint(durable.read_bytes(
                endpoint['known_hosts']))['sha256'] != endpoint['known_hosts_sha256']:
            raise ValueError('known-hosts changed during generation')
        blobs = ()
        if bundle is not None:
            blob = durable.checked(bundle['path'], bundle)
            if len(blob) > wire.BUNDLE_LIMIT:
                raise ValueError('bundle exceeds channel limit')
            blobs = (blob,)
        budget = remaining(self, self.timeout if timeout is None else min(timeout, self.timeout))
        value, returned = self.exchange(self.argv(request['gate'], action),
            wire.pack({'version': 1, 'request': request, 'bundle': bundle}, blobs),
            timeout=budget)
        if value.get('request') != request or value.get('endpoint_sha256') != endpoint['config_sha256']:
            raise ValueError('endpoint replied for another request/incarnation/config')
        if value.get('state') == 'terminal':
            if len(returned) != 3 or len(returned[0]) > wire.JSON_LIMIT or len(returned[1]) > wire.LOG_LIMIT:
                raise ValueError('missing/oversized result or log')
            value.update(result=returned[0], log=returned[1], supervision=returned[2])
        elif value.get('state') == 'reconciled':
            if len(returned) != 1:
                raise ValueError('missing reconciliation receipt')
            value['supervision'] = returned[0]
        elif returned:
            raise ValueError('unexpected artifacts')
        if 'supervision' in value:
            if (len(value['supervision']) > wire.BUNDLE_LIMIT or
                    durable.fingerprint(value['supervision']) != value.get('evidence', {}).get('receipt')):
                raise ValueError('supervision receipt channel corrupted')
        return value

    def start(self, request, bundle):
        return self.rpc(request, 'start', bundle)

    def status(self, request):
        return self.rpc(request, 'status')

    def completion_identity(self):
        return {'kind': 'subreaper-heavy-v1', 'transport': self.identity()}

    def verify_completion(self, request, evidence, endpoint):
        # Re-fetch authoritative receipt; do not accept caller-written proof.
        status = self.status(request)
        if (status.get('state') not in ('terminal', 'reconciled') or
                status.get('evidence') != evidence or status.get('quiescent') is not True):
            raise ValueError('supervisor completion not established for exact job')
        return evidence
