#!/usr/bin/env python3
"""Durable gate endpoint and Linux subreaper; design.md, 3.11.

No origin effects. Receipt publication follows child reap, supported heavy
release queries, immutable artifacts and final checkout verification.
"""
import argparse
import ctypes
import importlib.util
import os
from pathlib import Path
import subprocess
import signal
import sys
import uuid

sys.dont_write_bytecode = True
DIRECTORY = Path(__file__).resolve().parent

def module(name):
    spec = importlib.util.spec_from_file_location('supervisor_' + name, DIRECTORY / (name + '.py'))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

transport = module('transport')
train = module('train')
worker, durable, authority, wire = train.worker, train.durable, train.authority_module, transport.wire
MARKER = 'mq-transport-unresolved.json'


def boot():
    return Path('/proc/sys/kernel/random/boot_id').read_text().strip()


def process_identity():
    # Positive identity only. Absence never permits ownership release.
    return {'boot': boot(), 'pid': os.getpid(), 'start': Path('/proc/self/stat').read_text().rsplit(')', 1)[1].split()[19]}


def running(identity):
    try:
        fields = Path('/proc/' + str(identity['pid']) + '/stat').read_text().rsplit(')', 1)[1].split()
        return identity['boot'] == boot() and fields[0] != 'Z' and fields[19] == identity['start']
    except (OSError, KeyError):
        return False


def subreaper():
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0):
        raise RuntimeError('Linux child subreaper unavailable')


def reap():
    # ECHILD proves this subreaper has no remaining fork descendants. A terminal
    # direct child alone does not. Heavy has its separate containment proof.
    while True:
        try:
            os.waitpid(-1, 0)
        except ChildProcessError:
            return


def load_config(path, expected):
    raw = durable.read_bytes(path)
    config = transport.endpoint_config(durable.read_json(raw))
    if durable.fingerprint(durable.encoded(config))['sha256'] != expected:
        raise ValueError('endpoint config identity changed')
    if Path(config['runtime']) != DIRECTORY:
        raise ValueError('endpoint must execute its configured frozen runtime')
    for key in ('artifacts', 'jobs', 'runtime'):
        durable.directory(config[key])
    worker.canonical_directory(config['repo'])
    # Hash the complete heavy implementation, not just its Bash launcher.
    if heavy_identity(config['heavy']) != config['heavy_sha256']:
        raise ValueError('configured heavy runtime changed')
    return config


def heavy_identity(path):
    kit = Path(path).parent
    names = ('heavy', 'heavy-recovery.py', 'kit.sh', 'gate-policy.sh', 'receipt_inputs.py', 'recovery_message.py')
    return worker.digest(durable.encoded({name: worker.digest((kit / name).read_bytes()) for name in names}))


def heavy_query(config, label):
    env = dict(os.environ, DEV_ROOT=config['heavy_root'], DEV_TMP=config['heavy_tmp'])
    # This is heavy's supported read-only API. Queue code never opens its state.
    output = wire.exchange([config['heavy'], '--completion-query', label], b'',
                           timeout=60, env=env, raw=True)
    if len(output) > wire.LOG_LIMIT:
        raise RuntimeError('heavy release query exceeds bounded channel')
    return durable.read_json(output)


def check_heavy(query, intent, *, allow_absent=False):
    if (not isinstance(query, dict) or query.get('schema') != 'heavy-completion-query-v1' or
            query.get('label') != intent['label'] or not isinstance(query.get('queue'), str)):
        raise ValueError('heavy query identity mismatch')
    matches = query.get('matches')
    if not isinstance(matches, list) or len(matches) != 1:
        if allow_absent and matches == []:
            return
        raise ValueError('heavy admission identity unresolved or reused; ownership retained')
    task = matches[0]
    expected = {'label': intent['label'], 'repo': intent['cwd'], 'args': intent['payload'],
                'commit': intent['candidate'], 'queue': query['queue'], 'state': 'released'}
    if any(task.get(k) != v for k, v in expected.items()):
        raise ValueError('heavy task does not match command/candidate or is not released')
    receipt = task.get('receipt', {})
    if (receipt.get('schema') != 'heavy-receipt-v1' or
            any(receipt.get(k) != task.get(k) for k in ('token', 'id', 'queue', 'args', 'tree')) or
            any(receipt.get(k) is not True for k in ('release_complete', 'cgroup_empty', 'row_durable'))):
        raise ValueError('heavy release/containment receipt incomplete')
    import base64
    log = base64.b64decode(task['log_base64'], validate=True)
    if worker.digest(log) != receipt['log_sha256']:
        raise ValueError('heavy sealed log corrupted')


