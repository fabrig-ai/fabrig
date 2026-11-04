+++
id = "kv-open-reads"
part = "open-reads"
kind = "feature"
needs = []
files = ["skein/crates/skein-kv/src/store.rs", "skein/tests/kv/src/world.rs", "skein/tests/kv/tests/suite/open.rs", "skein/tests/kv/tests/suite/main.rs"]
effort = "medium"
minutes = 45
+++

# skein-kv: an open reads what its files hold

## Goal

A store's open reads each file in reads of `min(chunk, len − at)`, never a whole chunk past
the file's end, so a small store opens with reads of its own size whatever its profile's
chunk (`skein/docs/design/kv.md`, 3). Today each recovery read asks the full
`limits.chunk`, and `skein-io`'s file layer allocates a buffer of that size for it.

Acceptance:

- Every `file::Request::ReadAt` the store sends in recovery asks `read_size(chunk, len, at)`
  (`notes/contracts.md`, 1); a snapshot file of length 0 fails the open `Corrupt` without a
  read.
- `World` counts the reads it passes down (`notes/contracts.md`, 2).
- The new test below passes, and every test of `skein-kv` and `skein-kv-tests` stays green.

## Inputs

- `skein/docs/plan/kv-pay-per-use/notes/common.md` and `notes/contracts.md`, 1, 2, 5 and 6.
- `skein/docs/design/kv.md`, 3 (the paragraph "Recovery reads each file from its start a
  chunk at a time") and 9 ("What it pays for").
- The code: `skein/crates/skein-kv/src/store.rs`, `up`'s `OpeningSnapshot`,
  `ReadingSnapshot`, `OpeningLog` and `ReadingLog` arms; `skein/tests/kv/src/world.rs`;
  `skein/tests/kv/tests/suite/store.rs` for the world's idioms; `skein_heap::Span`
  (`skein/testing/skein-heap/src/lib.rs`).

## Owns and may change

- `skein/crates/skein-kv/src/store.rs`: only the four `ReadAt`s named above, the length-0
  snapshot check in the `OpeningSnapshot` arm, and the new private `read_size` placed after
  `log_number`. Another task (`kv-snapshot-bound`) changes `is_ready` and `fire` in the same
  file at the same time: leave them, and every other line, as they are.
- `skein/tests/kv/src/world.rs`: the two counters and their counting in `step`; nothing else.
- `skein/tests/kv/tests/suite/open.rs` (new) and its line `mod open;` after `mod memory;` in
  `skein/tests/kv/tests/suite/main.rs`.
- Options: if the file layer (`skein/crates/skein-io/src/file_layer.rs`) would need a change
  for the test to pass, do not change it: stop and ask. If an existing test asserted a read's
  size, update its assertion to the new size and say so in the report.

## Steps

1. `read_size` and the four reads; the length-0 snapshot check.
2. The world's counters.
3. `open.rs`, one test, `a_small_store_opens_with_reads_of_its_own_size`:
   - limits as `store.rs`'s `limits` but `chunk: 1_048_576` and `snapshot_after: 1` (one
     snapshot after every commit the world fires, under today's rule and the next);
   - open; commit a few puts with `world.request` (a snapshot follows); then three more
     commits with `world.request_no_fire` (they stay in the log); close;
   - read the root with `world.files()` and sum the bytes of `snapshot` and every `log-*`
     file: N, a few hundred bytes;
   - zero the counters; take a `skein_heap::Span`; `world.reopen()`; end the span;
   - assert `Opened` with the last number, `asked_bytes == N` (each file read once, no read
     past its end), `asked_reads` at least the number of non-empty files read, and the span's
     peak below `chunk / 2` (no chunk-sized buffer: today's open allocates a whole chunk for
     each read); every message names what it compares.
4. `heavy cargo nextest run -p skein-kv-tests --test suite -E 'test(=open::a_small_store_opens_with_reads_of_its_own_size)'`,
   then `heavy cargo nextest run -p skein-kv -p skein-kv-tests`, then the readers that open
   stores on files: `heavy cargo nextest run -p jig-test-store-files -p jig-protocol-store`.
5. `cargo fmt`; old forms; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets --
   -D warnings`; the citations check.
6. Commit (why: an open's reads follow what its files hold, not the chunk); push your branch.

## Report

As the implementer's text says, and also: N and the counts the test saw, the span's peak, and
the test's CPU time.
