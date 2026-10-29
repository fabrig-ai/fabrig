"""Explicit opt-in train operation and retirement; design.md, 3.12."""
from contextlib import contextmanager
import ctypes
import importlib.util
import importlib.machinery
import os
from pathlib import Path
import select
import signal
import shlex
import subprocess
import sys
import time
import uuid

sys.dont_write_bytecode = True
DIRECTORY = Path(__file__).resolve().parent

def module(name):
    path = DIRECTORY / (name if name == 'restart' else name + '.py')
    loader = importlib.machinery.SourceFileLoader('operation_' + name, str(path))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

train, transport = module('train'), module('transport')
worker, authority, durable = train.worker, train.authority_module, train.durable


def configuration(path):
    config = durable.read_json(durable.read_bytes(path))
    keys = {'version', 'authority_dir', 'authority_repo', 'origin', 'leads', 'preparation',
            'artifacts', 'generation', 'batch', 'full_every', 'disk_floor', 'endpoints',
            'rpc_timeout', 'stop', 'output'}
    if set(config) != keys or type(config['version']) is not int or config['version'] != 1:
        raise ValueError('unknown train configuration/version; no migration')
    for name in ('authority_dir', 'authority_repo', 'preparation', 'artifacts', 'stop', 'output'):
        transport.absolute(config[name])
    if not worker.TOKEN.fullmatch(config['generation']):
        raise ValueError('invalid train generation')
    for name in ('batch', 'full_every', 'disk_floor'):
        if type(config[name]) is not int or config[name] < 1:
            raise ValueError('positive train policy required')
    if len(config['endpoints']) != 2:
        raise ValueError('exactly two dedicated gates required')
    paths = [Path(config[k]) for k in ('authority_dir', 'authority_repo', 'preparation', 'artifacts')]
    if any(a == b or a in b.parents or b in a.parents for i,a in enumerate(paths) for b in paths[i+1:]):
        raise ValueError('authority, preparation and artifacts require separate paths')
    for endpoint in config['endpoints'].values():
        if endpoint['endpoint']['disk_floor'] != config['disk_floor']:
            raise ValueError('gate and controller disk floor mismatch')
    transport.Transport(config['endpoints'], timeout=config['rpc_timeout'])
    return config


@contextmanager
def open_controller(config, *, inherited=None, deadline=None):
    layout = authority.Layout(config['authority_dir'])
    with layout.acquire(inherited=inherited) as lease:
        if inherited is not None:
            os.close(inherited)
        if (layout.root / 'retired.json').exists():
            raise RuntimeError('generation retired; retained audit forbids revival')
        # Freeze executable mode/config before the first effect. A changed path,
        # known-hosts file, policy or endpoint cannot silently resume a generation.
        durable.publish(layout.root / 'train-operation.json', durable.encoded(config), immutable=True)
        endpoint = transport.Transport(config['endpoints'], timeout=config['rpc_timeout'])
        endpoint.deadline = deadline
        effects = authority.GitAuthority(layout, lease, Path(config['authority_repo']), config['origin'], config['leads'],
            runner=lambda *args, **kwargs: effect_run(endpoint, *args, **kwargs))
        with preparation_budget(endpoint, config, lease=lease):
            preparer = train.Preparer(Path(config['preparation']), Path(config['artifacts']),
                metadata_loader=worker.mq.load_metadata, disk_floor=config['disk_floor'])
        with preparation_budget(endpoint, config, preparer=preparer, lease=lease):
            controller = train.Controller(preparer, train.StateStore(layout.train), endpoint, effects,
                generation=config['generation'], gates=list(config['endpoints']),
                runtime=worker.runtime_identity(config['disk_floor']), now=time.time,
                batch=config['batch'], full_every=config['full_every'], completion_adapter=endpoint)
        with preparation_budget(endpoint, config, controller=controller):
            yield controller, lease


UNKNOWN_CLIENTS = []


CLIENT_KIND = 'preparation-client-v1'
CLIENT_KEYS = {'version', 'kind', 'client', 'attempt', 'generation', 'state',
               'snapshot', 'candidate', 'command', 'preparation'}