def heavy_payload(argv):
    args = list(argv[1:])
    while args and args[0] in ('--gate', '--land', '--tail', '--'):
        flag = args.pop(0)
        if flag == '--tail':
            args.pop(0)
        if flag == '--':
            break
    return args


class Commands:
    def __init__(self, config, claim, job, *, query=heavy_query):
        self.config, self.claim, self.job, self.query = config, claim, job, query
        self.sequence = []

    def run(self, argv, **kwargs):
        index = len(self.sequence)
        heavy = argv[0] == 'heavy'
        label = 'mq-' + self.claim['job'] + '-' + str(index)
        intent = {'index': index, 'command': list(argv), 'cwd': str(kwargs['cwd']),
                  'candidate': self.claim['request']['candidate'], 'heavy': heavy,
                  'label': label, 'payload': heavy_payload(argv) if heavy else None}
        path = self.job / ('command-' + str(index))
        durable.publish(path.with_suffix('.intent'), durable.encoded(intent), immutable=True)
        self.sequence.append(intent)
        env = dict(os.environ)
        if heavy:
            argv = [self.config['heavy'], *argv[1:]]
            env.update(HEAVY_LABEL=label, DEV_ROOT=self.config['heavy_root'], DEV_TMP=self.config['heavy_tmp'])
        timeout = kwargs.pop('timeout', None)
        timed_out = False
        try:
            with subprocess.Popen(argv, env=env, start_new_session=True, **kwargs) as child:
                try:
                    code = child.wait(timeout=timeout)
                except subprocess.TimeoutExpired:
                    # The tools timeout remains red after the entire owned group
                    # and any adopted descendants have been proved ended.
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                    timed_out, code = True, 124
                result = subprocess.CompletedProcess(argv, code)
        except BaseException:
            raise worker.UncertainCompletion('command interrupted; supervisor reconciliation required')
        reap()
        evidence = None
        if heavy:
            evidence = self.query(self.config, label)
            check_heavy(evidence, intent)
            # Captures the admitted token, immutable receipt and sealed log before
            # the worker can publish completion or start its next command.
            durable.publish(path.with_suffix('.heavy'), durable.encoded(evidence), immutable=True)
            if evidence['matches'][0]['receipt']['code'] != result.returncode:
                raise worker.UncertainCompletion('heavy client exit differs from released task')
        durable.publish(path.with_suffix('.done'), durable.encoded({'intent': intent,
            'returncode': result.returncode, 'heavy': evidence}), immutable=True)
        if timed_out:
            raise subprocess.TimeoutExpired(intent['command'], timeout)
        return result

    def evidence(self):
        return [{'intent': durable.read_json(durable.read_bytes(self.job / ('command-' + str(i) + '.intent'))),
                 'done': durable.read_json(durable.read_bytes(self.job / ('command-' + str(i) + '.done')))}
                for i in range(len(self.sequence))]


def claim_for(config, request, bundle):
    worker.validate_request(request)
    if request['gate'] != config['gate'] or request['runtime'] != worker.runtime_identity(config['disk_floor']):
        raise ValueError('request gate/runtime mismatch')
    return {'version': 1, 'job': worker.digest(worker.encoded(request)), 'request': request,
            'endpoint': config, 'bundle': {k: bundle[k] for k in ('bytes', 'sha256', 'refs')},
            'incarnation': config['incarnation']}


def read_claim(config, request):
    job = Path(config['jobs']) / request['attempt']
    claim = durable.read_json(durable.read_bytes(job / 'claim.json'))
    if claim['request'] != request or claim['endpoint'] != config or claim['job'] != worker.digest(worker.encoded(request)):
        raise ValueError('job incarnation/config/request mismatch')
    durable.checked(job / 'candidate.bundle', claim['bundle'])
    return job, claim


