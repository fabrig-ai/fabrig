+++
id = "store-copy"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-store/src/tests/commits.rs"]
effort = "medium"
minutes = 30
+++

# jig-protocol-store: `Write::Copy`'s two written rules gain tests, on a row that isn't a setting

## Goal

Finding 4 of `temper/docs/plan/debt-22/inputs/debt20-review.md`. `jig/docs/design/protocol/store.md`, 6 now says it, in this plan's own commit: a copy reads the map as its commit found it, and a first key the map doesn't hold fails the commit. The code does this today, and only a settings revision has a test. About 20 lines of test; no production changes.

Acceptance, in `src/tests/commits.rs`, using `committed_note`, `note`, `fixtures::header`, `assert_committed`, `assert_failed`:

- A copy of a row that isn't a setting (`note()`) puts the same bytes under `to`, read raw through `crate::open::get` and `keys::encode`, since a row under another key isn't for `jig_row`.
- A save of `from` then a copy of `from` in one commit leaves the old bytes under `to` and the new under `from`.
- A copy whose `from` the map doesn't hold answers `assert_failed` for its number.
- Making `commits.rs`'s `copy` read the commit's earlier saves isn't asked for. To see the tests bite: make `copy` answer an empty value for a missing key, and the third goes red; make `copy` put under `from` instead of `to`, and the first goes red. For the second: apply the save to the map before the copy, by hand, in a throwaway edit of `operations`; if that edit is more than a few lines, say so and report which you broke instead.
- `heavy cargo nextest run -p jig-protocol-store --lib` passes, with `tests/words.rs` among them: `store.md` is read by tests, and these sentences changed it. `heavy cargo nextest run -p jig-store-world` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, finding 4
- `jig/docs/design/protocol/store.md`, 6, "Writes become operations"
- `jig/crates/jig-protocol-store/src/commits.rs`, `operation`, `copy`, `unique`

## Owns and may change

- **Owns:** the test file.
- **Stays:** `jig/tests/store/src/model.rs`'s `panic!("revision row")`: the core copies only a settings revision, so what the world feeds can't reach it, and a test here doesn't go through that model. `Write::Copy`'s doc comment is already true.
- **If a test shows the code does otherwise than the sentences say,** stop and ask: don't change `commits.rs`.
- `store.md` isn't yours. If `tests/words.rs` goes red on these sentences, say so with the failure: that's the plan's, and a question.

## Steps

1. The three tests, one table or a helper if it shortens them.
2. Break each rule by hand as Acceptance lists; restore.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-protocol-store --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; each rule broken and what failed.
