# io: status

2026-11-06. Where skein's code lags or departs from [io.md](io.md).

- **Not yet:** 5: a tree's metadata, the filesystem's status and sync, and
  btrfs's subvolumes (`StatEntry` to `SubvolumePath`). 6, 6.2 to 6.4:
  cgroups and contained trees.
- **Departs:** 2: `Io::new`
  asserts `skein_io::valid`, but `skein-fake-peers/src/face.rs`'s `valid`
  and `skein-oauth-accounts/src/limits.rs`'s `io_capacity_bound` rebuild
  part of it by hand, without the operation table's ceiling (5): from
  2^24 sockets they pass what it refuses, and the fake's `Io::new` panics;
  no reason is recorded. 5:
  `skein-io/src/file_layer.rs` holds its read and directory buffers in `Vec`,
  under a module-wide expect, against
  `skein/docs/foundation/programming-model.md`, 10.3, because that item did
  not ask it of them, and a scan keeps its page sorted with `insert` and `pop`,
  which `List` lacks.
