+++
id = "tasks-families"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/tests.rs", "jig/crates/jig-domain-tasks/src/wake.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-tasks: the message-family table loses its copied columns, and pins all eight wake categories

## Goal

Findings 1 (first bullet) and 2 (second bullet) of `temper/docs/plan/debt-22/inputs/debt20-review.md`. `news_batch` separates only category 6 of eight, so merging `Notice` into `Words` in `wake.rs`'s `category` stays green: it must go red. `spoken`, `separate`, `hint` and `finish` follow from `family`, so those columns go. Net test lines fall by about 140.

Acceptance:

- `message_families_keep_their_inbox_and_activity_behaviour`'s rows are `(kind, family, category)`, one line each, every `MessageKind` row kept. No `#[expect(clippy::too_many_lines)]` on it unless the body still needs one.
- The loop reads `spoken`, `hint`, `separate` and `finish` from `family` in one short local `match`, and asserts what it asserts today: the activity line, `word.finish_requested()`, the subscription credit, `inbox::room`, and the batch's eligibility.
- The row's `category` is asserted against `crate::wake::category(&kind)`, and the eligibility assertion reads `category == 6`. `category` becomes `pub(crate)`; nothing else in `wake.rs` changes.
- Each of these goes red, once, and is restored: merging `Hint::Notice` into category 0 in `category`; moving `Slot::Met` out of the separate arm in `inbox.rs`; dropping the `Spoken` arm's `Line::Words` in `activity.rs`; making `finish_requested` answer for `Slot::Amendment`.
- `heavy cargo nextest run -p jig-domain-tasks` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, findings 1, 2
- `jig/crates/jig-domain-tasks/src/wake.rs`, `category`; `jig/crates/jig-domain-tasks/src/boundary.rs`, `MessageFamily`

## Owns and may change

- **Owns:** `jig-domain-tasks/src/tests.rs`, but for the line `mod revisions;`, which `tasks-pins` changes; `wake.rs`'s `category`, its visibility only.
- **Stays:** `result_shapes_keep_payload_and_owned_byte_accounting` and `placed_holds_keep_their_identity`: no finding names them.
- `separate-slots` follows you and reshapes `Slot`: keep the local `match` on `family` easy to change, one arm per family.

## Steps

1. The rows, then the loop.
2. Break each rule by hand as Acceptance lists; restore.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; each rule broken and what failed.
