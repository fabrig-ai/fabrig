"""Opt-in CPU collector and evaluator (tools/tiers/design.md, 7)."""

from collections import Counter
import contextlib
import io
import json
import math
import os
from pathlib import Path
import re
import runpy
import selectors
import signal
import statistics
import subprocess
import sys
import time
import tomllib
import xml.etree.ElementTree as ET


# Every budget red ends here (tools/tiers/budgets.md, 10.1).
REPAIR_LINE = "repair: read skein/docs/foundation/testing-strategy.md in full first, then ask 8.1's three questions in order: does the test belong to its tier; if it does, can it be made faster; if neither, take the measurements to the user."

GROUPS = ("skein", "smith", "jig", "temper", "hearth", "repository")
KEYS = {"cpu_suite_s", "cpu_test_s", "wall_hard_s"}
# nextest's abort message for the signals of cancellation (tools/tiers/design.md, 7.3).
CANCELLED = re.compile(r'\baborted with signal (?:9|15)\b')
# nextest's line when its global-timeout cancels a run (tools/tiers/design.md, 7.3).
GLOBAL_TIMEOUT = re.compile(rb'^\s*Cancelling due to global timeout\b', re.MULTILINE)
COLOUR = re.compile(rb'\x1b\[[0-9;]*m')
# Outlast nextest's ten-second grace and the wrapper's settlement (design.md, 7.3).
GRACE_S = 15.0
ONE_PROCESS = Path(__file__).with_name('one_process.py')


def timed_out(directory):
    """Whether nextest's global-timeout cancelled the run in `directory`."""
    try:
        output = (directory / 'command-output.txt').read_bytes()
    except FileNotFoundError:
        return False
    return GLOBAL_TIMEOUT.search(COLOUR.sub(b'', output)) is not None


def session_processes(session):
    """The live processes of `session`; a zombie is nothing left to stop."""
    processes = []
    for path in Path('/proc').iterdir():
        if not path.name.isdecimal():
            continue
        try:
            stat = (path / 'stat').read_bytes().rsplit(b')', 1)[1].split()
        except (FileNotFoundError, ProcessLookupError):
            continue
        if stat[0] != b'Z' and int(stat[3]) == session:
            processes.append(int(path.name))
    return processes


def signal_group(process, sig):
    try:
        os.killpg(process.pid, sig)
    except ProcessLookupError:
        pass


def stop(process, deadline):
    """Wait until `deadline` for the command's session to end, then SIGKILL what is left."""
    while session_processes(process.pid):
        if time.monotonic() >= deadline:
            break
        time.sleep(0.05)
    kill_until = time.monotonic() + 1.0
    while remaining := session_processes(process.pid):
        for pid in remaining:
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        if time.monotonic() >= kill_until:
            break
        time.sleep(0.05)
    process.wait()


def gate_ceilings(profile):
    """Map (tree, layer) to its ceiling (tools/tiers/budgets.md, 10)."""
    ceilings = {(tree, layer): row['cpu_s']
                for tree in GROUPS for layer, row in profile.get(tree, {}).items()
                if layer != 'ideal_s'}
    for move in profile.get('moves', []):
        source, target = tuple(move['from'].split('.')), tuple(move['to'].split('.'))
        ceilings[source] = ceilings.get(source, 0) - move['cpu_s']
        ceilings[target] = ceilings.get(target, 0) + move['cpu_s']
    return ceilings


def budgets(path, pairs=None, layers=Path('tools/tiers/layers.toml')):
    with Path(path).open("rb") as source:
        data = tomllib.load(source)

    def seconds(name, value):
        if type(value) not in (int, float) or not math.isfinite(value) or value <= 0:
            raise ValueError(f"{name} must be finite positive seconds")

    def validate(name, values, *, pair=False):
        keys = KEYS | {"auto_reap"} if pair else KEYS
        if not isinstance(values, dict) or values.keys() - keys:
            raise ValueError(f"invalid budget keys for {name}")
        for key, value in values.items():
            if key == "auto_reap":
                if not isinstance(value, list) or any(not isinstance(test, str) or not test for test in value):
                    raise ValueError(f"{name}.auto_reap must be a list of nonempty test names")
                continue
            seconds(f"{name}.{key}", value)
    for pair, values in data.items():
        if pair == "tools":
            if not isinstance(values, dict):
                raise ValueError("tools budgets must be a table keyed by suite ID")
            for suite, limits in values.items():
                if not suite:
                    raise ValueError("empty tools suite ID")
                validate(f"tools.{suite}", limits)
        elif pair == "gate":
            with Path(layers).open('rb') as source:
                layer_order = tomllib.load(source)['layers']
                layer_names = set(layer_order)
            if not isinstance(values, dict) or values.keys() - {'default', 'fuzzy'}:
                raise ValueError("gate budgets must be a table keyed by default or fuzzy")
            for profile, groups in values.items():
                name = f'gate.{profile}'
                if not isinstance(groups, dict) or groups.keys() - {*GROUPS, 'references', 'moves', 'reasons'}:
                    raise ValueError(f'invalid budget groups for {name}')
                references = groups.get('references')
                if not isinstance(references, dict) or references.keys() - (layer_names | {'checks'}):
                    raise ValueError(f'invalid references for {name}')
                for layer, value in references.items():
                    seconds(f'{name}.references.{layer}', value)
                for group, table in groups.items():
                    if group in ('references', 'moves', 'reasons'):
                        continue
                    allowed = {'checks'} if group == 'repository' else layer_names
                    if (not isinstance(table, dict) or 'ideal_s' not in table
                            or table.keys() - (allowed | {'ideal_s'})):
                        raise ValueError(f'invalid budget keys for {name}.{group}')
                    seconds(f'{name}.{group}.ideal_s', table['ideal_s'])
                    for layer, row in table.items():
                        if layer == 'ideal_s':
                            continue
                        if (not isinstance(row, dict) or 'cpu_s' not in row
                                or row.keys() - {'cpu_s', 'ideal_s', 'reference_s'}):
                            raise ValueError(f'invalid budget keys for {name}.{group}.{layer}')
                        for key, value in row.items():
                            seconds(f'{name}.{group}.{layer}.{key}', value)
                        if 'reference_s' not in row and layer not in references:
                            raise ValueError(f'no reference for {name}.{group}.{layer}')
                reasons = groups.get('reasons', [])
                if not isinstance(reasons, list):
                    raise ValueError(f'{name}.reasons must be an array of tables')
                seen = set()
                for index, line in enumerate(reasons):
                    label = f'{name}.reasons[{index}]'
                    if not isinstance(line, dict) or line.keys() != {'binary', 'test', 'cpu_s', 'why'}:
                        raise ValueError(f'invalid reason keys for {label}')
                    for key in ('binary', 'test', 'why'):
                        if not isinstance(line[key], str) or not line[key]:
                            raise ValueError(f'{label}.{key} must be a nonempty string')
                    seconds(f'{label}.cpu_s', line['cpu_s'])
                    key = line['binary'], line['test']
                    if key in seen:
                        raise ValueError(f'duplicate reason for {key}')
                    seen.add(key)
                moves = groups.get('moves', [])
                if not isinstance(moves, list):
                    raise ValueError(f'{name}.moves must be an array of tables')
                for index, move in enumerate(moves):
                    label = f'{name}.moves[{index}]'
                    if not isinstance(move, dict) or move.keys() != {'from', 'to', 'cpu_s', 'why'}:
                        raise ValueError(f'invalid move keys for {label}')
                    endpoints = {}
                    for key in ('from', 'to'):
                        value = move[key]
                        parts = value.split('.') if isinstance(value, str) else []
                        if len(parts) != 2 or parts[0] not in GROUPS[:-1] or parts[1] not in layer_names:
                            raise ValueError(f'{label}.{key} must be TREE.LAYER of a tree and layer')
                        endpoints[key] = parts
                    if layer_order.index(endpoints['to'][1]) >= layer_order.index(endpoints['from'][1]):
                        raise ValueError(f'{label}.to must be below from in the layers list')
                    seconds(f'{label}.cpu_s', move['cpu_s'])
                    if not isinstance(move['why'], str) or not move['why']:
                        raise ValueError(f'{label}.why must be a nonempty string')
                    if endpoints['to'][0] not in groups:
                        raise ValueError(f'{label}.to tree has no table for {name}')
                for (tree, layer), ceiling in gate_ceilings(groups).items():
                    if ceiling < 0:
                        raise ValueError(f'{name}.{tree}.{layer} ceiling below 0 after moves')
        else:
            if pairs is not None and pair not in pairs:
                raise ValueError(f"unknown budget pair {pair}")
            validate(pair, values, pair=True)
    return data


