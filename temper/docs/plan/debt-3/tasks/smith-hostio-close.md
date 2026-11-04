+++
id = "smith-hostio-close"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-local-service/src/host_io.rs",
  "smith/crates/smith-local-service/src/host_io/**",
]
effort = "high"
minutes = 130
+++

# smith D002, `HostIo`: its close, terminal input, signals and agent child as machines

## Goal

`HostIo` (`smith-local-service/src/host_io.rs`), the local host's io hub, holds four
lifecycles in flags and `Option`s beside each other: the hub's close, the terminal input
stream, the signal stream, and the agent child with the host's one read on it. Each becomes
its own state enum with its own transitions, in a module beside the owner, and no field beside
them is meaningful in some states only (`skein/docs/foundation/programming-model.md`, 5.4).
Every effect and its order stay. This is the first of two tasks on `HostIo`'s share of
`smith/docs/development/debt/D002-lifecycles-in-flags.md`; `smith-hostio-inspection` follows
with file inspection. The item stays open: leave its file.

Acceptance:

- `HostIo` holds 30 fields, from 34, and 3 `bool` or `Option` fields, from 10: `programs`,
  `shell_output` and `failed`, which stay for the reasons under "What stays".
- `rg -n 'self\.(process|reading|closing|close_root_pending|terminal_reading|signal_operation)\b([^(]|$)|adapter\.(process|reading|terminal_input)\b|struct Closing|closing_roots' smith/crates/smith-local-service/src/host_io.rs smith/crates/smith-local-service/src/host_io/`
  reads nothing. (`self.process_events` is not matched, nor a machine's own method such as
  `Close::closing`; `Checks::closing_root` is another name and stays.)
- The new modules `host_io/close.rs`, `host_io/terminal.rs`, `host_io/signals.rs` and
  `host_io/child.rs` each hold what the sketches below name, with exhaustive matches: no
  `_` arm, `matches!`, tuple scrutinee, or `if let` or `let else` on an own enum.
- No `pub` or `pub(crate)` signature of `HostIo` changes. `git diff --stat origin/main --
  smith/crates` lists only `smith-local-service/src/host_io.rs` and `src/host_io/*`.
- `heavy cargo nextest run -p smith-local-service --lib` passes today's count plus the 11 of
  step 6: 113, if your first run reads the 102 that `rg -c '#\[test\]'` over the crate's `src`
  sums to. No assertion of an existing test changes; the fixture lines step 1 names are the
  only edits to them.
- `smith-local-shell`'s lib, `smith-hosts-world` and `smith-local-process-world` pass as in
  step 7, with every count above zero.
