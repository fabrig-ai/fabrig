# io: status

2026-11-04. Where skein's code lags or departs from [io.md](io.md).

- **Not yet:** 5: a tree's metadata, the filesystem's status and sync, and
  btrfs's subvolumes (`StatEntry` to `SubvolumePath`). 6, 6.2 to 6.4:
  cgroups and contained trees.
- **Departs:** 3, 5, 7: for pipes, files and signals,
  `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no
  lifecycle in flags or `Option`s beside it, exhaustive matches and one
  handler per cell; `Pipe` and `Signals` in `skein-io/src/pipe.rs` and
  `skein-io/src/signals.rs` keep their lifecycle in flags and `Option`s,
  and `skein-io/src/file_layer.rs`'s `FileIo` keeps pending, outstanding,
  cancellation and stop state in separate `Option`s and repeats
  `Failed { committed: false, residue: None, .. }` literals; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D018-io-pipe-and-file-states.md`).
