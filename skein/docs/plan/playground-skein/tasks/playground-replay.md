+++
id = "playground-replay"
part = "replay"
kind = "feature"
needs = ["playground-slices", "playground-visitor", "ui-web-log"]
files = ["skein/crates/skein-playground/src/replay.rs", "skein/crates/skein-playground/src/replay/**", "skein/tests/playground/tests/suite/replay.rs", "skein/tests/playground/tests/suite/replay/**"]
effort = "high"
minutes = 120
+++

# skein-playground: a session log replayed natively

## Goal

A session log replayed as a test: the world built by the application from the log's seed and
start, each input entering at its stamp, each digest of what the world said checked, the
first difference failing the replay at its line and naming the build the log came from, and
the world handed back at the log's end (`skein/docs/design/playground.md`, 6 and 6.2).

Acceptance:

- `skein_playground::replay` holds exactly `notes/contracts.md`, section 8.3: `Differs`, `Why`
  and `replay`.
- **The loop** (playground.md, 6.2): the replay steps the world (`World::step_with(turns)`)
  exactly when the slices step it, so its step count at each moment is the browser's: read
  the trunk's `skein/crates/skein-playground/src/slices.rs` and mirror its decisions (while
  busy; at a due instant the clock was moved to, as it does), never the harness's
  one-step-per-iteration. Idle, it moves the clock to the next due instant if that is at or
  before the next target (the next stamp's instant, or the end's), else to the target.
  Between steps, before each: every entry whose stamp's step is the world's step count enters
  through `reach.enter`, in order, once the world's instant is checked against the stamp's
  and the fold since the input before against the entry's `said`, and the fold starts again.
  After each step, the visitor's reports (`reach.take`), the world's printed lines (mapped to
  `bridge::Printed` as the slices map them) and the world's stop (once, when it is first
  found settled) are folded with `skein_ui_web::log::Saying`.
- **The end:** `End::At { step, instant, said }`: when the step count reaches `step`, the
  instant and the fold since the last input are checked, and the world is handed back;
  `End::Cut`: the world is handed back once the last entry has entered.
- **A difference** (`Differs`): `line` is the log's line, the first line 1, the first entry 2,
  the last line after the last entry; `build` the log's; `why` `Moment` (a stamp's step reached
  at another instant, or its instant passed with the step count short of it, or the step count
  past it) or `Said`.
- **The world tests** in `tests/suite/replay.rs` (playground.md, 8, "The log"), over the site
  world of `skein/tests/playground/src/site.rs` (or the composition the memory task may have
  added in `src/native.rs`; if neither serves a `skein_sim_world::World`, compose the site's
  processes in this task's own test files), each over seeds, each failure naming its seed:
  - `a_log_recorded_in_slices_replays`: a world run in `Slices` at paces and budgets drawn from
    the seed, inputs sent at drawn browser instants, every post fed to a `log::Log` (as a page
    does: `sent` for each input, `heard` for each post); its text read back with `log::read`
    and replayed over a fresh world from the same seed and start: `Ok`, and the replayed
    world's trace equals the recorded world's up to the end;
  - `a_replay_that_differs_fails_at_its_first_input`: the same log with one entry's digest
    changed (and, apart, one stamp's instant changed): `Differs` at that entry's line, `Said`
    (and `Moment`);
  - `a_replay_of_another_build_goes_on`: the log's build `Some(..)`; it replays `Ok` natively,
    and a differing one names that build in its `Differs`;
  - `a_cut_log_replays_to_its_last_input`;
  - `another_version_is_refused`: a text whose first line names version 2 is refused by
    `log::read` naming it (the replay never starts);
  - `the_referee_observes_each_turn`: a `Turns` that counts turns sees every step's.
- `heavy cargo nextest run -p skein-playground-world -E 'test(/replay::/)'` passes, its count
  in the report; clippy over both packages and the wasm32 clippy of `skein-playground` are
  clean; within the default profile's budget.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1, 4 and
  8.
- `skein/docs/design/playground.md`, 3, 6, 6.1 and 6.2; `skein/docs/design/world.md`, 3 to 5.
- The trunk's `slices.rs`, `visitor.rs`, `skein_ui_web::log`, and
  `skein/tests/playground/src/site.rs`, as their tasks left them.

## Owns and may change

- Owns the files in the header.
- Read only: `slices.rs`, `visitor.rs`, `memory.rs`, `site.rs`, `scripted.rs`, `skein-ui-web`.
  If the slices' stepping cannot be mirrored without a helper `slices.rs` would have to make
  public, it is a question, with the helper's signature.
- Pre-approved: if a test needs a log written by hand, write its text in the test; if the
  `Moment` checks read clearer as a private enum of what was found, keep it private.

## Steps

1. `replay`'s loop, its checks and its end.
2. The world tests, the recorded log's first.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; the clippy runs; the citations
   check.
4. Commit (why: a visitor's session replays natively, so a bug met in a browser is in the
   lowest world); push your branch.

## Report

As the implementer's text says, and also: the stepping rule you mirrored from `slices.rs`, in
a line, and the seeds and paces the recorded log's test draws.
