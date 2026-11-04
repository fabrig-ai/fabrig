+++
id = "kv-recover-stream"
part = "recover-stream"
kind = "feature"
needs = ["kv-open-reads"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/snapshot.rs", "skein/crates/skein-kv/src/frame.rs", "skein/tests/kv/tests/suite/recovery.rs", "skein/tests/kv/tests/suite/main.rs"]
effort = "high"
minutes = 120
+++

# skein-kv: recovery a chunk and a frame at a time

## Goal

Recovery applies the snapshot a chunk at a time and the log a frame at a time as its reads
arrive, holding one read and the chunk or frame being applied, never a whole file
(`skein/docs/design/kv.md`, 3 and 7). Today the `ReadingSnapshot` and `ReadingLog` phases
gather a whole file in a `Vec` before decoding it, and the snapshot's rows are decoded into one
more box of every row, so a restart's transient heap grows with the retained log (up to the
profile's `snapshot_after`) and with the map (about three times it).

Acceptance:

- Recovery reaches the same outcome as today for every file content: the same `last`, map,
  failure (`Corrupt`, `Full`, `Io`), torn-tail repair and segments replayed; the existing
  crash, torn-tail, short-read, golden and real-ring tests of `skein-kv-tests` stay green
  unchanged.
- Its reads keep `read_size` (`notes/contracts.md`, 1), and
  `open::a_small_store_opens_with_reads_of_its_own_size` stays green.
- The tests below pass.

## Inputs

- `skein/docs/plan/kv-pay-per-use/notes/common.md` and `notes/contracts.md`, 1, 5 and 6.
- `skein/docs/design/kv.md`, 3 (the recovery rule and the paragraph "Recovery reads each file
  from its start a chunk at a time"), 7 ("The budget"), 9 ("What it pays for") and 12 (the
  snapshot's trailer).
- The code: `skein/crates/skein-kv/src/store.rs` (`Phase::ReadingSnapshot`,
  `Phase::ReadingLog`, `apply_snapshot`, `replay`, and `up`'s `OpeningSnapshot`,
  `ReadingSnapshot`, `OpeningLog` and `ReadingLog` arms), `snapshot.rs` (`decode`), `frame.rs`
  (`frame_len`, `decode`); `skein/tests/kv/tests/suite/store.rs` (its `frame` helper, the
  crash cuts) and `memory.rs` (a measured ceiling's comment); `skein_heap::Span`
  (`skein/testing/skein-heap/src/lib.rs`).

## Owns and may change

- `skein/crates/skein-kv/src/store.rs`: the region `notes/contracts.md`, 6 gives this task;
  the phases' fields are yours to reshape (the carried bytes live in the phase, D017), and
  `apply_snapshot` and `replay` may be replaced by what streams. Another task
  (`kv-snapshot-bound`) may be changing `is_ready` and `fire` at the same time: leave them,
  the `use` lines, and every other line as they are; reach new items through the existing
  `frame::` and `snapshot::` paths.
- `skein/crates/skein-kv/src/snapshot.rs` and `frame.rs`: yours, `pub(crate)` only; the bytes
  each writes are the stored format and do not change (the golden test holds them).
- `skein/tests/kv/tests/suite/recovery.rs` (new) and its line `mod recovery;` after
  `mod real;` in `skein/tests/kv/tests/suite/main.rs`.
- Options:
  - The snapshot's decoder: a `pub(crate)` incremental reader in `snapshot.rs` that takes the
    bytes as they are read and gives each chunk's rows once that chunk checks, with every
    check `decode` makes (magic, version, the header's CRC, a start past 0, each chunk's
    count past 0, its size within the chunk limit, its CRC, keys strictly increasing across
    chunks, the trailer's CRC, rows, payload and `END`, ending at the file's length). `decode`
    goes if nothing else uses it. A shape of your own inside those checks is yours; name it
    in the report.
  - A row may move from the decoder into the map without a second copy: if `KvMap::apply`'s
    clone of each key and value stands in the way, keep it (its signature is
    `kv-snapshot-bound`'s this wave) and say so in the report.
  - If your base has `kv-snapshot-bound`'s `KvMap`, use `check` and `apply` as it leaves
    them.

## Steps

1. The log: `ReadingLog` carries the bytes read and not yet applied. After each read, apply
   every whole frame from an offset into them, with today's checks in today's order
   (`frame_len` within `limits.commit`, `decode`, the ops count, the next number, the
   budget), then drop the applied prefix once per read, never a move per frame. A frame
   longer than a read waits for the next reads; bytes left at the file's end are a torn tail,
   as today. A segment whose first number is not the next is not replayed, as today (it may
   close without being read).
2. The snapshot: `ReadingSnapshot` carries the incremental reader; rows go into the map as
   each chunk checks, `last` is set from the header's start, and the open fails `Corrupt`
   when the bytes end before a checked trailer, as a whole one would (kv.md, 3). The
   snapshot's length cap stays.
3. `recovery.rs`, limits `key: 8, value: 64, ops: 4, commit: 512, queued: 8,
   queued_bytes: 8192, budget: 1_048_576, segment: 1_048_576, snapshot_after: u64::MAX,
   chunk: 4096, page: Page { rows: 16, bytes: 1024 }, deadline: 1 s`, and a frame helper as
   `store.rs`'s. A constant `RECOVERY_ALLOWANCE`: the world's own transient heap through an
   open (the directory's listing among it), measured as an empty root's open's
   `peak − net`, with a quarter of headroom rounded up to two significant digits, its comment
   saying so as `memory.rs`'s does.
   - `an_empty_root_opens_within_the_allowance`: that measurement, asserted; and the
     allowance is below a quarter of the long log and of the snapshot below, so the bounds
     show what they claim.
   - `recovery_holds_a_read_and_a_frame_whatever_the_log`: a laid segment
     `log-00000000000000000001` of one-put frames over 4 keys, of about 16 KiB and of about
     256 KiB in two runs. Each: `World::with_items`, `skein_heap::Span::start()`,
     `world.request_no_fire(Open)`, end the span; `Opened` with the last number, each key's
     last value read back; and the span's `peak − net` at most
     `3 × chunk + commit + RECOVERY_ALLOWANCE` (the file layer's read and its copy, and the
     bytes carried: a read beside a frame's start).
   - `recovery_holds_a_read_and_a_chunk_whatever_the_snapshot`: 2,048 rows of 64-byte values
     committed with `request_no_fire` (four puts a commit), then `world.drive(true)` (with
     `snapshot_after` set to 1 for this test, one snapshot of the whole map), then close.
     Measure `world.reopen()` with a span: `Opened`, every row read back, and `peak − net` at
     most `4 × chunk + RECOVERY_ALLOWANCE` (the file layer's read and its copy, the bytes
     carried, and one chunk's rows).
   - `a_snapshot_cut_short_fails_the_open`: with `chunk: 128` (a row a chunk), a snapshot of
     four rows, made by the store as above and read with `world.files()`; laid alone, cut at
     every length from 0 to one byte short, each open is `OpenFailed { failure: Corrupt }`
     with `world.io.open_files() == 0`, naming the length; the whole file opens with every
     row.
4. The narrowest runs first (`heavy cargo nextest run -p skein-kv-tests --test suite -E
   'test(=recovery::...)'`), then `heavy cargo nextest run -p skein-kv -p skein-kv-tests`,
   then the readers that recover stores from files:
   `heavy cargo nextest run -p jig-protocol-store -p jig-store-world -p temper-one-process-world`.
   A red there is a question: recovery's outcomes are not to change.
5. `cargo fmt`; old forms; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets --
   -D warnings`; the citations check.
6. Commit (why: recovery holds a read and a frame, not the log or the snapshot); push your
   branch.

## Report

As the implementer's text says, and also: each test's measured `peak − net` and its bound,
the allowance's measurement, the decoder's shape, and each test's CPU time.
