# The simulated world

Provisional, 2026-10-26. The design of `skein-sim-world`: a service's
processes hosted over the simulator and run a step at a time, as a
product a playground ships, and the two loops that run it, the world
harness's and a playground's. The harness (`skein-world`) is
examples.md, section 6 and testing.md, section 5; the simulator is
simulator.md; a world run in a browser is playground.md.

## 1. In one page

- **One world, two loops.** `skein-sim-world` holds a world: the
  simulator, its processes each a `Host` (shell.md, 12), the fake
  machine behind the simulator's seam, the program registry that hosts
  a spawn, and the clock. It runs no loop of its own. It gives a
  world's steps, and two loops run them: the world harness's run,
  which adds a scenario's referee and every world's checks
  (testing.md, 5), and a playground's slices, paced by a browser's
  clock (playground.md, 3).
- **A product, not a test kit.** A playground ships it, so it lives in
  `crates/` and links no crate under `testing/`: no referee, no
  counting allocator, no trace unless asked. The fake machine moves to
  `crates/` with it (testing.md, 4).
- **A step is one iteration:** every live process's turn, in admission
  order (reap, iterate, the hook, submit), then the machine's answers
  to the calls those submits made. Steps are counted from the world's
  start, and the count and the clock together name a moment of the
  world exactly (section 3).
- **Time moves only when the world is idle,** to the earlier of what the
  simulator has due and the processes' earliest deadline, or, when its
  loop asks, to an instant before that. Then the clock alone moves and
  no process turns, so a loop that moves time in small pieces makes the
  same run as one that jumps (section 4).
- **Input enters between steps,** into a process its loop names (a
  playground's visitor, the harness's referee acting), so a run is a
  function of its seed, its start and its inputs, each at the step and
  the instant it entered (section 5).
- **No verdicts of its own.** A step never fails the world for their
  count or for idling: the harness fails a world that does not settle
  within its steps, or that idles unsettled with nothing due; a
  playground waits (section 6).
- **The trace is kept whole or not at all,** the simulator's setting:
  whole for the harness, which replays from it; off in a playground,
  whose session log replays natively instead (section 7).
- **Portable.** It builds for `wasm32-unknown-unknown`, as the
  simulator, the fake machine and the shell's portable face do
  (shell.md, 14). The harness's real loop, its ring, its tree keeper
  and its end to end stay native (section 2).

## 2. In skein

| Crate | Where | Holds |
|---|---|---|
| `skein-sim-world` | `crates/` | the world and its steps; the program registry (`HostedProgram`, `Inherited`, startup roots and appends); the machine's seam (`Machine`, `NoMachine`); a process's cut (`Cut`: a kill, a power loss); a hosted pipe's capture (`PipeWriter`, `PipeCapture`, `PipeHost`); what processes print (`Printer`, `Printed`) |
| `skein-world` | `testing/` | the harness, over `skein-sim-world`, whose items it re-exports under their names: the run with its referee, its expectations and controls, the heap meter, the settled and teardown checks, `Outcome`, digests, and the faults and inputs hooks; the domain worlds' and protocol worlds' helpers (`domain`, `stream`); and, under `cfg(unix)`, the real loop, the ring, the tree keeper and end to end |
| `skein-fake-machine` | `crates/` | unchanged (testing.md, 4) |

- **What runs a world is the product's; what checks one is the
  harness's.** The rule decides each item: an item a playground needs
  to run its world goes in `skein-sim-world`; a referee, a meter, an
  assertion and anything native stay in `skein-world`.
- **No consumer changes.** `skein-world` keeps its name, its modules and
  its exports, re-exporting what moved, so every world in smith, jig and
  temper builds as it did. A product crate names `skein-sim-world`; a
  test may name either.
- **Depends on** lib, io, sim and shell, the shell for `Host` alone.
- **The heap meter wraps each process's turn and every harness
  operation on its containers,** so a turn is an operation of the
  world's own, which the harness's loop calls inside that process's
  ledger span, and the playground's loop calls bare. Delivering
  completions and draining submissions run inside the same span, as
  does the real loop's `ring.deliver`: a ring that grows on a harness
  push is charged to its process (testing-strategy.md, 6).

## 3. Steps

- **A step** turns every live process once, in admission order: its
  reap, its `iterate(now, wall)`, its hook, its submit to the
  simulator; a hosted child whose service has ended is dropped in its
  place. Then the machine answers every call the step's submits made
  (simulator.md, 3.1), and a spawn the registry matches is hosted.
