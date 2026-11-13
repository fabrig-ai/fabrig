# temper's tech debt

What temper's code and tests owe, one file per item. The first pass,
2026-11-03 at `41d63da8c3`, filled it from the sources that existed then,
each item checked against main.

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001`, are
  temper's own, and are never reused.
- **The highest ID used is D013.** The branch that adds an item raises it, so a paid ID isn't handed out again.
- **The branch that pays an item deletes its file;** there is no row to delete.
  Git keeps the history.
- **An item we choose to keep** stays, with status `kept` and its reason,
  so that a review does not find it again.
- **Reviews** add items; a dated review's raw report goes to `reviews/`,
  as `reviews/YYYY-MM-DD-<scope>.md`.

## What counts as debt

What the code or tests owe: dead or duplicate code, coupling, missing or
hidden primitives, needless complexity, naming, hand-copied lints, test
infrastructure copied around, workarounds left in place, files too large
to read. Not debt here:

- **a design gap,** where the code lags or departs from a design
  document: it goes to that document's `.status.md`. An item that is both
  covers the code's side and cites the status file;
- **a bug,** wrong behaviour;
- **a feature not built yet,** and QA's off-path findings: the backlog's.

## An entry

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** `crate-name/src/path.rs`, `crate-name` (paths that start with a crate's name are under `crates/`)
- **Seen:** <date> at `<commit>`, in <source, with section> (one line per source)
- **Checked:** <date> at `<commit>`
- **Value:** high | medium | low (what paying it buys: smaller, clearer, unblocks other work)
- **Size:** S (one task) | M (one plan of a few tasks) | L (several plans)
- **Status:** open | planned (<plan>) | kept (<reason>)
- **Paid when:** <a concrete, checkable condition>
- **Measure:** <optional: a command run at the repository's root, and its value when checked>

## What

<what is there, with evidence>

## Why it matters

## What pays it back

<the approach, and what it depends on or unblocks, citing other items by ID>
```

A **Measure** is given only where a count tracks the debt. **Paid when**
is always given; for debt no count tracks, it says what a reviewer
checks.

## Index

`python3 -I -B tools/debt/debt.py temper` prints it, from the repository's root, from each item file's
title, kind, value, size and status; with no tree, all five (`tools/debt/README.md`).
