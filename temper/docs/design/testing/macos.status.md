# macos: status

2026-11-26. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:**
  - 2: temper's one story on macOS has not run green
    on the runner, so `../draft/macos.md`'s acceptance stands open.
  - 7: the helper gives `browser-temper` no global timeout of its own
    on macOS: `tools/macos/global-timeouts.toml` and the copied
    configuration it reaches the run by are not built. [in flight: change 19381]
