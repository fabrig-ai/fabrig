+++
id = "answer-measure"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/reads/size.rs", "skein/crates/skein-forge/src/forgejo/plans/tests.rs"]
effort = "medium"
minutes = 25
+++

# skein-forge: Forgejo's plans measure an answer with `crate::measure`

## Goal

The last clause of `skein/docs/development/debt/D021-forgejo-plan-states.md`. `reads/size.rs` re-adds what `crate::measure` adds, and has drifted from it. It keeps its row limits, as a check beside the measure (`skein/docs/design/forge.md`, 3.7: "`rows` is checked beside it"). About 100 production lines fall.

Acceptance:

- `reads/size.rs` keeps `pub fn answer(&api::Answer, u32) -> Result<u64, api::Error>` and `pub(super) fn summary`, with today's signatures: `answer` is `within(answer, limit)?; crate::measure::answer(answer).ok_or(api::Error::TooLarge)`, and `summary` the same on `measure::summary`.
- `within` is one exhaustive match that counts list lengths against `limit`: every row limit it has today, the nested ones included. It keeps `Err(InvalidAnswer)` for `Directory` and for writes' answers.
- Search C of the note reads nothing.
- **A pin first,** one table in `plans/tests.rs`, on the base's code, in its own commit: `Item` with 9 labels and `rows` 8, `Protection` with 9 contexts, and `Pull` with 9 reviewers each answer `TooLarge`; the same at 8 answer a size. Dropping `within`'s label count goes red. `measure::answer` alone answers a size there, so this is what would otherwise change a refusal.
- `tests.rs`'s agreement rows, `plans/tests/job_resolution.rs`, and `skein-forge-connection`'s `tests/reads.rs` and `tests/contents.rs` pass, unchanged.
- The note's tests pass.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, "The searches", "Tests"
- `skein/docs/development/debt/D021-forgejo-plan-states.md`
- `skein/docs/design/forge.md`, 3.7
- `skein/crates/skein-forge/src/measure.rs`, `answer`, `summary`

## Owns and may change

- **Owns:** the files in the header. `crate::measure` stays as it is.
- **The drift:** `size::answer` skips `Pull`'s `dequeued` and `measure::answer` counts it. `reading.rs` always sets it `None` for Forgejo, so nothing a Forgejo path measures changes. If a test's number moves, stop and ask.
- **Stays:** the per-row charges in `commits.rs`, `turn.rs`, `contents.rs` and `reads.rs`, and the agreement tests.
- `reads::size::answer` stays public: `skein-forge-connection`'s tests read it.

## Steps

1. The pin; commit.
2. `within`; the two functions; the `too_many_lines` expect goes if it stops firing.
3. Break the pin by hand as above; restore.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search C's count; the rule broken and what failed.