def accept(config, request, bundle, data, *, launch=None):
    expected = claim_for(config, request, bundle)
    if len(data) > wire.BUNDLE_LIMIT or durable.fingerprint(data) != {k: bundle[k] for k in ('bytes', 'sha256')}:
        raise ValueError('bundle digest/length mismatch')
    jobs, repo = Path(config['jobs']), Path(config['repo'])
    with authority.Lock(jobs / 'endpoint.lock'):
        job = jobs / request['attempt']
        if job.exists():
            _, existing = read_claim(config, request)
            if any(existing[k] != v for k, v in expected.items()):
                raise ValueError('attempt identity reused with different inputs')
            return  # No second launch, even if the first acknowledgement was lost.
        with authority.checkout_lock(repo) as checkout:
            if (jobs / 'owner.json').exists():
                raise RuntimeError('endpoint has unresolved owner; status/reconcile it first')
            durable.directory(job, create=True)
            durable.publish(job / 'candidate.bundle', data, immutable=True)
            durable.publish(job / 'request.json', durable.encoded(request), immutable=True)
            claim = dict(expected, boot=boot(), nonce=uuid.uuid4().hex)
            durable.publish(job / 'claim.json', durable.encoded(claim), immutable=True)
            durable.publish(jobs / 'owner.json', durable.encoded(claim), immutable=True)
            durable.publish(repo / '.git' / MARKER, durable.encoded(claim), immutable=True, private_parent=False)
            # A crash from here is UNKNOWN; the same attempt is never rerun.
            durable.publish(job / 'launch-intent.json', durable.encoded(claim), immutable=True)
            if launch:
                launch(job, claim)
            else:
                with (job / 'supervisor.log').open('xb') as log:
                    subprocess.Popen(['python3', '-I', '-B', str(DIRECTORY / 'supervisor.py'),
                        '--config', CONFIG_PATH, '--config-sha256', CONFIG_SHA, 'supervise',
                        '--attempt', request['attempt'], '--checkout-fd', str(checkout.fd)], stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                        start_new_session=True, pass_fds=(checkout.fd,))


def proof_path(job):
    recovery = job / 'reconciliation.json'
    return recovery if recovery.exists() else job / 'completion.json'


def verify_receipt(config, job, claim):
    proof = durable.read_json(durable.read_bytes(proof_path(job)))
    if proof.get('claim') != claim or proof.get('kind') != 'subreaper-heavy-v1':
        raise ValueError('supervisor receipt identity mismatch')
    if proof.get('reconciled'):
        if claim['boot'] == proof.get('boot'):
            raise ValueError('reconciliation needs verified new host boot')
    else:
        if proof.get('boot') != claim['boot'] or proof.get('reaped') is not True:
            raise ValueError('supervisor lifetime not resolved')
        result = durable.read_json(durable.checked(job / 'result.json', proof['result']))
        durable.checked(job / 'checks.log', proof['log'])
        if result['requested'] != claim['request'] or result['completion_known'] is not True:
            raise ValueError('terminal worker request/completion mismatch')
        observed = [e for e in result['events'] if 'command' in e]
        if len(observed) != len(proof['commands']):
            raise ValueError('supervisor command sequence differs from terminal checks')
    intents = sorted(job.glob('command-*.intent'), key=lambda p: int(p.stem.split('-')[1]))
    if len(intents) != len(proof['commands']):
        raise ValueError('supervisor command journal incomplete')
    for index, item in enumerate(proof['commands']):
        if item['intent'] != durable.read_json(durable.read_bytes(intents[index])):
            raise ValueError('supervisor command intent changed')
        if not proof.get('reconciled'):
            actual = durable.read_json(durable.read_bytes(job / ('command-' + str(index) + '.done')))
            if actual != item['done'] or actual['intent'] != item['intent']:
                raise ValueError('supervisor completion journal changed')
            expected = worker.command_record(Path(config['repo']), observed[index]['name'],
                                              item['intent']['command'], Path(item['intent']['cwd']))
            if (any(observed[index].get(k) != v for k, v in expected.items()) or
                    observed[index]['returncode'] != actual['returncode']):
                raise ValueError('supervisor command/result sequence mismatch')
        intent = item['intent']
        if intent['heavy']:
            query = heavy_query(config, intent['label'])
            check_heavy(query, intent, allow_absent=proof.get('reconciled', False))
            if item['done'] is not None and query != item['done']['heavy']:
                raise ValueError('heavy completion identity changed')
        elif not proof.get('reconciled') and item['done'] is None:
            raise ValueError('command has no completion journal')
    return proof


