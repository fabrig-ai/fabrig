# The restart said on the channel

Provisional, 2026-11-16. How an engine that stops tells each worker that
it restarts, and how a worker that heard it keeps its runs through the
wait it was told rather than through its grace alone. It carries
temper's `draft/redeploys.md`, 5.2 and 5.3, for the channel. The
engine's side of the stop, keeping the word and then saying it, is
`../../domain/engine/restart.md`, 1, *Saying it restarts*; the hold over
the runs the next start restores is `../../domain/engine/fleet.md`, *A
restart said holds its runs*.

## 1. In one page

- **One record down,** `restarting`, carrying the restart wait, new in
  `jig-worker-wire` 15 (2).
- **Sent by the engine's component** when its owner says the engine
  restarts, on every channel open with its hello heard whose version
  has the kind (3). At an older version it is not sent: no mismatch,
  no floor raised (temper's `draft/redeploys.md`, 4.3), and the owner
  is told that worker was not told.
- **Kept by the worker's link with its channel** (4): when that channel
  is lost, the grace's clock runs for the longer of the wait and the
  grace, in place of the grace.
- **The rest is as before:** the redial, the clock stopped once the
  engine admits a hello, the runs cancelled for contact at the clock's
  end, and the stop bound's tail after it (`README.md`, 9.1;
  `../../domain/hosts/hub.md`, 6).

## 2. The record

```
record Restarting { wait_ms: u64 }
```

- **`FromEngine` gains `restarting: Restarting`** as its last variant,
  kind `0x0180` plus its tag, `0x018f` (`wires.md`, 4), a notice. It
  changes the channel's layout, so `jig-worker-wire` moves to 15, keeps
  14's text and brings its forward patch; its floor stays 12. Below 15
  the variant is unheld, so its reverse halts, which the engine never
  meets: it sends the kind only at a version that has it (3).
- **`wait_ms`** is the restart wait, from the engine's configuration
  (`fleet.restart_wait`, `../../domain/engine/fleet.md`), in
  milliseconds: how long after its link drops the engine asks the
  worker to keep its runs.
- **No running attempt waits on it** (`bridges.md`, 6.2): like `wires`,
  it goes only where its version has it, and is not among the kinds
  that must go down whole.
- **Its row** is `records.md`, section 2's.

## 3. The engine's side

- **Asked by its owner:** `jig-protocol-engine-workers` takes
  `Event::Restarting { wait }`, `wait` a `Duration`, through
  `host::Workers::restarting(wait)`. The owner asks once per stop, after
  the word is durable and before it closes the workers' part
  (`../../domain/engine/restart.md`, 1, *Saying it restarts*), so the
  record is queued before the close's finish and drains with the rest
  (`README.md`, 9.2).
- **Each channel open with its hello heard,** and not closing: where its
  version's kinds hold `restarting`, the component sends `Restarting {
  wait_ms }` and tells its owner the worker was told; otherwise it sends
  nothing and tells its owner the worker was not told, with the
  channel's version. A connection still opening, or past its open
  phase, is sent nothing and told of nothing.
- **What it tells:** one output per channel,

  ```rust
  Request::Told { worker: Box<[u8]>, version: u16, wait: Option<Duration> }  // jig-protocol-engine-workers' component
  Output::Told { worker: Box<[u8]>, version: u16, wait: Option<Duration> }   // its host::Workers
  ```

  `worker` the worker the channel authenticated as, as `Opened` names
  it; `version` the channel's version of
  `jig-worker-wire`; `wait` the wait sent, or none where the version
  lacks the kind. The application writes each as a line of its log
  (temper's `protocol/wires.md`, section 4).
- **Asked again,** it sends again on each such channel; a worker keeps
  the last.
- **Its frame fits every room:** a fixed record, checked with the other
  kinds when the component is made (`README.md`, section 8).

## 4. The worker's side

- **Kept with its channel.** `jig-protocol-worker-link` keeps the wait
  of the last `restarting` heard on its open channel, and tells its
  owner of each, `Request::Restarting { wait }`, `wait` a `Duration`,
  for its log (temper's `protocol/wires.md`, section 4).
- **At that channel's loss,** whatever its cause, the grace's clock is
  armed for the longer of the wait and the grace, from when the
  component notices the loss, in place of the grace (`README.md`, 9.1).
  A wait shorter than the grace never shortens what the worker keeps.
- **The word is its channel's:** it goes with the channel, so the next
  channel starts with none. A channel back that the engine admits stops
  the clock as before (`README.md`, 9.1), and its own loss, with no
  word on it, arms the grace. A channel refused, or lost before it is
  admitted, leaves the clock running from the first loss, as before, so
  a wait heard on the channel admitted before it still bounds it.
- **Redials while the clock runs:** no wait passes a quarter of the
  grace, not of the wait, and one dial is due at the clock's end less
  the handshake's deadline. So a worker in a restart wait dials a
  restarting engine as often as one in its grace, and its last dial
  leaves its handshake the time to be admitted.
- **At the clock's end,** `GracePassed` as at the grace's end: the root
  asks the hub to cancel every run for contact, and keeps the same tail
  of the stop bound after it, what is left of the bound after the
  notice and the grace (`../../domain/hosts/hub.md`, 6).
- **The stop bound the hello states is unchanged:** it bounds a link
  lost without the word, and a stop, and the engine's grace still
  outlasts it. A word heard has a bound of its own, the longer of the
  wait and the grace, then that tail, which the engine's hold covers
  (`../../domain/engine/fleet.md`, *A restart said holds its runs*).
- **Told to stop while it waits,** the worker drains as ever, within its
  stop bound (`../../domain/hosts/hub.md`, 7).

## 5. Testing

Focused, in the worlds that own each side, with their simulated clocks:

- **The worker's link** (jig's worker-link world): a wait heard, then
  the link lost: no `GracePassed` at the grace, one at the wait's end;
  the dials meanwhile at most a quarter of the grace apart, and one due
  at the wait's end less the handshake's deadline. A wait shorter than
  the grace: the grace. A wait heard, then a channel admitted and lost
  with no word: the grace from that loss. A wait heard, then a channel
  refused: the wait, from the first loss. `Request::Restarting` handed
  up once per record.
- **The engine's component:** a worker at 15 told, its frame sent and
  `Told` with the wait; one at 14 sent nothing, `Told` with none and
  14; a connection whose hello is not heard told nothing; a second ask
  sent again.
- **The wire:** generated records' drift, as skein's
  `data/codec/golden.md`, section 1 has it; the patch's goldens up and down
  with the variant refused below 15, and `KINDS` against the schema (`wires.md`, 7).
- **Fuzzy,** in the worker-link world's seeded runs: any order of words,
  losses, refusals, redials and admissions, and the clock never ends
  before the longer of the grace and the wait last heard on the channel
  whose loss armed it, counted from that loss.

## 6. Open questions

- **A worker back as a new process** proving the runs it no longer
  lists are gone, so the engine need not hold them (temper's
  `draft/redeploys.md`, section 12). Later.
