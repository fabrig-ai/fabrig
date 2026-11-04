+++
id = "smith-domain-machines"
part = "smith-lifecycles"
kind = "feature"
needs = ["smith-domain-test-helpers"]
files = [
  "smith/crates/smith-local-domain/src/chat.rs",
  "smith/crates/smith-local-domain/src/chat/machine.rs",
  "smith/crates/smith-local-domain/src/delivery/machine.rs",
  "smith/crates/smith-local-domain/src/reconcile.rs",
]
effort = "medium"
minutes = 30
+++

# smith review, findings 7, 8 and 9: `smith-local-domain`'s machines

## Goal

Taking the idle history is written once, one function builds the answer record, and the chat machines' transitions follow finding 9's rule. No machine's behaviour changes. No debt item closes.

Acceptance:

- `Chat::take_idle` serves both sites of `chat/machine.rs`; `rg -n 'fn reconcile_terminal_with_receipts' smith/crates/smith-local-domain` reads nothing.
- One `save_answer` builds the answer record, for `delivery/machine.rs` and `reconcile.rs`.
- `rg -n 'mem::replace' smith/crates/smith-local-domain/src/chat.rs` no longer finds the `Copy` transitions named in step 3.
- `heavy cargo nextest run -p smith-local-domain --lib` passes; `heavy cargo nextest run -p smith-local-world --test suite` passes; each count is above zero and the same as at base.
- `heavy cargo clippy -p smith-local-domain --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/smith-review.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md`, findings 7, 8 and 9.
- `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: the four files in the header.
- `Metadata` gains `Copy`. No other derive changes.
- `Chat::request_load`, the chat phases, `Delivering` and `reconcile` hold data that is not `Copy`: they keep `mem::replace`.

## Steps

1. Finding 7: `pub(crate) fn take_idle(&mut self) -> Option<Transcript>` in `chat.rs`, which leaves `State::Done`, called from `chat/machine.rs`, near lines 49 and 113. The two `unreachable!` texts become one.

2. Finding 8: in `delivery/machine.rs`, `pub(super) fn save_answer(name, delivery, landed, after: AfterSave, out)`. `reconcile_terminal_with_receipts` and its wrapper `reconcile_terminal` give way to it, with `after` carrying `Return { owner }` or `ResumeLoading`. Its five callers change (near `machine.rs` 171, 180, 247, 602 and 651), and those of `reconcile.rs`. `delivery/machine.rs` and `reconcile.rs` are sibling modules under `domain`; put `save_answer` where both reach it.

3. Finding 9, the rule in `temper/docs/plan/debt-5/notes/smith-review.md`:

   | Site | Becomes |
   |---|---|
   | `Chat::close_input`, `Chat::drain_until` (`chat.rs`) | `match *self` on the `Copy` state |
   | `HeadSurvey`'s transition (`chat/machine.rs`, near 359) | `match *self` |
   | `metadata_started`, `metadata_saved` | `Metadata` derives `Copy`, then `match *self` |

4. `cargo fmt`, the acceptance's commands. Commit per finding; finding 9's message states the rule and why a `Copy` state needs no placeholder, and push.

## Report

The lines added and removed, per finding; any transition you left and why.
