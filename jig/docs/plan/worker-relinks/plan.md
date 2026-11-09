# worker-relinks: the plan

A deployment at fabrig `2c8b9a3f7d` (`inputs/request.md`) had a worker
that relinked every two to two and a half minutes with nothing in either
log, an engine that then stopped taking it back, and an engine that did
not stop on SIGTERM and, restarted, failed about 23 briefs on
`transcript: its window could not be loaded`. The evidence is outside the
repository, so the plan pins each fault's candidates with failing rows,
fixes what they show, and makes every loss say why, so that the next one
is never silent.

## Parts and tasks

| Part | Task | Needs | Trees | Request item |
|---|---|---|---|---|
| `engine-link` | `tasks/engine-link.md` | | jig | 2, 3 (the workers' part of a stop), 1 (the engine's reasons) |
| `worker-link` | `tasks/worker-link.md` | | jig | 1 (at the link layers), the worker's reasons |
| `restart-loads` | `tasks/restart-loads.md` | | temper | 3 (the restart's windows) |
| `worker-reads` | `tasks/worker-reads.md` | `engine-link`, `worker-link` | temper | 1 (temper's worker), the log lines of 1 and 2 |

Four tasks: the request's limit. Item 4, the design, is this cut's own
writing, not a task. Nothing is dropped; what the plan reads narrowly is
said below.

## Design written by this cut

- `jig/docs/design/protocol/channel.md`, 9: *Silence counts bytes*,
  *Reading never waits for the root*, *A lost link says why*; 10.1: *A loss
  says why*; 10.2: *Every connection's end says why*, *Closing is
  bounded*, and *Shutting down* bounded by the stall deadline; 12: the
  rows and the fifty-cut story. Its status: two **Not yet** entries, apart,
  one for each jig task.
- `jig/docs/design/domain/hosts.md`, 6.6: *A loss is told, with why* and
  *A worker comes back as often as it is lost*. Its status: a **Not yet**
  6.6, which `worker-reads` removes.
- `temper/docs/design/domain/root.status.md`: a **Not yet** 8 for the
  transcript load that answers `Failed` for room, where root.md, 8 already
  says busy and due; `restart-loads` removes it with jig's matching
  **Departs** 9.1 in `engine.status.md`.
- `temper/docs/design/domain/worker.md`, 3.1: *The link is read and fired
  on every pass* and *The entrance holds the engine's cap*. Its status: a
  **Not yet** 3.1.
- `temper/docs/design/protocol/README.md`, 15.3: the `lost` line, the
  engine's `worker lost` and `worker opening ended` lines, *A link that
  drops is never silent*, and the workers' listener's line printed when
  it fails. Its status: a **Not yet** 15.3. `worker-reads` removes both,
  with hosts.status.md's 6.6.

## Decided

- **Fault 1's likeliest cause is temper's worker, by its design.**
  `temper/docs/design/domain/worker.md`, 3.1 has the worker read its link
  only once the root has taken its last output, and the code fires the
  link's deadlines only then (`temper-service/src/worker.rs`,
  `link_taken`; `system.rs`, `route_events`). With about eight goals'
  tasks, new work waits at the watermark, the 192-record held entrance
  fills, and the worker stops reading and pinging: the engine's silence
  deadline (15 s) drops a healthy link, both sides saying nothing. That
  breaks channel.md, 9's own premise (the peer stops reading only when it
  is gone), so the cut writes *Reading never waits for the root* and
  `worker-reads` builds it. It is not proven to be the deployment's cause:
  its row and the jig rows rule candidates in or out.
- **The jig candidates get rows, not assumptions** (`worker-link`): a
  busy healthy link in the joined world, a queue that never empties while
  frames drain, and a frame in pieces past the silence deadline. The last
  is a known gap (the worker restarts silence on whole frames only) and is
  fixed whatever the deployment's cause.
- **Fault 2's candidates:** no leak per completed relink was found by
  reading. The rows lose one worker fifty times each way (silence, stall,
  a send held below, the stream failing or ending, not admitted,
  replaced, framing) in the engine's world, and fifty cuts through real
  io in the protocol world. The engine's closing state after a deadline
  loss holds no deadline (`component.rs`, `fire`), where the worker's
  aborts at one: the cut writes *Closing is bounded*, which also bounds
  the workers' part of a stop (fault 3).
- **"Its stop grace" adds no figure.** The engine has no overall stop
  deadline, and the request forbids new figures without a question. The
  cut reads the request's stop grace as the bound its parts' own deadlines
  give, and bounds the one part found unbounded, the workers' channels, by
  the stall deadline. No temper-level stop row is written: the one-process
  world's worker is in-process and cannot stop reading; the workers'
  part's row is jig's (`engine-link`). If the user wants an overall stop
  deadline, that is a figure and a question for a later plan.
- **The restart's failures are a recorded departure.** About 23 briefs
  failing on the window, then five tasks held, match temper's root
  answering a full load-slot table `Failed` (transient) where root.md, 8
  says busy and due (`engine.status.md`, **Departs** 9.1).
  `restart-loads` answers the activation busy when no load slot is free.
- **Loss reasons are one closed set per side,** in `notes/contracts.md`:
  the engine's `Loss` on a new `Request::Ended` (once per connection, a
  refused or turned-away one included, so "an engine that refuses a link
  logs why"); the worker's `Loss` replacing `Closed` in its `Fact`.
  `Lost` stays as it is, for the fleet.
- **Cross-tree lines.** The jig tasks change types temper reads; each
  carries temper's compiler-forced lines only (an arm, a type path), as a
  required field's are carried. The temper log lines are `worker-reads`'.

## Critical path

`engine-link` (200 min) and `worker-link` (160 min) run at once, with
`restart-loads` (100 min); `worker-reads` follows both jig tasks. The path
is `engine-link` then `worker-reads`.

## Overlaps not avoided

- `ignored-misc`'s merged `drain-placement` changed both link components,
  the fleet and `temper-service/src/workers.rs` on its trunk, not yet on
  main: one line in `jig-protocol-worker-link/src/component.rs`, four in
  `jig-protocol-engine-workers/src/translate.rs`, one in `workers.rs`.
  `engine-link` and `worker-link` touch the same files; the hunks are
  apart, and whichever lands second merges them. The same trunk changed
  `temper-service/src/tests/worker.rs` (about fifty lines), which
  `worker-link` touches by a type path and `worker-reads` by a new row.
- `jig/tests/worker-link/src/joined.rs` is `worker-link`'s, and
  `engine-link` adds one arm at each of its two matches over the engine's
  requests; `worker-link` is told to leave those matches alone.