def ledger(path):
    """Read main's suite ledger (tools/tiers/budgets.md, 7)."""
    data = json.loads(Path(path).read_text())

    def keys(value, expected, label):
        if not isinstance(value, dict) or value.keys() != expected:
            raise ValueError(f'invalid ledger keys for {label}')

    def name(value, label, nullable=False):
        if not (nullable and value is None) and (not isinstance(value, str) or not value):
            raise ValueError(f'{label} must be a nonempty name')

    def figure(value, label, positive=False):
        if (type(value) not in (int, float) or not math.isfinite(value)
                or value < 0 or (positive and value == 0)):
            raise ValueError(f'{label} must be finite nonnegative seconds' if not positive
                             else f'{label} must be finite positive seconds')

    def groups(value, label):
        if not isinstance(value, dict) or value.keys() - set(GROUPS):
            raise ValueError(f'invalid ledger groups for {label}')

    keys(data, {'ledger', 'profile', 'binaries', 'budgets', 'moves'}, 'ledger')
    if type(data['ledger']) is not int or data['ledger'] not in (1, 2, 3):
        raise ValueError('ledger version must be 1, 2 or 3')
    if data['profile'] not in ('default', 'fuzzy'):
        raise ValueError('ledger profile must be default or fuzzy')
    if not isinstance(data['binaries'], dict):
        raise ValueError('ledger binaries must be an object')
    for binary, entry in data['binaries'].items():
        name(binary, 'binary id')
        cpu_key = 'cpu' if data['ledger'] == 3 else 'cpu_s'
        keys(entry, {'tree', 'layer', cpu_key, 'tests'} | ({'alone'} if data['ledger'] != 1 else set()), binary)
        if entry['tree'] not in GROUPS:
            raise ValueError(f'invalid ledger group for {binary}')
        name(entry['layer'], f'{binary}.layer', nullable=True)
        if data['ledger'] != 3:
            figure(entry['cpu_s'], f'{binary}.cpu_s')
            entry['cpu'] = [entry.pop('cpu_s')]
        if not isinstance(entry['cpu'], list) or not 1 <= len(entry['cpu']) <= 3:
            raise ValueError(f'{binary}.cpu must have one to three figures')
        for cpu in entry['cpu']:
            figure(cpu, f'{binary}.cpu')
        if not isinstance(entry['tests'], dict):
            raise ValueError(f'{binary}.tests must be an object')
        for test, walls in entry['tests'].items():
            name(test, 'test name')
            if not isinstance(walls, list) or not 1 <= len(walls) <= 3:
                raise ValueError(f'{binary} {test} must have one to three walls')
            for wall in walls:
                figure(wall, f'{binary} {test} wall')
        if data['ledger'] == 1:
            entry['alone'] = {}
        if not isinstance(entry['alone'], dict):
            raise ValueError(f'{binary}.alone must be an object')
        for test, figures in entry['alone'].items():
            if test not in entry['tests']:
                raise ValueError(f'{binary} {test} alone name not in tests')
            if not isinstance(figures, list) or not 1 <= len(figures) <= 3:
                raise ValueError(f'{binary} {test} must have one to three alone figures')
            for cpu in figures:
                figure(cpu, f'{binary} {test} alone')
    groups(data['budgets'], 'budgets')
    for tree, layers in data['budgets'].items():
        if not isinstance(layers, dict):
            raise ValueError(f'ledger budgets for {tree} must be an object')
        for layer, entry in layers.items():
            name(layer, f'{tree} layer')
            keys(entry, {'ceiling_s', 'lowest_s'}, f'{tree}.{layer}')
            for key, value in entry.items():
                figure(value, f'{tree}.{layer}.{key}')
    if not isinstance(data['moves'], list):
        raise ValueError('ledger moves must be an array')
    for move in data['moves']:
        keys(move, {'from', 'to', 'cpu_s', 'why'}, 'move')
        for key in ('from', 'to'):
            parts = move[key].split('.') if isinstance(move[key], str) else []
            if len(parts) != 2 or parts[0] not in GROUPS[:-1] or not parts[1]:
                raise ValueError(f'ledger move {key} must be TREE.LAYER')
        figure(move['cpu_s'], 'move cpu_s', positive=True)
        name(move['why'], 'move why')
    return data


def save(path, value):
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(value, allow_nan=False) + "\n")
    temporary.replace(path)


def toml_config(data, prefix=()):
    """Serialize nextest's TOML values; the private copy preserves parsed values."""
    lines = []
    def scalar(value):
        if isinstance(value, bool):
            return str(value).lower()
        if isinstance(value, (str, int, float)):
            return json.dumps(value, ensure_ascii=False, allow_nan=False)
        if isinstance(value, list):
            return '[' + ', '.join(scalar(item) for item in value) + ']'
        if isinstance(value, dict):
            return '{' + ', '.join(json.dumps(key) + ' = ' + scalar(item) for key, item in value.items()) + '}'
        raise ValueError(f"unsupported nextest configuration value {type(value).__name__}")
    for key, value in data.items():
        if not isinstance(value, dict):
            lines.append(f'{json.dumps(key)} = {scalar(value)}')
    for key, value in data.items():
        if isinstance(value, dict):
            path = (*prefix, key)
            lines.append('[' + '.'.join(json.dumps(part) for part in path) + ']')
            lines.append(toml_config(value, path))
    return '\n'.join(lines) + '\n'


