+++
id = "memory-worlds"
part = "memory-worlds"
kind = "feature"
needs = []
files = [
  "skein/tests/kv/tests/suite/target.rs",
  "skein/tests/kv/tests/suite/disk.rs",
  "skein/tests/kv/tests/suite/recovery.rs",
  "skein/tests/kv/src/world.rs",
  "skein/tests/llm/tests/suite/memory.rs",
  "skein/tests/tls/tests/suite/memory.rs",
  "skein/tests/tls/tests/suite/exchange.rs",
  "skein/tests/io/src/files.rs",
  "skein/tests/io/tests/suite/files.rs",
  "skein/tests/io/tests/suite/scenarios.rs",
  "skein/docs/plan/skein-limits-heavy/figures/memory-worlds.md",
]
effort = "medium"
minutes = 130
+++

# Four worlds under ceilings: `kv`, `llm`, `tls` and `io`

## Goal

Four rows of 3.43 CPU s between them, largest first: `skein-kv-tests::suite` `target`, `disk`, `recovery` (1.22); `skein-tls-world::suite` `memory`, `exchange` (0.82); `skein-llm-world::suite` `memory` (0.76); `skein-io-world::suite` `scenarios`, `files` (0.63). They all belong (`plan.md`, *Belonging*). Three counts are decided below. For the rest, find the cause and fix it, row by row, with the stop rule on each. Every ceiling stays where it is and every assertion stays as strict.

Acceptance:

- `skein/tests/kv/tests/suite/target.rs`'s `ROWS` is 2,500, and a `const` check holds the reason in step 2. `TARGET`, `CEILING`, `SNAPSHOT_BOUND` and `recover`'s bound don't change.
- `the_largest_admitted_store_stays_within_its_memory_ceiling` reads in larger pieces and asserts what it asserts, against a `CEILING` of 170,000.
- `skein/tests/io/tests/suite/scenarios.rs` runs two calm and two chaos seeds.
- `thirty_two_unknown_arrays_exercise_the_compact_record_ceiling` still has 32 arrays of 1,900 values, 4,100,000 as its ceiling and `most > 2_000_000`.
- Each smaller count bites, as the common note asks.
- `skein/docs/plan/skein-limits-heavy/figures/memory-worlds.md` has each row's costliest and median tests, before and after, and *Counts* lines.
- In each row, the next cause's gain is under about 0.05 CPU s, or what's left is under *Found and not fixed*.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, items 3 and 4, and these rows.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*, and 8.1.
- `skein/docs/design/kv.md`, 5, 7 and 9: what `target.rs` holds.

## Owns and may change

Owns the files in the header.

May change, for a cause a profile puts there, at the size it needs, with no change of behaviour and a focused test if it's product code:

- `skein/crates/skein-kv/**`, `skein/crates/skein-io/**`.
- `skein/testing/skein-llm-kit/**` and `skein/testing/skein-tls-kit/**`. These have readers in every tree: the kit's `limits`, `events`, `response` and PKI helpers reach `smith`, `jig` and `temper`. Keep a change's bytes and meaning the same, and check each reader package with `heavy cargo check --tests`.

Doesn't change: `skein-json`'s product code, `skein_heap`, or `Wire`. `skein-world-costs` will profile a median `skein-llm-world` and `skein-tls-world` test for the formatted-trace pattern in world helpers; a test's own fixture is this task's. If a merge meets theirs in a kit, keep their change.

Options, each decided:

