# kv-pay-per-use: contracts

The shapes that cross this plan's tasks, binding as written. Each names the task that makes
it; a task that builds on one has that task in its `needs`.

## 1. A recovery read's size (`kv-open-reads`)

kv.md, 3. In `skein/crates/skein-kv/src/store.rs`, one private function beside `log_number`:

```rust
/// What a recovery read asks: a chunk, or what the file holds past `at` (kv.md, 3).
fn read_size(chunk: u32, len: u64, at: u64) -> u32
```

`min(chunk, len − at)`; callers read only while `at < len`, so it is never 0.
Every `file::Request::ReadAt` recovery sends asks `max: read_size(env.limits.chunk, len, at)`.
A snapshot file of length 0 fails the open `Failure::Corrupt` without a read. Every later
task keeps this rule for every read it sends.

## 2. The world counts the reads it passes down (`kv-open-reads`)

In `skein/tests/kv/src/world.rs`, two public fields of `World`:

```rust
/// `file::Request::ReadAt`s the store sent below, and the sum of their `max`.
pub asked_reads: u64,
pub asked_bytes: u64,
```

Counted in `World::step` as each `ReadAt` goes to `file_layer::down`; `new`, `with_items`
start them at 0, and nothing else resets them (`crash` and `reopen` keep them). A test zeroes
them before what it measures.

## 3. The snapshot's bound (`kv-snapshot-bound`)

kv.md, 5 and 7. In `skein/crates/skein-kv/src/limits.rs`, exported from the crate root
(`pub use limits::{Limits, SNAPSHOT_FLOOR, SNAPSHOT_MULTIPLE};`):

```rust
/// The least log since a snapshot's start that the next waits for (kv.md, 5).
pub const SNAPSHOT_FLOOR: u64 = 65_536;
/// The next snapshot's trigger, in multiples of the live map's encoded size (kv.md, 5).
pub const SNAPSHOT_MULTIPLE: u64 = 2;

impl Limits {
    /// The log since the last snapshot's start past which the next starts (kv.md, 5).
    pub(crate) fn snapshot_bound(&self, live: u64) -> u64 {
        SNAPSHOT_MULTIPLE.saturating_mul(live).max(SNAPSHOT_FLOOR).min(self.snapshot_after)
    }
}
```

The live map's encoded size, kept by the map as it changes (never by a walk):

```rust
impl KvMap {
    /// Each row's key and value and 8 bytes of lengths, as a snapshot's chunks hold them.
    pub(crate) fn live(&self) -> u64
}
```

The store snapshots once `log_bytes >= env.limits.snapshot_bound(map.live())`, where it
compared with `snapshot_after` (`is_ready`, `fire`). A test computes the same bound from its
model: `live = Σ (8 + key.len() + value.len())` over the model's rows.

## 4. Recovery counts what it replays (`kv-recover-count`)

kv.md, 3 and 5. Once a recovery finishes, `log_bytes` is the bytes of the frames it replayed
from the snapshot's start (the accepted prefix, torn tails excluded). Segments named below the
snapshot's start stay in the store's segment list, unreplayed, so the next snapshot's removal
deletes them. Closing takes no snapshot.

## 5. The world's test files

Each task's tests in its own file of `skein/tests/kv/tests/suite/`, its `mod` line in
`main.rs` where named, so that tasks side by side touch separate lines:

| Task | File | `main.rs` |
|---|---|---|
| `kv-open-reads` | `open.rs` | `mod open;` after `mod memory;` |
| `kv-snapshot-bound` | `disk.rs` | `mod disk;` before `mod golden;` |
| `kv-recover-stream` | `recovery.rs` | `mod recovery;` after `mod real;` |
| `kv-recover-count` | `disk.rs` (appends) | none |

## 6. Who changes what of `store.rs`

| Task | Its region |
|---|---|
| `kv-open-reads` | the four `ReadAt`s in `up`'s `OpeningSnapshot`, `ReadingSnapshot`, `OpeningLog` and `ReadingLog` arms; `read_size` |
| `kv-snapshot-bound` | `is_ready`; `fire`'s `store.snapshot.is_none()` branch |
| `kv-recover-stream` | `Phase::ReadingSnapshot`, `Phase::ReadingLog`; `apply_snapshot`, `replay`; `up`'s `OpeningSnapshot`, `ReadingSnapshot`, `OpeningLog` and `ReadingLog` arms |
| `kv-recover-count` | `recover`, `apply_snapshot`, `next_log`, `start_recovery`, the replay code `kv-recover-stream` left |
