+++
id = "window-pages"
part = "window-pages"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/loads.rs", "temper/crates/temper-engine-domain/src/archive.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/tests/engine/domain/tests/suite/preparation.rs", "temper/tests/engine/domain/tests/suite/loads.rs", "temper/docs/design/domain/root.status.md"]
effort = "high"
minutes = 120
+++

## Goal

A transcript window longer than one load page loads whole whatever else
holds the load table: each later page is issued in the slot its earlier
page held, so room never fails a window. On the trunk a window is read
`root.loads.rows` (2) turns a page; the page's own slot stays retired
until the pass's reclaim, so when the table's other slot (`root.loads.loads`
is 2) is held, the next page's `loads::begin` is refused, `request_load`
answers `transcript_failed`, and the core fails the brief with
`transcript: its window could not be loaded`, transient, again at each
retry while the window stays longer than a page. That is the request's
held goal, its short goal (a few turns are already two pages), and its
restarts' minute of failures, when due tasks read their windows together.
Failing rows first.

Acceptance:

- The failing rows (Steps, 1 and 2) are committed first, alone, and the
  report quotes their failure on the trunk: which row failed, with the
  words `transcript: its window could not be loaded`.
- A window of any length loads whole while another load holds the
  table's other slot: no brief fails, no try is spent, and the next
  assignment carries every turn of the window, in order.
- A restart's due tasks whose windows span two pages each load their
  window and resume, none failing its preparation, none spending a try.
- A preparation that ends between its window's pages lets its slot go:
  no slot leaks (Steps, 4).
- The startup walk (waiter `u64::MAX`), the census and every other load
  keep their behaviour; `loads`' fence test
  (`abandoned_io_keeps_capacity_until_its_real_terminal_and_old_tokens_are_fenced`)
  still holds.
- The **Not yet** entry `8: a window's pages keep their load slot ...` in
  `temper/docs/design/domain/root.status.md` is removed, its lines alone.
- The report says which bound refused (`root.loads.loads`, with the page's
  slot retired until reclaim), and whether the restart's minute is this
  path (Steps, 2 says how to tell).

## Inputs

