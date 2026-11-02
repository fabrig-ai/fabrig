# examples: status

2026-11-02. Where skein's code lags or departs from [examples.md](examples.md).

- **Not yet:** 6: the echo's referee reads the echo's address from its
  state (`tests/echo/src/referee.rs`), not from its startup fact; the
  end-to-end kit starts its binary in a cgroup through the shell
  (`start_binary_in_cgroup`), not as an io tree.
