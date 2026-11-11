# io

Provisional, 2026-10-03, revised 2026-10-19. The design of `skein-io`'s
step layer: the lowest step layer of every service. It owns sockets,
pipes, files and child processes, the operations in flight on them, and
the receive and send queues, and it is the only layer that sees a
descriptor or a kernel error. Below it is the kernel boundary
(../kernel.md); above it, a service's protocol layer.

## 1. In one page

- **Entities, and streams.** io's vocabulary has two parts: entities
  (listen, accept, connect, spawn, a root, close) and bytes, which are the
  stream vocabulary of lib (../lib.md, 7), the same for a socket and a pipe.
- **Every completion goes through io first.** io turns the kernel's
  completions into events about its entities, and requests about its
  entities into kernel operations, and absorbs the mechanics in between:
  buffers in flight, short transfers, cancellation, settling, graceful
  close. The backend absorbs an uncancelled transfer's `EINTR` before any
  outward completion: `PipeWrite`, `Write`, `Append`, `Send`, `PipeRead`,
  `Read`, `Recv` and `ReadSignal` are reissued with identical buffers,
  offsets and token, without a retry bound (../readiness.md, 4). A requested
  cancellation completes `Cancelled`; other errors and short counts keep
  their usual results.
- **Flow control is explicit.** A receive is in flight only while the
  intake has room, a send only for queued output under its cap, and an
  accept only while the listener's owner has room.
- **Paths stay beneath a root,** spawned children are pidfds from the
  start, each in a process group of its own, and termination signals
  arrive as events.
- **Trees, in cgroups.** A child may start in a cgroup io made, as
  another user, and contained: in namespaces of its own, seeing the file
  system through a view the owner composes. A cgroup is stopped as a
  whole, and proved empty by the kernel (`processes.md`, 2 to 4).
- **Files are replaced whole, or appended to.** A record is replaced
  durably, privately when it is a secret (`files.md`, 2, 3); a trace or a log is
  a write stream on a file opened to append, with a write deadline for a
  filesystem that stalls (`files.md`, 1).
- **Addresses, never host names.** io connects to an address: a socket
  address, the host's name resolved before it, or an in-process name,
  which needs no resolving (section 3).
- **Joined sockets in one process.** A connect to an in-process name is
  joined in memory to a socket its listener accepts, with no kernel
  between them, through the same requests and events as TCP (`sockets.md`, 5).

## 2. In skein

`skein-io` depends on lib. A service's protocol layer depends on it, and
so do the backends, for the kernel records. io's limits cap its edges:
the shared entity slots, the operations in flight, each stream's intake
and queued output, children's pipe lists, and the in-process names
(`sockets.md`, 5). The command buffers supplied to `Spawn` belong to the caller.
`skein_io::valid` checks `Limits::is_usable()` and the derived operation,
flight and timer capacities. A zero byte or admission limit that makes
io unusable is refused; overflowing a sum of heap prices alone is no
reason to refuse otherwise usable limits.

```rust
// skein-io, the protocol side, for sockets
pub enum Request {
    Listen  { owner: Token, addr: Addr },
    Connect { owner: Token, addr: Addr },
    Bind    { socket: Token, owner: Token },          // attach to an accepted socket
    Reject  { socket: Token },
    Stream  { stream: Token, down: stream::Down },    // sockets and pipes alike
    Output  { stream: Token, down: stream::OutputDown }, // independent output reservation
    Spawn   { owner: Token, spawn: Spawn },
    Signal  { child: Token, signal: Signal, to: Target }, // the child, or its group (section 6)
    Usage   { owner: Token },                         // the process's resource usage (6.1)
    Cgroup  { owner: Token, op: cgroup::Op },         // cgroups (6.2)
    // File { owner, root, op }; Window { owner, stream } (section 7)
    Close   { entity: Token },                        // graceful (section 3); one Closed follows
    Abort   { entity: Token },
}

pub enum Event {
    Listening  { owner: Token, listener: Token, addr: Addr },
    Accepted   { owner: Token, socket: Token, peer: Addr },  // to the listener's owner
    Connecting { owner: Token, socket: Token },
    Connected  { owner: Token },
    Stream     { owner: Token, up: stream::Up },
    Output     { owner: Token, up: stream::OutputUp },
    Spawned   { owner: Token, child: Token, pipes: Box<[Token]> },
    Exited    { owner: Token, exit: Exit },
    Usage     { owner: Token, usage: Measured },      // for a Usage: its one terminal
    Cgroup    { owner: Token, event: cgroup::Event }, // 6.2
    Shutdown  { signal: ServiceSignal, from: Option<Token> }, // section 7
    // File { owner, result }; Resized, Window { owner, size } (section 7)
    Failed     { owner: Token, error: Error },
    Closed     { owner: Token },                      // terminal
}

pub enum Error { Refused, Unreachable, TimedOut, Reset, Busy, Other }

pub enum Addr { Ip(SocketAddr), InProcess(Name) }     // section 4
```

