"""Explicit opt-in train operation and retirement; design.md, 3.12."""
from contextlib import contextmanager
import ctypes
import importlib.util
import importlib.machinery
import os
from pathlib import Path
import select
import subprocess
import sys
import time

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
        preparer = train.Preparer(Path(config['preparation']), Path(config['artifacts']),
            metadata_loader=worker.mq.load_metadata, disk_floor=config['disk_floor'])
        controller = train.Controller(preparer, train.StateStore(layout.train), endpoint, effects,
            generation=config['generation'], gates=list(config['endpoints']),
            runtime=worker.runtime_identity(config['disk_floor']), now=time.time,
            batch=config['batch'], full_every=config['full_every'], completion_adapter=endpoint)
        yield controller, lease


def effect_run(endpoint, *args, **kwargs):
    budget = transport.remaining(endpoint, kwargs.get('timeout'))
    if budget is not None:
        kwargs['timeout'] = budget
    return subprocess.run(*args, **kwargs)


def stop(config):
    durable.publish(Path(config['stop']), b'', private_parent=False)


def cycle(controller, config, *, admit=True):
    if Path(config['stop']).exists() and not controller.state['stopping']:
        controller.stop()
    # Both physical gates, including invalidated descendants, are polled first.
    for owner in list(controller.state['gates'].values()):
        if owner:
            controller.poll(owner)
    # Completed preparation already owns its admission. Resume that exact
    # dispatch after a crash, including during stop; no new group is admitted.
    if not controller.unreconciled_gates:
        for attempt in list(controller.state['active']):
            record = controller.state['records'][attempt]
            if record['phase'] == 'prepared' and record['outcome'] is None and not record.get('invalidated'):
                marker = controller.preparer.repo / '.git/mq-preparation-unresolved.json'
                if marker.exists() or marker.is_symlink():
                    controller.reconcile_preparation(attempt)
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
