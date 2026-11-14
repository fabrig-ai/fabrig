"""Opt-in CPU collector and evaluator (tools/tiers/design.md, 7)."""

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
import subprocess
import sys
import time
import tomllib
import xml.etree.ElementTree as ET


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


def budgets(path, pairs=None):
    with Path(path).open("rb") as source:
        data = tomllib.load(source)
    def validate(name, values, *, pair=False):
        keys = KEYS | {"auto_reap"} if pair else KEYS
        if not isinstance(values, dict) or values.keys() - keys:
            raise ValueError(f"invalid budget keys for {name}")
        for key, value in values.items():
            if key == "auto_reap":
                if not isinstance(value, list) or any(not isinstance(test, str) or not test for test in value):
                    raise ValueError(f"{name}.auto_reap must be a list of nonempty test names")
                continue
            if (type(value) not in (int, float) or not math.isfinite(value) or value <= 0):
                raise ValueError(f"{name}.{key} must be finite positive seconds")
    for pair, values in data.items():
        if pair == "tools":
            if not isinstance(values, dict):
                raise ValueError("tools budgets must be a table keyed by suite ID")
            for suite, limits in values.items():
                if not suite:
                    raise ValueError("empty tools suite ID")
                validate(f"tools.{suite}", limits)
        elif pair == "gate":
            if not isinstance(values, dict):
                raise ValueError("gate budgets must be a table keyed by tree")
            for tree, profiles in values.items():
                if not tree:
                    raise ValueError("empty gate tree name")
                if not isinstance(profiles, dict):
                    raise ValueError(f"gate.{tree} must be a table keyed by profile")
                for profile, limits in profiles.items():
                    if not profile:
                        raise ValueError(f"empty gate.{tree} profile name")
                    validate(f"gate.{tree}.{profile}", limits)
        else:
            if pairs is not None and pair not in pairs:
                raise ValueError(f"unknown budget pair {pair}")
            validate(pair, values, pair=True)
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


def summary(directory, limits=None, pair=None, expected=None):
    directory = Path(directory)
    if pair == "tools":
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
        return one_process_summary(directory, limits, pair, run)
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


def one_process_summary(directory, limits, pair, run):
    """Read process CPU and test outcomes (tools/tiers/design.md, 10.2 and 11.8)."""
    limits = (limits or {}).get(pair, {})
    if 'cpu_test_s' in limits:
        raise ValueError('one-process runs cannot measure cpu_test_s')
    data = json.loads((directory / 'results.json').read_text())
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
                or item['kind'] not in ('tests', 'fresh', 'alone') or not names(item['tests'])
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
    for field in ('cpu_s', 'user_s', 'system_s'):
        suite[field] = figure(sum(process[field] for process in processes), field)
    over = []
    if 'cpu_suite_s' in limits and suite['cpu_s'] > limits['cpu_suite_s']:
        over.append(f"suite {pair}: {suite['cpu_s']:.6f}s > {limits['cpu_suite_s']}s")
    verdict = ('over budget' if over else 'starved' if suite['starved_run'] or suite['starved']
               else 'within budget' if limits else 'no budget set')
    return dict(pair=pair, tests=tests, suite=suite, processes=processes, failures=failures,
                unobserved=unobserved, warnings=warnings, over_budget=over, verdict=verdict)


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
    return dict(pair=pair, tests=tests, warnings=warnings, suite=dict(totals, tests=len(tests), wall_s=run['wall_s'],
                starved_run=wall_starved, unobserved=0, cpu_by='test'), processes=[], failures=failures, unobserved=[], over_budget=over,
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
    for message in result['warnings']:
        print(f'warning: {message}')
    for message in result['over_budget']:
        print(message)


def execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml')):
    """Run and report one process per binary (tools/tiers/design.md, 10.2)."""
    if 'cpu_test_s' in limits.get(profile, {}):
        raise ValueError('one-process runs cannot measure cpu_test_s')
    runpy.run_path(str(ONE_PROCESS))['run'](command, env, directory, profile, limits, config)
    result = summary(directory, limits, profile)
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