def release(config, job, claim):
    proof = verify_receipt(config, job, claim)
    repo, jobs = Path(config['repo']), Path(config['jobs'])
    # Receipt, never a released lock, authorizes removal of this exact marker.
    released = job / 'released.json'
    if released.exists():
        if durable.read_json(durable.read_bytes(released)) != durable.fingerprint(durable.read_bytes(proof_path(job))):
            raise ValueError('release audit mismatch')
        if not (jobs / 'owner.json').exists() or durable.read_json(durable.read_bytes(jobs / 'owner.json')) != claim:
            return proof
    with authority.Lock(jobs / 'endpoint.lock'), authority.Lock(repo.with_name(repo.name + '.queue.lock')):
        marker = repo / '.git' / MARKER
        if marker.exists():
            if durable.read_json(durable.read_bytes(marker)) != claim:
                raise ValueError('checkout marker belongs to another incarnation')
            if not proof.get('reconciled') and worker.snapshot(repo) != proof['snapshot']:
                raise ValueError('checkout changed after terminal receipt')
            if (repo / '.git/mq-worker-unresolved.json').exists():
                if not proof.get('reconciled'):
                    raise ValueError('worker ownership still unresolved')
                marker_worker = repo / '.git/mq-worker-unresolved.json'
                identity = durable.read_json(durable.read_bytes(marker_worker))
                if any(identity[k] != claim['request'][k] for k in ('attempt', 'generation', 'candidate')):
                    raise ValueError('worker marker belongs to another job')
                marker_worker.unlink()
            marker.unlink()
            durable.sync_directory(marker.parent)
        durable.publish(released, durable.encoded(durable.fingerprint(durable.read_bytes(proof_path(job)))), immutable=True)
        owner = jobs / 'owner.json'
        if owner.exists() and durable.read_json(durable.read_bytes(owner)) == claim:
            owner.unlink()
            durable.sync_directory(jobs)
    return proof


def status(config, request):
    job, claim = read_claim(config, request)
    reply = {'request': request, 'endpoint_sha256': worker.digest(durable.encoded(config))}
    if proof_path(job).exists():
        proof = release(config, job, claim)
        evidence = {'kind': proof['kind'], 'job': claim['job'], 'nonce': claim['nonce'],
                    'incarnation': claim['incarnation'], 'receipt': durable.fingerprint(durable.read_bytes(proof_path(job)))}
        reply.update(state='reconciled' if proof.get('reconciled') else 'terminal',
                     quiescent=True, evidence=evidence)
        return reply, ([durable.read_bytes(proof_path(job))] if proof.get('reconciled') else [durable.read_bytes(job / 'result.json'), durable.read_bytes(job / 'checks.log'), durable.read_bytes(proof_path(job))])
    started = job / 'started.json'
    alive = started.exists() and running(durable.read_json(durable.read_bytes(started)))
    reply.update(state='running' if alive else 'unknown',
                 reconciliation='inspect supervisor log; retry status; if supervisor lost, supported heavy recovery and explicit reconcile-reboot are required')
    return reply, []


