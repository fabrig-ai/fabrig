+++
id = "skein-kv-recovery"
part = "skein-kv-states"
kind = "feature"
needs = ["skein-kv-compaction"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/**"]
effort = "high"
minutes = 75
+++

# skein D017, recovery: `Serving`, `Repair`, and the reading states hold what they read

## Goal

Recovery's milestones are six `bool`s and `Option`s beside the phase, and its reading states keep a
failure found part way as data. They become `Serving` and `Repair`, and each recovery phase holds
what it needs, by the note's `What the flags were`. `#![expect(clippy::struct_excessive_bools)]`
goes. The item stays open.

Acceptance:

- `rg -n '\b(recovered|open_owner|recovering_after_failure|repairing|repair_required|has_snapshot|has_temp|tail_name|segment_index|replayed)\b' skein/crates/skein-kv/src/store*`
  reads nothing. (`announce_recovered` and `next_log` are other words.)
- `ReadingSnapshot` and `ReadingLog` hold the note's fields, with no `failure`.
- `Dir` and `Store` hold no `bool`.
- `struct_excessive_bools` is gone from `store.rs`.
- The note's `What a task runs` passes: `recovery.rs`, `open.rs` and `disk.rs` above all, with no
  test changed. If a test holds a read past the first damaged frame, or a failure found after the
  first, stop and ask.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/kv-store.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4; `skein/docs/design/kv.md`, 2, 3 and 4.

## Owns and may change

- Owns: `skein-kv/src/store.rs`, and `src/store/*.rs` if split.

## Steps

1. `Repair` on `Store`, for `repair_required` and `repairing`. `Serving` on `Dir`, for `recovered`,
   `open_owner` and `recovering_after_failure`. `available()` matches `serving`, then `phase`.
   `announce_recovered` matches `serving` and leaves `Yes`.
2. `Listing` reads the entries into locals. `RemovingTempOnOpen` holds `snapshot`; recovery's start
   is a function of it. `RemovingTail` holds its name.
3. `next: usize` is each recovery phase's, as the note has it. A damaged log's `next` is
   `segments`' length, which is what it set.
4. `ReadingSnapshot`: a read that finds a failure closes the file at once, by `open_failed_file`
   with that failure; `apply_snapshot` returns it. `ReadingLog`: `replay` keeps its `Result<bool,
   Failure>`, read at once: `Err` fails the start, `Ok(false)` closes as damage, and `Ok(true)`
   reads on. A read that ends with bytes left over is damage, as at base.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

`store.rs`'s lines before and after, and each `#![expect]` left and why.
