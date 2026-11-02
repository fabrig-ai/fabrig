# shell: status

2026-11-02. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 9: no macOS runner checks the members that reach rustls or
  runs Darwin's code.
- **Departs:** 7.4: the readiness backend makes an interrupted `Socket`,
  `Accept` or `Connect` again, as it does every operation but `Open` and
  `Close` (`readiness.rs`, `attempt`), where the ring completes them
  `Other(EINTR)`; a `Connect` made again calls `connect` a second time.
  The rule is open (section 10), and no reason is recorded.
