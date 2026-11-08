+++
id = "change-panel"
part = "change-panel"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/change_panel.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/panel.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs"]
effort = "medium"
minutes = 120
+++

## Goal

The engine-domain world's three ignored panel stories run again in the
focused suite, each committing its change to a record that already
exists through the product's own events. They re-send
`forge::Event::Change` for an existing task, which registers a change
and refuses one it already holds (`temper-engine-domain-forge/src/domain.rs`,
`register_change`), so each fails at its setup assertion. That refusal
is right: `Change` is the registration of an activated change task
(`boundary.rs`), and the root sends it only for a task with no row
(`forge_route.rs`, `start_change`). The stories are rewritten to the
events that do change a saved record; the one the product lacks, a
change's priority moving with its goal's (`forge.md`, 9: "Priorities
move with the goals'"), is built.

Acceptance:

- In `temper/tests/engine/domain/tests/suite/change_panel.rs` (package
  `temper-engine-domain-world`, binary `suite`), these lose their
  `#[ignore]` and pass:
  - `own_record_change_refreshes_a_watched_panel`: the change's own
    saved record marks its watched panel, whose delivery is the whole new
    panel (forge.md 8.8);
  - `priority_moving_ahead_marks_the_waiting_panel`: a change moving
    ahead in its queue by priority marks the watched change behind it
    (8.8, 9);
  - `queue_exit_and_reentry_refresh_the_panel_behind`: a change leaving
    its queue and coming back marks the change behind it, at place 2
    then 3 (8.8; 9: it comes back at its place by the time it first
    became ready).
- `register_change`'s refusal of an existing task is unchanged; no story
  re-sends `Change` for a task it holds.
- A tracked goal's priority moving moves its live changes' priority
  (forge.md 9), saved and marked by the existing `Domain::priority`.
- Each new or rewritten test bites: break the rule once (skip the save's
  mark; skip the priority's routing) and see it fail; restore.
- `heavy cargo nextest run -p temper-engine-domain-forge -p
  temper-engine-domain -p temper-engine-domain-world` passes; this
  changes what a goal's feed does to its changes, so also `heavy cargo
  nextest run -p temper-engine-forge-world -p temper-system-world` (the
  readers of the root's goal projection), and `heavy cargo check -p
  temper-service --tests`.

## Inputs

- `temper/docs/plan/ignored-engine/notes/common.md`: every rule there
  holds.
- `temper/docs/design/domain/forge.md`, 8.8 (Its panel: **Marked**) and
  9 (Landing queues: ordered by priority, its goal's; priorities move
  with the goals'; leaving the queue when held and coming back at its
  place).
- `temper/docs/design/client/forge.md`, 9.2 (a watched delivery is the
  whole current panel).
- `jig/docs/design/domain/core.md`, 6.2 (a goal's priority, the lower
  first) and `jig/crates/jig-domain-core/src/effects/projections.rs`
  (`ProjectionGoal.priority`, the feed's `plan`).
- The forge's own tests of the marks, which stay as they are:
  `temper-engine-domain-forge/src/tests/panel.rs` (save before mark,
  queue entry and departure marks, a bounded priority move).

## Owns and may change

- Owns the three stories in `change_panel.rs` and its private helpers;
  every assertion of a story that the design still asks is kept.
- **The own-record story:** in place of a second `Change` with
  `repairs = 1`, the record is changed by `forge::Event::Relayed { task:
  51, questions }` with one more question taken than the row holds, and
  the story asserts the durable row's `questions` and the panel's
  questions field (by its name in `forge::Panel`) in place of
  `repairs.used`. If `Relayed` is refused for a queued row, or changes
  no panel field, use `QueueRepairStarted` or a failed-CI `StepChange`
  (as `tests/engine/forge/tests/suite/simulation.rs` drives one),
  keeping the `repairs` assertion, and say which.
- **The queue story:** task 52 leaves the queue by a hold and comes back
  by its release: `forge::Event::RelayRefused { task: 52, questions,
  words }` then `forge::Event::ReleaseChange { task: 52 }`, each settled,
  asserting task 53's panel at place 2, then 3, as the story does now.
  If the hold or release takes another state than the design's (the
  release does not restore `Queued` at its first readiness), that is a
  product fault against forge.md 9: fix it in
  `temper-engine-domain-forge/src/domain.rs` with a row in
  `src/tests/panel.rs`. If the hold cannot be reached from a seeded
  queued row, use `StepChange` with `kept: true` then `false`, as
  `simulation.rs` does, and say so.
- **The priority story and its product half.** Sketch:

  ```rust
  // temper-engine-domain-forge/src/boundary.rs, Event:
  /// A change's goal's priority moved (domain/forge.md, 9).
  Priority { task: u64, priority: i32 },
  ```

  `forge::step` dispatches it to the existing `Domain::priority(env,
  task, priority, out)`, which saves the row and marks the panels behind
  (nothing new there). The root's `project_goal`
  (`temper-engine-domain/src/route/forge_route.rs`) pushes, for each task
  of the feed's `plan` that the forge holds as a live change whose
  priority differs from the goal's, `Work::Forge(forge::Event::Priority
  { task, priority })`, the goal's `u32` priority as `i32`
  (`i32::try_from(..).unwrap_or(i32::MAX)`). `admission.rs` lists the
  new variant beside `Change` in its exhaustive arms, as the compiler
  asks. The story sends `forge::Event::Priority { task: 53, priority: 0
  }` in place of its second `Change`, as the file already seeds rows by
  forge events.
- A story of the product half: a goal holding a queued change has its
  priority amended (a priority-only `amend`, jig's people or the run's
  tool) and the change's row follows. Put it beside the existing
  projection stories in the engine-domain world if one makes a goal's
  change cheaply (`temper/tests/engine/domain/tests/suite/forge_routes/`
  holds goals with changes; add a story to a file the `forge-routes`
  task does not own); otherwise a table row in `temper-engine-domain`'s
  own tests of `project_goal`. Name which in the report.
- Options, each pre-approved:
  - If a goal's priority-only amendment sends no projection feed, or the
    feed's `plan` does not name the goal's changes, the root's half is
    larger than this task: build the forge's `Event::Priority` and the
    story's use of it, leave `project_goal` unchanged, and add a **Not
    yet** entry to `temper/docs/design/domain/forge.status.md`: `9: a
    goal's priority moving does not move its changes': a change keeps
    the priority its call gave (parameter 5, forge_route.rs's
    start_change), and only the forge's own Event::Priority moves it.`
    Write it as its own bullet at the end of the gaps list (the
    `look-round-24` plan is editing that file). This departs from the
    implementer's text, which closes `docs/design/` to you. Say so in the
    report's first lines.
  - If a change outside any goal is in a feed's plan (it is not, by
    core.md 6.2), leave it its own priority.
  - If the compiler asks the new variant's arm elsewhere (a
    `match` over `forge::Event` in `temper-service` or a world), add it
    there, only for that.

## Steps

1. Remove the three `#[ignore]` lines; commit. Run each story alone
   (count 1) and quote its first failure (by reading: each setup
   assertion, the durable row unchanged).
2. Rewrite the own-record and queue stories; run each alone.
3. Add `Event::Priority` and its dispatch; rewrite the priority story;
   run it alone.
4. Route the goal's priority in `project_goal` and add its story; run it
   alone.
5. Break each rule once and see its test fail; restore.
6. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-engine-domain-world --all-targets --
   -D warnings`; the nextest runs and check in the acceptance.
7. Commit, run `python3 -I -B tools/citations/citations.py --base
   origin/main`, push.

## Report

The implementer's report, with: each story's first failure, which
events each now drives, where the goal's priority is routed (or the
**Not yet** entry, if the fallback was taken), the rules broken by hand
and the failures seen, and each story's time.
