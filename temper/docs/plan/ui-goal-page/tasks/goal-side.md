+++
id = "goal-side"
part = "goal-side"
kind = "feature"
needs = ["goal-header"]
files = [
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/task.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
]
effort = "high"
minutes = 90
+++

# A goal's side

## Goal

`aside.side`, beside the report: Tasks, Landing, How it runs, Decided by the goal, as
`temper/docs/design/client/README.md`, 5.8, says.

Acceptance:

- **Tasks:** the direct delegates alone, the tree's depth-1 rows and the ended page, merged by number, cancelled ones
  left out; the bar, `Role::Image` named "N landed of M", a span per task; each not yet landed in order, its link then
  its `goal::Line`'s words; the fold, `Role::Disclosure` "N landed", folded, its rows with their end in the clock form;
  "Proposing tasks from the draft…" or "No tasks."; "More of its tasks are not shown." past a page.
- **Landing:** `goal::landing`'s words; the count in words through nine, then a digit.
- **How it runs:** the way, then " · always asks you".
- **Decided by the goal,** only with a `Decided:` line, its items joined by " · ".
- The tests below pass, each having bitten.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8.
- The mock's `goalPage`.
- `pages/task.rs`, `Plan::of`, `EndedRows`, `tree_order`: how the tree and the ended page merge today.

## Owns and may change

Owns, as `files` lists them. In `pages/task.rs`, only `pub(super)` on what you reuse, or moving a helper; nothing it
draws changes.

Options, each pre-approved:

- **"waits on TITLE":** `goal::Line::WaitsOn(n)`: the title of row `n` among these rows; not among them, "waits on task N".
- **`yours`:** a card of the page's inbox whose `inbox::source_task` is that delegate, per `needs::Session::open`.
- **The word for 1:** "Lands whole once all one are checked.", literally. The request fixes the form.
- **A delegate's own delegates** in the tree: left out by depth; use `tree_order` or read `row.parent`, whichever is plainer.
- **A landed row with no ended time** (a tree row, ended since it was delivered): its link alone.

## Steps

1. The merge, returning rows in order; the lines; the bar; the fold.
2. Landing, How it runs, Decided.
3. **Tests,** `tests/goal.rs`:
   - `tasks_merge_the_trees_rows_and_the_ended_page_by_number`, a delegate in both drawn once;
   - `a_delegates_own_delegates_and_cancelled_ones_are_left_out`;
   - `the_bar_is_named_and_classed` with landed, running and neither;
   - one test per `goal::Line` for its words, "waits on TITLE" and "waits on task N" both;
   - `landed_rows_fold_under_n_landed_with_their_end`;
   - `no_tasks_while_planning_and_after`; `more_of_its_tasks_past_a_page`;
   - `landing_says` each case; `lands_whole_through_nine_and_past`, 1, 9, 10;
   - `how_it_runs_says_its_way_by_its_branch`;
   - `decided_only_with_a_decided_line`.
4. `heavy cargo nextest run -p temper-client-view --lib -E 'test(goal)'`, then the crate whole.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
