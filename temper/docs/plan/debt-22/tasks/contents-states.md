+++
id = "contents-states"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/contents.rs"]
effort = "high"
minutes = 60
+++

# skein-forge: `contents`'s plan holds its states

## Goal

`contents::Plan` onto the note's shape, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. Behaviour stays. Production grows by about 20 lines; the cut's cap is 40.

Acceptance:

- The note's search A reads nothing for `contents.rs`: 20 stage comparisons and 2 `let` forms at this plan's base.
- `raw_room` still charges in today's order, contents, then actions, then all. `file_cuts_to_remaining_answer_room` and `directory_entries_fill_the_byte_bound_or_are_cut` pass.
- A leaf's `404` stays `Asked::Leaf` for `Read::File`'s own read only (`skein/docs/design/forge.md`, 5.4's table). `a_raw_miss_reads_the_contents_and_lists_a_directory` passes: `missing` ignores the `Done` body, a malformed one included.
- `compare_distinct_files_still_obey_answer_bounds` passes, and each list keeps its capacity.
- The note's tests pass, with no assertion changed. `skein-forge-connection`'s `tests/contents.rs` runs unchanged, or with a pin added.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge.md`, 5.2, 5.4

## Owns and may change

- **Owns:** `contents.rs`. `skein-forge-connection/src/tests/contents.rs`, for a pin only.
- `reads.rs` reads this plan through its public methods, which stay. `reads-states` holds `reads.rs`.

## Steps

1. A pin, if search finds none: a malformed `Done` on a raw miss still reads as missing. Commit.
2. `Sides`, `Ask`, `State`; `begin`, `row`, `done`.
3. A rule to break: make `Before`'s state count a miss as an error; a test should go red. Report which.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the file; the rule broken and what failed.
