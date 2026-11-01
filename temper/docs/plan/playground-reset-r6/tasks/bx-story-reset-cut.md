+++
id = "bx-story-reset-cut"
part = "pg"
kind = "feature"
needs = ["bx-story-playground"]
files = ["temper/crates/temper/tests/reset/**"]
effort = "high"
minutes = 60
+++

## Goal

Draft section 8's second process story: the box killed during a reset, which ends whole at
the next start. The reset's kept acts are sent again at the next start (draft 6.2, "Cut by
a restart"), and the store reads as after an uncut reset.

Acceptance:
- `tests/reset/reset_cut.rs` (a `mod` line in `tests/reset/mod.rs`) holding
  `the_box_killed_during_a_reset_ends_it_whole_at_its_next_start`: as
  `a_first_run_into_the_playground_and_out_by_its_reset_leaves_the_store_empty` up to the
  playground running; then `playground reset` started without waiting
  (`support::ctl::Step::Start`), `KillBox` as soon as the box's stderr or the ctl's output
  shows the reset under way (the playground reading `resetting`), `StartBox`; then
  `playground` read until `not_started`; then the same empty reads as the first story (no
  forge, no scripted account, no `Playground` project, no `box/groups/N`, `box/started/N`
  or `forge-N`); the box exits 0 at the end with nothing left.
- A second case, `the_box_killed_twice_during_a_reset_ends_it_whole`: `KillBox` and
  `StartBox` again once the playground's forge reads removing (or is gone from the list
  while the row still reads resetting), then as above.
- Run each alone:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=reset::reset_cut::NAME)'`.

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, sections 4 ("Kept until whole"),
  6.2 and 8.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, T6.
- `tests/reset/playground.rs` (bx-story-playground), bx-harness's report.

## Owns and may change

- Owns: `temper/crates/temper/tests/reset/reset_cut.rs` and its `mod` line.
- May change: `tests/reset/playground.rs`, to share the story's first half as a function.
- Options, pre-approved:
  - If no read shows the reset under way before it ends (it ends too fast to cut), cut at
    the first ctl output of the `playground reset` call instead, and say whether the
    restart found a reset to resume (the box's stderr, or the playground read `resetting`
    after `StartBox`).

## Steps

1. The two cases; `heavy cargo check -p temper --tests`.
2. Run each alone; read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Commit and push.

## Report

For each case: where the kill landed (the reset's step as read), and what the next start
resumed.
