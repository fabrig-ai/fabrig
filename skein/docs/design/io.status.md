# io: status

2026-11-05. Where skein's code lags or departs from [io.md](io.md).

- **Not yet:** 5: a tree's metadata, the filesystem's status and sync, and
  btrfs's subvolumes (`StatEntry` to `SubvolumePath`). 6, 6.2 to 6.4:
  cgroups and contained trees.
- **Departs:** 2: `Io::new`
  asserts `skein_io::valid`, but `skein-fake-peers/src/face.rs`'s `valid`
  and `skein-oauth-accounts/src/limits.rs`'s `io_capacity_bound` rebuild
  part of it by hand, without the operation table's ceiling (5): from
  2^24 sockets they pass what it refuses, and the fake's `Io::new` panics;
  no reason is recorded. 5: every reader outside `skein-io`'s tests calls
  the thin forms, where the request of 2026-11-05 had `FileIo` take `Env`
  and `Limits` as its siblings do: 152 sites across the five trees call
  them, and only those tests call `FileIo::with_limits` and
  `file_layer::step`. The move is wide, some readers keep no clock, and
  that request keeps out of `hearth-service-builder/src/lib.rs`, which
  calls them. 5: `skein-io/src/file_layer/store.rs`'s `Store` keeps 9
  `Option`s and the `temp_exists` and `committed` flags beside its 20
  phases, and `skein-io/src/file_layer/private.rs`'s `Opening` 3 `Option`s
  and `made` beside its 9, against
  `skein/docs/foundation/programming-model.md`, 5.4, because that request's
  debt item names `FileIo`, not them, and it allowed a little more code. (The
  code's side of both is
  `skein/docs/development/debt/D049-file-layer-thin-forms.md`.) 5:
  `skein-io/src/file_layer.rs` holds its read and directory buffers in `Vec`,
  under a module-wide expect, against
  `skein/docs/foundation/programming-model.md`, 10.3, because that item did
  not ask it of them, and a scan keeps its page sorted with `insert` and `pop`,
  which `List` lacks.
