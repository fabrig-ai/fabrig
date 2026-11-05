+++
id = "home-decided"
part = "home-decided"
kind = "feature"
needs = ["away-client"]
files = [
  "temper/crates/temper-client-domain/src/forge/decided.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/decided.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# Home watches what was decided, and reads its tasks

## Goal

Home's facts say what its away form draws: `temper/docs/design/client/README.md`, 3.5 (*Away*, *Home, while a decision shows*), 4.6
(`decided`). The drawing is `home-away`'s.

Acceptance:

- `notes/contracts.md`, *The client*: `forge/decided.rs` but its `overturn`, which is `overturn`'s; `AwayForm`, `DecisionLine`, `DecisionTask`,
  and `Facts.away`.
- **`facts` takes the wall:** `pub fn facts(root: &Domain, project: u32, wall: Wall) -> Facts`, its callers edited for that alone.
- **The watch:** Home opens `decided` as it opens `status`, and leaves it with the page. A watch refused or ended is no rows: Home as it was.
- **Showing:** `at <= wall < at + 24 hours`, a row with `at` ahead of the wall (a clock behind) showing. `showing` counts them; `rows` holds the newest 7, oldest
  first; `expires` is the oldest of all showing's `at` plus 24 hours.
- **`away` is none** while To begin shows, and with no row showing.
- **`span`:** the party's row's `since` and its `ended(wall)`; none with no row delivered. **`landed`:** `status.landed` less the row's `landed`, saturating; none
  with either undelivered.
- **Reads:** each drawn row's task is read, `Read::Task { task }`, as `pages/inbox.rs` reads a card, once per page entry and again for a row a new document brings: a
  title and `DecisionTask`: `Live`, `Done(ended)`, `Ended(ended)`, `Unknown` until read and when refused or `Readout::Limit`.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*), 4.6.
- The models: `pages/home.rs`' `status` watch and `Told` arms; `forge/status.rs`; `pages/inbox.rs`' reads and `pages/goals.rs`' readouts.

## Owns and may change

Owns, as `files` lists them. May also change: `home::facts`' call sites in `temper-client-view/src/pages/home.rs` (with `times.now`), `src/tests/**`,
`temper-client-domain/src/tests/**`, `temper/tests/client/**`, `temper/tools/gallery/**`, for the wall alone; a new field's fixtures.

Options, each pre-approved:

- **The reads' bound:** Home's rows beside Needs' and Goals' must fit jig's `Limits.reads`. If 7 don't fit a shell's (`temper-client-shell-web/src/limits.rs`, `temper-client-shell-terminal/src/limits.rs`,
  `temper-client-tools/src/limits.rs`), raise that shell's `reads` by what they need, regenerate what it moves, and say so.
- **A task's title:** its spec's first line, as `pages/goals.rs` and the inbox have it; use the helper they use.
- **A task that ended:** "done" is what `pages/goals.rs`' `done` reads as; its ended time from the read row.

## Steps

1. `forge/decided.rs`, shaped as `forge/status.rs`. `src/tests/decided.rs`: `a_document_decodes`, `a_refused_or_ended_watch_is_no_rows`.
2. `pages/home.rs`. `src/tests/pages/home.rs`: `home_watches_decided_while_open`, `rows_show_for_twenty_four_hours`, `the_seven_newest_are_kept_oldest_first`,
   `showing_counts_past_seven`, `expires_is_when_the_oldest_stops_showing`, `no_form_while_to_begin_shows`, `span_is_since_and_its_end`,
   `landed_is_since_the_rows_count`, `each_row_reads_its_task`, `a_read_ended_or_done_says_so`, `a_limit_leaves_it_unknown`.
3. The check of the implementer's text, `temper-client-view` and `temper-client-gallery` as dependents. Push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-away/home-decided/foreman-1)

merge the landing trunk and regenerate Story A from the combined code
The merge conflicts in `temper/tests/playground/tests/fixtures/story-a.log`.
Task commit `67586abc10` regenerated it for Home’s decided watch and reads.
The landing trunk also regenerated it in `47e1ec14c6` (decided tool answers) and `562d9a3e1a` (stock goal instructions).
Preserve both sides’ source changes, including the automatically merged fixtures and `home::facts` callers.
Regenerate the conflicting log using `temper/tests/playground/tests/suite/walk.rs`; do not splice its encoded exchanges or retain either side’s recording unchanged.
Run through heavy, one command at a time:
`SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`
Then verify the regenerated recording:
`TEMPER_BUNDLE=none heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_kept_log_replays_natively)'`
This generated fixture is the task’s responsibility under `temper/docs/design/playground.md`, section 9. Include it in the merge resolution and report its regeneration.

## Rebase (2026-11-05, step merge, foreman ui-away/home-decided/foreman-2)

merge the landing trunk while preserving distinct Home watch handles and both sides’ tests
The content conflict is in `temper/crates/temper-client-domain/src/tests/pages/home.rs`.
Trunk commit `15c76c5a7f` changed Home’s handle allocation and session Settings assertions; task commit `67586abc10` added decided watches and wall-aware facts.
Preserve the task’s decision tests and every `home::facts(..., Wall)` argument.
Retain the trunk’s assertions that Settings survives Home, never reopens on reentry, and cannot send a second look.
In `a_watch_opens_with_the_page_and_closes_with_it`, construct the status handle with `Owner::Pages.asker()` explicitly; do not inherit the chats handle’s asker.
Check the automatically merged allocation code: trunk puts status at `number`, which collides with this branch’s decided handle.
In `temper/crates/temper-client-domain/src/pages/home.rs`, retain decided at `number` and status at `number + 1`.
In `temper/crates/temper-client-domain/src/pages.rs`, preserve the existing two-handle reservation before chats (`checked_add(2)`); trunk’s `checked_add(1)` would collide with status.
With that allocation, the status test’s number remains `chats.live.number - 1`.
Preserve the automatically merged view fixtures and wall-aware callers.
Validate through heavy, one command at a time:
`TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-domain --lib -E 'test(pages::home::)'`
Check that tests were selected and both status and decided lifecycle tests pass.
