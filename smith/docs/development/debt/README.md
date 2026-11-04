# smith's tech debt

What smith's code and tests owe, one file per item. An item says what is
there today, why it matters, and what pays it back.

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001` and are
  never reused.
- **The branch that pays an item deletes its file,** and its row below.
  Git keeps the history.
- **An item we choose to keep** stays, with status `kept` and its reason,
  so a later review does not find it again.
- **Reviews** that fed these items are kept raw in `reviews/`, named
  `YYYY-MM-DD-<scope>.md`. They are dated records, not contracts.

## What counts as debt

What the code or tests owe: dead or duplicate code, coupling, missing or
hidden primitives, needless complexity, naming, hand-copied lints, test
infrastructure copied around, workarounds left in place, files too large
to read.

Not here:

- **Where the code lags or departs from its design:** that is the
  document's status file (`smith/docs/design/STATUS.md` lists them). An
  item that is also a gap covers the code's side and names the status
  file.
- **Bugs,** and **features not built yet:** the backlog.
- **Mechanical forms the ratchet already counts** (`smith/scripts/old-forms.txt`),
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

Paths in **Where** are relative to `smith/`; a path that starts with a
crate's name is under `smith/crates/`. **Measure** is given only where a
count tracks the debt. **Paid when** is always given: for qualitative debt
it says what a reviewer checks.

## Index

| ID | title | kind | value | size | status |
|---|---|---|---|---|---|
| [D001](D001-events-hand-kept-records.md) | smith-events keeps every record in four hand-written hierarchies | duplication | medium | L | open |
| [D003](D003-host-io-route-table.md) | The local host routes io completions through its own table | primitives | low | M | open |
| [D005](D005-block-walks.md) | Every walk of a message destructures `Block` again | duplication | low | S | open |
| [D006](D006-backend-two-effect-paths.md) | The program backend's effects leave by two paths | primitives | low | S | open |
| [D007](D007-tools-namespace.md) | The tools serve a namespace no run produces | complexity | low | M | open |
| [D008](D008-one-program-map.md) | The agent shell maps endpoints to a one-variant program name | complexity | low | S | open |
| [D011](D011-bench-string-errors.md) | The benchmarks carry failures as strings | other | low | M | open |
| [D012](D012-large-domain-files.md) | `session.rs` and `run.rs` are too large to read | complexity | low | M | open |
| [D013](D013-names-left.md) | Names the 10-31 review flagged remain | naming | low | S | open |
| [D015](D015-path-modules.md) | Code is shared between targets by `#[path]` | tests | low | M | open |
| [D017](D017-domain-worlds-on-protocols.md) | Two domain worlds call protocol crates | coupling | low | S | open |
| [D018](D018-domain-formats-text.md) | The local domain composes person-facing text | coupling | low | S | open |
