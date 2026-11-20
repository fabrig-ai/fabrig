"""Stream gate progress and publish a bounded verdict for readers of the job log."""
import os
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from collections import deque
from pathlib import Path

DIAGNOSTICS = Path("gate-diagnostics.log")
SUMMARY = Path("gate-result.txt")
STEPS = {"metadata": "Metadata", "clippy": "Clippy", "build": "Build", "default": "Focused tests",
         "fuzzy": "Fuzzy tests", "clippy-wasm32": "Wasm clippy", "docs": "Docs"}
STEP = re.compile(r"gate: ([\w-]+): (green|RED), exit -?\d+, ([\d.]+) s")
ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
MAX_CASES = 5


def short(text, limit=220):
    text = " ".join(ANSI.sub("", text).split())
    return text if len(text) <= limit else text[:limit - 1] + "…"


def duration(seconds):
    minutes, seconds = divmod(round(seconds), 60)
    return f"{minutes}m {seconds:02d}s" if minutes else f"{seconds}s"


def junit_failures(path):
    """The existing report carries the case name, reason, and sometimes assertion/compiler text."""
    try:
        report = ET.parse(path)
    except (OSError, ET.ParseError):
        return []
    rows = []
    for case in report.iter("testcase"):
        for tag in ("failure", "error"):
            issue = case.find(tag)
            if issue is None:
                continue
            key = ".".join(filter(None, (case.get("classname"), case.get("name"))))
            reason = short(issue.get("message") or tag)
            details = [short(line) for line in (issue.text or "").splitlines() if line.strip()]
            # Include the assertion and its values, or the compiler error and source location.
            start = next((i for i, line in enumerate(details) if
                          re.search(r"panicked at|assertion|expected|^error(\[|:)|^fatal:", line, re.I)), None)
            excerpt = details[start:start + 4] if start is not None else details[-2:]
            label = "FAIL" if tag == "failure" else "ERROR"
            rows.append(f"{label}  {short(key, 140)} — {reason}")
            if excerpt:
                rows[-1] += "\n      " + short(" ".join(excerpt), 320)
    return rows


def run(command):
    start = time.monotonic()
    steps, scopes, warnings = set(), set(), set()
    candidate, base = os.environ.get("CANDIDATE", ""), os.environ.get("BASE", "")
    if candidate:
        print(f"Gate · candidate {short(candidate[:12])} · base {short(base[:12]) if base else 'unspecified'}",
              flush=True)
    evidence, wall_runs = [], []
    budget_example = None
    skipped = set()
    tail = deque(maxlen=3)
    with DIAGNOSTICS.open("wb") as log, subprocess.Popen(
            command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as gate:
        for raw in gate.stdout:
            log.write(raw)
            line = ANSI.sub("", raw.decode(errors="replace")).strip()
            if not line:
                continue
            tail.append(short(line))
            if line.startswith(("gate: selection: ", "gate: by test: ", "quick pass:", "ok: staging passthrough",
                                "no-code:", "citations:")) and line not in scopes:
                scopes.add(line)
                print(short(line.removeprefix("gate: ")), flush=True)
            step = STEP.fullmatch(line)
            if step and step[1] not in steps:
                steps.add(step[1])
                print(f"{'PASS' if step[2] == 'green' else 'FAIL'}  "
                      f"{STEPS.get(step[1], step[1]):14} {duration(float(step[3]))}", flush=True)
            skip = re.fullmatch(r"gate: ([\w-]+): skipped, (.+)", line)
            if skip and skip[1] not in skipped:
                skipped.add(skip[1])
                print(f"SKIP  {STEPS.get(skip[1], skip[1]):14} {short(skip[2])}", flush=True)
            if re.match(r"gate: [\w-]+: (warning: |main's red, a warning)", line):
                warnings.add(line)
                if budget_example is None and "budget" in line:
                    budget_example = short(line.removeprefix("gate: "))
            if re.match(r"gate: (default|fuzzy) \((head|again)\): .*past its wall limit", line):
                if line not in wall_runs:
                    wall_runs.append(line)
            if (line.startswith(("FAULT:", "Missing ", "unchecked save:", "gate: repair evidence:"))
                    or re.match(r"[^:]+:[0-9]+: ", line)
                    or re.match(r"gate: (default|fuzzy): RED,", line)):
                bounded = short(line.removeprefix("gate: "))
                if len(evidence) < MAX_CASES and bounded not in evidence:
                    evidence.append(bounded)
        code = gate.wait()
    footer = []
    if warnings:
        footer.append(f"WARN  {len(warnings)} distinct warnings · details in gate-diagnostics")
        if budget_example:
            footer.append(f"      {budget_example}")
    if code:
        failures = junit_failures(Path("junit.xml"))
        footer.extend(failures[:MAX_CASES])
        if len(failures) > MAX_CASES:
            footer.append(f"      {len(failures) - MAX_CASES} more cases · full report in junit")
        footer.extend(short(line.removeprefix("gate: ")) for line in wall_runs[:4])
        if not failures:
            footer.extend(f"      {line}" for line in (evidence or tail))
    verdict = "FAIL" if code else "PASS"
    count = f" · {len(steps)} {'check' if len(steps) == 1 else 'checks'}" if steps else ""
    footer.append(f"{verdict}{count} · {duration(time.monotonic() - start)}")
    # Forgejo uploads the artifacts first, then echoes this file into the tail read by Temper.
    SUMMARY.write_text("\n".join(footer) + "\n")
    return code if code >= 0 else 128 - code


if __name__ == "__main__":
    sys.exit(run(sys.argv[1:]))
