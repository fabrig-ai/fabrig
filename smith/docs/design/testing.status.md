# testing: status

2026-11-23. Where smith's code lags or departs from [testing.md](testing.md).

- **Not yet:**
  - 2.3, 2.5: `smith::end_to_end` still holds `smith exec`'s result-file
    and private-chat variants, which the local host's simulated world
    does not run [in flight: change 13996]
  - 2.3, 2.5: the local host's simulated world runs no `program`
    account: no trace of its agent, no live view, no chat crossing
    between backends [in flight: change 13997]
  - 2.3, 2.5: a `program` account's rechecks and the environment its
    program gets are tested only end to end [in flight: change 13998]
  - 2.3, 2.5: a sign-in ending in a checked commit, the standard
    profile's large write committed, an unreadable head and a grant's
    refresh at its lead or earliest time are tested only end to end
    [in flight: change 13999]
  - 2.3, 2.5: `smith exec`'s parked line, an empty prompt on standard
    input, a missing credential and the chat tree's failure are tested
    only end to end [in flight: change 14000]
  - 2.3, 2.5: `smith check`'s settings refusals, budget ceiling and
    program rows are tested only end to end [in flight: change 14001]
  - 2.3, 2.5: `smith login`'s and `smith logout`'s words, logout's
    sources and a program's refused sign-out are tested only end to end
    [in flight: change 14002]
  - 2.3, 2.5: `smith-agent`'s transport refusal, a program run's offered
    tools and a writing sub-agent's refusal are tested only end to end
    [in flight: change 14003]
  - 2.3: the fake program's lookup reads the file's metadata in the
    decision, and the live judge's acceptance runs only end to end
    [in flight: change 14004]
  - 6: smith's inspection adapter, `smith-inspect`, and its worlds'
    calls of the shared checks.
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
