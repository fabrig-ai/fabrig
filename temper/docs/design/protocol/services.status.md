# services: status

2026-11-12. Where the tree's code lags or departs from [services.md](services.md).

- **Not yet:** 3: the process and the agent's declaration are `temper-service`'s (`system.rs`; `worker_settings::Agent`, which the engine's start reads).
- **Not yet:** 2, 4 to 6, 8 and 9: one crate, `temper-service`, holds both halves and their composition, its worker's half its `worker` feature, on by default and off in `temper-engine`'s own build; `temper-worker`'s closure holds the engine's domain, `temper-engine-smith` and `temper-box-machine`, each an exception of `tests/boundary/tests/binaries.rs`'s worker row.
- **Not yet:** 7: each half's notes, stops and endings are formatted by `temper-service-shell` (`host.rs`), which names `temper-service`, and each binary's table supplies them.
