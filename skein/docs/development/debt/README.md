# skein's tech debt

What skein's code and tests owe, one file per item. Read it with
`skein/docs/foundation/programming-model.md` and
`skein/docs/foundation/testing-strategy.md`, whose rules most items
measure the code against.

## 1. The rule

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001`, are
  skein's own, and are never reused.
- **The highest ID used is D050.** The branch that adds an item raises it, so a paid ID isn't handed out again.
- **The branch that pays an item deletes its file;** there is no row to
  delete, and git keeps the history.
- **An item we choose to keep** stays, with status `kept` and its
  reason, so a later review does not find it again.
- **Dated review reports** are kept raw under `reviews/`, as
  `YYYY-MM-DD-<scope>.md`. Items cite them by file; reviews add items
  here.

## 2. What counts

Debt is what the code or tests owe: dead or duplicate code, coupling,
missing or hidden primitives, needless complexity, naming, hand-copied
lints, test infrastructure copied around, workarounds left in place,
files too large to read. Not here:

- **a design gap** (the code lags or departs from a design document):
  it goes in that document's status file. When an item is also a
  departure, the item covers the code's side and cites the design;
- **a bug** (wrong behaviour): it is fixed, or tracked where bugs are;
- **a feature not built yet.**

## 3. An entry

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** <paths; one that starts with a crate's name is under skein/crates/, others are from the repository root>
- **Seen:** <date> at `<commit>`, in <source, with its part> (one line per source)
- **Checked:** <date> at `<commit>`
- **Value:** high | medium | low (what paying it buys)
- **Size:** S (one task) | M (one plan of a few tasks) | L (several plans)
- **Status:** open | planned (<plan>) | kept (<reason>)
- **Paid when:** <a concrete, checkable condition>
- **Measure:** <optional: a command run at the repository's root, and its value when checked>

## What

## Why it matters

## What pays it back
```

A **Measure** is given only where a count tracks the debt; it runs at
the repository's root. **Paid when** is always given; for qualitative
debt it says what a reviewer checks.

## 4. Index

`python3 -I -B tools/debt/debt.py skein` prints it, from the repository's root, from each item file's
title, kind, value, size and status; with no tree, all five (`tools/debt/README.md`).
