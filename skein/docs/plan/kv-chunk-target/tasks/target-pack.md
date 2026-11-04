+++
id = "target-pack"
part = "pack"
kind = "feature"
needs = []
files = ["skein/crates/skein-kv/src/limits.rs", "skein/crates/skein-kv/src/lib.rs", "skein/crates/skein-kv/src/snapshot.rs", "skein/crates/skein-kv/src/store/compaction.rs", "skein/crates/skein-kv/src/tests.rs", "skein/tests/kv/src/world.rs", "skein/tests/kv/tests/suite/target.rs", "skein/tests/kv/tests/suite/main.rs"]
effort = "high"
minutes = 50
+++

# skein-kv: a snapshot chunk packs to the target

## Goal

A snapshot chunk packs to `CHUNK_TARGET` (1 MiB) and grows past it only to hold one row, up
to `Limits::chunk`, the ceiling (`skein/docs/design/kv.md`, 5). Today `compaction.rs`'s `chunk`
packs to `limits.chunk`, so a store under jig's `standard` profile writes 59.57 MB chunks as
soon as it holds that much, because a row may be that large. A snapshot's step then holds the
chunk's rows and their encoding, and today's `snapshot::chunk` builds that encoding twice.

Acceptance:

- Under a `chunk` past the target, many small rows make chunks of the target's size, and the
  peak of a snapshot's step stays near the target, not `limits.chunk`.
- A row past the target has a chunk of its own; a row at `limits.chunk` round-trips.
- Under a `chunk` at or below the target, a snapshot's bytes are the same as today:
  `golden.rs` and every existing test of `skein-kv` and `skein-kv-tests` pass unchanged.
- The tests below pass, and `jig-protocol-store`, `jig-store-world` and
  `temper-one-process-world` stay green.

## Inputs

