# kernel: status

2026-11-21. Where skein's code lags or departs from [kernel.md](kernel.md).

Section 6.3 has no known gap in reading a requested `SIGHUP` as a
shutdown, with the sender on Linux and none on macOS.

Section 6.2 has no known gap in `Way::Null`: Linux and macOS install
`/dev/null` read and write through spawn file actions, and the simulator
gives the descriptor an immediate read end and accepts its writes.

- **Not yet:** 6.1, 6.4: tree metadata records, including `StatEntry`,
  `ReadLink`, `MakeLink`, `SetMode`, `SetOwner`, attributes, times and owners.
- **Not yet:** 6.5: filesystem status and sync, and btrfs subvolume records.
- **Not yet:** 6.2, 6.6: cgroups and contained spawn, including the helper's
  ids and containment.
- **Not yet:** 7: the invariants of the missing metadata, filesystem and
  contained-spawn records.

