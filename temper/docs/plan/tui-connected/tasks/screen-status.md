+++
id = "screen-status"
part = "screen"
kind = "docs"
needs = ["screen-surface", "screen-render", "screen-roles", "screen-keys", "screen-terminal", "fail-stop-restore", "sim-window"]
files = [
  "skein/docs/design/ui-screen.status.md",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/simulator.status.md",
]
effort = "medium"
minutes = 15
+++

# skein's statuses say what the full-screen shell's tasks built

## Goal

The screen part's tasks ran side by side, so none closed
`skein/docs/design/ui-screen.status.md`'s entries but `screen-surface`
and `fail-stop-restore`. Once all have merged, this task reads the trunk
and leaves in each status file only what is still not built.

This task departs from the implementer's text in one way: the three
status files are yours, and nothing else is.

Acceptance:

1. `skein/docs/design/ui-screen.status.md` holds a **Not yet** entry only
   for what `skein/docs/design/ui-screen.md` designs and the trunk's code
   does not do, each named by section, and a **Departs** entry, with its
   reason, for what the code does otherwise (each task's report on the
   trunk's history says what it chose: `git log` and the reports' commits).
   With no gap, the file says "No gap." after its first line, as
   `ui-text.status.md` does. Its date is the day you write it.
2. `skein/docs/design/shell.status.md` and `simulator.status.md` hold no
   entry for 6.3's restoration at a panic and 3.2's window, if those are
   built; if a task left one there, it is removed; nothing else changes.
3. `python3 -I -B tools/citations/citations.py --base origin/main` lists
   nothing new.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `DESIGN.md`, section 2 (status files).
- `skein/docs/design/ui-screen.md`, all of it; `shell.md`, 6.3;
  `simulator.md`, 3.2.

## Owns and may change

**Owned:** the three status files. Code is not yours: a gap you find is a
**Not yet** entry, never a fix.

## Steps

1. Read each section of `ui-screen.md` against the crates
   `skein-ui-screen` and `skein-ui-screen-terminal` and `tests/ui-screen`
   on the trunk (names, behaviour, tests), by `rg` and reading, never by
   building.
2. Write the entries (acceptance 1 and 2).
3. The citations' check. Commit ("skein: the full-screen shell's statuses
   say what is built"), push.

## Report

As the implementer's text says, and each entry left, with the reason it is
still open.
