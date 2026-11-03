# testing: status

2026-11-03. Where skein's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  6, 10: `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers`
  are not in fab's gate list (`.config/gate/wasm32.txt`); their wasm32
  clippy is unmeasured with rustls's `std` native only (`tls.md`, 4),
  which takes rustls's `E0599` at `src/ticketer.rs:56` out of their
  tree. 7: `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
- **Departs:** 6: `crates/skein-fail-stop`, `crates/skein-fake-forge-protocol`
  (a memory suite) and `crates/skein-forge` have a `tests/` directory,
  where 6 puts what drives a crate from outside under `tests/`; no reason
  is recorded.
