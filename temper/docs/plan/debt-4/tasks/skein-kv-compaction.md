+++
id = "skein-kv-compaction"
part = "skein-kv-states"
kind = "feature"
needs = ["skein-kv-table"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/**"]
effort = "high"
minutes = 70
+++

# skein D017, the snapshot: one `Compaction` for its lifecycle

## Goal

A snapshot's run is spread over `snapshot: Option<Snapshot>`, whose `file` and `cursor` are
`Option`s, `snapshot_start`, `SnapStage` and `SnapAction`. They become the note's one `Compaction`,
each state holding its data, with one handler per cell. `snapshot_up`'s catch-all and
`#![expect(clippy::too_many_arguments)]` go. The item stays open.

Acceptance:

- `rg -n 'SnapStage|SnapAction|struct Snapshot|snapshot_start|\(_, _\)|match \(|too_many_arguments' skein/crates/skein-kv/src/store*`
  reads nothing.
- `Phase::SnapIo` holds only the log; `Dir` holds `compaction: Compaction`.
- `snapshot::` writes the same bytes: `skein/tests/kv/golden` does not move, and
  `disk.rs`'s crash cuts pass.
- `store.rs`'s lines do not rise. Past that, say why in your report.
- `#![expect(clippy::too_many_lines, ..)]` goes if `fire` was its last reason.
- The note's `What a task runs` passes, with no test changed.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/kv-store.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4; `skein/docs/design/kv.md`, 5.

## Owns and may change

- Owns: `skein-kv/src/store.rs`, and `src/store/*.rs` if split.
- Not this task's: `snapshot.rs`, which encodes the file's chunks and needs no change.

## Steps

1. Add `Compaction` and `Written`, as the note has them. `Announcing`'s end turns `Due { start }`
   into `Create { start }`, zeroing `log_bytes`, as it turns `snapshot_start` into a `Snapshot`.
   Each reader of `snapshot.is_none()` asks whether `compaction` is `Idle` by an exhaustive
   `match` in one `Compaction` method.
2. `fire`, on `Ready`: commits first, as at base; then `Idle` closes or starts a rollover as at
   base, and every other state sends its request by one match over `Compaction`, each arm a call
   to a small function. `Chunk` folds the chunk into `Written` and `cursor` as it sends; an empty
   chunk moves to `Trailer` and sends it. `RemoveOld` takes the next old segment out of `segments`
   and sends its `Remove`: none left moves to `CleanupSyncDirectory` and sends its sync. `Due` in `Ready` is
   `unreachable!` with its why.
3. `SnapIo`'s cell in `up`: on `Done::Finished` or `Done::Opened`, one match over `Compaction`
   gives the next state, then `Ready`, then `start_batch` if commits wait, as at base. `SyncDirectory`
   lands the repair, as at base.
4. On `Done::Failed`: `Create` with `Error::Exists` sends the `Remove` and moves to `RemoveTemp`;
   any state holding a `file` is the note's failure path, `SnapshotFailureClosing`, its `Close`
   state included (the note's `Behaviour that stays`); the rest fail the commits.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

`store.rs`'s lines before and after, and each `#![expect]` left and why.
