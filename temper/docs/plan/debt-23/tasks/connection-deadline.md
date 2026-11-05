+++
id = "connection-deadline"
part = "spot-review"
kind = "feature"
needs = ["connect-close"]
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/*.rs", "skein/crates/skein-forge-connection/src/tests/*.rs"]
effort = "medium"
minutes = 30
+++

# skein-forge-connection: a connection's deadline is derived from its state

## Goal

Finding 3 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, decided by fabrig/debt: derive the deadline from the state. `Connection::deadline` mirrors the `until` every `State` holds, and `follow` copies it after each transition. It becomes a function, and the copy goes. About -17 lines; no behaviour moves. Finding 7's one line in this crate comes along.

Acceptance:

- `follow`'s match is `pub(crate) fn deadline(&self) -> Option<Time>`. The field `Connection::deadline` and every `follow` call are gone. `Connection::fmt` prints `self.deadline()`.
- `hold` is folded into `rest`: its callers write `c.rest(Some(..))`.
- `rg -n 'fn follow|\.follow\(\)|fn hold|\.hold\(|deadline: None|pub\(crate\) deadline' skein/crates/skein-forge-connection/src/connection.rs` reads nothing, and `rg -n 'c\.hold\(' skein/crates/skein-forge-connection/src/component.rs skein/crates/skein-forge-connection/src/component` reads nothing. `gh.state.puller.hold(hint)` is another type's, and `hooks/connection.rs`'s `Hooked` keeps its own `follow` and `deadline`: both stay.
- `component/github.rs`'s `github_retire_unsent` (or whichever function now holds it) reads `Phase::Sending { written } => !written`: `phase()` builds `written` from `self.written`, so `!written && !c.written` tests one value twice.
- `heavy cargo nextest run -p skein-forge-connection --lib`, `-p skein-forge-machine-world`, `-p skein-forge-protocol-world`, `-p skein-forge-client-world` and `-p skein-forge-setup-world` pass. No test changes but `.deadline` becoming `.deadline()`.
- Heap pins in this crate's tests shrink or stay, since `Connection` loses a field. One that shrinks is updated and named in your report.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, findings 3 and 7 (`component/github.rs`)
- `skein/docs/design/forge-connection.md`, 8.1, "Deadlines are the holder's"

## Owns and may change

- **Owns:** `connection.rs`, and `.deadline` reads, `c.hold` calls and the one line above in `component.rs` and `component/*.rs`. `.deadline` reads in `src/tests/connection.rs` and `src/tests/dialects.rs`. `Waiting::deadline`, `Hooked::deadline`, `limits.deadline` and `gh.state.deadline()` are other things and stay.
- **The trap:** today's `self.deadline` is still the old value after `self.state.take()`, which leaves `Closed { until: None }`. So read the deadline before the `take` everywhere one precedes it: `connect`, `step`'s `LinkMoved::Sent` arm, `upload`'s `Down::Finish` arm. And `self.state.take().idle(self.deadline())` would evaluate its receiver first and read `None`: have `State::idle` read its own `until`, with the match on a small `State::until(&self)` that `Connection::deadline` also calls, rather than pass it in.
- **`Hooked` is `hook-slack`'s;** the rest of this file's state data is `connection-states`'s, which follows you.

## Steps

1. The function and the field; let the compiler find the readers; read each transition for the trap.
2. Fold `hold`; the `github.rs` line.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items. No new rule, so no test to bite; say which pins moved, if any.
