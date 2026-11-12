# Sockets and pipes

- **The entities** are listeners, sockets, children and their chosen pipes.
  Sockets and pipes are both streams; above io, they differ only in how
  they were made.
- **One receive is in flight** per stream while its intake has room,
  whether or not anything has been demanded yet. What arrives goes into
  the stream's `Intake`, which meets the demand from above.
- **One send is in flight** per stream, in order. The rest of the output
  waits in a queue under the output cap. A short send continues from an
  offset into the same `Box`; the remainder is never copied.
- **Accepts are re-armed one at a time** while the listener's owner has
  room, up to a configured number per iteration. A flood of connections
  then waits in the kernel's backlog.
- **Close is graceful:**
  1. flush the queued output;
  2. half-close;
  3. read and discard until the peer ends or the close deadline passes;
  4. close.

  This matters when a response goes out while the peer is still sending,
  such as a refusal in the middle of an upload: the response reaches the
  peer instead of being lost to a reset. Abort closes at once.
- **Cancellation is io's to settle.** An operation cancelled by a close, an
  abort or a deadline stays *settling* until its completion arrives, and
  the entity above sees one event (../../../foundation/programming-model.md, 5.3).

## 1. Entities and owners

| Entity | Made by | Owned by | Named above by | Ends with |
|---|---|---|---|---|
| listener | `Listen` | the owner of the `Listen` | `Listening.listener` | `Closed` |
| listener, held | `adopt_listener` (2) | io, until a `Listen` for its address claims it; then that `Listen`'s owner | the token `adopt_listener` returns, which `Listening.listener` names once claimed | `Closed` |
| socket, connecting | `Connect` | the owner of the `Connect` | `Connecting.socket` | `Closed` |
| socket, accepted | a listener's accept | io until answered; then the owner of the `Bind` | `Accepted.socket` | `Closed` once bound; nothing once rejected |
| operation | io, for an entity | that entity, until its completion | its `Submit` token, below | its completion |

- **A listener and a socket are one kind of entity** in one slab,
  `Limits::sockets` of them, each running the machine of its kind. The
  operation table holds the operations in flight, each for its entity. An
  entity is retired only once nothing of its is in flight, its cancels
  included, so a completion always finds it.
- **One `Closed` per entity with an owner,** the last event naming that
  owner. A `Listen` or `Connect` that fails is told `Failed`, and io
  closes what it made by itself, since its owner may hold no token for
  it. A listener whose accept stops is told `Failed` too, but stays: its
  owner, which holds its token, closes it. A rejected socket has no
  owner, and is closed without one.
- **`Failed` and `End` are told once each,** and never after `Closed`.
  After its owner's `Close` or `Abort`, an entity tells only `Closed`.
- **Stale handles.** A request naming an entity that is gone, or closing,
  is dropped. One naming an entity of the wrong kind, or an announced
  socket before its answer, is the layer above's bug, asserted. A
  completion always finds its operation and its entity, which is
  asserted.

- **Owners on both sides.** `Event::owner` and `Request::owner` name the owner a record is for, and `with_owner` re-addresses it, so a composing root routes by owner without matching every kind.

## 2. The listener

| State | Holds | In flight |
|---|---|---|
| Socket | the address | `Socket` |
| Binding | fd | `Bind` |
| Arming | fd, the bound address | `Listen` |
| Listening | fd, the accept, discards | `Accept` when armed; the closes of discarded sockets |
| Settling | fd, the accept if in flight, cancels, discards | those |
| Releasing | | `Close` |
| Closed | | |

The accept, in `Listening`, is `Armed` (an `Accept` in flight),
`Answering` (a socket announced and not yet bound or rejected: the owner
has no room until it answers, and holds its answer while it has none),
`Idle` (on the ready list for the next iteration's accept batch, among
the starved for a socket slot or a descriptor, or waiting for its retry
deadline), or `Stopped` (an error io cannot retry, told as `Failed`). Arming it takes a free socket slot, no
discard in flight, and room in the accept batch, which counts the accepts
armed in an iteration over every listener. The starved are woken when io
retires an entity or closes a discarded socket.

