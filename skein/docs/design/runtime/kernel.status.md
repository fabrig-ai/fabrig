# kernel: status

2026-11-13. Where skein's code lags or departs from [kernel.md](kernel.md).

Section 6.3 has no known gap in reading a requested `SIGHUP` as a
shutdown, with the sender on Linux and none on macOS.

Section 6.2 has no known gap in `Way::Null`: Linux and macOS install
`/dev/null` read and write through spawn file actions, and the simulator
gives the descriptor an immediate read end and accepts its writes.

- **Not yet:** 6.1, 6.4: a tree's metadata: `StatEntry`, `ReadLink`,
  `MakeLink`, `SetMode`, `SetOwner`, the attribute records, and the whole
  metadata, times and owners among it. 6.5: `FilesystemStatus`,
  `SyncFilesystem` and btrfs's subvolume records. 6.2, 6.6: cgroups and
  contained spawn, the helper among them. 7: the invariants of those
  records. 8: their conformance scenarios.
