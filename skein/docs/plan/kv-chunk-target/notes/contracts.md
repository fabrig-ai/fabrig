# kv-chunk-target: contracts

The shapes that cross this plan's tasks, as its binding calls.

## 1. The target

In `skein/crates/skein-kv/src/limits.rs`, beside `SNAPSHOT_FLOOR`, and exported in `lib.rs`'s
`pub use limits::{...}` line:

```rust
/// What a snapshot chunk packs to and the most a recovery read asks (skein/docs/design/kv.md, 3 and 5).
pub const CHUNK_TARGET: u32 = 1_048_576;
```

and on `Limits`, beside `snapshot_bound`:

```rust
/// The target under these limits: no chunk passes `chunk`, the ceiling (skein/docs/design/kv.md, 5).
pub(crate) fn chunk_target(&self) -> u32 {
    CHUNK_TARGET.min(self.chunk)
}
```

`Limits` keeps its fields, `is_usable` its checks: no literal in any tree changes.

## 2. Packing (`target-pack`)

A chunk takes rows in key order until the next would take it past `limits.chunk_target()`; a
row that alone passes the target goes alone. So, in `compaction.rs`'s `chunk`, the break is
`next > target && !rows.is_empty()`, and the row list's capacity is
`limits.chunk_target() / 8`. `snapshot::chunk(rows, limits.chunk)` keeps its signature and its
check against the ceiling. A row in the map fits the ceiling (`map.check` and `is_usable`), so
its `expect` stands.

## 3. Reading (`target-read`)

`read_size` asks `limits.chunk_target()` where it asked `limits.chunk`. `snapshot::Reader`
keeps its `chunk_limit` check against `limits.chunk`: that is what opens any chunk up to the
ceiling and refuses one past it. A frame carried across reads reserves its whole length once
its header has arrived.

## 4. The world

In `skein/tests/kv/src/world.rs`:

- `target-pack` adds:

  ```rust
  /// Whether the store has a chunk or a replacement step for `fire`, and io room to ask it.
  pub fn ready(&self) -> bool
  /// One `fire`, measured alone, then the world driven to rest without another.
  pub fn fire_measured(&mut self) -> skein_heap::Grown
  ```

  `ready` is the condition `step` fires on, and `step` calls it.
- `target-read` adds `pub asked_most: u32`, the largest `max` of a `ReadAt`, kept beside
  `asked_reads` and `asked_bytes`.

## 5. The suite

`skein/tests/kv/tests/suite/target.rs`, `mod target;` after `mod store;` in `main.rs`, is made
by `target-pack` and extended by `target-read`. It holds:

```rust
const TARGET: u32 = skein_kv::CHUNK_TARGET;
const CEILING: u32 = 16 * TARGET;
const OWNER: Token = Token::new(1);

/// Limits with a `CEILING` chunk, snapshotting whenever the store is fired.
fn limits() -> Limits;
/// A store of `rows` rows of a kilobyte, committed and not yet snapshotted, its world untraced.
fn small_rows(rows: u64) -> World;
/// Each chunk of a snapshot file, its rows and its bytes, read from its lengths.
fn chunks(snapshot: &[u8]) -> Vec<(u32, u32)>;
```
