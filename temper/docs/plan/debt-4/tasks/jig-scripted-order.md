+++
id = "jig-scripted-order"
part = "jig-scripted-rules"
kind = "feature"
needs = ["jig-scripted-inbox"]
files = [
  "jig/tests/client/src/stories/shared/**",
  "jig/testing/jig-client-scripted-engine/src/queries.rs",
  "jig/docs/design/client/domain/testing.status.md",
]
effort = "high"
minutes = 60
+++

# jig D028, task line order: shared stories hold the fake's `order`

## Goal

`queries.rs`'s `order` sorts task lines the fake builds live, so nothing of jig's can replace it; the
item's other choice is a shared story. Activity and Priority have theirs. Changed and Ended do not:
each gets one, on the scripted and joint fronts. The third of four tasks on
`jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`; the item stays open.

Acceptance:

- Two new stories pass on the scripted and joint fronts, as sketched, each reading the order from
  the tree.
- `tests/client/tests/suite/stories.rs`'s `a_project_s_tasks_section_watches_one_query_by_last_change`
  stays: it checks the watch's subject.
- If the joint front shows the fake's `QueryOrder::Ended` to differ from the core's, the fake
  changes, and your report says how: the story holds the core.
- The note's `What a task runs` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/jig-scripted.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`, whole.
- `jig/docs/design/client/domain/testing.md`, 4 and 5.
- The core's order: `jig-domain-core/src/documents/task_query.rs`, `sort_key`; `jig-domain-tasks/src/index.rs`, 108-129.
- The view's queries: `jig-client-view/src/state/page/project.rs` (Changed),
  `src/state/page/conversations.rs` and `src/state/queries.rs` (Ended).

## Owns and may change

- Owns: `jig/tests/client/src/stories/shared/**`; `E/queries.rs`'s `order`, for what the stories
  show and nothing else.
- Status: nothing here; `jig-scripted-requester` closes the entry.
- Pre-approved: a tie (equal keys, ordered by number) is not worth a seed of its own; if neither
  story reaches one, say so in your report and add none.

## Steps

1. In `stories/shared/b_conversations.rs`, beside `a_person_s_chats_list_live_then_ended_by_last_activity`:

   ```rust
   /// The newest ending first: `QueryOrder::Ended`.
   pub fn a_person_s_ended_chats_list_the_last_ended_first<F: Front<Test>>(world: &mut World<Test, F>);
   ```

   Start two chats, cancel the older, then the newer, and read the ended group's order.

2. In `stories/shared/b_goals.rs`, beside `moving_a_goal_up_reorders_the_board`:

   ```rust
   /// The last changed first: `QueryOrder::Changed`, on the project's tasks section.
   pub fn a_project_s_tasks_section_lists_the_last_changed_first<F: Front<Test>>(world: &mut World<Test, F>);
   ```

   Over `Facts::Goals`: act on the first goal so that it is the last changed, then read the
   section's order. Reuse the act `moving_a_goal_up_reorders_the_board` takes if it changes one.

3. One line each in `person_story_suite!`. What the note's `What a task runs` names; the citations
   check. Commit and push.

## Report

Each story's fronts and seconds, and any difference the joint front showed.
