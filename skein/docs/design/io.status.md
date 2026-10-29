# io: status

2026-10-29. Where skein's code lags or departs from [io.md](io.md).

- **Built:** 1, 2: limits checked by `valid`; interrupted transfers absorbed by the backend. 5.3: one reopen for a private-load replace race. 3.2, 6.3: held listeners passed to children, on Linux, macOS and the simulator, and retained for another spawn.
- **Not yet:** 4: DNS/datagrams. 5: tree metadata, filesystem/subvolumes, remaining roots by token and file-reading streams. 6, 6.2 to 6.4: contained trees. 10: stall faults, uninterruptible operations, syscall filters and tree terminals.
