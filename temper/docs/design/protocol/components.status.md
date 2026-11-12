# components: status

2026-11-12. Where the tree's code lags or departs from [components.md](components.md).

- **Not yet:** the whole document: the four tables' binaries depend on `temper-version` and their crates, `temper-box-shell`, `temper-engine-shell`, `temper-worker-shell` and `temper-agent-shell`, don't: today the crates do, and `temper-version` depends on `skein-shell` for `INTERNAL_COMMIT`, which `temper-shell` doesn't hold yet [in flight: change 586].
- **Not yet:** the whole document: `bin/temper-client` and `crates/temper-client-shell`: today `crates/temper-client` is both, linking `temper-version` [in flight: change 587].
- **Not yet:** `crates/temper-service-core`, `crates/temper-engine-service`, `crates/temper-worker-service` and `crates/temper-service`: one crate, `temper-service`, holds both halves and their composition, the worker's half its `worker` feature (`services.status.md`).
- **Not yet:** `crates/temper-service-shell`, `crates/temper-engine-shell`, `crates/temper-worker-shell` and `crates/temper-box-shell`: the service shell's host formats every half's lines on the engine's forge crates, and the engine's shell starts every shape, given `serve::WorkerHalf` (`services.status.md`).
