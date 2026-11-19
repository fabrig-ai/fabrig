# macos: status

2026-11-19. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:** 1, 4 to 6 and 8: `.github/workflows/macos.yml` runs on a schedule as one job, compiles every target and runs the one story alone: no dispatch inputs or commit check, no parts or `plan` and `report` jobs, no part files, `junit` report or `build.PART` case, one cache for the whole target, no Chrome for Testing and no `live` job; `tools/macos/macos.py` is not written [in flight: change 9695].
- **Not yet:** 7: `.config/nextest.toml` has no overrides for `cfg(target_os = "macos")` and no JUnit path for the nextest runs' profiles [in flight: change 9686].
- **Not yet:** 9: `.forgejo/workflows/` has no `tier-macos-e2e.yml` or `tier-macos-suites.yml` [in flight: change 9688].
- **Not yet:** 2: temper's one story on macOS has not run green
  on the runner, so `../draft/macos.md`'s acceptance stands open.
