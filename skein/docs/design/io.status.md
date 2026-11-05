# io: status

2026-11-05. Where skein's code lags or departs from [io.md](io.md).

- **Not yet:** 5: a tree's metadata, the filesystem's status and sync, and
  btrfs's subvolumes (`StatEntry` to `SubvolumePath`). 6, 6.2 to 6.4:
  cgroups and contained trees.
- **Departs:** 5: `FileIo`'s readers call the thin forms, not `file_layer::step`, because the
  move is wide (216 sites, 149 outside `skein-io`'s tests), some readers keep no clock, and
  `hearth-service-builder/src/lib.rs` is being migrated apart. `file_layer/store.rs`'s `Store`
  and `file_layer/private.rs`'s `Opening` keep phases in flags and `Option`s, against
  `skein/docs/foundation/programming-model.md`, 5.4. (The code's side is
  `skein/docs/development/debt/D049-file-layer-thin-forms.md`.)
  2: `Io::new`
  asserts `skein_io::valid`, but `skein-fake-peers/src/face.rs`'s `valid`
  and `skein-oauth-accounts/src/limits.rs`'s `io_capacity_bound` rebuild
  part of it by hand, without the operation table's ceiling (5): from
  2^24 sockets they pass what it refuses, and the fake's `Io::new` panics;
  no reason is recorded.
