#!/bin/sh
# The tiers the gate leaves out that main must keep green (agents'
# roles/owners.md, 4.5): the process tier, full QA and the browser tier, each
# independently on this checkout; the outer runner owns queueing and cleanup.
# Nothing runs this until the user designs how it should run.
set -eu
umask 077
cd -P "$(dirname "$0")/.."
exec python3 -I -B - <<'PY'
import datetime
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import time
import xml.etree.ElementTree as ET


def emit(event, **fields):
    print("after-landing " + json.dumps(dict(event=event, **fields), sort_keys=True), flush=True)


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


statuses = {}
measurements = []
failed_tests = {}
unavailable = {}


def phase(name, kind, command):
    start = stamp()
    clock = time.monotonic()
    emit("phase-start", phase=name, kind=kind, start=start)
    try:
        code = subprocess.call(command)
    except OSError as error:
        print(f"{name}: {error}", flush=True)
        code = 127
    statuses[name] = code
    result = dict(phase=name, kind=kind, start=start, end=stamp(),
                  command_wall_seconds=round(time.monotonic() - clock, 3), exit_code=code)
    measurements.append(result)
    emit("phase-end", **result)
    return code == 0


def blocked(name):
    statuses[name] = "blocked"
    emit("phase-blocked", phase=name, reason="failed prerequisite")


def tests(name, profile, command):
    result = target / "nextest" / profile / "after-landing.xml"
    # Never attribute a previous run's failures to a build/spawn failure.
    try:
        result.unlink(missing_ok=True)
    except OSError as error:
        statuses[name] = 1
        unavailable[name] = "could not clear stale JUnit"
        emit("junit-unavailable", phase=name, reason=str(error))
        return
    phase(name, "test", command)
    try:
        report = ET.parse(result)
        names = set()
        for suite in report.iter("testsuite"):
            for test in suite.findall("testcase"):
                if test.find("failure") is not None or test.find("error") is not None:
                    binary = test.get("classname") or suite.get("name")
                    if not binary or not test.get("name"):
                        raise ValueError("JUnit failure lacks binary/test name")
                    names.add(f"{binary}::{test.get('name')}")
        if names:
            failed_tests[name] = sorted(names)
        if statuses[name] != 0 and not names:
            unavailable[name] = "completed JUnit contains no failed test names"
    except (OSError, ET.ParseError, ValueError) as error:
        unavailable[name] = "missing or invalid completed JUnit"
        emit("junit-unavailable", phase=name, reason=str(error))


def finish(process_ok, qa_ok, browser_ok):
    code = 0 if process_ok and qa_ok and browser_ok else 1
    if code:
        failures = {name: status for name, status in statuses.items() if status != 0}
        details = dict(sha=sha, failed_phases=failures, failed_tests=failed_tests,
                       unavailable_names={name: unavailable.get(name, "not executed" if status == "blocked"
                                          else "no completed JUnit for this phase")
                                          for name, status in failures.items()
                                          if not failed_tests.get(name)})
        workspace = os.environ.get("DEV_TMP", "")
        if workspace and Path(workspace).is_absolute():
            inbox = Path(workspace) / "director/inbox/after-landing.md"
            details["log"] = None
            try:
                common = Path(subprocess.check_output(["git", "rev-parse", "--git-common-dir"], text=True).strip())
                repo = common.resolve().parent.name
                short = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], text=True).strip()
                details["log"] = str(Path(workspace) / "after-landing" / f"{repo}-{short}.log")
                inbox.parent.mkdir(parents=True, exist_ok=True)
                with inbox.open("a", encoding="utf-8") as output:
                    output.write(stamp() + " after-landing details: " + json.dumps(details, sort_keys=True) + "\n")
            except (OSError, subprocess.SubprocessError) as error:
                statuses["failure-report"] = 1
                emit("report-failed", reason=str(error), inbox=str(inbox))
        else:
            details["log"] = None
            emit("report-unavailable", reason="DEV_TMP absent or not absolute; stdout only, no inbox written")
        emit("failure-summary", **details)
    emit("run-end", sha=sha, host=socket.gethostname(), start=started, end=stamp(),
         process_exit_code=0 if process_ok else 1, qa_exit_code=0 if qa_ok else 1,
         browser_exit_code=0 if browser_ok else 1,
         phases=statuses, exit_code=code)
    raise SystemExit(code)


