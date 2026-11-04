# temper's tech debt

What temper's code and tests owe, one file per item. The first pass,
2026-11-03 at `41d63da8c3`, filled it from the sources that existed then,
each item checked against main.

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001`, are
  temper's own, and are never reused.
- **The branch that pays an item deletes its file,** and its row here;
  git keeps the history.
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

| ID | title | kind | value | size | status |
|---|---|---|---|---|---|
| D001 | temper-service carries a drifted copy of jig's reference service | duplication | high | M | open |
| D007 | `temper-docs` keeps a page kind no page has | dead code | low | S | open |
| D008 | Code names the old pipeline's tasks and rulings | naming | low | S | open |
| D010 | Test fixtures are shared by `#[path]` across crates | tests | low | S | open |
| D011 | `clippy::manual_map` is excused site by site, 127 times | lints | low | S | open |
| D012 | The forge's domain and route files, and nine more, are too large to read | complexity | medium | L | open |
