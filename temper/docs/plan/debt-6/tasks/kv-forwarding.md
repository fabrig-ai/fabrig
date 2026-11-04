+++
id = "kv-forwarding"
part = "kv-review"
kind = "feature"
needs = ["kv-admission"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/compaction.rs"]
effort = "medium"
minutes = 30
+++

# skein-kv: the forwarding handlers and the `unreachable!` arms go

## Goal

The review's Part A, findings 6 and 7. `compaction::follow` is exactly `start_batch`, which begins with
the same queue test: the cut read both. `recover`, `next_log` and `start_recovery` take `step`, and the
store wrappers that only unpack it go. The 17 three-line `unreachable!` arms in `up` become one
function's call. Behaviour does not change.

**What the cut read differently from the review:**

- `recovery_closed` closes a snapshot's file since `kv-snapshot-file`: it is a handler, and it stays.
  Three wrappers go, not four: `temp_removed`, `recovered_file_closed` and `tail_removed`.
- The request leaves the relabelling handlers as they are. The review counts eight; `compaction.rs`
  holds ten beside `directory_synced`. Leave every one of `created` to `cleanup_synced`.

Acceptance:

- `rg -n 'fn follow\b' skein/crates/skein-kv/src/store/compaction.rs` reads nothing, and
  `rg -n 'fn (temp_removed|recovered_file_closed|tail_removed)' skein/crates/skein-kv/src/store.rs` reads
  nothing. `compaction.rs`'s own `temp_removed` stays.
- `rg -c 'must match this phase' skein/crates/skein-kv/src/store.rs` reads 1.
- `up` stays exhaustive: every cell of `Done` is named, with no `_`.
- The note's `What a task runs` passes with no test changed.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/skein-kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: `skein-kv/src/store.rs` and `src/store/compaction.rs`.
- `up`'s `#[expect(clippy::too_many_lines, ..)]` goes if the lint no longer fires, and stays if it does.

## Steps

1. Delete `follow`; `opened` and `finished` end in `store.start_batch(dir, log, &step.env.limits, step.below)`.
2. `recover`, `next_log`, `start_recovery` and `finish_recovery` take `step: &mut Step<'_>`. In `down`'s
   `Open` arm and `recovery_closed`, pass it. `up`'s cells call `store.start_recovery`, `store.next_log`
   and `create_segment` directly, and the three wrappers go.
3. In `up`:

   ```rust
   /// A file answer `up` has no cell for: at most one request is outstanding, so the phase knows it.
   fn wrong_answer() -> ! {
       unreachable!("the file answer must match this phase's request")
   }
   ```

   Each of the 17 arms becomes one line, `Done::Listed { .. } | Done::Opened { .. } | Done::Read { .. } =>
   wrong_answer(),`. `Phase::SnapIo`'s arm keeps its own message.
4. What the note's `What a task runs` names; the citations check. One commit for each finding. Push.

## Report

The note's line counts. No test changed, so no hand-break.
