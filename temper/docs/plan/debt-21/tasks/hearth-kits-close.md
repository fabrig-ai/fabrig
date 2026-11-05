+++
id = "hearth-kits-close"
part = "hearth-kits"
kind = "docs"
needs = ["hearth-hostile", "hearth-kit-git", "hearth-builder-switch"]
files = ["hearth/docs/development/debt/D009-worlds-as-kits.md", "hearth/docs/design/domain/testing.status.md"]
effort = "medium"
minutes = 10
+++

# hearth: D009 goes

## Goal

Every edge has gone, so the item goes, with its status entry.

Acceptance:

- The item's **Measure** reads 0, and the note's fuller search lists nothing. If either lists a line, that's a `Question:`.
- `hearth/docs/development/debt/D009-worlds-as-kits.md` is deleted.
- `hearth/docs/design/domain/testing.status.md`'s **Not yet** entry for 2.9 goes whole. That's the one edit this task makes to a status file, which departs from the implementer's text for that entry alone.
- `python3 -I -B tools/citations/citations.py --all` reads 0, and `git grep -n 'D009-worlds-as-kits' -- hearth` finds nothing.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`

## Owns and may change

- **Owns:** the entry and the item. Nothing is built: `.config/gate/read-by-tests.txt` lists no hearth design file.

## Steps

1. The searches; the deletion; the entry.
2. Citations after you commit. Commit and push.

## Report

The common note's items; the two searches' results.
