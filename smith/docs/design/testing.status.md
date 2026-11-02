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
  - 2.3: the refresh story expects a refresh whatever its record's
    earliest refresh time, so it fails live on a fresh record (the
    after-landing run at main `df41529d3ee1`: the earliest refresh time
    2336 s ahead, the generation unchanged), and is named for the
    refresh alone: it does not choose its case from the record's facts,
    wait for a near earliest refresh time, report the case it checked,
    or fail by name on a record without a refresh token.
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (2.2).
