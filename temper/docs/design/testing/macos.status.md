# macos: status

2026-11-19. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:** 2: temper's one story on macOS has not run green
  on the runner, so `../draft/macos.md`'s acceptance stands open.
- **Not yet:** 6: the `browser` job gives the wasm C build no compiler but
  the runner's Apple `clang`, which has no `wasm32` target, so ring's build
  script fails and the part builds nothing [in flight: change 9934].
