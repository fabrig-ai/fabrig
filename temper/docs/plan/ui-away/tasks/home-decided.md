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