def client_identity(identity):
    """Strict private intent; a completed preparation is never client proof."""
    if (not isinstance(identity, dict) or set(identity) != CLIENT_KEYS or
            type(identity['version']) is not int or identity['version'] != 1 or
            identity['kind'] != CLIENT_KIND or
            not isinstance(identity['client'], str) or len(identity['client']) != 32 or
            any(c not in '0123456789abcdef' for c in identity['client']) or
            not isinstance(identity['generation'], str) or not worker.TOKEN.fullmatch(identity['generation'])):
        raise ValueError('unknown preparation client identity/version; no migration')
    transport.absolute(identity['state'])
    if identity['attempt'] is not None and (not isinstance(identity['attempt'], str) or
                                           not worker.TOKEN.fullmatch(identity['attempt'])):
        raise ValueError('invalid preparation client attempt')
    if identity['candidate'] is not None and (not isinstance(identity['candidate'], str) or
                                             not worker.OID.fullmatch(identity['candidate'])):
        raise ValueError('invalid preparation client candidate')
    command = identity['command']
    if (not isinstance(command, dict) or set(command) != {'argv', 'cwd', 'input'} or
            not isinstance(command['argv'], list) or not command['argv'] or
            any(not isinstance(arg, str) or '\0' in arg for arg in command['argv'])):
        raise ValueError('invalid preparation client command')
    transport.absolute(command['cwd'])
    for fingerprint in (identity['snapshot'], command['input']):
        if fingerprint is not None and (not isinstance(fingerprint, dict) or set(fingerprint) != {'bytes', 'sha256'} or
                type(fingerprint['bytes']) is not int or fingerprint['bytes'] < 0 or
                not isinstance(fingerprint['sha256'], str) or len(fingerprint['sha256']) != 64 or
                any(c not in '0123456789abcdef' for c in fingerprint['sha256'])):
            raise ValueError('invalid preparation client fingerprint')
    if identity['preparation'] is not None and identity['preparation'] != {
            'attempt': identity['attempt'], 'generation': identity['generation'], 'state': identity['state']}:
        raise ValueError('preparation client has mismatching preparation ownership')
    return identity


def begin_preparation_client(config, argv, options, body, *, controller=None, preparer=None):
    marker = Path(config['preparation']) / '.git/mq-preparation-unresolved.json'
    snapshot_path = Path(config['authority_dir']) / 'train/state.json'
    snapshot = durable.read_bytes(snapshot_path) if snapshot_path.exists() else None
    state = durable.read_json(snapshot) if snapshot is not None else None
    previous = durable.read_json(durable.read_bytes(marker)) if marker.exists() or marker.is_symlink() else None
    if previous is not None and ('kind' in previous or 'version' in previous):
        client_identity(previous)
        raise worker.UncertainCompletion('preparation client ownership unresolved; supported descendant completion proof required')
    request = preparer._client_identity if preparer is not None else None
    attempt = request['attempt'] if request is not None else previous['attempt'] if previous is not None else None
    record = state['records'].get(attempt, {}) if state is not None else {}
    identity = {'version': 1, 'kind': CLIENT_KIND, 'client': uuid.uuid4().hex,
        'attempt': attempt, 'generation': config['generation'], 'state': str(snapshot_path),
        'snapshot': durable.fingerprint(snapshot) if snapshot is not None else None,
        'candidate': request['candidate'] if request is not None else record.get('candidate'),
        'command': {'argv': [str(arg) for arg in argv], 'cwd': str(Path(options.get('cwd', os.getcwd())).resolve()),
                    'input': durable.fingerprint(body.encode(options.get('encoding') or 'utf-8', options.get('errors') or 'strict') if isinstance(body, str) else body)
                             if body is not None else None},
        'preparation': previous}
    client_identity(identity)
    if (state is not None and state['config']['generation'] != config['generation'] or
            attempt is not None and (attempt not in (state or {}).get('records', {}) or
                request is not None and (request['generation'] != config['generation'] or
                    record.get('candidate', request['candidate']) != request['candidate']))):
        raise ValueError('preparation client differs from durable admission')
    # The immutable intent may be orphaned before marker publication, but no
    # client can start until BOTH durable publications have succeeded.
    journal = Path(config['authority_dir']) / 'preparation-clients'
    durable.directory(journal, create=True)
    intent, proof = journal / (identity['client'] + '.intent.json'), journal / (identity['client'] + '.done.json')
    if any(path.exists() or path.is_symlink() for path in (intent, proof)):
        raise ValueError('preparation client identity already allocated')
    data = durable.encoded(identity)
    durable.publish(intent, data, immutable=True)
    durable.publish(marker, data, private_parent=False)
    return marker, intent, proof, identity


