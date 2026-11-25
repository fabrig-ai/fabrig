# macos: status

2026-11-25. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:**
  - 2: temper's one story on macOS has not run green
    on the runner, so `../draft/macos.md`'s acceptance stands open.
  - 3: temper-box's `domain` test binary does not build on macOS: it
    reaches temper-box-shell's `testing::serve`, offered on Linux alone
    though what it wraps runs on both, and temper-box-shell names a
    dependency, `skein-fail-stop`, that macOS's build finds unused
    [in flight: change 17936].
