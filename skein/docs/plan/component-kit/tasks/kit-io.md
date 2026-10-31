+++
id = "kit-io"
part = "kit-contract"
kind = "interface"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-io/src/lib.rs", "skein/crates/skein-io/src/stage.rs", "skein/tests/io/tests/suite/main.rs", "skein/tests/io/tests/suite/stage.rs", "skein/tests/echo/tests/suite/scenarios.rs", "skein/tests/fake-peers/tests/suite/main.rs", "skein/tests/fake-peers/tests/suite/stages.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 35
+++
# skein_io::stage::{up, down}, tested once

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.1 (io): the io stage as free functions, modelled on
`skein/examples/echo/service/src/io_stage.rs`, each checking the kernel's
submission room before it takes work, with one ordering test in
`tests/io` in place of the two `#[path]`-included copies.

## What to build

- `pub mod stage` in skein-io (`src/stage.rs`, declared in `lib.rs`):
  - `pub fn up(io: &mut Io, env: &Env<Limits>, completions: &mut
    Queue<Complete>, events: &mut Queue<Event>, submissions: &mut
    Queue<Submit>, now: Time)`: the model's order and bounds: the ready
    list (at most `sockets + refusals` resumes) while there is room for
    `MAX_OUT_RESUME`; then, once nothing is ready, the completions while
    there is room for `MAX_OUT_UP`; then, once none is left, the due
    timers (at most `2 * sockets`) while there is room for `MAX_OUT_FIRE`.
  - `pub fn down(io: &mut Io, env: &Env<Limits>, requests: &mut
    Queue<Request>, submissions: &mut Queue<Submit>)`: each request while
    `io.takes()` and there is room for `MAX_OUT_DOWN` (echo's `io_down`,
    `skein/examples/echo/service/src/lib.rs:403`).
  - A module doc citing `skein/docs/design/io.md, 2` and
    `testing-strategy.md, 6`, and a short `///` contract on each.
- The ordering test, `tests/io/tests/suite/stage.rs`: one test that the
  stage tells ready results before completions and completions before
  timers with the kernel queue near full, and that each stage stops at
  the room check. It replaces
  `the_service_drains_each_stage_in_order_with_the_kernel_queue_near_full`
  (`tests/echo/tests/suite/scenarios.rs` ~249-319, with its `#[path] mod
  io_stage`) and `the_face_tells_ready_results_before_completions_and_timers`
  (`tests/fake-peers/tests/suite/stages.rs`, whole file, and its `mod
  stages;` in that suite's `main.rs`), which both go. Build it on
  tests/io's own world and fake kernel; the two removed tests show the
  scenario (one held write, `KERNEL_ROOM = 2`; Room, then Usage, then
  Failed, then the held write's Cancel).
- One test that `down` stops at the room check and leaves the rest
  queued.

## Pre-approved defaults

- If tests/io's world cannot hold the scenario without a helper, add the
  helper in `stage.rs` itself.
- Echo's own `io_stage.rs` stays until echo-stage removes it.

## Acceptance

- `heavy cargo nextest run -p <tests/io's package> -E 'test(/stage/)'`
  selects and passes the new tests; `-p skein-io` passes; the echo and
  fake-peers suites build and pass without the removed tests.
- `io_resume` does not rise (the stage is inside skein-io).