def prepare(directory, profile, wrapper, config=Path('.config/nextest.toml')):
    """Create a fresh collector; returned arguments work with any nextest profile."""
    directory = Path(directory).resolve()
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    if any(directory.iterdir()):
        raise ValueError(f"CPU directory must be empty: {directory}")
    (directory / "records").mkdir(mode=0o700)
    script = "@tool:tiers-cpu:account"
    overlay = directory / "nextest.toml"
    repository = tomllib.loads(Path(config).read_text())
    repository['experimental'] = sorted(set(repository.get('experimental', [])) | {'wrapper-scripts'})
    repository.setdefault('profile', {}).setdefault(profile, {}).setdefault('junit', {})['path'] = str(directory / 'junit.xml')
    private = directory / 'repository.toml'
    private.write_text(toml_config(repository))
    if tomllib.loads(private.read_text()) != repository:
        raise ValueError('private nextest configuration does not round-trip')
    overlay.write_text(
        f'[scripts.wrapper.{json.dumps(script)}]\n'
        f'command = {json.dumps([str(Path(wrapper).resolve()), str(directory / "records")])}\n'
        'target-runner = "within-wrapper"\n'
        f'[[profile.{json.dumps(profile)}.scripts]]\n'
        'filter = "all()"\n'
        f'run-wrapper = {json.dumps(script)}\n'
        f'[profile.{json.dumps(profile)}.junit]\n'
        f'path = {json.dumps(str(directory / "junit.xml"))}\n')
    save(directory / "run.json", {"profile": profile, "finished": False, "starved": False})
    return ["--config-file", str(private), "--tool-config-file", f"tiers-cpu:{overlay}"]


def records(directory, *, tools=False):
    result = []
    seen = set()
    for path in sorted((directory if tools else directory / "records").iterdir()):
        if path.name.endswith(".tmp") or (tools and not path.name.endswith(".json")):
            continue
        lines = path.read_text().splitlines()
        if len(lines) != 1:
            raise ValueError(f"expected one record in {path}")
        item = json.loads(lines[0])
        key = (item["profile"], item["binary_id"], item["test_name"], item["attempt"])
        if (any(not isinstance(value, str) or not value for value in key[:3])
                or type(key[3]) is not int or key[3] < 1 or key in seen):
            raise ValueError(f"invalid or duplicate identity in {path}")
        seen.add(key)
        for field in ("user_s", "system_s", "wall_s", "start_s", "wrapper_cpu_s"):
            value = item[field]
            if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
                raise ValueError(f"invalid {field} in {path}")
        for field in ("complete", "pending"):
            if type(item[field]) is not bool:
                raise ValueError(f"invalid {field} in {path}")
        if tools and type(item.get('starved')) is not bool:
            raise ValueError(f"invalid starved in {path}")
        if type(item["leaked"]) is not int or item["leaked"] < 0:
            raise ValueError(f"invalid leaked in {path}")
        if ('settling' in item and (type(item['settling']) is not bool
                or item['settling'] and (item['pending'] or item['complete']))):
            raise ValueError(f"invalid settlement state in {path}")
        if 'complete_scope' in item and item['complete_scope'] != 'waited-and-reaped-descendants':
            raise ValueError(f"invalid completeness scope in {path}")
        if type(item['interrupted']) is not int or not 0 <= item['interrupted'] < signal.NSIG:
            raise ValueError(f"invalid interrupted signal in {path}")
        code, sig = item["exit_code"], item["signal"]
        if ((code is not None and (type(code) is not int or not 0 <= code <= 255))
                or (sig is not None and (type(sig) is not int or not 1 <= sig < signal.NSIG))
                or (item["pending"] and (item["complete"] or code is not None or sig is not None))
                or (not item["pending"] and ((code is None) == (sig is None)))):
            raise ValueError(f"invalid exit status in {path}")
        result.append(item)
    return result


def junit(directory, profile):
    root = ET.parse(directory / "junit.xml").getroot()
    expected, timeouts, failed, passed, aborts = set(), set(), {}, set(), set()
    for suite in root.findall("testsuite"):
        # Setup scripts have no wrapper and are not test attempts.
        if suite.attrib["name"].startswith("@setup-script:"):
            continue
        for case in suite.findall("testcase"):
            if case.find("skipped") is not None:
                continue
            binary, name = suite.attrib["name"], case.attrib["name"]
            reruns = [element for element in case if element.tag in (
                "flakyFailure", "flakyError", "rerunFailure", "rerunError")]
            count = 1 + len(reruns)
            flaky = any(element.tag.startswith("flaky") for element in reruns)
            primary = [element for element in case if element.tag in ("failure", "error")]
            # Failed runs put the first failure in the primary element; flaky
            # successes put the final success there and the earlier failures after it.
            outcomes = [*reruns, *primary] if flaky else [*primary, *reruns]
            if flaky:
                outcomes.append(None)
            if not outcomes:
                outcomes = [None]
            if len(outcomes) != count:
                # flaky-result=fail also adds a primary failure for the final attempt.
                if flaky and primary:
                    outcomes = [*reruns, *primary]
                if len(outcomes) != count:
                    raise ValueError(f"invalid JUnit attempts for {binary} {name}")
            for attempt, outcome in enumerate(outcomes, 1):
                key = (profile, binary, name, attempt)
                if key in expected:
                    raise ValueError(f"duplicate JUnit test {key}")
                expected.add(key)
                if outcome is None:
                    passed.add(key)
                else:
                    kind = outcome.get("type", "").lower()
                    if "timeout" in kind:
                        timeouts.add(key)
                    elif primary:
                        failed.setdefault(f"{binary} {name}", set()).add(key)
                    if "abort" in kind and (CANCELLED.search(outcome.get("message", ""))
                                             or CANCELLED.search("".join(outcome.itertext()))):
                        aborts.add(key)
    return expected, timeouts, failed, passed, aborts


