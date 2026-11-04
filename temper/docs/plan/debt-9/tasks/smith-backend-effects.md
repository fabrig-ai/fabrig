+++
id = "smith-backend-effects"
part = "smith-backend-effects"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-program/src/backend.rs", "smith/crates/smith-protocol-program/src/backend/**", "smith/crates/smith-protocol-program/src/mcp.rs", "smith/crates/smith-protocol-program/src/mcp/tests.rs", "smith/crates/smith-protocol-program/src/command.rs", "smith/docs/development/debt/D006-backend-two-effect-paths.md", "smith/docs/development/debt/README.md"]
effort = "high"
minutes = 100
+++

# smith: pay D006, the program backend's effects leave by one path

## Goal

Pay `smith/docs/development/debt/D006-backend-two-effect-paths.md`. `Backend` holds no queue of its own
requests: every request goes straight into the caller's `below`, and `flush` goes. `mcp::Server` takes its
output queue as an argument, and `Server::up` goes. No behaviour changes.

Acceptance:

- `rg -n 'self\.io\b|fn flush\b|&mut self\.io' smith/crates/smith-protocol-program/src` finds nothing.
- `rg -n 'above: Queue|pub fn up\b' smith/crates/smith-protocol-program/src/mcp.rs` finds nothing.
- `for _ in 0_u32..2` is gone from `backend/start.rs`; `server_events`' bound reads the queue's capacity.
- The worlds below pass with no test's events moved.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/design/protocol/programs.md`, 3 and 5; `skein/docs/foundation/programming-model.md`, 2 (the step's
  shape, and **Bounded**: no `while` in step code).
- `smith/docs/development/debt/D006-backend-two-effect-paths.md`.

## Owns and may change

- Owns: all of `smith-protocol-program/src/backend.rs`, `backend/**`, `mcp.rs`, `mcp/tests.rs`, `command.rs`.
- `relay.rs` and `relay/` are the shell's and keep `Queue<io::Request>`: no change.
- `smith-agent-service/src/service/program.rs` reads `backend::Below` as it is: no change. `Below`'s shape and
  `Backend`'s public step signatures stay.
- `mcp.rs` may name `crate::backend::Below`: a module cycle inside the crate is fine here.
- If `command.rs` has a reader outside the crate (at this plan's base it has none: check with `rg`), keep its
  signature, and in `startup.rs` drain a local `Queue::uncapped()` into `below`.
- A test that calls `backend.flush(&mut below)` drops the call, and reads the queue it passed to the step; one
  that `mcp/tests.rs`' harness pops from `self.server.up()` pops the harness's own `above`. Those are moved
  items. The harness's `drain` unwraps `Below::Io`.

## Steps

1. **One way down.** The helpers that take `&mut Queue<io::Request>` (`startup`, `stopping`, `interrupt`,
   `process`, `transport`, `command`, `mcp::Server`) take `below: &mut Queue<Below>` and push
   `Below::Io(request)`. Backend's helpers that reach them and lack `below` gain it (`terminate`, `permission`,
   `match_calls`, `read_stdout` if it pushes, and so on), in the step's argument order. The `io` field goes.
2. **`flush` goes.** Each call becomes `self.load_native(below)`, at the same place: a native snapshot's load
   stays last among a step's file requests.
3. **The server's face.** The methods that push an `Up` take it in the step's order:

   ```rust
   pub fn from_io(&mut self, now: Time, event: io::Event, above: &mut Queue<Up>, below: &mut Queue<Below>);
   pub fn fire(&mut self, now: Time, above: &mut Queue<Up>, below: &mut Queue<Below>);
   pub fn close(&mut self, above: &mut Queue<Up>, below: &mut Queue<Below>);
   ```

   and likewise `answer` and `verdict` if `settle` is reached from them. `Backend` owns the hop:

   ```rust
   /// What the MCP server has said and `server_events` has not read: at most `SERVER_UPS`, one event's worth.
   server_up: Queue<mcp::Up>,   // Queue::with_capacity(SERVER_UPS), with `const SERVER_UPS: u32 = 2;`
   ```

   The capacity stays 2: this is the queue the server held, in its owner's hands, and a third `Up` still
   panics at `push`. `Startup`'s calls take `above` from `Backend`.
4. **`server_events`:** the same arms, popping `self.server_up`, under
   `for _ in 0..self.server_up.capacity()`. It keeps reading what an arm's handling pushes in the same pass, as
   the poll did: not `0..len()`, and not `while let` (no `while` in step code).
5. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-protocol-program
   -p smith-agent-service --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-protocol-program
   -p smith-agent-service`; the worlds that run a program: `smith/tests/agent-process` and the program stories
   in `jig/tests/channel` (package names from their `Cargo.toml`).
6. Delete the item and its row; the citations check. Commit and push.

## What can move, and what stops you

This is a change of order inside one step, so it's the task's one hazard. At base a step's `Below::File`s all
precede its `Below::Io`s, because the io requests leave at `flush`. After it they interleave in the order the
step made them.

- **Io among Io is unchanged,** and so is File among File. `service/program.rs`' `down` splits `below` into
  `file_submissions` and `io_requests` at once, so the order between the two kinds reaches neither driver.
  What's left of it: an Io now ahead of a stalled File no longer waits behind it.
- **A path that left requests in `self.io` past its return** now sends them a step sooner: at base
  `yield_for_wind_down` returns before `flush` when `phase()` is `Ending`.

Both are what the item asks for. If either moves a test's events or outcome, stop and ask; don't reorder pushes
to match. Name in your report each path you found of the second kind.

## Report

The common note's items. The tests only move, so no break is owed, but do say which test files changed and how.