- **A hosted child inherits its launch directory,** as a kernel gives a
  child its working directory: the spawn's directory, opened beneath
  its root, is handed to the child with its pipes and roots, owned by
  it and closed at its exit. A program that works where it was
  launched (a shell) writes beneath it; one that doesn't ignores it.
  A world whose machine opens no launch directory, and a process the
  world starts itself, give none. The harness's real loop gives one at
  every hosted spawn.
- **Counted:** the world's step count starts at 0 and grows by one per
  step; a moment of the world is its count and its clock's instant.
- **What a loop asks of it,** between steps: whether it is busy (a
  process has work pending, or the simulator something to deliver
  now); its next due instant, the earlier of the simulator's and the
  processes' `next_deadline()`, or none; whether it has settled (every
  process empty, nothing in flight, every descriptor closed); its
  clock; a process, lent to the loop to give it input (section 5); and
  a cut, planned before a submission or made now (simulator.md, 3.3).
- **What processes print:** a shell's hook writes short lines to its
  standard output or error, through a writer its caller supplies
  (shell.md, 12). In a world, the world supplies it: a printer for a
  process and a stream, handed to the process at its making, which the
  world keeps whole lines from, in the order written, until its loop
  takes them; one unread backlog for the whole world, bounded, past
  which a write would block, as a pipe's does. A playground's loop brings them to the page
  (playground.md, 5), the printed link among them; the harness may read
  them as a referee reads any output.
- **The harness's run** is the loop of examples.md, section 6 over these
  steps, unchanged in what it checks: the referee acts before a step
  and observes each turn; the heap is metered around each turn and each
  harness operation on that process's containers (section 2); when
  the world is idle it checks the teardown invariant up to the next due
  instant and advances there; it ends once the referee passed and the
  world settled, then checks every process empty.

## 4. The clock

- **Monotonic from zero, the wall from a start the world is given:**
  the simulator's configuration names the wall at its start. The
  harness gives the fixed date it gives today, so a seed replays; a
  playground gives the browser's time at its start, which its session
  log records (playground.md, 6).
- **Time moves only when the world is idle:** never past its next due
  instant, either to it, or, when the loop asks, to an earlier instant.
  Moved to a due instant, the next step turns every process at it, as
  today. Moved short of it, only the clock moves: nothing was due, so
  no process turns until input enters or the next due instant comes.
  A world moved to T in many pieces is then the same world as one moved
  to T at once, which is what lets a browser pace it (playground.md, 3)
  and a replay ignore the pace.
- **Calm, when its loop wants it:** the simulator configured with no
  fault (simulator.md, section 4). A playground's world is calm at
  every layer.

## 5. Input between steps

- **A loop gives input to a process between steps,** by the world
  lending it the process, as the harness's referee acts on its
  processes today. The process takes it as a step function takes any
  input, and acts on it at its next turn.
- **What a run is a function of:** the seed, the clock's start, and each
  input with the moment it entered, its step count and its instant. A
  loop that records those can replay the run exactly, whatever pace it
  ran at (playground.md, 6).
- **Entering an input makes the world busy** for one step at least: the
  process that took it has work pending, so the loop steps at once,
  without moving the clock.

## 6. No verdicts

- **A step fails only on a broken contract:** the simulator's, the
  machine's, or a process's own panic, each fail-stop as now.
- **The harness's verdicts are its run's:** a world that has not
  settled within its steps (a million today), or that is idle,
  unsettled, with nothing due, fails the test as now. A playground's
  loop takes neither as a failure: its world idles waiting for its
  visitor, and runs as long as the page does.

## 7. The trace

- **Kept whole, or off,** by the simulator's configuration
  (simulator.md, 3): whole by default, as every world keeps it, and
  off for a playground's, whose memory it would grow without bound.
- **A playground's world replays without it:** its session log holds its
  inputs, and the harness replays them natively with the trace whole
  (playground.md, 6).

## 8. Testing

- **The steps' own tests** are `tests/world`'s (skein-world-tests),
  over scripted processes of raw records:
  - a world stepped in slices of sizes drawn from the seed, its clock
    moved toward its next due instant in pieces drawn too, ends with the
    trace of the same world run whole;
  - a clock moved short of the next due instant turns no process;
  - an input entered at a step and an instant, replayed at the same
    step and instant, gives the same trace.
- **The harness's tests** are unchanged, and every world in skein, smith,
  jig and temper runs as before.
- **For the portable target:** the gate checks `skein-sim-world`, the
  simulator, the fake machine and the shell's portable face with
  clippy for `wasm32-unknown-unknown` (testing.md, 10).

## 9. Open questions

- **The crate's name,** `skein-sim-world`: the product's world over the
  simulator, beside the harness's `skein-world`.
