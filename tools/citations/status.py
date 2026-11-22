#!/usr/bin/env python3
"""Roll up committed status files, or print a tree's fixed status page.

Usage, from the repository root:

    python3 -I -B tools/citations/status.py [--head REV] [TREE...]
    python3 -I -B tools/citations/status.py --page TREE

The readings, output and page are fixed by tools/citations/README.md,
"The status roll-up and the page". Nothing is written. Exit 0 when printed,
2 for a usage error or git failure.
"""
import argparse
import io
import os
import re
import subprocess
import sys
import tarfile

TREES = ("temper", "smith", "skein", "jig", "hearth")
LABELS = ("Not yet", "Departs", "Design lags", "Design departs", "Open")
MARK = 49_152
LABEL = re.compile(r"^[ \t]*- \*\*(" + "|".join(LABELS) + r"):\*\*", re.MULTILINE)
# The checker's section-list vocabulary (citations.py, status_label_matches).
NUM = r"\d+(?:\.\d+)*"
SEP = r"(?:\s*,\s*(?:and\s+|or\s+)?|\s+and\s+|\s+or\s+|\s+to\s+|\s*[–-]\s*)"
SECTION = re.compile(r"(?:\A[ \t]*|^[ \t]*(?:- )?|(?<=\. )[ \t]*)"
                     r"(" + NUM + r"(?:" + SEP + NUM + r")*)\s*:(?!\d)", re.MULTILINE)
# The tag ends the entry, as in tools/close/close.py's TAG.
TAG = re.compile(r"\s\[in flight: [^\]\r\n]*\][.,;:!?]*[ \t\r]*$")
PAGE = """# Status

The status of each of TREE's design documents and drafts is beside it,
as `<name>.status.md`: where the code lags or departs from a design
document (**Not yet**, **Departs**), and where the design lags or
departs from a draft (**Design lags**, **Design departs**, **Open**),
as `DESIGN.md`, 2, says. A gap goes in its document's status file,
never on this page.

To roll them up, from the repository root:

    python3 -I -B tools/citations/status.py TREE

It prints the open entries by label, a line for each status file with
an open entry, and what's due: the files over 48 KB, largest first, the
drafts whose status records no gap, and the status files it can't read.

This page is fixed: `python3 -I -B tools/citations/status.py --page TREE`
prints it, and the citation checker refuses any other text
(`DESIGN.md`, 4).
"""


def page(tree: str) -> str:
    """The fixed page, usable by the citation checker without a repository."""
    if tree not in TREES:
        raise ValueError(f"unknown tree: {tree}")
    return PAGE.replace("TREE", tree)


def is_draft(path):
    """Archive statuses are also draft statuses; both kinds accept all labels."""
    return path.split("/", 1)[1].startswith("docs/design/draft/")


def read_status(text):
    """Return (date, counts, in flight, done), or None for an unread file."""
    lines = text.splitlines()
    while lines and not lines[0].strip():
        lines.pop(0)
    if not lines or not lines.pop(0).startswith("# "):
        return None
    while lines and not lines[0].strip():
        lines.pop(0)
    if not lines or not (date := re.match(r"\d{4}-\d{2}-\d{2}\b", lines[0])):
        return None
    while lines and lines[0].strip():
        lines.pop(0)
    body = "\n".join(lines).strip()
    labels = list(LABEL.finditer(body))
    counts = dict.fromkeys(LABELS, 0)
    in_flight = 0
    if not labels:
        return (date[0], counts, 0, True) if body in ("", "No gap.") else None
    for index, label in enumerate(labels):
        end = labels[index + 1].start() if index + 1 < len(labels) else len(body)
        text = body[label.end():end]
        if text.strip().lower().removesuffix(".") == "none":
            continue
        sections = list(SECTION.finditer(text))
        entries = [text[section.end():sections[i + 1].start()
                        if i + 1 < len(sections) else len(text)].rstrip()
                   for i, section in enumerate(sections)] if sections else [text.rstrip()]
        counts[label[1]] += len(entries)
        if label[1] == "Not yet":
            in_flight += sum(TAG.search(entry) is not None for entry in entries)
    return date[0], counts, in_flight, not any(counts.values())


