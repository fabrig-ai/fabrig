# components: status

2026-11-11. Where the tree's code lags or departs from [components.md](components.md).

- **Not yet:** the whole document: the four tables' binaries depend on `temper-version` and their crates, `temper-box-shell`, `temper-engine-shell`, `temper-worker-shell` and `temper-agent-shell`, don't: today the crates do, and `temper-version` depends on `skein-shell` for `INTERNAL_COMMIT`, which `temper-shell` doesn't hold yet.
- **Not yet:** the whole document: `bin/temper-client` and `crates/temper-client-shell`: today `crates/temper-client` is both, linking `temper-version`.
