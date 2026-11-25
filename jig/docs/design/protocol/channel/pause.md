# A worker paused or drained over the channel

Provisional, 2026-11-20. How an engine asks a worker to take no new
work, to drain, or to take work again, and how the worker says where it
stands. It carries temper's `draft/archive/worker-restarts.md`, section 2, for
the channel. The hub's side, its states and its drain, is
`../../domain/hosts/hub.md`, 7, *Paused over the link*; the fleet's
placement is `../../domain/engine/fleet.md`, *A paused worker leaves
placement too*; the owner's ask is `../../domain/engine/workers.md`, 5,
*Paused, drained and taking again*.

## 1. In one page

- **One record down,** `pause`, with its mode: `pause`, take no new run
  or job; `drain`, pause and drain as a stop's drain does; `resume`,
  take work again. **One record up,** `paused`, with the worker's state:
  `taking`, `paused`, `draining` or `drained` (2).
- **Sent by the engine's component** when the fleet asks, on the
  worker's admitted channel, where the worker's layout has the kind (3).
  Where it lacks it nothing is sent and the owner is told: the fleet's
  placement follows the ask either way, and only the drain needs the
  worker.
- **Kept by the worker's hub, not its link** (4): a pause outlives a
  channel's loss and goes with the process, so a worker's restart ends
  it. The hub says its state at each change, and again after each hello
  while it is not taking.
- **Nothing here stops a process.** Drained, a worker stays linked and
  takes nothing until it is told to take work again or stopped. When to
  stop which worker is its operator's, or the deployment's redeploy
  tool's (temper's `draft/archive/worker-restarts.md`, section 10).

## 2. The records

```
enum PauseMode { resume, pause, drain }
record Pause { mode: PauseMode }

enum PauseState { taking, paused, draining, drained }
record Paused { state: PauseState }
```

- **`FromEngine` has `pause: Pause`** and **`FromWorker` has `paused:
  Paused`**, each a kind numbered by its place (`wires.md`, 4). The
  engine sends `pause` only where the worker's layout has it (3), and a
  worker sends `paused` only where the engine's terms list it (4;
  `layouts.md`, 4).
- **Notices, best effort in order:** no running attempt waits on either,
  and neither is among a running attempt's kinds (`layouts.md`, 5.2).
  Each is a fixed record, checked with the other kinds when its
  component is made (`liveness.md`, section 1).
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
- **Where the worker's layout has the kind,** the component sends
  `Pause { mode }`. **Otherwise** it sends nothing and tells its owner,
  once per ask:

  ```rust
  Output::Unpaused { worker: Box<[u8]>, version: u16, mode: PauseMode } // jig-protocol-engine-workers' host::Workers
  ```

  `worker` as `Opened` names it, `version` the channel's version of
  `jig-worker-wire`, its frozen one. The application writes it as a line of its log
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
  `paused` where the engine's terms list the kind, and is not sent
  otherwise. If a paused worker links to an engine without `pause`,
  that engine can still place on it. The worker refuses those assignments
  as busy while the pause stands, spending no try.
- **After each hello** while it is not taking, the hub says its state
  again, before anything its runs send after that hello, as it says
  `draining` again while it shuts down.

## 5. Testing

Focused, in the worlds that own each side:

- **The wire:** generated records' drift, as
  skein's `data/codec/golden.md`, section 1 has it; per-value refusals
  for `pause` to a worker's layout without it and `paused` up to an
  engine's layout without it (`layouts.md`, 8); `KINDS` against the
  schema (`wires.md`, 7).
- **The worker's link:** a `pause` of each mode handed up as
  `LinkDown::Pause`, in order with the records around it; `paused` sent
  only where the engine's terms list it.
- **The engine's component:** at the frozen wire version, a worker whose
  layout has `pause` is sent `Pause` for each mode; a layout without it
  gets no frame, and the owner gets `Unpaused` with that same version.
  A connection whose hello is not heard is sent and told nothing;
  `paused` is read up as `LinkUp::Paused` with each state.
- **The hub's states and the fleet's placement** are their worlds'
  (`../../domain/hosts/hub.md`, 7; `../../domain/engine/fleet.md`).

## 6. Open questions

- **A pause kept across a worker's restart,** by the engine, for a
  worker its operator means to keep out after it comes back. Today a
  restart ends a pause, which is what a redeploy wants. Later.