An accepted socket is announced to the listener's owner, which binds to
it or rejects it (../../../foundation/programming-model.md, 4.2). Files have requests and
events of their own, `skein_io::file`'s (`files.md`).

The loop drives io through four entry points: `resume` takes one entry
of the ready list, `up` one completion, `fire` one expired deadline (a
graceful close's, or a retry's), and `down` one request. The free functions
`skein_io::stage::{up, down}` drive those entry points for the loop:

- **`stage::up(io: &mut Io, env: &Env<Limits>, completions: &mut Queue<Complete>, told: &mut Queue<Event>, submissions: &mut Queue<Submit>, now: Time)`**
  takes ready work through `resume`, then completions through `up` only
  once the ready list is empty, then due deadlines through `fire` only
  once every completion is taken. What the down pass made is told first,
  and progress that came wins over a deadline that passed while the loop
  waited. Each entity is resumed at most once a stage. Before taking an
  entry it checks the kernel's submission queue has room for that entry
  point's most submissions: `MAX_OUT_RESUME.submissions` (1),
  `MAX_OUT_UP.submissions` (2), or `MAX_OUT_FIRE.submissions` (2).
- **`stage::down(io: &mut Io, env: &Env<Limits>, asked: &mut Queue<Request>, submissions: &mut Queue<Submit>)`**
  takes requests in order through `down`, only while `Io::takes` says io
  can hold a refusal and the kernel's submission queue has room for
  `MAX_OUT_DOWN.submissions` (2). Requests it cannot take stay queued.

Both leave work they cannot take for the next iteration, check room
before taking it, and assert that each entry point submits no more than
its `MAX_OUT_*`. Requests to the kernel go out as `Submit` records into
the queue the loop hands the kernel. The kernel's submission and
completion queues stay capped (../../../foundation/programming-model.md,
section 2). Queues between io and the protocol
layer use `Queue::uncapped`: a push cannot fail and the loop reserves no
room there. New external work waits at the root's watermark
(../../../foundation/programming-model.md, 7.1).

**Decisions.**

- **Every event names its owner first:** the token the layer above gave
  for the entity concerned. `Accepted` names the listener's owner, which
  answers it.
- **`Listening` carries the bound address,** so that a listener on port 0
  is usable.
- **`Connecting` hands over the socket at once,** in the up pass after the
  `Connect`, so that its owner can close or abort a connect before it
  connects. A listener's setup waits on nothing outside the host, so its
  owner closes it once it hears `Listening`.
- **io has its own `Error`,** carrying only what the layer above can act
  on: `Refused` (nothing listened there), `Unreachable`, `TimedOut` (the
  kernel gave up on the peer), `Reset`, and `Busy`, a refusal at io's
  entrance: no socket slot, no in-process name free (`sockets.md`, 5), or the kernel
  out of descriptors, local ports or buffers. Every other kernel error is
  `Other`, among them an address in use or not this host's; one the
  contract rules out is an assertion.
  A stream's failure is lib's `Fault`: `Reset` when the peer is gone
  (reset, timed out, unreachable, the pipe broken), `Other` otherwise.
- **io has no connect timeout.** That is policy above
  (../../../foundation/programming-model.md, 4): the connection's deadline lives in the
  protocol layer, which aborts the connect. io keeps only the mechanics'
  deadlines: a graceful close's, and a retry's; and, on files, the
  deadline an owner states for an operation a stalled filesystem may hold
  (5, `files.md`, 1).
