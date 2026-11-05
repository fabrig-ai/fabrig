+++
id = "actions-states"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/actions.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/job_resolution.rs"]
effort = "high"
minutes = 60
+++

# skein-forge: `actions`'s plan holds its states

## Goal

`actions::Plan` onto the note's shape, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. Behaviour stays. Production grows by about 20 lines; the cut's cap is 40.

Acceptance:

- The note's search A reads nothing for `actions.rs`: 13 stage comparisons and 1 `let` form at this plan's base.
- `job_cuts_to_remaining_answer_room` passes: the step's name is charged in `Job` and `Log`, as today.
- `plans/tests/job_resolution.rs` passes, its assertions unchanged. Its call of `reads::size::answer` is `answer-measure`'s, and doesn't change here.
- The note's tests pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** `actions.rs`; `plans/tests/job_resolution.rs`, for a pin only.
- `reads.rs` holds this plan in a variant and reads it through its public methods, which stay.

## Steps

1. `Checks`, `Shape`, `State`; `begin`, `row`, `done`. `Mode` goes.
2. A rule to break: make `RunLookup` keep the first run it sees without checking `run_number`; a test in `job_resolution.rs` should go red. If none does, that's a pin, one row.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the file; the rule broken and what failed.
