+++
id = "kv-snapshot-bound"
part = "snapshot-bound"
kind = "feature"
needs = []
files = ["skein/crates/skein-kv/src/limits.rs", "skein/crates/skein-kv/src/lib.rs", "skein/crates/skein-kv/src/map.rs", "skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/tests.rs", "skein/tests/kv/tests/suite/disk.rs", "skein/tests/kv/tests/suite/main.rs"]
effort = "high"
minutes = 75
+++

# skein-kv: a snapshot follows the live map

## Goal

A snapshot starts once the log since the last one's start passes
`min(snapshot_after, max(SNAPSHOT_FLOOR, SNAPSHOT_MULTIPLE × live))`, where `live` is the map's
encoded size: the disk a store keeps follows its live data, and `snapshot_after` is a ceiling
only (`skein/docs/design/kv.md`, 5 and 7). Today a snapshot waits for `snapshot_after` bytes of
log whatever the map holds, so a busy store with a map of a few MB keeps up to the profile's
`snapshot_after` (355 MB in jig's small profile) of stale log.

Acceptance:

- `SNAPSHOT_FLOOR`, `SNAPSHOT_MULTIPLE`, `Limits::snapshot_bound` and `KvMap::live` as
  `notes/contracts.md`, 3 writes them, `live` kept as the map changes; `is_ready` and `fire`
  compare `log_bytes` with `env.limits.snapshot_bound(map.live())`.
- The unit and world tests below pass; every test of `skein-kv` and `skein-kv-tests` stays
  green, the golden files unchanged (`skein/tests/kv/tests/suite/golden.rs`).
- The readers' tests have run (step 5), and any red among them is reported as that step
  says.

## Inputs

- `skein/docs/plan/kv-pay-per-use/notes/common.md` and `notes/contracts.md`, 3, 5 and 6.
- `skein/docs/design/kv.md`, 5 ("When"), 7 ("Limits are ceilings", "Sizes for the layer
  above") and 9 ("What it pays for").
- The code: `skein/crates/skein-kv/src/limits.rs`, `map.rs` (`check`, `apply`), `store.rs`
  (`is_ready`, `fire`, and how `log_bytes` grows in the `Syncing` arm and resets in the
  `Announcing` arm), `lib.rs`; `skein/tests/kv/src/world.rs` and
  `skein/tests/kv/tests/suite/store.rs` for the world's idioms.

## Owns and may change

- `skein/crates/skein-kv/src/limits.rs`, `lib.rs` (the export line), `map.rs`: yours.
- `skein/crates/skein-kv/src/store.rs`: `is_ready` and `fire`'s `store.snapshot.is_none()`
  branch only. Another task (`kv-open-reads`) changes the recovery arms of `up` in the same
  file at the same time: leave them, the `use` lines, and every other line as they are.
- `skein/crates/skein-kv/src/tests.rs`: your unit tests, appended at its end.
- `skein/tests/kv/tests/suite/disk.rs` (new) and its line `mod disk;` before `mod golden;` in
  `skein/tests/kv/tests/suite/main.rs`.
- Options:
  - `live` is kept in `apply` as `used` is: if keeping it there needs a change to `check`'s
    signature, make it and say so; never recompute it by walking the map.
  - If an existing skein test changes behaviour because a snapshot now starts sooner, it is
    a finding: report it, and change the test's limits only if the test's own purpose is
    unchanged by it (its comment says what it shows), saying so in the report.

## Steps

1. The constants and `snapshot_bound` in `limits.rs`, exported; `KvMap::live`; `is_ready` and
   `fire`. Document each public item with its kv.md section.
2. Unit tests in `skein/crates/skein-kv/src/tests.rs`:
   - `snapshot_bound_follows_the_live_map_between_floor_and_ceiling`: live 0 gives
     `SNAPSHOT_FLOOR`; a live size whose double passes the floor gives its double; a ceiling
     below either gives the ceiling; `u64::MAX` live saturates to the ceiling.
   - `the_map_keeps_its_encoded_size_through_puts_overwrites_and_erases`: on a
     `crate::map::KvMap`, each batch checked then applied as the store does: after puts, an
     overwrite with a longer and a shorter value, an erase, and one batch that puts and then
     erases one key, `live` equals `Σ (8 + key + value)` over the rows the map holds.
3. `disk.rs`, with limits `key: 8, value: 64, ops: 16, commit: 2048, queued: 8,
   queued_bytes: 16_384, budget: 262_144, segment: 8192, snapshot_after: 16_777_216,
   chunk: 1024, page: Page { rows: 16, bytes: 1024 }, deadline: 1 s` (the ceiling far above
   what the tests write), a model `BTreeMap` of what is committed, and two helpers: the bytes
   of every `log-*` file in `world.files()` (the retained log) and the bound from the model
   (`notes/contracts.md`, 3). Each commit goes through `world.request`, so the world fires a
   due snapshot to its end before the next.
   - `a_tiny_store_keeps_its_log_under_the_floor`: 16 two-byte keys, 2,000 one-put commits of
     64-byte values over them. After every tenth commit and at the end, the retained log is
     at most the bound plus one `segment`; at the end the `snapshot` file's start (its bytes
     6..14, big-endian) is past 1,000, so snapshots ran far below `snapshot_after`.
   - `a_churning_store_keeps_its_log_within_twice_its_map`: 512 two-byte keys filled by 32
     commits of 16 puts of 64-byte values, then 1,500 one-put commits over the same keys.
     The same check after every tenth commit; and the most retained log seen passes
     `SNAPSHOT_FLOOR`, so the multiple, not the floor, set the bound. Every model row reads
     back with `Get` at the end.
   Each assertion names the commit number, the retained log and the bound.
4. The narrowest runs first (`heavy cargo nextest run -p skein-kv --lib -E 'test(=tests::...)'`,
   `heavy cargo nextest run -p skein-kv-tests --test suite -E 'test(=disk::...)'`), then
   `heavy cargo nextest run -p skein-kv -p skein-kv-tests`.
5. The readers' tests: the bound changes what `snapshot_after` means (a snapshot may start
   where none did, in a store whose log passes 64 KiB under a larger ceiling). Run, one
   command each:
   `heavy cargo nextest run -p jig-protocol-store -p jig-test-store-files -p jig-store-world -p jig-one-process-world -p jig-channel-world`
   and
   `heavy cargo nextest run -p temper-one-process-world -p temper-service -p temper-engine-domain-world -p temper`.
   A red there that a snapshot starting sooner explains (a file list, a segment's name, a
   count of operations or files): read its log, then report it under `## Left` with the test,
   the log's lines and why; change nothing outside skein. A red that a snapshot cannot explain
   (a recovered value, a lost or extra event) is a question: stop and ask.
6. `cargo fmt`; old forms; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets --
   -D warnings`; the citations check.
7. Commit (why: the snapshot's trigger follows the live map, `snapshot_after` a ceiling);
   push your branch.

## Report

As the implementer's text says, and also: the most retained log each world test saw against
its bound, the number of snapshots each ran, each test's CPU time, and every reader's run with
its count and result.
