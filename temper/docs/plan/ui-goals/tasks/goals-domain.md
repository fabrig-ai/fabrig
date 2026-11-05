+++
id = "goals-domain"
part = "goals-domain"
kind = "feature"
needs = ["goals-address", "report-parse", "delegate-counts"]
files = [
  "temper/crates/temper-client-domain/src/pages/goals.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goals.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
]
effort = "high"
minutes = 90
+++

# Goals reads what its rows need

## Goal

On `Address::Goals`, the domain holds the project's goals, each one's report, and which need the person, and says each
one's state. The domain alone: the page is drawn by later tasks.

Acceptance:

- `pages/goals.rs` as `temper/docs/plan/ui-goals/notes/contracts.md`, The domain's page, gives it; `Pages::goals`;
  `Act::GoalsFilter` and `Act::MoreGoals`.
- **The query:** a live watch of `Subject::Tasks { query, window: limits.client.domain.rows }`, and a read of its ended
  group by `QueryOrder::Ended`, as `Chats` (`pages/chats.rs`), reconfigured from `me` in `step` after each event. Refreshing
  the first ended page as the watch changes, as `Chats`.
- **"More goals":** `Act::MoreGoals` reads the next ended page in place of the last, as `Chats::older`; `count` keeps counting
  every page read.
- **A report per row, one at a time:** one handle; the next row in row order whose `last_activity` differs from the one its
  report was read at gets `report::read`, answered by `report::newest` and `report::parse`. Notes have no watch: a moved
  `last_activity` reads again. An answer that isn't a value is no report, read again only as `last_activity` moves.
- **Needs:** `Read::Inbox { group: Needs, project: Some(project), kind: None, goal: None, after: None }`, as the watch or
  `me.needs_total` moves. `needs_person(task)` holds where any `selection` row's `goals` names it; a `selection` of `None` names none.
- **`state`,** for every row of `temper/docs/design/client/README.md`, 3.5's table, first match wins. A filter is a move: the
  page stays, its reads kept, as `Chats`.
- Leaving releases every handle, on a new address and on `Event::Left`, and `pages.next` passes them.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md` (The report, The domain's page, A line's counts).
- `temper/docs/design/client/README.md`, 3.5, *Goals*, *A goal's report*.
- `temper/crates/temper-client-domain/src/pages/chats.rs`; `pages/inbox.rs`; `pages/task.rs`, `row_state`.
- `jig/crates/jig-client-domain/src/contract/chip.rs`, `chip_phase.rs`, `task_hold.rs`, `inbox.rs`.

## Owns and may change

Owns, as `files` lists them. `pages.rs`: `pub mod goals`, `Pages.goals`, its arms in `went`, `told`, `page_told` and
the reconcile in `step`, as `Chats` has them; the `is_none` guard in `went`.

May also change `temper-client-domain/src/tests/fixtures*` (a goal template, a goal's line) and the view's `tests/fixtures.rs` for
the same, and `temper-client-view/src/root.rs` nowhere.

Options, each pre-approved:

- If `Requester` `None` in the query is refused by `task_query::admission` at a project's member, read the refusal's reason,
  and if it asks for a project role the person holds, the test's fixture gives it; otherwise it's a `Question:`.
- A filter-only move to the same project early-returns in `went`, as `Chats`' does; a move to another project rebuilds.
- `count` as a `u32`, checked as the subset says.
- Rows held for reports are bounded by `rows` live and `rows` of a page: size the list from them.

## Steps

1. **`pages/goals.rs`:** `query`, `state`, `shown`; then `Goals::new(first: u64)`, `leave`, `configure`, `told`, `older`.
2. **`pages.rs`:** as above.
3. **Tests,** `tests/pages/goals.rs`:
   - `goals_read_its_query_and_release_it`, with the query's fields, and a later page's handles new.
   - `each_state_of_the_table_reads`: every row of the table, its first-match order, one test per row or a table.
   - `reports_are_read_one_at_a_time_and_again_as_activity_moves`.
   - `no_report_is_no_error`.
   - `needs_come_from_the_selection_and_read_again_as_needs_total_moves`.
   - `more_goals_reads_the_next_page_and_keeps_the_count`.
   - `a_filter_is_a_move_that_keeps_the_reads`.
4. **Check,** the implementer's text (`temper-client-domain` and its dependents), then push.

## Report

As `notes/common.md`, Your report, with whether the live watch's `more` was reachable in the fixtures.