- **`Observed::check`'s `world.files()` is the cost in `disk`:** it reads and copies every file every ten commits, and `retained_log` may need only lengths. Add a `World` method that stats, if that's all it needs. `World::files` stays for the tests that read bytes.
- **`has_later` is a cost** (`skein/crates/skein-kv/src/map.rs`, a scan of a batch's later ops for each op): fix it in the product if a batch's keys can be read once. Refusals and order stay exact. A person pays that one too.
- **A `disk` or `recovery` count looks large:** 2,000 commits, 1,500 churn commits, eight of 480, or `SNAPSHOT_ROWS`. Lower one only with a *Counts* line showing, from its assertions, that it still does the same thing: it passes the floor, grows above it, or still crosses its allowance. Otherwise it stays.
- **`closes_in_every_state` stays over seeds,** with its assertions on which waits were seen. Lower `0..60` and `0..10` to the smallest ranges that hold every assertion, doubled for margin, in a named constant.
- **`failures_and_closes_in_every_state`'s 30 offsets** stay, unless you show which offsets add no state: it records none, so you may print the state each reaches. If a tail adds none, cut it and keep a few past the last new one. All four version and ending rows stay.
- **TLS settings rebuilt per row are a cost:** build the immutable settings once per test and share them. If `measure` counts them under its ceiling, they stay counted: a row must not read lower because its settings moved outside the meter. The ceiling doesn't move.
- **`response`'s `format!` per chunk, or `events`' copies, are the cost in `llm`:** write the same bytes more cheaply in the kit. The 37-byte chunks stay: other worlds' transcripts rest on them.
- **`skein/tests/io/src/files.rs`'s `Judge` rescans, or its `read_whole` allocates a buffer per read:** the second is product code, in `skein/crates/skein-io/src/file_layer.rs`. Fix it if it's above the stop rule once the read size is larger.

## Steps

1. **Profile each row before changing it,** with `--test suite`:
   - `-p skein-kv-tests`: `target::recovery_stays_near_the_target_under_a_large_chunk`, `target::many_small_rows_pack_to_the_target`, `disk::a_tiny_store_keeps_its_log_under_the_floor`.
   - `-p skein-tls-world`: `memory::failures_and_closes_in_every_state`, `exchange::closes_in_every_state`, `exchange::close_notify_from_the_server_ends_the_stream`.
   - `-p skein-llm-world`: `memory::thirty_two_unknown_arrays_exercise_the_compact_record_ceiling`, `memory::preparation_and_streaming_stay_under_the_peak_ceiling`.
   - `-p skein-io-world`: `files::the_largest_admitted_store_stays_within_its_memory_ceiling`, `scenarios::a_server_that_does_not_demand_stops_its_peer`, `files::a_conflict_leaves_the_other_writers_file`.
2. **`ROWS` to 2,500.** A smaller store must still be past the bounds it's held to, or these tests stop biting. At 1,040 bytes a row, 2,500 rows are 2.48 `TARGET`: two full chunks and a third. A snapshot or a recovery that held every row beside one whole encoding of them would hold 4.96 `TARGET`, past `SNAPSHOT_BOUND`'s 2.25 and `recover`'s 4.5. No fewer: below about 2,300 rows that last bound stops biting.

   ```rust
   // About 2.5 MiB: two full target chunks and a third, and rows and one whole
   // encoding of them pass every bound these tests hold.
   const ROWS: u64 = 2500;
   const _: () = assert!(2 * ROWS * ROW_BYTES as u64 > (4 * TARGET + TARGET / 2) as u64);
   ```

   Write that check in the file's idiom: `as` may be `u64::from` in a `const` helper or spelled with `u64` constants, as the lints allow. Bite: make the snapshot pack to its `chunk` limit in place of the target, in `skein-kv`. See the `eprintln!`'s peak pass `SNAPSHOT_BOUND`, and `recover` fail on a bound. Restore.
3. **The largest store reads in larger pieces.** `Owner::new` (`skein/tests/io/src/files.rs`) reads 2 bytes at a time, so this test makes 32,768 reads and an end. The ceiling is on the whole size, which is what the store holds, and small reads are what the small stories show. Use `read: READ.max(max / 64)`: 1,024 here, and 2 for every story with `max` under 192, with a comment saying why. Check `rg 'world\(|world_with_files\(' skein/tests/io` for another story with a large `max`, and that `fuzzy::files` still runs. `CEILING` stays 170,000. If its comment's `132,257 B` moves, write the new peak there. Bite: lower `CEILING` just below the new peak, and see the check fail, which shows the larger reads still reach the peak it holds. Restore. The *Counts* line gives the peak before and after: if it fell by more than the reads' own size, the smaller read was showing something, so say what, and keep the old size for this test.
4. **`scenarios.rs`'s seeds:** `CALM` is `[1, 2]`, `CHAOS` is `[5, 6]`. `fuzzy::scenarios` sweeps the same scenarios at many seeds. Section 8: "the few seeds that show its behaviour". Bite: make `backpressure`'s server demand early, so that its referee's bound fails at a seed kept. Restore. A test that truly needs a seed you've dropped keeps it in a local list, with a comment naming what it shows.
5. Fix each row's causes, largest first, re-measuring after each.
6. Write the figures.
7. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p skein-kv-tests -p skein-llm-world -p skein-tls-world -p skein-io-world --all-targets -- -D warnings` with any other package you changed. Then `heavy cargo nextest run` on those packages and each product or kit package you changed, and `--profile fuzzy` on `skein-io-world`. Commit and push.

## Report

For each row, what its tests paid for and the before and after, each bite, the causes found and not fixed, and which options you took. Any product change, with the test that holds it. Your implementation and check minutes apart.
