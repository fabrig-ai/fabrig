#!/usr/bin/env python3
"""Check or copy lint definitions: tools/lints/design.md, 5. Run at the repository root."""
import argparse
from itertools import zip_longest
from pathlib import Path
import re
import subprocess
import tomllib

TREES = ("skein", "jig", "temper", "hearth")
HERE = Path(__file__).resolve().parent
LINT = re.compile(r"^\[lints(?:\.[^\]]+)?\]")


def block(lines) -> tuple[int, int] | None:
    """The first [lints table's line and the line past its last, trailing comments and blanks out."""
    start = next((i for i, line in enumerate(lines) if LINT.match(line)), None)
    if start is None:
        return None
    end = next((i for i in range(start + 1, len(lines))
                if lines[i].startswith("[") and not LINT.match(lines[i])), len(lines))
    while end > start and (not lines[end - 1].strip() or lines[end - 1].lstrip().startswith("#")):
        end -= 1
    return start, end


def profiles() -> dict[str, str]:
    return {p.stem: p.read_text().rstrip() for p in sorted((HERE / "profiles").glob("*.toml"))}


def variants() -> dict[str, str]:
    return {p.stem: p.read_text().rstrip() for p in sorted((HERE / "clippy").glob("*.toml"))}


def selected(path, prefixes):
    return path.parts[0] in TREES and any(path.is_relative_to(p) for p in prefixes)


def packages(prefixes) -> list[Path]:
    members = tomllib.loads(Path("Cargo.toml").read_text())["workspace"]["members"]
    return sorted({Path(m) / "Cargo.toml" for m in members
                   if selected(Path(m) / "Cargo.toml", prefixes)})


def clippy_files(prefixes) -> list[Path]:
    tracked = subprocess.run(("git", "ls-files", "-z"), check=True, capture_output=True, text=True).stdout
    return sorted(Path(p) for p in tracked.split("\0")
                  if p and Path(p).name == "clippy.toml" and selected(Path(p), prefixes))


def name(text):
    first = text.splitlines()[0] if text else ""
    if first == "[lints]":
        return "step"
    match = re.search(r"(?:# |clippy variant: )(\w+)(?::| \()", first)
    return match[1] if match else None


def difference(text, definitions, kind):
    if text.rstrip() in definitions.values():
        return None
    key = name(text)
    if key not in definitions:
        return f"matches no {kind}; line 1: {text.splitlines()[0] if text else '<end>'!r} (unknown name)"
    for i, (got, expected) in enumerate(zip_longest(text.rstrip().splitlines(),
                                                   definitions[key].splitlines(), fillvalue="<end>"), 1):
        if got != expected:
            return f"matches no {kind}; {key} line {i}: {got!r}, expected {expected!r}"
    return None


def check(prefixes) -> list[str]:
    findings = []
    for path in packages(prefixes):
        lines = path.read_text().splitlines(keepends=True)
        bounds = block(lines)
        problem = "no block"
        if bounds is not None:
            start, end = bounds
            headers = [line.split("#", 1)[0].strip() for line in lines[start:end] if LINT.match(line)]
            problem = ("a second block" if block(lines[end:]) is not None or len(headers) != len(set(headers))
                       else difference("".join(lines[start:end]), profiles(), "profile"))
        if problem:
            findings.append(f"{path}: {problem}")
    for path in clippy_files(prefixes):
        problem = difference(path.read_text(), variants(), "variant")
        if problem:
            findings.append(f"{path}: {problem}")
    return findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--set", choices=profiles())
    mode.add_argument("--write", action="store_true")
    parser.add_argument("paths", nargs="*")
    args = parser.parse_args()
    if args.set and not args.paths:
        parser.error("--set needs PATH...")
    prefixes = [Path(p) for p in args.paths or TREES]
    if args.set or args.write:
        for path in packages(prefixes):
            lines = path.read_text().splitlines(keepends=True)
            start, end = block(lines) or (len(lines), len(lines))
            key = args.set or name("".join(lines[start:end]))
            if key not in profiles():
                parser.error(f"{path}: block names no profile")
            while args.set and start and lines[start - 1].lstrip().startswith("#"):
                start -= 1
            before = "".join(lines[:start])
            path.write_text(before + ("\n" if before and not before.endswith("\n") else "")
                            + profiles()[key] + "\n" + "".join(lines[end:]))
        if args.write:
            for path in clippy_files(prefixes):
                key = name(path.read_text())
                if key not in variants():
                    parser.error(f"{path}: file names no variant")
                path.write_text(variants()[key] + "\n")
    findings = check(prefixes)
    for finding in findings:
        print(finding)
    print(f"{len(findings)} findings")
    return bool(findings)


if __name__ == "__main__":
    raise SystemExit(main())
