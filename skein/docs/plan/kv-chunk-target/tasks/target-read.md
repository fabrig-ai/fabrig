+++
id = "target-read"
part = "read"
kind = "feature"
needs = ["target-pack"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/frame.rs", "skein/crates/skein-kv/src/tests.rs", "skein/tests/kv/src/world.rs", "skein/tests/kv/tests/suite/target.rs", "skein/tests/kv/tests/suite/recovery.rs"]
effort = "high"
minutes = 60
+++

# skein-kv: recovery reads the target, and any chunk up to the ceiling

## Goal

A recovery read asks at most the target, not `Limits::chunk` (`skein/docs/design/kv.md`, 3), so
a restart's transients follow the target and the largest row held, not the row ceiling. Today
`read_size` asks `limits.chunk` for the log and the snapshot alike, so a restart of a store of
small rows under a 59.57 MB `chunk` holds a read of up to that size, twice over.

`snapshot::Reader` already reads what a chunk's lengths say and checks them against
`limits.chunk`: that stays, and is why a store whose chunks are packed past the target opens at
no cost, with no format change. A frame or chunk longer than a read is now the ordinary case
for a large row: the snapshot's carry reserves a chunk's length once; the log's grows by a read
at a time, and takes the same rule.

Acceptance:

- Under a `chunk` past the target, no read asks past the target, for the log or the snapshot;
  the reads still total the files' bytes.
- A restart's transient heap stays near the target for small rows, and follows the largest
  chunk or frame for a large one.
- A chunk packed past the target and up to `limits.chunk` opens; one past `limits.chunk` fails
  the open `Corrupt`.
- A frame longer than a read replays, its whole length reserved once its header has arrived.
- Every existing test of `skein-kv` and `skein-kv-tests` passes unchanged (`recovery.rs`'s
  one edit below is a word's, in a helper), and `jig-protocol-store`, `jig-store-world` and
  `temper-one-process-world` stay green.

## Inputs

- `skein/docs/plan/kv-chunk-target/notes/common.md`; `notes/contracts.md`, 1, 3, 4 and 5.
- `skein/docs/design/kv.md`, 3 (the paragraph "Recovery reads each file"), 7 ("The budget",
  "`Limits::chunk` is a ceiling too") and 9 ("What it pays for").
- The code: `skein/crates/skein-kv/src/store.rs` (`read_size`, `read_next`, `replay`,
  `log_read`), `frame.rs` (`frame_len`, `HEADER`), `snapshot.rs` (`Reader::next`, `fill`: read,
  not changed); `skein/tests/kv/tests/suite/recovery.rs` (its measures and `crc32c`);
  `target.rs` and `world.rs` as `target-pack` left them.

## Owns and may change

- `store.rs`: `read_size`, `read_next`, the end of `replay`, and the wording of the five
  `reason`s and the `expect` message that say a read is a chunk (`Limits::commit plus one read`,
  `at most the target`). Nothing else.
- `frame.rs`: `reserve_frame` alone. Its bytes do not change.
- `tests.rs`: one test, below.
- `world.rs`: `asked_most` (`notes/contracts.md`, 4).
- `target.rs`: its new tests and helpers, below.
- `recovery.rs`: `fn crc32c` becomes `pub(super) fn crc32c`, so that `target.rs` builds its
  file with it. Nothing else there.
- `snapshot.rs` stays as it is: if a test below needs it changed, that is a question.
- Options:
  - If a test's measured `peak − net` passes its bound, by less than another `TARGET`, and you
    can name what holds it (a row's boxes, the simulator's read buffer), widen that bound's
    slack by what it took and name the measurement in the report. Past that, a question with
    the measurements.
  - If a test is over the 0.5 s budget, take its row or value down within what it must show:
    `small_rows` to 2,200 rows; the long row and the frame stay past `TARGET`.

## Steps

1. `store.rs`: `read_size` takes the target, and `read_next` passes
   `step.env.limits.chunk_target()`:

   ```rust
   /// What a recovery read asks: the target, or what the file holds past `at` (kv.md, 3).
   fn read_size(target: u32, len: u64, at: u64) -> u32
   ```

2. `frame.rs`, and the end of `replay`, after the applied prefix is dropped:

   ```rust
   /// Reserves a carried frame's whole length once its header has arrived, so that it moves
   /// at most once (skein/docs/design/kv.md, 3).
   pub(crate) fn reserve_frame(bytes: &mut Vec<u8>, max: u32)
   ```

   It does nothing while the header is short or `frame_len` refuses it: the next pass of
   `replay` judges that frame, as today. `log_read`'s `reserve_exact(read.len())` stays, for
   the frames past this one.
3. `tests.rs`: `a_carried_frame_reserves_its_whole_length_once_its_header_arrives`: a frame's
   first `frame::HEADER` bytes in a `Vec` of that capacity, `reserve_frame`, and its capacity
   holds the whole frame; its first `HEADER - 1` bytes leave the capacity as it was.
4. `world.rs`: `pub asked_most: u32`, the largest `max` of a `ReadAt`, in `step` beside
   `asked_reads`.
5. `target.rs`:
   - `recovery_stays_near_the_target_under_a_large_chunk`: `small_rows(ROWS)`, `drive(true)`,
     close, `world.files()`; zero the three `asked` counters; a `Span` around `world.reopen()`;
     `Opened` at the last commit; `asked_most` at most `TARGET`; `asked_bytes` the `snapshot`'s
     and the log segments' bytes; `peak − net` at most `4 * TARGET + TARGET / 2` (the file
     layer's read and its copy, the chunk carried, its rows decoded), below `CEILING / 2`;
     every 64th row reads back.
   - `a_row_past_the_target_recovers_within_its_chunk`: `a`, `b` and `c` as
     `a_row_past_the_target_has_a_chunk_of_its_own` commits them (share the setup), the same
     measure; `asked_most` at most `TARGET`; `peak − net` at most
     `2 * TARGET + 2 * chunk + TARGET / 2`, where `chunk` is `chunk_bytes(1, b's length)`.
   - `a_frame_longer_than_a_read_replays`: `snapshot_after: u64::MAX`; one commit of `b` alone,
     `3 * TARGET / 2` bytes, and close, so that `world.files()` holds a log and no `snapshot`;
     the same measure; `Opened { last: 1 }`; `asked_most` at most `TARGET`; `peak − net` at
     most `3 * TARGET + 2 * frame + TARGET / 2`, where `frame` is
     `FRAME_BYTES + put_bytes(1, b's length)`; `Get` returns the value.
   - `a_chunk_is_read_up_to_the_ceiling`: a `snapshot` file laid by hand, as a store packing to
     its ceiling wrote one: the header (`SKVS`, version 1, start 1, its checksum), one chunk of
     two rows of 640 KiB, about `5 * TARGET / 4`, and the trailer (`SNPE`, rows, payload, its
     checksum, `END!`), big-endian, with `recovery::crc32c` (`snapshot.rs` has the layout).
     `Item::file(b"snapshot", ...)` under `limits()`: `Opened { last: 0 }`, `asked_most` at most
     `TARGET`, both rows read back. The same file under `chunk: TARGET` and
     `value: TARGET - 28`: `OpenFailed { failure: Corrupt }` with `world.io.open_files() == 0`.
6. The narrowest runs first (`heavy cargo nextest run -p skein-kv-tests --test suite -E
   'test(=target::recovery_stays_near_the_target_under_a_large_chunk)'`), then
   `heavy cargo nextest run -p skein-kv -p skein-kv-tests`, then the readers
   (`notes/common.md`). A red among the readers is a question: recovery's outcomes are not to
   change.
7. `cargo fmt`; old forms; clippy; citations (`notes/common.md`).
8. Commit (why: a restart's reads follow the target and the largest row held, not the row
   ceiling); push your branch.

## Report

As the implementer's text says, and also: each test's measured `peak − net` and its bound,
what `asked_most` and `asked_reads` were, and each new test's CPU time. `target-status` writes
the bounds into `kv.status.md` from these figures.
