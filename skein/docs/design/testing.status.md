# testing: status

2026-11-03. Where skein's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  6, 10: the gate's wasm32 list (`.config/gate/wasm32.txt`) leaves out
  `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers`, and no
  listed package reaches them on wasm32, so the gate's wasm32 clippy
  lints none of them; run outside the gate, their wasm32 clippy passes,
  warnings denied.
  7: `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
- **Departs:** 6: `crates/skein-fail-stop`, `crates/skein-fake-forge-protocol`
  (a memory suite) and `crates/skein-forge` have a `tests/` directory,
  where 6 puts what drives a crate from outside under `tests/`; no reason
  is recorded.
  6: step tests belong in `src/tests.rs` and `src/tests/`, but
  `skein-shell/src/listener_calls.rs` loads sibling `listener_tests.rs`
  through `#[path]`, as other crates load `*_tests.rs`, and
  `skein-llm/src/translate.rs` and other source files hold inline test
  modules; no reason is recorded (the code's side is
  `skein/docs/development/debt/D027-step-test-layouts.md`). 6: shared
  harnesses belong in `testing/`, but worlds under `tests/` serve as
  kits: `skein/tests/llm-connection/Cargo.toml` depends on
  `skein-llm-world` and `skein-tls-world`, and `skein-llm/Cargo.toml`
  uses `skein-json-world` as a dev-dependency; no reason is recorded
  (the code's side is
  `skein/docs/development/debt/D028-worlds-as-kits.md`).
