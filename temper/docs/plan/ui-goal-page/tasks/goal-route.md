+++
id = "goal-route"
part = "goal-route"
kind = "migration"
needs = ["goal-bands"]
files = [
  "temper/crates/temper-client-view/src/pages/task.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/tests/suite/task_pages.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-shell-web/task-pages.css",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures/task.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/task.txt",
  "temper/crates/temper-client-shell-terminal/src/client/tests.rs",
]
effort = "high"
minutes = 90
+++

# A tracked task's page hands over to the goal's

## Goal

For `Shape::Tracked` alone, `pages/task.rs`'s `main` calls `pages/goal.rs`'s. The old goal's page leaves, with what only
it drew. `Shape::Agent` and `Shape::HeaderAndPlan` don't move.

Acceptance:

- `main`'s hand-over, as `Shape::Agent`'s; the `dead_code` expectation gone.
- **Gone from `pages/task.rs`,** as no other page reads them: `asked`, `budget`, `latest`, `activity`; `Plan.goal`, its counts
  and its filter's drawing in `Plan::of`, `heading`, `plan`; the `Shape::Tracked` branches in `main`; the Cancel arm in `act`
  if nothing else reaches it. `result`, `header`, `controls`, `dependencies`, `plan`, `amendment`, `outcome`, `act` stay.
- **Gone from `look.rs`, `task-pages.css` and `temper.css`,** with `the_task_tokens_stay_on_their_roles` and the name tables
  following: `TASK_CONTRACT`, `TASK_BUDGET`, `EVENTS`, `TASK_DANGER`, unless a later look shows another reader. `NAMES` stays
  positional: renumber, or leave a retired name, whichever `look.rs` holds to.
- **The domain:** a tracked page reads no card of its own and none for a delegate (`temper/docs/design/client/README.md`, 5.11).
- `golden/task.txt` is re-recorded on a goal: the terminal's fixture `task_page()` builds the goal's page, the regions in
  5.8's order, the bar by name, the fold's rows under it. **Your commit says so.**
  `each_fixture_page_prints_as_its_golden_file` passes; `agent_task.txt` doesn't move.
- `tests/task.rs` and `tests/suite/task_pages.rs` lose the goal's old page and keep the rest (below); `temper-client-view` is
  green whole.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8 (its last rules, for a plan on the other pages), 5.11; 6.

## Owns and may change

Owns, as `files` lists them. May change, for the hand-over alone: `temper-client-view/src/frame.rs` where it reads
`Shape::Tracked`; call sites of what you remove.

**Tests, by what they hold** (from a survey of 100 in `tests/task.rs`; yours may differ a little):

- *A goal's page's own, removed:* the header's "Goal · priority N", "What was asked" and its contract, "Latest activity",
  "Budget and authority", the toolbar's order, its Cancel dialog, the filter and the counts, among them
  `only_a_goals_plan_has_counts_and_a_filter`, and in `tests/suite/task_pages.rs`
  `goal_filter_is_operable_before_the_first_ended_answer`, `live_filter_keeps_counts_and_receives_refreshed_ended_rows`,
  `goal_refresh_keeps_old_rows_and_cut_count_until_answered`, `goal_budget_uses_supplied_amounts_without_recomputing_left`,
  `goal_header_uses_exact_wait_count_and_scoped_role_identity`, `goal_stop_confirmation_hides_other_toolbar_acts`,
  `live_filter_does_not_call_a_folded_only_plan_empty`.
- *What the other pages still draw, re-pointed:* plan rows, their order, dependency lines, held cards in rows, the marks: move
  each to an agent's task or a person's fixture (`priority: None`), its assertions unchanged. `overlapping_ended_and_live_rows_
  use_the_tree_row_once` goes to `tests/goal.rs`' merge if it's there already, otherwise to a procedure's.
- *Agent, procedure, person, read states:* stay as they are.
- `untracking_a_goal_switches_to_the_agent_page` stays: it's the hand-over's other side.
- `temper-client-shell-terminal/src/client/tests.rs`' `quiet_tracked_task_snapshot_*`: keep what they say of the snapshot.

Options, each pre-approved:

- **`Word`s left unread** (`AllPlan`, `LivePlan`, `PlanCounts`, `PlanMoreCounts`, the toolbar's): those of the filter are
  `plan-filter-out`'s; others go now, `PHRASES` and `ALL` moving.
- **`temper/tests/client` and `temper/tools/gallery` go red here** on the old page's words. `goal-worlds` and `goal-gallery`
  own them: list them under `## Left`, and don't run them to green. `heavy cargo check --tests` on them must still compile.
- **A test that's neither clearly a goal's nor another page's:** if its behaviour is in 5.8 or 5.11, keep it re-pointed; if
  neither, remove it and say so.

## Steps

1. The hand-over, then the removals, by the compiler.
2. The domain's skipped cards.
3. The tests, as above; the terminal's fixture; `SKEIN_CODEGEN_WRITE=1` or the test's own writer for the golden, by reading
   `render.rs`.
4. `heavy cargo nextest run -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p
   temper-client-shell-web`; `heavy cargo check --tests -p temper-client-world -p temper-client-gallery -p temper`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, counting the tests removed, re-pointed and kept.
