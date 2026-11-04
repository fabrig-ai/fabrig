# hearth's tech debt

What hearth's code and tests owe, one file per item. An item says what is
there today, why it matters, and what pays it back.

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001` and are
  never reused.
- **The highest ID used is D013.** The branch that adds an item raises it, so a paid ID isn't handed out again.
- **The branch that pays an item deletes its file;** there is no row to delete.
  Git keeps the history.
- **An item we choose to keep** stays, with status `kept` and its reason,
  so a later review does not find it again.
- **Reviews** that feed these items are kept raw in `reviews/`, named
  `YYYY-MM-DD-<scope>.md`. They are dated records, not contracts.

## What counts as debt

What the code or tests owe: dead or duplicate code, coupling, missing or
hidden primitives, needless complexity, naming, hand-copied lints, test
infrastructure copied around, workarounds left in place, files too large
to read.

Not here:

- **Where the code lags or departs from its design:** that is
  `hearth/docs/design/STATUS.md`. An item that is also a gap covers the
  code's side and names the status file.
- **Bugs,** and **features not built yet:** the backlog.
- **Mechanical forms the ratchet already counts** (`hearth/scripts/old-forms.txt`),
  unless paying them needs more than a sweep.

## The entry format

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** `crate-name/src/path.rs`, `crate-name`
- **Seen:** <date> at `<commit>`, in <source, with section> (one line per source)
- **Checked:** <date> at `<commit>`
- **Value:** high | medium | low (what paying it buys)
- **Size:** S (one task) | M (one plan of a few tasks) | L (several plans)
- **Status:** open | planned (<plan>) | kept (<reason>)
- **Paid when:** <a concrete, checkable condition>
- **Measure:** <optional: a command run at the repository's root, and its value>

## What

## Why it matters

## What pays it back
```

Paths in **Where** are relative to `hearth/`; a path that starts with a
crate's name is under `hearth/crates/`. **Measure** is given only where a
count tracks the debt. **Paid when** is always given: for qualitative debt
it says what a reviewer checks.

## Index

`python3 -I -B tools/debt/debt.py hearth` prints it, from the repository's root, from each item file's
title, kind, value, size and status; with no tree, all five (`tools/debt/README.md`).
