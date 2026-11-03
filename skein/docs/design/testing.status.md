# testing: status

2026-11-03. Where skein's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  6, 10: these crates fail the gate's wasm32 clippy, so fab's gate list
  (`.config/gate/wasm32.txt`) leaves them out; each at its first error:
  `skein-tls`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
  `skein-fake-forge-github`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`;
  `skein-fake-peers`: `E0599` at `rustls 0.23.41 src/ticketer.rs:56`.
  7: `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
- **Departs:** 6: `crates/skein-fail-stop`, `crates/skein-fake-forge-protocol`
  (a memory suite) and `crates/skein-forge` have a `tests/` directory,
  where 6 puts what drives a crate from outside under `tests/`; no reason
  is recorded.