def supervise(config, attempt, *, inherited=None):
    job = Path(config['jobs']) / attempt
    request = durable.read_json(durable.read_bytes(job / 'request.json'))
    job, claim = read_claim(config, request)
    repo = Path(config['repo'])
    with authority.Lock(job / 'supervisor.lock'), authority.Lock(repo.with_name(repo.name + '.queue.lock'), inherited=inherited) as checkout:
        if inherited is not None:
            os.close(inherited)
        if durable.read_json(durable.read_bytes(repo / '.git' / MARKER)) != claim:
            raise ValueError('supervisor checkout marker mismatch')
        if claim['boot'] != boot() or (job / 'started.json').exists():
            raise ValueError('supervisor incarnation already started or host rebooted')
        authority.checkout_guard(repo, transport_claim=claim)
        subreaper()
        durable.publish(job / 'started.json', durable.encoded(process_identity()), immutable=True)
        train.import_bundle(repo, dict(claim['bundle'], path=str(job / 'candidate.bundle')), request)
        commands = Commands(config, claim, job)
        def checks(*args, **kwargs):
            return worker.mq.run_checks(*args, runner=commands.run, **kwargs)
        result = worker.run_job(repo, Path(config['artifacts']), request, gate=config['gate'],
            disk_floor=config['disk_floor'], checker=checks, owned_checkout=checkout)
        reap()
        # No receipt if even one admitted command lacks its supported proof.
        sequence = commands.evidence()
        if result['completion_known'] is not True or result['reconciliation_required']:
            raise RuntimeError('worker/checker completion unresolved; retain all ownership')
        worker.runtime_unchanged()
        if result.get('before') is not None:
            worker.immutable(repo, result['before'])
        raw = durable.read_bytes(Path(config['artifacts']) / attempt / 'result.json')
        log = durable.checked(Path(result['log']['path']), result['log'])
        durable.publish(job / 'result.json', raw, immutable=True)
        durable.publish(job / 'checks.log', log, immutable=True)
        proof = {'kind': 'subreaper-heavy-v1', 'claim': claim, 'boot': boot(), 'reaped': True,
                 'commands': sequence, 'result': durable.fingerprint(raw), 'log': durable.fingerprint(log),
                 'snapshot': worker.snapshot(repo)}
    durable.publish(job / 'completion.json', durable.encoded(proof), immutable=True)
    # Only status/release removes the marker, after independently checking proof.


def reconcile_reboot(config, request):
    job, claim = read_claim(config, request)
    if claim['boot'] == boot():
        raise RuntimeError('same boot: lost supervisor has no containment proof; do not remove markers; drain/recover heavy and arrange dedicated host reboot')
    with authority.Lock(job / 'supervisor.lock'):
        commands = []
        intents = sorted(job.glob('command-*.intent'), key=lambda p: int(p.stem.split('-')[1]))
        for i, path in enumerate(intents):
            intent = durable.read_json(durable.read_bytes(path))
            if intent['index'] != i or intent['candidate'] != request['candidate']:
                raise ValueError('command journal corrupted')
            done = None
            if intent['heavy']:
                query = heavy_query(config, intent['label'])
                check_heavy(query, intent, allow_absent=True)
                done = {'heavy': query}
            commands.append({'intent': intent, 'done': done})
        proof = {'kind': 'subreaper-heavy-v1', 'claim': claim, 'boot': boot(),
                 'reconciled': True, 'commands': commands}
        durable.publish(job / 'reconciliation.json', durable.encoded(proof), immutable=True)
    release(config, job, claim)


def main():
    global CONFIG_PATH, CONFIG_SHA
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', required=True)
    parser.add_argument('--config-sha256', required=True)
    parser.add_argument('action', choices=('start', 'status', 'supervise', 'reconcile-reboot'))
    parser.add_argument('--attempt')
    parser.add_argument('--checkout-fd', type=int)
    args = parser.parse_args()
    CONFIG_PATH, CONFIG_SHA = args.config, args.config_sha256
    try:
        config = load_config(CONFIG_PATH, CONFIG_SHA)
        if args.action == 'supervise':
            if not args.attempt or not worker.TOKEN.fullmatch(args.attempt):
                raise ValueError('invalid attempt')
            supervise(config, args.attempt, inherited=args.checkout_fd)
            return 0
        value, blobs = wire.unpack(sys.stdin.buffer.read(wire.FRAME_LIMIT + wire.JSON_LIMIT + 1))
        if set(value) != {'version', 'request', 'bundle'} or value['version'] != 1:
            raise ValueError('unknown endpoint envelope version')
        request = worker.validate_request(value['request'])
        if args.action == 'start':
            if len(blobs) != 1 or value['bundle'] is None:
                raise ValueError('start needs exact bundle')
            accept(config, request, value['bundle'], blobs[0])
        elif blobs or value['bundle'] is not None:
            raise ValueError('unexpected endpoint input artifacts')
        if args.action == 'reconcile-reboot':
            reconcile_reboot(config, request)
        reply, returned = status(config, request)
        sys.stdout.buffer.write(wire.pack(reply, returned))
        return 0
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError) as error:
        print('mq endpoint: ' + str(error), file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
