# testing: status

2026-11-14. Where smith's code lags or departs from [testing.md](testing.md).

- **Not yet:**
  - 2.1: only the local-process world sweeps the simulator's faults
    (`tests/local-process/tests/suite/fuzzy/world.rs`), and no world
    asserts that each fault fell.
  - 2.2: the real loop runs one story (`tests/real`,
    `a_chat_ends_with_a_commit_in_place`): no real signal is sent, and
    the local host's simulated scenarios do not run there.
  - 2.3: the live stories choose their account with
    `-c endpoints.<name>.account=`, since `--account` is not built
    (`shell/commands.status.md`, Not yet, the whole document).
  - 2.3: whether the refresh story passes live is unknown: main holds
    it as 2.3 has it from `c2922ded2b10`, and no after-landing run of a
    main holding it is read. The last run read, at main
    `df41529d3ee1`, passed every other story it runs, its Claude Code
    release unknown.
  - 5: the host world's common parent referee and the inline world's
    check of an answer panic on a difference, and their tests are
    `#[should_panic]`. [in flight: change 3252]
  - 5: the protocol world's reading of a native request panics on its
    cache markers and breakpoints, and its tests are `#[should_panic]`.
    [in flight: change 3253]
  - 5: the session world's window referee panics on a difference, the
    local-process world's person panics on a terminal write that fails
    where its script doesn't expect it, and their tests are
    `#[should_panic]`. [in flight: change 3254]
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (2.2).
- **Departs:**
  - 2: domain worlds use domains and fakes only, never protocol crates,
    but `tests/agent/src/backend.rs` uses `smith-protocol-llm`,
    `tests/agent/tests/suite/memory_returns.rs` derives its limits
    through `smith-agent-service`'s profile,
    `tests/run/tests/suite/checks.rs` runs `smith-machine-world`, and
    `tests/host/src/inline.rs` runs `smith-machine-world` with
    `smith-protocol-machine`'s types. No reason is recorded, and no debt
    item names them (section 9).
