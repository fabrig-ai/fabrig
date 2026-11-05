+++
id = "goal-reads"
part = "goal-reads"
kind = "feature"
needs = ["needs-base"]
files = [
  "temper/crates/temper-client-domain/src/pages/goal.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
  "temper/crates/temper-client-domain/src/lib.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-tools/src/limits.rs",
  "temper/crates/temper-client-shell-terminal/src/limits.rs",
]
effort = "high"
minutes = 90
+++

# What a goal's page reads and decides

## Goal

The domain of the goal's page: on a tracked task's page, its report, its own task line and its needs are read
and kept, and the pure functions the view words are there. This task draws nothing.

Acceptance:

- `pages/goal.rs` holds what `temper/docs/plan/ui-goal-page/notes/contracts.md`, The goal's page in the domain,
  gives, names and signatures as written.
- `task::Task.goal` is `Some` once `shape` is `Shape::Tracked`, and an agent's task's, a person's and a change's
  pages read nothing new.
- The reads as `temper/docs/design/client/README.md`, 5.8, What it reads, says: one read out at a time, again
  when `last_activity` moves for the report, when `changed` or `last_activity` moves for the line.
- `pages.inbox` runs inside the page, its cards the page's (the note); a choice from a card answers as on Needs
  you.
- The tests below pass, each new one having bitten.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8; 3.5, *A goal's report*, *Goals*.
- `temper/crates/temper-client-domain/src/pages/goals.rs`: `state`, its report reads and `reconcile`, your
  pattern. `pages/report.rs`. `pages/task.rs`'s `Ended`, one read at a time with its stale flag.
- `pages/needs.rs`, `pages/inbox.rs`, `question.rs`, which `needs-base` brought.

## Owns and may change

Owns, as `files` lists them. `pages.rs` and `pages/task.rs` change only for the page's reads and the routing:
`Pages::cards`, `cards_mut`, `page_told`, the opening and leaving of `pages.inbox`.

Options, each pre-approved:

- **Presets:** the page holds the task's handles, these two reads and the inbox's. If a test shows a preset short,
  raise `handles`, `watches` and `reads` in all the presets together, with `temper-client-tools/src/tests/limits.rs`
  moving with them, and say why.
- **The inbox's handles:** take a range that can't meet the task's: `task.next` and `inbox.next` both grow. Any way
  that keeps them apart and `pages.next` past both after leaving will do.
- If `goals::State` or `goals::state` needs a `pub(crate)` helper, add it. Their meaning doesn't change.
- `Told` arms and `Address` arms the compiler asks for are yours, wherever they stand in this crate.

## Steps

1. `Goal`, `own_line_query`, the read loop: the live group's `live_after`, then the ended group's `after`, at most
   8 pages, until a row's `chip.task` is the goal. `Goal.counts`: the line's `delegates_done`, `delegates_total`;
   without it, the design's fallback.
2. The pure functions: `draft_of`, `file_of`, `holds_branch` (read `temper-engine-domain-forge/src/items.rs`,
   `goal_branch`, for the name's form), `line`, `fallback`, `landing`.
3. The inbox inside the page, and the card routing.
4. **Tests,** `tests/pages/goal.rs`, on the crate's own fixtures (`tests/fixtures/task.rs`):
   - `a_tracked_task_reads_its_report_and_line_and_opens_its_inbox`;
   - `an_agents_task_reads_none_of_them`;
   - `the_report_reads_again_when_last_activity_moves_and_only_then`;
   - `the_line_is_found_past_a_page`, `the_line_is_given_up_after_eight_pages`;
   - `the_counts_are_the_lines_and_until_then_what_the_page_holds`;
   - `a_choice_on_a_goals_band_answers_through_its_inbox_card`, by `Act::Needs(needs::Act::Choose { .. })`;
   - `leaving_the_page_closes_its_inbox`;
   - `draft_of_reads_a_lines_start_alone` ("Draft: " in the middle of a line isn't one), its chat, neither;
   - `holds_branch` with `temper/12/g12`, `temper/12/g13` and no resource;
   - `line` in every case of the note, `fallback` in each state, `landing` in each state.
5. **Its readers:** `heavy cargo nextest run -p temper-client-domain`, then `heavy cargo check --tests -p
   temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-tools`.
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with each preset you raised and by what.
