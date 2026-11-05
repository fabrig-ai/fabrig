# testing: status

2026-11-05. Where skein's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  7: `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
- **Departs:** 5, 10: `skein/testing/skein-world/src/lib.rs` fences six
  items `cfg(unix)`, though these sections have the crate native, on
  Linux and macOS, so off unix it fails to build however it is fenced; no
  reason is recorded.
  6: step tests belong in `src/tests.rs` and `src/tests/`, but
  `skein-shell/src/listener_calls.rs` loads sibling `listener_tests.rs`
  through `#[path]`, as other crates load `*_tests.rs`, and
  `skein-llm/src/translate.rs` and other source files hold inline test
  modules; no reason is recorded (the code's side is
  `skein/docs/development/debt/D027-step-test-layouts.md`). 6: `skein-scratch`
  is the tests' scratch directory, removed when dropped, but
  `skein/testing/skein-codegen/tests/suite/patch_run.rs` and
  `patch_generated.rs` make theirs by hand and remove them after their
  assertions, so a failing test leaves one behind, and a reused pid then
  fails on it; no reason is recorded.
