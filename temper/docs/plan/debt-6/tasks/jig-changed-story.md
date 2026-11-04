+++
id = "jig-changed-story"
part = "jig-review"
kind = "feature"
needs = []
files = ["jig/tests/client/src/stories/shared/b_goals.rs", "jig/tests/client/src/stories/shared/b_seed.rs"]
effort = "high"
minutes = 40
+++

# jig: the `Changed` story can see `Changed`

## Goal

The review's Part B, finding 1. In `a_project_s_tasks_section_lists_the_last_changed_first` goal 1 is
the last changed, so the expected `[1, 2]` equals the number tie-break, and replacing the scripted
engine's `Changed` arm in `queries.rs` with `Ordering::Equal` stays green. The cut read the story, the
seed and `priorities.rs` (the scripted amend sets `changed`) and agrees. The story asserts both orders:
goal 2 first before the move, goal 1 first after it.

Acceptance:

- The story reads `[2, 1]` before the move and `[1, 2]` after it, by titles, on the scripted and the
  joint fronts.
- With `QueryOrder::Changed => Ordering::Equal` in
  `jig/testing/jig-client-scripted-engine/src/queries.rs`, `scripted::a_project_s_tasks_section_lists_the_last_changed_first`
  goes red. That is the hand-break the request names.
- The other stories on `Facts::Goals` pass unchanged.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/client/domain/testing.md`, 4 and 5.

## Owns and may change

- Owns: `b_goals.rs`'s story, and `b_seed.rs`'s `Facts::Goals` arms, `scripted` and `core`.
- If the two goals share a `changed` on the joint front, add an `Act::Pause` between them in
  `b_seed::core`, as `b_inbox::core` does with its `SPACING`. Say so in the report.
- If the scripted amend's `wall` is not past the seeded `changed`, seed smaller values, never a change
  to the fake.
- Not this task's: `queries.rs`, which is only hand-broken; `goal_order` and the other stories.

## Steps

1. In `b_seed::scripted`'s `Facts::Goals`, seed each goal's `changed` in number order
   (`task.changed = Wall::from_nanos(number)`), so goal 2 is the last changed on both fronts: on the
   joint it is created second.
2. Rewrite the story: read the tasks section first, then move, then read it again.

   ```rust
   pub fn a_project_s_tasks_section_lists_the_last_changed_first<F: Front<Test>>(world: &mut World<Test, F>) {
       let client = ready(world, ProjectSection::Tasks);
       assert_eq!(goal_titles(world, client), [TASK_TWO, TASK_ONE]);
       // Go to the goals section, Move up, Confirm, as moving_a_goal_up_reorders_the_board does.
       // Go back to the tasks section; See both links.
       assert_eq!(goal_titles(world, client), [TASK_ONE, TASK_TWO]);
   }
   ```

   `goal_titles` is the `titles` read the story already holds, as a helper beside `goal_order`.
3. `heavy cargo nextest run -p jig-client-world --test b_shared -E 'test(/goal/) | test(/project/)'`,
   then the hand-break.
4. `cargo fmt`; `heavy cargo clippy -p jig-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client-world`; the citations check. Commit and push.

## Report

The common note's table: the `Changed` arm's hand-break, what the red test read, and whether the joint
front needed the pause. Net lines. This task may add lines; the part as a whole must not.
