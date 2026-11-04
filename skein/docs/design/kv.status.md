# kv: status

2026-11-04. Where skein's code lags or departs from [kv.md](kv.md).

- **Not yet:** 9: the model-map referee
  across simulated-disk commits, loads, snapshots and crash recovery,
  commits racing every snapshot chunk, and the recovered value check
  over the real ring. In
  `skein/tests/kv/tests/suite/fuzzy/map.rs`,
  `generated_commits_and_pages_match_the_model` uses `Store::memory`;
  `store.rs` has commit and snapshot crash cuts, but
  `commits_race_fuzzy_snapshot_chunks` races a commit only after the
  first chunk; `real.rs`'s `commit_and_recover` checks the reopened number,
  without reading the value; no reason is recorded.
- **Departs:** 2, 4: for the store's requests and commits,
  `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no
  lifecycle in flags or `Option`s beside it, exhaustive matches and one
  handler per cell; `skein-kv/src/store.rs`'s `Store` keeps recovery and
  snapshot flags and `Option`s beside `phase`, matches phase and event
  tuples with catch-all arms, and has five module-wide subset exceptions;
  no reason is recorded (the code's side is
  `skein/docs/development/debt/D017-kv-store-states.md`).
