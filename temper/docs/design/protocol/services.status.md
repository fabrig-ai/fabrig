# services: status

2026-11-12. Where the tree's code lags or departs from [services.md](services.md).

- **Not yet:** 2 and 3: no `temper-service-core`; its items are `temper-service`'s modules.
- **Not yet:** 2, 4 to 6, 8 and 9: one crate, `temper-service`, holds both halves and their composition, its worker's half its `worker` feature, on by default and off in `temper-engine`'s own build; `temper-worker`'s closure holds the engine's domain, `temper-engine-smith` and `temper-box-machine`, each an exception of `tests/boundary/tests/binaries.rs`'s worker row.
- **Not yet:** 7: `temper-service-shell`'s host drives `temper-service`'s `Service` and formats every half's notes, stops and endings, on the engine's forge crates; `temper-engine-shell` starts every shape, the worker's half given by `serve::WorkerHalf`.