- `skein/docs/plan/kv-chunk-target/notes/common.md`; `notes/contracts.md`, 1, 2, 4 and 5.
- `skein/docs/design/kv.md`, 5 ("A chunk packs to a target", "Why a constant"), 7 ("`Limits::chunk`
  is a ceiling too", "Sizes for the layer above") and 9 ("What it pays for").
- The code: `compaction.rs` (`chunk`), `snapshot.rs` (`chunk`, `chunk_bytes`), `limits.rs`,
  `skein/tests/kv/src/world.rs`, `skein/tests/kv/tests/suite/recovery.rs` (the shape of
  `snapshot_world` and its heap measures), `skein/crates/skein-kv/src/tests.rs`
  (`snapshot_chunks_share_framing_across_rows`).

## Owns and may change

- `limits.rs`: `CHUNK_TARGET`, `Limits::chunk_target` (`notes/contracts.md`, 1), and one doc
  line on the field `chunk`: the ceiling, which holds the largest row.
- `lib.rs`: `CHUNK_TARGET` in the `pub use limits::{...}` line.
- `store/compaction.rs`: `chunk` alone.
- `snapshot.rs`: `chunk`'s body alone, its signature and its bytes unchanged.
- `tests.rs`: one test, below.
- `world.rs`: `ready` and `fire_measured` (`notes/contracts.md`, 4); `step` calls `ready`.
- `target.rs` (new) and `mod target;` after `mod store;` in `main.rs`.
- `store.rs` is `target-read`'s: leave it, and its `read_size`, as they are. Under a `chunk`
  past the target your tests' reopens read a whole file at once; that is expected until then.
- Options:
  - If a test's measured peak passes its bound for a reason other than a third copy of the
    chunk (the row list's growth, a row's boxes), widen that bound's slack up to `TARGET / 2`
    and name the measurement in the report. A peak past `2 × TARGET + TARGET / 2` is a
    question, with the measurements.
  - If `small_rows` is over the 0.5 s budget, take its rows down to 2,200 (three chunks); the
    assertions follow from `ROWS`.

## Steps

1. `limits.rs` and `lib.rs`, as `notes/contracts.md`, 1 gives them.
2. `compaction.rs`'s `chunk`, as `notes/contracts.md`, 2:

   ```rust
   let target = usize::try_from(limits.chunk_target()).expect("target fits usize");
   let mut rows = List::with_capacity(limits.chunk_target() / 8);
   // ...
   for (key, value) in store.map.after(cursor.as_deref()) {
       // ...
       let Some(next) = size.checked_add(row_size) else { break };
       // A row past the target has a chunk of its own, up to the ceiling (kv.md, 5).
       if next > target && !rows.is_empty() {
           break;
       }
       // ...
   }
   ```

   `snapshot::chunk(rows.as_slice(), limits.chunk)` stays: the ceiling, whose `expect` holds
   since a row in the map fits it.
3. `snapshot.rs`'s `chunk`: one buffer. `total` is known before any byte is written, so
   `bytes` takes the count, `total − CHUNK_BYTES` as the body's length, each row, and the
   checksum, and `body` goes. A snapshot's step then holds the rows and one encoding
   (`skein/docs/design/kv.md`, 7).
4. `tests.rs`: `chunk_target_is_the_target_under_a_higher_ceiling`: `chunk_target()` under a
   `chunk` of 4096 is 4096, of `CHUNK_TARGET` is `CHUNK_TARGET`, of `CHUNK_TARGET + 1` is
   `CHUNK_TARGET`.
5. `world.rs`: `ready` and `fire_measured`:

   ```rust
   pub fn ready(&self) -> bool {
       self.io.takes() && self.below.is_empty() && skein_kv::is_ready(&self.store, &self.env.limits)
   }

   pub fn fire_measured(&mut self) -> skein_heap::Grown {
       let span = skein_heap::Span::start();
       skein_kv::fire(&mut self.store, &self.env, &mut self.above, &mut self.below);
       let grown = span.end();
       self.drive(false);
       grown
   }
   ```

6. `target.rs`, its module doc citing `skein/docs/design/kv.md`, 5, 7 and 9, and its constants
   and helpers as `notes/contracts.md`, 5 has them. `limits()`: `key: 8`, `value: CEILING - 28`,
   `ops: 64`, `commit: CEILING`, `queued: 8`, `queued_bytes` and `segment` `2 * CEILING`,
   `budget: 4 * CEILING`, `snapshot_after: 1`, `chunk: CEILING`,
   `page: Page { rows: 16, bytes: 1024 }`, `deadline` one second. `small_rows(rows)`: an
   untraced world, opened, with rows of an 8-byte big-endian key and a 1,024-byte value,
   64 a commit, each by `request_no_fire` and answered `Committed`, as `recovery.rs`'s
   `snapshot_world` does. A row takes 1,040 chunk bytes. `chunks` walks a snapshot file past its
   18-byte header, reading each chunk's count and body length (big-endian `u32`s) and skipping
   its 12 + length bytes, until the magic `SNPE`.
   - `many_small_rows_pack_to_the_target`, `ROWS = 3_300` (about 3.3 MiB): `small_rows(ROWS)`;
     `while world.ready()`, the largest `world.fire_measured().peak`; close. `chunks` of
     `world.files()`'s `snapshot` has four chunks; each but the last at most `TARGET` bytes
     and past `TARGET - 1_040`; their rows total `ROWS`. The peak is at most
     `2 * TARGET + TARGET / 4` (the rows, a little over a target with their boxes, and their
     encoding), which is below `CEILING / 4`. Print the peak with `eprintln!`, as
     `recovery.rs` does.
   - `a_row_past_the_target_has_a_chunk_of_its_own`: one commit putting `a` and `c`, 1,024
     bytes each, around `b`, `3 * TARGET / 2` bytes; `world.drive(true)`; close. `chunks` is
     three chunks of one row each, the middle's bytes `skein_kv::chunk_bytes(1, b's length)`.
     `world.reopen()`, `Opened`, and `Get` of each key returns its value.
   - `a_row_at_the_ceiling_round_trips`: limits with `chunk: 2 * TARGET`,
     `value: 2 * TARGET - 28` and `commit: 2 * TARGET + 64`; one commit of one row of an
     8-byte key and a value at `value`; `drive`; close. `chunks` is one chunk of `2 * TARGET`
     bytes, `chunk_bytes(8, value)`. Reopen; `Get` returns the value.
7. The narrowest runs first (`heavy cargo nextest run -p skein-kv-tests --test suite -E
   'test(=target::many_small_rows_pack_to_the_target)'`), then
   `heavy cargo nextest run -p skein-kv -p skein-kv-tests`, then the readers
   (`notes/common.md`).
8. `cargo fmt`; old forms; clippy; citations (`notes/common.md`).
9. Commit (why: a snapshot chunk packs to a target, and the row ceiling stops setting a
   snapshot's buffers); push your branch.

## Report

As the implementer's text says, and also: each test's measured peak and its bound, the chunk
sizes `many_small_rows_pack_to_the_target` saw, and each new test's CPU time.

## Answers

2026-11-04 (agents/loop). The work is pushed: commit a0f45f8d3c is at origin `kv-chunk-target/target-pack` (pushed by hand after the GitHub incident of 09:49Z). Access is back. Your branch starts from it: check it, finish what's left and report `Done:`; don't redo the work.
