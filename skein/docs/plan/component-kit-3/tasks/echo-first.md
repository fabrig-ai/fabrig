+++
id = "echo-first"
part = "echo-first"
kind = "migration"
needs = ["io-stage"]
files = ["skein/examples/echo/service/**", "skein/testing/skein-echo-client/**", "skein/tests/echo/tests/suite/scenarios.rs"]
effort = "medium"
minutes = 35
+++

## Goal

The echo example's service and its client run their io stage on
`skein_io::stage::{up, down}` (`skein/docs/design/draft/component-kit.md`,
3.2): the first root to run end to end on the contract. The service's
private `io_stage.rs` and the `#[path]`-included copy of its ordering test
go; `tests/io`'s stage test (task `io-stage`) replaces it.

Acceptance:
- `examples/echo/service/src/io_stage.rs` is gone; the service's `io_up`
  and `io_down` call `skein_io::stage::up` and `skein_io::stage::down`;
  `io_room` goes if nothing else uses it;
- `testing/skein-echo-client`'s io stage calls them too;
- `tests/echo/tests/suite/scenarios.rs` no longer includes `io_stage.rs`
  (the `#[path]` module and
  `the_service_drains_each_stage_in_order_with_the_kernel_queue_near_full`
  go);
- the echo world's tests pass: `heavy cargo nextest run -p skein-echo-world`,
  its end-to-end test included;
- the `io_resume` count falls by the two sites (`notes/counts.md`).

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/counts.md`.
- `skein/docs/design/io.md`, section 2 (io's stage); `skein/docs/design/examples.md`
  for the echo example's loop.
- `skein/crates/skein-io/src/stage.rs` (the contract, as merged).

## Owns and may change

- Owns `skein/examples/echo/service/**` and `skein/testing/skein-echo-client/**`;
  in `skein/tests/echo/`, only `tests/suite/scenarios.rs`, and there only the
  `#[path]` module and the one test that uses it.
- Not `examples/echo/protocol` (its `resume` is the protocol layer's, not io's).
- Pre-approved:
  - `stage::up` reads the time from `env.now`. The service sets
    `io_env.now` before its up pass; if the client does not, set it where
    the client builds its env each iteration.
  - If the client's stage differs from the service's in a way the shared
    stage cannot carry (a different order between io and another layer),
    keep that order outside the stage calls, and say so.

## Steps

1. Service: `io_up` and `io_down` onto the stage; delete `io_stage.rs` and
   its `mod`; remove what is left dead.
2. Client: likewise.
3. `scenarios.rs`: remove the `#[path]` module and its test.
4. From `skein/`: `python3 -I -B scripts/old_forms.py --write` (if the
   `counts` task has merged; otherwise skip and say so).
5. `cargo fmt`; `heavy cargo clippy -p skein-echo-service -p skein-echo-client -p skein-echo-world --all-targets -- -D warnings`
   (the package names in each `Cargo.toml`); `heavy cargo nextest run -p skein-echo-world`
   and the service's and client's own tests. Commit and push.

## Report

The two sites moved, the lines removed, the echo world's test count, and
the `io_resume` count after.