def release_preparation_client(marker, proof, identity):
    receipt = durable.read_json(durable.read_bytes(proof))
    if (not isinstance(receipt, dict) or set(receipt) != {'version', 'kind', 'intent', 'reaped', 'returncode'} or
            type(receipt['version']) is not int or receipt['version'] != 1 or
            receipt['kind'] != CLIENT_KIND or receipt['intent'] != identity or
            receipt['reaped'] is not True or type(receipt['returncode']) is not int):
        raise ValueError('preparation client descendant completion proof differs')
    if durable.read_json(durable.read_bytes(marker)) != identity:
        raise ValueError('preparation client ownership changed before release')
    if identity['preparation'] is not None:
        durable.publish(marker, durable.encoded(identity['preparation']), private_parent=False)
    else:
        marker.unlink()
        durable.sync_directory(marker.parent)
    return receipt['returncode']


class PreparationClient:
    """One isolated subreaper; captured channels are private journal files."""
    def __init__(self, argv, intent, proof, identity, inherited, options, body, endpoint, budget):
        self.pid, self.returncode, self.pidfd = None, None, None
        self.files = []
        self.channels = {}
        self.text = options.get('text', False) or options.get('universal_newlines', False) or options.get('encoding') is not None
        self.encoding, self.errors = options.get('encoding') or 'utf-8', options.get('errors') or 'strict'
        options = dict(options)
        try:
            for name in ('stdout', 'stderr'):
                if options.get(name) == subprocess.PIPE:
                    stream = intent.with_suffix('.' + name).open('x+b')
                    os.fchmod(stream.fileno(), 0o600)
                    self.files.append(stream)
                    self.channels[name] = stream
                    options[name] = stream
            if body is not None:
                stream = intent.with_suffix('.stdin').open('x+b')
                os.fchmod(stream.fileno(), 0o600)
                stream.write(body.encode(self.encoding, self.errors) if isinstance(body, str) else body)
                stream.flush();stream.seek(0)
                self.files.append(stream)
                options['stdin'] = stream
            transport.remaining(endpoint, budget)
            self.pid = os.fork()
            if self.pid == 0:
                try:
                    os.setsid()
                    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0):
                        raise RuntimeError('Linux child subreaper unavailable')
                    # Only the exact held leases and command channels survive
                    # in this isolated owner, never unrelated controller FDs.
                    keep = set(inherited) | {stream.fileno() for stream in self.files}
                    for name in ('stdin', 'stdout', 'stderr'):
                        stream = options.get(name)
                        if hasattr(stream, 'fileno'):keep.add(stream.fileno())
                        elif isinstance(stream, int) and stream >= 3:keep.add(stream)
                    lower = 3
                    for fd in sorted(fd for fd in keep if fd >= 3):
                        os.closerange(lower, fd);lower = fd + 1
                    os.closerange(lower, os.sysconf('SC_OPEN_MAX'))
                    transport.remaining(endpoint, budget)
                    child = subprocess.Popen(argv, pass_fds=tuple(inherited), **options)
                    # This process has exactly one launched family. ECHILD
                    # proves all adopted descendants ended, including setsid.
                    while True:
                        try:
                            pid, status = os.waitpid(-1, 0)
                        except ChildProcessError:
                            break
                        if pid == child.pid:
                            child.returncode = os.waitstatus_to_exitcode(status)
                    durable.publish(proof, durable.encoded({'version': 1, 'kind': CLIENT_KIND,
                        'intent': identity, 'reaped': True, 'returncode': child.returncode}), immutable=True)
                    os._exit(0)
                except BaseException:
                    os._exit(1)
            self.pidfd = os.pidfd_open(self.pid)
        except BaseException:
            self.close()
            raise

    def poll(self):
        if self.returncode is None:
            pid, status = os.waitpid(self.pid, os.WNOHANG)
            if pid:
                self.returncode = os.waitstatus_to_exitcode(status)
                if self.pidfd is not None:
                    os.close(self.pidfd);self.pidfd = None
        return self.returncode

    def communicate(self, *, input=None, timeout=None):
        if not select.select([self.pidfd], [], [], timeout)[0]:
            raise subprocess.TimeoutExpired('preparation client', timeout)
        os.waitid(os.P_PIDFD, self.pidfd, os.WEXITED | os.WNOWAIT)
        self.poll()
        result = []
        for name in ('stdout', 'stderr'):
            stream = self.channels.get(name)
            if stream is None:
                result.append(None)
            else:
                stream.seek(0);data = stream.read()
                result.append(data.decode(self.encoding, self.errors).replace('\r\n', '\n').replace('\r', '\n')
                              if self.text else data)
        return tuple(result)

    def close(self):
        for stream in self.files:
            stream.close()


