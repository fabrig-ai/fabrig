+++
id = "forgejo-commits-oldest"
part = "commits-capacity"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/commits.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/calls/turn.rs", "skein/docs/design/forge.status.md"]
effort = "high"
minutes = 75
+++

## Goal

Forgejo's `Read::Commits` plan stops at its capacity and answers a page
with `more: true`, where today it holds the whole comparison and refuses a
range past the answer's rows or bytes as `TooLarge`. Forgejo lists a
comparison newest first, so the plan keeps the **oldest** commits that fit,
giving up the newest it holds as older ones come: page 1, oldest first,
stays exact however long the range (`skein/docs/design/forge.md`, 5.2, the
`Read::Commits` row).

Acceptance:

- `skein-forge/src/forgejo/plans/commits.rs` answers as
  `notes/contracts.md`, A, *Forgejo's commits plan*, has it: a page the
  kept commits hold whole is answered oldest first with
  `more = total > skip + page_size`; a page they don't hold whole, or a
  range whose oldest commit alone passes the answer's bytes, is
  `TooLarge`; `total_commits` unequal to the rows given stays
  `InvalidAnswer`; a decoder's `Bound` or `TooLong` stays `TooLarge`. The
  plan never holds more than its capacity (rows and answer bytes), as now.
- The plan's table tests in `plans/tests/calls/turn.rs` hold it (Steps, 1),
  their new rows written and seen failing first.
- The **Not yet** entry 5.2 in `skein/docs/design/forge.status.md`
  ("Forgejo's `Read::Commits` holds the whole range …") is removed, its
  lines alone (`notes/common.md`, *Your status entry*).

## Inputs

- `skein/docs/design/forge.md`, 5.2: the `Read::Commits` row; 3.2, its
  call; 3.4, `TooLarge`.
- `temper/docs/plan/landing-behind-main/notes/contracts.md`, A.
- `temper/docs/plan/landing-behind-main/notes/common.md`.
- `skein/tests/forge/dialect/tests/suite/forgejo_files.rs`,
  `the_recorded_range_keeps_commit_messages_and_parents`: the recorded wire
  order, newest first.
- `skein-lib`'s `Queue` (`skein/crates/skein-lib/src/queue.rs`): a bounded
  first-in first-out list.

## Owns and may change

- Owns the files in `files`.
- Inside `commits.rs` the shape is yours. A sketch: `commits` becomes a
  `skein_lib::Queue<api::CommitMessage>` of the list's capacity, `held`
  stays the kept bytes, and a new `given: u32` counts every row. On a row:
  a commit whose own bytes pass `answer_bytes` empties what is kept (each
  kept commit is newer than it); otherwise pop the front (the newest kept)
  while the row would pass the capacity or the bytes, then push it. At
  `done`, the page from the oldest is answered when the kept commits hold
  it whole.
- Pre-approved: if a memory or size test in `skein-forge` prices the
  plan's list, keep its figure with the queue's price, or, if the queue's
  price differs, update that one figure to the queue's own
  (`Queue::price`) and say so in the report.
- Pre-approved: if a row's bounds make the decoder refuse the range before
  the plan sees its rows (a `Malformed` at `done`), give that row bounds
  the decoder takes and the plan doesn't, and say so in the report.
- Not yours: the GitHub dialect (it pages natively), `Read::Compare`'s
  plan, the decoder, any bound or limit, `forge.md`, temper.

## Steps

1. **The rows, failing first,** in `plans/tests/calls/turn.rs`. Rename
   `commit_messages_fit_exactly_within_the_answer_byte_bound` to
   `a_range_past_the_answer_keeps_its_oldest_commits`, its rows (the
   range fixture: `first`, `second`, `merge`, sent newest first; page 1;
   `RANGE_BYTES` as now; the bytes of a kept commit are its native slot,
   32 per parent and its message):

   | row | bounds | answer |
   |---|---|---|
   | `all_three_at_the_answer_bound` | `answer_bytes: RANGE_BYTES` | `[first, second]`, `more: true` (as now) |
   | `one_byte_under_keeps_the_oldest_two` | `answer_bytes: RANGE_BYTES - 1` | `[first, second]`, `more: true` |
   | `two_rows_keep_the_oldest_two` | `rows: 2` | `[first, second]`, `more: true` |
   | `only_the_oldest_kept_holds_no_whole_page` | `answer_bytes`: `first` and `second`'s bytes less one | `TooLarge` |
   | `the_oldest_alone_past_the_answer` | `answer_bytes`: `first`'s bytes less one | `TooLarge` |

   Keep `commits_reverse_before_paging_and_keep_merge_parents` and
   `a_commit_page_past_the_range_bound_is_refused` as they are, and add a
   row, there or in the table, for page 2 at `rows: 2`, which the plan
   refuses before sending, as now. Run them: the second and third rows
   fail today with `TooLarge`; note it for the report.

2. **The plan** as `notes/contracts.md`, A, has it, its module comment
   saying the plan keeps the oldest commits within its capacity, Forgejo
   sending them newest first (`forge.md`, 5.2). Run the table: every row
   passes. Break the rule once by hand (pop the oldest instead of the
   newest), see `one_byte_under_keeps_the_oldest_two` fail with the wrong
   commits, restore, and name it in the report.

3. **Narrow runs, then the readers.** This changes what Forgejo's answer
   to a long range is, so it runs the tests that read it:
   - `heavy cargo nextest run -p skein-forge --lib -E 'test(/plans::tests::calls::turn::/)'`
   - `heavy cargo nextest run -p skein-forge --lib`
   - `heavy cargo nextest run -p skein-forge-dialect-tests --test suite`
   - `heavy cargo nextest run -p skein-forge-client-world --test suite`
   Check each run's count; one that selected nothing proves nothing.

4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-forge --all-targets -- -D warnings`; remove the status
   entry; commit saying why (an ancestry read of a head far behind its base
   gets a page, not a refusal); push.

## Report

As the implementer's text has it, and: each new row's failure before the
fix; the rule broken for the bite and the failure seen; the plan's shape
inside the sketch; any figure from the pre-approved option; the counts of
the runs in step 3.
