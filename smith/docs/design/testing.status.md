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
  - 2.3: three `live-smith` stories fail live (the after-landing run at
    main `46e6e70d9b17`, Claude Code 2.1.312): the Claude story requires
    `finish` among a result-contract run's offered tools; the chat
    crossing's prompt names the directory, and its Claude leg once ended
    with the spawned agent killed by signal 9 before its run started,
    cause unknown; the refresh story's generation does not rise, though
    its lead resolves as 400 days, cause not found in code. The live
    report keeps no record refresh facts, no agent failure line, no
    unknown notice's kind and no missing file found elsewhere; no
    offline story crosses a chat Direct, program, Direct through the
    product or refreshes on a raised lead.
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (2.2).
