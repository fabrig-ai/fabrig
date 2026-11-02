# testing: status

2026-11-02. Where smith's code lags or departs from [testing.md](testing.md).

- **Not yet:**
  - 2.1: only the local-process world sweeps the simulator's faults
    (`tests/local-process/tests/suite/fuzzy/world.rs`), and no world
    asserts that each fault fell.
  - 2.2: the real loop runs one story (`tests/real`,
    `a_chat_ends_with_a_commit_in_place`): no real signal is sent, and
    the local host's simulated scenarios do not run there.
  - 2.3: the live stories choose their account with
    `-c endpoints.<name>.account=`, since `--account` is not built
    (shell.status.md).
  - 2.3: whether the three `live-smith` stories red at main
    `46e6e70d9b17` (Claude Code 2.1.312) pass on the current main is
    unknown: no after-landing run of it is read. The refresh story's
    generation, equal before and after its run, has no cause found in
    code: offline, an exec on a raised lead refreshes a held record at
    once and keeps one whose earliest refresh time is ahead
    (`credentials.rs` in `tests/end-to-end`; 8).
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (2.2).
