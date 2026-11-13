# components: status

2026-11-13. Where the tree's code lags or departs from [components.md](components.md).

- **Not yet:** `crates/temper-service-core`, `crates/temper-engine-service`, `crates/temper-worker-service` and `crates/temper-service`: one crate, `temper-service`, holds both halves and their composition, the worker's half its `worker` feature (`services.status.md`).
- **Not yet:** `crates/temper-service-shell`, `crates/temper-engine-shell`, `crates/temper-worker-shell` and `crates/temper-box-shell`: the service shell's host formats every half's lines on the engine's forge crates, and the engine's shell starts every shape, given `serve::WorkerHalf` (`services.status.md`).
- **Not yet:** `crates/temper-engine-service-component`, `crates/temper-engine-service-startup`, `crates/temper-engine-service-stock` and `crates/temper-engine-service-vocabulary`: `crates/temper-engine-service` holds all four (`services.status.md`) [in flight: change 2280].
