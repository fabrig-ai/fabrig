# The store: commits

## 1. Commits

- **Numbers are shared.** The journal numbers the root's commits, and
  skein-kv numbers its own from one, across restarts. Every root commit
  is one skein-kv commit, and nothing else writes the store but
  the upgrade, before the root starts, whose last commit writes the header
  with its own number (`opening.md`, 2), so the numbers are equal whenever the
  root runs. The component checks this as it submits, and stops on a
  mismatch. At start, skein-kv's recovered last commit must equal
  the commit the deployment's header names, which every writing decision
  saves; a store that disagrees refuses the start.
- **Writes become operations** in order: a save a put of its encoded key
  and record, an erase an erase, a copy a put under its second key of the
  bytes the map holds under its first, the application's bytes under its
  tag. The store's writes name no domain's operation: which key a copy
  reads and which it writes are the domain's (the core's for a settings
  revision that enters history). A copy reads the map as its commit found
  it, so a save of its first key earlier in that commit is not what it
  copies; a first key the map doesn't hold fails the commit.
  A key written more than once in one commit becomes one operation, its
  last write, which leaves the map as the writes in order would:
  skein-kv applies a commit whole (skein's `data/kv.md`, section 7), so
  nothing sees the writes between.
  A turn's save becomes a frame in its task's file, written first
  (`transcripts.md`, 2), and puts of its place in the map.
- **In order, whatever waits.** A commit with turns goes to skein-kv once
  their files are synced, and the commits after it queue behind it, so
  numbers reach skein-kv in order. The wait is one sync, and only for the
  commits behind a turn.
- **Group commit.** While a sync is in flight, commits queue, and the next
  write carries them all (skein's `data/kv.md`, section 4). The root's bound
  on commits in flight is checked at start against skein-kv's queue and
  queued bytes, and the component's queue behind transcripts' syncs, so
  the store never answers a commit busy.
- **Committed n** goes up when skein-kv answers that commit, which makes
  every earlier one durable too, with the store's fill (`growth.md`, 1.2).
- **A failure stops the engine.** skein-kv answers failed for every commit
  in flight when a write or a sync fails or stalls, and a failed write or
  sync of a transcript fails its commit the same way. The component
  answers failed, the root stops (`../../domain/engine/decisions.md`, 1.1), and the
  process ends; the next opens the store, which recovers. The engine
  never carries on over a store that recovered beneath it.
- **A refusal is a failure.** Past the checks at start, a commit skein-kv
  refuses (too large, the map full) would be a decision already made and
  not kept, so it stops the engine as a failure does. `growth.md`, section 1 keeps
  the map from filling.
- **A failure says why.** `Event::Failed { number, why }` carries
  `why: CommitFailure`: which of the store's failures it was, as data,
  bounded, with no record's contents and no text the store builds. The
  application words it (temper's `protocol/store-versions.md`, 1). It is
  in-process: no journal record, stored family or wire message carries
  it.

  | `CommitFailure` | The commit |
  |---|---|
  | `Refused(skein_kv::Refusal)` | skein-kv refused it at its entrance, its refusal as skein's `data/kv.md`, 2 gives it: the map full with its bytes, its budget and the bytes this commit would leave; a key, a value, the ops or the frame past its limit; busy; unavailable |
  | `Failed` | skein-kv answered it failed: a write or a sync of its log failed or stalled, and its outcome is uncertain |
  | `Owner { owner }` | skein-kv answered `Committed` for `number` under `owner`, the root's commit it was submitted for, which isn't `number` |
  | `Transcript` | a file step of its transcripts failed: a frame's file opened, made, written or synced, its directory's sync or close, a dropped transcript's removal, or the writer's own bookkeeping (its table of open files, its counter) |
  | `Check(CommitCheck)` | the store's own checks refused it before skein-kv saw it, naming the check |
  | `Stopped` | it was queued, or waiting on its frames, when another commit's failure stopped the store |

  `CommitCheck` names the check: `Number` (its number isn't the store's next,
  or is at or before the last durable one), `Closed` (the store isn't
  open), `Queue` (the queue behind transcripts' syncs is full), `Writes`
  (more writes than the root's bound, `Commits::writes`), `Switching`
  (wires, a dropped transcript or a turn while an upgrade switch is
  pending), `Encoding` (a key, a record, a turn's frame or the first
  commit's own rows past its codec's limits), `Copy` (a copy's first
  key the map doesn't hold, or its read failing), `Reads` (a read of the
  map the commit needs, for a turn's place, a dropped transcript's
  attempts or the openings it erases, failing or not decoding),
  `Capacity` (more operations, places or keys than the commit's bounds
  hold), `Bytes` (its operations past the store's commit bytes,
  `Limits::commit` of its skein-kv limits). `CommitFailure` and `CommitCheck`
  are `Copy`.

  The commit whose failure it is carries its cause, and each commit
  skein-kv answers failed carries `Failed`; every other commit the stop
  fails, queued behind it or waiting on its frames, carries `Stopped`,
  as do the earlier pending commits a `FrameTooLarge` fails (below). A
  failed commit of the secrets' store answers as before.
- **An oversized transcript frame refuses the whole commit before data
  IO.** Each actual encoded current frame, including all metadata, payload
  and its wrapper, must fit the store's page byte bound `P`; equality is
  admitted. All frames and map operations are prepared before any of that
  commit's data is written, so a later oversized turn leaves none of the
  refused batch on disk. Its one terminal is
  `FrameTooLarge { number, task, bytes, limit }`, where `bytes` is the full
  encoded frame length and `limit` is `P`. There is no additional `Failed`
  for that number. This terminal comes first, followed by the
  failures of earlier pending commits, and the component stops as for an
  ordinary commit failure. Previously durable data remains recoverable.
  The event is strictly in-process: no journal record, stored family or
  wire message carries it. An application feeds the rejected number to its
  uncommitted-commit policy and may report the named cause.
- **A commit's operations are bounded,** as skein-kv applies each commit
  in one step (skein's `data/kv.md`, section 7). Where removing a record would
  erase an unbounded number of rows (a transcript's attempts' places,
  `transcripts.md`, 5), its commit erases the row that names the rest, so nothing reads
  them from then on, and the rest, orphans, go in the commits that
  follow: the component adds a bounded number of their erases to each
  commit it makes for the root, within that commit's operations, until
  none is left. At open it finds any a restart left by the rows they
  name being gone. Every commit stays one decision's, numbered as the
  root numbers it.
