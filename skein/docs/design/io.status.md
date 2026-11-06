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
  `skein/docs/foundation/programming-model.md`, 10.3, because no request has
  asked it of them, and a scan keeps its page sorted with `insert` and `pop`,
  which `List` lacks. 5: `skein-io/src/file_layer/store.rs` and `private.rs`
  hold each lifecycle in one enum, and a completion's cells each have their
  handler, but `store.rs`'s `failed_operation` and `stopped` and `private.rs`'s
  `stopped` keep a failure's and a stop's cells as arms of one `match`, against
  `skein/docs/foundation/programming-model.md`, 5.4's one small handler per
  cell, which the `debt-28` plan's tasks asked; no reason is recorded, and that
  plan's item is deleted. Seen by reading. 5: that plan's request of 2026-11-06
  has each of its items end with fewer lines than it started, the file layer's
  states apart; moving every reader onto `file_layer::step` and removing the
  thin forms ended 133 lines of Rust up, 53 in code and 80 in tests, since each
  reader gained a kept `file_layer::Limits` and an `Env`.
