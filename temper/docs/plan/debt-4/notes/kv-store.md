# debt-4: skein-kv's store as a 5.4 machine

The binding shape for the `skein-kv-*` tasks, which pay
`skein/docs/development/debt/D017-kv-store-states.md`. Each task moves `skein-kv/src/store.rs`
one step towards it, with `skein/tests/kv` green; names below are binding, and what is inside a
handler is the implementer's. `store.rs` may split into `store/*.rs` modules if a task finds that
shorter; a module-wide `#![expect]` is out in those too.

## What the store is

`State` is the store's life, `Phase` the one outstanding file request, and each lifecycle beside
them one enum of its own (`skein/docs/foundation/programming-model.md`, 5.4).

```rust
pub struct Store {
    state: State,
    repair: Repair,
    last: u64,
    map: KvMap,
    queue: Queue<Pending>,
    queued_bytes: u64,
    admitted: Queued,          // map.rs; every commit in flight or queued
}

enum State {
    Closed,                    // the terminal state, and the transitions' placeholder
    Memory,                    // Store::memory: commits apply at once
    Open { dir: Dir, phase: Phase },
}

/// What an open directory holds in every phase.
struct Dir {
    root: Fd,
    serving: Serving,
    close: Option<Token>,      // the slot's whole state: a `Close` waits for the store to settle
    segments: ...,             // the log's segment names, in order
    log_bytes: u64,
    compaction: Compaction,
}

/// Who hears when the store serves (kv.md, 2).
enum Serving {
    Yes,
    Opening { owner: Token },  // the owner's `Open`, not answered yet: `Opened`
    Recovering,                // no request waits: `Recovered`
}

/// Whether the log needs a snapshot before the store serves (kv.md, 3 and 4).
enum Repair { Clean, Required }
```

`Phase` holds what each request needs and nothing else:

```rust
enum Phase {
    Listing,
    RemovingTempOnOpen { snapshot: bool },   // a `snapshot` file was listed
    OpeningSnapshot,
    ReadingSnapshot { file: Token, len: u64, at: u64, reader: snapshot::Reader },
    ClosingSnapshot { next: usize },         // the first segment to replay
    OpeningLog { name: Box<[u8]>, next: usize },
    ReadingLog { file: Token, len: u64, at: u64, bytes: Vec<u8>, next: usize },
    ClosingRecovered { next: usize },
    RemovingTail { name: Box<[u8]> },
    Creating { name: Box<[u8]> },
    Announcing { file: Token },
    Ready { log: Log },
    Writing { log: Log, batch: Queue<Pending> },
    Syncing { log: Log, batch: Queue<Pending> },
    RolloverClosing,
    SnapIo { log: Log },
    SnapshotFailureClosing { log: Log },
    RecoveryClosing,
    Closing,
    OpenFailureClosing { failure: Failure },
}

#[derive(Clone, Copy)]
struct Log { file: Token, offset: u64 }
```

`Compaction` is a snapshot's way from the log's bound to the replaced file (kv.md, 5). It lives
beside `Phase` because commits interleave with it. Each state is the step `fire` sends next, and
stays the step in flight until `up` answers it, so `SnapStage` and `SnapAction` are one enum:

```rust
enum Compaction {
    Idle,
    Due { start: u64 },                     // starts with the next segment
    Create { start: u64 },
    RemoveTemp { start: u64 },              // `Create` found one: sent from `up`
    Header { start: u64, file: Token },
    Chunk { written: Written, cursor: Option<Box<[u8]>> },
    Trailer { written: Written },
    Sync { start: u64, file: Token },
    Close { start: u64, file: Token },
    Rename { start: u64 },
    SyncDirectory { start: u64 },
    RemoveOld { start: u64 },
    CleanupSyncDirectory,
}

/// What the snapshot file holds so far.
struct Written { start: u64, file: Token, offset: u64, rows: u64, payload: u64 }
```

