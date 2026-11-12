# examples: status

2026-11-12. Where skein's code lags or departs from [examples.md](examples.md).

- **Not yet:** 6: the echo's referee reads the echo's address from its
  state (`tests/echo/src/referee.rs`), not from its startup fact; the
  end-to-end kit starts its binary in a cgroup through the shell
  (`start_binary_in_cgroup`), not as an io tree.
  6: a test that fails: the walk's forced settle kills neither the
  descendants it found below the binary's group nor the orphans a
  killed binary leaves after its listing, and reports no process left
  running (`LinuxTree::force_settle`). [in flight: change 1552]
