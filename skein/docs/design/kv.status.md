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
  - 4: a failed create or directory sync on a rollover ends the store as a
    failed start does: `skein-kv/src/store.rs`'s `Creating` and `Announcing`
    close it with no event, and leave its queued commits unanswered, where
    this section answers them `Failed` and recovers from the disk; no reason
    is recorded.
  - 3, 7, 9: a restart's transient heap does not grow with a file's
    length, but it is more than the read and the frame or snapshot chunk
    these sections say, and than the chunk plus a frame the user's
    request of 11-03 asked for: the file layer holds a read's buffer and
    its copy, `ReadingLog`'s `bytes` and `snapshot::Reader` copy each read
    again to carry a frame or chunk across reads, and a chunk's rows are
    decoded beside that. `skein/tests/kv/tests/suite/recovery.rs` measures
    the whole world, so its bounds are `3 × chunk + commit` over the log
    and `4 × chunk` over the snapshot, each plus its `RECOVERY_ALLOWANCE`.