| State | Event | Next, and what it does |
|---|---|---|
| Socket | `Socket` ok / error | Binding, `Bind` / Closed: `Failed`, `Closed` |
| Binding | `Bind` ok / error | Arming, `Listen` / Releasing: `Failed`, `Close` |
| Arming | `Listen` ok / error | Listening: `Listening`, arm / Releasing: `Failed`, `Close` |
| Listening | `Accept` ok | Answering: `Accepted`; with no slot left, the socket discarded and Idle, starved |
| Listening | `Accept`: no descriptor | Idle, starved, and its retry deadline armed |
| Listening | `Accept`: no buffer, or an error with no name | Idle, its retry deadline armed |
| Listening | `Accept`: the connection's own error | Idle, retried in the next iteration |
| Listening | the retry deadline | arm |
| Listening | `Accept`: any other error | Stopped: `Failed` |
| Listening | a discard done | the starved woken |
| Listening | its socket answered; resumed or woken while Idle | arm |
| Listening | `Close`, `Abort` | Settling: `Cancel` of the armed accept; Settling for the discards; else Releasing: `Close` |
| Settling | `Accept` done | an accepted socket discarded |
| Settling | `Cancel` done | retried, unsubmitted, while the accept waits |
| Settling | nothing left in flight | Releasing: `Close` |
| Releasing | `Close` done | Closed: `Closed` |
| Settling, Releasing, Closed | `Close`, `Abort` | ignored |
| any but Listening and Idle | the retry deadline | impossible: cancelled once the accept leaves Idle |
| Socket, Binding, Arming | `Close`, `Abort` | impossible: the owner has no token yet |
| any | `Bind`, `Reject`, `Stream` | impossible: the layer above's bug |

A socket accepted after the owner closed the listener is discarded by
io, unannounced: the owner asked for no more.

**Accept errors.** Out of descriptors, the listener starves until io
gives one back. Out of buffers, or with the network error of the
connection it took (a reset, a timeout, no route, or an error the kernel
gave no name: accept(2) has these retried), it tries again in the next
iteration. Any other error says the socket no longer listens, and stops
it.

**A held listener.** A listening socket opened before the loop, by the
service's startup or inherited from its parent (../shell.md, 6.6), is
adopted with the address it is bound to,
`Io::adopt_listener(fd: Fd, addr: SocketAddr) -> Result<Token, Fd>`, and
held: io arms no accept on it, and the kernel queues the connections it
takes in its backlog until someone accepts.

| State | Event | Next, and what it does |
|---|---|---|
| Held | a `Listen` for its address | Listening, the `Listen`'s owner its owner: `Listening`, arm |
| Held | `Close`, `Abort` on its token | Releasing: `Close` |
| Held | given to a child (`processes.md`, 3) | Held: io keeps its descriptor |

- **Claimed by its address.** A `Listen { owner, addr }` whose address
  equals a held listener's, its IP and its port, takes that listener in
  place of making a socket: no `Socket`, `Bind` or `Listen` is
  submitted, and it is the owner's from then on as if it had bound,
  `Listening` naming the held listener's token. A `Listen` for any other
  address, port 0 among them, makes its socket as before. So the
  components of a service never know whether their listener was
  inherited: the address their configuration gives is the one they
  listen on, and the one `Listening` tells them.
- **Held for giving.** A held listener's token may be given to a child,
  which inherits the same socket at the descriptor its `Way::Given`
  entry names (`processes.md`, 3).
  io keeps its own descriptor, so that a child started again can be
  given it again, and nothing else can bind the address meanwhile. A
  parent that gives never claims: it accepts nothing.
- **Refused,** the caller keeping the descriptor (`Err(fd)`) as every
  adoption does: on a full slab, which counts it among
  `Limits::sockets`, and when a listener at the same address is already
  held. io checks nothing of the socket itself: startup has (../shell.md,
  6.6).
