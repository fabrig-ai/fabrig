# testing: status

2026-11-20. Where skein's code lags or departs from [README.md](README.md).

Unlisted sections have no known gap. Section 6's test-only fork kit and
its documented process boundary are present, as is the row runner's
admission.
Section 9 carries deferred cost work, including further folds,
allocation layout and the native fork floor.

Section 6's full-screen renderer and terminal crates, golden frames and
simulated-terminal world are present.

Section 5's helper processes use `Starved::exit`, and real-loop cleanup
precedes the verdict. The deadline and real-loop tests cover the
one-process mode and children remove both runner variables when checking
failure. Section 9 leaves caller adoption and suite CPU measurement separate.
A span counts its thread's allocations (`Span::made`, held by
`skein/tests/heap`'s `a_span_counts_the_allocations_its_thread_made`),
and skein-codec's patcher step tests show reuse by rooms and by none
counted around each reused pass, never by an address.

- **Not yet:** 4: the probe's report of a contained tree.
  4: the probe for a reach, `skein_fake_machine::probe`, and the machine honouring a reach.
  [in flight: change 9030]
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
  6: `smith/benchmarks/tasks/repository/shell-cwd/grader`, outside the
  workspace, reads `skein_llm_world`, because its first grade takes that world
  from the frozen workspace's own git source.
  6: the `debt-24` request of 2026-11-05 has its items end with fewer test
  lines unless a story is newly held, but they stand above that plan's start:
  forge-connection's crate tests, with the `fixtures.rs` they share, 8,140 below
  it, its machine world with the forge kit 8,692 above, and `tests/boundary`
  273 above; no reason is recorded.
