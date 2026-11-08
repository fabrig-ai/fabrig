# testing: status

2026-11-08. Where skein's code lags or departs from [testing.md](testing.md).

Unlisted sections have no known gap. Section 6's test-only fork kit and
its documented process boundary are present, as are the row runner's
admission, continuation, failure diagnostics and expected-panic helper.
Section 9 carries deferred cost work, including further folds,
allocation layout and the native fork floor.

Section 6's full-screen renderer and terminal crates, golden frames and
simulated-terminal world are present.

Section 5's one-process starvation path is present: `Starved::pass` and
`pass_starved` unwind with `StarvedPass`, helper processes use
`Starved::exit`, and real-loop cleanup precedes the verdict. The deadline
and real-loop tests cover the mode and children remove both runner
variables when checking failure. No implementation gap is known in this
path. Section 9 leaves caller adoption and suite CPU measurement separate.

- **Not yet:** 4: the fake machine's entry inodes, groups, modes past
  `0o777`, times and attributes, its subvolume and filesystem answers,
  and the programs for trees.
  7: `live-skein` has no CPU budget or wall hard limit in
  `tools/tiers/budgets.toml`.
  7: the focused cost repairs' at-least-6 CPU s saving is unproved.
  `skein/docs/development/skein-limits-heavy.md` estimates 5.311441 CPU s
  saved from selected instruction ratios, short of that request's aim.
  The process-world row has no complete instruction pair, other tests
  have unknown individual counts, and the row ratios come from different
  baselines. Shared-helper measurements in
  `skein/docs/development/world-costs.md` establish no suite total either.
  The suite's total is the merge gate's: its full reads of 11-08 put it
  at 28.5 CPU s (the user's word of 11-08, which set the 40 CPU s
  budget), and no measurement in this repository records it.
  7: the skein-limits-rows request asked a margin of at least 2.5 CPU s
  below a 30 CPU s limit; the gate's 28.5 CPU s leaves 1.5 below that,
  and 11.5 below the 40 CPU s budget that replaced it, the user's of
  11-08. The rows' own repairs establish no margin:
  Callgrind pairs for the two checking-signature
  paths fell from 28,168,371 to 17,682,754 instructions in the forge
  machine world's measured hand test, 126,199,178 to 58,107,517 in
  `skein-fake-forge-github`'s, and 61,539,809 to 51,804,727 in
  `github-conformance`'s; folds removed 397 existing test processes and
  added eight row-kit tests of their own. These floor estimates and
  individual instruction counts establish no whole-suite saving.
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