def committed_markdown(rev, trees):
    """Read only selected trees' design Markdown, through git archive in memory."""
    names = subprocess.run(("git", "ls-tree", "-rz", "--name-only", rev),
                           check=True, capture_output=True).stdout.split(b"\0")
    paths = {os.fsdecode(name) for name in names if name.endswith(b".md")
             and any(name.startswith(f"{tree}/docs/design/".encode()) for tree in trees)}
    if not paths:
        return {}
    patterns = [f":(glob){tree}/docs/design/**/*.md" for tree in trees
                if any(path.startswith(f"{tree}/") for path in paths)]
    archive = subprocess.run(("git", "archive", "--format=tar", rev, "--", *patterns),
                             check=True, capture_output=True).stdout
    with tarfile.open(fileobj=io.BytesIO(archive)) as contents:
        return {member.name: contents.extractfile(member).read()
                for member in contents if member.isfile() and member.name in paths}


def counts_line(counts, in_flight, all_labels=False):
    return ", ".join(f"{counts[label]} {label}"
                     + (f" ({in_flight} in flight)"
                        if label == "Not yet" and (all_labels or in_flight) else "")
                     for label in LABELS if all_labels or counts[label])


def roll_up(tree, files):
    totals = dict.fromkeys(LABELS, 0)
    in_flight = 0
    opened, oversized, done, unread = [], [], [], []
    for path in sorted(files, key=os.fsencode):
        if not path.startswith(f"{tree}/docs/design/"):
            continue
        data = files[path]
        if len(data) > MARK:
            oversized.append((path, len(data)))
        if not path.endswith(".status.md"):
            continue
        status = read_status(data.decode("utf-8", errors="replace"))
        if status is None:
            unread.append(path)
            continue
        date, counts, flying, finished = status
        for label in LABELS:
            totals[label] += counts[label]
        in_flight += flying
        if any(counts.values()):
            opened.append(f"{path}, {date}: {counts_line(counts, flying)}")
        if (finished and is_draft(path)
                and not path.startswith(f"{tree}/docs/design/draft/archive/")
                and not path.removesuffix(".status.md").endswith((".notes", ".execution"))):
            done.append(path.removesuffix(".status.md") + ".md")
    oversized.sort(key=lambda item: (-item[1], os.fsencode(item[0])))
    sections = [("Status files with an open entry:", opened),
                (f"Over the mark, {MARK:,} bytes, largest first:",
                 [f"{path}: {size:,} bytes" for path, size in oversized]),
                ("Drafts done, outside draft/archive/:", sorted(done, key=os.fsencode)),
                ("Unread status files:", unread)]
    lines = [f"# {tree}", "", f"Open entries: {counts_line(totals, in_flight, True)}."]
    for heading, items in sections:
        lines.extend(["", heading, *(f"- {item}" for item in items or ["none"])])
    return "\n".join(lines) + "\n"


def main(argv) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--page", action="store_true")
    parser.add_argument("trees", nargs="*", choices=TREES)
    try:
        args = parser.parse_args(argv)
        if args.page and len(args.trees) != 1:
            parser.error("--page needs exactly one tree")
    except SystemExit as error:
        return error.code
    if args.page:
        print(page(args.trees[0]), end="")
        return 0
    trees = args.trees or TREES
    try:
        files = committed_markdown(args.head, trees)
    except (subprocess.CalledProcessError, OSError, tarfile.TarError) as error:
        detail = getattr(error, "stderr", None) or str(error)
        if isinstance(detail, bytes):
            detail = detail.decode("utf-8", errors="replace")
        print(f"status: {detail}", file=sys.stderr)
        return 2
    print("\n".join(roll_up(tree, files) for tree in trees), end="")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
