# places: status

2026-11-20. Where temper's code lags or departs from [places.md](places.md).

- **Not yet:**
  - 3: the install's check is still a `sha256sum --check` process, not the tool's own check naming each failed file. [in flight: change 10611]
  - 4 and 8: the archive still runs GNU tar's `--anchored --no-wildcards --exclude`, not options both tars take with what a start pins set aside at a save and removed at a restore. [in flight: change 10612]

Unlisted sections have no known gap.
