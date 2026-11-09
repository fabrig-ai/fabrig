# worker-link-quiet: the plan

A deployment at fabrig `a66fcfb9f7` (`inputs/request.md`) had one task's
start make a remote worker's link go quiet each time it was released:
the worker lost the link by silence about 15 s later, every run on it
was lost, the link came back and went quiet again about every 2 min
16 s, and after eight losses the task held. Nothing crossed the link in
the quiet, pings included. The evidence is outside the repository.

## The cause, by reading

- skein's channel asks its stream for room for a whole frame
  (`skein/crates/skein-channel/src/machine.rs`, `Output::Free`); skein-io
  leaves an ask past its output cap unanswered, by its contract
  (`skein/docs/design/lib.md`, 7.1;
  `skein/crates/skein-io/src/stream.rs` and `joined/end.rs`,
  `OutputDown::Room`). Nothing checks a link's frames against that cap.
- While a frame is queued, the engine's component arms no ping
  (`jig/crates/jig-protocol-engine-workers/src/connection.rs`,
  `next_deadline`). A frame past the cap therefore silences the engine
  until the worker's 15 s silence deadline, before the engine's 30 s
  stall: the worker logs `silence`, the engine `stream`, both sockets
  idle and empty.
- Both stock profiles admit a resumed start far past the 1 MiB
  `io.output`: 32 committed calls of up to 69,632 bytes (small) or
  135,168 (standard) alone pass it. A producer resumed after lost runs
  carries the calls committed since its last turn.
- The fleet's grace (120 s) plus the silence (15 s) is the 2 min 16 s
  cycle; the lost-runs allowance ends it at eight.

The request's other candidates are ruled out by reading, and the rows
say so: both sides refuse an opening whose terms leave a gap
(`jig/crates/jig-protocol-worker-wire/src/kinds.rs`, `terms_gap`), so no
frame passes the peer's terms but by a broken contract; the worker reads
its link on every pass (`worker-relinks`); a frame past the worker's
terms that arrives is a framing loss, named.

## Parts and tasks

| Part | Task | Needs | Trees | Request item |
|---|---|---|---|---|
| `engine-room` | `tasks/engine-room.md` | | jig (temper's lines) | 1 (the failing story, at the joined link), 2 |
| `worker-room` | `tasks/worker-room.md` | | jig (temper's line) | 2 (the worker's side) |
| `temper-room` | `tasks/temper-room.md` | `engine-room`, `worker-room` | temper | 1 (the stock settings' story), 2 (the line) |

Three tasks, within the request's four. Item 3, the design, is this
cut's own writing. Nothing is dropped.

## Design written by this cut

- `jig/docs/design/protocol/channel.md`: 5.2, *A start the link cannot
  carry ends there too*; 9, *Every frame fits the stream below* and *A
  start past the room fails its run, never the link*; 11, the room among
  *The component's own limits*, and *The room is the stream's, not the
  domains'*; 12, *A frame past the room, in the machine worlds*. Its
  status: two **Not yet** entries, apart, one for each jig task.
- `temper/docs/design/domain/worker.md`, 4.3: *A start the link cannot
  carry*. Its status: a **Not yet** 4.3, `temper-room`'s.
- `temper/docs/design/protocol/README.md`, 15.3: the `worker unsent`
  line's row. Its status: a **Not yet** 15.3, `temper-room`'s.

## Decided

- **The fix is the room, in jig's two components, not in skein.**
  skein's design already puts the bound on whoever stacks a machine on a
  stream (lib.md, 7.1: a Room's bytes fit the lower's cap); the link's
  owners never checked it. Each component is given its stream's room
  (`largest_room`), and no frame past it reaches the channel. skein
  stays unchanged, which keeps smith, hearth and every other channel
  owner out of this plan.
- **Every kind but the engine's starts fits the room by a check when the
  component is made;** the starts cannot, since both profiles admit
  starts past it. A start past the room is answered by the engine's
  component itself, as one that cannot encode is today: refused,
  invalid start, its words naming its bytes and the room. The fleet
  counts it as any invalid start; the link, its pings and the other runs
  go on.
- **`Unsent` is told once** for every record not sent, with its kind,
  bytes and bound, and serve writes `worker unsent` (README 15.3). A
  refused run also gets serve's existing refused-run line, with the
  words.
- **A channel refusal of a start** (anything but a full queue), today
  dropped unanswered, takes the same path; it is a broken contract after
  a terms-checked opening, so rows reach it at the component's step
  level if at all. A job that cannot encode, today discarded, is
  answered too.
- **The worker's side changes nothing at runtime:** its frames are
  bounded by its own limits, so a check when it is made is the whole
  guarantee.
- **No figure.** Making the deployment's start run needs one (a larger
  `io.output`, fewer committed calls in a start, or frames in pieces):
  `[[question]] start-past-room`, for a later plan; no task waits.
- **Cross-tree lines.** The jig tasks carry temper's compiler-forced
  lines only: the `room` values (`notes/contracts.md`, section 4) and a
  no-op arm for `Output::Unsent`. `temper-room` writes the note.

## Critical path

`engine-room` (200 min) and `worker-room` (80 min) start at once;
`temper-room` (180 min) follows both. The path is `engine-room` then
`temper-room`, about 380 implementer-minutes.

## Overlaps not avoided

- `engine-room` and `worker-room` both reach
  `jig/tests/worker-link/src/joined.rs`: `engine-room` owns it (the
  joined story, the pipe's cap, the engine's `Limits` and its requests'
  matches); `worker-room` adds one `room:` line in the worker's `Limits`
  literal, if the file has one. Both edit
  `jig/docs/design/protocol/channel.status.md`, each removing its own
  entry, three entries apart.
- Both jig tasks add one `room:` line in temper's and
  `jig-test-service`'s limits builders, in different functions
  (`workers.rs`, `limits` and `worker_settings.rs`, `link_limits`;
  `limits.rs`'s `workers` and `worker_link`).
- `ui-ask-m4`'s `see-withheld` owns `temper/crates/temper-service/src/tests/**`;
  `temper-room` adds a new module there, its `mod` line after the last
  one in `tests.rs`, a row in `tests/workers.rs` and a constructor in
  `fixtures/worker_peers.rs`. Whichever lands second merges them.
