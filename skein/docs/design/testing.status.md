# testing: status

2026-11-03. Where skein's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  6, 10: of the portable crates, fab's gate list (`.config/gate/wasm32.txt`) holds
  `skein-playground` alone, so lib, io, the simulator, the shell's portable face, the
  simulated world, the fake machine, the fakes a world hosts, the key-value store, HTTP,
  JSON, the channel, the codec and TLS are not in the gate's wasm32 clippy. 7:
  `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
- **Departs:** 6: `crates/skein-fail-stop`, `crates/skein-fake-forge-protocol`
  (a memory suite) and `crates/skein-forge` have a `tests/` directory,
  where 6 puts what drives a crate from outside under `tests/`; no reason
  is recorded.
