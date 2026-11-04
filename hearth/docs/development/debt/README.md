# hearth's tech debt

What hearth's code and tests owe, one file per item. An item says what is
there today, why it matters, and what pays it back.

- **One file per item,** `Dnnn-<slug>.md`. IDs start at `D001` and are
  never reused.
- **The branch that pays an item deletes its file,** and its row below.
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

| ID | title | kind | value | size | status |
|---|---|---|---|---|---|
| [D001](D001-room-checks-and-max-out.md) | The service and its machines pace work by room checks and `MAX_OUT` | complexity | high | L | open |
| [D002](D002-process-worst-case.md) | The builder sums a process-wide worst case nothing runs, in formulas under the old name | complexity | high | M | open |
| [D003](D003-memory-tests-on-formulas.md) | Memory tests hold peaks to formulas, not to ceilings | tests | medium | M | open |
| [D004](D004-service-drives-io-by-hand.md) | The service drives io and files by hand, doubling every operation token | primitives | medium | S | open |
| [D005](D005-private-hex-decimal-octal.md) | hex, decimal and octal digits are written by hand across the crates | duplication | low | S | open |
| [D006](D006-lifecycles-in-flags.md) | Lifecycles of the builder's protocol layer and store are spread over flags | complexity | medium | M | open |
| [D007](D007-two-process-fakes.md) | mmdebstrap's fake is written twice, as a fake-machine process and as a hosted one | duplication | low | S | open |
| [D008](D008-path-modules.md) | Test modules are reached by `#[path]` | tests | low | S | open |
| [D009](D009-worlds-as-kits.md) | World crates are used as kits by other test crates | tests | low | M | open |
| [D010](D010-lint-config-copied.md) | hearth's test crates and shell copy one lint block by hand | lints | medium | S | open |
| [D011](D011-ratchet-counts-two-forms.md) | hearth's ratchet counts two forms, and counts what the others exempt | other | medium | S | open |
| [D012](D012-render-test-shapes.md) | Test shapes the render run left | tests | low | S | open |
| [D013](D013-records-three-times.md) | A record's shape is written up to three times, with hand translations both ways | duplication | low | L | open |
