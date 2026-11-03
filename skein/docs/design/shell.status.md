# shell: status

2026-11-03. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 7.4: the readiness backend makes an interrupted `Bind`,
  `Listen`, `Shutdown`, `Truncate`, `Sync`, `Stat`, `Rename`, `Remove`,
  `MakeDirectory`, `List`, `Lock`, `Wait`, `Spawn`, `Signal`, `Usage` or
  `Window` again, in its shared part and, for all but `List`, `Spawn`,
  `Signal`, `Usage` and `Window`, in each system's call; the ring's
  `Lock`, which shares Linux's call, is made again too. 9: no macOS
  runner checks the members that reach rustls or runs Darwin's code.
