+++
id = "token-lanes"
part = "token-lanes"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-service/src/service.rs", "smith/crates/smith-local-service/src/credentials.rs", "smith/crates/smith-local-service/src/credentials/transport.rs", "smith/crates/smith-local-service/src/credentials/lock.rs", "smith/crates/smith-local-service/src/credentials/lock/events.rs", "smith/crates/smith-agent-service/src/service.rs", "smith/crates/smith-agent-service/src/service/**", "smith/docs/design/protocol/agent.md", "skein/docs/design/lib.md", "smith/docs/development/debt/D020-hand-set-token-tags.md"]
effort = "high"
minutes = 90
+++

# smith's token tags use `Token::tagged`; the agent routes completions by lane

## Goal

Pay smith's D020, in the item's order: bit 61 goes first, then `Token::tagged` replaces each hand-set namespace bit, then the agent service's `operations` table goes, its completions routed by `file_layer::owns` as `HostIo` does.

Acceptance:

- `rg -n 'ACCOUNT_OPERATION_BIT|EFFECT_OPERATION_BIT|SOCKET_OPERATION|COMPONENT_OWNER_BIT|ROOT_FILE|LOCK_FILE|Operation::|next_operation' smith/crates -g '*.rs'` finds nothing.
- `Stages` has no `operations` field, and `enum Operation` is gone.
- The file layer's seed is unchanged: a counter that starts at 1 and rises by one per routed submission, read by `seed_randomness` in `service/down.rs` and `service/program.rs` as before.
- A kernel's in-flight bound is what `operations`' capacity gave, `limits.routes`, and teardown's "settled" means what it did.
- `smith/docs/design/protocol/agent.md`'s Later bullet "A completion's route" is gone, and `skein/docs/design/lib.md`, 14, no longer says smith's tags wait on the item.
- `smith/docs/development/debt/D020-hand-set-token-tags.md` is deleted.
- No world's outcome moves.

## Inputs

- `smith/docs/development/debt/D020-hand-set-token-tags.md`, whole.
- `skein/docs/design/lib.md`, 4; `skein/docs/design/io.md`, 5.
- `smith/docs/design/protocol/agent.md`, 4 to 6.
- `smith/crates/smith-local-service/src/host_io.rs`, `split` and `edge_room`, and `host_io/signals.rs`: how the local host does this.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header; `lib.md` and `agent.md` in the lines named above alone. **Departure:** `lib.md` is another tree's, and this is a citation your deletion would break.

**The tags keep the values their bits gave,** so that no token's value moves but the socket lane's: `Token::tagged(4, v)` is `v | 1 << 62`, and `Token::tagged(8, v)` is `v | 1 << 63`.

| Crate | Const today | Becomes |
|---|---|---|
| `smith-local-service/src/service.rs` | `ACCOUNT_OPERATION_BIT`, `EFFECT_OPERATION_BIT` | `ACCOUNT_TAG: u8 = 4`, `EFFECT_TAG: u8 = 8` |
| `smith-local-service/src/credentials.rs` | `ROOT_FILE` | `ROOT_TAG: u8 = 8` |
| `smith-local-service/src/credentials/lock.rs` | `LOCK_FILE` | `LOCK_TAG: u8 = 4` |
| `smith-agent-service/src/service.rs` | `COMPONENT_OWNER_BIT` | `COMPONENT_TAG: u8 = 8` |

The item didn't name `ROOT_FILE` and `LOCK_FILE`, but they're the same hand-set rule on the accounts' file owners, and its **Paid when** says "smith's token tags".

**Exempted, not changed:** `host_io.rs`'s `PROGRAM_ROOT` (`1 << 60`) names one operation, and nothing nests under it. But it's why `HostIo`'s lane carries no tag: `Token::tagged(0, 1 << 60)` is `None`, so that lane's submissions pass through as they are. Also test fakes' own constants: `smith/tests/fake-claude/src/fake.rs`'s `FILE_BIT`, `smith/tests/local-process/src/terminal.rs`'s `ROOT_CLOSE`, `git_child.rs`'s test `IO_OWNER`. Name them in your report.

Options, each pre-approved:

- Inside `service.rs`'s `flush` and `namespace_submit`, the shape is yours: an `Option<u8>` or two functions. The `HostIo` lane keeps an assertion that its tokens carry neither tag. A `Cancel` keeps its `target.raw() != u64::MAX` guard; say in your report whether anything still mints that target.
- **`kernel_cancel_targets_the_right_layer_operation`** (`service/tests.rs`) pins the table: the same literal token 11 on both lanes, told apart by `route_submission`'s `file` flag. The rule holds by lane now. Rewrite it with file tokens that carry `file_layer::LANE`, keeping what it checks: a `Cancel` names its own layer's operation, and a completion reaches that layer. Report it as the one assertion that changed, and why.
- `Input::OperationSubmitted`, `OperationCompleted` and `assert!(service.stages.operations.is_empty())` become whatever reaches the new counters, keeping what each checks.
- A memory or size pin that shrinks with `operations`' `Map` is updated, and reported.
- If `limits.routes` is set past 2^28 by any test, stop and ask, since the guard below would refuse it.

## Steps

**1. Bit 61** (`credentials/transport.rs`, `credentials.rs`). An account's sockets and files are one `Io` and one `FileIo`, and `file_layer::owns` tells them apart (`skein/docs/design/io.md`, 5). `Accounts::complete`: if `file_layer::owns(complete.op)`, `file_layer::up`; otherwise push to `socket_completions`. `socket_completion` and `socket_submit` go; `socket_down` pushes each submission as it is. Update the module's first comment. Narrow test: `heavy cargo nextest run -p smith-local-service`.

**2. Tags.** `Token::tagged(TAG, value).expect("...")`, `op.tag() == TAG`, `op.untagged()` in place of each mask. `route_completions` reads `tag()`: `ACCOUNT_TAG`, `EFFECT_TAG`, and anything else is `HostIo`'s, as `PROGRAM_ROOT`'s tag 1 is. `component_owner`'s value is a `route.token()`, a slot in the top 32 bits, so `Token::tagged` refuses a slot from 2^28 up, where the old assertion only refused 2^31 up. So `limits.routes` above `1 << 28` is refused with `ConfigError::Queue`, beside `Service::new`'s `lower_routes` check, or in `Stages::parts` if `Effects::new` doesn't pass that check: both constructors refuse it.

**3. Lanes** (`smith-agent-service/src/service/`).

```rust
// Stages: in place of `operations` and `next_operation`
in_flight: u32,    // kernel operations submitted and not completed
signal_reads: u32, // of those, `ReadSignal`s
submitted: u64,    // kernel submissions so far, from 1; seeds the file layer

// routing.rs
pub(super) fn route_submission(service: &mut Stages, submit: kernel::Submit);
// up.rs, io_up, as `HostIo::split` does
if file_layer::owns(complete.op) { file_layer::up(..) } else { service.io_completions.push(complete) }
// readiness.rs
service.submissions.room() > 0 && service.in_flight < service.limits.routes
// teardown.rs, operations_settled
service.in_flight == if service.signals.is_some() { service.signal_reads } else { 0 }
```

`route_submission` loses its `file` flag and its search for a `Cancel`'s target: tokens now reach the kernel as `Io` and `FileIo` minted them, and they differ by `file_layer::LANE`. A `Cancel` of a token no longer in flight gets `TooLate` from the kernel, as `u64::MAX` did (`skein-io/src/kernel.rs`, "Cancelling"). `signal_reads` rises on `kernel::Op::ReadSignal` and falls on a completion whose `kind` is one, a `match` under the same `clippy::wildcard_enum_match_arm` expect. `seed_randomness(service.submitted)` replaces `next_operation` in `down.rs` and `program.rs`.

One small assertion that `submitted` rises by one per `route_submission`, in the existing test. Break the increment by hand, and see it go red.

**Tests.** This changes what a shared token holds, so its readers run: `cargo tree -i smith-agent-service --workspace` and `-i smith-local-service` build nothing and name them, and each runs with `heavy cargo nextest run -p PKG`: `smith-local-shell`, `smith-agent-shell`, and the worlds in `smith/tests/` among them. `heavy cargo check --tests` on any temper or jig package they name.

`cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`, `--write` if a count falls; the same for `skein/scripts/` only if a count there moved; `heavy cargo clippy -p smith-local-service -p smith-agent-service --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. `smith/docs/design/**` is read by tests: run those whose source reads `protocol/agent.md`, if any. Commit each of the three steps, push.

A world whose outcome moves is a stop: read its log, and ask.

## Report

As your text says: the search's reading, what you broke and what failed, the cancel test's change, any pin updated, what stayed exempt, and the net lines.
