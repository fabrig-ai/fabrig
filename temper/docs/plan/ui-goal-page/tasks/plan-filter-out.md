+++
id = "plan-filter-out"
part = "plan-filter-out"
kind = "migration"
needs = ["start-sheet"]
files = ["temper/crates/**", "temper/tests/**", "temper/tools/gallery/**", "temper/tools/journeys/**"]
effort = "medium"
minutes = 60
+++

# `PlanFilter` leaves the address

## Goal

Only a goal's old page drew it. `Address::Task { task, filter }` becomes `Address::Task { task }`, and `/tasks/9?state=live` reads as `/tasks/9`, as `temper/docs/design/client/browser.md`, 2.3, says.

Acceptance:

- No `PlanFilter`, `Act::PlanFilter`, `Word::AllPlan`, `Word::LivePlan`, `Word::PlanCounts` or `Word::PlanMoreCounts` in the tree, `Task.filter` gone, `plan_filter` gone; `Word::AllPlan`'s shared arm keeps `AllModels` and `AllChats`.
- `address.rs` drops `state` on a task's address, printing the plain one, with its test: `a_tasks_state_query_reads_as_the_plain_address`.
- `temper/docs/design/client/browser.status.md`'s 2.3 line is gone.
- Every package you touched passes; the compiler's the survey.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`.
- `temper/docs/design/client/browser.md`, 2.3.

## Owns and may change

Owns every site: at the cut, about 110 in `temper-client-domain`, `temper-client-view`, `temper-client-shell-terminal`, `temper/tests/client`, `temper/tests/one-process`, `temper/tools/gallery`, `temper/crates/temper/tests`. Each edit is only for the field. Remove `use` lines it leaves unread.

Options, each pre-approved:

- **`sed` is fine:** `Address::Task { task, filter: PlanFilter::All }` to `Address::Task { task }`, and `{ task: N, filter: PlanFilter::All }` likewise; `cargo fmt` after. By hand what's left.
- **Tests only of the filter,** in `temper-client-domain`: `goal_filter_moves_without_wire_work_and_the_plain_address_reopens_all`, `plan_filter_moves_keep_the_ended_page_and_drafts_without_reading_again`, the address's `state=live` cases: removed, and named in your report.
- **An other plan's line,** after a catch-up, the same edit.

## Steps

1. `sed`, then the compiler: `heavy cargo check --tests -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-tools -p temper-client-world -p temper-client-gallery -p temper`.
2. The address's test, by the design.
3. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: counts of sites, tests removed.
