# testing-tiers: status

2026-11-25. Where smith's code lags or departs from [testing-tiers.md](testing-tiers.md).

- **Not yet:**
  - 1: only the local-process world sweeps the simulator's faults
    (`tests/local-process/tests/suite/fuzzy/world.rs`), and no world
    asserts that each fault fell.
  - 2: the real loop runs one story (`tests/real`,
    `a_chat_ends_with_a_commit_in_place`): no real signal is sent, and
    the local host's simulated scenarios do not run there.
  - 3: the live stories choose their account with
    `-c endpoints.<name>.account=`, since `--account` is not built
    (`shell/commands.status.md`, Not yet, the whole document).
  - 3: whether the refresh story passes live is unknown: main holds
    it as section 3 has it from `c2922ded2b10`, and no after-landing run of a
    main holding it is read. The last run read, at main
    `df41529d3ee1`, passed every other story it runs, its Claude Code
    release unknown.
  - 3: a story's `PATH` is `/usr/bin` and `/bin` alone, without the
    directory of the test process's rg, so on macOS, whose pinned rg
    lies elsewhere, smith refuses for want of rg.
    [in flight: change 17920]
  - 5: the chat store's layout cases run over a real directory in
    `smith-local-protocol::suite`, not on the fake disk.
    [in flight: change 13947]
  - 5: smith-local-shell's tests and `smith::step`'s live trace test
    name `/bin/true` as `programs.claude_code`, a path macOS lacks,
    not a stand-in in their scratch.
    [in flight: change 17919]
- **Departs:**
  - the whole document: domain worlds use domains and fakes only, never protocol crates,
    but `tests/agent/src/backend.rs` uses `smith-protocol-llm`,
    `tests/agent/tests/suite/memory_returns.rs` derives its limits
    through `smith-agent-service`'s profile,
    `tests/run/tests/suite/checks.rs` runs `smith-machine-world`, and
    `tests/host/src/inline.rs` runs `smith-machine-world` with
    `smith-protocol-machine`'s types. No reason is recorded, and no debt
    item names them (`testing.md`, section 9).
