+++
id = "reads-states"
part = "forgejo-states"
kind = "feature"
needs = ["reads-delegates"]
files = ["skein/crates/skein-forge/src/forgejo/plans/reads.rs", "skein/crates/skein-forge/src/forgejo/plans/reads/done.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/edges.rs", "skein/crates/skein-forge-connection/src/tests/reads.rs"]
effort = "high"
minutes = 120
+++

# skein-forge: `reads`'s plan holds its states

## Goal

`reads::Plan` onto the note's shape, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. Its 24 fields become a `State` beside `repository`, `request`, `cut`, `count` and `bound`. `read`, `stage` and `Rows` go, with six 23-arm lists over `api::Read`. Behaviour stays. About 150 production lines fall.

Acceptance:

- The note's search A reads nothing for `reads.rs` and `reads/done.rs`: 4 `let` forms and 11 stage comparisons at this plan's base. `rg -n 'row\.as_ref\(\)\.expect|self\.read\b|\bRows\b' skein/crates/skein-forge/src/forgejo/plans/reads.rs skein/crates/skein-forge/src/forgejo/plans/reads/done.rs` reads nothing.
- The call is matched once, on entry, and states hold what follows.
- The pins below pass on the base's code first, in their own commit, and unchanged after.
- `a_malformed_read_answer_is_invalid_and_closes_its_connection`, `a_missing_branch_prefix_reads_its_404_body_and_keeps_the_connection`, `missing_branch_prefix_is_an_empty_answer_and_keeps_the_repository` and `branch_prefix_past_eight_matches_is_too_large` pass.
- The note's tests pass, with no assertion changed. `full_forgejo_holders_stay_under_the_ceiling` passes: a `reads::Plan` is what the holders hold, so box a large variant first; over budget, stop and ask with the measurement.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge.md`, 3.7, 5.2, 5.4

## Owns and may change

- **Owns:** the files in the header; the two test files for the pins only, at most 80 lines all told.
- **The odd row cells** are decided in the note: a row kind the state's exchange can't decode answers `InvalidAnswer`. If a test sees that, stop and ask.
- **Stays:** `after()` is read at every exchange's start (`skein-forge-connection/src/component/calls.rs`): hold it in `Issue` and `Comments`. The per-row charge here stays.

## Steps

1. **Pins first,** on the base's code, as rows. Search first: skip a row an existing test holds, and say so.
   - `tables.repository()` at begin touches recency, for `Locate` and `Settings` too.
   - `row` goes: count, bound, then `keep_row`, then validate. A row over the bound keeps nothing.
   - The leaf: `Read::Branch`'s own read is `Asked::Leaf`, not its resolution, nor `Read::Protection`'s branch read (`skein/docs/design/forge.md`, 5.4's table).
   - `Items` needs a `Date` on its own head only, not on resolution's.
   - `Collaborators` end through `Step::Done` with no second measure, if `terminal` ends up shared.
2. `State`, `Whose`, `Roster`, then `begin`, `row`, `done`. The compiler leads from there. `Delegate` moves in.
3. Break a pin by hand: make the resolution's read `Asked::Leaf`, see its row red; restore.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge -p skein-forge-connection --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the two files; the rule broken and what failed; the pins an existing test already held; the measured heap ceiling before and after.
