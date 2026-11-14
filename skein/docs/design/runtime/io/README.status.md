# io: status

2026-11-14. Where skein's code lags or departs from [README.md](README.md).

Section 4 has no known gap in the requested hangup event or its distinct
`ServiceSignal::Hangup` name.

`processes.md` has no known gap in `SpawnAlone` closing and reaping the child
without killing its group, or `Way::Null` giving a child `/dev/null`
descriptors without parent pipe entities. Process rows cover closure
before and after exit and mixed null, given and pipe descriptors.

- **Not yet:** 5: the step tests of a stale token going up and of a send past the room or with none granted, `#[should_panic]`, go with `temper/docs/design/draft/panics.execution.md`, P8.
- **Departs:** 2: `Io::new`
  asserts `skein_io::valid`, but `skein-fake-peers/src/face.rs`'s `valid`
  and `skein-oauth-accounts/src/limits.rs`'s `io_capacity_bound` rebuild
  part of it by hand, without the operation table's ceiling (5): from
  2^24 sockets they pass what it refuses, and the fake's `Io::new` panics;
  no reason is recorded. 
  the file-buffer departure remains `files.status.md` (Departs, the whole document).  The file-handler departure remains `files.status.md` (Departs, the whole document).  that plan's request of 2026-11-06
  has each of its items end with fewer lines than it started, the file layer's
  states apart; moving every reader onto `file_layer::step` and removing the
  thin forms ended 133 lines of Rust up, 53 in code and 80 in tests, since each
  reader gained a kept `file_layer::Limits` and an `Env`.
