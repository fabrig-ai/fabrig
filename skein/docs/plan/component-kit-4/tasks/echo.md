+++
id = "echo"
part = "echo-first"
kind = "migration"
needs = ["io-stage", "counts"]
files = ["skein/examples/echo/service/**", "skein/testing/skein-echo-client/**", "skein/tests/echo/tests/suite/scenarios.rs"]
effort = "high"
minutes = 45
+++

## Goal

The run's first root end to end on the kit (slice 1;
`skein/docs/design/draft/component-kit.md, 3.2`): the echo example's
service and its client run their io stage through `skein_io::stage`, the
service's private `io_stage.rs` is gone, and echo's story tests stay
green.

Acceptance:

- `examples/echo/service/src/io_stage.rs` is deleted; the service's
  `io_up` and `io_down` call `skein_io::stage::up` and `down`.
- `testing/skein-echo-client` calls `skein_io::stage` for its io stage.
- The `#[path]` include of the deleted file in
  `tests/echo/tests/suite/scenarios.rs` (line 249) and the test it serves
  are gone: tests/io's `tests/suite/stage.rs` covers it (name the map in
  the report).
- The `io_resume` count falls by 2; no other count rises.
- `skein-echo-service`, `skein-echo-client` and `skein-echo-world` pass
  their tests, the end-to-end story (`tests/echo/tests/end_to_end.rs`)
  included.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and `notes/migrations.md`
  (the io stage).
- `skein/docs/design/draft/component-kit.md, 3.2`; `skein/docs/design/examples.md, 3`.
- `skein/crates/skein-io/src/stage.rs` (its docs: the contract).
- `examples/echo/service/src/lib.rs` (`iterate`, `io_up`, `io_down`,
  `io_room`), `io_stage.rs`; `testing/skein-echo-client/src/lib.rs`.

## Owns and may change

- Owns `examples/echo/service/`, `testing/skein-echo-client/`, and in
  `tests/echo/` only `tests/suite/scenarios.rs` (the dispatchers of
  tests/echo are task echo-world's).
- The service's module doc ("The loop") may change to say the stage is
  io's; the doc's order of the passes stays.

## Steps

1. The service: `io_up` calls `stage::up` (the env's `now` is already set
   by `iterate`); `io_down` calls `stage::down` with `svc.asked`. Delete
   `io_stage.rs`, its `mod` line, and `io_room` if unused.
2. The client: its resume/up/fire and down loops go the same way.
3. Remove the `#[path]` module and its test from `scenarios.rs`.
4. `cargo fmt`; `heavy cargo clippy -p skein-echo-service -p
   skein-echo-client -p skein-echo-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-echo-service -p skein-echo-client -p
   skein-echo-world`.
5. `old_forms.py --write`; commit and push.

## Report

As common.md says.
