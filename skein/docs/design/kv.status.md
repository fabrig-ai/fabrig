# kv: status

2026-11-04. Where skein's code lags or departs from [kv.md](kv.md).

- **Not yet:** 9: the model-map referee across simulated-disk commits,
  loads, snapshots and crash recovery, commits racing every snapshot
  chunk, and the recovered value check over the real ring. In
  `skein/tests/kv/tests/suite/fuzzy/map.rs`,
  `generated_commits_and_pages_match_the_model` uses `Store::memory`;
  `store.rs` has commit and snapshot crash cuts, and `disk.rs`'s cuts
  read a model back by `Get`, never a load, with no commit racing them;
  `commits_race_fuzzy_snapshot_chunks` races a commit only after the
  first chunk; `real.rs`'s `commit_and_recover` checks the reopened number,
  without reading the value; no reason is recorded.
- **Departs:**
  - 2, 3, 4: for the store's requests, recovery and commits,
    `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
    state enum per lifecycle, each state holding exactly its data, no
    lifecycle in flags or `Option`s beside it, exhaustive matches and one
    handler per cell; `skein-kv/src/store.rs`'s `Store` keeps recovery and
    snapshot flags and `Option`s beside `phase`, its reading phases keep a
    failure found part way as data (`failure`, `replayed`), it matches
    phase and event tuples with catch-all arms, and it has five
    module-wide subset exceptions; no reason is recorded (the code's side
    is `skein/docs/development/debt/D017-kv-store-states.md`).
  - 3, 7, 9: a restart's transient heap does not grow with a file's
    length, but it is more than the read and the frame or snapshot chunk
    these sections say, and than the chunk plus a frame the user's
    request of 11-03 asked for: the file layer holds a read's buffer and
    its copy, `ReadingLog`'s `bytes` and `snapshot::Reader` copy each read
    again to carry a frame or chunk across reads, and a chunk's rows are
    decoded beside that. `skein/tests/kv/tests/suite/recovery.rs` measures
    the whole world, so its bounds are `3 × chunk + commit` over the log
    and `4 × chunk` over the snapshot, each plus its `RECOVERY_ALLOWANCE`.
