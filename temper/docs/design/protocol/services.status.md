# services: status

2026-11-12. Where the tree's code lags or departs from [services.md](services.md).

- **Not yet:** 3: the process's credit sizing and the agent's declaration are `temper-service`'s (`system.rs`; `worker_settings::Agent`, which the engine's start reads) [in flight: change 1514].
- **Not yet:** 5: the worker's half is `temper-service`'s `worker` feature, not the crate `temper-worker-service`, and `temper-worker-shell` names `temper-service` [in flight: change 1522].
- **Not yet:** 2, 4, 6, 8 and 9: one crate, `temper-service`, holds the engine's half and the composition, its worker's half its `worker` feature, on by default and off in `temper-engine`'s own build; `temper-worker`'s closure holds the engine's domain, `temper-engine-smith` and `temper-box-machine`, each an exception of `tests/boundary/tests/binaries.rs`'s worker row [in flight: change 1523].
- **Not yet:** 7: `temper-engine-shell` formats the joined notes, stops and endings, the worker's among them, since the halves share one vocabulary [in flight: change 1523].
- **Not yet:** 2, *Readers depend on what they use*: crates and worlds using one half's items alone depend on `temper-service` [in flight: change 1524].
