#!/usr/bin/env python3
"""Print debt indexes from item files and check each README's highest ID.

Usage, from the repository root:

    python3 -I -B tools/debt/debt.py [TREE]

TREE is skein, smith, jig, temper or hearth; omitted, print all five.
Exit 0 indexes printed, 1 missing fields or invalid highest ID, 2 usage.
"""
from pathlib import Path
import re
import sys

TREES = ("skein", "smith", "jig", "temper", "hearth")
FIELDS = ("kind", "value", "size", "status")


def main(args):
    trees = args or TREES
    if len(args) > 1 or any(tree not in TREES for tree in trees) or any(
            not Path(tree, "docs/development/debt").is_dir() for tree in trees):
        print("usage: python3 -I -B tools/debt/debt.py [TREE]"
              " (skein smith jig temper hearth)", file=sys.stderr)
        return 2
    errors, indexes = [], []
    for tree in trees:
        directory = Path(tree, "docs/development/debt")
        items = sorted((path for path in directory.glob("D*-*.md")
                        if re.fullmatch(r"D[0-9]+-.*\.md", path.name)),
                       key=lambda path: int(path.name.split("-", 1)[0][1:]))
        rows = []
        for path in items:
            identity = path.name.split("-", 1)[0]
            header = re.split(r"^## ", path.read_text(), maxsplit=1, flags=re.M)[0]
            title = re.fullmatch(r"# " + identity + r": (.*)", header.split("\n", 1)[0])
            values = {"title": title[1].strip() if title else ""}
            for field in FIELDS:
                found = re.search(r"^- \*\*" + field.title() + r":\*\*([^\n]*)", header, re.M)
                values[field] = found[1].split(" (", 1)[0].strip() if found else ""
            for field, value in values.items():
                if not value:
                    errors.append(f"{path}: no {field}")
            rows.append("| " + " | ".join((identity, *values.values())) + " |")
        readme = directory / "README.md"
        note = re.search(r"^- \*\*The highest ID used is (D[0-9]+)\.\*\*(?: .*)?$",
                         readme.read_text() if readme.is_file() else "", re.M)
        if not note:
            errors.append(f"{readme}: no highest ID")
        elif items and int(note[1][1:]) < int(items[-1].name.split("-", 1)[0][1:]):
            highest = items[-1].name.split("-", 1)[0]
            errors.append(f"{readme}: highest ID {note[1]} is below {highest}")
        indexes.append("\n".join((f"## {tree}", "", "| ID | title | kind | value | size | status |",
                                  "|---|---|---|---|---|---|", *rows)))
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("\n\n".join(indexes))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