def summary(directory, limits=None, pair=None, expected=None, gate=False, ledger=None, base=None):
    directory = Path(directory)
    if pair == "tools":
        if base is not None:
            raise ValueError('base requires a one-process run, not tools')
        if ledger is not None:
            raise ValueError('ledger requires a one-process run, not tools')
        return tools_summary(directory, limits, expected)
    if expected is not None:
        raise ValueError("--expect requires --pair tools")
    run = json.loads((directory / "run.json").read_text())
    if run.get("finished") is not True or type(run.get("starved")) is not bool:
        raise ValueError("run is not finished")
    profile = run["profile"]
    wall = run['wall_s']
    if not isinstance(profile, str) or not profile or type(wall) not in (int, float) or not math.isfinite(wall) or wall < 0:
        raise ValueError('invalid run profile or wall time')
    pair = pair or profile
    if pair != profile:
        raise ValueError(f"pair {pair} does not match run profile {profile}")
    if run.get('mode') == 'one-process':
        return one_process_summary(directory, limits, pair, run, gate, ledger, base)
    if base is not None:
        raise ValueError('base requires a one-process run, not nextest')
    if ledger is not None:
        raise ValueError('ledger requires a one-process run, not nextest')
    if 'mode' in run:
        raise ValueError(f"invalid run mode: {run['mode']}")
    if timed_out(directory):
        run = dict(run, starved=True)
    items = records(directory)
    actual = {(r['profile'], r['binary_id'], r['test_name'], r['attempt']) for r in items}
    if any(key[0] != profile for key in actual):
        raise ValueError("record profile does not match run")
    expected, passed = set(), set()
    unobserved = set()
    if (directory / "junit.xml").exists():
        expected, timeouts, failed, passed, aborts = junit(directory, profile)
        missing = expected - actual
        cancelled = set()
        if run["starved"]:
            # Cancellation can stop a wrapper before its pre-exec record.
            unobserved = missing & aborts
            missing -= unobserved
            by_key = {(r['profile'], r['binary_id'], r['test_name'], r['attempt']): r for r in items}
            cancelled = {key for key in aborts & actual
                         if by_key[key]['pending'] or by_key[key]['signal'] in (signal.SIGTERM, signal.SIGKILL)}
            timeouts |= cancelled
        extra = actual - expected
        if missing or (extra and not run["starved"]):
            raise ValueError(f"record coverage: missing {sorted(missing)}; unexpected {sorted(extra)}")
        failures = [name for name, keys in failed.items() if keys - unobserved - cancelled]
    elif run.get('returncode') == 4 and not items:
        timeouts, failures = set(), []
    elif run["starved"]:
        # SIGKILL may come before JUnit; the pre-exec records survive it.
        # Counts describe started attempts, never unstarted tests.
        timeouts, failures = set(), []
    else:
        raise ValueError("missing nextest JUnit report (cannot verify record coverage)")
    timeouts.update((r['profile'], r['binary_id'], r['test_name'], r['attempt']) for r in items
                    if r['interrupted'] and r['signal'] is not None and not run['starved'])
    # JUnit represents nextest's on-timeout=pass policy as success even when
    # the test was terminated. Ordinary signal failures have a failure element.
    timeouts.update((r['profile'], r['binary_id'], r['test_name'], r['attempt']) for r in items
                    if r['signal'] in (signal.SIGTERM, signal.SIGKILL)
                    and (r['profile'], r['binary_id'], r['test_name'], r['attempt']) in passed)
    unresolved = {}
    superseded_junit = set()
    junit_last = {}
    for _, binary, name, attempt in expected:
        junit_last[binary, name] = max(junit_last.get((binary, name), 0), attempt)
    for item in sorted(items, key=lambda r: r['attempt']):
        if item['pending']:
            continue
        test = item['binary_id'], item['test_name']
        key = (profile, *test, item['attempt'])
        if item['exit_code'] == 0:
            unresolved.pop(test, None)
            if item['attempt'] > junit_last.get(test, 0):
                superseded_junit.add(f"{test[0]} {test[1]}")
        elif item['signal'] not in (signal.SIGTERM, signal.SIGKILL) or key not in timeouts:
            unresolved[test] = f"{test[0]} {test[1]}"
    failures = sorted((set(failures) - superseded_junit) | set(unresolved.values()))
    result = summarize_records(items, pair, limits, run, timeouts, failures)
    result['unobserved'] = [dict(binary_id=binary, test_name=name, attempt=attempt)
                            for _, binary, name, attempt in sorted(unobserved)]
    result['suite']['unobserved'] = len(unobserved)
    result['warnings'].extend(
        f"{pair} {binary} {name}: attempt {attempt} unobserved, cancelled before its record"
        for _, binary, name, attempt in sorted(unobserved))
    return result


