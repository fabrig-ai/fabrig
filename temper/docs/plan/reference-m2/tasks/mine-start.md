+++
id = "mine-start"
part = "mine"
kind = "feature"
needs = ["mine-call"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_landing.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs"]
effort = "high"
minutes = 120
+++

# mine-start: `"branch": "mine"` makes the goal's landing, a change into main on its goal's branch

## Goal

The root turns `change`'s `branch: mine` (task `mine-call`) into a change whose branch is
the goal branch its requester holds, g: no producer, no hold taken or handed, so g stays its
goal's (`temper/docs/design/domain/forge.md`, 8.1; `temper/docs/design/domain/agent.md`,
4.3.1). Contract C4 has the resolution, the refusals' words, parameter 11 and the holdings.

Acceptance, as engine stories:
- A tracked goal G holding g, after one change of its own has landed into g, calls `change`
  with `{"words":"Land the goal","branch":"mine"}`: one change task L, its spec's parameters
  1, 2 (`main`), 3 (g's name) and 11 (`1`); g's forge hold and jig holding stay G's
  throughout; L opens a pull request from g into main with no producer, passes CI, merges at
  g's head, and lands; no `DeleteBranch` names g as L closes.
- The three refusals of C4, each with its exact words and no delegate built:
  `landing goal` with `branch mine`; a task holding no goal branch (modelled on
  `change_tool.rs`'s `landing_goal_without_a_held_goal_branch_is_refused`); a second
  `branch mine` while L lives, and two in one call.
- Every `forge_routes` story and `temper-engine-domain`'s lib tests pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (`branch`).
- `temper/docs/design/domain/forge.md`, 8.1 ("A change on its requester's held branch") and
  section 11 (its sentence on a change on its requester's held branch).
- `temper/docs/plan/reference-m2/notes/contracts.md`, C1 and C4.
- `temper/docs/plan/reference-m2/notes/common.md`.
- `temper/docs/design/domain/agent.status.md`: its **Not yet** "4.3.1: `change`'s `branch`,
  ...", which this task removes.

## Owns and may change

Owns the files in the header; in `forge_route.rs`, only `change_batch`, `change_landing` (or
a sibling `change_branch` beside it), `chat_change`'s parameters, `implicit_task_holdings`,
`start_change`'s parameter match and its `Event::Hold`, and any other match over a change's
parameters that refuses an unknown number (`rg 'name: 10' forge_route.rs`). Other plans
change this file: keep each hunk there.

- `change_batch`: resolve and refuse as C4 says, in its order, before `chat_change`; pass
  the resolved branch to `chat_change`, which appends parameters 3 and 11 after
  `change_parameters`' two.
- `implicit_task_holdings`: a change spec carrying parameter 11 gets no write holding for
  its parameter 3 (it keeps the base's landing-holder check as any change).
- `start_change`: accept `Number { name: 11, value: 1 }`; with it, push `Event::Names` as
  today and no `Event::Hold` for the branch.
- `goal_branch.rs` (the suite's): its helpers `adopted_world`, `running_goal`, `resource`,
  `change`, `writer` may become `pub(super)` for the new file; visibility only.
- `forge_routes.rs`: one `mod goal_landing;` line beside `mod goal_changes;`.
- If L, started on g, does not reach opening at once (the procedure's producing step finds
  g's head: forge-change `producing`), or its merge into main needs g's writer slot: stop at
  what the story shows and report it under `## Left` with the step and state; do not change
  the forge crates (task `mine-forge` owns them).

## Steps

1. Stories first, in `temper/tests/engine/domain/tests/suite/forge_routes/goal_landing.rs`,
   built from `goal_branch.rs`'s `running_goal` and
   `a_change_by_the_goal_into_its_own_branch_lands_there_taking_its_writer_slot` (G's change
   into g, landed), then L as above with `change_tool`'s `land_change` or the steps it
   drives (CI on L's head through the fake forge, the merge):
   `a_goals_landing_lands_its_branch_into_main_whole_leaving_it_held_by_the_goal`,
   `branch_mine_with_landing_goal_is_refused`,
   `branch_mine_without_a_goal_branch_is_refused`,
   `a_second_landing_of_the_same_goal_branch_is_refused_naming_the_first` (and two in one
   call, in the same story or a second).
   See them fail on the trunk's tip (refused as a plan's member it does not resolve, or a
   delegate built without parameter 11), and say so in the report.
2. The code, as above; a unit test in `route/forge_route/tests.rs` beside the
   `task_holdings` tests: `a_change_on_its_requesters_branch_asks_no_write_holding`.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::goal_landing::/)'` (check the count); `heavy cargo nextest run -p
   temper-engine-domain --lib -E 'test(/task_holdings|requesters_branch/)'`; then `heavy
   cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`
   and `heavy cargo nextest run -p temper-engine-domain --lib`.
4. Remove the **Not yet** entry above from `agent.status.md`. `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-domain-world --all-targets -- -D warnings`. Commit ("change: branch mine
   lands its requester's goal branch") and push.

## Report

As the implementer's text says, with the stories' failure on the tip before the code.
