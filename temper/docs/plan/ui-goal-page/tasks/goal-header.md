+++
id = "goal-header"
part = "goal-header"
kind = "feature"
needs = ["goal-reads", "goal-look", "made"]
files = [
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-view/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# A goal's header and report

## Goal

`pages/goal.rs` draws a goal's `.col.wide`, its header and its report, every word as
`temper/docs/design/client/README.md`, 5.8, says, in every state. It isn't reached yet: `goal-route` hands over.

Acceptance:

- `pub(crate) fn main(b, root, words, bindings, rows, times)`, the shape of `pages/task.rs`'s `main`, opens `Role::Main`
  and `look::GOAL`; in order, so far, `header`, then `.two` holding `report`.
- The header: `.goal-head`, the meta's items with `<span>·</span>` between (a `Role::Text` per item), the state's word
  in its `GOALS_STATE_*` token as `goals::row` says it, "6 of 9 tasks", "31 of 40 spent", "since 05:40"; `<h1>`,
  the report's headline or `goal::fallback`'s words; the byline; `.acts` with "Draft" and "Chat T2" as 5.8 says.
- The report: `article.report` (`Role::Region` named "Report"), each paragraph, the planning dots and words, Left,
  Risk, a report-less end's words, the byline.
- A document not yet read draws as `task::reading` does.
- The tests below pass in `tests/goal.rs`, each having bitten.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, its section A goal's page, as the chain builds it; `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8, 5.12.
- The mock's `goalPage` (`temper/docs/design/client/greenfield/app.js`).
- `temper/crates/temper-client-view/src/pages/goals.rs`, `row`; `pages/task.rs`, `main`, `reading`.

## Owns and may change

Owns, as `files` lists them. In `pages/goals.rs`, only: factor `row`'s state word and token into a `pub(crate) fn
state_word`, and `Way` stays as it is. `tests.rs`: its `mod` line.

Options, each pre-approved:

- `"Draft"`'s target: `goal::draft_of`'s chat, as `Address::Chat { task }`. "Chat T2": the requester's number.
- Bold `<b>`/`<strong>`: a `Role::Text` with the token (`GOAL_META`'s first child, `GOAL_REPORT`'s), or `Run::Strong` if
  `skein_ui::Run` has it. Terminal text stays plain.
- In the terminal, the meta prints as one line, its items joined by " · " (`words.shell`, as `goals::row` branches).
- `Word`s: add as `words.rs` holds them; move `PHRASES` and `ALL`'s length. Reuse `GoalPlanningLine`, `GoalLandedLine`,
  `GoalTasksLanded`, `GoalStoppedLine` where their words are exact, and change a word `goals::row` shares only if its
  own tests say the same.
- `Role::Figure` named "Planning" holds three `Role::Text` for `.typing`.

## Steps

1. The module, its `mod` line, the words.
2. **Tests,** `tests/goal.rs`, on `tests/fixtures/task.rs`'s `document()` with `priority: Some(1)`, setting `Goal`'s
   `report` and `line` as `temper-client-domain` lets a test (`pages.goal` is public), called through `goal::main`:
   - `the_meta_says_title_state_count_spend_and_since`, and no count with no tasks;
   - one test per `goals::State` for its word and token;
   - `the_heading_is_the_reports`, and `a_planning_goal_names_its_draft`, `a_planning_goal_with_no_draft`,
     `a_stopped_goals_heading`, `a_landed_goals_heading_is_its_results_first_line`, `landed_alone`,
     `a_failed_goals_heading_says_why`, `otherwise_it_counts_what_landed`;
   - `no_byline_with_no_report`;
   - `draft_and_chat_links`: words with both, a task's requester, neither;
   - `the_report_draws_paragraphs_left_and_risk_in_order`, `planning_draws_dots_and_no_left_or_risk`,
     `a_report_less_end_draws_its_result_or_reason`.
3. `heavy cargo nextest run -p temper-client-view --lib -E 'test(goal)'`, then the crate whole.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
