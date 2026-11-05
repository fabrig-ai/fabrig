+++
id = "stock-after-asking"
part = "stock-words"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock/coding.txt"]
effort = "medium"
minutes = 15
+++

## Goal

A coding run read its instructions as allowing `finish` with `failure` to end its run
after asking. Say how to end after asking, in the fewest words that hold.

Acceptance: `coding.txt` holds the sentence below; `temper-service`'s tests pass.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11, *Its own agents' words*.
- `temper/docs/plan/producer-failure/notes/common.md`.

## Owns and may change

Owns `temper/crates/temper-service/src/stock/coding.txt`.

May change: `temper/crates/temper-service/src/tests/stock_brief.rs`, only a measured
figure the longer text moves, and only if the test says so. The report says what moved
and by how much. A pin of the sentence goes in
`temper/crates/temper-service/src/tests/stock.rs`, one test added.

Options: if a reader in `temper/crates/temper/src/tests/reference_settings.rs` or
`temper/tests/engine/domain/tests/suite/settings/b2.rs` compares bytes it reads from
the same file, it needs nothing. If a bound refuses the text, stop and ask; budgets
are the user's.

## Steps

In `coding.txt`, directly after `message your change only in the question form.`, add:

`After asking, call wait, not finish: the answer reaches this run.`

Change nothing else in the file. A test in `tests/stock.rs` asserts charter 2's
instructions hold that sentence; break it once, see it fail, restore.

Narrow test, `cargo fmt`, clippy on `temper-service`, its focused tests, then
`heavy cargo nextest run -p temper --lib -E 'test(reference_settings)'` and the `b2`
settings tests, as they read the file. Commit, push.

## Report

What `stock_brief.rs` moved by, or that it did not.
