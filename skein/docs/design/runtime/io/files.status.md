# files: status

2026-11-13. Where the tree's code lags or departs from [files.md](files.md).

- **Not yet:** the whole document: a tree's metadata, the filesystem's status and sync, and
  btrfs's subvolumes (`StatEntry` to `SubvolumePath`), but `HardLink`. 
  contained trees remain `processes.status.md` (Not yet, the whole document).
- **Departs:**  the io-validity gap remains `README.status.md`, 2. the whole document:
  `skein-io/src/file_layer.rs` holds its read and directory buffers in `Vec`,
  under a module-wide expect, against
  `skein/docs/foundation/programming-model.md`, 10.3, because no request has
  asked it of them, and a scan keeps its page sorted with `insert` and `pop`,
  which `List` lacks. the whole document: `skein-io/src/file_layer/store.rs` and `private.rs`
  hold each lifecycle in one enum, and a completion's cells each have their
  handler, but `store.rs`'s `failed_operation` and `stopped` and `private.rs`'s
  `stopped` keep a failure's and a stop's cells as arms of one `match`, against
  `skein/docs/foundation/programming-model.md`, 5.4's one small handler per
  cell, which the `debt-28` plan's tasks asked; no reason is recorded, and that
  plan's item is deleted. Seen by reading. the whole document: that plan's request of 2026-11-06
  has each of its items end with fewer lines than it started, the file layer's
  states apart; moving every reader onto `file_layer::step` and removing the
  thin forms ended 133 lines of Rust up, 53 in code and 80 in tests, since each
  reader gained a kept `file_layer::Limits` and an `Env`.
