# layout: status

2026-11-24. Where the tree's code lags or departs from [layout.md](layout.md).

- **Departs:** 1: step tests belong in `src/tests.rs` and `src/tests/`, but
  `skein-shell/src/listener_calls.rs` loads sibling `listener_tests.rs`
  through `#[path]`, as other crates load `*_tests.rs`, and
  `skein-llm/src/translate.rs` and other source files hold inline test
  modules; no reason is recorded (the code's side is
  `skein/docs/development/debt/D027-step-test-layouts.md`).
  1: `smith/benchmarks/tasks/repository/shell-cwd/grader`, outside the
  workspace, reads `skein_llm_world`, because its first grade takes that world
  from the frozen workspace's own git source.
  1: the `debt-24` request of 2026-11-05 has its items end with fewer test
  lines unless a story is newly held, but they stand above that plan's start:
  forge-connection's crate tests, with the `fixtures.rs` they share, 8,140 below
  it, its machine world with the forge kit 8,692 above, and `tests/boundary`
  273 above; no reason is recorded.
