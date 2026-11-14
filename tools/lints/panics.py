#!/usr/bin/env python3
"""Count catches against a falling list: tools/lints/design.md, 7. Run at the repository root."""
import argparse
import os
from pathlib import Path
import re
import subprocess

PATTERNS = ("catch_unwind", "resume_unwind", "should_panic")
WORDS = tuple(re.compile(r"\b" + name + r"\b") for name in PATTERNS[:2])
ATTRIBUTE = re.compile(r"#\[\s*should_panic\b")
LIST = Path("tools/lints/panics.txt")
RULE = "skein/docs/foundation/programming-model.md, 10.4"
HEADER = ("# tools/lints/design.md, 7: the count of catches.\n"
          "# --write lowers this list; nothing raises it.\n")


def order(key):
    path, pattern = key
    return path, PATTERNS.index(pattern)


def counts():
    tracked = subprocess.run(("git", "ls-files", "-z"), check=True, capture_output=True).stdout
    current = {}
    for path in sorted(set(os.fsdecode(p) for p in tracked.split(b"\0") if p.endswith(b".rs"))):
        try:
            text = Path(path).read_text(encoding="utf-8", errors="replace")
        except FileNotFoundError:
            continue
        if not any(name in text for name in PATTERNS):
            continue
        totals = [0, 0, 0]
        for line in text.splitlines():
            for index, word in enumerate(WORDS):
                totals[index] += bool(word.search(line))
            if not line.lstrip().startswith("//"):
                totals[2] += len(ATTRIBUTE.findall(line))
        for pattern, count in zip(PATTERNS, totals):
            if count:
                current[path, pattern] = count
    return current


def read_list(lines):
    limits = {}
    findings = []
    previous = None
    for number, line in enumerate(lines, 1):
        if line.startswith("#"):
            continue
        fields = line.removesuffix("\n").rsplit(" ", 2)
        if len(fields) == 3:
            path, pattern, count = fields
            valid = (path.endswith(".rs") and not path.startswith("/")
                     and not any(part in ("", ".", "..") for part in path.split("/"))
                     and not any(char in path for char in "\t\r\n")
                     and pattern in PATTERNS and re.fullmatch(r"[0-9]+", count)
                     and int(count) >= 1)
        else:
            valid = False
        if not valid:
            findings.append(f"{LIST}: line {number}: expected PATH PATTERN COUNT (count at least 1)")
            continue
        key = path, pattern
        if key in limits:
            findings.append(f"{LIST}: line {number}: repeated {path} {pattern}")
        elif previous is not None and order(key) < order(previous):
            findings.append(f"{LIST}: line {number}: {path} {pattern} is out of order")
        limits[key] = int(count)
        previous = key
    return limits, findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true", help="initialize or lower the list")
    args = parser.parse_args()
    current = counts()
    try:
        lines = LIST.read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)
    except FileNotFoundError:
        if args.write:
            lines = HEADER.splitlines(keepends=True) + [
                f"{path} {pattern} {current[path, pattern]}\n"
                for path, pattern in sorted(current, key=order)
            ]
            limits, findings = current, []
        else:
            lines, limits = [], {}
            findings = [f"{LIST}: no list; initialize with --write"]
    else:
        limits, findings = read_list(lines)
        for key in sorted(current, key=order):
            count, limit = current[key], limits.get(key, 0)
            if count > limit:
                path, pattern = key
                findings.append(f"{path}: {pattern} count {count} exceeds list {limit}")
    if args.write and not findings:
        lowered = []
        for line in lines:
            if line.startswith("#"):
                lowered.append(line)
            else:
                path, pattern, _ = line.removesuffix("\n").rsplit(" ", 2)
                count = current.get((path, pattern), 0)
                if count:
                    lowered.append(f"{path} {pattern} {count}\n")
        LIST.write_text("".join(lowered), encoding="utf-8")
    elif not args.write:
        for key in sorted(limits, key=order):
            count, limit = current.get(key, 0), limits[key]
            if count < limit:
                path, pattern = key
                print(f"{path}: {pattern} count {count}, list {limit}; lower with --write")
    for finding in findings:
        print(finding)
    print(f"{len(findings)} findings ({RULE})")
    return bool(findings)


if __name__ == "__main__":
    raise SystemExit(main())
