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
  - 2.3: three `live-smith` stories were red in the after-landing run at
    main `da9027dbf5ae` with no cause found in code: the refresh story's
    generation does not rise, though its lead resolves as 400 days; the
    chat crossing fails a leg (by elimination its first, ChatGPT leg);
    and the second run's resume fails live within about 6 s while its
    offline twins pass. The run's other three reds were repaired in code
    (protocol/programs.status.md, 8); whether all six pass live on a main
    holding the repairs is unknown.
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (2.2).
