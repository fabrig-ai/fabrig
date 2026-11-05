+++
id = "writes-states"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/writes.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/calls.rs", "skein/crates/skein-forge-connection/src/tests/writes.rs"]
effort = "high"
minutes = 75
+++

# skein-forge: `writes`'s plan holds its states, and `error` keeps its answers

## Goal

`writes::Plan` onto the note's shape, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. The flag `resolving` and the 14-arm lists over `api::Write` go. Behaviour stays. About 80 production lines fall.

Acceptance:

- The note's search A reads nothing for `writes.rs`: 6 `let` forms at this plan's base. `rg -n 'resolving\b|row\.as_ref\(\)\.expect' skein/crates/skein-forge/src/forgejo/plans/writes.rs` reads nothing.
- `Plan::error(&self, error)` answers what it does today, per state, from the new states: `Timeout` once merged; `failure::before_write` otherwise, with `Timeout` mapped to `Unavailable` on the open-pull states.
- **The placeholder:** `heard`'s `self.error(error)` runs after `next` returns, and `error` reads the state. With the idiom's placeholder in place that mapping would silently change. Work out what `error` needs before moving the state out (a small `Copy` value), or have handlers answer the target state with the result.
- Done's order stays: 409, status, body, `check_pull_ids`, `check_repository`, `keep_document`, resolve, then the next step. `keep_document` feeds `lookup`.
- `Plan::new`'s and `begin`'s use of `tables.repository()` stays: it touches the LRU's recency at begin.
- The pins below pass on the base's code first, in their own commit, and unchanged after.
- `merge_read_429_is_timeout`, `open_pull_read_failure_is_unavailable`, `open_pull_repository_mismatch_is_unavailable` and `reviewers_are_added_before_removed` pass.
- The note's tests pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge.md`, 5.2, 5.4

## Owns and may change

- **Owns:** `writes.rs`. `plans/tests/calls.rs` and `skein-forge-connection/src/tests/writes.rs`, for the pins only, at most 60 lines all told.

## Steps

1. **Pins first,** on the base's code, as rows. Search first: skip a row an existing test holds, and say so.
   - `skein-forge-connection`: a refused write keeps its connection (`Ok(Some(Done(Err)))`), and a malformed answer closes it (`Err`). `Run::hear` flattens the two, so a plan's test can't see it: `reads` has `a_malformed_read_answer_is_invalid_and_closes_its_connection` to copy the shape of.
   - `Plan::error` directly, per state: after `Merged`, and on each open-pull state.
   - `tables.repository()` at begin touches recency: with a full table, a begin on the older repository, then keeping a new one, evicts the other.
   - Done's order: a body whose pull ids mismatch answers before anything is kept.
2. `Open`, `Looking`, `State`. `write` is held only in `New`, `Resolving` and `Writing`.
3. Break a pin by hand: read `error` from the placeholder, see the per-state row red; restore.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge -p skein-forge-connection --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the file; the rule broken and what failed; the pins an existing test already held.
