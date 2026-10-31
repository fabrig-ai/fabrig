+++
id = "lib-fake-forge"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/src/rates.rs", "skein/crates/skein-fake-forge-github/src/pages.rs", "skein/crates/skein-fake-forge-protocol/src/server.rs", "skein/crates/skein-fake-forge-domain/src/actions.rs", "skein/crates/skein-fake-forge-domain/src/promises.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# The fake forge on lib's clear, hex and decimal

## Goal

Draft, 3.4: the fake forge's servers empty their queues with pop loops,
and its domain and GitHub face write decimal and hex digits by hand;
they use lib's, by `notes/lib.md`.

Acceptance:

- Pop loops become `clear()`: `skein-fake-forge-protocol/src/server.rs`
  (6, from ~384) and `skein-fake-forge-github/src/server.rs` (2).
- `skein-fake-forge-github/src/rates.rs:206` (`fn decimal(value) ->
  Box<[u8]>`) and `skein-fake-forge-domain/src/actions.rs:278` (`fn
  decimal(out, number)`) go; their callers take `Decimal::of(n).as_bytes()`.
- `skein-fake-forge-domain/src/promises.rs:920` (`fn hex(nibble)`) goes
  for `hex::digit`.
- `skein-fake-forge-github/src/pages.rs:542` (`fn hex(rng, length)`)
  draws random digits: it keeps its name and work and maps each drawn
  nibble with `hex::digit`.
- `pop_loop` falls by 8, `hex_decimal` by 3 (`--write`).
- `heavy cargo nextest run -p skein-fake-forge-protocol -p
  skein-fake-forge-github -p skein-fake-forge-domain -p
  skein-fake-forge-tests -p skein-fake-peers-tests` passes (counts
  named); clippy clean for the three crates.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::{hex, Decimal, Queue::clear}` on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`; callers of the
  two `decimal`s elsewhere in their crates change only at the call; name
  the files.
- `server.rs`'s `resume` functions are the fake's own protocol pass, not
  io's: not `io_resume` sites, not yours. The flat `Answer` and
  `Request` (the review's theme 3) are later runs.

## Steps

1. Each site by `lib.md`; `pages::hex`'s draws must stay the same digits
   for the same seed (a test may pin them).
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
