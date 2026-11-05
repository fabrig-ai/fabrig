+++
id = "tasks-pins"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-tasks/src/stored/models.rs", "jig/crates/jig-domain-tasks/src/value.rs", "jig/crates/jig-domain-tasks/src/tests/revisions.rs", "jig/crates/jig-domain-tasks/src/tests/startup/detail.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-tasks: two tests that can't fail go, one refusal gains its pin, `LIMITS` is one

## Goal

Findings 1 (the `LIMITS` line), 2 (first and fourth bullets) and 4 (the last sentence) of `temper/docs/plan/debt-22/inputs/debt20-review.md`. It was read, not run: check each first. Net test lines fall by about 90.

Acceptance:

- `Limits::word_bytes` is the largest of `message_bytes`, `result_bytes` and the notice's length. The `.max(self.inbox_bytes.min(OWED))` term is gone: the reminder is shorter than the notice, so it never wins and no value changes.
- `const _: () = assert!(crate::questions::OWED_WORDS.len() < crate::run::NOTHING_AWAITED_WORDS.len());` stands beside `word_bytes`, the form `jig/crates/jig-domain-core/src/limits.rs` uses, so lengthening the reminder past the notice stops the build instead of under-bounding a word.
- `word_bytes_covers_fixed_notices_and_caller_limits` keeps its `NOTICE - 1` and `NOTICE + 1` bounds and its three columns. `REMINDER` goes. Removing the notice's `.max` and removing `message_bytes` from `word_bytes` each go red.
- `remap_models_updates_a_live_authority_and_preserves_all` goes (see Steps), and `Models::remap` has a table test.
- `rg -n 'const LIMITS' jig/crates/jig-domain-tasks/src` finds one, in `tests/revisions.rs`.
- `FinishRequested` has a load-refusal row in `tests/startup/detail.rs`: three of them refuse with `bound(B::AmendmentSlots, 3, 2)`. Moving `Slot::FinishRequested` to the ordinary arm in `stored/fit.rs` goes red.
- `heavy cargo nextest run -p jig-domain-tasks` passes; `heavy cargo check -p jig-protocol-store --tests` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, findings 1, 2, 4
- `jig/docs/design/protocol/store.md`, 5.1

## Owns and may change

- **Owns:** the files in the header.
- **May change:** `jig/crates/jig-domain-tasks/src/tests.rs`, only the line `mod revisions;` to `pub(crate) mod revisions;`; `tasks-families` holds the rest of that file. `LIMITS` in `tests/revisions.rs` becomes `pub(crate)`.
- **Readers:** `word_bytes` is read by `jig/crates/jig-protocol-store/src/codec/tasks.rs`; its values don't change, so that file doesn't.
- `limits::tests` is `pub(crate)` only for `stored/models.rs`. If nothing else reads it, it becomes a private test module.
- **Options:** the arms of `Stored::remap_models`, see Steps.

## Steps

1. `limits.rs`: the term, the const assertion, the test's rows; one `LIMITS`, read as `crate::tests::revisions::LIMITS` by this test and by `stored/models.rs`.
2. A `Models::remap` table beside the type or in `stored/models.rs`'s tests, about 15 lines: `All` stays; two bits merging into one; a departing bit (0 in `moved`) drops; bit 63 moves; `Named(0)` stays. Removing a `result |= today` or the `All` arm goes red.
3. Replace the 64-line test. `Stored::remap_models` has arms the old test never reached: `proposal` and `recurring` on a live row, `History`'s `proposal` and `amendment`, `PersonProposal`. **Default:** one short table over those arms using fixtures already in `crate::tests` (`revisions::Harness`, `revisions::task`, `tests/startup/detail.rs`'s rows), at most 25 lines, where each row sets one retained authority to `Named(1)`, remaps with `moved[0] = 2`, and reads `Named(2)` back. Deleting that arm from `remap_models` goes red, once per arm. **If** an arm can't fit that cap, leave it unpinned and name it in the report: the review's own fix keeps only the `Models::remap` test, and the request lets a test that can't bite go.
4. In `tests/startup/detail.rs`, beside the `Amendment` case, the `FinishRequested` row: a loop over the two kinds is fine.
5. Break each rule by hand as Acceptance lists; restore.
6. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; each rule broken and what failed; which `remap_models` arms bite and which don't.
