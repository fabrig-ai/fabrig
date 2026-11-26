# macos: status

2026-11-26. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:**
  - 2: temper's one story on macOS has not run green
    on the runner, so `../draft/macos.md`'s acceptance stands open.
  - 3: `temper-box-shell`'s `testing::serve` offers the `serve` items its
    tests read on Linux alone, so temper-box's `domain` binary and
    temper-box-shell's `real` binary, which read them on every system,
    don't build on macOS. [in flight: change 18410]