- **Its events.** Unclaimed, the held listener's token is its own
  owner, as an adopted pipe's is: its `Closed` names it. Claimed, its
  events are the claimant's. One never claimed is closed at io's close
  as any entity is.
- **Not taken: a short reservation.** The smaller design frees a
  reserved port just before the step that binds it starts, and needs
  nothing of io. It leaves a window, from the free to the bind, in which
  another process takes the port, and the step then fails at its bind:
  the address it reports is no longer certainly its own.

## 3. The stream

Sockets and write pipes also expose ../lib.md, section 7.1's independent
output reservations through `Request::Output` and `Event::Output`. They
keep one pending or granted output token beside the classic demand; a
read-only demand remains live while that output progresses. The native
reservation checks both byte capacity and a Send record slot, including
the write in flight. A matching Send spends the full grant and enters the
same bounded output queue and kernel write path as classic Send. Read
pipes cannot admit output reservations.

Classic room ownership and independent output ownership are mutually
exclusive. Stale Cancel/Send/Release tokens do not affect a current
reservation. Room requests require an open, connected or bound stream;
they are not admitted after failure or closing. Cancelling an admitted
pending reservation stages its exact terminal for the next up pass.
Close/Abort likewise stages Cancelled before Closed; a genuine stream
failure emits the pending reservation's Failed terminal before classic
Failed. A previously emitted Granted is never answered again. Close and
Abort's promise of only Closed applies to the classic face; the
independent face also owes its already admitted pending terminal.

Bytes, End and an independent output terminal enter the uncapped queue
to the protocol layer without a room reservation. The extra cell and
staged terminal cost what they hold. Over TLS, a framed connection uses
TLS's native face (../../net/tls.md, 3.6), which implements it at both its plaintext
and ciphertext boundaries.

| State | Holds | In flight | Serves | Deadline |
|---|---|---|---|---|
| Opening | owner, address | `Socket` | | |
| Socket | owner, address | `Socket` | | |
| Connecting | owner, fd | `Connect` | | |
| Announced | fd, its listener | | | |
| Open | owner, fd, intake, demand, reader, writer, output | a `Recv` while the intake has room; a `Send` or a `Shutdown` | the demand | |
| Broken | owner, fd, what is in flight | what was, finishing | | |
| Closing | owner, fd, drain, flush, output | a `Recv`, discarding; a `Send` or a `Shutdown` | | close |
| Settling | owner if bound, fd if made, what it waits for, cancels | those | | |
| Releasing | owner if bound | `Close` | | |
| Closed | | | | |

- **Opening** is a connect whose `Connecting` is not told yet. It is on
  the ready list, which the loop drains before it hands io completions.
