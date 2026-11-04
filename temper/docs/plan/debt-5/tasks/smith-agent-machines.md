+++
id = "smith-agent-machines"
part = "smith-lifecycles"
kind = "feature"
needs = ["smith-agent-test-helpers"]
files = [
  "smith/crates/smith-agent-service/src/service/**",
  "smith/crates/smith-agent-service/src/service.rs",
  "smith/crates/smith-agent-service/src/terminal.rs",
]
effort = "medium"
minutes = 50
+++

# smith review, findings 1, 3, 4, 9 and 10: `smith-agent-service`'s machines

## Goal

`Teardown` has the design's `Live` phase, `Roots` returns from its matches, `Last` is the one line a run keeps, and the service's transitions follow finding 9's rule. `Roots::reported` says it mutates. No machine's behaviour changes. No debt item closes.

Acceptance:

- `rg -n 'LiveUnsent' smith --type rust` reads nothing; `Teardown::Live` holds `failure: Option<Failure>`.
- `rg -n 'enum Line\b' smith/crates/smith-agent-service/src` reads nothing.
- `rg -n 'fn reported|\.reported\(\)' smith/crates/smith-agent-service/src/service` reads nothing about `Roots`. The free functions, and the channel's `Ending::reported`, are different items and stay.
- `rg -n 'let (next|reported);' smith/crates/smith-agent-service/src/service/roots.rs` reads nothing.
- `heavy cargo nextest run -p smith-agent-service`, `heavy cargo nextest run -p smith-agent-shell --lib` and `heavy cargo nextest run -p smith-agent-world --test suite` pass, each count above zero and as at base.
- `heavy cargo check --tests -p smith-agent-process-world` passes; `heavy cargo clippy -p smith-agent-service --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/smith-review.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md`, findings 1, 3, 4, 9 and 10.
- `smith/docs/design/protocol/agent.md`, 6; `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: `smith-agent-service/src/service/**`, `src/service.rs`, `src/terminal.rs`.
- `Teardown` and `Last` are `pub`, and `smith-agent-shell/src/agent_shell.rs` reads `Last`. `Last` gains `Clone`, an addition; that reader does not change. `Teardown`'s field is private, and nothing outside the crate reads it.
- `Last::Quiet`'s doc widens to cover a run with no answer yet.
- `Last::Unanswered` is never stored in `line`: it is a kept value, and no machine, so that stays.
- `failure_record` now runs at `remember` rather than at `last`, with the same `now` and the same result. This is not a change of behaviour.

## Steps

1. Finding 1, `service/teardown.rs`:

   ```rust
   /// (doc of the design's phase, agent.md, 6)
   Live { failure: Option<Failure> },
   ```

   The 10 `Live | LiveUnsent` arms become `Live { .. }` (`teardown.rs` near 94, 151, 205, 225; `stages.rs:147`; `program.rs` near 676, 713, 714; `routing.rs:15`; `up.rs:390`). `failure()` becomes `Self::Live { failure } => failure`; `failed` becomes one arm with `first.or(Some(failure))`, as `End::record` does; `answered` matches the field directly. The constructor at `stages.rs:172` and `tests/lifecycles.rs:43` and `:67` follow. The `Option` lives in the state, so 5.4's rule on `Option`s beside the state does not apply.

2. Findings 3 and 10, `service/roots.rs`:

   ```rust
   pub(super) fn take_report(&mut self) -> bool {
       let (state, reported) = match mem::replace(self, Self::Empty) {
           Self::Closing { roots } => (Self::Held { roots }, true),
           state @ (Self::Empty | Self::Held { .. }) => (state, false),
       };
       *self = state;
       reported
   }
   ```

   `close_next` takes the same form, `(Self::Closing { roots }, Some(root))` and so on, with `last` plus `truncate` kept (`List` has no `pop`). `adopt` matches to get `roots`, pushes once, and sets `*self = Self::Held { roots }`. The rename reaches `up.rs:91`, the module doc at `roots.rs:3` and the tests in `roots.rs`.

3. Finding 4, `src/terminal.rs`: `Line` goes; `Last` derives `Clone`; `remember` builds a `Last`; `last` ends in `Some(service.stages.line.clone())`. `service.rs:213` and `stages.rs:210` follow.

4. Finding 9, by the rule in `temper/docs/plan/debt-5/notes/smith-review.md`:

   | Site | Becomes |
   |---|---|
   | `Admission::admit`, `cancel`, `follow` (`service/admission.rs`) | `match *self` on the `Copy` state |
   | `Opening::attached` (`service/opening.rs`) | a same-variant change, matching `self` in place |

   `Opening::attach`, `contract` and `take` hold data that is not `Copy`, and keep `mem::replace`. `teardown::{answered, failed, shutdown, follow}` take the state by value and conform.

5. `cargo fmt`, the acceptance's commands. One commit per finding; finding 9's message states the rule and why a `Copy` state needs no placeholder, and push.

## Report

The lines added and removed, per finding; any finding you found wrong or left, and why.