`fire` folds a chunk into `Written` and `cursor` as it sends it, and takes an old segment's name out
of `segments` as it sends its `Remove`: a failure drops them anyway, since recovery lists the
directory again. On an empty chunk it moves to `Trailer` and sends that.

## One file answer

At most one request is outstanding and the state knows which, so an answer without a payload says
only that the request finished. `up` narrows the event once:

```rust
/// What the one outstanding request came back with.
enum Done {
    Listed { entries: Box<[file::Entry]> },
    Opened { file: Token, len: u64 },
    Read { bytes: Box<[u8]> },
    Finished,                               // Written, Synced, Closed, Removed, Renamed
    Failed { error: Error },
}

fn done(event: file::Event) -> Done;        // every other file event is unreachable! with its why
```

The cell table is then `match state`, and in each arm a `match` on `Done`: one or two lines of
live cells, one `Done::.. | Done::.. => unreachable!("why")`. Each cell is a call to a handler
that takes the source state's data by value and returns the target. A small struct carries
`env`, `above` and `below` through the handlers, so that none takes more than seven arguments.

```rust
let state = mem::replace(&mut store.state, State::Closed);
store.state = match state {
    State::Closed | State::Memory => unreachable!("no file request is outstanding"),
    State::Open { dir, phase } => match phase { .. },
};
```

## What the flags were

Read against `store.rs` at the plan's base, for the tasks that drop them:

- **`repairing` and `repair_required` are one flag.** `repairing` is written with `repair_required`
  at every damage site, is set from it in `finish_recovery`, is cleared with it when the repair's
  snapshot lands, and is read only after `finish_recovery`. `recover()` clears `repairing` alone,
  but nothing reads it before `finish_recovery` sets it again. `Repair` holds the one value, on
  `Store`, because it survives a failed start: `open_failed` leaves `repair_required` set, and the
  next `Open` repairs.
- **`recovered`, `open_owner` and `recovering_after_failure` are `Serving`,** and `Store::memory`'s
  `recovered` is `State::Memory`. `fail_commits` turns `Yes` into `Recovering` and leaves
  `Opening` as it is, as `announce_recovered`'s order says. `available()` is `Serving::Yes` and a
  phase that takes commits.
- **`has_snapshot`** is written at the end of a snapshot and read by nothing after: `recover()`
  clears it first. It goes, and `Listing` reads the entries into a local. **`has_temp`** is local
  to `Listing`. **`tail_name`** is `RemovingTail`'s, and **`segment_index`** is each recovery
  phase's `next`.
- **`active`** is `Writing`'s and `Syncing`'s `batch`. **`snapshot`** and **`snapshot_start`** are
  `Compaction`.
- **`failure` and `replayed` go:** recovery stops at the first failure or damaged frame, as kv.md,
  3 says, and closes the file at once. A read that is never sent cannot fail, and a doubly bad
  snapshot reports what was found first. Those are the only visible differences.

## Behaviour that stays

- `Request::Open` on a `Store::memory` store opens it, and `recover` clears what it held.
- Every refusal stays in its order.
- A `Close` while recovering waits, and `open_failed` answers it.
- A failed rollover create or directory sync stops the store as today, unanswered:
  `skein/docs/design/kv.status.md`, 4. Do not fix it here.
- A `Close` answered `Failed` by the file layer on a snapshot's file closes it once more, as the
  failure path does at base.
- `is_ready`, `fire`, `up` and `down` keep their public signatures; `_above` stays.

## What a task runs

`cargo fmt`; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets -- -D warnings`;
`heavy cargo nextest run -p skein-kv -p skein-kv-tests`. No golden moves, and no test of
`skein/tests/kv` or `skein-kv/src/tests.rs` changes unless a spec says so. `skein/tests/kv/golden`
holds the on-disk format. This departs from the implementer's text: `skein-kv`'s public items do
not change, nor what they mean, so its dependents run only in `skein-kv-subset`.
