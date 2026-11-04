+++
id = "kv-recovery"
part = "kv-review"
kind = "feature"
needs = ["kv-forwarding"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/compaction.rs", "skein/tests/kv/tests/suite/recovery.rs"]
effort = "high"
minutes = 40
+++

# skein-kv: the recovery leftovers, and the part's lines

## Goal

The review's Part A, finding 8, the part's last. The cut read each of its five items in the code and
agrees:

- `snapshot_read` and `log_read` share an `if match` plus `.expect`, and four sites build the same
  `ReadAt` request: a `read_end` and a `read_next` helper.
- `log_opened` takes a `ReadAt` whose `at` is always 0.
- `snapshot_read`'s `next` is an offset and then a segment index.
- `fire_ready`'s `RemoveTemp` arm sends a `Remove` from a cell that cannot happen: `create_failed` sends
  it from `up` and `finished` answers it, so `RemoveTemp` never meets `Phase::Ready`.
- `remove_old`'s dedup exists only because `finish_recovery` keeps the tail's listed name in `segments`
  while `segment_created` pushes it again.

Acceptance:

- `rg -n 'file::Request::ReadAt' skein/crates/skein-kv/src/store.rs` reads 1.
- `rg -n 'Recreating an empty tail' skein/crates/skein-kv/src/store/compaction.rs` reads nothing.
- `fire_ready`'s `RemoveTemp` arm is `unreachable!` with its why, and `remove_temp` stays for `create_failed`.
- Recovery over an empty tail followed by a snapshot removes that segment once and fails nothing, on a
  test that goes red if `finish_recovery` keeps the name with the dedup gone.
- The note's `What a task runs` passes, and the tests of `skein/tests/kv` that exist today pass unchanged.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/skein-kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/design/kv.md`, 3 and 5; `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: `skein-kv/src/store.rs`'s recovery handlers and `finish_recovery`; `src/store/compaction.rs`'s
  `fire_ready` and `remove_old`.
- `skein/tests/kv/tests/suite/recovery.rs`: it may add the tail test, and change nothing there. If a test
  there or in `store.rs` already goes red on that hand-break, name it and add none.
- If `log_opened` needs more than seven arguments once `at` goes, keep a small struct without `at`.

## Steps

1. The helpers, near `read_size`:

   ```rust
   /// Where this read ends, if it is a read this file can give.
   fn read_end(source: &ReadAt, read: &[u8]) -> Option<u64>;

   /// Ask the next read of `file`, at `at`.
   fn read_next(file: Token, len: u64, at: u64, step: &mut Step<'_>);
   ```

   `snapshot_opened`, `snapshot_read`, `log_opened` and `log_read` use them; a `None` is the
   `open_failed_file(.., Failure::Corrupt, ..)` each holds today.
2. `log_opened` takes `file` and `len`. In `snapshot_read`, the offset is `end`, as in `log_read`, and
   `next` is the segment index alone.
3. `fire_ready`'s `RemoveTemp` is `unreachable!`, with its why beside `Due`'s.
4. `finish_recovery` takes the tail's name out of `dir.segments` as it finds it, so `segment_created`'s
   push is its one entry, and `remove_old`'s dedup loop goes. Test as the acceptance says.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

The common note's table for the tail test. The note's line counts, and the part's total against the
request's aim: `store.rs` and `compaction.rs` together read 1,670 at the base, and about 200 down less
what the fixes add is near 1,490, not 1,250. Report the number; do not cut anything else to reach it.
