# shell: status

2026-11-03. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 7.4: the readiness backend makes an interrupted `Socket`,
  `Accept`, `Connect` or `Open` again: `linux`'s and `darwin`'s
  `socket`, `accept`, `connect` and `files::open` retry `EINTR`, and
  `attempt` makes the first three again (`readiness.rs`); a `Connect`
  made again calls `connect` a second time, which a socket whose
  connection is under way answers `EALREADY`. 9: no macOS runner checks
  the members that reach rustls or runs Darwin's code.