def one_process_summary(directory, limits, pair, run, gate=False, ledger=None, base=None):
    """Read process CPU and test outcomes (tools/tiers/design.md, 10.2 and 11.8)."""
    gate_limits = (limits or {}).get('gate', {}).get(run['profile'], {})
    limits = (limits or {}).get(pair, {})
    if 'cpu_test_s' in limits:
        raise ValueError('one-process runs cannot measure cpu_test_s')
    data = json.loads((directory / 'results.json').read_text())
    if ledger is not None:
        if ledger['profile'] != run['profile']:
            raise ValueError('ledger profile does not match run')
        if 'workspace' not in data:
            raise ValueError('ledger requires results workspace')
    if data['profile'] != run['profile']:
        raise ValueError('results profile does not match run')
    if (type(data['selected']) is not int or data['selected'] < 0
            or data['no_tests'] not in ('fail', 'warn', 'pass')
            or data['cancelled'] not in (None, 'global-timeout', 'wall-hard')):
        raise ValueError('invalid selection or cancellation')

    def figure(value, field):
        if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
            raise ValueError(f'invalid {field} in one-process results')
        return value

    def names(value):
        return isinstance(value, list) and all(isinstance(name, str) and name for name in value)

    def identity(item):
        key = item['binary_id'], item['test_name']
        if any(not isinstance(value, str) or not value for value in key):
            raise ValueError('invalid test identity')
        return key

    if any(not isinstance(data[key], list) for key in ('processes', 'tests', 'unobserved')):
        raise ValueError('invalid one-process result lists')
    processes = []
    for item in data['processes']:
        if (not isinstance(item['binary_id'], str) or not item['binary_id']
                or item['kind'] not in ('tests', 'fresh', 'alone', 'reference', 'empty') or not names(item['tests'])
                or type(item['leaked']) is not bool):
            raise ValueError('invalid process identity, kind, tests or leaked')
        for field in ('user_s', 'system_s', 'wall_s', 'start_s'):
            figure(item[field], field)
        code, sig = item['exit_code'], item['signal']
        if ((code is None) == (sig is None)
                or (code is not None and (type(code) is not int or not 0 <= code <= 255))
                or (sig is not None and (type(sig) is not int or not 1 <= sig < signal.NSIG))):
            raise ValueError('invalid process exit status')
        process = {key: item[key] for key in ('binary_id', 'tests', 'user_s', 'system_s',
                   'wall_s', 'exit_code', 'signal', 'kind', 'leaked')}
        process.update(tree=item.get('tree'), layer=item.get('layer'))
        process['cpu_s'] = figure(item['user_s'] + item['system_s'], 'cpu_s')
        processes.append(process)
    tests, failures, warnings, seen = [], [], [], set()
    for item in sorted(data['tests'], key=identity):
        key = identity(item)
        if key in seen:
            raise ValueError('duplicate selected test')
        seen.add(key)
        outcome, index = item['outcome'], item['process']
        if outcome not in ('passed', 'failed', 'crashed', 'starved', 'cancelled'):
            raise ValueError('invalid test outcome')
        if index is None:
            if data['cancelled'] is None or outcome not in ('passed', 'failed', 'starved'):
                raise ValueError('invalid test process index')
        elif (type(index) is not int or not 0 <= index < len(processes)
                or processes[index]['kind'] not in ('tests', 'fresh')
                or processes[index]['binary_id'] != key[0]):
            raise ValueError('invalid test process index')
        if outcome == 'cancelled' and data['cancelled'] is None:
            raise ValueError('cancelled test without cancellation')
        if not names(item['before']) or item['alone'] not in (None, 'passed', 'failed'):
            raise ValueError('invalid test diagnostic')
        tests.append(dict(binary_id=key[0], test_name=key[1], attempts=1,
                          cpu_s=None, user_s=None, system_s=None,
                          wall_s=figure(item['wall_s'], 'test wall_s'),
                          starved=int(outcome in ('starved', 'cancelled')), leaked=None,
                          incomplete=None, alone=item['alone'], before=item['before']))
        if outcome in ('failed', 'crashed'):
            failures.append(f'{key[0]} {key[1]}')
            if item['alone'] == 'passed':
                warnings.append(f'{pair} {key[0]} {key[1]}: may depend on an earlier test: '
                                f"it passed alone after {len(item['before'])} tests in its process")
    warnings.extend(f"{pair} {item['binary_id']}: a process left a descendant holding its output"
                    for item in processes if item['leaked'])
    unobserved = []
    single_process = False
    crashed_binaries = {t['binary_id'] for t in data['tests'] if t['outcome'] == 'crashed'}
    if data['unobserved'] and data['cancelled'] is None:
        path = directory / 'plan.json'
        single_process = path.is_file() and json.loads(path.read_text()).get('single_process', False)
    for item in sorted(data['unobserved'], key=identity):
        key = identity(item)
        if key in seen:
            raise ValueError('duplicate selected test')
        seen.add(key)
        reason = 'the run was cancelled before it'
        if data['cancelled'] is None:
            # Again runs promise one process, so a crash leaves later names
            # unobserved instead of starting the ordinary fresh process (11.9).
            if not single_process or key[0] not in crashed_binaries:
                raise ValueError('unobserved test without cancellation')
            reason = 'the again process crashed before it'
        unobserved.append(dict(binary_id=key[0], test_name=key[1], attempt=1))
        warnings.append(f'{pair} {key[0]} {key[1]}: unobserved, {reason}')
    if len(seen) != data['selected']:
        raise ValueError('selected count disagrees with test coverage')
    suite = dict(tests=len(tests), attempts=len(tests), wall_s=run['wall_s'],
                 test_wall_s=figure(sum(test['wall_s'] for test in tests), 'test_wall_s'),
                 starved=sum(test['starved'] for test in tests),
                 starved_run=run['starved'] or ('wall_hard_s' in limits and run['wall_s'] > limits['wall_hard_s']),
                 unobserved=len(unobserved), leaked=None, incomplete=None, cpu_by='process')
    suite_processes = [p for p in processes if p['kind'] in ('tests', 'fresh', 'alone')]
    for field in ('cpu_s', 'user_s', 'system_s'):
        suite[field] = figure(sum(process[field] for process in suite_processes), field)
    over = []
    if 'cpu_suite_s' in limits and suite['cpu_s'] > limits['cpu_suite_s']:
        over.append(f"suite {pair}: {suite['cpu_s']:.6f}s > {limits['cpu_suite_s']}s")
    # The layer comparison and ordering are tools/tiers/budgets.md, 4 and 8.
    head = head_binaries(processes, data.get('workspace', []), ledger)
    measured, main = {}, {}
    for process in suite_processes:
        key = process['tree'], process['layer']
        measured[key] = measured.get(key, 0.0) + process['cpu_s']
    grouped = dict(measured)
    ran = {process['binary_id'] for process in suite_processes}
    for binary, entry in head.items():
        if binary not in ran:
            key = entry['tree'], entry['layer']
            grouped[key] = grouped.get(key, 0.0) + entry['cpu_s']
    if ledger is not None:
        main = ledger_figures(ledger['binaries'])
    layer_order = {name: index for index, name in enumerate(data.get('layers', []))}

    def row_order(key):
        tree, layer = key
        tree_index = GROUPS.index(tree) if tree in GROUPS else len(GROUPS)
        if layer is None:
            order = len(layer_order) + 2
        elif layer == 'checks':
            order = len(layer_order) + 1
        else:
            order = layer_order.get(layer, len(layer_order))
        return tree_index, tree or '', order, layer or ''

    ceilings = gate_ceilings(gate_limits)
    binaries = fold_binaries(data, head, tests, unobserved, ledger)
    held = held_budgets(gate_limits, ledger, ledger_figures(binaries))
    layers, trees, layer_over = [], {}, []
    for tree, layer in sorted(grouped.keys() | main.keys(), key=row_order):
        key = tree, layer
        table = gate_limits.get(tree)
        budget = (table or {}).get(layer, {})
        ceiling = ceilings.get(key)
        state = held.get(key, {})
        limit = state.get('budget_s')
        main_limit = main_budget(ledger, key)
        cpu = grouped.get(key, 0.0)
        main_cpu = main.get(key, 0.0) if ledger is not None else None
        main_over = main_limit is not None and main_cpu > main_limit
        is_over = layer is not None and (cpu > limit if limit is not None else table is not None)
        if main_over and is_over and cpu - main_cpu <= max(main_cpu / 20, 1):
            is_over = False
            warnings.append(f'layer over budget at main: {tree} {pair} {layer}: '
                            f'{main_cpu:.2f} CPU s at main, budget {main_limit} s; {cpu:.2f} CPU s at the head')
        grew = growing_binaries(key, head, tests, ledger) if is_over and ledger is not None else []
        layers.append(dict(tree=tree, layer=layer, cpu_s=cpu, measured_s=measured.get(key, 0.0),
                           main_s=main_cpu, ceiling_s=ceiling, budget_s=limit, main_budget_s=main_limit,
                           lowest_s=state.get('lowest_s'), main_over=main_over, grew=grew,
                           ideal_s=budget.get('ideal_s'),
                           reference_s=budget.get('reference_s', gate_limits.get('references', {}).get(layer)),
                           over=is_over))
        if tree not in trees:
            total = None if table is None else sum(
                value for (group, _), value in ceilings.items() if group == tree)
            held_total = None if table is None else sum(
                value['budget_s'] for (group, _), value in held.items() if group == tree)
            trees[tree] = dict(tree=tree, cpu_s=0.0, main_s=None if ledger is None else 0.0,
                               ceiling_s=total, budget_s=held_total, ideal_s=(table or {}).get('ideal_s'))
        trees[tree]['cpu_s'] += cpu
        if ledger is not None:
            trees[tree]['main_s'] += main_cpu
        if is_over:
            label = 'no budget' if limit is None else f'budget {limit} s'
            line = f'layer over budget: {tree} {pair} {layer}: {cpu:.2f} CPU s, {label}'
            if ledger is not None:
                if ceiling is not None:
                    line += f', ceiling {ceiling} s'
                line += f', at main {main_cpu:.2f} CPU s'
                line += growth_lines(grew)
            layer_over.append(line)
    warnings.extend(f'unplaced: {binary} has no layer (tools/tiers/layers.md)'
                    for binary in sorted({p['binary_id'] for p in processes if p['layer'] is None}))
    (over if gate else warnings).extend(layer_over)
    references = reference_rows(data, gate_limits, ledger, base, warnings, over if gate else warnings)
    over = [line + '\n' + REPAIR_LINE for line in over]
    verdict = ('over budget' if over else 'starved' if suite['starved_run'] or suite['starved']
               else 'within budget' if limits or (gate and gate_limits) else 'no budget set')
    return dict(pair=pair, tests=tests, suite=suite, processes=processes, failures=failures,
                layers=layers, trees=list(trees.values()), references=references,
                unobserved=unobserved, warnings=warnings, over_budget=over, verdict=verdict)


def reference_values(profile, tree, layer, binary, name):
    """A test's reference and reason's line (tools/tiers/references.md, 6)."""
    reference = (profile.get(tree, {}).get(layer, {}).get(
        'reference_s', profile.get('references', {}).get(layer)) if layer is not None else None)
    line = next((line['cpu_s'] for line in profile.get('reasons', [])
                 if (line['binary'], line['test']) == (binary, name)), None)
    return reference, line


def reference_limit(profile, tree, layer, binary, name, why, main_cpu, running=False):
    """One rule for running and judging a test (tools/tiers/references.md, 4)."""
    reference, line = reference_values(profile, tree, layer, binary, name)
    limit = line if line is not None else reference
    if limit is None:
        return None
    if why == 'new':
        return limit
    if main_cpu is not None:
        return max(limit, 1.5 * main_cpu)
    return limit if running else line


