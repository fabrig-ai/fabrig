# jig's debt

What jig's code and tests owe, one file per item. A branch that pays an item deletes its
file; there is no row to delete (git keeps the history). An item we choose to keep stays, with
status `kept` and its reason, so a later review does not find it again. Reviews add items;
their reports are kept raw in `reviews/`, each file named by its date and scope.

## 1. What counts

Debt is what the code or tests owe: dead or duplicate code, coupling, missing or hidden
primitives, needless complexity, naming, hand-copied lints, test infrastructure copied
around, workarounds left in place, files too large to read. Not here:

- **design gaps**, where the code lags or departs from a design document: they go in that
  document's `.status.md`. An item that is also a gap covers the code's side and cites
  the status file;
- **bugs** (wrong behaviour), **features not built yet** and QA's off-path findings: they
  go to their owner or to the backlog.

## 2. An item

One file per item, named by its ID and a slug (`D007-contract-written-by-hand.md`).
IDs run from `D001` and are never reused:

```md
# D007: <title, a short claim>

- **Kind:** dead code | duplication | coupling | primitives | complexity | naming | tests | lints | other
- **Where:** <paths>
- **Seen:** <date> at `<commit>`, in <source, with its own label> (one line per source)
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

- **The highest ID used is D044.** The branch that adds an item raises it, so a paid ID isn't handed out again.
- **Paths** in **Where** are under `jig/`; one that starts with a crate's name is under
  `jig/crates/`. A **Measure** runs at the repository's root.
- **Sources** named in **Seen**: jig's tech-debt list (first look 2026-10-10, refreshed
  2026-10-18 at `86971be`; `reviews/2026-10-18-jig-tech-debt.md`); the jig code-quality
  review (2026-10-31 at `c30cf6a03`; `reviews/2026-10-31-jig.md`), cited by its own labels
  (themes T1 to T10, area findings such as A1 F3, its order-of-work items); and the
  quality run's debt list (2026-10-31 at `1b5c98834`; `reviews/2026-10-31-jig-quality-run.md`),
  which carried out the review's first phases.

## 3. Index

`python3 -I -B tools/debt/debt.py jig` prints it, from the repository's root, from each item file's
title, kind, value, size and status; with no tree, all five (`tools/debt/README.md`).