- **What found the kernel out of buffers or descriptors is retried after
  `Limits::retry`,** not at once: a receive, a send, a half-close or an
  accept that failed for want of a buffer, an accept that failed with an
  error the kernel gave no name, and an accept out of descriptors, which
  may be the system's (`ENFILE`), given back by another process. Retried
  at once, each would spin the loop for as long as the shortage lasts.
  The retry deadline sits in io's own table beside the close deadlines,
  one of each per entity.
- **Listeners, streams, children and pipes share one entity slab,** whose
  capacity is `Limits::sockets`: one namespace of tokens, so that `Close`
  and `Abort` name any of them. A spawn reserves one slot for its child
  and one per requested pipe before it goes to the kernel.
- **A refusal is held until the next up pass.** A `Listen`, `Connect` or
  `Spawn` refused for want of entity slots has no entity to hold its `Failed`
  and `Closed`, so io holds the owner's token in a list of its own,
  bounded by `Limits::refusals`, and `resume` tells it. The loop hands io
  a request only while io can hold a refusal (`Io::takes`), as it
  reserves room in the submissions.
- **The startup check.** Whoever stacks a machine on a stream checks that
  the machine's largest exact read (a fill or a scan) is no more than
  `Limits::largest_read` (the intake cap), and its largest room no more
  than `Limits::largest_room` (the output cap). A demand past either
  could never be met, and io asserts it, as lib's intake does (../lib.md,
  7). A read of what has arrived is met whatever its maximum.

## 3. Addresses and names

- **io connects to addresses only:** an `Addr` is a socket address, IPv4
  or IPv6 with a port, or an in-process name. The kernel's records carry
  only socket addresses (../kernel.md, 3).
- **To start:** the shell resolves the configured peer host names at
  startup and hands in the socket addresses as configuration (../shell.md,
  6).
- **An in-process name** is at most `Name::MAX` bytes, compared as bytes
  and held inline, so an `Addr` stays a plain value that allocates
  nothing. It lives in io's own namespace: the names its listeners hold,
  which no other process, and no other io, sees. Nothing resolves it, at
  startup or later: configuration gives it as it is. A listener on one
  accepts joined sockets (`sockets.md`, 5); the empty name is a connector's, which
  has none.
- **Later:** when a service must resolve host names while it runs, the
  next protocol machine is a DNS client over UDP sockets. It reads the
  hosts file and the resolver configuration at startup. io gains
  datagram sockets for it.

## 4. Signals to the service

Termination signals are blocked at startup (../shell.md, 6), read from a
signalfd through the ring, and arrive as `Shutdown` events. The domain
decides what shutting down means. A service that asks also reads `SIGHUP`
so, as `ServiceSignal::Hangup` (../shell.md, 6.3): a terminal's closing, which
may arrive more than once, and which the domain tells from a second
termination by its name.

`Shutdown { signal: ServiceSignal, from: Option<Token> }` names the sender
only when it is a live child this io spawned: `from` is `Some(child)` after
that child's successful `Spawn` completion and before its `Exited` event.
The kernel boundary carries `Signalled::Termination { signal, sender:
Option<kernel::Sender> }`; io compares that opaque identity for equality
with the identities returned by `Done::Spawned { pidfd, sender }` (`processes.md`).
`from` is `None` for the kernel, a terminal, this process itself, an
unrelated process, or a child whose exit io has reported, even before its
reap. Linux reads the identity from signalfd's `ssi_pid`. macOS's kqueue
signal source has no sender identity, so `from` is always `None` there.

A service at a terminal also blocks `SIGWINCH` at startup (../shell.md,
6.3). It is read from the same signalfd and arrives as `Resized`, after
which the owner asks for the size: `Window { owner, stream }`, naming the
stream that reads the terminal, answered `Window { owner, size }`, or
`Gone` if the terminal has hung up, with rows
and columns, its one terminal (../kernel.md, 6.3). A `Window` naming a stream
gone or closing reaches no kernel: its owner, who waits on that terminal,
is told `Gone` on the next pass up, where `sockets.md`, section 1 would drop it.