def reference_runs(entry):
    """Read a run alone, including the older single-run form (references.md, 4)."""
    return entry.get('runs', [entry])


def reference_rows(data, profile, ledger, base, warnings, over):
    """Each test's reference and figure at main (tools/tiers/references.md, 6)."""
    at_base = {}
    if base is not None:
        base_data = json.loads((Path(base) / 'results.json').read_text())
        if 'references' not in base_data:
            raise ValueError('base results require references')
        at_base = {(entry['binary_id'], entry['test_name']): entry
                   for entry in base_data['references']}
    placements = {entry['binary_id']: (entry.get('tree'), entry.get('layer'))
                  for entry in data.get('workspace', [])}
    for process in data['processes']:
        if process['kind'] in ('tests', 'fresh', 'alone'):
            placements[process['binary_id']] = process.get('tree'), process.get('layer')
    walls = {(entry['binary_id'], entry['test_name']): entry['wall_s'] for entry in data['tests']}
    rows = []
    for entry in data.get('references', []):
        why = entry['why']
        if why not in ('new', 'flagged'):
            continue
        binary, name = key = entry['binary_id'], entry['test_name']
        tree, layer = placements.get(binary, (None, None))
        reference, line = reference_values(profile, tree, layer, binary, name)
        if reference is None and line is None:
            continue
        old = (ledger or {}).get('binaries', {}).get(binary, {})
        main_cpu, main_from, main_wall = None, None, None
        main_alone = []
        if why == 'flagged':
            history = old.get('tests', {}).get(name, [])
            main_wall = statistics.median(history) if history else None
            base_entry = at_base.get(key, {})
            main_cpu = base_entry.get('cpu_s')
            if main_cpu is not None:
                main_from = 'base'
                main_alone = [run['cpu_s'] for run in reference_runs(base_entry)]
            else:
                history = old.get('alone', {}).get(name, [])
                if history:
                    main_cpu, main_from = statistics.median(history), 'ledger'
        limit = reference_limit(profile, tree, layer, binary, name, why, main_cpu)
        cpu = entry['cpu_s']
        is_over = cpu is not None and limit is not None and cpu > limit
        label = f'{binary} {name}'
        if cpu is None:
            warnings.append(f"test not measured alone: {label}: {entry['outcome']}")
        elif is_over:
            over.append(f'test over its reference: {label}: {cpu:.3f} CPU s alone, limit {limit:.3f} s')
        elif why == 'flagged' and line is None and main_cpu is None and cpu > reference:
            warnings.append(f'test flagged without its figure at main: {label}: '
                            f'{cpu:.3f} CPU s alone, reference {reference:.3f} s')
        runs = reference_runs(entry)
        alone = [run['cpu_s'] for run in runs]
        if cpu is not None:
            for run in runs:
                if run['cpu_s'] is None:
                    warnings.append(f"test not measured alone: {label}: {run['outcome']}")
        milliseconds = [round(value * 1000) for value in alone if value is not None]
        swinging = (len(alone) == len(milliseconds) == 3
                    and max(milliseconds) * 2 > min(milliseconds) * 3)
        if swinging:
            figures = ', '.join(f'{value:.3f}' for value in alone)
            warnings.append(f'test swings alone at the head: {label}: {figures} CPU s alone')
        rows.append(dict(binary_id=binary, test_name=name, tree=tree, layer=layer, why=why,
                         wall_s=walls.get(key), main_wall_s=main_wall, cpu_s=cpu,
                         main_cpu_s=main_cpu, main_from=main_from, reference_s=reference,
                         line_s=line, limit_s=limit, over=is_over,
                         alone=alone, main_alone=main_alone, swinging=swinging))
    return rows


def binary_median(entry):
    """A binary's figure at main (tools/tiers/budgets.md, 8)."""
    return statistics.median(entry['cpu'])


def ledger_figures(binaries):
    """Layer figures by the ledger's placements (tools/tiers/budgets.md, 9)."""
    figures = {}
    for entry in binaries.values():
        key = entry['tree'], entry['layer']
        figures[key] = figures.get(key, 0.0) + binary_median(entry)
    return figures


def head_binaries(processes, workspace, ledger):
    """The head's whole figures, including unlisted targets (budgets.md, 8)."""
    head = {}
    for process in processes:
        if process['kind'] not in ('tests', 'fresh', 'alone'):
            continue
        binary = process['binary_id']
        if binary not in head:
            head[binary] = dict(tree=process['tree'], layer=process['layer'], cpu_s=0.0)
        head[binary]['cpu_s'] += process['cpu_s']
    if ledger is not None:
        for entry in workspace:
            binary = entry['binary_id']
            if binary not in head and not entry['listed'] and binary in ledger['binaries']:
                head[binary] = dict(tree=entry['tree'], layer=entry['layer'],
                                    cpu_s=binary_median(ledger['binaries'][binary]))
    return head


def main_budget(ledger, key):
    entry = (ledger or {}).get('budgets', {}).get(key[0], {}).get(key[1])
    if entry is None:
        return None
    lowest = entry['lowest_s']
    return min(entry['ceiling_s'], math.ceil(round(lowest + max(lowest / 10, 1), 3)))


def held_budgets(profile, ledger, figures):
    """Held ceilings and their next low figures (tools/tiers/budgets.md, 9)."""
    def moves(items):
        return Counter((m['from'], m['to'], m['cpu_s'], m['why']) for m in items)

    net = {}
    if ledger is not None:
        new = moves(profile.get('moves', [])) - moves(ledger['moves'])
        for (source, target, seconds, _), count in new.items():
            for endpoint, sign in ((source, -1), (target, 1)):
                key = tuple(endpoint.split('.'))
                net[key] = net.get(key, 0) + sign * seconds * count
    held = {}
    for key, ceiling in gate_ceilings(profile).items():
        entry = (ledger or {}).get('budgets', {}).get(key[0], {}).get(key[1])
        shift = net.get(key, 0)
        cpu = figures.get(key, 0.0)
        if entry is not None and round(ceiling - shift, 3) == round(entry['ceiling_s'], 3):
            limit = min(ceiling, main_budget(ledger, key) + shift)
            lowest = max(0, min(entry['lowest_s'] + shift, cpu))
        else:
            limit, lowest = ceiling, cpu
        held[key] = dict(ceiling_s=ceiling, budget_s=limit, lowest_s=lowest)
    return held


def growing_binaries(key, head, tests, ledger):
    """Uncut causes, ordered by CPU growth then wall growth (budgets.md, 8)."""
    by_binary = {}
    for test in tests:
        by_binary.setdefault(test['binary_id'], []).append(test)
    grew = []
    for binary, entry in head.items():
        if (entry['tree'], entry['layer']) != key:
            continue
        old = ledger['binaries'].get(binary)
        main = binary_median(old) if old is not None and (old['tree'], old['layer']) == key else None
        if entry['cpu_s'] <= (main or 0):
            continue
        walls = (old or {}).get('tests', {})
        grown_tests = []
        for test in by_binary.get(binary, []):
            name = test['test_name']
            median = statistics.median(walls[name]) if name in walls else None
            if test['wall_s'] > (median or 0):
                grown_tests.append(dict(test_name=name, wall_s=test['wall_s'], main_wall_s=median))
        grown_tests.sort(key=lambda test: (-(test['wall_s'] - (test['main_wall_s'] or 0)), test['test_name']))
        grew.append(dict(binary_id=binary, cpu_s=entry['cpu_s'], main_s=main, tests=grown_tests))
    grew.sort(key=lambda entry: (-(entry['cpu_s'] - (entry['main_s'] or 0)), entry['binary_id']))
    return grew


