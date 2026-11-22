# A worker paused or drained over the channel

Provisional, 2026-11-20. How an engine asks a worker to take no new
work, to drain, or to take work again, and how the worker says where it
stands. It carries temper's `draft/worker-restarts.md`, section 2, for
the channel. The hub's side, its states and its drain, is
`../../domain/hosts/hub.md`, 7, *Paused over the link*; the fleet's
placement is `../../domain/engine/fleet.md`, *A paused worker leaves
placement too*; the owner's ask is `../../domain/engine/workers.md`, 5,
*Paused, drained and taking again*.

## 1. In one page

- **One record down,** `pause`, with its mode: `pause`, take no new run
  or job; `drain`, pause and drain as a stop's drain does; `resume`,
  take work again. **One record up,** `paused`, with the worker's state:
  `taking`, `paused`, `draining` or `drained`. Both are new in
  `jig-worker-wire` 19 (2).
- **Sent by the engine's component** when the fleet asks, on the
  worker's admitted channel, where its version has the kind (3). At an
  older version nothing is sent and the owner is told: the fleet's
  placement follows the ask either way, and only the drain needs the
  worker.
- **Kept by the worker's hub, not its link** (4): a pause outlives a
  channel's loss and goes with the process, so a worker's restart ends
  it. The hub says its state at each change, and again after each hello
  while it is not taking.
- **Nothing here stops a process.** Drained, a worker stays linked and
  takes nothing until it is told to take work again or stopped. When to
  stop which worker is its operator's, or the deployment's redeploy
  tool's (temper's `draft/worker-restarts.md`, section 10).

## 2. The records

```
enum PauseMode { resume, pause, drain }
record Pause { mode: PauseMode }

enum PauseState { taking, paused, draining, drained }
record Paused { state: PauseState }
```

- **`FromEngine` gains `pause: Pause`** as its last variant, after
  `figures`: kind `0x0180` plus its tag, `0x0191` (`wires.md`, 4).
  **`FromWorker` gains `paused: Paused`** as its last variant, after
  `ask`: kind `0x0100` plus its tag, `0x010e`. The layout changes, so
  `jig-worker-wire` moves from 18 to 19, keeps 18's text and brings its
  forward patch; its floor does not move.
- **Below 19 both variants are unheld,** so their reverses halt, which
  neither side meets: the engine sends `pause` only at a version that
  has it (3), and a worker sends `paused` only on such a channel (4).
- **Notices, best effort in order:** no running attempt waits on either
  (`bridges.md`, 6.2), and neither is among the kinds that must go down
  whole. Each is a fixed record, checked with the other kinds when its
  component is made (`README.md`, section 8).
- **The domain's values** are `jig_domain_link::PauseMode { Resume,
  Pause, Drain }` and `jig_domain_link::PauseState { Taking, Paused,
  Draining, Drained }`, the link vocabulary the hub and the fleet both
  name. The hub's link vocabulary carries them,
  `host::LinkDown::Pause { mode: PauseMode }` and
  `host::LinkUp::Paused { state: PauseState }` (`jig-domain-host`,
  `link.rs`); the fleet's are `fleet::Request::Pause { channel, mode }`
  and `fleet::Event::Paused { channel, state }`.
- **Their rows** are `records.md`, sections 1 and 2.

## 3. The engine's side

- **Asked by the fleet:** `fleet::Request::Pause { channel, mode }`
  goes down as a program source's `check` does: the core hands it to the
  root as a held request, the root's link delivers it on that channel
  as `host::LinkDown::Pause { mode }`, and `jig-protocol-engine-workers`
  translates it.
- **At a version with the kind,** the component sends `Pause { mode }`.
  **Otherwise** it sends nothing and tells its owner, once per ask:

  ```rust
  Output::Unpaused { worker: Box<[u8]>, version: u16, mode: PauseMode } // jig-protocol-engine-workers' host::Workers
  ```

  `worker` as `Opened` names it, `version` the channel's version of
  `jig-worker-wire`. The application writes it as a line of its log
  (temper's `protocol/wires.md`, section 4). A channel not yet admitted,
  or closing, is sent nothing and told of nothing.
- **Up:** `paused` is read as `host::LinkUp::Paused { state }`, and the
  root's link hands it to the fleet as `fleet::Event::Paused { channel,
  state }`, as `draining` becomes `fleet::Event::Draining`.

## 4. The worker's side

- **Down:** the worker's link hands `pause` up as every kind but
  `restarting` and `figures` goes: `Request::Down { down:
  host::LinkDown::Pause { mode }, .. }`, which the root gives its hub
  (`../../domain/hosts/hub.md`, 7).
- **Up:** the hub's `Request::Up(LinkUp::Paused { state })` goes as
  `paused` on a channel whose version has the kind. On an older channel
  it is not sent: only an engine at 19 or above can have asked, so the
  worker has since linked to an older one, which places on it and has
  its assignments refused as busy while the pause stands, spending no
  try.
- **After each hello** while it is not taking, the hub says its state
  again, before anything its runs send after that hello, as it says
  `draining` again while it shuts down.

## 5. Testing

Focused, in the worlds that own each side:

- **The wire:** generated records' drift, as
  skein's `data/codec/README.md`, section 6 has it; the patch's goldens
  up and down, both variants refused below 19; `KINDS` against the
  schema (`wires.md`, 7).
- **The worker's link:** a `pause` of each mode handed up as
  `LinkDown::Pause`, in order with the records around it; `paused` sent
  at 19, and not at 18.
- **The engine's component:** a worker at 19 sent `Pause` for each mode;
  one at 18 sent nothing and `Unpaused` with 18; a connection whose
  hello is not heard sent and told nothing; `paused` read up as
  `LinkUp::Paused` with each state.
- **The hub's states and the fleet's placement** are their worlds'
  (`../../domain/hosts/hub.md`, 7; `../../domain/engine/fleet.md`).

## 6. Open questions

- **A pause kept across a worker's restart,** by the engine, for a
  worker its operator means to keep out after it comes back. Today a
  restart ends a pause, which is what a redeploy wants. Later.
