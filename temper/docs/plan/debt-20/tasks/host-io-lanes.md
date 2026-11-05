+++
id = "host-io-lanes"
part = "host-lanes"
kind = "feature"
needs = ["shell-on-stage"]
files = ["smith/crates/smith-local-service/src/host_io.rs", "smith/crates/smith-local-service/src/host_io/**", "smith/crates/smith-local-service/src/credentials.rs", "smith/crates/smith-local-service/src/credentials/transport.rs", "smith/crates/smith-local-service/src/lib.rs"]
effort = "high"
minutes = 50
+++

# smith: `HostIo` routes by lane

## Goal

Part of `smith/docs/development/debt/D003-host-io-route-table.md`. skein's lane is there: `file_layer::owns(op)` says whose an operation is (`skein/docs/design/io.md`, 5). So `HostIo`'s `Map<Token, Operation>` and `next_operation` go, submissions pass unchanged, and a completion routes by its token. `native_io_valid` goes for `io::valid`, whose capacity checks are what its doubling checked. The services' `io::resume` becomes `stage::up`.

```rust
/// The one close `HostIo` itself submits, on a tag of its own (skein/docs/design/lib.md, 4).
const PROGRAM_ROOT: Token = Token::new(1 << 60);
fn split(&mut self, complete: kernel::Complete);       // signals, in_flight; root | file | io_completions
fn forward(&mut self, submit: kernel::Submit);         // for route_submission: record the signal read, count, push
fn edge_room(&self) -> bool;                           // room, and in_flight < limits.queue
pub(crate) fn io_up(&mut self, env: &Env<io::Limits>); // split all, then io::stage::up
```

| Site | Today | After |
|---|---|---|
| `HostIo::new` | sizes the table, starts the counter | `in_flight: u32`, `io_completions: Queue::uncapped()` |
| `settled` | table's length against `signals.reads()` | `in_flight == reads`, and the queue empty |
| `reap` | removes the route, panics if it's missing | the lane split; `io::up` and `file_layer::up` still panic on an unknown op |
| the flush | holds submits when the table is full | the same check, on `in_flight` |
| `route_submission` | mints a token, rewrites `Cancel`, records the signal read, inserts | `forward` |
| `close.rs` | mints and routes the root's close | `PROGRAM_ROOT` |

Acceptance:

- `rg -n 'Map<Token, Operation>|next_operation|enum Operation' smith/crates/smith-local-service/src/host_io.rs smith/crates/smith-local-service/src/host_io` finds nothing. `rg -n 'native_io_valid' smith/crates` finds nothing.
- `rg -n 'io::resume' smith/crates/smith-local-service/src/host_io.rs smith/crates/smith-local-service/src/credentials/transport.rs` finds nothing, but for the option below.
- `heavy cargo nextest run -p smith-local-service -p smith-local-shell -p smith-local-process-world -p smith-hosts-world` passes, with no assertion changed but the exceptions below.
- By hand: invert `file_layer::owns` in `split` (`simulated_store_failure_reaches_chat_before_a_later_marker` goes red); drop `in_flight` from `edge_room` (`the_kernel_edge_stays_capped_at_its_queue`); don't call `root_closed` for `PROGRAM_ROOT` (a closing test; `a_line_before_interrupt_never_spawns_its_silent_check`, from reading); skip `io_completions` in `work_pending` (add a test for it, none pins it today). Restore each.
- The branch's net production lines fall (estimate: -45). `io_resume` falls by three, or two with the option below.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D003-host-io-route-table.md` (stale: the lane and `valid` exist in skein, nothing collides any more, and the doubling is inside `valid`)
- `skein/docs/design/io.md`, 2 and 5; `skein/docs/design/lib.md`, 4
- `smith/docs/design/protocol/limits.md`, 4; `smith/docs/design/protocol/hosts.md`, 3 and 5.6
- `skein/crates/skein-io/src/stage.rs`

## Owns and may change

- **Owns:** the files in `files`.
- **This task's exceptions to the common note on assertions, and no others:**
  - `host_io::tests::invalid_limits_are_refused` asserts `u32::MAX / 8` sockets is valid. The valid list becomes `[0, (1 << 24) - 1]`, `1 << 24` and `u32::MAX / 8` join the refused list, and it calls `host_io::valid`. That's the **Departs** in `smith/docs/design/protocol/limits.status.md`: today those limits pass and `Io::new` panics at start.
  - Three tests reach into the table and change how they do, keeping what they mean: `the_kernel_edge_stays_capped_at_its_queue` (its file submission's token needs `file_layer::LANE | 13`), `the_process_backlog_counts_only_internal_handoffs`, and `signals.rs`'s `only_the_retained_read_is_left_out_of_settling`.
- **A stale cancel** passes its target's raw token. A missing one gives `TooLate` on the sim, readiness and the ring alike, which is what `u64::MAX` gave.
- **The order of IO before files** is `flush`'s, and stays.
- **`credentials/transport.rs` today runs one ready, one completion and one fired per pass; `stage::up` drains them, so it's this task's riskiest site.** If a credentials test's events change because of it (`pending_account_file_events_precede_another_accounts_private_action`, seed 61, step bound 4000), keep this site's hand drive, say so in your report, and let the floor rise by one. Its hand-set bit 61 stays: `D020`'s.
- **`io::down` loops stay, and `service/tests.rs`'s one `io::resume` stays:** it asserts what one `resume` does.
- **A reader outside `files`:** `service.rs`'s splitting on bits 62 and 63 keeps `HostIo` namespace 0.

## Steps

1. `native_io_valid` goes; `io::valid`; the test's list; `credentials.rs`; `lib.rs`'s re-export.
2. `PROGRAM_ROOT`, `split`, `forward`, `edge_room`, `in_flight`; the table goes; the three tests.
3. `io_up` and `transport.rs` on the stage.
4. Break the rules by hand, as the acceptance says.
5. `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p smith-local-service -p smith-local-shell --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

## Report

The common note's items; whether `transport.rs` moved; what you broke and what failed.
