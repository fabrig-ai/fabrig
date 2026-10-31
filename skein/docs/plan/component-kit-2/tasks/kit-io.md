+++
id = "kit-io"
part = "kit-contract"
kind = "interface"
needs = ["counts"]
files = ["skein/crates/skein-io/src/lib.rs", "skein/crates/skein-io/src/stage.rs", "skein/tests/io/tests/suite/main.rs", "skein/tests/io/tests/suite/stage.rs", "skein/tests/echo/tests/suite/scenarios.rs", "skein/tests/fake-peers/tests/suite/main.rs", "skein/tests/fake-peers/tests/suite/stages.rs"]
effort = "high"
minutes = 40
+++
# `skein_io::stage::{up, down}`, tested once in tests/io

## Goal

Draft, 3.1 (io): io's stage as two free functions in skein-io, modelled
on `skein/examples/echo/service/src/io_stage.rs` and echo's `io_down`,
each checking the kernel's submission room before it takes work; one
ordering test in tests/io in place of the two `#[path]`-included copies.
No host moves onto it here.

Acceptance:

- `skein_io::stage::{up, down}` exist with the signatures below and
  their `///` contracts.
- `heavy cargo nextest run -p skein-io-world -E 'test(/stage/)'` selects
  the new tests (at least three) and passes; `heavy cargo nextest run -p
  skein-io` passes.
- The echo test `the_service_drains_each_stage_in_order_with_the_kernel_queue_near_full`
  and the fake-peers test file `stages.rs` are gone, with their `#[path]`
  includes; `heavy cargo nextest run -p skein-echo-world -p
  skein-fake-peers-tests` passes.
- `heavy cargo clippy -p skein-io -p skein-io-world -p skein-echo-world
  -p skein-fake-peers-tests --all-targets -- -D warnings` is clean.
- `io_resume` does not rise: the stage is inside skein-io, and the
  removed tests called `io::resume` only through their includes.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`.
- `skein/docs/design/io.md`, 2 (io in skein; the stage's order) and 8
  (testing: the harness checks the kernel's room);
  `skein/docs/foundation/testing-strategy.md`, 6.
- The model: `skein/examples/echo/service/src/io_stage.rs` (`up`, `room`)
  and `skein/examples/echo/service/src/lib.rs:401-417` (`io_down`,
  `io_room`). The bounds: `skein/crates/skein-io/src/limits.rs:116-134`
  (`MaxOut`, `MAX_OUT_*`).
- The two tests that go, for the scenario they pin:
  `skein/tests/echo/tests/suite/scenarios.rs:249-320` and
  `skein/tests/fake-peers/tests/suite/stages.rs` (97 lines).

## Owns and may change

- `skein/crates/skein-io/src/stage.rs` (new) and its `pub mod stage;` in
  `src/lib.rs`.
- `skein/tests/io/tests/suite/stage.rs` (new) and its `mod stage;` in that
  suite's `main.rs`.
- In `skein/tests/echo/tests/suite/scenarios.rs`: remove the `#[path]
  mod io_stage;` and the one test, and imports only they used. Nothing
  else in the file.
- `skein/tests/fake-peers/tests/suite/stages.rs` (deleted) and its `mod
  stages;` in that suite's `main.rs`.
- Echo's own `io_stage.rs` stays: `echo-stage` removes it.

Pre-approved:

- If tests/io's world cannot hold the scenario without a helper, write
  the helper in `suite/stage.rs` itself, not in `tests/io/src/`.
- If `up`'s bounds (`sockets + refusals` resumes, `2 * sockets` timers)
  need a `Limits` field the model reads that is private to skein-io, read
  it in-crate; the signature stays.

## Steps

1. `src/stage.rs`, a module doc of two or three lines citing
   `skein/docs/design/io.md, 2` and `testing-strategy.md, 6`:

   ```rust
   /// io's stage of the up pass: the ready list, then, once nothing is
   /// ready, `completions`, then, once none is left, the timers due at
   /// `now`. Each step is taken only while `submissions` has room for its
   /// `MAX_OUT_*`; a step that would not fit waits for the next pass.
   pub fn up(
       io: &mut Io,
       env: &Env<Limits>,
       completions: &mut Queue<Complete>,
       events: &mut Queue<Event>,
       submissions: &mut Queue<Submit>,
       now: Time,
   )

   /// io's stage of the down pass: each of `requests`, oldest first,
   /// while io can hold a refusal (`Io::takes`) and `submissions` has room
   /// for `MAX_OUT_DOWN`. What is not taken stays queued.
   pub fn down(
       io: &mut Io,
       env: &Env<Limits>,
       requests: &mut Queue<Request>,
       submissions: &mut Queue<Submit>,
   )
   ```

   The bodies are the model's: `up` is `io_stage::up` with its loop
   bounds; `down` is echo's `io_down`. One private `room(submissions,
   max)` helper. `Complete`, `Submit` are `crate::kernel`'s; `Event`,
   `Request`, `Limits`, `Io` are the crate's.
2. `tests/io/tests/suite/stage.rs`, one behaviour per test. The echo
   test builds its scenario on io directly (an `Io`, its queues, a
   `Queue<Submit>` of capacity 2), with no world; do the same, calling
   `skein_io::stage`:
   - `the_stage_tells_ready_results_before_completions_and_timers`: with
     one held write and a kernel room of 2, `up` tells Room, then Usage,
     then Failed, then the held write's cancel, in that order (the
     scenario of the two tests that go).
   - `the_up_stage_stops_at_the_kernel_room_and_resumes_next_pass`.
   - `the_down_stage_stops_at_the_kernel_room_and_leaves_the_rest_queued`.
3. Remove the two old tests and their includes.
4. Checks of `common.md`; commit; push.

## Report

The final signatures, the tests added and removed (names), the checks
with counts, any helper written and why.

## Answers

2026-10-31 (kit-io-1): the `io_resume` baseline was missing from the base.
The counts task has since merged into the trunk
(`component-kit-2/integration` at `23a86bf59`, `io_resume 39`); this task
now `needs = ["counts"]`. Rerun on a base that holds it, and stop as
before if `io_resume` is still absent. Do not proceed without the
baseline.