def growth_lines(grew):
    lines = []
    for binary in grew[:10]:
        main = binary['main_s']
        detail = 'new' if main is None else f"+{binary['cpu_s'] - main:.2f} CPU s, at main {main:.2f} CPU s"
        lines.append(f"  {binary['binary_id']}: {binary['cpu_s']:.2f} CPU s, {detail}")
        for test in binary['tests'][:5]:
            median = test['main_wall_s']
            detail = 'new' if median is None else f'main median {median:.6f} wall s'
            lines.append(f"    {test['test_name']}: {test['wall_s']:.6f} wall s, {detail}")
        if len(binary['tests']) > 5:
            lines.append(f"    {len(binary['tests']) - 5} more tests")
    if len(grew) > 10:
        lines.append(f'  {len(grew) - 10} more binaries')
    return ''.join('\n' + line for line in lines)


def fold_binaries(data, head, tests, unobserved, ledger):
    """Keep whole measurements and their histories (tools/tiers/budgets.md, 7)."""
    old = (ledger or {}).get('binaries', {})
    alone = {(entry['binary_id'], entry['test_name']): entry['cpu_s']
             for entry in data.get('references', []) if entry['cpu_s'] is not None}
    by_binary = {}
    for test in tests:
        by_binary.setdefault(test['binary_id'], []).append(test)
    partial = {t['binary_id'] for t in unobserved}
    partial.update(t['binary_id'] for t in data['tests'] if t['process'] is None)
    binaries = {}
    for binary, entry in head.items():
        ran = by_binary.get(binary, [])
        if ran and binary not in partial and not any(t['starved'] for t in ran):
            history = old.get(binary, {}).get('tests', {})
            walls = {t['test_name']: [*history.get(t['test_name'], [])[-2:], t['wall_s']] for t in ran}
            history = old.get(binary, {}).get('alone', {})
            figures = {}
            for name in walls:
                cpu = alone.get((binary, name))
                if cpu is not None:
                    figures[name] = [*history.get(name, [])[-2:], cpu]
                elif name in history:
                    figures[name] = history[name]
            cpu = [*old.get(binary, {}).get('cpu', [])[-2:], entry['cpu_s']]
            binaries[binary] = dict(tree=entry['tree'], layer=entry['layer'], cpu=cpu,
                                    tests=walls, alone=figures)
        elif binary in old:
            binaries[binary] = dict(old[binary], tree=entry['tree'], layer=entry['layer'],
                                    alone=old[binary].get('alone', {}))
    # An unobserved binary may have no process and still keeps its last whole CPU.
    workspace = {entry['binary_id']: entry for entry in data.get('workspace', [])}
    for binary in partial - head.keys():
        if binary in old and binary in workspace:
            entry = workspace[binary]
            binaries[binary] = dict(old[binary], tree=entry['tree'], layer=entry['layer'],
                                    alone=old[binary].get('alone', {}))
    return binaries


def fold_ledger(directory, result, limits, ledger=None):
    """The ledger should this candidate land (tools/tiers/budgets.md, 7 and 9)."""
    data = json.loads((Path(directory) / 'results.json').read_text())
    head = head_binaries(result['processes'], data.get('workspace', []), ledger)
    binaries = fold_binaries(data, head, result['tests'], result['unobserved'], ledger)
    figures = ledger_figures(binaries)
    profile = (limits or {}).get('gate', {}).get(result['pair'], {})
    folded_budgets = {}
    for (tree, layer), state in held_budgets(profile, ledger, figures).items():
        folded_budgets.setdefault(tree, {})[layer] = {key: state[key] for key in ('ceiling_s', 'lowest_s')}
    return dict(ledger=3, profile=result['pair'], binaries=binaries, budgets=folded_budgets,
                moves=profile.get('moves', []))


def tools_summary(directory, limits, expected):
    if not expected or any(not isinstance(suite, str) or not suite for suite in expected):
        raise ValueError("tools coverage requires at least one --expect ID")
    items = records(directory, tools=True)
    actual = {(r['profile'], r['binary_id'], r['test_name'], r['attempt']) for r in items}
    wanted = {('tools', suite, 'suite', 1) for suite in expected}
    if actual != wanted:
        raise ValueError(f"record coverage: missing {sorted(wanted - actual)}; unexpected {sorted(actual - wanted)}")
    if any(not r['complete'] or r['pending'] or r.get('settling', False) for r in items):
        raise ValueError("incomplete tools suite record")
    failures = [f"{r['binary_id']} suite" for r in items
                if r['exit_code'] != 0 and not (r['starved'] and r['signal'] in (signal.SIGTERM, signal.SIGKILL))]
    timeouts = {(r['profile'], r['binary_id'], r['test_name'], r['attempt'])
                for r in items if r['starved']}
    run = dict(starved=False, wall_s=sum(r['wall_s'] for r in items))
    return summarize_records(items, 'tools', limits, run, timeouts, failures)


def summarize_records(items, pair, limits, run, timeouts, failures):
    limits = (limits or {}).get(pair, {})
    wall_starved = run['starved'] or (pair != 'tools' and 'wall_hard_s' in limits and run['wall_s'] > limits['wall_hard_s'])
    tests, over = [], []
    totals = dict(cpu_s=0.0, user_s=0.0, system_s=0.0, test_wall_s=0.0,
                  starved=0, leaked=0, incomplete=0, attempts=len(items))
    groups = {}
    for item in items:
        key = (pair, item['binary_id'], item['test_name'], item['attempt'])
        if item['pending'] and key not in timeouts and not run['starved']:
            raise ValueError(f"unfinished attempt without starvation evidence: {key}")
        starved = key in timeouts or (run['starved'] and item['pending'])
        if not item['complete'] and not starved and not run['starved']:
            raise ValueError(f'incomplete descendant CPU accounting: {key}')
        test = groups.setdefault(key[1:3], dict(binary_id=key[1], test_name=key[2], attempts=0,
                                              cpu_s=0.0, user_s=0.0, system_s=0.0, wall_s=0.0,
                                              starved=0, leaked=0, incomplete=0))
        cpu = item['user_s'] + item['system_s']
        for target in (totals, test):
            target['cpu_s'] += cpu
            target['user_s'] += item['user_s']
            target['system_s'] += item['system_s']
            target['starved'] += starved
            target['leaked'] += item['leaked']
            target['incomplete'] += not item['complete']
        wall = item['wall_s']
        if item['pending'] or item.get('settling', False):
            start = min(r['start_s'] for r in items)
            wall = max(wall, start + run['wall_s'] - item['start_s'])
        totals['test_wall_s'] += wall
        test['wall_s'] += wall
        test['attempts'] += 1
    tests = [groups[key] for key in sorted(groups)]
    for test in tests:
        test_limits = limits.get(test['binary_id'], {}) if pair == 'tools' else limits
        if 'cpu_test_s' in test_limits and test['cpu_s'] > test_limits['cpu_test_s']:
            over.append(f"test {test['binary_id']} {test['test_name']}: {test['cpu_s']:.6f}s > {test_limits['cpu_test_s']}s")
        if pair == 'tools':
            if 'cpu_suite_s' in test_limits and test['cpu_s'] > test_limits['cpu_suite_s']:
                over.append(f"suite {test['binary_id']}: {test['cpu_s']:.6f}s > {test_limits['cpu_suite_s']}s")
            if 'wall_hard_s' in test_limits and test['wall_s'] > test_limits['wall_hard_s']:
                wall_starved = True
    if pair != 'tools' and 'cpu_suite_s' in limits and totals['cpu_s'] > limits['cpu_suite_s']:
        over.append(f"suite {pair}: {totals['cpu_s']:.6f}s > {limits['cpu_suite_s']}s")
    warnings = [f"{pair} {name}: auto-reaped descendant CPU may be absent from complete_scope"
                for name in limits.get('auto_reap', [])] if pair != 'tools' else []
    over = [line + '\n' + REPAIR_LINE for line in over]
    return dict(pair=pair, tests=tests, warnings=warnings, suite=dict(totals, tests=len(tests), wall_s=run['wall_s'],
                starved_run=wall_starved, unobserved=0, cpu_by='test'), processes=[], layers=[], trees=[], references=[], failures=failures, unobserved=[], over_budget=over,
                verdict="over budget" if over else "starved" if wall_starved or timeouts else "within budget" if limits else "no budget set")