root = Path.cwd()
sha = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
started = stamp()
emit("run-start", sha=sha, host=socket.gethostname(), start=started)
# The bundle builder and outer cleanup use target/ too. Ignore an inherited
# external target without inspecting or touching it; all outputs belong here.
target = root / "target"
if target.is_symlink() or (target.exists() and not target.is_dir()):
    statuses["setup"] = 1
    emit("setup-failed", reason="after-landing requires the worktree's physical target directory")
    finish(False, False, False)
os.environ.update(CARGO_TARGET_DIR=str(target), TEMPER_BUNDLE="none",
                  TEMPER_TEST_BUNDLE=str(target / "temper-bundle"),
                  TEMPER_TEST_CLAUDE=str(target / "debug/fake-claude"),
                  NEXTEST_DOUBLE_SPAWN="0", NEXTEST_USER_CONFIG_FILE="none")
tool_config = ["--tool-config-file", f"after-landing:{root / '.config/nextest-after-landing.toml'}"]

programs = ["cargo", "build", "-p", "temper", "--bin", "temper",
            "-p", "temper-fake-claude", "--bin", "fake-claude"]
# Keep these process commands aligned with proof-one-process.sh. Separate
# build/discovery from command wall time; nextest still reports execution time.
# --no-run rejects execution-only flags; add them only to test invocations.
process = ["cargo", "nextest", "run", "-p", "temper", "--profile", "process"] + tool_config
qa = ["cargo", "nextest", "run", "--workspace", "--profile", "qa"] + tool_config
phase("process-lab", "test", ["python3", "-I", "-B", "-m", "unittest", "discover",
                              "-s", "lab/tests/process", "-v"])
programs_ok = phase("process-programs", "build", programs)
if programs_ok and phase("process-build", "build-discovery", process + ["--no-run"]):
    tests("process-tests", "process", process + ["--no-fail-fast", "--failure-output", "immediate-final"])
else:
    if not programs_ok:
        blocked("process-build")
    blocked("process-tests")
process_ok = all(code == 0 for code in statuses.values())
emit("process-end", end=stamp(), exit_code=0 if process_ok else 1)

# This entire interval is the QA extension, including prep and build. Join its
# timestamps to the outer heavy receipt/TSV; raw queue wait is not added wait.
extension = stamp()
clock = time.monotonic()
emit("qa-extension-start", start=extension)
bundle_ok = phase("qa-bundle", "build", ["scripts/bundle.sh", "--loopback", "target/temper-bundle"])
if not programs_ok:
    programs_ok = phase("qa-programs", "build", programs)
# Use the same designated local-owned Forgejo/runner as the process tier.
# The existing fixtures own pin validation and scratch services; never fetch here.
chromium = os.environ.get("SKEIN_TEST_CHROMIUM") or shutil.which("chromium")
required = [chromium, os.environ.get("TEMPER_FORGEJO"), os.environ.get("TEMPER_FORGEJO_RUNNER")]
environment_ok = all(path and Path(path).is_file() and os.access(path, os.X_OK) for path in required)
statuses["qa-environment"] = 0 if environment_ok else 1
emit("qa-environment", end=stamp(), exit_code=statuses["qa-environment"],
     requirement="Chromium and designated pinned TEMPER_FORGEJO/TEMPER_FORGEJO_RUNNER executables")
if bundle_ok and programs_ok and environment_ok:
    if phase("qa-build", "build-discovery", qa + ["--no-run"]):
        tests("qa-tests", "qa", qa + ["--no-fail-fast", "--retries", "0", "--failure-output", "immediate-final"])
    else:
        blocked("qa-tests")
else:
    blocked("qa-build")
    blocked("qa-tests")
qa_ok = all(code == 0 for name, code in statuses.items() if name.startswith("qa-"))
emit("qa-extension-end", start=extension, end=stamp(),
     command_wall_seconds=round(time.monotonic() - clock, 3), exit_code=0 if qa_ok else 1)

# The browser tier (client/browser.md, 4.3), outside the gate: Chromium
# against temper serve on the QA extension's bundle, as scripts/browser.sh runs it.
browser = ["cargo", "nextest", "run", "--workspace", "--profile", "browser"] + tool_config
if bundle_ok and programs_ok:
    tests("browser-tests", "browser", browser + ["--no-fail-fast", "--failure-output", "immediate-final"])
else:
    blocked("browser-tests")
browser_ok = statuses["browser-tests"] == 0
# Repeat timings at the end so the outer heavy tail retains them too.
for result in measurements:
    emit("phase-summary", **result)
finish(process_ok, qa_ok, browser_ok)
PY
