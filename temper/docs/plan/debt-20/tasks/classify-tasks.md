+++
id = "classify-tasks"
part = "classifiers"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-domain-tasks/src/owned.rs", "jig/crates/jig-domain-tasks/src/inbox.rs", "jig/crates/jig-domain-tasks/src/activity.rs", "jig/crates/jig-domain-tasks/src/compound/staged.rs", "jig/crates/jig-domain-tasks/src/positions.rs", "jig/crates/jig-domain-tasks/src/subscriptions.rs", "jig/crates/jig-domain-tasks/src/wake.rs", "jig/crates/jig-domain-tasks/src/placed.rs", "jig/crates/jig-domain-tasks/src/stored/fit.rs", "jig/crates/jig-domain-tasks/src/tests.rs"]
effort = "high"
minutes = 90
+++

# jig: tasks asks `MessageKind` and `Hold` their questions

## Goal

Part of `jig/docs/development/debt/D015-exhaustive-lists.md`. tasks' production names `MessageKind::` 464 times, and `impl MessageKind`'s only method has one caller. The questions below become methods in `boundary.rs`, and their callers match the answer.

| Question | Lists today (lines) | Bypassed today |
|---|---|---|
| is it a result, and which | `boundary.rs` (2), `owned.rs` (4), `stored/fit.rs` (2), `inbox.rs` (115) | `Word::payload_bytes` |
| is it spoken words | `activity.rs`, `inbox.rs` (2), `compound/staged.rs`, `positions.rs` (83) | |
| does it take a reserved slot | `inbox.rs`, `stored/fit.rs` (2), `boundary.rs` (63) | `ordinary` in `fit.rs` |
| is it a merged hint | `subscriptions.rs` (2), `inbox.rs` (51) | `hint_of` in `subscriptions.rs` |
| is it batched | `wake.rs` (17) | `category` in `wake.rs` |
| `Hold`: is it placed | `placed.rs` (2) (32) | `is_placed`, beside `placed_reason` |

```rust
impl MessageKind {
    pub fn family(&self) -> MessageFamily;              // { Spoken(Spoken), Hint(Hint), Slot(Slot), Result, Virtual }
    pub fn result(&self) -> Option<&ResultKind>;
}
impl Hold { pub fn placed(self) -> Option<u64>; }
```

`Spoken`, `Hint` and `Slot` are small enums (`Copy` if `NewsClass` and `NoticeState` are: check; if not, an answer without their payload, and the caller keeps its own match on it).

Acceptance:

- The item's **Measure** on `jig/crates/jig-domain-tasks/src` reads at most 660 (861 at base). `rg -c 'MessageKind::' jig/crates/jig-domain-tasks/src -g '!**/tests/**' -g '!tests.rs' | awk -F: '{s+=$2} END {print s}'` reads at most 250 (464 at base).
- The note's table tests cover all of `MessageKind` for `family`, and all of `Hold` for `placed`. They pass before and after.
- `heavy cargo nextest run -p jig-domain-tasks --lib` and `heavy cargo nextest run -p jig-tasks-world --test suite` pass, with no assertion changed. `heavy cargo check -p jig-domain-core -p jig-protocol-store -p jig-ops-domain --tests` compiles.
- By hand, per the note.
- The branch's net production lines fall by at least 100 (estimate: -200).

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/classifiers.md`: the rule, pin first, the run
- `jig/docs/development/debt/D015-exhaustive-lists.md` (its line number for `impl MessageKind` has moved)
- `skein/docs/foundation/programming-model.md`, 10

## Owns and may change

- **Owns:** the files in `files`. A new enum's export is one line of `lib.rs`. The item stays: `classify-close` deletes it.
- **A second writer in this crate, by design:** `store-remaps` works in `llm.rs`, `limits.rs`, `value.rs`, `stored.rs`, `stored/models.rs`, `questions.rs` and `run.rs`, and you stay out of them. Neither builds on the other.
- **What stays, as routing:** `wake.rs`'s rules and order (its other two lists), `stored/fit.rs`'s bounds, `inbox.rs`'s `merge_key` singling out `QuestionOwed`, `waits.rs`, `hold_origin.rs`, the rest of `boundary.rs`. About 20 lines of `ResultKind` lists stay too, since a borrowing answer would need a lifetime.
- **Public items that stay:** the `Word` methods, and `procedure_input` (`ops` reads it).

## Steps

1. The table tests against today's code. Commit.
2. A question at a time, a commit each: `result`, `family`'s answers, `Hold::placed`.
3. By hand, per the note.
4. The note's run. Commit and push.

## Report

The note's items.