- `temper/docs/design/domain/root.md`, 8: *Loads for the core*, *An
  issued transcript load that fails*, *A window's pages keep their load
  slot* (this plan's), and 9.1, step 7.
- `jig/docs/design/domain/engine.md`, 7.2 (*Resuming*: the window, read
  for resuming or for the tail), 9.1 (the table's transcript row), and 16
  (*A transcript's load with no slot*: no capacity answer, so a root never
  lets room fail a load).
- `jig/docs/plan/transcript-window/notes/common.md`.
- `jig/docs/plan/transcript-window/inputs/request.md`, items 1 and 2.

## Owns and may change

- Owns the files in `files`.
- Reader files, call sites only: `temper/crates/temper-engine-domain/src/route.rs`
  (`activate`'s `busy`, if your mechanism changes what `has_room` counts),
  `temper/crates/temper-engine-domain/src/domain.rs` (`reclaim`, one line
  if your mechanism releases there), `temper/tests/engine/domain/src/direct.rs`
  (a driver helper the rows need, such as many turns in one attempt).
- Pre-approved:
  - **The mechanism, default:** `loads::loaded` leaves the entry of a
    non-startup `CoreRange::TaskTranscript` page that names a `next`
    (and is not cut) held for its waiter, not retired (a new `Phase`,
    such as `Paging`); the archive's continuation re-arms it with a new
    `pub fn` in `loads.rs` (such as `next(domain, waiter, after, most,
    out) -> Option<Token>`), which issues the next `Request::Load` in that
    entry; an entry not re-armed by the end of its pass is retired at
    `reclaim`, so a waiter that does not continue (a stale preparation, a
    seam cut, a refused row) never leaks it. The re-armed page may keep
    the entry's owner token: each issued request still has exactly one
    terminal. **Another mechanism** holding the same guarantees (a
    continuation never refused for room; no leak; fences hold) is yours
    to choose; name it in the report. Raising `root.loads.loads` or the
    slab's capacity is not one: it is a figure.
  - **Other paged core ranges** (`Turns`, notes `Lines`): if your
    mechanism covers them at no cost, say so; otherwise they keep their
    behaviour, and the report names them.
  - **`a_transcript_continuation_without_load_room_remains_preparation_failure`**
    in `archive.rs`'s tests: it saturates the table and calls
    `request_load` for a continuation directly. Turn it into the rule now
    true (a page answered with a `next` while every other slot is taken
    issues its continuation, and the core hears no `Failed`), renamed to
    say so; if a continuation can still reach `request_load` without a
    held entry, keep a row for that path too and say why it can.
- Not yours: `temper-service` (its routing and limits stay), `jig/**`,
  every figure.

## Steps

1. **The failing table, first.** In
   `temper/tests/engine/domain/tests/suite/preparation.rs`, a table test
   `a_window_of_any_length_loads_while_another_load_holds_a_slot`, with
   the shipped `root.loads.loads` (2) and the walking world's
   `root.loads.rows` (2). Each row parks a chat whose window holds N turns
   (several turns in one attempt, then `tasks::End::Parked`, as
   `park_turn` does for one), parks a second chat with one turn, wakes
   both, holds the second chat's transcript read in flight
   (`transcript_in_flight`, its store answer left queued), then settles
   the first chat's pages. Rows, by the window's length against the page:
   - N = 2: one page, just inside; passes on the trunk (a guard);
   - N = 3: two pages, just past; fails on the trunk;
   - N = 7: four pages; fails on the trunk.
   Each row asserts the first chat's next assignment carries its N turns
   in order, no `Fact::BriefFailed` names `transcript: its window could
   not be loaded` (`no_transcript_failure`), and its tries stay
   `tasks::Tries::NONE`. The walking world's resume bound
   (`root.journal.transcript_bytes`, 8192 in `walking.rs`) holds N short
   turns whole; keep each turn's words short, never raise it.
2. **The restart's row.** Make
   `tasks_due_together_past_the_load_slots_each_load_their_window` a
   table over the parked chats' window length: 1 turn (the existing case)
   and 3 turns (two pages). `parked_chats` takes the length. The 3-turn
   row fails on the trunk if the restart's minute is this path; if it
   passes there, the report says the restart's minute is another path and
   what the row showed.
3. **The rule** (`Owns and may change`, the mechanism), in `loads.rs`,
   `archive.rs` (`transcript_loaded`'s continuation) and `restart.rs`
   (`request_load`, if the continuation goes through it). The module
   comment of `loads.rs` says a window's pages keep their slot
   (`temper/docs/design/domain/root.md`, 8).
4. **No leak.** In `temper/tests/engine/domain/tests/suite/loads.rs`, or
   as a row of the table above: a preparation that ends between its
   window's pages (a message to the task while its first page is in
   flight, as
   `a_transcript_read_landing_after_a_message_ended_its_preparation_is_dropped`
   does) leaves the table with no slot taken after the pass's reclaim
   (`Loads::quiescent`), and the task's next preparation loads its window.
   The archive's unit test (`Owns and may change`).
5. Break the rule once by hand (retire the paging entry at `loaded`, as
   the trunk does), see the N = 3 row fail for that reason, restore; name
   it in the report.
6. **Runs.** `heavy cargo nextest run -p temper-engine-domain --lib`;
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^preparation::/) | test(/^loads::/) | test(/^restore_fit::/) | test(/^switch::/) | test(/^upgrade_transcripts::/)'`.
   This changes when a load slot frees, which every story that loads
   sees: also `heavy cargo nextest run -p temper-engine-domain-world`
   whole, and `heavy cargo nextest run -p temper-service -E 'test(/restart|preparation|brief|store_route|transcript/)'`.
   Check each run's count.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`;
   remove the status entry; commit saying why (a window's later pages
   keep its slot, so room never fails a long window's brief); the
   citations check; push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: the failing rows' output on
the trunk (Steps, 1 and 2), quoted; the bound that refused; whether the
restart's minute is this path, by the row; the mechanism you chose and
why; the rule broken by hand and what failed; the runs and their counts;
the branch and commit pushed; implementation and check minutes apart.