- `smith/scripts/old-forms.txt` does not change.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md`, whole.
- `skein/docs/foundation/programming-model.md`, sections 1, 5.2 and 5.4.
- `smith/docs/design/protocol/hosts.md`, 2 ("Reads are the domain's", "The end") and 3;
  `smith/docs/design/shell.md`, 9 ("The end"). They say what the hub does and need no change:
  no design or status file moves, so no test that reads `smith/docs/design/**` runs for this.
- The form: `smith-protocol-program/src/backend/process.rs` (a small machine, its handlers
  and `follow`), `smith-local-domain/src/delivery/machine.rs`.
- The readers: `smith-local-service/src/service.rs` (`iterate`, `route_local`'s `Exit`) and
  `service/ending.rs` use the hub only through `pub(crate)` methods, which keep their shapes.

### What stays, and why

| Field | Reading |
|---|---|
| `programs: Option<Box<Checks>>` | Handed once before the first pass and never taken: `None` says the chat has no program account. Configuration, meaningful in every state. |
| `shell_output: Option<Token>` | The route key of the shell's adopted writer; the shell owns its close and nothing else reads it. The `Option` is the slot's whole state. |
| `failed: bool` | A one-way latch, set on a refused channel or a refused send, read by `Service::failed()`. The child goes on after it, in any state, so it belongs to no lifecycle here. |
| `send`, `next_operation`, `file_roots`, `limits`, `launch`, `endpoints`, the queues | Counters, configuration and handoffs, meaningful in every state. |
| `operations`, `native_io_valid`, every `io::resume` | `smith/docs/development/debt/D003-host-io-route-table.md`, another item, waiting on skein. |

### What moves

| Fields today | Becomes | Lifecycle |
|---|---|---|
| `closing: Option<Closing>`, `close_root_pending: bool` | `close: Close` | the hub's close |
| `terminal_input: Option<Token>`, `terminal_reading` | `terminal_input: TerminalInput` | the input stream |
| `window_read: WindowRead` (already an enum, its transitions inline) | `window_read: WindowRead` | one terminal-size read |
| `signals: Option<Token>`, `signal_operation: Option<Token>` | `signals: Signals` | the signal stream and its retained read |
| `process: Option<protocol::Process>`, `reading: bool` | `child: Child` | the agent child and the host's read |

## Owns and may change

- `smith-local-service/src/host_io.rs`: the struct, `new`, the methods that read these fields,
  the `Closing` struct and `WindowRead` (moved out), the test fixtures of step 1.
- New: `host_io/close.rs`, `host_io/terminal.rs`, `host_io/signals.rs`, `host_io/child.rs`.
  Each opens with a module doc that says what state is kept, what is never known, and its
  entry points, citing the design at module level (`smith/AGENTS.md`).
- `host_io/store.rs`: the one line `self.closing.is_some()` in `store_pass`.
- `host_io/delivery.rs`: the root terminal's check in `markers_up`, and nothing else
  (`smith-hostio-inspection` owns the rest of the file).

Not this task's, whatever it sees: the route table (`operations`, `Operation`, the routing in
`reap` and `route_submission`), `DeliveryIo` and `StoreIo`, `Checks`, `smith-host-protocol`'s
`Process`, and the name `adapter` in existing tests. New tests bind `hub`. No debt, design or
status file changes.

Options, each pre-approved:

- `clippy::large_enum_variant` on `Child`: the sketch boxes `Process`, as `programs` boxes
  `Checks`. If a counted-memory test objects, keep it unboxed under a scoped `#[expect]`.
- A method of a sketch cannot be `const`: drop the `const`.
- A sketched `unreachable!` fires in a test or a world. For `Child` on a `Spawn` over a held
  read: carry `read` into the new child, which is today's effect. For `TerminalInput::adopted`
  or `Signals::adopted` when not `Absent`: assign, which is today's effect. Say so in the
  report, with the test. For `Signals::reading` in `Absent`, stop and ask.
- A helper named here reads better split or merged: names and types of the sketches bind;
  how a handler is cut inside does not.

## Steps

Keep the handlers small. Where the state holds nothing that must move, a transition may match
`*self` and assign; where it does, use 5.4's `mem::replace` idiom. Effects stay at the stages
where they happen today: the order of `io_requests` is what the tests and worlds read.

### 1. The tests stop writing private state

First commit, on today's fields, so the tests prove the conversion. Add four private methods
to `HostIo`, have production call the first three, and move the fixtures onto them:

```rust
fn spawned(&mut self, process: protocol::Process);          // `request`'s Spawn; `self.process = Some(process)`
fn adopted_input(&mut self, stream: Token, read_input: bool); // `adopt_terminal`'s input half, its Fill(1) when asked
fn hold_read(&mut self);                                    // `request`'s Read: the assert, then `self.reading = true`
fn read_held(&self) -> bool;                                // `process_work_pending`, `drain_process`
```

- `internal_io_burst` (`host_io.rs` ~1316, ~1364): `adapter.adopted_input(terminal, false)` and
  `adapter.spawned(process)`.
- `prefetched_adapter` (~1493): after building the hub, `adapter.spawned(...)` with a
  `protocol::Process::new(owner, &adapter.limits.channel, adapter.limits.detail_bytes,
  adapter.endpoints.clone()).unwrap()`, never launched: a held read needs a child once the
  read lives in `Child`.
- `a_write_failure_does_not_spend_the_hosts_pending_read` (~1536, ~1551, ~1556):
  `adapter.hold_read()` for each write, `adapter.read_held()` for the read. Its comment ends
  "a real Read requires an opened Process".

Run `heavy cargo nextest run -p smith-local-service --lib 'host_io::'`, then the whole lib;
commit. These four methods stay to the end, forwarding to the machines, so the tests do not
move again.

### 2. `host_io/close.rs`: the hub's close

```rust
const ROOT_OWNER: Token = Token::new(u64::MAX); // the two literals of the root close

/// How far the adopted roots have closed: one `Close` in flight, last root first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Roots {
    /// `left` roots wait for the file layer, and none is in flight.
    Next { left: u32 },
    /// One root's close awaits its terminal; `left` follow it.
    Waiting { left: u32 },
}

/// The hub's own close (programming-model.md, section 5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Close {
    /// Nothing closes; a termination signal interrupts the chat.
    Open,
    /// From the local domain's last word, permanently. `shutdown` holds a signal for the service.
    Closing { roots: Roots, shutdown: bool },
}

impl Close {
    pub(super) fn begin(&mut self, roots: u32) -> bool;       // false: already closing
    pub(super) const fn closing(&self) -> bool;
    pub(super) const fn roots_left(&self) -> u32;             // `left` of either `Roots`; 0 when Open
    pub(super) const fn root_due(&self) -> bool;              // Closing, Next, left > 0
    pub(super) fn next_root(&mut self) -> u32;                // Next { left } to Waiting { left - 1 }: that position
    pub(super) fn root_answered(&mut self, owner: Token) -> bool; // Waiting and ROOT_OWNER: to Next, true
    pub(super) fn shutdown(&mut self) -> bool;                // Closing: latch, true. Open: false
    pub(super) fn take_shutdown(&mut self) -> bool;
}

impl HostIo {
    /// What closing implies for one pass. `down` runs it once, where its two blocks sit today.
    pub(super) fn follow_close(&mut self, now: Time);
}
```

| State | `close()` | `follow_close` | root terminal | `Event::Shutdown` | `take_shutdown` |
|---|---|---|---|---|---|
| `Open` | to `Closing { Next { left: roots }, false }`; programs cancel, then the input closes | nothing | not this machine's: `root_answered` is false and the event goes on as today | `terminal.interrupt` | false |
| `Closing`, `Next { left > 0 }` | ignored | when `files.takes()`: `file::Request::Close` for `file_roots[left - 1]`, to `Waiting { left - 1 }`; then the program root | as `Open` | latch | the latch, cleared |
| `Closing`, `Next { 0 }` | ignored | the program root | as `Open` | latch | the latch, cleared |
| `Closing`, `Waiting` | ignored | the program root | to `Next { left }`, event consumed | latch | the latch, cleared |

"The program root" is today's block (`host_io.rs` 718-729), unchanged, under its same room
conditions; it uses `operations` and `Operation::ProgramRoot`, which stay.

Readers: `close()` calls `if self.close.begin(roots)`, then `cancel_programs` and the input's
close, in that order. `settled` reads `self.close.closing() && ... && self.close.roots_left()
== 0`: `Waiting { 0 }` counts as none left, as `closing_roots()` does today, and the file
conjuncts cover the terminal in flight. `work_pending`: `self.close.closing()` for `checks`,
and `self.close.root_due() && self.files.takes()` for the last line. `store_pass` and
`take_shutdown` read the machine. `markers_up`: `if self.close.root_answered(event.owner()) {
continue; }` in place of today's check, at the same point. `Event::Shutdown` in `up`:
`if !self.close.shutdown() { terminal.interrupt(terminal_events); }`.

Test (step 6's), run the lib, commit.

### 3. `host_io/terminal.rs`: the input stream and the window read

```rust
/// The terminal's input stream (protocol/hosts.md, section 5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TerminalInput {
    /// None adopted, or its `Closed` came up.
    Absent,
    /// Open and the person may type: each read asks for the next.
    Reading { stream: Token },
    /// Open and held by the local domain: no demand.
    Held { stream: Token },
    /// The hub's close went down; bytes still reach the terminal, and nothing asks for more.
    Closing { stream: Token },
}

impl TerminalInput {
    pub(super) fn adopted(&mut self, stream: Token, read_input: bool, below: &mut Queue<io::Request>);
    pub(super) fn reading(&mut self, reading: local::Reading, below: &mut Queue<io::Request>);
    pub(super) fn bytes(&self, below: &mut Queue<io::Request>);
    pub(super) fn close(&mut self, below: &mut Queue<io::Request>);
    pub(super) fn abort(&self, below: &mut Queue<io::Request>);
    pub(super) fn closed(&mut self);
    pub(super) const fn owns(&self, owner: Token) -> bool;    // any state with this stream
    pub(super) const fn open(&self) -> Option<Token>;         // Reading, Held
    /// What entering a state asks of the stream.
    fn follow(&self, below: &mut Queue<io::Request>);
}
```

`follow` is exhaustive: `Reading` pushes `Demand { read: Fill(1), room: 0 }`, `Held` pushes
`Demand { read: Nothing, room: 0 }`, `Closing` pushes `io::Request::Close`, `Absent` nothing.
It runs after each transition, and only those: a cell marked "ignored" below owes no request,
which is why `Bytes` while `Held` sends nothing.

| Event | `Absent` | `Reading` | `Held` | `Closing` |
|---|---|---|---|---|
| `adopted` | to `Reading`; `follow` only if `read_input` | `unreachable!`, adopted once | the same | the same |
| `reading(Open)` | ignored | ignored | to `Reading`, `follow` | ignored |
| `reading(Held)` | ignored | to `Held`, `follow` | ignored | ignored |
| `Up::Bytes` (terminal fed first, as today) | not owned | `follow` | ignored | ignored |
| `Up::End`, `Up::Failed` | not owned | `terminal.closed`, as today; no transition | the same | the same |
| hub's close | ignored | to `Closing`, `follow` | to `Closing`, `follow` | ignored |
| `abort` | ignored | `io::Request::Abort` | the same | the same |
| `Closed` | not owned: the event goes on to the `other` arm | to `Absent` | to `Absent` | to `Absent` |

`reading` while `Absent` is dropped where today it is remembered: `Request::Input` only comes
out of a local step, and `local_host.rs` adopts before its first pass. The headless front
adopts with `read_input` false into `Reading`, as today's `Open` with no demand.

```rust
/// One terminal-size read at a time. Its operation has no entity below, so its answer
/// can follow the stream's `Closed`: this is a lifecycle of its own, never `TerminalInput`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowRead { Idle, Requested, Pending, Again }

impl WindowRead {
    pub(super) fn asked(&mut self);             // Idle, Requested: Requested. Pending, Again: Again
    pub(super) fn answered(&mut self);          // Again: Requested. Idle, Requested, Pending: Idle
    pub(super) const fn requested(&self) -> bool;
    pub(super) fn follow(&mut self, input: Option<Token>, below: &mut Queue<io::Request>);
}
```

`follow` is `down`'s first block: `Requested` with `Some(stream)` goes to `Pending` and pushes
`io::Request::Window { owner: stream, stream }`; with `None` to `Idle`; the others stay.
`read_window` asks only while `self.terminal_input.open()` is `Some`. The `Event::Window` arm
calls `answered` whatever the owner, then pushes the event to the shell, as today.

`host_io.rs`'s readers: `adopted_input` forwards to `adopted`. `input` becomes
`self.terminal_input.reading(reading, ...)`. The terminal arms of `up` guard on `owns`, the
`Bytes` arm feeds, then calls `bytes`. `abort` forwards. `close()` calls `close`.

Test, run the lib, commit.

### 4. `host_io/signals.rs`: the signal stream

```rust
/// The adopted signal stream and io's retained read (programming-model.md, section 5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Signals {
    /// None adopted, or its `Closed` came up.
    Absent,
    /// Adopted, with no read in the kernel.
    Open { stream: Token },
    /// io's retained read is in the kernel under `read`; closing waits for no such operation.
    Reading { stream: Token, read: Token },
}

impl Signals {
    pub(super) fn adopted(&mut self, stream: Token);    // Absent to Open; else unreachable!
    pub(super) fn reading(&mut self, read: Token);      // Open to Reading; Reading: today's assert; Absent: unreachable!
    pub(super) fn completed(&mut self, operation: Token); // Reading with this `read` to Open; else ignored
    pub(super) fn closed(&mut self);                    // to Absent
    pub(super) fn close(&self, below: &mut Queue<io::Request>); // Open, Reading: Close; Absent: ignored
    pub(super) const fn owns(&self, owner: Token) -> bool;
    pub(super) const fn streams(&self) -> u32;          // `signal_streams`: 0 or 1
    pub(super) const fn reads(&self) -> u32;            // `settled`: 0 or 1
}
```

No state implies an effect, so this machine has no `follow`: the one request is the `close`
cell's, and `settled` reads the rest. It needs no `Closing` variant: `advance_end` closes
signals once, and no cell differs on it. A read cannot outlast its `Closed`: io closes the
descriptor after the read lands, and `reap` sees that first (`skein-io/src/signals.rs`).

`route_submission` calls `reading(global)` where it asserts and sets today; `reap` calls
`completed(complete.op)` in place of its first `if`; `settled` compares `operations.len()`
with `self.signals.reads()`. The two lines touch the route table's code and leave it as it is.

Test, run the lib, commit.

### 5. `host_io/child.rs`: the agent child and the host's read

```rust
/// The host's one read of the child's channel (protocol/hosts.md, section 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Read { Idle, Held }

/// The agent child (protocol/hosts.md, section 3).
#[derive(Debug)]
pub(super) enum Child {
    /// No agent process: runs are inline, or none has spawned yet.
    Absent,
    /// The latest spawn, from its Spawn through its reap: `Process` holds how far it has gone.
    Spawned { process: Box<protocol::Process>, read: Read },
}

impl Child {
    pub(super) fn spawned(&mut self, process: protocol::Process);
    pub(super) fn hold(&mut self);
    pub(super) const fn read_held(&self) -> bool;
    pub(super) fn answered(&mut self);
}
```

| Event | `Absent` | `Spawned`, `Idle` | `Spawned`, `Held` |
|---|---|---|---|
| Spawn, channel refused | `failed`, `Unspawned`, no transition, as today | the same | the same |
| Spawn | to `Spawned`, `Idle` | to `Spawned`, `Idle`; the reaped child drops, as today | `unreachable!`: the host retires an agent only after its channel's end answered a read (`smith-host-domain/src/domain.rs`, `agent.reading.ended()`) |
| Send, Signal | `unreachable!("Spawn precedes ...")`, today's `expect` | as today | as today |
| Read | `unreachable!("Spawn precedes Read")` | to `Held`, then `process.read` | today's assert, "one host read at a time" |
| a read terminal in `drain_process` | retained | retained | to `Idle`, goes up |
| an `other` io event in `up` | to `shell_events` | `process.from_io` | `process.from_io` |
| `fire`, `next_channel_down`, `next_deadline` | nothing | the process's | the process's |

`drain_process` is this machine's `follow`, and keeps its name, which the tests call: `Held`
lets one read terminal up, and `Idle` retains it, in the one place `up` runs. Non-terminal
events pass in every state, as today. There is no `Gone` variant: today's `Some(process)`
stays after a reap, and an `other` event keeps going to `from_io`, not to the shell.

`spawned`, `hold_read` and `read_held` on `HostIo` forward. `request`'s arms, `up`,
`next_deadline` and `down`'s channel loop `match &mut self.child` where they held `if let
Some(process)` or `.expect(...)`.

Test, run the lib, commit.

### 6. The cells' tests

Each module's `#[cfg(test)] mod tests`, pure and small, in ordinary Rust (`hub` where a whole
hub is needed, a `Queue::uncapped()` below). They name the cells; the whole-hub tests stay
as they are.

- `close.rs`: `a_second_close_changes_nothing`; `roots_close_one_at_a_time_last_first`;
  `a_foreign_file_terminal_answers_no_root`; `a_shutdown_while_closing_waits_for_one_take`.
- `terminal.rs`: `a_stream_demands_as_its_reading_changes`;
  `bytes_ask_for_more_only_while_reading`; `a_closing_stream_takes_no_reading_and_still_aborts`;
  `a_requested_window_settles_unread_once_the_input_closes`.
- `signals.rs`: `only_the_retained_read_is_left_out_of_settling`;
  `a_closed_stream_counts_no_more`.
- `child.rs`: `a_new_spawn_holds_no_read`.

Each lands with its machine's commit. The lib reads 113 at the end.

### 7. The whole

```
heavy cargo nextest run -p smith-local-service --lib
heavy cargo nextest run -p smith-local-shell --lib
heavy cargo nextest run -p smith-hosts-world --test suite -E 'not test(/^fuzzy::/)'
heavy cargo nextest run -p smith-local-process-world --test suite -E 'not test(/^fuzzy::/)'
cargo fmt
python3 -I -B scripts/old_forms.py          # from smith/
heavy cargo clippy -p smith-local-service --all-targets -- -D warnings
```

The local-process world runs both agent placements, so it reads the terminal, the signals and
the spawned child's reads and reap. Then `common.md`'s citations check; commit and push.

## Report

Add to the implementer's text: fields before and after, by name; the lib's test count; each
cell a test or a world reached that the table says `unreachable!`, and the option taken; any
effect whose order moved, with the test that shows it (none is expected); anything met in
`operations` or `route_submission` that a later item needs to know.
