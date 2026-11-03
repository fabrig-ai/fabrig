# D033: Tests write out whole `Limits` and `Config` literals instead of one constructor

- **Kind:** tests
- **Where:** tests and worlds across skein, for `skein_io::Limits`, `skein_json::tokenizer::Limits`, oauth's `ClientLimits` and the components' `Config`s
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, fixtures) and `reviews/2026-10-31-skein-areas.md` (A0, F11)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** M
- **Status:** open
- **Paid when:** each widely used `Limits` or `Config` has one test constructor (as `skein_sim::Config::calm` is), and tests change only the fields they test (`skein/docs/foundation/testing-strategy.md`, 10: a setup written once).
- **Measure:** `rg -c 'io::Limits \{' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`: 62 at base

## What

`io::Limits {` is written out 62 times in 45 files,
`tokenizer::Limits {` 60 times in 37 files, oauth's `ClientLimits {` 30
times in 10 files, and 61 files define a local `fn limits()`. Rows
also carry bare numbers (the forge machine world's memory rows of 15
integers, `skein/tests/forge/machine/tests/suite/memory.rs`).

## Why it matters

A new limit field touches every literal, and a reader cannot tell which
field a test means to set.

## What pays it back

A `calm` (or `for_tests`) constructor per type where the type lives,
then the literals replaced by struct update from it. Labelled rows
where a row holds numbers.
