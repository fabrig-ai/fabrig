# io: status

2026-10-31. Where skein's code lags or departs from [io.md](io.md).

- **Built:** 1, 2: limits checked by `valid`; interrupted transfers absorbed by the backend. 2.1: the stage, `stage::{up, down}`, which skein's hosts call; those that must see each call keep their own loops (2.1's last point). 5.3: one reopen for a private-load replace race. 3.2, 6.3: held listeners passed to children, on Linux, macOS and the simulator, and retained for another spawn.
- **Not yet:** 4: DNS/datagrams. 5: tree metadata, filesystem/subvolumes, remaining roots by token and file-reading streams. 6, 6.2 to 6.4: contained trees. 10: stall faults, uninterruptible operations, syscall filters and tree terminals.
- **Departs:** 2: skein-ui-text's terminal hands its io each completion before it drains the ready list, and calls `down` from its event handlers, between io's own calls.
