# The channel: the worker's link

## 1. The worker's link

### 1.1 The worker's side

- **Dial at once,** to the engine's address from the worker's
  configuration, with TLS by its server name and trusted roots, in
  plaintext to a loopback address, or, in one process, to an in-process
  name (skein's `runtime/io/README.md`, section 3).
- **Local refusal names** are public `u16` constants exported by
  `jig-protocol-worker-link`: `REFUSED_LIMITS = 2` and
  `REFUSED_FRAMING = 3`. These are skein's channel namespace
  (skein's `net/channel.md`, 5.1), separate from jig's application reasons
  starting at 256 (`README.md`, 4.1). All three local `Refuse`/loss pairs use them:
  incompatible terms use `REFUSED_LIMITS`, and malformed records,
  including `TurnBusy`, use `REFUSED_FRAMING`. The sent reason and
  corresponding `RefusedHere` loss agree. Applications match those
  names; wire numbers and `Closed` facts do not change.
- **Redial on loss,** after a jittered backoff that doubles up to a
  ceiling, starting over once a channel has proved itself: it carried the
  hello and stayed open for the configured span. While the grace's clock
  runs (below), no wait passes a quarter of the grace, whatever the
  ceiling, so a worker dials an engine that is restarting at least four
  times before its grace passes. One dial is due at the grace's end less
  the handshake's deadline: a wait begun before that point that would
  carry past it ends at it, and a dial after it waits as before. So the
  last dial leaves its handshake the time to be admitted before the
  grace passes, and an engine answering by then takes the hello.
- **The hub hears the link,** through the worker's root: up, once ready,
  when it asks the hub to report what it hosts and sends the hello first;
  lost, when it starts the grace's clock.
- **A loss says why** (`liveness.md`, section 1): every channel or dial that ends goes
  up once as the component's `Fact`, with its `Loss`: as skein's channel
  ended it (`Closed`: the stream, a framing error, a refusal sent or
  received), silence, stall, the handshake's deadline, the output cap
  filled, a turn busy past the busy turns it keeps (`records.md`, 2, *Busy,
  sent again*), a fault its root raised, or a dial that failed. The owner logs
  the loss of a link that said hello, beside its line for the link.
- **The grace's clock is the component's.** The hub arms no timers
  (`../../domain/hosts/hub.md`, 2). It runs from when the component notices
  the link lost. Past the grace with no channel back, the
  component tells the worker's root, which asks the hub to cancel every
  run it hosts for lost contact (`../../domain/hosts/hub.md`, 6). A channel
  back within the grace stops the clock once the engine has admitted its
  hello: at the first record the engine sends after it, `sources` (`README.md`, 4.3).
  A channel refused, at its opening or at its hello, or lost before that
  record, leaves the clock running from the first loss, never armed
  again: so a worker the engine does not take back stops its runs within
  the stop bound it stated, as one with no channel does, and the
  engine's grace, counted from its own view, still outlasts it.
- **A restart said lengthens the clock** (`restart.md`): a channel on
  which the engine said it restarts arms it, at its loss, for the
  longer of the restart wait and the grace.
- **The stop bound counts from the engine's view.** A silent partition is
  noticed by each side on its own clock, a silence deadline after the last
  frame it heard, and the engine may have last heard from the worker a
  ping interval before the partition. So the stop bound the worker states
  in its hello counts from the last frame the engine may have heard from
  it: the longer of its ping interval, its silence deadline and its grace
  together, and its drain's bound, which a worker's own shutdown runs
  first (`../../domain/hosts/hub.md`, 7); then the longer of its agents'
  stop graces and a delivery's deadline, then a save
  (`../configuration.md`, section 4). The engine's grace, which counts from when the engine notices the link lost,
  must be strictly longer, and the fleet refuses a hello whose stop bound
  is not (`README.md`, 4.3). Then no next attempt is placed while this one may still
  write (`../../domain/engine/fleet.md`).
- **Relays outlive channels.** A relayed call's answer may come on a later
  channel than the call. The component keeps every relay in flight in one
  table across channels, bounded by the hub's slots and calls per run,
  and ends each once: answered, or cancelled by the hub. Every active call
  is validated before the phase-specific send: a valid call is kept for
  replay even before hello or during an outage; an invalid call ends
  locally under its original name. Full replay capacity returns retryable
  busy without registering the call. Cancellation acknowledges only a
  relay still retained. An answer or local refusal followed by a late or
  repeated cancellation produces no second terminal. The owner stops the
  link after draining its hub and agents; the component preserves its
  stopping and closed guards and emits nothing after closed.
- **Shutting down,** the worker stops redialling once the hub has given up
  what it keeps (`../../domain/hosts/hub.md`, 7).

### 1.2 The engine's side

- **A listener per configured address,** TLS, loopback plaintext, or an
  in-process name in one process, its accepted connections bounded by
  the fleet's workers plus a few opening at once, accepted one at a time
  (skein's `runtime/io/sockets.md`).
- **A channel lost,** by silence, stall, the stream, a framing error or a
  refusal, goes up as the fleet's lost channel, once, if its hello was
  heard; the fleet keeps the worker's runs for its grace
  (`../../domain/engine/fleet.md`).
- **Every connection's end says why** (`liveness.md`, section 1): each accepted
  connection goes up once as the component's `Ended` when it ends, with
  the worker it authenticated as, the channel its hello opened, if any,
  and its `Loss`, whether or not its hello was heard: a lost channel, a
  refused opening and a connection turned away for room each say why,
  and the owner logs each.
- **Closing is bounded.** A connection that leaves its open phase, lost,
  refused or closed, holds the stall deadline from then; past it the
  component aborts the socket below, whether or not its channel has
  ended. So a worker that stopped reading never keeps a connection, its
  room or the engine's stop, and a worker lost any number of times is
  taken back each time (`README.md`, 4.2; `../../domain/hosts/hub.md`, 6).
- **During a restart,** the listener is open before decisions are: the
  core's restart adopts runs as workers say hello (`../../domain/engine/restart.md`
  ; `../service.md`, section 3).
- **Shutting down,** new openings are refused with *stopping*, and open
  channels close once the engine's last outputs to them have drained, or
  at the stall deadline from the close, past which each is aborted: the
  workers' part closes within the stall deadline of the close, whatever
  its workers do.

### 1.3 Connections' states

Each side's connection is a state machine whose state names the deadline
that runs (skein/docs/foundation/programming-model.md, section 9):

```
engine:  accepted ─► opening ─► terms ─► hello ─► open ─► closing ─► closed
worker:  idle ─► connecting ─► opening ─► open ─► closing ─► idle (redial after backoff)
```

Every state also takes the stream failing or ending, a framing error and
its deadline passing, which close it. A domain output for a channel that
is gone is dropped: its domain has heard, or will hear, that it is lost,
and resends what it must after the next hello.
