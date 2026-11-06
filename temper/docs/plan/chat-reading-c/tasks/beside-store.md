+++
id = "beside-store"
part = "activity-later"
kind = "feature"
needs = ["beside-surface"]
files = [
  "jig/crates/jig-protocol-store/src/reads/paged.rs",
  "jig/crates/jig-protocol-store/src/tests/reads/activity.rs",
  "jig/crates/jig-domain-core/src/tests.rs",
  "jig/tests/client-protocol/tests/suite/**",
  "jig/docs/design/protocol/store.status.md",
  "jig/docs/design/client/domain/contract.status.md",
]
effort = "high"
minutes = 45
+++

# The store walks a task's activity upward from a position

## Goal

`Read::ActivityBeside { side: Later }` answers the lines just after a position, within the page's bounds, and says whether more remain. This is the read that brings the newest lines back, through the real engine and store.

Acceptance:

- `paged::activity` with `Range.backwards` false walks `position + 1` upward, at most `capacity` rows, stopping at the same bounds as the backward walk (the encoded bytes, a turn's join, a cut row). `next` is the last row taken when rows remain above it or a bound stopped the walk, and none at `owner.activity`.
- Past the end, an empty page and no cursor. A first row a bound refuses leaves a cursor at the position read from, and retries, as the backward walk's first row does.
- `Earlier` at p and `Later` at p - 1 are a seam: the two pages hold no line twice, and miss none.
- In `jig/tests/client-protocol`, through the real engine and native bytes: a client reads a task's end, then earlier beside its oldest line, and then later beside a line it holds, and has every line once, with `after` none at the end.
- The clause `7.4: the lines after a position` leaves `jig/docs/design/protocol/store.status.md`'s **Not yet**, and the line `6: A page beside a line` leaves `jig/docs/design/client/domain/contract.status.md`.

## Inputs

- `jig/docs/design/protocol/store.md`, 7.3 and 7.4, *Reading backwards*.
- `jig/docs/design/client/domain/contract.md`, section 6, *A page beside a line*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *The activity read*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

`jig/docs/design/protocol/store.md` is read by `jig/crates/jig-protocol-store/src/tests/words.rs`: run it, as this plan changed that file's words.

## Owns and may change

Owns the files in the header. May also change, these edits alone:

- `jig/crates/jig-domain-core/src/reads/rows/positions.rs`, `reads/rows.rs`: where a check on a page's `next` assumes a backward page for a dense family, so that a forward page passes it on the same rule. If the check holds as it is, leave them.
- `jig/crates/jig-protocol-store/src/transcripts/read.rs`: a turn line's read goes by `range.after`; if a forward page needs anything else, the least, said in the report.

Options, each decided:

- If the walk shares enough to be one loop over positions going either way, make it one; if that would change the backward path's lines, keep two, and share the row's taking.
- If the story's window doesn't fit `jig/tests/client-protocol`'s budget as it stands, make its task smaller rather than the budget larger.

Not this task: `Thread`, any temper file.

## Steps

1. Red first: `later_beside_a_line_walks_upward_within_its_bounds`, `later_past_the_end_is_empty`, `a_bound_stops_a_later_page_with_a_cursor_that_retries`, `earlier_and_later_meet_with_no_line_twice` in `tests/reads/activity.rs`; the world's `a_page_beside_a_line_reads_both_ways_through_the_engine`.
2. The walk; the one line `beside-surface` left goes.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-store -p jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-store -p jig-domain-core`; the world's narrow test, and `heavy cargo nextest run -p jig-client-protocol-world`. Commit and push.

## Report

As the implementer's text asks, with what a forward page's cursor holds, and the world story's wall time.
