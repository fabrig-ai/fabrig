+++
id = "smith-host-io"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-local-service/src/host_io.rs",
  "smith/crates/smith-local-service/src/host_io/**",
]
effort = "medium"
minutes = 40
+++

# smith review, findings 5, 6, 9 and 10: `smith-local-service`'s `HostIo`

## Goal

`HostIo` loses four one-line forwards, `plain_action` is one match, `Close`'s mutating methods say so, and the stale panic text goes. No machine's behaviour changes. No debt item closes. This crate shares no file with finding 2's helpers, so it does not wait for them.

Acceptance:

- `rg -n 'fn (spawned|adopted_input|hold_read|read_held)\b' smith/crates/smith-local-service/src/host_io.rs` reads nothing.
- `rg -n 'marker inspection' smith/crates/smith-local-service/src` reads nothing.
- `rg -n 'root_answered|fn shutdown' smith/crates/smith-local-service/src/host_io/close.rs` reads nothing.
- `heavy cargo nextest run -p smith-local-service --lib` and `heavy cargo nextest run -p smith-local-process-world --test suite` pass, each count above zero and as at base.
- `heavy cargo clippy -p smith-local-service --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/smith-review.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md`, findings 5, 6, 9 and 10.
- `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: `smith-local-service/src/host_io.rs` and `src/host_io/**`.
- Panic texts are not read by any test here, and no `should_panic` test exists in the crate.

## Steps

1. Finding 5, `host_io.rs`, each forward deleted and its callers calling the field:

   | Forward | Production | Tests | Becomes |
   |---|---|---|---|
   | `spawned` | 725 | 1301, 1433, `host_io/child.rs` 79, 89 | `x.child.spawned(..)` |
   | `adopted_input` | 200 | 1253 | `x.terminal_input.adopted(.., &mut x.io_requests)` |
   | `hold_read` | 794 | 1482, 1497, `child.rs:84` | `x.child.hold()` |
   | `read_held` | 565, 592 | 1502, `child.rs` 77, 83, 85, 87, 93 | `x.child.read_held()` |

   Add `Child::process_mut(&mut self) -> &mut protocol::Process` for the three panicking matches at `host_io.rs` 730, 787 and 795; their three `unreachable!` texts become its one. Lines 329, 518, 536 and 624 have real `Absent` arms and stay. `Child::spawned`'s two identical arms collapse:

   ```rust
   assert!(!self.read_held(), "...before Spawn");
   *self = Self::Spawned { process: Box::new(process), read: Read::Idle };
   ```

2. Finding 6, `host_io/delivery.rs`, `plain_action`:

   ```rust
   let finished = match action {
       File { .. } => { /* push */ return; }
       Done => true,
       Failed => false,
   };
   ```

   then one `match` on the inspection, with `if finished` in the two arms that differ. Order is unchanged, and `mem::replace` stays after the `File` return.

3. Findings 9 and 10:

   - `Close::shutdown` becomes `latch_shutdown`: `host_io.rs:492`, tests `close.rs` 150, 189, 192, 193.
   - `Close::root_answered` becomes `settle_root(owner)`: `delivery.rs:300`, tests `close.rs` 167, 168, 177, 181, 183.
   - `delivery.rs:313`'s panic: `"every other file terminal belongs to the one inspection"`. `Capturing` and `Comparing` also arrive there, so "marker inspection" is stale.
   - `Signals::adopted` and `WindowRead::follow` are `match self` plus `*self =` on a `Copy` state: they become `match *self`, by the rule in `temper/docs/plan/debt-5/notes/smith-review.md`. `Signals::reading`, `completed`, `TerminalInput::*`, `Close::begin` and `WindowRead::asked`, `answered` conform.

4. `cargo fmt`, the acceptance's commands. One commit per finding; finding 9's message states the rule, and push.

## Report

The lines added and removed, per finding; any finding you found wrong or left, and why.