def quarantine_preparation(config, controller=None):
    """The pre-launch client intent, never a preparation marker, retains ownership."""
    marker = Path(config['preparation']) / '.git/mq-preparation-unresolved.json'
    identity = client_identity(durable.read_json(durable.read_bytes(marker)))
    snapshot_path = Path(identity['state'])
    state = controller.state if controller is not None else (
        train.recovery.validate_state(durable.read_json(durable.read_bytes(snapshot_path))) if snapshot_path.exists() else None)
    if state is not None and identity['attempt'] in state['records']:
        state['records'][identity['attempt']]['preparation_unresolved'] = True
        if controller is not None:
            controller.save()
        else:
            durable.atomic_json(snapshot_path, state)


@contextmanager
def preparation_budget(endpoint, config, *, controller=None, preparer=None, lease=None):
    """Scope every policy Git/metadata call, including nested certificate checks.

    No global subprocess monkeypatch: each independently loaded policy module
    has one synchronous command boundary, restored when this scope ends.
    """
    if controller is not None:
        preparer = controller.preparer
        lease = controller.authority.lock
    commands = preparer.commands if preparer is not None else worker.mq
    original = commands.command
    original_runner = preparer.runner if preparer is not None else None

    def run(argv, **options):
        timeout = options.pop('timeout', None)
        budget = transport.remaining(endpoint, timeout)
        inherited = list(options.pop('pass_fds', ()))
        if lease is not None:
            lease.guard()
            inherited.append(lease.fd)
        if preparer is not None and preparer._ownership is not None:
            preparer._ownership.guard()
            inherited.append(preparer._ownership.fd)
        body = options.pop('input', None)
        if body is not None:
            options['stdin'] = subprocess.PIPE
        # Recheck after setup so an expired scope cannot launch another client.
        budget = transport.remaining(endpoint, budget)
        child = None
        try:
            marker, intent, proof, identity = begin_preparation_client(config, argv, options, body,
                controller=controller, preparer=preparer)
            transport.remaining(endpoint, budget)
            child = PreparationClient(argv, intent, proof, identity, tuple(set(inherited)), options, body, endpoint, budget)
            output, errors = child.communicate(input=body, timeout=transport.remaining(endpoint, budget))
            transport.remaining(endpoint, budget)
            if child.returncode != 0:
                raise worker.UncertainCompletion('preparation client supervisor lost; descendant completion unknown')
            code = release_preparation_client(marker, proof, identity)
            return subprocess.CompletedProcess(argv, code, output, errors)
        except BaseException as error:
            if controller is not None and not isinstance(error, (subprocess.TimeoutExpired, TimeoutError)):
                controller.failed = True
            try:
                if child is not None and isinstance(error, (subprocess.TimeoutExpired, TimeoutError)):
                    quarantine_preparation(config, controller)
            finally:
                # No signal is completion proof. The distinct pre-launch marker
                # survives timeout, controller loss and any publication failure.
                if child is not None:
                    if child.returncode is None:
                        try:
                            os.killpg(child.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    if child.poll() is None:
                        UNKNOWN_CLIENTS.append(child)
            if isinstance(error, (subprocess.TimeoutExpired, TimeoutError)):
                raise worker.UncertainCompletion('preparation/certificate client deadline; descendant completion unknown; retain preparation ownership') from error
            raise
        finally:
            if child is not None:
                child.close()

    def command(argv, cwd, *, check=True, body=None, separate_stderr=False):
        options = {'stdin': subprocess.DEVNULL} if body is None else {'input': body}
        if commands.WRITER_LEASE is not None:
            options['pass_fds'] = commands.writer_fds()
        result = run(argv, cwd=cwd, text=True, stdout=subprocess.PIPE,
                     stderr=subprocess.PIPE if separate_stderr else subprocess.STDOUT, **options)
        if check and result.returncode:
            raise RuntimeError(f'{shlex.join(argv)} exited {result.returncode}:\n{result.stdout}{result.stderr or ""}')
        return result

    commands.command = command
    if preparer is not None:
        preparer.runner = run
    try:
        yield
    finally:
        commands.command = original
        if preparer is not None:
            preparer.runner = original_runner


def effect_run(endpoint, *args, **kwargs):
    budget = transport.remaining(endpoint, kwargs.get('timeout'))
    if budget is not None:
        kwargs['timeout'] = budget
    return subprocess.run(*args, **kwargs)


def stop(config):
    durable.publish(Path(config['stop']), b'', private_parent=False)


def cycle(controller, config, *, admit=True):
    with preparation_budget(controller.transport, config, controller=controller):
        return cycle_owned(controller, config, admit=admit)


def cycle_owned(controller, config, *, admit=True):
    if Path(config['stop']).exists() and not controller.state['stopping']:
        controller.stop()
    # Resolve a completed preparation's exact snapshot-backed marker before
    # any predecessor certificate can need the preparation checkout.
    marker = controller.preparer.repo / '.git/mq-preparation-unresolved.json'
    if marker.exists() or marker.is_symlink():
        identity = durable.read_json(durable.read_bytes(marker))
        if 'kind' in identity or 'version' in identity:
            client_identity(identity)
            raise worker.UncertainCompletion('incomplete preparation client ownership; supported descendant completion proof required')
        attempt = identity.get('attempt')
        if attempt not in controller.state['records']:
            raise RuntimeError('preparation command ownership unknown; retain marker and reconcile supported evidence')
        controller.reconcile_preparation(attempt)
    # Both physical gates, including invalidated descendants, are then polled.
    for owner in list(controller.state['gates'].values()):
        if owner:
            controller.poll(owner)
    # Completed preparation already owns its admission. Resume that exact
    # dispatch after a crash, including during stop; no new group is admitted.
    if not controller.unreconciled_gates:
        for attempt in list(controller.state['active']):
            record = controller.state['records'][attempt]
            if record['phase'] == 'prepared' and record['outcome'] is None and not record.get('invalidated'):
                controller.dispatch(attempt)
    # Includes uncertain promotion-intents: fresh main must be exact P or C.
    while controller.adjudicate_head():
        pass
    controller.flush_outbox()
    if admit and not Path(config['stop']).exists() and not controller.state['stopping']:
        for _ in range(2):
            if Path(config['stop']).exists():
                controller.stop()
                break
            if controller.admit() is None:
                break
    return controller.recovery_inventory()


def resume(controller, config):
    cycle(controller, config, admit=False)
    if (not controller.drained() or controller.unreconciled_gates or not controller.main_consistent() or
            any(not e['delivered'] for e in controller.state['outbox'].values())):
        raise RuntimeError('resume requires drained gates, resolved preparation/promotions and delivered outbox; inspect status')
    # Pending bisection remains in the snapshot and will run in its recorded order.
    Path(config['stop']).unlink(missing_ok=True)
    durable.sync_directory(Path(config['stop']).parent)
    controller.state['stopping'] = False
    controller.save()


def retire(controller, config):
    controller.guard()
    controller.stop()
    if (not controller.drained() or controller.state['pending'] or controller.state['bisecting'] or
            any(not e['delivered'] for e in controller.state['outbox'].values())):
        raise RuntimeError('retirement refused: active/unknown ownership, pending bisection or undelivered outbox; inspect status')
    layout = controller.authority.layout
    receipt = {'version': 1, 'generation': controller.config['generation'],
               'state': durable.fingerprint(durable.read_bytes(controller.store.path)),
               'owner': durable.fingerprint(durable.read_bytes(layout.root / 'train-owner.json')),
               'operation': durable.fingerprint(durable.read_bytes(layout.root / 'train-operation.json')),
               'time': worker.mq.utc_now()}
    durable.publish(layout.root / 'retired.json', durable.encoded(receipt), immutable=True)
    # State, pins, requests, receipts, logs and outbox remain in place.
    return receipt


def inspect(config):
    root = Path(config['authority_dir'])
    state_path = root / 'train/state.json'
    if not state_path.exists():
        retained = state_path.is_symlink() or (root / 'train-owner.json').exists()
        return {'state': 'missing-snapshot' if retained else 'uninitialized', 'main': 'unknown'}
    if durable.read_json(durable.read_bytes(root / 'train-operation.json')) != config:
        raise ValueError('inspection configuration differs from retained operation')
    snapshot = train.StateStore(root / 'train').load()
    return {'generation': snapshot['config']['generation'], 'main': 'unknown',
            'stopping': snapshot['stopping'], 'active': snapshot['active'],
            'retired': (root / 'retired.json').exists(),
            'ownership': 'recorded; use train-reconcile for endpoint evidence',
            **train.recovery.inventory(snapshot)}


def episode(controller, error):
    path = controller.authority.layout.root / 'error-episode.json'
    record = durable.read_json(durable.read_bytes(path)) if path.exists() else {'version': 1, 'errors': {}}
    message = str(error)
    if message not in record['errors']:
        import uuid
        identity = 'cycle-' + uuid.uuid4().hex
        record['errors'][message] = {'id': identity, 'delivered': False}
        durable.atomic_json(path, record)
    event = record['errors'][message]
    if not event['delivered']:
        code = controller.authority.post({'kind': 'infrastructure', 'id': event['id'],
            'body': 'Event: ' + event['id'] + '\nTrain cycle error: ' + message + '\nEvidence: ' + str(path) + '\n'})
        event['delivered'] = type(code) is int and code in (0, 3)
        durable.atomic_json(path, record)


def completed_cycle(controller):
    path = controller.authority.layout.root / 'error-episode.json'
    if path.exists():
        durable.atomic_json(path, {'version': 1, 'errors': {}})


def run(config, action, *, inherited=None, startup_fd=None):
    if action == 'train-stop':
        stop(config)
        return 0
    if action == 'train-status':
        print(durable.encoded(inspect(config)).decode(), end='')
        return 0
    if action not in ('run', 'once', 'train-drain', 'train-reconcile', 'train-resume', 'train-retire'):
        raise ValueError('unsupported train action')
    with open_controller(config, inherited=inherited) as (controller, lease):
        if action == 'train-resume':
            resume(controller, config)
            return 0
        if action == 'train-retire':
            print(durable.encoded(retire(controller, config)).decode(), end='')
            return 0
        if action == 'train-reconcile':
            cycle(controller, config, admit=False)
            # Explicitly release a proved physical owner whose check certificate
            # was INFRA. No result is converted into a green/red decision.
            for owner in list(controller.state['gates'].values()):
                if owner:
                    record = controller.state['records'][owner]
                    reply = controller.transport.status(record['request'])
                    if reply['state'] in ('terminal', 'reconciled'):
                        controller.reconcile_job(owner, quiescent=reply.get('quiescent'), evidence=reply.get('evidence'), supervision=reply.get('supervision'))
            print(durable.encoded(cycle(controller, config, admit=False)).decode(), end='')
            return 0
        if action == 'train-drain':
            stop(config)
            controller.stop()
        cycle(controller, config, admit=False)
        if startup_fd is not None:
            if controller.unreconciled_gates or not controller.main_consistent():
                raise RuntimeError('startup workers unknown; stop retained')
            os.write(startup_fd, b'READY\n')
            os.close(startup_fd)
        while True:
            try:
                cycle(controller, config, admit=action != 'train-drain')
                completed_cycle(controller)
                if action == 'once' or (controller.state['stopping'] and controller.drained()):
                    return 0
            except Exception as error:
                if controller.failed or action != 'run':
                    raise
                try:
                    episode(controller, error)
                except Exception as reporting:
                    print('train error reporting: ' + str(reporting), file=sys.stderr)
                print('train cycle: ' + str(error), file=sys.stderr)
            wait_cycle(Path(config['stop']), 60, draining=controller.state['stopping'])


def wait_cycle(stop_path, seconds, *, draining=False):
    """Existing cadence, interruptible by the stop file without poll sleeps."""
    libc = ctypes.CDLL(None, use_errno=True)
    fd = libc.inotify_init1(os.O_CLOEXEC | os.O_NONBLOCK)
    if fd < 0:
        raise OSError(ctypes.get_errno(), 'inotify_init1')
    try:
        if libc.inotify_add_watch(fd, os.fsencode(stop_path.parent), 0x100 | 0x80) < 0:
            raise OSError(ctypes.get_errno(), 'inotify_add_watch')
        if stop_path.exists():
            if not draining:
                return
            select.select([], [], [], seconds)
            return
        deadline = time.monotonic() + seconds
        while not stop_path.exists():
            left = deadline - time.monotonic()
            if left <= 0 or not select.select([fd], [], [], left)[0]:
                return
            os.read(fd, 65536)
    finally:
        os.close(fd)


def drain(controller, config, wait, *, now=time.monotonic, tick=None):
    """Bounded physical drain for restart; no update or replacement on failure."""
    stop(config)
    controller.stop()
    previous = getattr(controller.transport, 'deadline', None)
    deadline = now() + wait
    if previous is not None:
        deadline = min(deadline, previous)
    controller.transport.deadline = deadline
    try:
        with preparation_budget(controller.transport, config, controller=controller):
            while True:
                if controller.drained():
                    return
                left = deadline - now()
                if left <= 0:
                    raise TimeoutError('train drain timed out; stop retained, runtime unchanged')
                cycle(controller, config, admit=False)
                if controller.drained():
                    return
                if controller.unreconciled_gates or any(r.get('error') for r in controller.state['records'].values()
                        if r['id'] in controller.state['gates'].values()):
                    raise RuntimeError('physical drain unknown; inspect status and endpoint reconciliation; stop retained')
                left = deadline - now()
                if left <= 0:
                    raise TimeoutError('train drain timed out; stop retained, runtime unchanged')
                if tick:
                    tick()
                else:
                    select.select([], [], [], min(60, left))
    finally:
        controller.transport.deadline = previous


def launch(*args, **kwargs):
    return subprocess.Popen(*args, **kwargs)


def restart(config_path, wait=1800):
    """Restart same frozen runtime/config, under one authority open description.

    Updating code or config of a generation is deliberately refused. Retire it
    explicitly before deployment; this helper never edits a running runtime.
    """
    config = configuration(config_path)
    layout = authority.Layout(config['authority_dir'])
    stop(config)
    deadline = time.monotonic() + wait
    with layout.restart_lock():
        try:
            # Wait for a live controller to finish; death only permits inspection.
            # The common authority's actual holder is identified through its inode.
            helper = module('restart')
            pid = helper.queue_pid_for_lock(layout.root / 'authority.lock')
            if pid is not None:
                try:
                    fd = os.pidfd_open(pid)
                except ProcessLookupError:
                    fd = None
                if fd is not None:
                    try:
                        if not select.select([fd], [], [], max(0, deadline-time.monotonic()))[0]:
                            raise TimeoutError('controller drain timed out; no replacement, stop retained')
                    finally:
                        os.close(fd)
            with open_controller(config, deadline=deadline) as (controller, lease):
                # A dead controller may have left TWO independent physical owners.
                drain(controller, config, max(0, deadline-time.monotonic()))
                resume(controller, config)
                output = Path(config['output'])
                durable.directory(output.parent)
                read_fd, write_fd = os.pipe()
                try:
                    with output.open('ab') as log:
                        os.chmod(output, 0o600)
                        transport.remaining(controller.transport)
                        child = launch(['python3', '-I', '-B', str(DIRECTORY / 'mq'), 'run',
                            '--train-config', str(config_path), '--authority-fd', str(lease.fd),
                            '--startup-fd', str(write_fd)], stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                            start_new_session=True, pass_fds=(lease.fd, write_fd))
                    os.close(write_fd)
                    write_fd = None
                    helper.await_start(child, read_fd, output)
                except BaseException:
                    stop(config)
                    raise
                finally:
                    os.close(read_fd)
                    if write_fd is not None:
                        os.close(write_fd)
        except BaseException:
            stop(config)
            raise
