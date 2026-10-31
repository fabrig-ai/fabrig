+++
id = "echo-stage"
part = "echo-first"
kind = "migration"
needs = ["kit-io", "counts"]
files = ["skein/examples/echo/service/src/lib.rs", "skein/examples/echo/service/src/io_stage.rs", "skein/testing/skein-echo-client/src/lib.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++
# The echo service and client on `skein_io::stage`

## Goal

Draft, 3.2: the first root to run end to end on the contract. The echo
example's service and its client harness each write io's stage by hand;
both call `skein_io::stage::{up, down}` instead, and the echo world's
stories stay green.

Acceptance:

- `skein/examples/echo/service/src/io_stage.rs` and its `mod io_stage;`
  are gone; the service's `io_up` / `io_down` / `io_room` become one
  `skein_io::stage::up` and one `skein_io::stage::down` call at their
  call sites (lib.rs ~299, ~313).
- The client's `io_up` / `io_down` / `io_room` in
  `skein/testing/skein-echo-client/src/lib.rs` (~359 on) do the same.
- `io_resume` falls by 2, to 37 (`--write`).
- `heavy cargo nextest run -p skein-echo-world -p skein-echo-service -p
  skein-echo-client -p skein-echo-shell` passes; name the story tests
  and the count.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md` and `notes/stage.md`
  (its "What every task does", item 1 only, and its defaults).
- `skein/docs/design/examples.md` (the echo example), for the words.
- `skein_io::stage` (`skein/crates/skein-io/src/stage.rs`, from
  `kit-io`): its contracts.

## Owns and may change

- The files in the header.
- The service's size bound at lib.rs ~84-88 (the largest `MAX_OUT_*`)
  stays as it is.
- Not here: echo's hosts' deadlines, its test dispatchers and referees.
  `echo-surfaces` moves them after `kit-shell` lands; leave
  `next_deadline` and the `tests/echo` files as they are.

## Steps

1. Service: replace `io_up` and `io_down` with the stage calls; delete
   `io_stage.rs` and the private room helper.
2. Client: the same.
3. `--write` from `skein/`; checks of `common.md` for the four
   packages; commit; push.

## Report

The sites moved (path:line), `io_resume` before and after, the tests run
with counts.
