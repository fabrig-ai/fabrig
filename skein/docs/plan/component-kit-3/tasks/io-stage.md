+++
id = "io-stage"
part = "kit-contract"
kind = "interface"
needs = []
files = ["skein/crates/skein-io/src/stage.rs", "skein/crates/skein-io/src/lib.rs", "skein/crates/skein-io/src/layer.rs", "skein/tests/io/tests/suite/stage.rs", "skein/tests/io/tests/suite/main.rs"]
effort = "high"
minutes = 50
+++

## Goal

io's stage of each pass, written once in skein-io as
`skein_io::stage::{up, down}` (`skein/docs/design/draft/component-kit.md`,
3.1): every host's io stage today is a private copy of it, and only some
copies check the kernel's submission room before each step. Modelled on
`skein/examples/echo/service/src/io_stage.rs` (the up stage) and that
service's `io_down` (`skein/examples/echo/service/src/lib.rs`, the down
stage). One ordering test in `tests/io` covers it. No host moves onto it
in this task.

Acceptance:
- `skein_io::stage::up` and `skein_io::stage::down` with the signatures
  below, each step taken only while the submission queue has room for that
  step's `MAX_OUT_*`;
- the tests below pass in `tests/io` (`heavy cargo nextest run -p skein-io-world`), and skein-io's own
  tests and clippy pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/design/io.md`, section 2 (the four entry points and io's stage
  in the up pass, the kernel's capped queues); `skein/docs/foundation/programming-model.md`,
  section 2 (the loop's two passes); `skein/docs/foundation/testing-strategy.md`,
  section 6 (no edge queue past its cap).
- The models: `skein/examples/echo/service/src/io_stage.rs`,
  `skein/examples/echo/service/src/lib.rs` (`io_down`, `io_room`); the two
  copies of the ordering test, `skein/tests/echo/tests/suite/scenarios.rs`
  (`the_service_drains_each_stage_in_order_with_the_kernel_queue_near_full`)
  and `skein/tests/fake-peers/tests/suite/stages.rs`.
- `skein/crates/skein-io/src/layer.rs` (`resume`, `up`, `fire`, `down`,
  `Io::is_ready`, `is_due`, `takes`) and `src/limits.rs` (`MAX_OUT_*`).

## Owns and may change

- Owns a new `skein/crates/skein-io/src/stage.rs`, its `pub mod stage;` in
  `src/lib.rs`, and a new `skein/tests/io/tests/suite/stage.rs` with its `mod`
  line in `tests/io/tests/suite/main.rs`.
- `src/layer.rs`: only a `pub(crate)` accessor the stage's bounds need (how
  many entries io's ready list and deadlines hold), nothing else.
- Not the two copies of the ordering test, nor echo's or fake-peers' stage:
  their own tasks delete them once they move.
- Pre-approved: the loops' bounds. Each loop is a bounded `for` (no `loop`,
  no `while`); bound `resume` by the ready list's length at the stage's
  start, `up` by the completions' length at its start, and `fire` by the
  deadlines io holds at its start, through the `pub(crate)` accessors. If a
  bound cannot come from io itself, use echo's (`sockets + refusals`,
  `2 * sockets`) and say so.

## Steps

1. The surface, exactly:

   ```rust
   //! io's stage of each pass (io.md, 2): the loop calls `up` in its up pass
   //! and `down` in its down pass, with the kernel's capped queues.

   /// `resume` while the ready list holds an entry, then `up` for each
   /// completion only once it is empty, then `fire` while a deadline is due
   /// at `env.now`. Each step waits for room in `subs` for what it may
   /// submit (`MAX_OUT_RESUME`, `MAX_OUT_UP`, `MAX_OUT_FIRE`): without it the
   /// stage stops there, and the loop's next iteration takes it up.
   pub fn up(
       io: &mut Io,
       env: &Env<Limits>,
       completions: &mut Queue<Complete>,
       up: &mut Queue<Event>,
       subs: &mut Queue<Submit>,
   );

   /// `down` for each request in order, while io takes one (`Io::takes`) and
   /// `subs` has room for `MAX_OUT_DOWN`; the rest stay queued, in order.
   pub fn down(
       io: &mut Io,
       env: &Env<Limits>,
       requests: &mut Queue<Request>,
       subs: &mut Queue<Submit>,
   );
   ```

   `Complete` and `Submit` are `skein_io::kernel`'s. Neither reclaims: the
   reclaim point stays the host's. `up` reads the time from `env.now`
   (echo's copy takes a separate `now`; its callers pass the same value).
2. `tests/io/tests/suite/stage.rs`, each test one behaviour, with the
   world's existing limits fixture:
   - `the_stage_tells_ready_results_before_completions_and_timers`: the
     ordering test the two copies share, ported onto `stage::up` and
     `stage::down` (a held write in a submission queue of two blocks all
     three steps; once the kernel takes it, the room event, the usage, then
     the timed stream's failure, in that order);
   - `the_up_stage_waits_for_room_at_each_step`: with room for fewer than a
     step's `MAX_OUT_*`, nothing of that step happens and its work is still
     there;
   - `the_down_stage_keeps_what_io_does_not_take_in_order`: with room for one
     `MAX_OUT_DOWN`, the first request goes down and the rest stay queued in
     order; likewise while `takes` is false.
3. `cargo fmt`; `heavy cargo clippy -p skein-io -p skein-io-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-io` and the new tests by name. Commit and push.

## Report

The surface as it landed, the bounds you chose, and the three tests with
their counts.
