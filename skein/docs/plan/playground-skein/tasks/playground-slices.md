+++
id = "playground-slices"
part = "playground"
kind = "feature"
needs = ["playground-crate", "ui-web-surface"]
files = ["skein/crates/skein-playground/src/slices.rs", "skein/crates/skein-playground/src/slices/**", "skein/tests/playground/src/scripted.rs", "skein/tests/playground/src/scripted/**", "skein/tests/playground/tests/suite/slices.rs", "skein/tests/playground/tests/suite/slices/**"]
effort = "high"
minutes = 120
+++

# skein-playground: the slices' loop

## Goal

The worker's loop over a world whose processes include the visitor: inputs queued and entered
between steps, stamped; steps while busy up to a slice's budget; the clock moved toward the
browser's elapsed time times the speed while idle; the posts of each slice in order, its
clock last; and when to wake next (`skein/docs/design/playground.md`, 3 and 5). It names no
visitor type: it reaches the visitor through `Reach`'s functions, so its tests use a scripted
stand-in.

Acceptance:

- `skein_playground::slices` holds exactly `notes/contracts.md`, section 8.1: `Reach`, `Wake`,
  `Slices` with `new`, `input`, `pace`, `slice`, `world`, `world_mut`. `Reach<P>` is `Clone`
  and `Copy` for every `P` (write the impls; a derive would ask `P: Copy`).
- **A slice** at browser instant `b` (playground.md, 3), with `elapsed` the browser time since
  the last slice (or since `new`):
  1. each queued input enters, in order: `(reach.enter)(world.process(reach.at), input)`,
     then `ToPage::Stamp { step: world.steps(), instant: world.now() }` posted;
  2. while the world is busy and the budget lasts, a step;
  3. once idle, `target = now + elapsed × speed / 1000` (checked arithmetic, saturating at
     the world's far future): while the next due instant is at or before the target and the
     budget lasts, the clock moves to it (`World::advance`) and the world steps while busy;
     otherwise the clock moves to the target, alone, and the slice ends. Elapsed time a busy
     slice could not use is dropped: no catch-up;
  4. the visitor's reports (`reach.take`), each posted as `ToPage::Report`, then the lines
     printed (`World::printed()`), each as `ToPage::Printed` (contracts, 8.1); then
     `ToPage::Stopped` once, at the slice that finds the world settled; then
     `ToPage::Clock { step, now, wall, idle, next_due, memory: 0, slice: ZERO }` last.
  It returns `Wake::Now` while busy; idle, `Wake::At(b')`, the browser instant at which the
  next due instant comes at this speed (`b + (due - now) × 1000 / speed`, rounded up); and
  `Wake::Message` while paused, settled or with nothing due.
- **Paused** (`Speed::PAUSED`): no clock moves, but a busy world steps, and inputs enter and
  are answered at the paused instant (playground.md, 3, "Paused, it is still").
- **No verdicts:** no step cap, no idle failure (world.md, 6).
- **The visitor's index** is `reach.at`, a process the world starts itself before any hosted
  child, so a child dropped never moves it; `new` asserts the index names a process.
- **The scripted stand-in** `skein/tests/playground/src/scripted.rs`: a process enum for the
  slices' tests (or `skein-world-tests`'s `Script` processes, if they serve, as a
  dependency `skein-world-tests = { path = "../world" }`, said in the report) with a
  stand-in visitor Host that answers each input with a fetch report after a scripted number
  of steps or a scripted delay, so a slice sees both work that waits on time and work that
  does not, and a process that prints lines; the world built with
  `skein_sim_world::World` with the trace whole.
- **The world tests** in `tests/suite/slices.rs` (playground.md, 8, "The slices"), over
  seeds, each failure naming its seed:
  - `a_world_paced_in_slices_ends_as_the_world_run_whole`: browser intervals, speeds (0
    among them) and budgets drawn from the seed, inputs entered at instants drawn too; the
    stamps it posts recorded; the same world run whole, each input entered at its stamp's
    step and instant: the same trace, and the same reports and lines, each stream in its
    order;
  - `an_input_entered_at_a_stamp_replays_at_it`;
  - `a_paused_world_answers_an_input_at_its_paused_instant`: paused, an input that waits on no
    time is answered, the stamp and the clock at the paused instant;
  - `a_paused_world_moves_no_clock`: slices at later browser instants, the clock still;
  - `a_slice_steps_at_most_its_budget`;
  - `an_idle_world_wakes_when_its_next_due_comes_at_its_speed`: `Wake::At` at 1× and 10×,
    `Wake::Message` paused and with nothing due;
  - `the_posts_keep_their_order`: stamps, reports, lines, the stop, the clock last;
  - `a_settled_world_says_it_stopped_once`.
- `heavy cargo nextest run -p skein-playground-world -E 'test(/slices::/)'` passes, its count
  in the report; clippy over both packages and the wasm32 clippy of `skein-playground` are
  clean; within the default profile's budget.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1 and 8.1.
- `skein/docs/design/playground.md`, 3 and 5; `skein/docs/design/world.md`, 3 to 6.
- Models: `skein/tests/world/tests/suite/steps.rs` and `skein/tests/world/tests/steps/support.rs`
  (a world stepped in slices against the world run whole), `skein/crates/skein-sim-world/src/world.rs`
  (`step`, `advance`, `busy`, `next_due`, `settled`, `printed`).

## Owns and may change

- Owns the files in the header. `skein/tests/playground/Cargo.toml` may gain a dependency line
  for the scripted processes (`skein-world-tests`, `skein-fake-machine`), listed in the report.
- Read only: `skein-sim-world` (a change it needs is a question), `skein-ui-web`.
- Pre-approved: if `World::advance` refuses the target because a process's deadline is earlier
  than `next_due` said, it is a question with the seed; if the posts' `Printed` mapping wants a
  `From`, write it private in `slices.rs`.

## Steps

1. `Reach`, `Wake`, `Slices` and a slice's body.
2. The scripted stand-in and the world tests, one by one.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs; the citations
   check.
4. Commit (why: the worker's loop steps a world in slices paced by the browser); push your
   branch.

## Report

As the implementer's text says, and also: how a slice's target and wake are computed, in a few
lines, and the seeds the tests run.
