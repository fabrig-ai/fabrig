+++
id = "box-first-block"
part = "box-terminal"
kind = "feature"
needs = ["box-own-log"]
files = ["temper/crates/temper-box/src/terminal.rs", "temper/crates/temper-box/src/host.rs", "temper/crates/temper-box/src/sink.rs", "temper/crates/temper-box/src/tests/**", "temper/crates/temper-box/tests/real_loop.rs"]
effort = "high"
minutes = 90
+++

# The box prints its block before any step, and gives serve the secret

## Goal

Given a `FirstLink`, the box writes the sign-in block once its logs have moved and before any step starts, then starts the steps, and gives the secret to serve's first spawn alone, on its standard input. A fresh link is a blank line and the link.

Acceptance, each in this crate's tests, its clock the test's:

- **The block's exact bytes:** stderr `PRELUDE`, stdout the URL and a line feed, stderr `FOOTER`, in that order, each written before the next, and then `after_block`. Nothing else on either stream.
- **Before any step:** no spawn is asked until the block is written, and the last run's logs have moved before its first byte. A held write holds the first spawn.
- **Anything the start says about logs or records** while they move (`box: cannot write ...`) waits behind the block.
- **The secret:** serve's first `Spawn` has a pipe at child 0, whose bytes are the secret's alone, then its close. No spawn's `args` or `env` holds the secret or the URL. Serve's second `Spawn` has no pipe at child 0. A first spawn that fails to start spends it. A step other than serve never has one.
- **A fresh link:** stderr `FRESH`, stdout the link, neither timed. Serve's later links are all fresh.
- **With no `FirstLink`:** serve's first link is written as the same block, at that link, and `after_block` at the start. A link step's handoff has no block, as today.
- **A termination signal** while the block is unwritten starts no step, and the box ends as its stop does.
- **`Ctrl-C stops temper.`** is in `box.log` after `Logs:`, and on no terminal stream.
- **`Debug`** of `Settings` and `FirstLink` shows neither the URL nor the secret.
- **`real_loop`:** a real child reading its stdin gets the secret's bytes and an end of file, over a real pipe.
- All of `heavy cargo nextest run -p temper-box` passes.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1, *The first link's secret*; 7.2, *The sign-in block* and the FIFO's paragraph.
- `temper/docs/plan/box-link-first/notes/contracts.md`, section *temper-box*.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns `temper/crates/temper-box/src/terminal.rs`, `host.rs`, `sink.rs`, `temper-box/src/tests/**` and `temper-box/tests/real_loop.rs`.

Reader and neighbour files, each only for `first_link: None, after_block: Box::new([])` in a `Settings` literal:

- `temper/crates/temper/src/box_command.rs`, in `run_configuration`.
- `temper/crates/temper/tests/suite/box_kinds.rs`, about line 161.

Left for later tasks, to report under `## Left`: reds in package `temper`'s tests.

Options, each pre-approved:

- A new phase between `Phase::New` and `Phase::Running`, or a flag, whichever reads best. `work_pending` holds a copy of `New`'s condition: both must agree.
- `Formatter::block` may lose its `wall` parameter.
- `entries` becomes at least 4. `bounds.output` covers `after_block`'s length. `FIXED_BYTES` covers `PRELUDE` and `FOOTER`: its test says so.
- If `Terminal` has no way to say it has drained, add one in `sink.rs`.
- `spawn_pipes`' `List::with_capacity(n + 4)` and `io_limits`' entity count ("up to four entities per step") each grow by the one pipe. Skein's `Spawn` needs no change: it makes a pipe at child 0 on Linux and on macOS, as a link step's stdin does.

## Steps

The outer shape in `host.rs`:

```rust
fn spawn_pipes(step: &Step, serve: &[u8], listeners: &[(SocketAddr, Token)], secret: bool) -> Box<[kernel::Pipe]>;
// with secret, last of all: kernel::Pipe { child: 0, way: kernel::Way::In, parent: None }
// in `spawned`, serve's: Sink::new(*pipes.get(4).expect("serve stdin"), 1), pushed the secret's bytes
```

`Way::Given` makes no token, so serve's tokens are stdout, stderr, requests, answers, then stdin. `state.stdin` and its close already work for a link step: the code that reads it (about lines 420, 836, 955, 1490) is not kind-gated, so reuse it.

The readers to move, from the survey, under `temper/crates/temper-box/`: `src/tests/terminal.rs`, about lines 13-15 and 23-26; `src/tests/host.rs`, about lines 818-826, 924-948; `src/tests/sink.rs`, about lines 80-101; `src/tests/logs.rs`, about line 69; `tests/real_loop.rs`, about lines 33-35, 492-514.

Write first, in `src/tests/host.rs`: `the_block_is_the_terminal_before_any_spawn`, comparing both streams' bytes; `a_held_block_holds_the_first_spawn`; `the_secret_is_on_serves_first_stdin_alone` (reading `Ask::Spawn`'s `args`, `env` and `pipes` as `fixtures.rs` records them, about line 289, and `harness.read_stdin`); `a_restarted_serve_has_no_stdin_and_its_link_is_fresh`; `a_termination_before_the_block_is_written_starts_no_step`.

Narrow runs: `heavy cargo nextest run -p temper-box --lib -E 'test(block) or test(secret)'`. Then fmt, clippy and the focused tests of `temper-box` as your text says, with `heavy cargo check -p temper --tests`. Commit and push.

## Report

As your text says, naming how the first spawn is held.
