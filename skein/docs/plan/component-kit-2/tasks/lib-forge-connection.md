+++
id = "lib-forge-connection"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts", "abort-connection"]
files = ["skein/crates/skein-forge-connection/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# skein-forge-connection on lib's earliest and clear

## Goal

Draft, 3.4: forge-connection merges deadlines inline (the review's
"inline copies" of `earliest`) and empties queues with pop loops; it
uses `skein_lib::earliest` and `clear`, by `notes/lib.md`.

Acceptance:

- The inline earliest merges become `skein_lib::earliest`:
  `src/component.rs:545-588` (the component's next deadline, six
  merges), `src/hooks.rs:113-120`, `src/installations.rs:646` on, and any
  other `earliest.is_none() || Some(t) < earliest` merge `rg` finds in
  `src/`.
- The four pop loops become `clear()`: `src/connection.rs:343` and
  `:346`, `src/installations.rs` (one), `src/tests.rs` (one).
- `pop_loop` falls by 4 (`--write`).
- `heavy cargo nextest run -p skein-forge-connection -p
  skein-forge-machine-world -p forgejo-conformance` passes (counts
  named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::{earliest, Queue::clear}` on your base.
- `abort-connection`'s `Request::Abort` is merged on your base: keep it.

## Owns and may change

- `skein/crates/skein-forge-connection/src/**`, `skein/scripts/old-forms.txt`.
- The connection's flag-struct state (the review's theme 2) and its
  shared transport (1a) are later runs: change nothing but the merges and
  the loops.
- A merge guarded by a condition (`if c.closing != Closing::Sent && ...`)
  keeps its guard: `if guard { next = earliest(next, Some(t)); }`.

## Steps

1. Each merge and loop by `lib.md`.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`, with the merges moved (path:line).
