+++
id = "goal-worlds"
part = "goal-worlds"
kind = "feature"
needs = ["goal-route"]
files = [
  "temper/tests/client/tests/suite/task.rs",
  "temper/tests/client/tests/suite/task_pages.rs",
  "temper/tests/client/tests/suite/b2.rs",
  "temper/tests/one-process/**",
  "temper/crates/temper-client-domain/tests/suite/task_pages.rs",
  "temper/crates/temper/tests/support/person.rs",
  "temper/crates/temper/tests/support/forge_land.rs",
]
effort = "high"
minutes = 90
+++

# The joint worlds walk the new page

## Goal

The worlds that walked a goal's old page walk its new one, keeping each story's point: a goal's delegates reached from it, a
held delegate released on its own page, a goal finished.

Acceptance:

- `heavy cargo nextest run -p temper-client-world` passes whole, and the domain's `tests/suite/task_pages.rs`.
- The stories as `temper/docs/design/client/README.md`, 10, words them, each still proving its story through both real sides.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8, 5.11, 10; `temper/docs/design/domain/testing.md`, 2.7.

## Owns and may change

Owns, as `files` lists them. What each story becomes (the survey's list; the compiler and the runs win):

- `a_goals_page_shows_its_plan_with_the_held_task_marked`: its task line says "held". `a_held_task_released_from_a_goals_plan_runs_again`:
  from the goal's task line to the delegate's page, Release there, running again.
- `a_goals_plan_says_which_task_a_delegate_waits_for`: "waits on TITLE". `a_finished_dependency_says_completed_in_its_dependants_line`: the
  delegate's own page's line.
- `a_goal_cancelled_from_its_page_ends_its_delegates`: "Stop", its dialog, delegates ended, "Stopped before it landed."
- `a_finished_goals_page_shows_its_report`: the landed heading and result, its name as it now holds.
- `a_goals_specification_amended_from_its_page_reaches_the_engine`: stays; the editor's words are unchanged.
- `a_delegates_page_shows_its_brief_run_and_report`, `a_client_closed_on_a_tasks_page_opens_at_the_inbox_again`: from the goal's link.
- `a_forge_panel_on_a_goals_page_says_shown_on_the_forge`: a change's page, or drop its goal's half if the goal's page no longer draws
  it, and say so.
- `goal_finishes_while_live_filter_keeps_its_result_and_counts`, `fresh_ended_goal_reload_restores_all_filter_for_its_page`: the filter
  has gone from the page; the result and counts remain, without `?state=live`, which `plan-filter-out` removes from the address.
- `roadmap_b2_through_the_task_pages`: as 10 says; rung B2's asserts move to a delegate's own page where the goal's no longer draws them.
- In the domain's `tests/suite/task_pages.rs`, `goal_filter_moves_without_wire_work_and_the_plain_address_reopens_all` stays until
  `plan-filter-out`; `#[ignore]`d backlog tests stay as they are.

Options, each pre-approved:

- **A world's assert on a phrase now elsewhere:** follow the phrase to where the design says it is. If it's nowhere on any page, it was
  the old page's alone: drop that assert, and name it in your report.
- If `temper/docs/design/domain/testing.md`, 2.7, says a rung's words that moved, edit that line, and say so.
- If `Pause` or `Stop`'s domain isn't reachable yet (`goal-acts` follows): `Act::Close` and `Act::Confirm` reach Stop, whose dialog words
  may still be the old "Cancel" until `goal-acts`: assert the ending, not the dialog's words, and leave those to `goal-acts`.

## Steps

1. Run the world, read the first failure's log, repair; one at a time.
2. `heavy cargo nextest run -p temper-client-world -p temper-client-domain`; `heavy cargo nextest run -p temper-one-process -E
   'test(m15)'`, taking the package's name from `temper/tests/one-process/Cargo.toml`.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: each story changed, in a line, and each assert dropped.