## 5. Testing

io's tier is io worlds (../../../foundation/testing-strategy.md, 2.6): io over the
simulator, with a scripted owner as the step above it. It listens,
connects, streams, opens files and spawns, in an order the test chooses,
with closes and aborts in every state, under tiny limits and every fault
the simulator injects. The worlds check what io adds: buffers in flight,
cancellation and settling, graceful close, the receive and send queues
under their caps, and the accept budget; and every contract and
invariant of ../../../foundation/testing-strategy.md, 6.

A test world's limits start from `Limits::TEST`, one small set, and change the
fields the test means (../../../foundation/testing-strategy.md, 10).

Files and processes go to skein's minimal fake machine: a few files
beneath a root, a program that echoes its input, one that exits with a
given status, one that never exits. Its files lie behind the
simulator's machine seam (../../testing/README.md, 4).

- **Step tests** (`crates/skein-io/src/tests/`): each cell of both
  machines driven by hand, the refusals at a full slab, stale tokens
  dropped going down and asserted going up, every outcome of a cancel in
  every order, each call with the kernel queues' declared room; sends
  within the room granted, and one past it, or with none granted,
  asserted.
- **io worlds** (`tests/io`, `skein-io-world`): `skein_world`'s loop
  over the simulator drives each process's io, on the stage a service
  runs on, and a scripted owner, which runs
  each connection by a plan of sends within the room granted and demands
  of every kind; a referee holds each scenario's expectations; the stage
  holds each call to the kernel queues' `MAX_OUT_*`, and the world checks
  the accept batch each iteration and both halves of the stream contract in a
  ledger (io's, and the owner's as the side above), the invariants once
  settled, and replay. Thirteen scenarios: accept, bind and reject;
  connects made, refused for a slot and by the peer; connects waiting on
  a full backlog, cancelled; descriptors run out, so a socket is refused
  and a listener starves; two listeners under one accept batch; a socket
  discarded for want of a slot; a burst of connects past the slab and
  the refusals; an exchange both ways; backpressure through io; a
  refusal mid-upload that still reaches the peer, which hears the end
  and no reset; abort; the close deadline; closes and aborts at random
  moments. Each admission point's scenario checks its trace for evidence
  the point was reached. A few seeds each, calm and chaotic, in the
  focused suite; 150 of each in the fuzzy one, which asserts that every
  fault fell and that a cancel of each operation io cancels was seen to
  stop it, to come too late, and to go unsubmitted.
- **One exchange over the real ring**, in the focused suite.
- **Memory:** io driven by hand to its limits and back, every call
  checked under its scenario's named ceiling with headroom by the
  counting allocator, at four sets of limits, one of them with receive
  buffers that dwarf the rest; and a listener's life, its sockets
  announced, rejected, bound and discarded.
- **Processes and pipes:** step tests exercise spawn, wait, signal, the
  one-way pipe streams and child closure after its pipes. A separate
  counting-allocator test fills the entity slab with a child and its
  pipes, checks the spawn completion and arms a read on every pipe
  under the scenario's named ceiling with headroom.
- **Files:** step tests drive `FileIo` by hand
  (`crates/skein-io/src/tests/file.rs`, `store.rs`): short transfers,
  bounds refused after the close, a scan's prefix across batches, each
  request's deadline, cancels that win and that come too late, and every
  step of a store, its conflicts, a taken temporary name, and a rename
  that settles at its deadline. The kernel records for files are held to
  the ring by the conformance suite (../kernel.md, 8), and kv's tests run
  the driver over the simulator, on a fake machine that crashes at every
  step of a commit and a snapshot (../../data/kv.md, section 9). A world that cuts
  a power loss lays nothing its requests make: its private root and its
  directories start absent, so a cut after each make and before its
  answer finds the directory there only if the request answered it.
- **Process groups:** on the real kernel, a child that starts a
  descendant holding its pipe and exits, then a signal to its group,
  which ends the descendant and the pipe; a close of a child whose group
  still runs; a signal to a group after its leader exited, before its
  close. In the simulator, groups the minimal machine's programs make,
  and the same cases.