def print_summary(result, as_json=False):
    if as_json:
        print(json.dumps(result, allow_nan=False))
        return
    for test in result['tests']:
        cpu = 'not measured' if test['cpu_s'] is None else f"{test['cpu_s']:.6f}s"
        print(f"{test['binary_id']} {test['test_name']}: CPU {cpu} "
              f"wall {test['wall_s']:.6f}s attempts {test['attempts']} "
              f"starved {test['starved']} leaked {test['leaked']} incomplete {test['incomplete']}")
    for process in result['processes']:
        status = f"exit {process['exit_code']}" if process['signal'] is None else f"signal {process['signal']}"
        print(f"{process['binary_id']} process {process['kind']}: CPU {process['cpu_s']:.6f}s "
              f"(user {process['user_s']:.6f}s system {process['system_s']:.6f}s) "
              f"wall {process['wall_s']:.6f}s tests {len(process['tests'])} {status}")
    suite = result['suite']
    print(f"{result['pair']}: {suite['tests']} tests, {suite['attempts']} attempts; "
          f"CPU {suite['cpu_s']:.6f}s (user {suite['user_s']:.6f}s system {suite['system_s']:.6f}s), "
          f"wall {suite['wall_s']:.6f}s; starved {suite['starved']} "
          f"starved_run {suite['starved_run']} unobserved {suite['unobserved']} "
          f"leaked {suite['leaked']} incomplete {suite['incomplete']}; {result['verdict']}")
    for kind, rows in (('layer', result['layers']), ('tree', result['trees'])):
        for row in rows:
            label = f"{row['tree']} {row['layer']}" if kind == 'layer' else row['tree']
            figures = ', '.join(f"{key.removesuffix('_s')} {row[key] if row[key] is not None else 'none'} s"
                                for key in ('cpu_s', 'ceiling_s', 'budget_s', 'ideal_s', *(['reference_s'] if kind == 'layer' else []),
                                            *(['main_s'] if row['main_s'] is not None else [])))
            print(f'{kind} {label}: {figures}')
    for row in result['references']:
        figures = ', '.join(f"{key.removesuffix('_s')} {row[key] if row[key] is not None else 'none'} s"
                            for key in ('cpu_s', 'reference_s', 'line_s', 'main_cpu_s', 'limit_s'))
        print(f"reference {row['binary_id']} {row['test_name']} ({row['why']}): {figures}; over {row['over']}")
    for message in result['warnings']:
        print(f'warning: {message}')
    for message in result['over_budget']:
        print(message)


def execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml'), walls=None, gate=False, ledger=None):
    """Run and report one process per binary (tools/tiers/design.md, 10.2)."""
    if 'cpu_test_s' in limits.get(profile, {}):
        raise ValueError('one-process runs cannot measure cpu_test_s')
    runpy.run_path(str(ONE_PROCESS))['run'](command, env, directory, profile, limits, config, walls=walls, ledger=ledger)
    result = summary(directory, limits, profile, gate=gate, ledger=ledger)
    save(Path(directory) / 'ledger.json', fold_ledger(directory, result, limits, ledger))
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        print_summary(result)
    text = output.getvalue()
    print(text, end='')
    with (Path(directory) / 'command-output.txt').open('a') as log:
        log.write(text)
    if result['over_budget']:
        return 1
    if result['failures']:
        return 100
    if result['suite']['starved_run'] or result['suite']['starved']:
        return 0
    if not result['suite']['tests']:
        data = json.loads((Path(directory) / 'results.json').read_text())
        if data['selected'] == 0 and data['no_tests'] == 'fail':
            return 4
    return 0


def execute(command, env, directory, profile, limits):
    """Supervise one command in its own session (tools/tiers/design.md, 7.3)."""
    directory = Path(directory).resolve()
    starved = False
    first = None
    current = []
    output = bytearray()
    kill_at = None
    process = subprocess.Popen(command, env=env, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        with process.stdout, selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            while selector.get_map():
                for key, _ in selector.select(0.05):
                    data = os.read(key.fileobj.fileno(), 65536)
                    if not data:
                        selector.unregister(key.fileobj)
                        continue
                    sys.stdout.buffer.write(data)
                    sys.stdout.buffer.flush()
                    output.extend(data)
                hard = limits.get(profile, {}).get('wall_hard_s')
                if first is None or hard:
                    current = records(directory)
                    if current and first is None:
                        first = min(item['start_s'] for item in current)
                if hard and any(item['pending'] or item.get('settling', False) for item in current) and first is not None and time.monotonic() - first > hard and not starved and process.poll() is None:
                    starved = True
                    save(directory / 'starved.json', {'wall_hard_s': hard})
                    print(f"tiers: starved: {profile} exceeded wall hard limit {hard}s", flush=True)
                    signal_group(process, signal.SIGTERM)
                    kill_at = time.monotonic() + GRACE_S
                if kill_at is not None and time.monotonic() >= kill_at:
                    break
        if kill_at is not None:
            stop(process, kill_at)
    except BaseException:
        # Collector faults and caller interruptions release the command's session.
        signal_group(process, signal.SIGTERM)
        try:
            stop(process, time.monotonic() + GRACE_S)
        except BaseException:
            stop(process, time.monotonic())
        raise
    code = process.wait()
    (directory / 'command-output.txt').write_bytes(output)
    current = records(directory)
    end = time.monotonic() if starved else max((item['start_s'] + item['wall_s'] for item in current), default=first or 0.0)
    save(directory / 'run.json', dict(profile=profile, finished=True, starved=starved,
                                     wall_s=0.0 if first is None else end - first,
                                     returncode=code, cancel_code=None))
    result = summary(directory, limits, profile)
    print_summary(result)
    if result['over_budget']:
        return 1
    if result['failures']:
        return 100
    if starved:
        return 0
    if code == 100 and (result['suite']['starved'] or result['suite']['starved_run']):
        return 0
    return code