- **Open's reader** is `Receiving` (a `Recv` in flight), `Full` (the
  intake is), `Stalled` (no buffer: waiting for the retry deadline),
  `Ended` (the peer ended, `End` not told yet) or `Told`. **Its writer**
  is `Idle`, `Sending` (finishing or not: the half-close follows the last
  of the output), `Stalled` (holding the send's box and offset), `Shutting`,
  `Unshut` (a half-close that found no buffer) or `Shut`. **Closing's
  drain** is a `Recv` in flight, `Stalled`, or done; **its flush** is
  `Sending`, `Stalled`, `Shutting`, `Unshut`, or done.

| State | Event | Next, and what it does |
|---|---|---|
| Opening | resumed | Socket: `Connecting` |
| Socket | `Socket` ok / error | Connecting, `Connect` / Closed: `Failed`, `Closed` |
| Connecting | `Connect` ok / error | Open: `Connected`, `Recv` / Releasing: `Failed`, `Close` |
| Announced | `Bind` / `Reject` | Open, `Recv` / Releasing, `Close`; the listener re-arms |
| Open | `Recv` of n > 0 | the intake appended; delivered |
| Open | `Recv` of 0 | reader Ended; delivered |
| Open | `Send` of n | the same box from its new offset, or the next, or the half-close when finishing; delivered |
| Open | `Shutdown` ok | writer Shut |
| Open | `Recv`, `Send`, `Shutdown`: no buffer | that side stalls, the same again at the retry deadline |
| Open | the retry deadline | what stalled submitted again |
| Open | `Recv`, `Send`, `Shutdown`: any other error | Broken: `Failed` |
| Open | `Demand` | stored; on the ready list |
| Open | `Send` | queued; sent at once by an idle writer |
| Open | `Finish` | the half-close, once flushed |
| Open | resumed | delivered |
| Open | `Close` | Closing, its deadline armed; Releasing if nothing is left to flush or drain |
| Broken | a completion | that operation is done |
| Closing | `Recv` | discarded; again, until 0 or an error |
| Closing | `Send`, `Shutdown` | flushed, then half-closed; an error ends the flush |
| Closing | drained and flushed | Releasing: `Close`, the deadline cancelled |
| Closing | `Recv`, `Send`, `Shutdown`: no buffer | that side stalls, the same again at the retry deadline |
| Closing | the retry deadline | what stalled submitted again |
| Closing | the close deadline | Settling: `Cancel`s; what stalled is dropped |
| Socket, Connecting, Broken | `Close` | as `Abort`: there is nothing to flush |
| Socket, Connecting, Open, Broken, Closing | `Abort` | Settling: `Cancel`s; Releasing if nothing is in flight |
| Closing | `Close` | ignored |
| Settling | a completion | done; an unsubmitted cancel retried while its target waits; nothing left: Releasing, `Close`; or Closed, `Closed`, if no fd was made |
| Releasing | `Close` done | Closed: `Closed` if bound |
| Broken, Closing, Settling, Releasing, Closed | `Stream` | dropped: the stream failed, or its owner closed it |
| Settling, Releasing, Closed | `Close`, `Abort` | ignored |
| Opening, Announced | `Close`, `Abort`, `Stream` | impossible: no token yet, or no answer yet |
| Socket, Connecting | `Stream` | impossible: a stream before it is connected |
| any | `Bind`, `Reject` but Announced | impossible: the layer above's bug |
| any but Settling | a completion `Cancelled` | impossible: io cancels only when settling |

**Delivered** is one function of Open, applied once after every
transition in the up pass (a completion, a deadline), whatever cell made
it, and by `resume` after a request in the down pass
(../../../foundation/programming-model.md, 2 and 5.4):

1. the demand's answer, if there is one: `Bytes`, if the intake meets its
   read (never after `End`); or else `Room`, if it asks for room, the
   writer takes sends, and the output has that many bytes and one more
   `Send` free. Either answer ends the demand, read and room;
2. `End`, once, if the peer ended and a read is outstanding that can
   never be met, or none is and the intake is empty;
3. a `Recv`, if none is in flight and the intake has room again.

io keeps the contract of a stream (../lib.md, 7) as the side below:

- **A demand is answered at most once:** by `Bytes`, what its read asks
  for (../lib.md, 7: exactly, or for a read of what has arrived, what the
  intake holds up to its maximum), or by `Room`, whichever io can give
  first; either answer ends it, and nothing is outstanding until the side
  above states its next. The side above states its next demand only after
  an answer, never in place of one outstanding, which io asserts.
- **`Read::Nothing` with no room withdraws** the outstanding demand, and
  only when the side above will read no more: it is closing, or its read
  crossed `End`. An answer
  already on its way may still arrive, and the side above drops it. No
  other `Bytes` or `Room` come without a demand.
- **`Io::cancel_read` pauses a readable pipe:** it withdraws the
  demand and cancels its read in flight, `Io::read_cancelled` true once both
  settle (bytes that win the race meet the old demand first); repeated
  withdrawal is inert, and the next `Stream` demand starts reading again.
- **`End` comes once nothing io holds can meet a demand:** with one
  outstanding, when it can never be met; with none, only when nothing is
  held. It comes once, and ends reading only. A read that crosses it is
  never met, and `End` does not end the demand: it stays outstanding
  until `Room` answers it, if it asked for room, or until the side above
  withdraws it. Room may still be granted after `End`, as the stream can
  still send to a peer that only half-closed. A read larger than what is
  left before the end is never met; a side above that must see every
  byte reads by its framing, or reads what has arrived.
- **`Failed` may come at any time,** a demand outstanding or not, and
  nothing follows it but `Closed`. After `End` it says only that the
  stream can no longer send: what was read stands.
- **A scan that meets no delimiter within its maximum delivers exactly
  the maximum,** and the side above decides what that means.
- **A read of what has arrived is met by the first delivered that finds
  a byte in the intake:** in the up pass after the down pass that stated
  it, if the intake holds one, or else in the pass its next receive
  lands. A receive asks for the room the intake has (at most
  `Limits::receive`), so a stream read so costs one delivery per receive,
  not one per byte. Pipes meet it as sockets do (`sockets.md`).
- **Room grants one more `Send`** of up to that many bytes, and the
  output counts a `Send` in flight until all of it is sent. A `Send` past
  the output cap or `Limits::sends`, or after `Finish`, is the layer
  above's bug, asserted; so is room demanded after `Finish`.
- **A `Send` is within the room granted** (../lib.md, 7), and io holds the
  layer above to it: each open stream keeps what its last `Room` granted,
  less what was sent since. `Room` sets it to the room asked for, each
  `Send` must fit within it and spends it, and a demand for no room leaves
  it as it is. A `Send` past it is the layer above's bug, asserted, in
  every world: it is what keeps the output under its cap by the layer
  above's asking, not by io's refusing.
- **`Failed` is told at once,** and drops what the intake held and the
  output queued: the peer is gone.
- **Discarding starts with the close,** not after the half-close, so that
  a peer blocked on an upload drains, then reads the response.
- **Only operations that wait are cancelled:** `Accept`, `Connect`,
  `Recv` and `Send`. `Socket`, `Bind`, `Listen`, `Shutdown` and `Close`
  are waited for.
- **No buffer is not a failure.** A `Recv`, `Send` or `Shutdown` that
  fails for want of kernel buffers did nothing: its side *stalls*, a
  stalled send keeping its box and offset, and is submitted again once
  the retry deadline passes. The deadlines a state implies are one
  function of it, applied after every transition: the close deadline runs
  only while closing, the retry deadline only while a side stalls.

## 4. Memory

Per entity: a slot in the slab; for each stream, an intake of `Limits::intake`; a
receive buffer of at most `Limits::receive`; the output, at most
`Limits::output` bytes in at most `Limits::sends` boxes; and a slot in the
ready lists, and two in the deadlines. An entity has at most four
operations in flight (a `Recv`, a `Send` and a cancel of each). A pipe
has at most one read or write and its cancel; a child waits and may have
one signal in flight. Thus `operations(limits)` is four per entity slot,
and one for a `Usage` (`processes.md`, 1), the ring's size (../kernel.md, 5);
the operation table holds twice that, as an operation retired in an
iteration keeps its slot until the reclaim point. Containers grow as
they fill, including a child's pipe IDs and the temporary arrays on
spawn completion. There is no per-layer heap formula to sum: scenarios
check measured peaks under named ceilings with headroom
(../../../foundation/programming-model.md, 6.3; ../../../foundation/testing-strategy.md, 6). The caller owns the
command buffers it supplies to `Spawn`.

## 5. Joined sockets

A listener on an in-process name (`README.md`, section 3) accepts sockets joined in
memory to those that connect to it, with no kernel between them. Their
owners make the same requests and hear the same events, in the same
orders, as over TCP, so a component listens and connects where its
configuration says, and its stack runs over either. jig's one-process
shape joins its roots this way.

| On a socket | On a joined socket |
|---|---|
| `Socket`, `Bind`, `Listen` | the name taken, in `down` |
| `Accept` | the first connector waiting on the name, joined |
| `Connect` | waiting on the listener until it accepts |
| `Recv` | none: the peer's moves fill the intake |
| `Send` | a move into the peer's intake, as far as its room allows |
| `Shutdown` | the peer's end, after the last byte moved |
| `Close` | the pair unjoined |
| `Cancel` | none: nothing waits on the kernel |

The machines are 2's and 3's, each kernel operation replaced as the
table says. The states that wait only on the kernel (a listener's
Socket, Binding and Arming; a stream's Socket, Settling and Releasing)
are passed through at once, and no retry deadline is ever armed.

- **The name.** `Listen` takes the name for its listener until the
  listener's `Closed`, which frees it. A name already taken, or the empty
  name, is answered `Failed` with `Other`, as an address in use is. With
  all `Limits::names` held, a `Listen` on a name is refused at the
  entrance (`Busy`), as one without a slot is.
- **Connecting.** `Connecting` comes as over TCP, and io looks the name
  up then. A name unbound, or held by a listener that is closing, is told
  `Failed` with `Refused`, then `Closed`. Otherwise the connector waits on
  the listener, in its own slot and in the order connectors came, until
  it is accepted or its owner aborts it (io has no connect timeout,
  `README.md`, section 2). A listener that closes refuses those still waiting.
- **Accepting** follows 2: armed one at a time while the listener's
  owner has room, with a free slot and room in the accept batch, starved
  without a slot. Armed, it takes the first connector waiting: io makes
  the accepted end, joins the two, tells the listener's owner `Accepted`,
  its `peer` the empty name, and the connector `Connected`. The owner
  binds or rejects it as over TCP. Nothing moves into an end until it is
  bound.
- **Moving bytes.** Each end has an intake and an output under `sockets.md`
  's caps, and no receive buffer. One move is in flight per end, in
  order: the head of its output, copied into the peer's intake as far as
  that has room, a short move continuing from the same box's offset, the
  box dropped once all of it is in. A full intake stops the peer's moves,
  so the peer's output fills and its owner's room waits: the reader's
  intake is the backpressure. Demands are met by 3's delivered, and
  independent output reservations (3) are offered as on a socket, the
  move counting as the send in flight.
- **Ends.** A half-close reaches the peer after the last byte moved, and
  the peer hears `End` as 3 tells it. A graceful close keeps `sockets.md`'s
  four steps: what is queued moves; the peer is half-closed; what the
  peer moves meanwhile is discarded as it comes, until its own end or the
  close deadline; then the pair is unjoined. An end whose peer has closed
  fails as `Reset` at its next move, its bytes lost, as after TCP's late
  reset. One whose peer aborts, is rejected, or meets its close deadline
  before its flush is done fails as `Reset` at once, or, if closing, ends
  its close. 1's rules hold as they stand; an end never settles, as it
  has nothing in the kernel, so a close with nothing to flush or drain,
  or an abort, is told `Closed` in the next up pass.
- **Nothing happens inside a request.** `down` records a request and puts
  its end on the ready list, telling nothing (`README.md`, section 2). io moves bytes,
  joins, unjoins and tells what came of each when it resumes an end, in
  the next up pass, as early as a kernel's completion could come; a move,
  or room made in an intake, wakes the peer through the same list. A
  joined end submits nothing and holds no slot of the operation table:
  its move in flight is state, the head box and its offset. Boxes come
  only from a down pass's `Send`s, and an intake drains only to a demand
  stated there or to a close, so each up pass ends.
- **Memory.** A pair holds two slots of `Limits::sockets`, one per end,
  as a loopback connection within one process does. An end holds no more
  than a socket's stream, so 4's edge limits cover it. The names table
  holds at most `Limits::names` entries of a name and its listener's id.
- **In the simulator** joined sockets run as in the real loop: they are
  io's own, never the kernel's records, so the simulator's faults and
  latency never reach them and its trace does not show them; a cut
  (../../testing/simulator.md, 3.3) drops both ends with their process. What an end
  meets comes from the owners at both ends and from the limits: short
  moves under small intakes, a peer that stops reading, closes early,
  never ends, or aborts.
- **No TLS.** The bytes never leave the process: TLS is not stacked on a
  joined socket, and no world checks it there.