- **Cgroups and trees:** step tests of each cgroup request and its
  refusals, of a close that kills, waits, and removes deepest first, and
  of a contained spawn's every stage failed in turn. io worlds over the
  simulator: trees whose first process exits before the rest, whose
  processes ignore the polite signal or the termination, that fork
  during a terminate, that escape their group but not their cgroup, and
  whose end the kernel reports late. On the real kernel, the conformance
  suite's (../kernel.md, 8).
- **A tree's metadata and its filesystem:** step tests of each request,
  its bounds refused before any record, each record's errors, a deadline
  that passes while a synchronous record runs (the request still answers
  its own), and `SubvolumeChildren`'s pages; the records themselves are
  the conformance suite's, btrfs's on the ring alone (../kernel.md, 8). A
  world over the simulator, on a fake machine that cuts a power loss,
  shows a link's name kept only once its directory was synced, and a
  mode, an owner and an attribute only once the filesystem was.
- **Append streams:** io worlds over the simulator with sends within the
  room granted, short writes, a close that flushes, and the simulator's
  hung `Write`, met by the write deadline while open and by the close
  deadline while closing; the conformance suite, on the ring and the
  simulator, for a write at the end of a file opened to append.
- **Durable replace and private files:** every step of the replace
  failed in turn, the temporary removed each time, and a conflict; a
  private directory and file refused for each bit, link and kind; and a
  cut at each `Sync` and `Rename` (../../testing/simulator.md, 3.3), after which the
  target is old or new and whole.
- **Resource usage:** in the simulator, a usage asked before and after a
  child's close, its part joining the children's only at the close; on
  the real kernel, the conformance suite's (../kernel.md, 8).
- **Joined sockets** (`sockets.md`, 5): step tests of each cell the table changes,
  and a check at every call that a joined end submits nothing. io worlds
  with scripted owners at both ends of each pair, in one process: accept,
  bind and reject, the connector then reset; connects refused by an
  unbound name and by a closing listener; connectors waiting on a
  listener with no room or no slot, one aborted as it waits; two
  listeners under one accept batch; a name taken, then taken again after
  its listener's `Closed`; every name held; an exchange both ways under
  intakes small enough to cut every move; backpressure through io; the
  refusal mid-upload that still reaches the peer, which hears the end and
  no reset; abort, heard as a reset; the close deadline against a peer
  that never ends; closes and aborts at random moments. The ledger holds
  both halves of the stream contract as for sockets. A few seeds each in
  the focused suite, the random one in the fuzzy suite.
- **The same owners over either:** a calm world whose owners run the
  same plans over a loopback pair and over a joined pair, and read the
  same bytes, ends and failures over each.
- **Memory:** every name held and every slot a joined end, under the
  scenario's named ceiling with headroom.
- **One process, on the real ring:** a listener on loopback and one on an
  in-process name in one `drive`, an exchange over each.

## 6. Remaining work

What of this document is not built is in [`README.status.md`](README.status.md).

## 7. Open questions

- **A stream's fault for a stall.** A write deadline fails an append
  stream with lib's `Fault::Other`, as a full disk does. Whether `Fault`
  should name a stall, so its owner can tell a filesystem that stopped
  from one that refused.
- **Operations the kernel cannot interrupt.** A cancelled write on a
  filesystem in uninterruptible sleep keeps its stream settling, and its
  process from ending (../../../foundation/notes.md). So does a cgroup whose process sleeps
  so, which its close waits for (`processes.md`, 2).
- **Filters on a contained tree's calls.** A contained tree has no
  seccomp filter, and keeps the capabilities its ids have in its
  namespace. Whether io offers a filter, when a service needs one.
- **A terminal for a tree.** A contained tree's command has pipes, never
  a terminal; whether a tree may be given one, when a service needs it.
- **Faults between joined ends.** The simulator's faults never reach a
  joined pair (`sockets.md`, 5), so an application's one-process worlds meet only
  what the owners at either end do. Whether io gives a world a way to
  reset a pair, or cut a move short, as a socket world can.
- **`FileIo`'s buffers in a `List`:** a scan's sorted page wants `insert` and
  `pop`, which `List` lacks. Whether `List` gains them, or a scan keeps its
  page another way.
